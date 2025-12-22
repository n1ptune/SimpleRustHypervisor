#![allow(unused)]
use core::arch::asm;

use crate::exception::psci::{SMC32_PSCI_FID_MAX, SMC32_PSCI_FID_MIN, SMC64_PSCI_FID_MAX, SMC64_PSCI_FID_MIN, handle_psci_call};
use crate::exception::smc::{SMCCC_ARCH_FEATURES, SMCCC_ARCH_SOC_ID, SMCCC_VERSION, handle_vsmc_call};
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

pub fn dump_regs(ctx: &mut Vcpu, esr: EsrEl2){
    unsafe {
        info!("============================");
        info!("Exception Context:");
        for i in (0..30).step_by(2) {
            info!("x{}: 0x{:x}, x{}: 0x{:x}", i, ctx.regs.x[i], i + 1, ctx.regs.x[i + 1]);
        }
        info!("x{}: 0x{:x}", 30, ctx.regs.x[30]);

        let far = read_sysreg!(FAR_EL2);
        info!("ESR_EL2: 0x{:x}, FAR_EL2: 0x{:x}", esr.raw(), far); // ctx.regs.esr.raw(), ctx.regs.far);
        info!("ELR_EL2: 0x{:x}, SPSR_EL2: 0x{:x}", ctx.regs.elr, ctx.regs.spsr);
        info!("SP_EL1: 0x{:x}", ctx.regs.sp_el1);
        info!("============================");
    }

}
#[no_mangle]
pub extern "C" fn handle_sync_exception_from_asm() {
    // dump_regs(ctx);
    // panic!("vm exit");
    // dispatch ec
    // info!("handle_sync_exception_from_asm");

    let vcpu_ptr = get_current_vcpu_ptr();

    if vcpu_ptr.is_null() {
        panic!("Critical: Exception in hypervisor context!");
    }

    let vcpu_ref = unsafe { &mut *vcpu_ptr };

    
    let esr = EsrEl2::new(read_sysreg!(ESR_EL2));

    match Ec::from_u8(esr.ec()) {
        Ec::DataAbort => handle_data_abort(vcpu_ref, esr),
        Ec::InstAbort => handle_insn_abort(vcpu_ref),
        Ec::Hvc       => handle_smc_call(vcpu_ref, esr),
        Ec::Smc       => handle_smc_call(vcpu_ref, esr),
        _ => {
            dump_regs(vcpu_ref, esr);
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


    let ipa = (read_sysreg!(HPFAR_EL2) << 8) | (read_sysreg!(FAR_EL2) & 0xfff);

    let access = MmioAccess {
                            ipa: ipa as usize, 
                            pc: vcpu.regs.elr as usize, 
                            wnr: wnr, 
                            access_size: AccessSize::from_size(sas).unwrap() 
                        };


    if mmio_manager.lock().handle_mmio(vcpu, srt, access){
        vcpu.regs.elr += 4;
        return;
    }
    info!("unknown data abort at IPA=0x{:x}", ipa);
}

pub fn handle_insn_abort(vcpu: &mut Vcpu){
    debug!("handle_insn_abort");
    panic!();
}

pub fn handle_sync_exception(ctx: &mut ExceptionContext) -> VmExitAction {
    let ec = ctx.esr.ec();
    
    info!("Sync exception: EC=0x{:x}, ESR=0x{:x}, FAR=0x{:x}, PC=0x{:x}", 
          ec, ctx.esr.raw(), ctx.far, ctx.pc);
    VmExitAction::Stop
}

fn handle_smc_call(vcpu: &mut Vcpu, esr: EsrEl2) {
    //  id in reg x0
    let function_id = vcpu.regs.x[0];
    // info!("SMC/HVC call with function id: 0x{:x}", function_id);
    match function_id {
        SMCCC_VERSION | SMCCC_ARCH_FEATURES | SMCCC_ARCH_SOC_ID => {
            // Handle SMCCC version call
            // info!("Handling SMCCC_VERSION call");
            vcpu.regs.x[0] = handle_vsmc_call(function_id, vcpu.regs.x[1]); // Example version 1.1
        },
        id if (id >= SMC32_PSCI_FID_MIN && id <= SMC32_PSCI_FID_MAX) ||
              (id >= SMC64_PSCI_FID_MIN && id <= SMC64_PSCI_FID_MAX) => {
            // Handle PSCI calls
            // info!("Handling PSCI call: 0x{:x}", function_id);
            // For simplicity, just return success for known PSCI calls
            vcpu.regs.x[0] = handle_psci_call(vcpu, function_id, vcpu.regs.x[1], vcpu.regs.x[2]); // PSCI_SUCCESS
        },
        _ => {
            info!("Unknown SMC/HVC function id: 0x{:x}", function_id);
            vcpu.regs.x[0] = 0xffffffff; // Indicate failure or unknown function
        }
    }

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