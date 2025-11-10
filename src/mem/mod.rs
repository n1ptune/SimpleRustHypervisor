mod frame;
mod heap;
mod page;
mod addr;

pub use addr::*;
pub use page::*;
pub use frame::Frame;

pub fn init() {
    heap::init_heap();
    frame::init_frame_allocator();
}