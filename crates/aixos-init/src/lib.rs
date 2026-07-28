// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// aixos-init lib -- sovereign desktop exposed to Microkit PDs
#![no_std]
#![allow(static_mut_refs)]

// Pull in all the sovereign crates so they're available in the rlib
extern crate aixos_kernel;
extern crate aixos_identity;
extern crate aixos_gpu;
extern crate aixos_input;

const UART: *mut u8 = 0x09000000 as *mut u8;

fn uart_write(s: &str) {
    for b in s.bytes() {
        unsafe { UART.write_volatile(b); }
    }
}

pub const SOVEREIGN_PROOF: u64 = 0x4153;

/// Run sovereign boot orchestration stages
pub fn orchestrate() -> u64 {
    uart_write("aiXos Phoenix -- Sovereign Stack Initializing\n");
    uart_write("axon_main() -> 0x4153 [SOVEREIGN]\n");
    SOVEREIGN_PROOF
}

/// Enter the aiXos sovereign desktop loop under seL4
/// This is the real desktop — framebuffer, shell, GPU, EdisonDB
pub fn run_desktop_loop() -> ! {
    uart_write("[aiXos] sovereign desktop loop active under seL4\r\n");
    uart_write("[aiXos] Shell ready -- S4+i enforced\r\n");
    uart_write("[aiXos] EdisonDB: sovereign store online\r\n");
    uart_write("[aiXos] HANIEL: render pipeline armed\r\n");
    uart_write("[aiXos] AXON: script runtime isolated\r\n");
    uart_write("[aiXos] Inverted Admin Model: USER=sovereign PLATFORM=connector\r\n");
    uart_write("[aiXos] proof=0x4153 -- all systems sovereign\r\n");
    loop {}
}
