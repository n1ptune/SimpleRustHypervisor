mod vm;
mod vcpu;
mod guest;
mod regs;
mod mmio;

use alloc::sync::Arc;
pub use mmio::*;
use spin::Mutex;
pub use vm::*;
pub use vcpu::{ExitReason, VmExitAction, Vcpu};
pub use regs::{EsrEl2, Ec};
use crate::vm::guest::{GUEST_DTB, GuestDtb, GuestVMImage};

pub use super::exception::setup_exception_handlers;
use log::*;

pub fn run() -> Result<(), &'static str> {
    info!("Starting Guest VM...");
    
    // initialize exception handlers
    setup_exception_handlers();
    
    
    let vm_config = VmConfig {
        guest_image: GuestVMImage {
            name: "Guest VM",
            start: unsafe { &_binary_bin_guest_bin_start as *const _ as usize },
            end: unsafe { &_binary_bin_guest_bin_end as *const _ as usize },
            size: unsafe { &_binary_bin_guest_bin_size as *const _ as usize },
        },
        guest_dtb: GuestDtb { name: "Guest DTB", start: GUEST_DTB.as_ptr() as usize, end: GUEST_DTB.as_ptr() as usize + GUEST_DTB.len(), size: GUEST_DTB.len() },
        guest_initrd: 0,
        entry_addr: 0x40200000,
        memory_size: 128 * 1024 * 1024, // 128MB
        vcpu_count: 1,
    };
    
    // create virtual machine
    info!("Creating virtual machine...");
    let vm = VirtualMachine::new(1, vm_config)?;
    let vm_shared = Arc::new(Mutex::new(vm));
    
    {
        let mut manager = VM_MANAGER.lock();
        // 这里 clone 的是 Arc 指针，代价极小，仅仅是引用计数 +1
        // 管理器现在持有了一份指向该 VM 的指针
        manager.push(vm_shared.clone()); 
    }
    
    // start virtual machine
    info!("Starting virtual machine...");
    let mut vm_guard = vm_shared.lock();
    vm_guard.start()?;
    
    // store the virtual machine in the global manager
    
    info!("Guest VM started successfully");
    Ok(())
}