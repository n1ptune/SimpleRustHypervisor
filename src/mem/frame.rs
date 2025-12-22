#![allow(dead_code)]
use spin::Mutex;
use log::*;
use crate::{config::{FRAME_BASE, PHYEND}, mem::PhysAddr};
use bitmap_allocator::BitAlloc;
use super::addr::{align_up, page_count, is_aligned}; 
use crate::config::PAGE_SIZE;

// Support max 1M * 4096 = 4GB memory.
type FrameAlloc = bitmap_allocator::BitAlloc1M;

trait FrameAllocator {
    unsafe fn alloc(&mut self) -> Option<PhysAddr>;
    unsafe fn dealloc(&mut self, addr: PhysAddr);
    unsafe fn alloc_contiguous(&mut self, frame_count: usize, align_log2: usize) -> Option<PhysAddr>;
    unsafe fn dealloc_contiguous(&mut self, target: PhysAddr, frame_count: usize);
}

pub struct PhysManager{
    base: usize,
    allocator: FrameAlloc,

}

#[derive(Debug, Clone)]
pub struct Frame {
    start_paddr: PhysAddr,
    frame_count: usize,
}

impl PhysManager{
    pub fn init(&mut self){
            let base = align_up(*FRAME_BASE);
            let count = page_count(PHYEND - base);
            self.base = base;
            self.allocator.insert(0..count);
    }

    const fn new() -> Self {
        Self {
            base : 0, 
            allocator: FrameAlloc::DEFAULT
        }
    }

}

impl FrameAllocator for PhysManager {
    unsafe fn alloc(&mut self) -> Option<PhysAddr> { 
        if let Some(ppn) = self.allocator.alloc() {
            Some(ppn * PAGE_SIZE + self.base)
        } else {
            None
        }
    }

    unsafe fn alloc_contiguous(&mut self, frame_count: usize, align_log2: usize) -> Option<PhysAddr> { 
        let ret = self
            .allocator
            .alloc_contiguous(frame_count, align_log2)
            .map(|idx| idx * PAGE_SIZE + self.base);
        trace!(
            "Allocate {:x} frames with alignment {:x}: {:x?}",
            frame_count,
            1 << align_log2,
            ret
        );
        ret
    }

    unsafe fn dealloc(&mut self, addr: PhysAddr) {
        self.allocator.dealloc((addr - self.base) / PAGE_SIZE)
    }

    unsafe fn dealloc_contiguous(&mut self, target: PhysAddr, frame_count: usize) { 
        let start_idx = (target - self.base) / PAGE_SIZE;
        for i in start_idx..start_idx + frame_count {
            self.allocator.dealloc(i)
        }
    }

}

pub static  PHYS_MANAGER: Mutex<PhysManager> = Mutex::new(PhysManager::new());


#[allow(dead_code)]
impl Frame {
    /// Allocate one physical frame.
    pub fn new() -> Option<Self> {
        unsafe {
            PHYS_MANAGER
                .lock()
                .alloc()
                .map(|start_paddr| Self {
                    start_paddr,
                    frame_count: 1,
                })
        }
    }

    /// Allocate one physical frame and fill with zero.
    pub fn new_zero() -> Option<Self> {
        let mut f = Self::new()?;
        f.zero();
        Some(f)
    }

    /// Allocate contiguous physical frames.
    pub fn new_contiguous(frame_count: usize, align_log2: usize) -> Option<Self> {
        unsafe {
            PHYS_MANAGER
                .lock()
                .alloc_contiguous(frame_count, align_log2)
                .map(|start_paddr| Self {
                    start_paddr,
                    frame_count,
                })
        }
    }

    /// Constructs a frame from a raw physical address without automatically calling the destructor.
    ///
    /// # Safety
    ///
    /// This function is unsafe because the user must ensure that this is an available physical
    /// frame.
    pub unsafe fn from_paddr(start_paddr: PhysAddr) -> Self {
        assert!(is_aligned(start_paddr));
        Self {
            start_paddr,
            frame_count: 0,
        }
    }

    /// Get the start physical address of this frame.
    pub fn start_paddr(&self) -> PhysAddr {
        self.start_paddr
    }

    /// Get the total size (in bytes) of this frame.
    pub fn size(&self) -> usize {
        self.frame_count * PAGE_SIZE
    }

    /// convert to raw a pointer.
    pub fn as_ptr(&self) -> *const u8 {
        self.start_paddr as *const u8
    }

    /// convert to a mutable raw pointer.
    pub fn as_mut_ptr(&self) -> *mut u8 {
        self.start_paddr as *mut u8
    }

    /// Fill `self` with `byte`.
    pub fn fill(&mut self, byte: u8) {
        unsafe { core::ptr::write_bytes(self.as_mut_ptr(), byte, self.size()) }
    }

    /// Fill `self` with zero.
    pub fn zero(&mut self) {
        self.fill(0)
    }

    /// Forms a slice that can read data.
    pub fn as_slice(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.as_ptr(), self.size()) }
    }

    /// Forms a mutable slice that can write data.
    pub fn as_slice_mut(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self.as_mut_ptr(), self.size()) }
    }
}

impl Drop for Frame {
    fn drop(&mut self) {
        unsafe {
            match self.frame_count {
                0 => {} // Do not deallocate when use Frame::from_paddr()
                1 => PHYS_MANAGER.lock().dealloc(self.start_paddr),
                _ => PHYS_MANAGER
                    .lock()
                    .dealloc_contiguous(self.start_paddr, self.frame_count),
            }
        }
    }
}

pub fn init_frame_allocator() {
    let mut pm = PHYS_MANAGER.lock();
    pm.init();
    info!("Page allocator initialized at {:#x} end to {:#x}", *FRAME_BASE, PHYEND);
}