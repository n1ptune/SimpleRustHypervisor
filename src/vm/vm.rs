use crate::mem::{Frame, MemFlags, PageTableRoot};
use crate::vm::MmioManager;
use crate::vm::vcpu::{Vcpu, VcpuState, ExitReason, VmExitAction};
use crate::write_sysreg;
use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::Mutex;
use log::*;
use crate::mem::page_count;
use crate::gic::VgicDist;
use crate::vdevices::VirtualUart;

extern "C" {
    pub static _binary_bin_guest_bin_start: usize;
    pub static _binary_bin_guest_bin_size: usize;
}

#[derive(Debug, Clone)]
pub struct GuestVMImage {
    pub name: &'static str,
    pub start: usize,
    pub size: usize,
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct VmConfig {
    pub guest_image: GuestVMImage,
    pub guest_dtb: usize,
    pub guest_initrd: usize,
    pub entry_addr: usize,
    pub memory_size: usize,
    pub vcpu_count: u32,
}

#[allow(unused)]
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
#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VmState {
    Created,
    Running,
    Paused,
    Stopped,
    Error,
}
#[allow(unused)]
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
    
    pub fn setup_memory(&mut self) -> Result<(), &'static str> {
        info!("Setting up guest memory for VM {}", self.id);
        
        // allocate physical memory for guest
        let guest_phys_base = &self.guest_memory_base;
        let guest_ipa_base = 0x40000000; // guest physical memory base
        
        info!("Guest memory base: ipa=0x{:x}, pa=0x{:x}", guest_ipa_base, guest_phys_base.start_paddr());

        self.root_page_table.create_map(guest_ipa_base,
                                    guest_phys_base.start_paddr(),
                                    self.config.memory_size,
                                    MemFlags::S2PTE_RW | MemFlags::S2PTE_NORMAL);
        
        info!("Guest memory mapped: ipa=0x{:x} -> pa=0x{:x}, size=0x{:x}", 
              guest_ipa_base, 
              guest_phys_base.start_paddr(), 
              self.config.memory_size);
        

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
                (guest_load_addr.start_paddr() + 0x200000) as *mut u8,
                image.size
            );
        }
        
        info!("Guest image '{}' loaded at 0x{:x}, size: 0x{:x}", 
              image.name, guest_load_addr.start_paddr(), image.size);
        
        Ok(())
    }
    

    pub fn start(&mut self) -> Result<(), &'static str> {
        if self.state != VmState::Created {
            return Err("VM is not in created state");
        }
        
        // setup memory
        self.setup_memory()?;
        
        // load guest image
        self.load_guest_image()?;

        self.vgic_dist.lock().init(&mut *self.mmio_manager.lock());
        
        // Set up stage 2 translation by writing the page table address to VTTBR_EL2
        let page_table_addr = self.root_page_table.get_root() as u64;
        info!("Setting VTTBR_EL2 to page table address: 0x{:x}", page_table_addr);
        write_sysreg!(vttbr_el2, page_table_addr);
        
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
    
    pub fn stop(&mut self) {
        self.state = VmState::Stopped;
        for vcpu in &mut self.vcpus {
            vcpu.state = VcpuState::Stopped;
        }
        info!("VM {} stopped", self.id);
    }
    
    pub fn handle_vm_exit(&mut self, vcpu_id: u32, exit_reason: ExitReason) -> VmExitAction {
        info!("VM {} VCPU {} exit: {:?}", self.id, vcpu_id, exit_reason);
        
        // 打印寄存器信息
        if let Some(vcpu) = self.vcpus.get(0) {
            info!("=== Guest VCPU Registers ===");
            info!("PC: 0x{:x}", vcpu.regs.elr);
            info!("ELR_EL1: 0x{:x}", vcpu.sysregs.elr_el1);
            // info!("SP_EL0: 0x{:x}", vcpu.regs.sp_el0);
            // info!("SP_EL1: 0x{:x}", vcpu.regs.sp_el1);
            info!("SPSR_EL1: 0x{:x}", vcpu.sysregs.spsr_el1);
            info!("ESR_EL1: 0x{:x}", vcpu.sysregs.esr_el1);
            info!("FAR_EL1: 0x{:x}", vcpu.sysregs.far_el1);
            info!("SCTLR_EL1: 0x{:x}", vcpu.sysregs.sctlr_el1);
            info!("TCR_EL1: 0x{:x}", vcpu.sysregs.tcr_el1);
            info!("TTBR0_EL1: 0x{:x}", vcpu.sysregs.ttbr0_el1);
            info!("TTBR1_EL1: 0x{:x}", vcpu.sysregs.ttbr1_el1);
            info!("MAIR_EL1: 0x{:x}", vcpu.sysregs.mair_el1);
            info!("VBAR_EL1: 0x{:x}", vcpu.sysregs.vbar_el1);
            
            // 打印通用寄存器 X0-X30
            for i in (0..31).step_by(4) {
                let x1 = if i+1 < 31 { vcpu.regs.x[i+1] } else { 0 };
                let x2 = if i+2 < 31 { vcpu.regs.x[i+2] } else { 0 };
                let x3 = if i+3 < 31 { vcpu.regs.x[i+3] } else { 0 };
                info!("X[{}]: 0x{:x}  X[{}]: 0x{:x}  X[{}]: 0x{:x}  X[{}]: 0x{:x}", 
                      i, vcpu.regs.x[i],
                      i+1, x1,
                      i+2, x2,
                      i+3, x3);
            }
            info!("============================");
        }
        
        if let Some(vcpu) = self.vcpus.get_mut(vcpu_id as usize) {
            vcpu.handle_exit(exit_reason)
        } else {
            VmExitAction::Stop
        }
    }
}

pub static VM_MANAGER: Mutex<Vec<Arc<Mutex<VirtualMachine>>>> = Mutex::new(Vec::new());
