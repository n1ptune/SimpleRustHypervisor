use log::{info, warn};

use crate::{arch::smc_call, vm::Vcpu};

pub const SMC32_PSCI_FID_MIN: u64 = 0x84000000;
pub const SMC32_PSCI_FID_MAX: u64 = 0x84000014;

pub const SMC64_PSCI_FID_MIN: u64 = 0xC4000000;
pub const SMC64_PSCI_FID_MAX: u64 = 0xC4000014;

pub const PSCI_VERSION: u64 = 0x84000000;
pub const PSCI_MIGRATE_INFO_TYPE: u64 = 0x84000006;
pub const PSCI_SYSTEM_OFF: u64 = 0x84000008;
pub const PSCI_SYSTEM_RESET: u64 = 0x84000009;
pub const PSCI_SYSTEM_CPUON: u64 = 0xc4000003;
pub const PSCI_FEATURE: u64 = 0x8400000a;

pub fn vpsci_version() -> u64 {
    unsafe { smc_call(PSCI_VERSION, 0, 0) } // PSCI version 1.2
}

pub fn vpsci_migrate_info_type() -> u64 {
    unsafe { smc_call(PSCI_MIGRATE_INFO_TYPE, 0, 0) }
}
//这里要改，但是目前没有用到先放下
pub fn vpsci_cpu_on(_vcpu: &mut Vcpu, target_cpu: u64, entry_point: u64) -> u64 {
    unsafe { smc_call(PSCI_SYSTEM_CPUON, target_cpu, entry_point) }
}

pub fn handle_psci_call(vcpu: &mut Vcpu, function_id: u64, target_cpu: u64, entry_point: u64) -> u64 {
    match function_id {
        PSCI_VERSION => {
            vpsci_version()
        }
        PSCI_MIGRATE_INFO_TYPE => {
            vpsci_migrate_info_type()
        }
        PSCI_SYSTEM_OFF => {
            warn!("PSCI_SYSTEM_OFF called - shutting down the system");
            // Here you would add code to power off the system
            0 // Indicate success
        }
        PSCI_SYSTEM_RESET => {
            warn!("PSCI_SYSTEM_RESET called - resetting the system");
            // Here you would add code to reset the system
            0 // Indicate success
        }
        PSCI_SYSTEM_CPUON => {
            vpsci_cpu_on(vcpu, target_cpu, entry_point)
        }
        PSCI_FEATURE => {
            0 // Indicate feature not supported
        }
        _ => {
            info!("Unsupported PSCI function ID: 0x{:x}", function_id);
            0xffffffff // Indicate error for unsupported function
        }
    }
}