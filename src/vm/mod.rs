mod guest;
mod mmio;
mod regs;
mod vcpu;
mod vm;

use crate::vm::guest::{GUEST_DTB, GUEST_IMAGE, GUEST_INITRD, GuestDtb, GuestInitrd, GuestVMImage};
use alloc::sync::Arc;
pub use mmio::*;
pub use regs::{Ec, EsrEl2};
use spin::Mutex;
pub use vcpu::{Vcpu};
pub use vm::*;

pub use super::exception::setup_exception_handlers;
use log::*;

pub fn run() -> Result<(), &'static str> {
    info!("Starting Guest VM...");

    // initialize exception handlers
    setup_exception_handlers();

    let vm_config = VmConfig {
        guest_image: GuestVMImage {
            name: "Guest VM",
            start: GUEST_IMAGE.as_ptr() as usize,
            end: GUEST_IMAGE.as_ptr() as usize + GUEST_IMAGE.len(),
            size: GUEST_IMAGE.len(),
        },
        guest_dtb: GuestDtb {
            name: "Guest DTB",
            start: GUEST_DTB.as_ptr() as usize,
            end: GUEST_DTB.as_ptr() as usize + GUEST_DTB.len(),
            size: GUEST_DTB.len(),
        },
        guest_initrd: GuestInitrd {
            name: "Guest Initrd",
            start: GUEST_INITRD.as_ptr() as usize,
            end: GUEST_INITRD.as_ptr() as usize + GUEST_INITRD.len(),
            size: GUEST_INITRD.len(),
        },
        entry_addr: 0x40200000,
        memory_size: 128 * 1024 * 1024 + 2 * 1024 * 1024, // 128MB + 2MB for dtb
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
