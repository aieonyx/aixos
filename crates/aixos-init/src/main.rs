// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
#![no_std]
#![no_main]
#![cfg(not(test))]
#![allow(clippy::empty_loop)]
use core::panic::PanicInfo;
const UART0: *mut u8 = 0x09000000 as *mut u8;
fn uart_write(s: &str) {
    for b in s.bytes() {
        unsafe { core::ptr::write_volatile(UART0, b); }
    }
}
#[no_mangle]
pub extern "C" fn aixos_main() -> ! {
    uart_write("aiXos Phoenix - Sovereign Stack Initializing...\n");
    aixos_init::orchestrate();
    uart_write("axon_main() -> 0x4153\n");
    // Brief delay for display initialization
    let mut delay = 0u64;
    while delay < 10_000_000 { delay += 1; }
    match aixos_gpu::init() {
        Some(_) => {
            uart_write("GPU: ok\n");
            aixos_gpu::desktop::render_desktop();
            aixos_gpu::desktop::render_status_bar(
                "aiXos Phoenix  |  axon_main() -> 0x4153  |  Sovereign");
            aixos_gpu::desktop::render_dock();
            uart_write("Desktop rendered\n");
        }
        None => { uart_write("GPU: none\n"); }
    }
    let _ = aixos_input::init();
    uart_write("axos> \n");
    // Keep OS alive - write heartbeat to UART every ~1 second
    let mut count = 0u64;
    loop {
        count = count.wrapping_add(1);
        if count % 100_000_000 == 0 {
            uart_write(".");
        }
    }
}
#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    uart_write("aiXos: panic\n");
    loop {}
}
