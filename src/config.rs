extern "C" {
    static SRH_END: usize;
}

pub const PAGE_SIZE:usize = 4096;
pub const HEAP_SIZE:usize = 256 * 1024 * 1024;
pub const PHYEND :usize =  0xC0000000;
lazy_static! {
    pub static ref HEAP_BASE: usize = {
        unsafe { &SRH_END as *const usize as usize }
    }; 
    pub static ref FRAME_BASE: usize = {
        *HEAP_BASE + HEAP_SIZE
    }; 
}

