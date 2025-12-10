use alloc::{format, string::String};
use log::*;

use crate::vm::{MmioAccess, MmioSpace, Vcpu};

pub struct VirtualUart {
    pub base_addr: usize,
    pub size: usize,
    pub data_reg: u32,
    pub status_reg: u32,
    pub control_reg: u32,
}


impl VirtualUart {
    pub fn new(base_addr: usize) -> Self {
        VirtualUart { base_addr, size: 0x1000, data_reg: 0, status_reg: 0x90, control_reg: 0 }
    }
}

impl MmioSpace for VirtualUart {
    fn base_addr(&self) -> usize {
        self.base_addr
    }

    fn size(&self) -> usize {
        self.size
    }

    fn mmio_read(&self, _vcpu: &mut Vcpu, reg: &mut u64, offset: usize, _access: MmioAccess) -> bool {
        // debug!("mmio read {:x}", offset);
        match offset {
            0x00 => {
                *reg = self.data_reg as u64;
                true
            }, // UARTDR
            0x18 => {
                *reg = self.status_reg as u64; 
                true
            }, // UARTFR
            0x30 => {
                *reg = self.control_reg as u64; 
                true
            }, // UARTCR
            _ => {
                warn!("UART: Unknown read offset 0x{:x}", offset);
                return false;
            }
        }
    }

    fn mmio_write(&mut self, _vcpu: &mut Vcpu, val: u64, offset: usize, _access: MmioAccess) -> bool {
        // debug!("mmio write {:x} {:x}", offset, value);
        match offset {
            0x00 => {
                // UARTDR - 数据寄存器
                let ch = (val & 0xFF) as u8;
                print!("{}", ch as char);
                true
            },
            0x30 => {
                // UARTCR - 控制寄存器
                self.control_reg = val as u32;
                true
            },
            _ => {
                warn!("UART: Unknown write offset 0x{:x}", offset);
                return false;
            }
        }
    }

    fn debug_info(&self) -> String {
        format!("VirtualUart start {} size {}", self.base_addr, self.size)
    }
}
