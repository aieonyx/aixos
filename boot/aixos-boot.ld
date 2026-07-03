/* Copyright (c) 2026 Edison Lepiten / AIEONYX */

ENTRY(_start)

MEMORY
{
    RAM (rwx) : ORIGIN = 0x40000000, LENGTH = 128M
}

SECTIONS
{
    . = ORIGIN(RAM);
    .text : {
        KEEP(*(.text.boot))
        *(.text*)
    } > RAM
    .rodata : { *(.rodata*) } > RAM
    .data   : { *(.data*)   } > RAM
    .bss    : {
        *(.bss*)
        . = ALIGN(16);
        . += 0x10000;
        _stack_top = .;
    } > RAM
}
