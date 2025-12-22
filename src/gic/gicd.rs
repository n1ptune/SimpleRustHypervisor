#![allow(unused)]

// GIC Distributor registers
pub const GICD_BASE: usize = 0x08000000;
pub const GICD_SIZE: usize = 0x10000;

pub const GICD_CTLR: usize = 0x0;
pub const GICD_TYPER: usize = 0x4;
pub const GICD_IIDR: usize = 0x8;
pub const GICD_TYPER2: usize = 0xc;
pub const GICD_PIDR2: usize = 0xffe8;

pub const GICD_CTLR_ENABLE_G1A: u32 = 1 << 1;
pub const GICD_CTLR_ENABLE_G1: u32 = 1 << 0;
pub const GICD_CTLR_ARE_NA: u32 = 1 << 4;


// GIC Distributor register access functions
pub fn gicd_read32(offset: usize) -> u32 {
    let addr = (GICD_BASE + offset) as *const u32;
    unsafe { core::ptr::read_volatile(addr) }
}

pub fn gicd_write32(offset: usize, val: u32) {
    let addr = (GICD_BASE + offset) as *mut u32;
    unsafe { core::ptr::write_volatile(addr, val) }
}


// GICD register offsets
pub const fn gicd_igroupr(n: u64) -> usize {
    (0x080 + n * 4) as usize
}

pub const fn gicd_isenabler(n: u64) -> usize {
    (0x100 + n * 4) as usize
}

pub const fn gicd_icenabler(n: u64) -> usize {
    (0x180 + n * 4) as usize
}

pub const fn gicd_ispendr(n: u64) -> usize {
    (0x200 + n * 4) as usize
}

pub const fn gicd_icpendr(n: u64) -> usize {
    (0x280 + n * 4) as usize
}

pub const fn gicd_isactiver(n: u64) -> usize {
    (0x300 + n * 4) as usize
}

pub const fn gicd_icactiver(n: u64) -> usize {
    (0x380 + n * 4) as usize
}

pub const fn gicd_ipriorityr(n: u64) -> usize {
    (0x400 + n * 4) as usize
}

pub const fn gicd_itargetsr(n: u64) -> usize {
    (0x800 + n * 4) as usize
}

pub const fn gicd_icfgr(n: u64) -> usize {
    (0xc00 + n * 4) as usize
}

pub const fn gicd_irouter(n: u64) -> usize {
    (0x6000 + n * 8) as usize
}