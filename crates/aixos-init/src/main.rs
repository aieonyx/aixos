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

fn uart_write_byte(b: u8) {
    unsafe { core::ptr::write_volatile(UART0, b); }
}

/// Window open state
static mut WINDOW_OPEN: bool = false;

/// Shell command buffer — fixed 64 bytes, no heap required.
struct ShellBuf {
    data: [u8; 64],
    len: usize,
}

impl ShellBuf {
    const fn new() -> Self { ShellBuf { data: [0u8; 64], len: 0 } }

    fn push(&mut self, b: u8) -> bool {
        if self.len < self.data.len() {
            self.data[self.len] = b;
            self.len += 1;
            true
        } else {
            false
        }
    }

    fn pop(&mut self) -> bool {
        if self.len > 0 { self.len -= 1; true } else { false }
    }

    fn clear(&mut self) { self.len = 0; }

    fn as_slice(&self) -> &[u8] { &self.data[..self.len] }
}

/// Execute a typed command. Returns a static string for display.
fn execute_cmd(buf: &ShellBuf) -> &'static str {
    let cmd = buf.as_slice();
    if cmd == b"help" {
        "help  clear  version  sovereignty  node-id  awp-status  mem  window  close  reboot"
    } else if cmd == b"clear" {
        // Signal clear — handled by render path
        "axos> "
    } else if cmd == b"version" {
        "aiXos Phoenix v0.1.0 — Sovereign Stack"
    } else if cmd == b"reboot" {
        uart_write("axos> reboot\n");
        // Spin until QEMU exits. With -no-reboot in run-qemu.sh, QEMU
        // terminates when the guest halts — no PSCI call needed at this stage.
        loop {}
    } else if cmd == b"window" {
        unsafe { WINDOW_OPEN = true; }
        aixos_gpu::desktop::render_window(
            "Sovereign Node — aiXos Phoenix",
            &[
                "aiXos Phoenix v0.1.0",
                "Arch:    aarch64 (QEMU virt)",
                "Proof:   0x4153 [SOVEREIGN]",
                "Node:    a1e04851 40100001",
                "ARPi:    active  AWP: lite-live",
                "Input:   virtio+uart",
                "Display: ramfb 1280x720",
                "",
                "type close to dismiss",
            ]
        );
        "window opened"
    } else if cmd == b"close" {
        unsafe {
            if WINDOW_OPEN {
                WINDOW_OPEN = false;
                aixos_gpu::desktop::clear_window();
                "window closed"
            } else {
                "no window open"
            }
        }
    } else if cmd.is_empty() {
        ""
    } else {
        "axos: command not found"
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

    // Initialize input — v1 fix applied inside init_device()
    let _kbd = aixos_input::init();
    uart_write("axos> \n");

    // PL-14: Shell loop — reads from UART (stdin pipe via -serial stdio)
    // This is the primary path. virtio-input is polled as secondary.
    shell_loop();
}

fn shell_loop() -> ! {
    let mut buf = ShellBuf::new();

    // Render initial empty prompt on framebuffer
    aixos_gpu::desktop::render_input_line(b"", 0);

    loop {
        // Poll both UART and virtio-input each iteration
        if let Some(ev) = aixos_input::poll() {
            handle_key(&mut buf, ev.code, ev.ch);
        }
    }
}

fn handle_key(buf: &mut ShellBuf, code: u16, ch: Option<char>) {
    match code {
        28 /* ENTER / \r / \n */ => {
            // Echo newline to UART
            uart_write("\n");
            let result = execute_cmd(buf);
            if !result.is_empty() {
                uart_write(result);
                uart_write("\n");
                // Render result on dock bar (teal colour)
                aixos_gpu::desktop::render_command_result(result);
            }
            buf.clear();
            // Small pause then restore prompt
            let mut d = 0u64;
            while d < 5_000_000 { d += 1; }
            aixos_gpu::desktop::render_input_line(b"", 0);
            uart_write("axos> ");
        }
        1 /* ESC */ => {
            // Clear buffer and re-render prompt
            buf.clear();
            aixos_gpu::desktop::render_input_line(b"", 0);
            uart_write_byte(b'\r');
            uart_write("axos> ");
        }
        14 /* BACKSPACE */ => {
            if buf.pop() {
                // Erase last character on UART: \x08 space \x08
                uart_write_byte(0x08);
                uart_write_byte(b' ');
                uart_write_byte(0x08);
                aixos_gpu::desktop::render_input_line(buf.as_slice(), buf.len);
            }
        }
        _ => {
            if let Some(c) = ch {
                let b = c as u8;
                if b >= 0x20 && b < 0x7f {
                    if buf.push(b) {
                        uart_write_byte(b); // local echo
                        aixos_gpu::desktop::render_input_line(buf.as_slice(), buf.len);
                    }
                }
            }
        }
    }
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    uart_write("aiXos: panic\n");
    loop {}
}

