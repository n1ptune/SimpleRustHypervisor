#![allow(unused)]

// GIC Redistributor registers
pub const GICR_BASE: usize = 0x080a0000;
pub const GICR_SIZE: usize = 0x80000; // 0x2000 for one core
pub const GICR_STRIDE: usize = 0x20000;

pub const GICR_CTLR: usize = 0;
pub const GICR_IIDR: usize = 0x4;
pub const GICR_TYPER: usize = 0x8;
pub const GICR_WAKER: usize = 0x14;
pub const GICR_PIDR2: usize = 0xffe8;

// SGI base is at 64K offset from Redistributor
pub const SGI_BASE: usize = 0x10000;

pub const GICR_IGROUPR0: usize = SGI_BASE + 0x80;
pub const GICR_ISENABLER0: usize = SGI_BASE + 0x100;
pub const GICR_ICENABLER0: usize = SGI_BASE + 0x180;
pub const GICR_ICPENDR0: usize = SGI_BASE + 0x280;
pub const GICR_ISACTIVER0: usize = SGI_BASE + 0x300;
pub const GICR_ICACTIVER0: usize = SGI_BASE + 0x380;
pub const GICR_ICFGR0: usize = SGI_BASE + 0xc00;
pub const GICR_ICFGR1: usize = SGI_BASE + 0xc04;
pub const GICR_IGRPMODR0: usize = SGI_BASE + 0xd00;

pub const fn gicr_ipriorityr(n: usize) -> usize {
    (SGI_BASE + 0x400 + n * 4) as usize
}

// Helper function to calculate GICR base address for a specific core
pub const fn gicr_base_n(coreid: usize) -> usize {
    GICR_BASE + coreid * GICR_STRIDE
}

// GIC Redistributor register access functions
pub fn gicr_read32(coreid: usize, offset: usize) -> u32 {
    let addr = (gicr_base_n(coreid) + offset as usize) as *const u32;
    unsafe { core::ptr::read_volatile(addr) }
}

pub fn gicr_write32(coreid: usize, offset: usize, val: u32) {
    let addr = (gicr_base_n(coreid) + offset as usize) as *mut u32;
    unsafe { core::ptr::write_volatile(addr, val) }
}

// GIC Redistributor 64-bit register access functions
pub fn gicr_read64(coreid: usize, offset: usize) -> u64 {
    let addr = (gicr_base_n(coreid) + offset as usize) as *const u64;
    unsafe { core::ptr::read_volatile(addr) }
}

pub fn gicr_write64(coreid: usize, offset: usize, val: u64) {
    let addr = (gicr_base_n(coreid) + offset as usize) as *mut u64;
    unsafe { core::ptr::write_volatile(addr, val) }
}