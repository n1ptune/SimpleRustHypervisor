
use crate::{arch::flush_tlb, isb, read_sysreg, write_sysreg};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VcpuRegs {
    // 通用寄存器
    pub x: [u64; 31],       // X0-X30
    pub sp_el0: u64,        // 用户栈指针
    pub sp_el1: u64,        // 内核栈指针
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

    pub pc: u64,
}

impl VcpuRegs {
    pub fn new(entry_point:u64) -> Self {
        VcpuRegs {
            x: [0; 31],
            sp_el0: 0,
            sp_el1: 0,
            elr_el1: 0,
            pc: entry_point,
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
#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VcpuState {
    Stopped,
    Running,
    Waiting,
    Blocked,
}
#[allow(unused)]
#[derive(Debug)]
pub struct Vcpu {
    pub id: u32,
    pub regs: VcpuRegs,
    pub state: VcpuState,
    pub entry_point: u64,
}

#[allow(unused)]
impl Vcpu {
    pub fn new(id: u32, entry_point: u64) -> Self {
        let mut vcpu = Vcpu {
            id,
            regs: VcpuRegs::new(entry_point),
            state: VcpuState::Stopped,
            entry_point,
        };
        
        // setup entry point
        vcpu.regs.elr_el1 = entry_point;
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
        write_sysreg!(sp_el1, self.regs.sp_el1);
        write_sysreg!(sp_el0, self.regs.sp_el0);
        write_sysreg!(elr_el1, self.regs.elr_el1);
        write_sysreg!(spsr_el1, self.regs.spsr_el1);
        write_sysreg!(sctlr_el1, self.regs.sctlr_el1);
        write_sysreg!(tcr_el1, self.regs.tcr_el1);
        write_sysreg!(ttbr0_el1, self.regs.ttbr0_el1);
        write_sysreg!(ttbr1_el1, self.regs.ttbr1_el1);
        write_sysreg!(mair_el1, self.regs.mair_el1);
        write_sysreg!(vbar_el1, self.regs.vbar_el1);
        write_sysreg!(esr_el1, self.regs.esr_el1);
        write_sysreg!(far_el1, self.regs.far_el1);

        write_sysreg!(elr_el2, self.regs.pc);
        isb!();
    }

    fn save_guest_context(&mut self) {
        self.regs.sp_el1 = read_sysreg!(sp_el1);
        self.regs.sp_el0 = read_sysreg!(sp_el0);
        self.regs.elr_el1 = read_sysreg!(elr_el1);
        self.regs.spsr_el1 = read_sysreg!(spsr_el1);
        self.regs.sctlr_el1 = read_sysreg!(sctlr_el1);
        self.regs.tcr_el1 = read_sysreg!(tcr_el1);
        self.regs.ttbr0_el1 = read_sysreg!(ttbr0_el1);
        self.regs.ttbr1_el1 = read_sysreg!(ttbr1_el1);
        self.regs.mair_el1 = read_sysreg!(mair_el1);
        self.regs.vbar_el1 = read_sysreg!(vbar_el1);
        self.regs.esr_el1 = read_sysreg!(esr_el1);
        self.regs.far_el1 = read_sysreg!(far_el1);

        self.regs.pc = read_sysreg!(elr_el2);
        isb!();
    }


    fn world_switch_to_guest(&mut self) {
        flush_tlb();
        self.restore_guest_context();
        write_sysreg!(spsr_el2, 0x3c5);
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