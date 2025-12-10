use log::info;

use crate::exception::exception_handler::get_current_vcpu_ptr;
use crate::gic::{GIC_IRQ_OPS};
use crate::gic::{GicIrqOps, virq_inject};

#[no_mangle]
extern "C" fn el1_irq_proc() {
    let gic_irq_ops = GIC_IRQ_OPS.lock();
    let irq = gic_irq_ops.get_irq() & 0x3FF;

    info!("el1 IRQ: {}", irq);
    gic_irq_ops.guest_eoi(irq);

    let vcpu_ptr = get_current_vcpu_ptr();

    if vcpu_ptr.is_null() {
        panic!("Critical: Exception in hypervisor context!");
    }

    let vcpu_ref = unsafe { &mut *vcpu_ptr };

    let _ = virq_inject(vcpu_ref, irq, irq);
}
