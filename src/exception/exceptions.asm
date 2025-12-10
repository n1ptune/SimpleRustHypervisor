// 异常向量表，每个异常入口 128 字节对齐
.section .text.exceptions, "ax"
.align 11
.global exception_vector_table
exception_vector_table:

// Current EL with SP_EL0
.align 7
curr_el_sp0_sync:
    b exception_handler
.align 7
curr_el_sp0_irq:
    b exception_handler
.align 7
curr_el_sp0_fiq:
    b exception_handler
.align 7
curr_el_sp0_serror:
    b exception_handler



// Current EL with SP_ELx
.align 7
curr_el_spx_sync:
    b exception_handler
.align 7
curr_el_spx_irq:
    b exception_handler
.align 7
curr_el_spx_fiq:
    b exception_handler
.align 7
curr_el_spx_serror:
    b exception_handler



// Lower EL using AArch64
.align 7
lower_el_aarch64_sync:
    b sync_exception_handler
.align 7
lower_el_aarch64_irq:
    b irq_exception_handler
.align 7
lower_el_aarch64_fiq:
    b fiq_exception_handler
.align 7
lower_el_aarch64_serror:
    b serror_exception_handler



// Lower EL using AArch32
.align 7
lower_el_aarch32_sync:
    b exception_handler
.align 7
lower_el_aarch32_irq:
    b exception_handler
.align 7
lower_el_aarch32_fiq:
    b exception_handler
.align 7
lower_el_aarch32_serror:
    b exception_handler




.macro save_vm_regs
    stp x0, x1, [sp, #-16]!     /* save x0, x1 on stack */
    mrs x0, tpidr_el2           /* x0 = &vcpu->regs */

    stp x2, x3, [x0, #8 * 2]
    stp x4, x5, [x0, #8 * 4]
    stp x6, x7, [x0, #8 * 6]
    stp x8, x9, [x0, #8 * 8]
    stp x10, x11, [x0, #8 * 10]
    stp x12, x13, [x0, #8 * 12]
    stp x14, x15, [x0, #8 * 14]
    stp x16, x17, [x0, #8 * 16]
    stp x18, x19, [x0, #8 * 18]
    stp x20, x21, [x0, #8 * 20]
    stp x22, x23, [x0, #8 * 22]
    stp x24, x25, [x0, #8 * 24]
    stp x26, x27, [x0, #8 * 26]
    stp x28, x29, [x0, #8 * 28]

    mrs x1, spsr_el2 /* x1 = spsr_el2 */
    mrs x2, elr_el2  /* x2 = elr_el2 */
    mrs x3, sp_el0 /* x3 = sp_el0 */
    mrs x4, sp_el1  /* x4 = sp_el1 */
    ldp x5, x6, [sp], #16   /* x5 = x0, x6 = x1 */
    stp x30, x1, [x0, #8 *30]
    stp x2, x3, [x0, #8 *32]
    str x4, [x0, #8 * 34]
    stp x5, x6, [x0, #8 * 0]
.endm



.macro restore_vm_regs
    mrs x0,  tpidr_el2          /* x0 = &vcpu->regs */
    ldp x30, x1, [x0, #8 * 30]  /* x1 = spsr_el2 */
    ldp x2, x3, [x0, #8 * 32]  /* x2 = elr_el2, x3 = sp_el0 */
    ldr x4, [x0, #8 * 34]       /* x4 = sp_el1 */
    msr spsr_el2, x1
    msr elr_el2,  x2
    msr sp_el0, x3
    msr sp_el1, x4

    ldp x3, x4, [x0, #8 * 0]    /* x3 = x0, x4 = x1 */
    stp x3, x4, [sp, #-16]!     /* save x0, x1 on stack */

    ldp x2, x3, [x0, #8 * 2]
    ldp x4, x5, [x0, #8 * 4]
    ldp x6, x7, [x0, #8 * 6]
    ldp x8, x9, [x0, #8 * 8]
    ldp x10, x11, [x0, #8 * 10]
    ldp x12, x13, [x0, #8 * 12]
    ldp x14, x15, [x0, #8 * 14]
    ldp x16, x17, [x0, #8 * 16]
    ldp x18, x19, [x0, #8 * 18]
    ldp x20, x21, [x0, #8 * 20]
    ldp x22, x23, [x0, #8 * 22]
    ldp x24, x25, [x0, #8 * 24]
    ldp x26, x27, [x0, #8 * 26]
    ldp x28, x29, [x0, #8 * 28]

    ldp x0, x1, [sp], #16
.endm



// 通用异常处理程序
exception_handler:
    save_vm_regs
    bl handle_exception
    restore_vm_regs
    eret

sync_exception_handler:
    save_vm_regs
    bl handle_sync_exception_from_asm
    restore_vm_regs
    eret

// IRQ 异常处理程序
irq_exception_handler:
    save_vm_regs
    bl el1_irq_proc
    restore_vm_regs
    eret

// FIQ 异常处理程序  
fiq_exception_handler:
    b exception_handler

// SError 异常处理程序
serror_exception_handler:
    b exception_handler