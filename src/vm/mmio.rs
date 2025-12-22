use core::fmt;

use alloc::{boxed::Box, string::String, vec::Vec};


use crate::vm::Vcpu;


pub trait MmioSpace{
    fn base_addr(&self) -> usize;
    fn size(&self) -> usize;

    fn mmio_read(&mut self, vcpu: &mut Vcpu, srt: usize, offset: usize, access: MmioAccess) -> bool;
    fn mmio_write(&mut self, vcpu: &mut Vcpu, srt: usize, offset: usize, access: MmioAccess) -> bool;

    fn debug_info(&self) -> String ;
}

#[derive(Debug, Clone, Copy)]
pub enum AccessSize {
    VmMioByte       = 1,
    VmMioHalfword   = 2,
    VmMioWord       = 4,
    VmMioDoubleword = 8,
}
impl AccessSize {
    pub fn from_size(size: u8) -> Option<AccessSize> {
        match size {
            1 => Some(AccessSize::VmMioByte),
            2 => Some(AccessSize::VmMioHalfword),
            4 => Some(AccessSize::VmMioWord),
            8 => Some(AccessSize::VmMioDoubleword),
            _ => None,
        }
    }
}

#[allow(unused)]
pub struct MmioAccess{
    pub ipa: usize,
    pub pc: usize,
    pub wnr: bool,
    pub access_size: AccessSize,
}

pub struct MmioManager{
    mmio_spaces : Vec<Box<dyn MmioSpace + Send>>
}

impl fmt::Debug for MmioManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let names: Vec<_> = self.mmio_spaces.iter().map(|s| s.debug_info()).collect();
        f.debug_struct("MmioManager")
         .field("mmio_spaces", &names)
         .finish()
    }
}

impl MmioManager{
    pub fn new() -> Self{
        MmioManager{
            mmio_spaces: Vec::new(),
        }
    }

    pub fn add_mmio_space(&mut self, mmio_space: Box<dyn MmioSpace + Send>){
        self.mmio_spaces.push(mmio_space);
    }

    #[allow(unused)]
    pub fn del_mmio_spaces(&mut self, base_addr: usize) -> bool{
        for i in 0..self.mmio_spaces.len(){
            if self.mmio_spaces[i].base_addr() == base_addr{
                self.mmio_spaces.remove(i);
                return true;
            }
        }
        false
    }

    pub fn handle_mmio(&mut self, vcpu: &mut Vcpu, srt: usize, access: MmioAccess) -> bool{
        let addr = access.ipa;
        let wnr = access.wnr;
        for space in self.mmio_spaces.iter_mut(){
            let base = space.base_addr();
            if addr >= base && addr < base + space.size(){
                if wnr {
                    return space.mmio_write(vcpu, srt, addr - base, access);
                } else {
                    return space.mmio_read(vcpu, srt, addr - base, access);
                }
            }
        }
        false
    }
}
