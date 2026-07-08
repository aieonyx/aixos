// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// PE/COFF header + EFI stub for aarch64 (PL-13a)
// Verified: xorriso ISO -> EDK2 BDS -> PE -> stub -> relocate -> 0x40000000

    .section .head, "ax"
    .globl _pe_head
_pe_head:
    .ascii  "MZ"
    .fill   0x3c - 2, 1, 0
    .long   pe_header - _pe_head

pe_header:
    .ascii  "PE\0\0"
coff_header:
    .short  0xaa64
    .short  1
    .long   0
    .long   0
    .long   0
    .short  section_table - opt_header
    .short  0x0206
opt_header:
    .short  0x20b
    .byte   0, 0
    .long   _text_size
    .long   0
    .long   0
    .long   _entry_rva
    .long   4096
    .quad   0
    .long   4096
    .long   512
    .short  0, 0
    .short  0, 0
    .short  2, 0
    .long   0
    .long   _img_size
    .long   4096
    .long   0
    .short  10
    .short  0
    .quad   0, 0, 0, 0
    .long   0
    .long   6
    .quad   0, 0, 0, 0, 0, 0
section_table:
    .ascii  ".text\0\0\0"
    .long   _text_size
    .long   4096
    .long   _text_size
    .long   4096
    .long   0, 0
    .short  0, 0
    .long   0xe0000020

    .section .text.efistub, "ax"
    .globl efi_entry
efi_entry:
    mov     x19, x0
    ldr     x21, [x1, #96]
    sub     sp, sp, #0x5000
    mov     x25, #4
1:  mov     x0, #0x4000
    str     x0, [sp, #0]
    mov     x0, sp
    add     x1, sp, #0x1000
    add     x2, sp, #8
    add     x3, sp, #16
    add     x4, sp, #24
    ldr     x8, [x21, #56]
    blr     x8
    mov     x0, x19
    ldr     x1, [sp, #8]
    ldr     x8, [x21, #232]
    blr     x8
    cbz     x0, 2f
    subs    x25, x25, #1
    b.ne    1b
3:  wfe
    b       3b

2:  msr     daifset, #0xf
    adrp    x22, _pe_head
    add     x22, x22, :lo12:_pe_head
    ldr     x23, =_file_size
    mov     x4, x22
    add     x5, x22, x23
4:  dc      cvac, x4
    add     x4, x4, #64
    cmp     x4, x5
    b.lo    4b
    dsb     sy
    mrs     x4, sctlr_el1
    bic     x4, x4, #(1 << 0)
    bic     x4, x4, #(1 << 2)
    bic     x4, x4, #(1 << 12)
    msr     sctlr_el1, x4
    isb
    mov     x0, #0x40000000
    cmp     x22, x0
    b.eq    6f
    mov     x1, x22
    mov     x2, x23
5:  ldp     x3, x4, [x1], #16
    stp     x3, x4, [x0], #16
    subs    x2, x2, #16
    b.gt    5b
6:  ic      iallu
    dsb     sy
    isb
    ldr     x9, =_start
    br      x9
