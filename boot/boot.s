// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
.section .text.boot
.global _start
_start:
    ldr x0, =_stack_top
    mov sp, x0
    // Zero .bss — required for firmware boot (EFI pages are dirty)
    ldr x1, =_bss_start
    ldr x2, =_bss_end
1:  cmp x1, x2
    b.hs 2f
    str xzr, [x1], #8
    b 1b
2:  bl aixos_main
    b .
