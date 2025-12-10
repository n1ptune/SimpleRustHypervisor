use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        println!(
            "panicked at '{}', {}:{}:{}",
            info.message(),
            location.file(),
            location.line(),
            location.column()
        );
    }
    else{
        println!("panicked at '{}'", info.message());
    }
    loop {}
}