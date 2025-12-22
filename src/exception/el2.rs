use log::info;

use crate::gic::{GIC_IRQ_OPS, UART_IRQ_LINE};
use crate::pl011::pl011_irq_handler;
use crate::gic::GicIrqOps;

#[no_mangle]
extern "C" fn handle_exception() {
    info!("handle exception");
    let gic_irq_ops = GIC_IRQ_OPS.lock();
    info!("el2 IRQ");
    let irq = gic_irq_ops.get_irq() & 0x3FF;

    info!("el2 IRQ: {}", irq);
    match irq {
        UART_IRQ_LINE => pl011_irq_handler(),
        _ => info!("Unhandled interrupt"),
    }

    gic_irq_ops.hyp_eoi(irq);
    info!("el2 IRQ handled!");
}
