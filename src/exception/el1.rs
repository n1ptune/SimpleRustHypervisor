#[allow(unused)]
use log::info;

use crate::exception::exception_handler::get_current_vcpu_ptr;
use crate::gic::{GIC_IRQ_OPS};
use crate::gic::{GicIrqOps, virq_inject, vgic_enter};
use crate::pl011::pl011_getc;
use crate::vm::{AccessSize, MmioAccess, Vcpu};

#[no_mangle]
extern "C" fn el1_irq_proc() {
    // info!("el1_irq_proc");
    let vcpu_ptr = get_current_vcpu_ptr();
    if vcpu_ptr.is_null() {
            panic!("Critical: Exception in hypervisor context!");
    }
    let vcpu_ref = unsafe { &mut *vcpu_ptr };

    vgic_enter(vcpu_ref);    


    let gic_irq_ops = GIC_IRQ_OPS.lock();
    let irq = gic_irq_ops.get_irq() & 0x3FF;

    if irq == 33 {
        // info!("PL011 UART IRQ received");
        handle_pl011_uart_irq(vcpu_ref);
    }
    gic_irq_ops.guest_eoi(irq);
    let _ = virq_inject(vcpu_ref, irq, irq);
}


pub fn handle_pl011_uart_irq(vcpu: &mut Vcpu) {
    let char = pl011_getc().unwrap();
    
    let access = MmioAccess {
                            ipa: 0x090000ff, 
                            pc: 0, 
                            wnr: true, 
                            access_size: AccessSize::from_size(1).unwrap() 
                        };
    let mmio_manager = vcpu.mmio_manager.clone();
    let x0 = vcpu.regs.x[0];
    vcpu.regs.x[0] = char as u64;
    mmio_manager.lock().handle_mmio(vcpu, 0, access);
    // info!("PL011 UART IRQ: {}", char as char);
    vcpu.regs.x[0] = x0;
}