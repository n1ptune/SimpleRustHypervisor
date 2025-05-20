ENTRY(_start)          /* 指定入口符号 */

SRH_BASE = 0x40200000
SECTIONS {
    . = 0x7C00;      /* BIOS 加载引导扇区的地址 */
    .text : {
        *(.text)      /* 代码段 */
    }
    .data : {
        *(.data)      /* 数据段 */
        *(.rodata*)   /* 只读数据 */
    }
    .bss : {
        *(.bss)       /* 未初始化数据 */
    }
    /DISCARD/ : {     /* 丢弃不需要的节 */
        *(.eh_frame)
        *(.note*)
    }
}