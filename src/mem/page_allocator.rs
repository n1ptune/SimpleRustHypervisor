#![allow(dead_code)]
use alloc::vec::Vec;
use spin::Mutex;
use lazy_static::lazy_static;
use crate::config::{frame_base, PHYEND};

trait FrameAllocator {
    fn new() -> Self;
    fn alloc(&mut self) -> Option<usize>;
    fn dealloc(&mut self, ppn: usize);
}

pub struct PhysManager{
    used: Vec<usize>,
    free: Vec<usize>,
}
impl PhysManager{
    pub fn init(&mut self){
            for addr in (frame_base()..PHYEND).step_by(4096) {
                self.free.push(addr);
            }
    }
}
impl FrameAllocator for PhysManager {
    fn new() -> Self {
        let used: Vec<usize> = Vec::new();
        let free = Vec::new();
        
        PhysManager { used, free,  }
    }

    fn alloc(&mut self) -> Option<usize> {
        if self.free.is_empty() {
            None
        } else {
            let ppn = self.free.pop().unwrap();
            self.used.push(ppn);
            Some(ppn)
        }
    }

    fn dealloc(&mut self, ppn: usize) {
        if let Some(pos) = self.used.iter().position(|&x| x == ppn) {
            self.used.remove(pos);
            unsafe {
                core::ptr::write_bytes(ppn as *mut u8, 0, 4096);
            }
            self.free.push(ppn);
        }
    }
}

lazy_static! {
    pub static ref PHYS_MANAGER: Mutex<PhysManager> = Mutex::new(PhysManager::new());
}

pub fn alloc_page() -> Option<usize> {
    let mut pm = PHYS_MANAGER.lock();
    pm.alloc()
}
pub fn free_page(ppn: usize) {
    let mut pm = PHYS_MANAGER.lock();
    pm.dealloc(ppn);
}


pub fn init_page_allocator() {
    let mut pm = PHYS_MANAGER.lock();
    pm.init();
    println!("Page allocator initialized at {:#x} with size {:#x}", frame_base(), PHYEND);
}