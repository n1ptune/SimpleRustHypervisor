use log::{debug};

use crate::gic::{GIC_IRQ_OPS, UART_IRQ_LINE};
use crate::pl011::pl011_irq_handler;
use crate::gic::GicIrqOps;

#[no_mangle]
extern "C" fn handle_exception() {
    debug!("handle exception");
    let gic_irq_ops = GIC_IRQ_OPS.lock();
    debug!("el2 IRQ");
    let irq = gic_irq_ops.get_irq() & 0x3FF;

    debug!("el2 IRQ: {}", irq);
    match irq {
        UART_IRQ_LINE => pl011_irq_handler(),
        _ => debug!("Unhandled interrupt"),
    }

    gic_irq_ops.hyp_eoi(irq);
    debug!("el2 IRQ handled!");
}
