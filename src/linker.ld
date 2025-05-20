ENTRY(_start)
EXTERN(_start)
EXTERN(sp_stack_start)
EXTERN(sp_stack_end)
EXTERN(NCPU)
EXTERN(SZ_4K)
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


    HIMAGE_END = .;
}