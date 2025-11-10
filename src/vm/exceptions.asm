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

// 通用异常处理程序
exception_handler:
    // 保存所有寄存器
    stp x0, x1, [sp, #-16]!
    stp x2, x3, [sp, #-16]!
    stp x4, x5, [sp, #-16]!
    stp x6, x7, [sp, #-16]!
    stp x8, x9, [sp, #-16]!
    stp x10, x11, [sp, #-16]!
    stp x12, x13, [sp, #-16]!
    stp x14, x15, [sp, #-16]!
    stp x16, x17, [sp, #-16]!
    stp x18, x19, [sp, #-16]!
    stp x20, x21, [sp, #-16]!
    stp x22, x23, [sp, #-16]!
    stp x24, x25, [sp, #-16]!
    stp x26, x27, [sp, #-16]!
    stp x28, x29, [sp, #-16]!
    str x30, [sp, #-8]!

    // 调用 Rust 异常处理函数
    bl handle_exception

    // 恢复寄存器并返回
    ldr x30, [sp], #8
    ldp x28, x29, [sp], #16
    ldp x26, x27, [sp], #16
    ldp x24, x25, [sp], #16
    ldp x22, x23, [sp], #16
    ldp x20, x21, [sp], #16
    ldp x18, x19, [sp], #16
    ldp x16, x17, [sp], #16
    ldp x14, x15, [sp], #16
    ldp x12, x13, [sp], #16
    ldp x10, x11, [sp], #16
    ldp x8, x9, [sp], #16
    ldp x6, x7, [sp], #16
    ldp x4, x5, [sp], #16
    ldp x2, x3, [sp], #16
    ldp x0, x1, [sp], #16
    eret

sync_exception_handler:
    str x30, [sp, #-8]!         
    stp x28, x29, [sp, #-16]!   
    stp x26, x27, [sp, #-16]! 
    stp x24, x25, [sp, #-16]!
    stp x22, x23, [sp, #-16]!
    stp x20, x21, [sp, #-16]!
    stp x18, x19, [sp, #-16]!
    stp x16, x17, [sp, #-16]!
    stp x14, x15, [sp, #-16]!
    stp x12, x13, [sp, #-16]!
    stp x10, x11, [sp, #-16]!
    stp x8, x9, [sp, #-16]!
    stp x6, x7, [sp, #-16]!
    stp x4, x5, [sp, #-16]!
    stp x2, x3, [sp, #-16]!
    stp x0, x1, [sp, #-16]!

    // 保存异常相关寄存器
    mrs x0, esr_el2
    mrs x1, far_el2
    mrs x2, elr_el2
    mrs x3, spsr_el2
    stp x0, x1, [sp, #-16]!
    stp x2, x3, [sp, #-16]!

    mrs x4, sp_el1
    str x4, [sp, #-8]!

    // 将栈指针传递给 Rust 函数
    mov x0, sp
    bl handle_sync_exception_from_asm

    // 恢复寄存器
    ldr x4, [sp], #8
    msr sp_el1, x4

    ldp x2, x3, [sp], #16
    ldp x0, x1, [sp], #16
    msr esr_el2, x0
    msr far_el2, x1
    msr elr_el2, x2
    msr spsr_el2, x3

    ldp x0, x1, [sp], #16
    ldp x2, x3, [sp], #16
    ldp x4, x5, [sp], #16
    ldp x6, x7, [sp], #16
    ldp x8, x9, [sp], #16
    ldp x10, x11, [sp], #16
    ldp x12, x13, [sp], #16
    ldp x14, x15, [sp], #16
    ldp x16, x17, [sp], #16
    ldp x18, x19, [sp], #16
    ldp x20, x21, [sp], #16
    ldp x22, x23, [sp], #16
    ldp x24, x25, [sp], #16
    ldp x26, x27, [sp], #16
    ldp x28, x29, [sp], #16
    ldr x30, [sp], #8
    eret

// IRQ 异常处理程序
irq_exception_handler:
    // 保存所有寄存器
    stp x0, x1, [sp, #-16]!
    stp x2, x3, [sp, #-16]!
    stp x4, x5, [sp, #-16]!
    stp x6, x7, [sp, #-16]!
    stp x8, x9, [sp, #-16]!
    stp x10, x11, [sp, #-16]!
    stp x12, x13, [sp, #-16]!
    stp x14, x15, [sp, #-16]!
    stp x16, x17, [sp, #-16]!
    stp x18, x19, [sp, #-16]!
    stp x20, x21, [sp, #-16]!
    stp x22, x23, [sp, #-16]!
    stp x24, x25, [sp, #-16]!
    stp x26, x27, [sp, #-16]!
    stp x28, x29, [sp, #-16]!
    str x30, [sp, #-8]!

    // 调用 Rust IRQ 处理函数
    bl handle_exception

    // 恢复寄存器并返回
    ldr x30, [sp], #8
    ldp x28, x29, [sp], #16
    ldp x26, x27, [sp], #16
    ldp x24, x25, [sp], #16
    ldp x22, x23, [sp], #16
    ldp x20, x21, [sp], #16
    ldp x18, x19, [sp], #16
    ldp x16, x17, [sp], #16
    ldp x14, x15, [sp], #16
    ldp x12, x13, [sp], #16
    ldp x10, x11, [sp], #16
    ldp x8, x9, [sp], #16
    ldp x6, x7, [sp], #16
    ldp x4, x5, [sp], #16
    ldp x2, x3, [sp], #16
    ldp x0, x1, [sp], #16
    eret

// FIQ 异常处理程序  
fiq_exception_handler:
    b exception_handler

// SError 异常处理程序
serror_exception_handler:
    b exception_handler