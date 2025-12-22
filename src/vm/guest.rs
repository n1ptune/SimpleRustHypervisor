#[allow(unused)]
#[derive(Debug, Clone)]
pub struct GuestDtb{
    pub name: &'static str,
    pub start: usize,
    pub end: usize,
    pub size: usize,
}
#[allow(unused)]
#[derive(Debug, Clone)]
pub struct GuestVMImage {
    pub name: &'static str,
    pub start: usize,
    pub end: usize,
    pub size: usize,
}
#[allow(unused)]
#[derive(Debug, Clone)]
pub struct GuestInitrd {
    pub name: &'static str,
    pub start: usize,
    pub end: usize,
    pub size: usize,
}

pub static GUEST_DTB: &[u8] = include_bytes!("../../bin/virt.dtb");
pub static GUEST_INITRD: &[u8] = include_bytes!("../../bin/rootfs.cpio.gz");
pub static GUEST_IMAGE: &[u8] = include_bytes!("../../bin/Image");