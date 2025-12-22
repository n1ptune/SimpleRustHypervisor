use crate::arch::smc_call;

pub const SMCCC_VERSION: u64 = 0x80000000;
pub const SMCCC_ARCH_FEATURES: u64 = 0x80000001;
pub const SMCCC_ARCH_SOC_ID: u64 = 0x80000002;

pub fn vsmc_version() -> u64 {
    // unsafe { smc_call(SMCCC_VERSION, 0, 0) } // PSCI version 1.2
    0x00010001
}

pub fn vsmc_arch_features(x1: u64) -> u64 {
    unsafe { smc_call(SMCCC_ARCH_FEATURES, x1, 0) } // PSCI version 1.2
}

pub fn vsmc_arch_soc_id() -> u64 {
    unsafe { smc_call(SMCCC_ARCH_SOC_ID, 0, 0) } // PSCI version 1.2
}

pub fn handle_vsmc_call(function_id: u64, x1: u64) -> u64 {
    match function_id {
        SMCCC_VERSION => {
            vsmc_version()
        }
        SMCCC_ARCH_FEATURES => {
            vsmc_arch_features(x1)
        }
        SMCCC_ARCH_SOC_ID => {
            vsmc_arch_soc_id()
        }
        _ => {
            // Unsupported SMC call
            0xffffffff // Indicate error for unsupported function
        }
    }
}