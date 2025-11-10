use alloc::vec::Vec;
use alloc::boxed::Box;
use spin::Mutex;
use log::*;

// 设备 MMIO 访问权限
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeviceAccess {
    Read,
    Write,
}

// 虚拟设备 trait
pub trait VirtualDevice {
    fn name(&self) -> &str;
    fn base_address(&self) -> u64;
    fn size(&self) -> u64;
    
    // 处理 MMIO 访问
    fn mmio_read(&mut self, offset: u64, size: u8) -> Result<u64, &'static str>;
    fn mmio_write(&mut self, offset: u64, size: u8, value: u64) -> Result<(), &'static str>;
    
    // 设备初始化
    fn init(&mut self) -> Result<(), &'static str>;
    
    // 设备重置
    fn reset(&mut self);
}

pub struct VirtualUart {
    pub base_addr: u64,
    pub data_reg: u32,
    pub status_reg: u32,
    pub control_reg: u32,
}

impl VirtualUart {
    pub fn new(base_addr: u64) -> Self {
        VirtualUart {
            base_addr,
            data_reg: 0,
            status_reg: 0x90, // TX empty, RX empty
            control_reg: 0,
        }
    }
}

impl VirtualDevice for VirtualUart {
    fn name(&self) -> &str {
        "Virtual UART"
    }
    
    fn base_address(&self) -> u64 {
        self.base_addr
    }
    
    fn size(&self) -> u64 {
        0x1000 // 4KB
    }
    
    fn mmio_read(&mut self, offset: u64, size: u8) -> Result<u64, &'static str> {
        // debug!("mmio read {:x}", offset);
        match offset {
            0x00 => Ok(self.data_reg as u64), // UARTDR
            0x18 => Ok(self.status_reg as u64), // UARTFR
            0x30 => Ok(self.control_reg as u64), // UARTCR
            _ => {
                warn!("UART: Unknown read offset 0x{:x}", offset);
                Ok(0)
            }
        }
    }
    
    fn mmio_write(&mut self, offset: u64, _size: u8, value: u64) -> Result<(), &'static str> {
        // debug!("mmio write {:x} {:x}", offset, value);
        match offset {
            0x00 => {
                // UARTDR - 数据寄存器
                let ch = (value & 0xFF) as u8;
                print!("{}", ch as char);
                Ok(())
            },
            0x30 => {
                // UARTCR - 控制寄存器
                self.control_reg = value as u32;
                Ok(())
            },
            _ => {
                warn!("UART: Unknown write offset 0x{:x}, value=0x{:x}", offset, value);
                Ok(())
            }
        }
    }
    
    fn init(&mut self) -> Result<(), &'static str> {
        info!("Initializing Virtual UART at 0x{:x}", self.base_addr);
        self.status_reg = 0x90; // TX empty, ready
        Ok(())
    }
    
    fn reset(&mut self) {
        self.data_reg = 0;
        self.status_reg = 0x90;
        self.control_reg = 0;
    }
}

// 虚拟时钟设备
pub struct VirtualTimer {
    pub base_addr: u64,
    pub counter: u64,
    pub compare: u64,
    pub control: u32,
}

impl VirtualTimer {
    pub fn new(base_addr: u64) -> Self {
        VirtualTimer {
            base_addr,
            counter: 0,
            compare: 0,
            control: 0,
        }
    }
}

impl VirtualDevice for VirtualTimer {
    fn name(&self) -> &str {
        "Virtual Timer"
    }
    
    fn base_address(&self) -> u64 {
        self.base_addr
    }
    
    fn size(&self) -> u64 {
        0x1000
    }
    
    fn mmio_read(&mut self, offset: u64, _size: u8) -> Result<u64, &'static str> {
        match offset {
            0x00 => Ok(self.counter),
            0x08 => Ok(self.compare),
            0x10 => Ok(self.control as u64),
            _ => Ok(0)
        }
    }
    
    fn mmio_write(&mut self, offset: u64, _size: u8, value: u64) -> Result<(), &'static str> {
        match offset {
            0x08 => self.compare = value,
            0x10 => self.control = value as u32,
            _ => {}
        }
        Ok(())
    }
    
    fn init(&mut self) -> Result<(), &'static str> {
        info!("Initializing Virtual Timer at 0x{:x}", self.base_addr);
        Ok(())
    }
    
    fn reset(&mut self) {
        self.counter = 0;
        self.compare = 0;
        self.control = 0;
    }
}

// 设备管理器
pub struct DeviceManager {
    devices: Vec<Box<dyn VirtualDevice + Send>>,
}

impl DeviceManager {
    pub fn new() -> Self {
        DeviceManager {
            devices: Vec::new(),
        }
    }
    
    pub fn add_device(&mut self, device: Box<dyn VirtualDevice + Send>) {
        info!("Adding virtual device: {}", device.name());
        self.devices.push(device);
    }
    
    pub fn handle_mmio(&mut self, addr: u64, access: DeviceAccess, size: u8, value: Option<u64>) -> Result<u64, &'static str> {
        for device in &mut self.devices {
            let base = device.base_address();
            if addr >= base && addr < base + device.size() {
                let offset = addr - base;
                match access {
                    DeviceAccess::Read => {
                        return device.mmio_read(offset, size);
                    },
                    DeviceAccess::Write => {
                        if let Some(val) = value {
                            device.mmio_write(offset, size, val)?;
                            return Ok(0);
                        }
                    }
                }
            }
        }
        Err("No device found for address")
    }
    
    pub fn init_all_devices(&mut self) -> Result<(), &'static str> {
        for device in &mut self.devices {
            device.init()?;
        }
        Ok(())
    }
}

lazy_static! {
    pub static ref DEVICE_MANAGER: Mutex<DeviceManager> = Mutex::new(DeviceManager::new());
}

pub fn init_virtual_devices() -> Result<(), &'static str> {
    let mut dm = DEVICE_MANAGER.lock();
    
    let uart = Box::new(VirtualUart::new(0x09000000));
    dm.add_device(uart);
    

    dm.init_all_devices()?;

    Ok(())
}