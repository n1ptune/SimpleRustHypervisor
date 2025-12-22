use crate::arch::{sync_guest_memory};
use crate::mem::{Frame, MemFlags, PageTableRoot};
use crate::vm::MmioManager;
use crate::vm::guest::{GuestDtb, GuestInitrd, GuestVMImage};
use crate::vm::vcpu::{Vcpu, VcpuState};
use crate::write_sysreg;
use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::Mutex;
use log::*;
use crate::mem::page_count;
use crate::gic::VgicDist;
use crate::vdevices::VirtualUart;


#[derive(Debug, Clone)]
pub struct VmConfig {
    pub guest_image: GuestVMImage,
    pub guest_dtb: GuestDtb,
    pub guest_initrd: GuestInitrd,
    pub entry_addr: usize,
    pub memory_size: usize,
    pub vcpu_count: u32,
}


#[derive(Debug)]
pub struct VirtualMachine {
    pub id: u32,
    pub config: VmConfig,
    pub vcpus: Vec<Vcpu>,
    pub state: VmState,
    pub guest_memory_base: Frame,
    pub root_page_table: PageTableRoot,
    pub vgic_dist : Arc<Mutex<VgicDist>>,
    pub mmio_manager: Arc<Mutex<MmioManager>>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VmState {
    Created,
    Running,
    Stopped,
}

impl VirtualMachine {
    pub fn new(id: u32, config: VmConfig) -> Result<Self, &'static str> {
        let mut vcpus = Vec::with_capacity(config.vcpu_count as usize);
        let mmio_manager = Arc::new(Mutex::new(MmioManager::new()));
        let vgic_dist = Arc::new(Mutex::new(VgicDist::new()));
        // create vcpus
        for i in 0..config.vcpu_count {
            let vcpu = Vcpu::new(i as usize, id, config.entry_addr as u64, vgic_dist.clone(), mmio_manager.clone());
            vcpus.push(vcpu);
        }
        
        let memory_size = config.memory_size;
        if let Some(guest_memory_base) = Frame::new_contiguous(page_count(memory_size), 0){
            Ok(VirtualMachine {
            id,
            config,
            vcpus,
            state: VmState::Created,
            guest_memory_base, // default guest memory base
            root_page_table: PageTableRoot::new(),
            vgic_dist,
            mmio_manager,
        })
        } else {
            panic!("no mem");
        }
        
    }
    
    pub fn map_memory(&mut self) -> Result<(), &'static str> {
        info!("Setting up guest memory for VM {}", self.id);
        
        // allocate physical memory for guest
        let guest_phys_base = &self.guest_memory_base;
        let guest_ipa_base = 0x40000000; // guest physical memory base
        
        info!("Guest memory base: ipa=0x{:x}, pa=0x{:x}", guest_ipa_base, guest_phys_base.start_paddr());

        self.root_page_table.create_map(guest_ipa_base,
                                    guest_phys_base.start_paddr(),
                                    self.config.memory_size,
                                    MemFlags::S2PTE_RW | MemFlags::S2PTE_NORMAL);
        
        info!("Guest memory mapped: ipa=0x{:x} -> pa=0x{:x}, size=0x{:x} end=0x{:x}", 
              guest_ipa_base, 
              guest_phys_base.start_paddr(), 
              self.config.memory_size, 
              guest_ipa_base + self.config.memory_size);
        

        self.set_up_mmio()?;
        Ok(())
    }

    pub fn set_up_mmio(&mut self) -> Result<(), &'static str> {
        info!("Setting up MMIO for VM {}", self.id);

        let uart = VirtualUart::new(0x09000000);
        self.mmio_manager.lock().add_mmio_space(Box::new(uart));

        Ok(())
    }
    
    pub fn load_guest_image(&mut self) -> Result<(), &'static str> {
        info!("Loading guest image for VM {}", self.id);
        
        let image = &self.config.guest_image;
        let guest_load_addr = &self.guest_memory_base ;
        
        // copy guest image to allocated memory
        unsafe {
            core::ptr::copy_nonoverlapping(
                image.start as *const u8,
                (guest_load_addr.start_paddr()) as *mut u8,
                image.size
            );
            sync_guest_memory(guest_load_addr.start_paddr(), image.size);
        }

        info!("Guest image '{}' loaded at 0x{:x}, size: 0x{:x}", 
              image.name, guest_load_addr.start_paddr(), image.size);
        
        Ok(())
    }

    pub fn load_guest_dtb(&mut self) -> Result<(), &'static str> {
        info!("Loading guest dtb for VM {}", self.id);
        
        let dtb = &self.config.guest_dtb;
        let guest_load_addr = &self.guest_memory_base ;
        
        // copy guest dtb to allocated memory
        unsafe {
            core::ptr::copy_nonoverlapping(
                dtb.start as *const u8,
                (guest_load_addr.start_paddr() + 0x7800000) as *mut u8,
                dtb.size
            );
        }
        
        for vcpu in &mut self.vcpus {
            if vcpu.id == 0 {
                vcpu.regs.x[0] = 0x47800000;
            }
        }
        info!("Guest dtb '{}' loaded at 0x{:x}, size: 0x{:x}", 
              dtb.name, guest_load_addr.start_paddr() + 0x7800000, dtb.size);
        
        Ok(())
    }

    pub fn load_guest_initrd(&mut self) -> Result<(), &'static str> {
        info!("Loading guest initrd for VM {}", self.id);
        
        let initrd = &self.config.guest_initrd;
        let guest_load_addr = &self.guest_memory_base ;
        
        // copy guest initrd to allocated memory
        unsafe {
            core::ptr::copy_nonoverlapping(
                initrd.start as *const u8,
                (guest_load_addr.start_paddr() + 0x4000000) as *mut u8,
                initrd.size
            );
        }
        
        info!("Guest initrd '{}' loaded at 0x{:x}, size: 0x{:x}", 
              initrd.name, guest_load_addr.start_paddr() + 0x4000000, initrd.size);
        
        Ok(())
    }
    

    pub fn start(&mut self) -> Result<(), &'static str> {
        if self.state != VmState::Created {
            return Err("VM is not in created state");
        }
        
        // setup memory
        self.map_memory()?;
        
        // load guest image
        self.load_guest_image()?;
        self.load_guest_dtb()?;
        self.load_guest_initrd()?;

        self.vgic_dist.lock().init(&mut *self.mmio_manager.lock());
        
        // Set up stage 2 translation by writing the page table address to VTTBR_EL2
        // VTTBR_EL2 format: [63:48]=VMID, [47:1]=Physical base address (bits 50:2), [0]=CnP
        let page_table_paddr = self.root_page_table.get_root() as u64;
        let vmid_bits = (self.id as u64 & 0xFFFF) << 48;  // VMID in upper 16 bits
        let vttbr_val = vmid_bits | (page_table_paddr & 0xFFFF_FFFF_FFFF_F000);
        
        info!("Setting VTTBR_EL2: page_table_addr=0x{:x}, vmid={}, vttbr=0x{:x}", 
              page_table_paddr, self.id, vttbr_val);
        write_sysreg!(vttbr_el2, vttbr_val);
        
        // Invalidate TLB entries for the virtual address space
        unsafe {
            core::arch::asm!("tlbi alle1is");  // Invalidate all stage 2 TLB entries
            core::arch::asm!("dsb ish");        // Data barrier
            core::arch::asm!("isb");            // Instruction synchronization barrier
        }
        
        // start main vcpu
        if let Some(vcpu) = self.vcpus.get_mut(0) {
            info!("run vcpu {}", vcpu.id);
            vcpu.run()?;
        } else {
            return Err("No vCPUs available"); // Return an error if no vCPUs are available
        }
        
        self.state = VmState::Running;
        info!("VM {} started successfully", self.id);
        
        Ok(())
    }
    #[allow(unused)]
    pub fn stop(&mut self) {
        self.state = VmState::Stopped;
        for vcpu in &mut self.vcpus {
            vcpu.state = VcpuState::Stopped;
        }
        info!("VM {} stopped", self.id);
    }
    
}

pub static VM_MANAGER: Mutex<Vec<Arc<Mutex<VirtualMachine>>>> = Mutex::new(Vec::new());
