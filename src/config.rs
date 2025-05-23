extern "C" {
    static SRH_END: usize;
}

pub fn heap_base() -> usize {
    unsafe { &SRH_END as *const usize as usize }
}

pub const HEAP_SIZE:usize = 10 * 1024 * 1024;

pub fn frame_base() -> usize {
    heap_base() + HEAP_SIZE
}


pub const PHYBASE :usize =  0x40000000;
pub const PHYSIZE :usize =  512 * 1024 * 1024;
pub const PHYEND :usize =  PHYBASE + PHYSIZE;