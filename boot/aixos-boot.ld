/* Copyright (c) 2026 Edison Lepiten / AIEONYX */
ENTRY(_pe_head)

MEMORY
{
    RAM   (rwx) : ORIGIN = 0x40000000, LENGTH = 256M
    FBMEM (rw)  : ORIGIN = 0x44000000, LENGTH = 4M
}

SECTIONS
{
    . = ORIGIN(RAM);

    /* PE/COFF header — must be first, 4096-byte aligned */
    .head : { KEEP(*(.head)) . = ALIGN(4096); } > RAM

    /* Kernel text starts here (RVA 0x1000) */
    _text_start = .;
    .text : {
        KEEP(*(.text.efistub))
        KEEP(*(.text.boot))
        *(.text*)
        *(.rodata*)
        *(.data*)
        . = ALIGN(512);
    } > RAM

    /* Linker symbols consumed by PE header */
    _file_end  = .;
    _file_size = _file_end - ORIGIN(RAM);
    _text_size = _file_end - _text_start;
    _entry_rva = DEFINED(efi_entry) ? efi_entry - ORIGIN(RAM) : 0;
    . = ALIGN(4096);
    _img_end   = .;
    _img_size  = _img_end - ORIGIN(RAM);

    /* BSS and stack — not included in file image */
    .bss (NOLOAD) : {
        _bss_start = .;
        *(.bss*)
        . = ALIGN(16);
        _bss_end = .;
        . += 0x10000;
        _stack_top = .;
    } > RAM

    /* Framebuffer — separate memory region */
    .fb (NOLOAD) : { *(.fb) } > FBMEM
}
