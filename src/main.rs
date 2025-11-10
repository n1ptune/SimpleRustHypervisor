#![no_std]
#![no_main]

use log::*;

#[macro_use]
extern crate lazy_static;


#[macro_use]
mod console;
mod logging;
mod panic;
mod pl011;
mod mem;
mod config;
mod arch;
mod vm;

extern crate alloc;
core::arch::global_asm!(include_str!("entry.asm"));



#[no_mangle]
pub extern "C" fn rust_main() -> ! {

    pl011::pl011_init();
    logging::init();
    info!("hypervisor started!");

    vm::setup_exception_handlers();
    mem::init();
    
    // initialize Stage 2 MMU
    mem::stage2_mmu_init();
    mem::hyper_setup();
    
    // run virtual machine
    if let Err(e) = vm::run() {
        panic!("Failed to run VM: {}", e);
    }

    info!("hypervisor shutdown!");
    arch::shutdown();
}