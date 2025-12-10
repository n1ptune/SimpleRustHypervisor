
#![allow(dead_code)]
use bitflags::bitflags;
use crate::{arch::{DEVICE_NGNRNE_INDEX, NORMAL_WB_INDEX}, config::PAGE_SIZE};

pub type VirtAddr = usize;
pub type PhysAddr = usize;

// pub type GuestVirtAddr = usize;
// pub type GuestPhysAddr = usize;

pub const fn s2pte_attr(attr: u64) -> u64 {
    (attr & 7) << 2
}

bitflags! {
    pub struct MemFlags: u64 {
        const S2PTE_MASK = 3 << 6;
        const S2PTE_RO = 1 << 6;   // 只读
        const S2PTE_WO = 2 << 6;   // 只写
        const S2PTE_RW  = 3 << 6;   // 读写

        const S2PTE_NORMAL = s2pte_attr(NORMAL_WB_INDEX);
        const S2PTE_DEVICE = s2pte_attr(DEVICE_NGNRNE_INDEX);
    }
}


pub const fn align_down(addr: usize) -> usize {
    addr & !(PAGE_SIZE - 1)
}

pub const fn align_up(addr: usize) -> usize {
    (addr + PAGE_SIZE - 1) & !(PAGE_SIZE - 1)
}

pub const fn is_aligned(addr: usize) -> bool {
    page_offset(addr) == 0
}

pub const fn page_count(size: usize) -> usize {
    align_up(size) / PAGE_SIZE
}

pub const fn page_offset(addr: usize) -> usize {
    addr & (PAGE_SIZE - 1)
}

pub const fn pt_idx(addr: usize, level:usize) -> usize {
    (addr >> (39 - (level * 9))) & 0x1FF 
}