#![no_std]
#![no_main]
mod panic;
mod pl011;
#[macro_use]
mod console;
use pl011::pl011_init;

core::arch::global_asm!(include_str!("entry.asm"));


#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    // this function is the entry point, since the linker looks for a function
    // named `_start` by default
    pl011_init();
    println!("execute rust_main!");
    loop {}
}