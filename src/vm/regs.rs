// pub trait REGS{

// }
#![allow(unused)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Ec {
    Unknown        = 0x00,       // Unknown reason
    Wfx            = 0x01,       // WF* instruction
    McrMrc15       = 0x03,       // MCR or MRC access, coproc==0b1111
    McrrMrrc15     = 0x04,       // MCRR or MRRC access, coproc==0b1111
    McrMrc14       = 0x05,       // MCR or MRC access, coproc==0b1110
    LdcStc         = 0x06,       // LDC or STC instruction
    Fpu            = 0x07,       // SVE/Advanced SIMD/FP not implemented
    McrMrc7        = 0x08,       // MCR or MRC access, coproc==0b0111
    PAuth          = 0x09,       // Pointer authentication instruction
    Other          = 0x0A,       // Not covered by other EC values
    /* 0x0B Reserved */
    McrrMrrc14     = 0x0C,       // MCRR or MRRC access, coproc==0b1110
    Bte            = 0x0D,       // Branch Target Exception
    Ie             = 0x0E,       // Illegal Execution state

    /* AArch32 exceptions (only when FEAT_AA32 implemented) */
    Svc32          = 0x11,       // SVC instruction execution in AArch32 state
    Hvc32          = 0x12,       // HVC instruction execution in AArch32 state
    Smc32          = 0x13,       // SMC instruction execution in AArch32 state

    /* AArch64 system/service calls */
    MsrrMrrs       = 0x14,       // MSRR/MRRS or 128-bit System instruction
    Svc            = 0x15,       // SVC instruction execution in AArch64 state
    Hvc            = 0x16,       // HVC instruction execution in AArch64 state
    Smc            = 0x17,       // SMC instruction execution in AArch64 state
    SysReg         = 0x18,       // Trapped MSR/MRS/System instruction
    Sve            = 0x19,       // Access to SVE trapped
    Eret           = 0x1A,       // ERET/ERETAA/ERETAB instruction
    Tstart         = 0x1B,       // TSTART instruction when TME disabled
    PacFail        = 0x1C,       // PAC Fail exception
    Sme            = 0x1D,       // Access to SME trapped
    /* 0x1E-0x1F Reserved */

    InstAbort      = 0x20,       // Instruction Abort from lower EL
    InstAbortSame  = 0x21,       // Instruction Abort without change in EL
    PcAlign        = 0x22,       // PC alignment fault
    /* 0x23 Reserved */
    DataAbort      = 0x24,       // Data Abort from lower EL
    DataAbortSame  = 0x25,       // Data Abort without change in EL
    SpAlign        = 0x26,       // SP alignment fault
    MemOp          = 0x27,       // Memory Operation Exception (FEAT_MOPS)
    Fpe32          = 0x28,       // Trapped floating-point exception from AArch32
    /* 0x29-0x2B Reserved */
    Fpe64          = 0x2C,       // Trapped floating-point exception from AArch64
    SError         = 0x2F,       // SError interrupt

    /* Debug breakpoints / watchpoints */
    Bkpt32         = 0x38,       // BKPT instruction execution in AArch32 state
    /* 0x39 Reserved */
    Vector32       = 0x3A,       // Vector Catch exception from AArch32 state
    /* 0x3B Reserved */
    Brk64          = 0x3C,       // BRK instruction execution in AArch64 state
    Profiling      = 0x3D,       // Profiling exception (FEAT_EBEP)
    /* 0x3E-0x3F Reserved */
}

impl Ec {
    pub const fn from_u8(val: u8) -> Self {
        use Ec::*;
        match val {
            0x00 => Unknown,
            0x01 => Wfx,
            0x03 => McrMrc15,
            0x04 => McrrMrrc15,
            0x05 => McrMrc14,
            0x06 => LdcStc,
            0x07 => Fpu,
            0x08 => McrMrc7,
            0x09 => PAuth,
            0x0A => Other,
            0x0C => McrrMrrc14,
            0x0D => Bte,
            0x0E => Ie,
            0x11 => Svc32,
            0x12 => Hvc32,
            0x13 => Smc32,
            0x14 => MsrrMrrs,
            0x15 => Svc,
            0x16 => Hvc,
            0x17 => Smc,
            0x18 => SysReg,
            0x19 => Sve,
            0x1A => Eret,
            0x1B => Tstart,
            0x1C => PacFail,
            0x1D => Sme,
            0x20 => InstAbort,
            0x21 => InstAbortSame,
            0x22 => PcAlign,
            0x24 => DataAbort,
            0x25 => DataAbortSame,
            0x26 => SpAlign,
            0x27 => MemOp,
            0x28 => Fpe32,
            0x2C => Fpe64,
            0x2F => SError,
            0x38 => Bkpt32,
            0x3A => Vector32,
            0x3C => Brk64,
            0x3D => Profiling,
            v => Other,
        }
    }
}
#[derive(Debug, Clone, Copy)]
pub struct EsrEl2(u64);

/*
#define ESR_ELx_SRT_SHIFT	(16)
#define ESR_ELx_SRT_MASK	(UL(0x1F) << ESR_ELx_SRT_SHIFT)
#define ESR_ELx_SAS_SHIFT	(22)
#define ESR_ELx_SAS		(UL(3) << ESR_ELx_SAS_SHIFT)
#define ESR_ELx_WNR_SHIFT	(6)
#define ESR_ELx_WNR		(UL(1) << ESR_ELx_WNR_SHIFT)

#define ESR_ELx_EC_SHIFT	(26)
#define ESR_ELx_EC_WIDTH	(6)
#define ESR_ELx_EC_MASK		(UL(0x3F) << ESR_ELx_EC_SHIFT)
#define ESR_ELx_EC(esr)		(((esr) & ESR_ELx_EC_MASK) >> ESR_ELx_EC_SHIFT)

#define ESR_ELx_ISS_MASK	(GENMASK(24, 0))
#define ESR_ELx_ISS(esr)	((esr) & ESR_ELx_ISS_MASK)

*/
const fn genmask(high: u32, low: u32) -> u64 {
    if high >= low {
        ((1u64 << (high - low + 1)) - 1) << low
    } else {
        0
    }
}

const ESR_ELX_SRT_SHIFT: u64 = 16;
const ESR_ELX_SRT_MASK: u64 = 0x1F << ESR_ELX_SRT_SHIFT;

const ESR_ELX_SAS_SHIFT: u64 = 22;
const ESR_ELX_SAS: u64 = 0x3 << ESR_ELX_SAS_SHIFT;

const ESR_ELX_WNR_SHIFT: u64 = 6;
const ESR_ELX_WNR: u64 = 0x1 << ESR_ELX_WNR_SHIFT;

const ESR_ELX_EC_SHIFT: u64 = 26;
const ESR_ELX_EC_WIDTH: u64 = 6;
const ESR_ELX_EC_MASK: u64 = 0x3F << ESR_ELX_EC_SHIFT;

const ESR_ELX_ISS_MASK: u64 = genmask(24, 0);


impl EsrEl2{
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }
    
    pub const fn raw(self) -> u64 {
        self.0
    }

    pub const fn ec(self) -> u8 {
        ((self.0 & ESR_ELX_EC_MASK) >> ESR_ELX_EC_SHIFT) as u8
    }

    pub const fn iss(self) -> u32 {
        (self.0 & ESR_ELX_ISS_MASK) as u32
    }

    pub const fn is_write(self) -> Option<bool> {
        match Ec::from_u8(self.ec()) {
            Ec::InstAbort | Ec::DataAbort => Some(((self.iss() >> ESR_ELX_WNR_SHIFT) & 1) != 0),
            _ => None,
        }
    }

    pub const fn srt(self) -> u64 {
        (self.0 & ESR_ELX_SRT_MASK) >> ESR_ELX_SRT_SHIFT
    }

    pub const fn sas(self) -> u8 {
        1 << ((self.0 & ESR_ELX_SAS) >> ESR_ELX_SAS_SHIFT)
    }


    pub const fn dfsc(self) -> Option<u8> {
        match Ec::from_u8(self.ec()) {
            Ec::InstAbort | Ec::DataAbort => Some((self.iss() & 0x3F) as u8),
            _ => None,
        }
    }

    pub const fn is_data_abort(self) -> bool {
        matches!(Ec::from_u8(self.ec()), Ec::DataAbort)
    }

    pub const fn is_inst_abort(self) -> bool {
        matches!(Ec::from_u8(self.ec()), Ec::InstAbort)
    }
}