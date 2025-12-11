#![allow(unused)]

#[derive(Debug, Clone)]
pub struct GuestDtb{
    pub name: &'static str,
    pub start: usize,
    pub end: usize,
    pub size: usize,
}

#[derive(Debug, Clone)]
pub struct GuestVMImage {
    pub name: &'static str,
    pub start: usize,
    pub end: usize,
    pub size: usize,
}

pub static GUEST_DTB: &[u8] = include_bytes!("../../virt.dtb");