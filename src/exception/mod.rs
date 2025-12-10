mod exception_handler;
mod el1;
mod el2;

core::arch::global_asm!(include_str!("exceptions.asm"));

pub use exception_handler::{setup_exception_handlers};
