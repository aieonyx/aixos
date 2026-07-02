/* Copyright (c) 2026 Edison Lepiten / AIEONYX */
/* Output: target/aarch64-unknown-none/release/aixos.elf */

ENTRY(_start)

MEMORY
{
    RAM (rwx) : ORIGIN = 0x40000000, LENGTH = 2M
}

SECTIONS
{
    . = ORIGIN(RAM);
    .text   : { *(.text*)   } > RAM
    .rodata : { *(.rodata*) } > RAM
    .data   : { *(.data*)   } > RAM
    .bss    : { *(.bss*)    } > RAM
}
