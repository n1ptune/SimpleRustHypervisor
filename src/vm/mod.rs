mod vm;
mod vcpu;
mod exception_handler;
mod devices;
mod regs;

pub use vm::*;
pub use vcpu::*;
pub use exception_handler::*;
pub use devices::init_virtual_devices;
use log::*;

core::arch::global_asm!(include_str!("exceptions.asm"));

#[no_mangle]
extern "C" fn handle_exception() {
    panic!("Unexpected exception occurred!");
}

pub fn run() -> Result<(), &'static str> {
    info!("Starting Guest VM...");
    
    // initialize exception handlers
    setup_exception_handlers();
    
    // initialize virtual devices
    init_virtual_devices()?;
    
    let vm_config = VmConfig {
        guest_image: GuestVMImage {
            name: "Guest VM",
            start: unsafe { &_binary_bin_guest_bin_start as *const _ as usize },
            size: unsafe { &_binary_bin_guest_bin_size as *const _ as usize },
        },
        guest_dtb: 0,
        guest_initrd: 0,
        entry_addr: 0x40200000,
        memory_size: 128 * 1024 * 1024, // 128MB
        vcpu_count: 1,
    };
    
    // create virtual machine
    info!("Creating virtual machine...");
    let mut vm = VirtualMachine::new(1, vm_config)?;
    
    // start virtual machine
    info!("Starting virtual machine...");
    vm.start()?;
    
    // store the virtual machine in the global manager
    {
        let mut vm_manager = VM_MANAGER.lock();
        *vm_manager = Some(vm);
    }
    
    info!("Guest VM started successfully");
    Ok(())
}