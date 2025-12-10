#[allow(unreachable_code)]
pub fn shutdown() -> ! {
    const PSCI_SYSTEM_OFF: u64 = 0x84000008;
    println!("Shutting down QEMU...");
    unsafe {
        core::arch::asm!(
            "smc #0",
            in("x0") PSCI_SYSTEM_OFF, // PSCI_CPU_OFF命令（来自设备树）
            in("x1") 0,            // context ID (未使用)
            options(noreturn)
        );
    }
    panic!("Shutdown failed!"); // 如果未退出QEMU则panic
}

#[macro_export]
macro_rules! read_sysreg {
    ($reg:ident) => {{
        let mut value: u64;
        unsafe {
            core::arch::asm!(
                concat!("mrs {0}, ", stringify!($reg)),
                out(reg) value,
                options(nostack, nomem)
            );
        }
        value
    }};
}

#[macro_export]
macro_rules! write_sysreg {
    ($reg:ident, $value:expr) => {{
        unsafe {
            core::arch::asm!(
                concat!("msr ", stringify!($reg), ", {0:x}"),
                in(reg) $value,
                options(nostack, nomem)
            );
        }
    }};
}

#[macro_export]
macro_rules! dsb {
    ($opt:ident) => {{
        unsafe {
            core::arch::asm!(
                concat!("dsb ", stringify!($opt)),
                options(nostack, nomem)
            );
        }
    }};
}

#[macro_export]
#[allow(unused_unsafe)]
macro_rules! isb {
    () => {{
        unsafe {
            core::arch::asm!(
                "isb",
                options(nostack, nomem)
            );
        }
    }};
}