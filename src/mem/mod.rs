mod page_allocator;
mod heap_allocator;

pub fn init() {
    heap_allocator::init_heap();
    page_allocator::init_page_allocator();
}