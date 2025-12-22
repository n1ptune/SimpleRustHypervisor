use alloc::{format, string::String};
use log::*;

use crate::vm::{MmioAccess, MmioSpace, Vcpu};
#[derive(Debug)]
pub struct VirtualUart {
    pub base_addr: usize,
    pub size: usize,
    pub data_reg: u32,
    pub status_reg: u32,
    pub control_reg: u32,
    pub ris_reg: u32,
    pub imsc_reg: u32,
}


impl VirtualUart {
    pub fn new(base_addr: usize) -> Self {
        VirtualUart { base_addr, 
                    size: 0x1000, 
                    data_reg: 0, 
                    status_reg: 0x90, 
                    control_reg: 0, 
                    ris_reg: 0,
                    imsc_reg: 0
                }
    }
}

impl MmioSpace for VirtualUart {
    fn base_addr(&self) -> usize {
        self.base_addr
    }

    fn size(&self) -> usize {
        self.size
    }

    fn mmio_read(&mut self, vcpu: &mut Vcpu, srt: usize, offset: usize, _access: MmioAccess) -> bool {
        // debug!("mmio read {:x}", offset);
        if srt == 31 {
            return true;
        }
        match offset {
            0x00 => {
                vcpu.regs.x[srt] = self.data_reg as u64;
                self.status_reg |= 1 << 4;
            }, // UARTDR
            0x18 => {
                // debug!("mmio read {:x}", offset);
                vcpu.regs.x[srt] = self.status_reg as u64; 
            }, // UARTFR
            0x30 => {
                vcpu.regs.x[srt] = self.control_reg as u64; 
            }, // UARTCR
            0x38 => {
                vcpu.regs.x[srt] = self.imsc_reg as u64; 
            }, 
            0x3c => {
                // debug!("mmio read {:x}", offset);
                vcpu.regs.x[srt] = self.ris_reg as u64;
            },
            0x40 => {
                vcpu.regs.x[srt] = (self.ris_reg & self.imsc_reg) as u64;
            },
            0xFE0 => vcpu.regs.x[srt] = 0x11, // UARTPeriphID0
            0xFE4 => vcpu.regs.x[srt] = 0x10, // UARTPeriphID1
            0xFE8 => vcpu.regs.x[srt] = 0x34, // UARTPeriphID2
            0xFEC => vcpu.regs.x[srt] = 0x00, // UARTPeriphID3
            0xFF0 => vcpu.regs.x[srt] = 0x0D, // UARTPCellID0
            0xFF4 => vcpu.regs.x[srt] = 0xF0, // UARTPCellID1
            0xFF8 => vcpu.regs.x[srt] = 0x05, // UARTPCellID2
            0xFFC => vcpu.regs.x[srt] = 0xB1, // UARTPCellID3
            _ => {
                info!("UART: Unknown read offset 0x{:x}", offset);
                return false;
            }
        }
        true
    }

    fn mmio_write(&mut self, vcpu: &mut Vcpu, srt: usize, offset: usize, _access: MmioAccess) -> bool {
        // debug!("mmio write {:x}", offset);
        let val = if srt == 31 { 0 } else { vcpu.regs.x[srt] };
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
            0x38 => {
                self.imsc_reg = val as u32;
                true
            },
            0x44 => {
                self.ris_reg &= !(val as u32); 
                true
            },
            0x24 | 0x28 | 0x2c | 0x34 => {
                true
            },
            0xff => {
                //模拟存入数据
                self.data_reg = val as u32;
                self.status_reg &= !(1 << 4);
                self.ris_reg |= 1 << 4;
                true
            }
            _ => {
                info!("UART: Unknown write offset 0x{:x}", offset);
                return false;
            }
        }
    }

    fn debug_info(&self) -> String {
        format!("VirtualUart start {} size {}", self.base_addr, self.size)
    }
}
