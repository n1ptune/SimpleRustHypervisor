#![allow(unused)]
use spin::Mutex;

use super::gicd::*;
use super::gicr::*;
use super::GicIrqOps;
use crate::arch::coreid;

lazy_static! {
    pub static ref GIC_MAX_LRS: Mutex<u64> = Mutex::new(0);
    pub static ref GIC_MAX_SPI: Mutex<u32> = Mutex::new(0);
}

pub const GIC_MIN_SPI0: u32 = 32;
pub const GIC_MIN_LPI: u32 = 8192;

fn gic_dist_wait_for_rwp() {
    while (gicd_read32(GICD_CTLR) >> 31) & 1 != 0 {
        // Wait for RWP bit to be set
    }
}

// Implement GIC distributor initialization here
fn gic_dist_init() {
    gicd_write32(GICR_CTLR, 0); //  Disable Distributor
    gic_dist_wait_for_rwp();

    let gicd_type = gicd_read32(GICD_TYPER);
    let gic_irqs  = ((gicd_type & 0x1F) + 1) * 32;

    let nr_regs = (gic_irqs + 31) / 32;

    let mut gic_max_spi = GIC_MAX_SPI.lock();
    *gic_max_spi = gic_irqs - 1;

    // irq 0 ~ 31 for SGIs and PPIs
    for i in 1..nr_regs {
        // Set interrupt is Non-secure Group 1
        gicd_write32(gicd_igroupr(i as u64), !0u32);
        
        // Disable all IRQs
        gicd_write32(gicd_icenabler(i as u64), !0u32);
        
        // Clear all active IRQs
        gicd_write32(gicd_icactiver(i as u64), !0u32);
        
        // Clear all pending IRQs
        gicd_write32(gicd_icpendr(i as u64), !0u32);
    }

    // Set default trigger type for all SPIs as level triggered
    let nr_regs = (gic_irqs + 15) / 16;
    
    for i in 1..nr_regs {
        gicd_write32(gicd_icfgr(i as u64), 0);
    }

    gic_dist_wait_for_rwp();

    /* Enabel Distributor */
    gicd_write32(GICD_CTLR, (GICD_CTLR_ENABLE_G1A | GICD_CTLR_ENABLE_G1));
    gic_dist_wait_for_rwp();

}

// Function to initialize GIC redistributor
fn gic_redist_init() {
    let cpu = coreid();
    
    /************ gic redistributor configuration ************/
    
    /* Enable GIC Redistributor */
    let waker = gicr_read32(cpu, GICR_WAKER);
    /* This PE is not in, and is not entering, a low power state. */
    gicr_write32(cpu, GICR_WAKER, waker & !(1 << 1));
    /* Wait until An interface to the connected PE might be active */
    while gicr_read32(cpu, GICR_WAKER) & (1 << 2) != 0 {}
    
    /* Configure SGIs and PPIs as non-secure Group 1 */
    gicr_write32(cpu, GICR_IGROUPR0, !0u32);
    gicr_write32(cpu, GICR_IGRPMODR0, 0);
    
    /* Disable all SGIs/PPIs */
    gicr_write32(cpu, GICR_ICENABLER0, !0u32);
    /* Clear all active SGIs/PPIs */
    gicr_write32(cpu, GICR_ICACTIVER0, !0u32);
    /* Clear all pending SGIs/PPIs */
    gicr_write32(cpu, GICR_ICPENDR0, !0u32);
    
    /* Configure PPIs as level triggered type */
    /* SGIs are set by hardware to be edge-triggered only */
    gicr_write32(cpu, GICR_ICFGR1, 0);
}

// Function to initialize GIC CPU interface
fn gic_icc_init() {
    let cpu = coreid();

    /************ cpu interface configuration ***********/

    /* Enable sysrem register */
    let sre = crate::read_sysreg!(ICC_SRE_EL2);
    /* bit 3: EL1 accesses to ICC_SRE_EL1 do not trap to EL2.
     * bit 0: The System register interface to the ICH_* registers
     * and the EL1 and EL2 ICC_* registers is enabled for EL2
     */
    crate::write_sysreg!(ICC_SRE_EL2, sre | (1 << 3) | 1);

    /* make sure ICC_SRE_EL2.SRE already set to 1, must? */
    crate::isb!();

    let sre = crate::read_sysreg!(ICC_SRE_EL1);
    /* The System register interface for the current Security state is enabled. */
    crate::write_sysreg!(ICC_SRE_EL1, sre | 1);

    /* Set the idle priority as the priority mask to allow all unterrupts */
    crate::write_sysreg!(ICC_PMR_EL1, 0xFF);
    /* Set binary points, only for Group 1 */
    crate::write_sysreg!(ICC_BPR1_EL1, 0x7);
    /* Set EOImode as split mode */
    crate::write_sysreg!(ICC_CTLR_EL1, (1 << 1));
    /* Enable SGIs/PPIs as Ns-Group 1 */
    crate::write_sysreg!(ICC_IGRPEN1_EL1, 1);
}

// Function to initialize GIC hypervisor interface
fn gic_hyp_init() {
    /* Virtual Group 1 interrupts are enabled. */
    crate::write_sysreg!(ICH_VMCR_EL2, (1 << 1));

    /* Virtual CPU interface operation enabled. */
    crate::write_sysreg!(ICH_HCR_EL2, (1 << 0));

    /* The number of implemented List registers - 1 */
    let vtr = crate::read_sysreg!(ICH_VTR_EL2);
    let mut gic_max_lrs = GIC_MAX_LRS.lock();
    *gic_max_lrs = (vtr & 0x1F) + 1;
}

pub fn gic_percpu_init() {
    gic_redist_init();
    gic_icc_init();
    gic_hyp_init();
}

pub fn gic_v3_init() {
    gic_dist_init();
    gic_percpu_init();
}


pub struct GicV3Ops;

impl GicV3Ops {
    pub const fn new() -> Self {
        GicV3Ops
    }
}

impl GicIrqOps for GicV3Ops {
    fn name(&self) -> &'static str {
        "ARM GICv3"
    }
    
    fn mask(&self, irq: u32) {
        gic_disable_int(irq);
    }
    
    fn unmask(&self, irq: u32) {
        gic_enable_int(irq);
    }
    
    fn get_irq(&self) -> u32 {
        gic_get_iar()
    }
    
    fn guest_eoi(&self, irq: u32) {
        gic_guest_eoi(irq);
    }
    
    fn hyp_eoi(&self, irq: u32) {
        gic_hyp_eoi(irq);
    }
    
    fn dir(&self, irq: u32) {
        gic_dir(irq);
    }
    
    fn configure(&self, irq: u32, irq_type: u32) {
        gic_set_config(irq, irq_type);
    }
    
    fn set_affinity(&self, irq: u32, cpu_num: u32) {
        gic_set_target(irq, cpu_num);
    }
    
    fn set_priority(&self, _irq: u32, _prio: u32) {
        todo!()
    }
    
    fn clear_pending(&self, _irq: u32) {
        todo!()
    }
    
    fn set_pending(&self, _irq: u32) {
        todo!()
    }
    
    fn get_pending(&self, _irq: u32) -> bool {
        todo!()
    }
    
    fn set_active(&self, _irq: u32) {
        todo!()
    }
}


// Disable IRQ for SPIs/SGIs/PPIs
fn gic_disable_int(irq: u32) {
    let cpu = coreid();
    
    if irq >= GIC_MIN_LPI {
        // LOG_WARN("X-Hyper don't support LPIs\n");
        return;
    }

    if irq >= GIC_MIN_SPI0 {  /* Configure for SPIs */
        let reg = gicd_read32(gicd_icenabler((irq / 32) as u64));
        let reg = reg | (1u32 << (irq % 32));
        gicd_write32(gicd_icenabler((irq / 32) as u64), reg);
    } else {  /* Configure for SGIs/PPIs */
        let reg = gicr_read32(cpu, GICR_ICENABLER0);
        let reg = reg | (1u32 << (irq % 32));
        gicr_write32(cpu, GICR_ICENABLER0, reg);
    }
}

// Enable IRQ for SPIs/SGIs/PPIs
fn gic_enable_int(irq: u32) {
    let cpu = coreid();
    
    if irq >= GIC_MIN_LPI {
        // LOG_WARN("X-Hyper don't support LPIs\n");
        return;
    }

    if irq >= GIC_MIN_SPI0 {  /* Configure for SPIs */
        let reg = gicd_read32(gicd_isenabler((irq / 32) as u64));
        let reg = reg | (1u32 << (irq % 32));
        gicd_write32(gicd_isenabler((irq / 32) as u64), reg);
    } else {  /* Configure for SGIs/PPIs */
        let reg = gicr_read32(cpu, GICR_ISENABLER0);
        let reg = reg | (1u32 << (irq % 32));
        gicr_write32(cpu, GICR_ISENABLER0, reg);
    }
}

// Get irq number
fn gic_get_iar() -> u32 {
    crate::read_sysreg!(ICC_IAR1_EL1) as u32
}

fn gic_dir(irq: u32) {
    crate::write_sysreg!(ICC_DIR_EL1, irq);
}

fn gic_hyp_eoi(irq: u32) {
    /* In split EOI mode, we to set EOI and DIR */
    crate::write_sysreg!(ICC_EOIR1_EL1, irq);
    crate::write_sysreg!(ICC_DIR_EL1, irq);
}

fn gic_set_target(irq: u32, target: u32) {
    if irq < GIC_MIN_SPI0 {
        return;
    }

    if irq >= GIC_MIN_LPI {
        // LOG_WARN("X-Hyper don't support LPIs\n");
        return;
    }

    let reg = gicd_read32(gicd_itargetsr((irq / 4) as u64));
    let reg = reg & (0xFFu32 << (irq % 4 * 8));
    gicd_write32(gicd_itargetsr((irq / 4) as u64), reg | (target << (irq % 4 * 8)));
}

fn gic_set_config(irq: u32, config: u32) {
    let cpu = coreid();
    
    if irq >= GIC_MIN_SPI0 {
        let shift = (irq % 16) * 2;
        let reg = gicd_read32(gicd_icfgr((irq / 16) as u64));
        let reg = reg & (!((0x03u32) << shift));
        let reg = reg | ((config << shift) as u32);
        gicd_write32(gicd_icfgr((irq / 16) as u64), reg);
    } else {
        let shift = (irq % 16) * 2;
        let reg = gicr_read32(cpu, GICR_ICFGR1);
        let reg = reg & (!((0x03u32) << shift));
        let reg = reg | ((config << shift) as u32);
        gicr_write32(cpu, GICR_ICFGR1, reg);
    }
}


fn gic_guest_eoi(irq: u32) {
    crate::write_sysreg!(ICC_EOIR1_EL1, irq as u64);
    crate::write_sysreg!(ICC_DIR_EL1, irq as u64);
}

lazy_static!(
    pub static ref GIC_IRQ_OPS: Mutex<GicV3Ops> = Mutex::new(GicV3Ops::new());
);

pub fn hyper_spi_config(irq : u32, typed : u32) {
    let gic_irq_ops = GIC_IRQ_OPS.lock();
    gic_irq_ops.set_affinity(irq, 0);
    gic_irq_ops.configure(irq, typed);
    gic_irq_ops.unmask(irq);
}


pub const GIC_LEVEL_TRIGGER: u32 = 0;
pub const GIC_EDGE_TRIGGER: u32 = 2;
pub const UART_IRQ_LINE: u32 = 33;

pub fn gic_write_list_reg(n: usize, val: u64) {
    match n {
        0 => crate::write_sysreg!(ICH_LR0_EL2, val),
        1 => crate::write_sysreg!(ICH_LR1_EL2, val),
        2 => crate::write_sysreg!(ICH_LR2_EL2, val),
        3 => crate::write_sysreg!(ICH_LR3_EL2, val),
        4 => crate::write_sysreg!(ICH_LR4_EL2, val),
        5 => crate::write_sysreg!(ICH_LR5_EL2, val),
        6 => crate::write_sysreg!(ICH_LR6_EL2, val),
        7 => crate::write_sysreg!(ICH_LR7_EL2, val),
        8 => crate::write_sysreg!(ICH_LR8_EL2, val),
        9 => crate::write_sysreg!(ICH_LR9_EL2, val),
        10 => crate::write_sysreg!(ICH_LR10_EL2, val),
        11 => crate::write_sysreg!(ICH_LR11_EL2, val),
        12 => crate::write_sysreg!(ICH_LR12_EL2, val),
        13 => crate::write_sysreg!(ICH_LR13_EL2, val),
        14 => crate::write_sysreg!(ICH_LR14_EL2, val),
        15 => crate::write_sysreg!(ICH_LR15_EL2, val),
        _ => panic!("Unknown ICH LR number"),
    }
}


// LR字段定义
const fn lr_state(n: u64) -> u64 {
    (n & 0x3) << 62
}

const fn lr_group(n: u64) -> u64 {
    (n & 0x1) << 60
}

const fn lr_pintid(n: u64) -> u64 {
    (n & 0x3ff) << 32
}

const fn lr_vintid(n: u64) -> u64 {
    n & 0xffffffff
}

const fn lr_is_inactive(lr: u64) -> bool {
    ((lr >> 62) & 0x3) == 0
}

// 常量定义
const LR_HW: u64 = 1 << 61;
const LR_PENDING: u64 = 1;

pub fn gic_create_lr(pirq: u32, virq: u32) -> u64 {
    /*
     * The state of the interrupt is pending
     * The interrupt maps directly to a hardware interrupt
     * This is a Group 1 virtual interrupt
     */
    lr_state(LR_PENDING) | LR_HW | lr_group(1) | lr_pintid(pirq as u64) | lr_vintid(virq as u64)
}