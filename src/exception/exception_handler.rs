#![allow(unused)]
use core::arch::asm;

use crate::read_sysreg;
use crate::vm::{AccessSize, ExitReason, MmioAccess, VM_MANAGER, Vcpu, VmExitAction};
use crate::vm::{EsrEl2, Ec};
use log::*;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ExceptionContext {
    pub sp_el1: u64,  // SP_EL1
    pub pc: u64,     // ELR_EL2
    pub spsr: u64,    // SPSR_EL2
    pub esr: EsrEl2,     // ESR_EL2
    pub far: u64,     // FAR_EL2
    pub x: [u64; 31], // x0-x30
}

pub fn get_current_vcpu_ptr() -> *mut Vcpu {
    let tpidr_el2: u64;
    unsafe {
        asm!("mrs {}, tpidr_el2", out(reg) tpidr_el2);
    }
    tpidr_el2 as *mut Vcpu
}

pub fn dump_regs(ctx: &mut ExceptionContext){
    unsafe {
        info!("============================");
        info!("Exception Context:");
        for i in (0..30).step_by(2) {
            info!("x{}: 0x{:x}, x{}: 0x{:x}", i, ctx.x[i], i + 1, ctx.x[i + 1]);
        }
        info!("x{}: 0x{:x}", 30, ctx.x[30]);
        info!("ESR_EL2: 0x{:x}, FAR_EL2: 0x{:x}", ctx.esr.raw(), ctx.far);
        info!("ELR_EL2: 0x{:x}, SPSR_EL2: 0x{:x}", ctx.pc, ctx.spsr);
        info!("SP_EL1: 0x{:x}", ctx.sp_el1);
        info!("============================");
    }

}
#[no_mangle]
pub extern "C" fn handle_sync_exception_from_asm() {
    // dump_regs(ctx);
    // panic!("vm exit");
    // dispatch ec
    let vcpu_ptr = get_current_vcpu_ptr();

    if vcpu_ptr.is_null() {
        panic!("Critical: Exception in hypervisor context!");
    }

    let vcpu_ref = unsafe { &mut *vcpu_ptr };

    
    let esr = EsrEl2::new(read_sysreg!(ESR_EL2));

    match Ec::from_u8(esr.ec()) {
        Ec::DataAbort => handle_data_abort(vcpu_ref, esr),
        Ec::InstAbort => handle_insn_abort(vcpu_ref),
        _ => {
            info!("Exception Class: 0x{:x}", esr.ec());
        }
    }
    // ctx.pc += 4;
}

pub fn handle_data_abort(vcpu: &mut Vcpu, esr: EsrEl2) {
    
    let mmio_manager = vcpu.mmio_manager.clone();


    let sas = esr.sas();
    let srt = esr.srt() as usize;
    let wnr = esr.is_write().unwrap();


    let far = read_sysreg!(FAR_EL2);

    let access = MmioAccess {
                            ipa: far as usize, 
                            pc: vcpu.regs.elr as usize, 
                            wnr: wnr, 
                            access_size: AccessSize::from_size(sas).unwrap() 
                        };


    if mmio_manager.lock().handle_mmio(vcpu, srt, access){
        vcpu.regs.elr += 4;
        return;
    }
    info!("unknown data abort at FAR=0x{:x}", far);
    // let mut dm = DEVICE_MANAGER.lock();
    // // debug!("ctx esr iss {:b}", ctx.esr.iss());
    // let len = ctx.esr.sas();
    // let rt = ctx.esr.srt() as usize; 
    // match ctx.esr.is_write(){
    //     Some(false) => {
    //         ctx.x[rt] = dm.handle_mmio(ctx.far,DeviceAccess::Read , len, None).unwrap();
    //         ctx.pc += 4;
    //     }
    //     Some(true) => {
    //         // dump_regs(ctx);
    //         dm.handle_mmio(ctx.far, DeviceAccess::Write, len, Some(ctx.x[rt]));
    //         ctx.pc += 4;
    //     }
    //     _ => {
    //         error!("handle_data_abort");
    //     }
    // }
    // dm.handle_mmio(ctx.far, access, 1, value);
    // debug!("handle_data_abort done");
}

pub fn handle_insn_abort(vcpu: &mut Vcpu){
    debug!("handle_insn_abort");
    panic!();
}

pub fn handle_sync_exception(ctx: &mut ExceptionContext) -> VmExitAction {
    let ec = ctx.esr.ec();
    
    info!("Sync exception: EC=0x{:x}, ESR=0x{:x}, FAR=0x{:x}, PC=0x{:x}", 
          ec, ctx.esr.raw(), ctx.far, ctx.pc);
    
    // let exit_reason = match ec {
    //     ESR_EC_HVC64 => {
    //         info!("HVC call from guest: X0=0x{:x}", ctx.x[0]);
    //         handle_hvc_call(ctx)
    //     },
    //     ESR_EC_DATA_ABORT => {
    //         info!("Data abort: FAR=0x{:x}", ctx.far);
    //         ExitReason::DataAbort
    //     },
    //     ESR_EC_INST_ABORT => {
    //         info!("Instruction abort: FAR=0x{:x}", ctx.far);
    //         ExitReason::InstructionAbort
    //     },
    //     _ => {
    //         error!("Unknown exception class: 0x{:x}", ec);
    //         return VmExitAction::Stop;
    //     }
    // };
    
    // notify vm manager
    // if let Some(mut vm_manager) = VM_MANAGER.try_lock() {
    //     if let Some(ref mut vm) = *vm_manager {
    //         return vm.handle_vm_exit(0, exit_reason);
    //     }
    // }
    
    VmExitAction::Stop
}

fn handle_hvc_call(ctx: &mut ExceptionContext) -> ExitReason {
    // HVC id in reg x0
    match ctx.x[0] {
        0 => {
            // HVC 0: print string
            info!("Guest HVC: Print string at 0x{:x}", ctx.x[1]);
        },
        1 => {
            // HVC 1: exit vm
            info!("Guest HVC: Exit VM");
            return ExitReason::Hvc;
        },
        _ => {
            info!("Guest HVC: Unknown call 0x{:x}", ctx.x[0]);
        }
    }
    
    // pc += 4
    ctx.pc += 4;
    
    ExitReason::Hvc
}

pub fn handle_irq_exception(ctx: &mut ExceptionContext) -> VmExitAction {
    info!("IRQ exception at PC=0x{:x}", ctx.pc);
    
    //handle irq
    
    VmExitAction::Continue
}

pub fn setup_exception_handlers() {
    extern "C" {
        static exception_vector_table: u64;
    }
    
    unsafe {
        // setup exception vector table
        core::arch::asm!(
            "msr vbar_el2, {}",
            in(reg) &exception_vector_table as *const _ as u64
        );
        
        // synchronous instructions
        core::arch::asm!("isb");
    }
    
    info!("Exception handlers setup complete");
}