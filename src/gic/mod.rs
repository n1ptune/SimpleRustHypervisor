#![allow(unused)]
mod gicv3;
mod gicd;
mod gicr;
mod vgic;

use alloc::{boxed::Box, vec::Vec};
pub use gicv3::{gic_percpu_init, gic_v3_init, GIC_MAX_LRS, GIC_MAX_SPI, hyper_spi_config, UART_IRQ_LINE, GIC_EDGE_TRIGGER, GIC_IRQ_OPS};
use spin::Mutex;
use vgic::*;
use gicd::*;
use gicr::*;

pub use vgic::virq_inject;

use crate::vm::MmioManager;

pub trait GicIrqOps {
    fn name(&self) -> &'static str;
    fn mask(&self, irq: u32);
    fn unmask(&self, irq: u32);
    fn get_irq(&self) -> u32;
    fn guest_eoi(&self, irq: u32);
    fn hyp_eoi(&self, irq: u32);
    fn dir(&self, irq: u32);
    fn configure(&self, irq: u32, irq_type: u32);
    fn set_affinity(&self, irq: u32, cpu_num: u32);
    fn set_priority(&self, irq: u32, prio: u32);
    fn clear_pending(&self, irq: u32);
    fn set_pending(&self, irq: u32);
    fn get_pending(&self, irq: u32) -> bool;
    fn set_active(&self, irq: u32);
}

pub const GIC_NSGI: usize = 16;
pub const GIC_NPPI: usize = 16;

#[derive(Debug,Clone,Copy)]
pub struct VgicIrqConfig{
    pub priority:u8,
    pub affinity:u8,
    pub enabled:u8,
    pub group:u8,
}

#[derive(Debug, Clone)]
pub struct VgicVcpu{
    pub used_lr:u16,
    pub sgis: Vec<VgicIrqConfig>,
    pub ppi: Vec<VgicIrqConfig>,
}

impl VgicVcpu {
    pub fn new() -> Self {
        VgicVcpu { used_lr: 0 ,
            sgis: Vec::with_capacity(GIC_NSGI),
            ppi: Vec::with_capacity(GIC_NPPI),
        }
    }

    // Initialize SGIs
    pub fn init(&mut self, vcpuid: usize){
        self.used_lr = 0;

        // Initialize SGIs with iterator and map
        self.sgis = (0..GIC_NSGI).map(|_| VgicIrqConfig {
            priority: 0,
            affinity: 1 << vcpuid,
            enabled: 0,
            group: 1,
        }).collect();
        
        // Initialize PPIs with iterator and map
        self.ppi = (0..GIC_NPPI).map(|_| VgicIrqConfig {
            priority: 0,
            affinity: 1 << vcpuid,
            enabled: 0,
            group: 1,
        }).collect(); 
    }
}

#[derive(Debug)]
pub struct VgicDist{
    pub nspis:u32,
    pub enabled:bool,
    pub spis:Vec<VgicIrqConfig>,
}

impl VgicDist {
    pub fn new() -> Self {
        let gic_max_spi = GIC_MAX_SPI.lock();
        let nspis = *gic_max_spi - 31;
        VgicDist { nspis, 
            enabled: false ,
            spis: Vec::with_capacity(nspis as usize),
        }
    }

    pub fn init(&mut self, mmio_manager:&mut MmioManager){
        mmio_manager.add_mmio_space(Box::new(VirtualGicd::new(GICD_BASE, GICD_SIZE)));
        mmio_manager.add_mmio_space(Box::new(VirtualGicd::new(GICR_BASE, GICR_SIZE)));
    }
}
