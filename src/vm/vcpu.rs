
use alloc::sync::Arc;
#[allow(unused)]
use log::*;
use spin::Mutex;

use crate::{arch::flush_tlb, 
    gic::{GIC_IRQ_OPS, VgicDist, VgicVcpu, GicIrqOps}, 
    isb, 
    read_sysreg, 
    write_sysreg,
    utils::IrqRef, 
    vm::MmioManager,

};
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VcpuRegs {
    // 通用寄存器
    pub x: [u64; 31],       // X0-X30
    pub spsr: u64,   
    pub elr: u64,      
  
    // pub sp_el0: u64,        // 用户栈指针
    // pub sp_el1: u64,        // 内核栈指针
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VcpuSysRegs {
    pub elr_el1: u64,       // 异常链接寄存器
    pub spsr_el1: u64,      // 程序状态寄存器
    
    // 系统寄存器
    pub sctlr_el1: u64,     // 系统控制寄存器
    pub tcr_el1: u64,       // 转换控制寄存器
    pub ttbr0_el1: u64,     // 转换表基址寄存器0
    pub ttbr1_el1: u64,     // 转换表基址寄存器1
    pub mair_el1: u64,      // 内存属性间接寄存器
    pub vbar_el1: u64,      // 向量基址寄存器
    pub esr_el1: u64,       // 异常综合寄存器
    pub far_el1: u64,       // 故障地址寄存器
}

impl VcpuSysRegs {
    pub fn new() -> Self {
        VcpuSysRegs {
            elr_el1: 0,
            spsr_el1: 0x3c5, // EL1h, IRQ/FIQ masked
            sctlr_el1: 0x30c50830, // 默认系统控制寄存器值
            tcr_el1: 0,
            ttbr0_el1: 0,
            ttbr1_el1: 0,
            mair_el1: 0,
            vbar_el1: 0,
            esr_el1: 0,
            far_el1: 0,
        }
    }
}

impl VcpuRegs {
    pub fn new(entry_point:u64) -> Self {
        VcpuRegs {
            x: [0; 31],
            // sp_el0: 0,
            // sp_el1: 0,
            elr: entry_point,
            spsr: 0,
        }
    }
}
#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VcpuState {
    Stopped,
    Running,
    Waiting,
    Blocked,
}
#[repr(C)]
#[derive(Debug, Clone)]
pub struct Vcpu {
    pub regs: VcpuRegs,
    pub sysregs: VcpuSysRegs,
    pub id: usize,
    pub vm_id: u32,
    pub state: VcpuState,
    pub entry_point: u64,
    pub vgic : VgicVcpu,
    pub vgic_dist : Arc<Mutex<VgicDist>>,
    pub mmio_manager: Arc<Mutex<MmioManager>>,
}

#[allow(unused)]
impl Vcpu {
    pub fn new(id: usize, vm_id: u32, entry_point: u64, vgic_dist: Arc<Mutex<VgicDist>>, mmio_manager: Arc<Mutex<MmioManager>>) -> Self {
        let mut vcpu = Vcpu {
            id,
            vm_id,
            regs: VcpuRegs::new(entry_point),
            sysregs: VcpuSysRegs::new(),
            state: VcpuState::Stopped,
            entry_point,
            vgic: VgicVcpu::new(),
            vgic_dist,
            mmio_manager,
        };
        
        vcpu.vgic.init(id);
        // setup entry point
        vcpu.regs.elr = entry_point;
        vcpu
    }
    
    pub fn run(&mut self) -> Result<(), &'static str> {
        if self.state != VcpuState::Stopped {
            return Err("VCPU is not in stopped state");
        }
        
        self.state = VcpuState::Running;
        
        // now let's switching to guest
        self.world_switch_to_guest();
        
        Ok(())
    }
    
    fn restore_guest_context(&mut self) {
        // write_sysreg!(sp_el1, self.regs.sp_el1);
        // write_sysreg!(sp_el0, self.regs.sp_el0);
        
        write_sysreg!(elr_el1, self.sysregs.elr_el1);
        write_sysreg!(spsr_el1, self.sysregs.spsr_el1);
        write_sysreg!(sctlr_el1, self.sysregs.sctlr_el1);
        write_sysreg!(tcr_el1, self.sysregs.tcr_el1);
        write_sysreg!(ttbr0_el1, self.sysregs.ttbr0_el1);
        write_sysreg!(ttbr1_el1, self.sysregs.ttbr1_el1);
        write_sysreg!(mair_el1, self.sysregs.mair_el1);
        write_sysreg!(vbar_el1, self.sysregs.vbar_el1);
        write_sysreg!(esr_el1, self.sysregs.esr_el1);
        write_sysreg!(far_el1, self.sysregs.far_el1);

        write_sysreg!(elr_el2, self.regs.elr);
        isb!();
    }

    fn save_guest_context(&mut self) {
        // self.regs.sp_el1 = read_sysreg!(sp_el1);
        // self.regs.sp_el0 = read_sysreg!(sp_el0);
        self.sysregs.elr_el1 = read_sysreg!(elr_el1);
        self.sysregs.spsr_el1 = read_sysreg!(spsr_el1);
        self.sysregs.sctlr_el1 = read_sysreg!(sctlr_el1);
        self.sysregs.tcr_el1 = read_sysreg!(tcr_el1);
        self.sysregs.ttbr0_el1 = read_sysreg!(ttbr0_el1);
        self.sysregs.ttbr1_el1 = read_sysreg!(ttbr1_el1);
        self.sysregs.mair_el1 = read_sysreg!(mair_el1);
        self.sysregs.vbar_el1 = read_sysreg!(vbar_el1);
        self.sysregs.esr_el1 = read_sysreg!(esr_el1);
        self.sysregs.far_el1 = read_sysreg!(far_el1);

        self.regs.elr = read_sysreg!(elr_el2);
        isb!();
    }


    fn world_switch_to_guest(&mut self) {
        flush_tlb();
        self.restore_guest_context();
        write_sysreg!(spsr_el2, 0x3c5);
        write_sysreg!(tpidr_el2, self as *mut Vcpu as u64);
        // write_sysreg!(cpacr_el1, 3 << 20);
        isb!();
        unsafe {
            self.enter_guest();
        }
    }
    
    unsafe fn enter_guest(&mut self) {
        // restore guest context
        core::arch::asm!(
            "ic ialluis",
            "mov x0, {entry}",
            "eret",
            entry = in(reg) self.entry_point,
            options(noreturn)
        );
    }
    
    pub fn handle_exit(&mut self, exit_reason: ExitReason) -> VmExitAction {
        match exit_reason {
            ExitReason::Hvc => {
                // 处理 HVC 调用
                VmExitAction::Continue
            },
            ExitReason::DataAbort => {
                // 处理数据中止
                VmExitAction::Continue
            },
            ExitReason::InstructionAbort => {
                // 处理指令中止
                VmExitAction::Continue
            },
            ExitReason::Irq => {
                // 处理中断
                VmExitAction::Continue
            },
        }
    }

    // pub fn get_vm(&self) -> Option<Arc<Mutex<VirtualMachine>>> {
    //     let vm_manager = VM_MANAGER.lock();
    //     debug!("get_vm {}", self.vm_id);
    //     for vm_arc in vm_manager.iter() {
    //         let vmid = vm_arc.lock().id;
    //         debug!("vm_id {}", vmid);
    //         if vmid == self.vm_id {
    //             debug!("vm_id {}", vmid);
    //             return Some(vm_arc.clone());
    //         }
    //     }

    //     None
    // }



    pub fn vgic_irq_get<'a>(&'a mut self, irq_num: usize) -> Option<IrqRef<'a>> {
        if irq_num < 16 {
            // SGI
            self.vgic.sgis.get_mut(irq_num).map(IrqRef::Local)
        } else if irq_num < 32 {
            // PPI
            let idx = irq_num - 16;
            self.vgic.ppi.get_mut(idx).map(IrqRef::Local)
        } else {
            // SPI
            let idx = irq_num - 32;
            // 1. 获取锁
            let mut dist = self.vgic_dist.lock();
            
            // 2. 检查索引
            if idx < dist.spis.len() {
                // 3. 将 锁 和 索引 一起打包返回
                // 此时锁没有释放，而是交给了调用者
                Some(IrqRef::Shared(dist, idx))
            } else {
                // 没找到，dist 离开作用域，锁自动释放
                None
            }
        }
    }

    pub fn vgic_irq_enable(irq_num: usize) {
        let gic_irq_ops = GIC_IRQ_OPS.lock();
        gic_irq_ops.unmask(irq_num as u32);
    }

    pub fn vgic_irq_disable(irq_num: usize) {
        let gic_irq_ops = GIC_IRQ_OPS.lock();
        gic_irq_ops.mask(irq_num as u32);
    }

    pub fn vgic_target_set(irq_num: usize, target: u8) {
        let gic_irq_ops = GIC_IRQ_OPS.lock();
        gic_irq_ops.set_affinity(irq_num as u32, target as u32);
    }

}

#[allow(unused)]
#[derive(Debug, Clone, Copy)]
pub enum ExitReason {
    Hvc,           // Hypervisor Call
    DataAbort,     // 数据访问异常
    InstructionAbort, // 指令访问异常
    Irq,           // 中断
}

#[allow(unused)]
#[derive(Debug, Clone, Copy)]
pub enum VmExitAction {
    Continue,      // 继续运行虚拟机
    Stop,          // 停止虚拟机
    Restart,       // 重启虚拟机
}