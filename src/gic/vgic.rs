use alloc::{format, string::String};
use crate::{gic::{GIC_MAX_LRS, VgicIrqConfig, VgicVcpu, gicr::*, gicv3::{gic_create_lr, gic_read_list_reg, gic_write_list_reg, lr_is_inactive}}, vm::{MmioAccess, Vcpu}};
use super::gicd::*;
use crate::vm::{MmioSpace};
#[allow(unused)]
use log::*;
pub struct VirtualGicd {
    pub base_addr: usize,
    pub size: usize,
}

impl VirtualGicd {
    pub fn new(base_addr: usize, size: usize) -> Self {
        VirtualGicd { base_addr, size }
    }
}

impl MmioSpace for VirtualGicd {
    fn base_addr(&self) -> usize {
        self.base_addr
    }

    fn size(&self) -> usize {
        self.size
    }

    fn mmio_read(&mut self, vcpu: &mut Vcpu, srt: usize, offset: usize, access: MmioAccess) -> bool {
        // info!("vgicd_read offset {:x} access {:x?}", offset, access.wnr);
        // 目标寄存器是零寄存器（WZR/XZR），丢弃结果即可
        if srt == 31 {
            return true;
        }
        match offset {
            GICD_CTLR => {
                let mut vgic_dist = vcpu.vgic_dist.lock();
                let mut data = GICD_CTLR_ARE_NA;
                if vgic_dist.enabled {
                    data |= GICD_CTLR_ENABLE_G1A;
                }
                vcpu.regs.x[srt] = data as u64;
                return true;
            },
            GICD_TYPER => {
                // ITLinesNumber
                let mut vgic_dist = vcpu.vgic_dist.lock();
                let mut reg_val = (((vgic_dist.nspis + 32) >> 5) - 1) as u32;
                // CPUNumber
                reg_val |= (8 - 1) << 8; // GICD_TYPER_CPUNumber_SHIFT is 8
                reg_val &= !((1 << 26) | (1 << 8) | (1 << 19));
                vcpu.regs.x[srt] = reg_val as u64;
                return true;
            },
            GICD_IIDR => {
                vcpu.regs.x[srt] = gicd_read32(GICD_IIDR) as u64;
                return true;
            },
            GICD_TYPER2 => {
                // Linux reads this reg to the feature of gicv3, fake it
                vcpu.regs.x[srt] = 0;
                return true;
            },
            // Handle GICD_IGROUPR ranges
            o if o >= gicd_igroupr(0) && o < gicd_igroupr(31) + 4 => {
                let irq_num = (offset - gicd_igroupr(0)) / 4 * 32;
                let mut igrp = 0u32;
                for i in 0..32 {
                    if let Some(irq) = vcpu.vgic_irq_get(irq_num + i) {
                        igrp |= (irq.group as u32) << i;
                    }
                }
                vcpu.regs.x[srt] = igrp as u64;
                return true;
            },
            // Handle GICD_ISENABLER ranges
            o if o >= gicd_isenabler(0) && o < gicd_isenabler(31) + 4 => {
                let irq_num = (offset - gicd_isenabler(0)) / 4 * 32;
                let mut isen = 0u32;
                for i in 0..32 {
                    if let Some(irq) = vcpu.vgic_irq_get(irq_num + i) {
                        isen |= (irq.enabled as u32) << i;
                    }
                }
                vcpu.regs.x[srt] = isen as u64;
                return true;
            },
            // Handle GICD_IPRIORITYR ranges
            o if o >= gicd_ipriorityr(0) && o < gicd_ipriorityr(254) + 4 => {
                let irq_num = (offset - gicd_ipriorityr(0)) / 4 * 4;
                let mut iprio = 0u32;
                for i in 0..4 {
                    if let Some(irq) = vcpu.vgic_irq_get(irq_num + i) {
                        iprio |= (irq.priority as u32) << (i * 8);
                    }
                }
                vcpu.regs.x[srt] = iprio as u64;
                return true;
            },
            // Handle GICD_ITARGETSR ranges
            o if o >= gicd_itargetsr(0) && o < gicd_itargetsr(254) + 4 => {
                let irq_num = (offset - gicd_itargetsr(0)) / 4 * 4;
                let mut itar = 0u32;
                for i in 0..4 {
                    if let Some(irq) = vcpu.vgic_irq_get(irq_num + i) {
                        itar |= (irq.affinity as u32) << (i * 8);
                    }
                }
                vcpu.regs.x[srt] = itar as u64;
                return true;
            },
            // Handle other cases that return 0
            o if (o >= gicd_ispendr(0) && o < gicd_ispendr(31) + 4) ||     // GICD_ISPENDR
                  (o >= gicd_icpendr(0) && o < gicd_icpendr(31) + 4) ||     // GICD_ICPENDR
                  (o >= gicd_isactiver(0) && o < gicd_isactiver(31) + 4) ||     // GICD_ISACTIVER
                  (o >= gicd_icactiver(0) && o < gicd_icactiver(31) + 4) ||     // GICD_ICACTIVER
                  (o >= gicd_icfgr(0) && o < gicd_icfgr(63) + 4) ||     // GICD_ICFGR
                  (o >= gicd_irouter(0) && o < gicd_irouter(31) + 4) ||   // GICD_IROUTER(0-31)
                  (o >= gicd_irouter(32) && o < gicd_irouter(1019) + 4) => { // GICD_IROUTER(32-1019)
                vcpu.regs.x[srt] = 0;
                return true;
            },
            GICD_PIDR2 => {
                vcpu.regs.x[srt] = gicd_read32(GICD_PIDR2) as u64;
                return true;
            },
            _ => {
                error!("[vgicd_read] Unable to handle the GICD_* request");
            }
        }
        false
    }

    fn mmio_write(&mut self, vcpu: &mut Vcpu, srt: usize, offset: usize, access: MmioAccess) -> bool {
        // info!("vgicd_write offset {:x} access {:x?}", offset, access.wnr);
        // srt==31 表示零寄存器，值恒为 0，不访问寄存器数组
        let val = if srt == 31 { 0 } else { vcpu.regs.x[srt] };
        match offset {
            // simulate GICD_CTLR
            GICD_CTLR => {
                let mut vgic_dist = vcpu.vgic_dist.lock();
                if (val as u32) & GICD_CTLR_ENABLE_G1A != 0 {
                    vgic_dist.enabled = true;
                } else {
                    vgic_dist.enabled = false;
                }
                return true;
            },
            // simulate GICD_TYPER and GICD_IIDR
            GICD_TYPER | GICD_IIDR => {
                // Read only register
                return true;
            },
            // Handle GICD_IGROUPR ranges
            o if o >= gicd_igroupr(0) && o < gicd_igroupr(31) + 4 => {
                // Implementation would go here
                return true;
            },
            // Handle GICD_ISENABLER ranges
            o if o >= gicd_isenabler(0) && o < gicd_isenabler(31) + 4 => {
                let irq_num = (offset - gicd_isenabler(0)) / 4 * 32;
                for i in 0..32 {
                    if let Some(mut irq) = vcpu.vgic_irq_get(irq_num + i) {
                        if ((val as u32) >> i) & 0x1 != 0 {
                            irq.enabled = 1;
                            Vcpu::vgic_irq_enable(irq_num + i);
                        }
                    }
                }
                return true;
            },
            // Handle GICD_ICENABLER ranges
            o if o >= gicd_icenabler(0) && o < gicd_icenabler(31) + 4 => {
                let irq_num = (offset - gicd_icenabler(0)) / 4 * 32;
                for i in 0..32 {
                    if let Some(mut irq) = vcpu.vgic_irq_get(irq_num + i) {
                        if ((val as u32) >> i) & 0x1 != 0 {
                            irq.enabled = 0;
                            Vcpu::vgic_irq_disable(irq_num + i);
                        }
                    }
                }
                return true;
            },
            // Handle GICD_IPRIORITYR ranges
            o if o >= gicd_ipriorityr(0) && o < gicd_ipriorityr(254) + 4 => {
                let irq_num = (offset - gicd_ipriorityr(0)) / 4 * 4;
                for i in 0..4 {
                    if let Some(mut irq) = vcpu.vgic_irq_get(irq_num + i) {
                        irq.priority = (((val as u32) >> (i * 8)) & 0xff) as u8;
                    }
                }
                return true;
            },
            // Handle GICD_ITARGETSR ranges
            o if o >= gicd_itargetsr(0) && o < gicd_itargetsr(254) + 4 => {
                let irq_num = (offset - gicd_itargetsr(0)) / 4 * 4;
                for i in 0..4 {
                    if let Some(mut irq) = vcpu.vgic_irq_get(irq_num + i) {
                        irq.affinity = (((val as u32) >> (i * 8)) & 0xff) as u8;
                        Vcpu::vgic_target_set(irq_num + i, irq.affinity);
                    }
                }
                return true;
            },
            // Handle other cases that do nothing
            o if (o >= gicd_ispendr(0) && o < gicd_ispendr(31) + 4) ||     // GICD_ISPENDR
                  (o >= gicd_icpendr(0) && o < gicd_icpendr(31) + 4) ||     // GICD_ICPENDR
                  (o >= gicd_isactiver(0) && o < gicd_isactiver(31) + 4) ||     // GICD_ISACTIVER
                  (o >= gicd_icactiver(0) && o < gicd_icactiver(31) + 4) ||     // GICD_ICACTIVER
                  (o >= gicd_icfgr(0) && o < gicd_icfgr(63) + 4) ||     // GICD_ICFGR
                  (o >= gicd_irouter(0) && o < gicd_irouter(31) + 4) ||   // GICD_IROUTER(0-31)
                  (o >= gicd_irouter(32) && o < gicd_irouter(1019) + 4) => { // GICD_IROUTER(32-1019)
                return true;
            },
            _ => {
                error!("[vgicd_write] Unable to handle the GICD_* request");
                // vcpu.regs.elr += 4;
            }
        }
        false
    }

    fn debug_info(&self) -> String {
        format!("VirtualGicd start {} size {}",  self.base_addr, self.size)
    }
}


pub struct VirtualGicr {
    pub base_addr: usize,
    pub size: usize,
}

impl VirtualGicr {
    pub fn new(base_addr: usize, size: usize) -> Self {
        VirtualGicr { base_addr, size }
    }
}

impl MmioSpace for VirtualGicr {
    fn base_addr(&self) -> usize {
        self.base_addr
    }

    fn size(&self) -> usize {
        self.size
    }

    fn mmio_read(&mut self, vcpu: &mut Vcpu, srt: usize, offset: usize, access: MmioAccess) -> bool {
        // info!("vgicr_read offset {:x} access {:x?}", offset, access.wnr);
        if srt == 31 {
            return true;
        }

        let _ = access;
        let gicr_index = offset / GICR_STRIDE;
        let gicr_offset = offset % GICR_STRIDE;
        
        match gicr_offset {
            GICR_CTLR | GICR_WAKER | GICR_IGROUPR0 => {
                vcpu.regs.x[srt] = 0;
                return true;
            },
            GICR_IIDR => {
                vcpu.regs.x[srt] = gicr_read32(vcpu.id, GICR_IIDR) as u64;
                return true;
            },
            GICR_TYPER => {
                vcpu.regs.x[srt] = gicr_read64(vcpu.id, GICR_TYPER);
                return true;
            },
            GICR_PIDR2 => {
                vcpu.regs.x[srt] = gicr_read32(vcpu.id, GICR_PIDR2) as u64;
                return true;
            },
            GICR_ISENABLER0 => {
                let mut isen = 0u32;
                for i in 0..32 {
                    if let Some(irq) = vcpu.vgic_irq_get(i) {
                        isen |= (irq.enabled as u32) << i;
                    }
                }
                vcpu.regs.x[srt] = isen as u64;
                return true;
            },
            GICR_ICENABLER0 | GICR_ICPENDR0 | GICR_ISACTIVER0 | GICR_ICACTIVER0 | GICR_ICFGR0 | GICR_ICFGR1 | GICR_IGRPMODR0 => {
                vcpu.regs.x[srt] = 0;
                return true;
            },
            o if o >= gicr_ipriorityr(0) && o < gicr_ipriorityr(7) + 4 => {
                let irq_num = (offset - gicr_ipriorityr(0)) / 4 * 4;
                let mut iprio = 0u32;
                for i in 0..4 {
                    if let Some(irq) = vcpu.vgic_irq_get(irq_num + i) {
                        iprio |= (irq.priority as u32) << (i * 8);
                    }
                }
                vcpu.regs.x[srt] = iprio as u64;
                return true;
            },
            GICR_ICFGR0 | GICR_ICFGR1 | GICR_IGRPMODR0 => {
                vcpu.regs.x[srt] = 0;
                return true;
            },
            _ => {
                error!("[vgicr_read] Unable to handle the GICR_* request");
            }
        }
        false
    }

    fn mmio_write(&mut self, vcpu: &mut Vcpu, srt: usize, offset: usize, access: MmioAccess) -> bool {
        // info!("vgicr_write offset {:x} access {:x?}", offset, access.wnr);
        let val = if srt == 31 { 0 } else { vcpu.regs.x[srt] };

        let gicr_index = offset / GICR_STRIDE;
        let gicr_offset = offset % GICR_STRIDE;
        
        let val = vcpu.regs.x[srt];
        match gicr_offset {
            GICR_CTLR | GICR_WAKER | GICR_IGROUPR0 | GICR_TYPER | GICR_PIDR2 => {
                return true;
            },
            GICR_ISENABLER0 => {
                for i in 0..32 {
                    if let Some(mut irq) = vcpu.vgic_irq_get(i) {
                        if ((val >> i) & 0x1) != 0 {
                            irq.enabled = 1;
                            Vcpu::vgic_irq_enable(i);
                        }
                    }
                }
                return true;
            },
            GICR_ICENABLER0 | GICR_ICPENDR0 | GICR_ISACTIVER0 | GICR_ICACTIVER0 => {
                return true;
            },
            o if o >= gicr_ipriorityr(0) && o < gicr_ipriorityr(7) + 4 => {
                let irq_num = (offset - gicr_ipriorityr(0)) / 4 * 4;
                for i in 0..4 {
                    if let Some(mut irq) = vcpu.vgic_irq_get(irq_num + i) {
                        irq.priority = ((val >> (i * 8)) & 0xff) as u8;
                    }
                }
                return true;
            },
            GICR_ICFGR0 | GICR_ICFGR1 | GICR_IGRPMODR0 => {
                return true;
            },
            _ => {
                error!("[vgicr_write] Unable to handle the GICR_* request");
            }
        }
        false
    }

    fn debug_info(&self) -> String {
        format!("VirtualGicr start {} size {}",  self.base_addr, self.size)
    }
    
}


fn alloc_lr(vgic_cpu: &mut VgicVcpu) -> Option<u64> {
    let gic_max_lrs = GIC_MAX_LRS.lock();
    
    for i in 0..*gic_max_lrs {
        if (vgic_cpu.used_lr & (1 << i)) == 0 {
            vgic_cpu.used_lr |= 1 << i;
            return Some(i);
        }
    }
    
    None
}

pub fn virq_inject(vcpu: &mut Vcpu, pirq: u32, virq: u32) -> Result<(), &'static str> {
    let lr = gic_create_lr(pirq, virq);
    
    if let Some(n) = alloc_lr(&mut vcpu.vgic) {
        gic_write_list_reg(n, lr);
        // debug!("Injected IRQ {} to List Register {}", virq, n);
        Ok(())
    } else {
        // info!("Failed to inject IRQ {}: no available List Register", virq);
        Err("No List Register")
    }
}


pub fn vgic_enter(vcpu: &mut Vcpu) {
    let mut vgic_cpu = &mut vcpu.vgic;
    let gic_max_lrs = GIC_MAX_LRS.lock();
    for i in 0..*gic_max_lrs {
        if (vgic_cpu.used_lr & (1 << i)) != 0 {
            let lr = gic_read_list_reg(i);
            if lr_is_inactive(lr) {
                // info!("VGIC: Reclaiming active LR {} with value {:x}", i, lr);
                vgic_cpu.used_lr &= !(1 << i);
            }
        }
    }

}