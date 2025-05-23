#[allow(unreachable_code)]
pub fn shutdown() -> ! {
    const PSCI_SYSTEM_OFF: u64 = 0x84000008;
    unsafe {
        core::arch::asm!(
            "hvc #0",
            in("x0") PSCI_SYSTEM_OFF, // PSCI_CPU_OFF命令（来自设备树）
            in("x1") 0,            // context ID (未使用)
            options(noreturn)
        );
    }
    panic!("Shutdown failed!"); // 如果未退出QEMU则panic
}