ENTRY(_start)
EXTERN(_start)

NCPU = 4;
SZ_4K = 0x00001000;
SRH_BASE = 0x40200000;
SECTIONS
{
    /* hyper image link virtual addr, here equal to phys addr */
    . = SRH_BASE;

    .text.boot : {
      KEEP(*(.text.boot))
    }

    . = ALIGN(4096);

    .text : {
      *(.text) *(.text.*)
    }

    .rodata : {
      *(.rodata) *(.rodata.*)
    }

    .data : {
      *(.data) *(.data.*)
    }
    . = ALIGN(4096);

    .bss : {
      __bss_start = .;
      *(.bss .bss.*)
      __bss_end = .;
    }
    . = ALIGN(4096);

    .stack (NOLOAD) : ALIGN(4096) {
      sp_stack_start = .;
      . = . + SZ_4K * NCPU;
      sp_stack_end = .;
      FILL(0x00)
    }
    . = ALIGN(4096);


    .heap (NOLOAD) : ALIGN(4096) {
      __heap_start = .;
      . = . + 0x200000;   /* 2MB 堆空间，可根据需要调整 */
      __heap_end = .;
      FILL(0x00)
    }
    . = ALIGN(4096);

    SRH_END = .;
}