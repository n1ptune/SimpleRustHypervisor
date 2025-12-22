#![allow(unused)]
use core::arch::asm;

use crate::exception::psci::{SMC32_PSCI_FID_MAX, SMC32_PSCI_FID_MIN, SMC64_PSCI_FID_MAX, SMC64_PSCI_FID_MIN, handle_psci_call};
use crate::exception::smc::{SMCCC_ARCH_FEATURES, SMCCC_ARCH_SOC_ID, SMCCC_VERSION, handle_vsmc_call};
use crate::read_sysreg;
use crate::vm::{AccessSize, MmioAccess, VM_MANAGER, Vcpu};
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
        debug!("============================");
        debug!("Exception Context:");
        for i in (0..30).step_by(2) {
            debug!("x{}: 0x{:x}, x{}: 0x{:x}", i, ctx.regs.x[i], i + 1, ctx.regs.x[i + 1]);
        }
        debug!("x{}: 0x{:x}", 30, ctx.regs.x[30]);

        let far = read_sysreg!(FAR_EL2);
        debug!("ESR_EL2: 0x{:x}, FAR_EL2: 0x{:x}", esr.raw(), far); // ctx.regs.esr.raw(), ctx.regs.far);
        debug!("ELR_EL2: 0x{:x}, SPSR_EL2: 0x{:x}", ctx.regs.elr, ctx.regs.spsr);
        debug!("SP_EL1: 0x{:x}", ctx.regs.sp_el1);
        debug!("============================");
    }

}
#[no_mangle]
pub extern "C" fn handle_sync_exception_from_asm() {
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
            error!("Exception Class: 0x{:x}", esr.ec());
        }
    }
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
    error!("unknown data abort at IPA=0x{:x}", ipa);
}

pub fn handle_insn_abort(vcpu: &mut Vcpu){
    debug!("handle_insn_abort");
    panic!();
}

fn handle_smc_call(vcpu: &mut Vcpu, esr: EsrEl2) {

    let function_id = vcpu.regs.x[0];

    match function_id {
        SMCCC_VERSION | SMCCC_ARCH_FEATURES | SMCCC_ARCH_SOC_ID => {
            vcpu.regs.x[0] = handle_vsmc_call(function_id, vcpu.regs.x[1]); // Example version 1.1
        },
        id if (id >= SMC32_PSCI_FID_MIN && id <= SMC32_PSCI_FID_MAX) ||
              (id >= SMC64_PSCI_FID_MIN && id <= SMC64_PSCI_FID_MAX) => {
            vcpu.regs.x[0] = handle_psci_call(vcpu, function_id, vcpu.regs.x[1], vcpu.regs.x[2]); // PSCI_SUCCESS
        },
        _ => {
            error!("Unknown SMC/HVC function id: 0x{:x}", function_id);
            vcpu.regs.x[0] = 0xffffffff; 
        }
    }

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