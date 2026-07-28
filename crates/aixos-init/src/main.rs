// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
#![no_std]
#![no_main]
#![cfg(not(test))]
#![allow(clippy::empty_loop)]
use core::panic::PanicInfo;

// run_desktop_loop is defined in lib.rs (same crate)
// Call it directly without crate:: prefix
#[no_mangle]
pub extern "C" fn aixos_main() -> ! {
    run_desktop_loop()
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}
