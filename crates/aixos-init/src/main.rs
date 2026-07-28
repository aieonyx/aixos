// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
#![no_std]
#![no_main]
#![cfg(not(test))]
#![allow(clippy::empty_loop)]
use core::panic::PanicInfo;

// Import from this crate's lib.rs
use aixos_init::run_desktop_loop;

#[no_mangle]
pub extern "C" fn aixos_main() -> ! {
    unsafe { run_desktop_loop() }
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}
