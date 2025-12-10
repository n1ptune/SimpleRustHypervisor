use core::ops::{Deref, DerefMut};
use spin::{MutexGuard};

use crate::gic::{VgicDist, VgicIrqConfig}; // 假设你用的是 spin crate

// 这是一个“智能引用”
pub enum IrqRef<'a> {
    // 针对 SGI/PPI：直接持有引用
    Local(&'a mut VgicIrqConfig),
    
    // 针对 SPI：持有 锁守卫(Guard) + 数组索引
    // 注意：我们必须持有 Guard，否则锁就释放了
    Shared(MutexGuard<'a, VgicDist>, usize),
}

// 实现 Deref：允许你像读引用一样读它
impl<'a> Deref for IrqRef<'a> {
    type Target = VgicIrqConfig;

    fn deref(&self) -> &Self::Target {
        match self {
            IrqRef::Local(ref_mut) => ref_mut,
            // 从锁守卫中，通过索引找到对应的数据
            IrqRef::Shared(guard, idx) => &guard.spis[*idx],
        }
    }
}

// 实现 DerefMut：允许你像修改引用一样修改它
impl<'a> DerefMut for IrqRef<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            IrqRef::Local(ref_mut) => ref_mut,
            IrqRef::Shared(guard, idx) => &mut guard.spis[*idx],
        }
    }
}