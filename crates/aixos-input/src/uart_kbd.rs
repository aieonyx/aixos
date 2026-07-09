// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// PL011 UART keyboard input

const UART_BASE: usize = 0x09000000;
const UART_DR:   *mut u8  = UART_BASE as *mut u8;
const UART_FR:   *const u32 = (UART_BASE + 0x018) as *const u32;
const UART_FR_RXFE: u32 = 1 << 4; // Receive FIFO empty

/// Read one byte from PL011 UART if available.
/// On aarch64 bare-metal: polls the real MMIO register.
/// On host (test builds): always returns None — no hardware present.
pub fn read_byte() -> Option<u8> {
    #[cfg(target_arch = "aarch64")]
    unsafe {
        let fr = core::ptr::read_volatile(UART_FR);
        if fr & UART_FR_RXFE != 0 { return None; }
        Some(core::ptr::read_volatile(UART_DR))
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        None
    }
}
