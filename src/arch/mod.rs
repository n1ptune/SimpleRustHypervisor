#[macro_use]
mod qemu;
pub use qemu::*;
mod page_table;
pub use page_table::*;

/// 刷新TLB (Translation Lookaside Buffer)
#[inline]
pub fn flush_tlb() {
    // 数据同步屏障
    dsb!(ishst);
    
    // 刷新所有中间物理机的TLB条目
    unsafe {
        core::arch::asm!(
            "tlbi vmalls12e1"
        );
    }
    
    // 数据同步屏障
    dsb!(ish);
    
    // 指令同步屏障
    isb!();
}

pub fn flush_tlb_ipa_s2(ipa: usize) {
    let addr_for_tlbi = ipa >> 12;

    // 步骤 1: 确保 D-Cache 写回主内存
    dsb!(ISH); // Data Synchronization Barrier
    // 这个 DSB 屏障确保了在它之前的内存写入操作
    // (比如修改页表项) 对于系统中的其他部分
    // (比如 MMU 的硬件页表遍历器) 是可见的。
    unsafe {
        // 步骤 2: 无效化 TLB

        core::arch::asm!(
            "tlbi ipas2e1is, {}", 
            in(reg) addr_for_tlbi, options(nostack)
            );

        // 现在 TLB 被清空了。
    }
    // 步骤 3 & 4: 确保同步完成
    dsb!(ISH);
    isb!();
    // 确保 TLB 无效化操作完成，并清空指令流水线，
    // 强迫 CPU 在下次访问时，一定会去主内存
    // (现在已经是最新的了) 重新进行页表遍历。
}

pub fn coreid() -> usize {
    let val = read_sysreg!(mpidr_el1);
    (val & 0xf) as usize
}