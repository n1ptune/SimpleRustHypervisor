use core::ptr::{read_volatile, write_volatile};

const PL011_BASE: usize = 0x0900_0000;

#[inline(always)]
fn reg(offset: usize) -> *mut u32 {
    (PL011_BASE + offset) as *mut u32
}

// PL011 各寄存器偏移
const PL011_DR:    usize = 0x00; // Data Register
const PL011_FR:    usize = 0x18; // Flag Register
const PL011_IBRD:  usize = 0x24; // Integer Baud Rate Register
const PL011_FBRD:  usize = 0x28; // Fractional Baud Rate Register
const PL011_LCRH:  usize = 0x2c; // Line Control Register
const PL011_CR:    usize = 0x30; // Control Register
const PL011_IMSC:  usize = 0x38; // Interrupt Mask Set/Clear Register
const PL011_MIS:   usize = 0x40; // Masked Interrupt Status Register
const PL011_ICR:   usize = 0x44; // Interrupt Clear Register

// Flag Register bits
const PL011_FR_RXFE: u32 = 1 << 4; // Receive FIFO empty
const PL011_FR_TXFF: u32 = 1 << 5; // Transmit FIFO full
const PL011_FR_RXFF: u32 = 1 << 6; // Receive FIFO full
const PL011_FR_TXFE: u32 = 1 << 7; // Transmit FIFO empty

// Line Control Register bits
const PL011_LCRH_FEN:       u32 = 1 << 4; // Enable/Disable FIFOs
const PL011_LCRH_WLEN_8BIT: u32 = 3 << 5; // Word length - 8 bits

// Interrupt bits
const PL011_INTRX_ENABLED: u32 = 1 << 4; // RX interrupt enable
const PL011_INTTX_ENABLED: u32 = 1 << 5; // TX interrupt enable

pub fn pl011_putc(c: u8) {
    // 等待发送FIFO非满
    while unsafe { read_volatile(reg(PL011_FR)) & PL011_FR_TXFF != 0 } {
        core::hint::spin_loop();
    }
    unsafe { write_volatile(reg(PL011_DR), c as u32); }
}

pub fn pl011_puts(s: &str) {
    for &b in s.as_bytes() {
        pl011_putc(b);
    }
}

pub fn pl011_getc() -> Option<u8> {
    // 如果接收FIFO为空，返回None
    if unsafe { read_volatile(reg(PL011_FR)) & PL011_FR_RXFE != 0 } {
        None
    } else {
        Some(unsafe { read_volatile(reg(PL011_DR)) as u8 })
    }
}

pub fn pl011_init() {
    // 关闭UART
    unsafe { write_volatile(reg(PL011_CR), 0); }
    // 关闭所有中断
    unsafe { write_volatile(reg(PL011_IMSC), 0); }
    // 使能FIFO并设置8位数据
    unsafe { write_volatile(reg(PL011_LCRH), PL011_LCRH_FEN | PL011_LCRH_WLEN_8BIT); }
    // 使能接收和发送
    unsafe { write_volatile(reg(PL011_CR), 0x301); }
    // 使能接收中断
    unsafe { write_volatile(reg(PL011_IMSC), 1 << 4); }
}