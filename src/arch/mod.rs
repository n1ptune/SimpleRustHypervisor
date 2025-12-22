#[macro_use]
mod qemu;
use core::arch::asm;

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
        asm!(
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

        asm!(
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

pub unsafe fn sync_guest_memory(start: usize, size: usize) {
    let line_size = 64; // 通常是 64 字节
    let end = start + size;
    
    // 1. Clean D-Cache: 把你 memcpy 进去的数据从 CPU 缓存推送到内存
    // 如果不做这一步，MMU 看到的页表可能是空的！
    let mut addr = start & !(line_size - 1);
    while addr < end {
        // DC CVAU: Data Cache Clean by VA to Point of Unification
        asm!("dc cvau, {0}", in(reg) addr);
        addr += line_size;
    }
    asm!("dsb ish"); // 确保数据真的到了内存

    // 2. Invalidate I-Cache: 告诉 CPU 指令缓存该更新了
    // 如果不做这一步，Guest 可能会执行到旧的指令
    addr = start & !(line_size - 1);
    while addr < end {
        // IC IVAU: Instruction Cache Invalidate by VA to Point of Unification
        asm!("ic ivau, {0}", in(reg) addr);
        addr += line_size;
    }
    
    // 3. 最后的屏障
    asm!("dsb ish");
    asm!("isb");
}


pub unsafe fn smc_call(function_id: u64, arg1: u64, arg2: u64) -> u64 {
    let mut ret0: u64;

    asm!(
        "smc #0",
        in("x0") function_id,
        in("x1") arg1,
        in("x2") arg2,
        lateout("x0") ret0,
    );

    ret0
}