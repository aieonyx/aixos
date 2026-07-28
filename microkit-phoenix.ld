/* Copyright (c) 2026 Edison Lepiten / AIEONYX */
/* Microkit-compatible linker script for aiXos Phoenix-Desktop PD */
/* Single PT_LOAD segment — Microkit 1.4.1 requirement */

PHDRS
{
    load PT_LOAD;
}

ENTRY(_start);

SECTIONS
{
    . = 0x200000;

    .text :
    {
        _text = .;
        *(.text.start)
        *(.text*)
        *(.rodata*)
        _text_end = .;
    } :load

    . = ALIGN(0x1000);

    .data :
    {
        __init_array_start = .;
        KEEP(*(.init_array*))
        __init_array_end = .;
        *(.data*)
    } :load

    . = ALIGN(0x1000);

    /* BSS — zero-initialised, not stored in file */
    .bss (NOLOAD) :
    {
        _bss_start = .;
        *(.bss*)
        *(COMMON)
        /* Framebuffer — 4MB NOLOAD */
        *(.fb*)
        . = ALIGN(0x1000);
        _bss_end = .;
    } :load

    /DISCARD/ : {
        *(.comment*)
        *(.debug*)
        *(.eh_frame*)
        *(.note*)
        *(.ARM.*)
    }
}
