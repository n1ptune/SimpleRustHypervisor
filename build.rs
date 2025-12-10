use std::path::PathBuf;

fn main() {
    let bin = PathBuf::from("bin");


    // 2. 告诉 cargo 链接这个 .o
    println!("cargo:rustc-link-arg={}", bin.join("guest.o").display());
}