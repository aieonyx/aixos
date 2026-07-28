// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// Phoenix-Desktop PD -- Microkit wrapper for aiXos sovereign desktop
#![no_std]
#![no_main]

#[no_mangle]
#[link_section = ".bss"]
pub static mut __sel4_ipc_buffer_obj: [u8; 4096] = [0u8; 4096];

const UART: *mut u8 = 0x09000000 as *mut u8;

fn uart_write(s: &[u8]) {
    for &b in s { unsafe { UART.write_volatile(b); } }
}

#[no_mangle]
pub extern "C" fn init() {
    uart_write(b"[Phoenix-Desktop] seL4 PD live proof=0x4153\r\n");
    let proof = aixos_init::orchestrate();
    if proof == 0x4153u64 {
        uart_write(b"[Phoenix-Desktop] sovereign proof CONFIRMED\r\n");
    } else {
        uart_write(b"[Phoenix-Desktop] WARNING: proof mismatch\r\n");
    }
    uart_write(b"[Phoenix-Desktop] entering sovereign desktop loop\r\n");
    unsafe { aixos_init::run_desktop_loop() }
}

#[no_mangle]
pub extern "C" fn notified(_ch: u8) {}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    uart_write(b"[Phoenix-Desktop] PANIC\r\n");
    loop {}
}
