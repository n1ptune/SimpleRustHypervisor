
use buddy_system_allocator::LockedHeap;
use crate::config::{heap_base, HEAP_SIZE};
#[global_allocator]
/// heap allocator instance
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();



pub fn init_heap() {
    unsafe {
        HEAP_ALLOCATOR
            .lock()
            .init(heap_base(), HEAP_SIZE);
    }
    println!("Heap initialized at {:#x} with size {:#x}", heap_base(), HEAP_SIZE);
}