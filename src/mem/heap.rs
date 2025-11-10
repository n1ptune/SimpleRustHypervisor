
use buddy_system_allocator::LockedHeap;
use crate::config::{HEAP_BASE, HEAP_SIZE};
use log::*;
#[global_allocator]
/// heap allocator instance
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();



pub fn init_heap() {
    unsafe {
        HEAP_ALLOCATOR
            .lock()
            .init(*HEAP_BASE, HEAP_SIZE);
    }
    info!("Heap initialized at {:#x} with size {:#x}", *HEAP_BASE, HEAP_SIZE);
}