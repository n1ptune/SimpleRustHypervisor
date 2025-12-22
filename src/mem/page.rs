#![allow(dead_code)]

use crate::arch::*;
use crate::{read_sysreg, write_sysreg, isb};
use spin::Mutex;
use alloc::vec::Vec;
use super::frame::Frame;
use super::addr::{PhysAddr, VirtAddr, MemFlags, pt_idx};
use log::*;


#[repr(align(4096))]
pub struct PageTable {
    pub entries: [PageTableEntry; 512],
    //root: frame
    pub frames: Mutex<Vec<Frame>>,
}
impl PageTable{
    pub fn new() -> Self {
        PageTable {
            entries: [PageTableEntry::empty(); 512],
            frames: Mutex::new(Vec::new()),
        }
    }

    pub fn new_with_ptr(ptr:usize) -> Self {
        unsafe {
            let data = core::slice::from_raw_parts_mut(ptr as *mut PageTableEntry, 512);
            PageTable {
                entries: data.try_into().unwrap(),
                frames: Mutex::new(Vec::new()),
            }
        }
    }

    pub fn brrow_as_mut(&mut self) -> &mut PageTable {
        self
    }

    // walk page table
    pub fn walk(&mut self, phys:PhysAddr, virt: VirtAddr, mattr: MemFlags, alloc:bool) {
        let mut current_table: *mut PageTable = self;
        for i in 0..4 {
            let idx = pt_idx(virt, i);
            unsafe {
                let entry = &mut (*current_table).entries[idx];
                // info!("Walking page table entry {:x} at level {} {:x}", current_table as u64, i, entry.to_bits());
                
                if i == 3 && !entry.is_valid(){
                    
                    entry.set_valid();
                    entry.set_address(phys);
                    entry.set_table(true); 
                    entry.set_flags(mattr);
                    entry.set_accessed();
                    // info!("Creating new page table entry at level {} {:x} {:x} {:x}", i, current_table as u64, phys, entry.to_bits());
                    return;
                }
                // if i == 3 && entry.is_valid() { 
                //     info!("valid {:x}", entry.to_bits());
                // }
                if entry.is_valid() && entry.is_table() {
                    // info!("table entry, go to next");
                    current_table = entry.next().unwrap();
                } else if alloc {
                    
                    let addr = Frame::new_zero().unwrap();
                    let paddr = addr.start_paddr();
                    // info!("alloc {:x}", addr.start_paddr());
                    self.frames.lock().push(addr);
                    entry.set_address(paddr);
                    entry.set_valid();
                    entry.set_table(true);
                    // info!("entry {:x} {}", entry.to_bits(), entry.is_block());
                    current_table = entry.next().unwrap();
                } else {
                    error!("Page table entry not valid and allocation not allowed");
                }
            }
        }
        // info!("Walking page table failed");
    }

    // create virt:phys map
    pub fn create_map(&mut self, virt: VirtAddr, phys: PhysAddr, size: usize, mattr: MemFlags) { 
        for offset in (0..size).step_by(4096) {
            let virt_addr = virt + offset;
            let phys_addr = phys + offset;
            // isb!();
            // flush_tlb();
            // info!("Mapping virt: {:x} to phys: {:x}", virt_addr.as_usize(), phys_addr.as_usize());
            self.walk(phys_addr, virt_addr, mattr, true);
            flush_tlb_ipa_s2(virt_addr);
        }
    }
}
#[derive(Debug)]
pub struct PageTableRoot{
    root: Mutex<*mut PageTable>,
    root_frame:Frame
}

unsafe impl Send for PageTableRoot {}

impl PageTableRoot{
    pub fn new() -> Self {
        let root_frame = Frame::new_zero().unwrap();
        info!("page table root: {:x}", root_frame.start_paddr());
        return PageTableRoot{
            root: Mutex::new(PageTable::new_with_ptr(root_frame.start_paddr() as usize).brrow_as_mut()),
            root_frame
        }
    }

    pub fn get_root(&self) -> *mut PageTable {
        *self.root.lock()
    }

    pub fn create_map(&self, virt: VirtAddr, phys: PhysAddr, size: usize, mattr: MemFlags){
        unsafe {
            if let Some(page_table) = self.get_root().as_mut() {
                page_table.create_map(virt, phys, size, mattr);
        }
    }
    }

}

pub fn stage2_mmu_init() {
    info!("Stage2 Translation MMU initialization ...");

    // Physical Address range supported
    let feature = read_sysreg!(id_aa64mmfr0_el1);
    info!("PARange bits is {}", pa_range(feature));

    /*
     * T0SZ = 64 - 20 = 44 : (IPA range is 2^(64-20) = 2^44)
     * SL0  = 2 : starting level is level-0
     * TG0  = 0 : 4K Granule size
     * PS   = 4 : Physical address Size
     */
    let vtcr = vtcr_t0sz(20) | vtcr_sl0(2) |
               vtcr_sh0(0) | vtcr_tg0(0) | VTCR_NSW |
               VTCR_NSA | vtcr_ps(4);

    info!("Setting vtcr_el2 to 0x{:x}", vtcr);
    write_sysreg!(vtcr_el2, vtcr);

    let mair = (DEVICE_NGNRNE << (8 * DEVICE_NGNRNE_INDEX)) | 
               (NORMAL_WB << (8 * NORMAL_WB_INDEX));
    info!("Setting mair_el2 to 0x{:x}", mair);
    write_sysreg!(mair_el2, mair);

    isb!();
}

/* Provides configuration controls for virtualization */
pub fn hyper_setup() {
    let hcr = HCR_TSC | HCR_RW | HCR_FMO | HCR_IMO | HCR_VM;
    info!("Setting hcr_el2 to 0x{:x} and enable stage 2 address translation", hcr);
    write_sysreg!(hcr_el2, hcr);
    isb!();
}

//timer_sysctl_init
//free_area_init
//0x0000000040d45898