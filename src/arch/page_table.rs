use bitflags::bitflags;
use crate::mem::{MemFlags, PageTable};
bitflags! {
    pub struct PageTableFlags: u64 {
            // 表示PTE是否有效（1=有效，0=无效）。
            const VALID = 1;
            // 表示PTE是页描述符（1）还是块描述符（0）
            const TABLE = 1 << 1;
            // 表示页面是否被访问（硬件或软件维护）。
            const AF = 1 << 10; 
            // 表示是否使用连续页面优化（硬件支持）。
            const CONTIGUOUS = 1 << 52; 
            // 禁止用户模式执行（1表示不可执行）。
            const XN = 1 << 53; 
            // 表示页面是否属于非安全世界（用于TrustZone）。
            const NS = 1 << 63;
    }
}




const PHYS_ADDR_MASK:u64 = 0x0000_FFFF_FFFF_F000;
#[derive(Copy, Clone)]
pub struct PageTableEntry(u64);
#[allow(dead_code)]
impl PageTableEntry {
    pub fn empty() -> Self {
        PageTableEntry(0)
    }

    pub fn from_bits(bits: u64) -> Self {
        PageTableEntry(bits)
    }

    pub fn to_bits(self) -> u64 {
        self.0
    }

    pub fn set_valid(&mut self) {
        self.0 |= PageTableFlags::VALID.bits();
    }
    pub fn set_invalid(&mut self) {
        self.0 &= !PageTableFlags::VALID.bits();
    }
    pub fn is_valid(&self) -> bool{
        self.0 & PageTableFlags::VALID.bits() != 0
    }

    pub fn set_table(&mut self, page: bool) {
        if page {
            self.0 |= PageTableFlags::TABLE.bits();
        } else {
            self.0 &= !PageTableFlags::TABLE.bits();
        }
    }
    pub fn is_table(&mut self) -> bool {
        self.0 & PageTableFlags::TABLE.bits() != 0
    }

    pub fn set_accessed(&mut self) {
        self.0 |= PageTableFlags::AF.bits();
    }
    pub fn clear_accessed(&mut self) {
        self.0 &= !PageTableFlags::AF.bits();
    }

    pub fn set_ns(&mut self) {
        self.0 |= PageTableFlags::NS.bits();
    }
    pub fn clear_ns(&mut self) {
        self.0 &= !PageTableFlags::NS.bits();
    }
    pub fn is_ns(&self) -> bool {
        self.0 & PageTableFlags::NS.bits() != 0
    }

    pub fn set_flags(&mut self, flags: MemFlags) {
        self.0 |= flags.bits();
    }

    pub fn set_address(&mut self, paddr: usize) {
        self.0 = (self.0 & !PHYS_ADDR_MASK) | (paddr as u64 & PHYS_ADDR_MASK);
    }

    pub fn get_address(&self) -> usize {
        (self.0 & PHYS_ADDR_MASK) as usize
    }

    pub fn next(&mut self) -> Option<&mut PageTable> {
        if self.is_valid() {
            unsafe{ Some(&mut *(self.get_address() as *mut PageTable)) }
        } else {
            None
        }
    }
}


// VTCR_EL2 寄存器字段定义
pub const fn vtcr_t0sz(val: u64) -> u64 { val & 0x3F }
pub const fn vtcr_sl0(val: u64) -> u64 { (val & 0x3) << 6 }
pub const fn vtcr_sh0(val: u64) -> u64 { (val & 0x3) << 12 }
pub const fn vtcr_tg0(val: u64) -> u64 { (val & 0x3) << 14 }
pub const fn vtcr_ps(val: u64) -> u64 { (val & 0x7) << 16 }

pub const VTCR_NSW: u64 = 1 << 29;
pub const VTCR_NSA: u64 = 1 << 30;

// HCR_EL2 寄存器字段定义
pub const HCR_VM: u64 = 1 << 0;  // VM bit
pub const HCR_RW: u64 = 1 << 31; // RW bit

// Memory attribute indices
pub const DEVICE_NGNRNE_INDEX: u64 = 0;
pub const NORMAL_WB_INDEX: u64 = 1;

// Memory attribute values
pub const DEVICE_NGNRNE: u64 = 0x00;
pub const NORMAL_WB: u64 = 0xff;

pub const fn pa_range(feature: u64) -> u64 {
    feature & 0xF
}