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

struct ShellBuf { data: [u8; 64], len: usize }

impl ShellBuf {
    const fn new() -> Self { ShellBuf { data: [0u8; 64], len: 0 } }
    fn push(&mut self, b: u8) -> bool {
        if self.len < self.data.len() {
            self.data[self.len] = b; self.len += 1; true
        } else { false }
    }
    fn pop(&mut self) -> bool {
        if self.len > 0 { self.len -= 1; true } else { false }
    }
    fn clear(&mut self) { self.len = 0; }
    fn as_slice(&self) -> &[u8] { &self.data[..self.len] }
}

fn execute_cmd(buf: &ShellBuf) -> &'static str {
    let cmd = buf.as_slice();
    match cmd {
        b"help" =>
            "help clear version sovereignty node-id awp-status mem window close reboot",
        b"clear" => "axos> ",
        b"version" => "aiXos Phoenix v0.1.0 — Sovereign Stack",
        b"sovereignty" =>
            "S4+i: Security Sovereignty Simplicity Speed +Intelligence",
        b"node-id" => "node-id: 0x0000000000000000 [ARPi pending]",
        b"awp-status" => "AWP: stub — not yet on packet path",
        b"mem" => "RAM: 512M  FB: 4M@0x44000000  Stack: 64K",
        b"reboot" => {
            uart_write("axos> reboot\n");
            loop {}
        }
        b"window" => {
            unsafe { WINDOW_OPEN = true; }
            aixos_gpu::desktop::render_window(
                "Sovereign Node — aiXos Phoenix",
                &["aiXos Phoenix v0.1.0","Arch:    aarch64 (QEMU virt)",
                  "Proof:   0x4153 [SOVEREIGN]","Node:    a1e04851 40100001",
                  "ARPi:    active  AWP: lite-live","Input:   virtio+uart",
                  "Display: ramfb 1280x720","","type close to dismiss"]);
            "window opened"
        }
        b"close" => {
            unsafe { if WINDOW_OPEN { WINDOW_OPEN = false;
                aixos_gpu::desktop::clear_window(); "window closed"
                } else { "no window open" } }
        }
        b"" => "",
        _ => "axos: command not found",
    }
}

static mut WINDOW_OPEN: bool = false;

#[no_mangle]
pub extern "C" fn aixos_main() -> ! {
    uart_write("aiXos Phoenix - Sovereign Stack Initializing...\n");

    let proof = aixos_init::orchestrate();
    if proof == 0x4153 {
        uart_write("axon_main() -> 0x4153 [SOVEREIGN]\n");
    } else {
        uart_write("axon_main() -> boot incomplete\n");
    }

    let mut delay = 0u64;
    while delay < 10_000_000 { delay += 1; }

    let virtio_ok;
    match aixos_gpu::init() {
        Some(_) => {
            uart_write("GPU: ok\n");
            aixos_gpu::desktop::render_desktop();
            // Render panel content immediately after desktop
            aixos_gpu::desktop::render_left_panel(proof, aixos_identity::node_id());
            aixos_gpu::desktop::render_right_panel();
            // Status bar reflects actual proof state
            if proof == 0x4153 {
                aixos_gpu::desktop::render_status_bar(
                    "aiXos Phoenix  |  axon_main() -> 0x4153  |  Sovereign");
            } else {
                aixos_gpu::desktop::render_status_bar(
                    "aiXos Phoenix  |  boot incomplete  |  check PDs");
            }
            aixos_gpu::desktop::render_dock();
            uart_write("Desktop rendered\n");
        }
        None => { uart_write("GPU: none\n"); }
    }

    let kbd = aixos_input::init();
    virtio_ok = kbd.is_some();
    if virtio_ok {
        uart_write("Input: virtio+uart\n");
        aixos_gpu::desktop::render_right_panel_input(true);
    } else {
        uart_write("Input: uart only\n");
        aixos_gpu::desktop::render_right_panel_input(false);
    }

    let mut mouse = aixos_input::mouse::init();
    let mut mouse_state = aixos_input::mouse::MouseState { x: 640, y: 360, left: false, right: false };
    if mouse.is_some() {
        uart_write("Mouse: virtio-tablet\n");
        aixos_gpu::draw_cursor(mouse_state.x, mouse_state.y);
    } else {
        uart_write("Mouse: none\n");
    }
    uart_write("axos> ");
    shell_loop(mouse, mouse_state);
}

fn handle_click(x: i32, y: i32) {
    if y > 40 && y < 670 && x > 200 && x < 1080 {
        unsafe {
            if !WINDOW_OPEN {
                WINDOW_OPEN = true;
                aixos_gpu::desktop::render_window(
                    "Sovereign Node \u{2014} aiXos Phoenix",
                    &["aiXos Phoenix v0.1.0", "Arch: aarch64 (QEMU virt)",
                      "Proof: 0x4153 [SOVEREIGN]", "type close to dismiss"]);
            }
        }
    }
}

fn shell_loop(
    mut mouse: Option<aixos_input::mouse::VirtioMouse>,
    mut mouse_state: aixos_input::mouse::MouseState,
) -> ! {
    let mut buf = ShellBuf::new();
    aixos_gpu::desktop::render_input_line(b"", 0);
    loop {
        if let Some(ref mut m) = mouse {
            let old_x = mouse_state.x;
            let old_y = mouse_state.y;
            let prev_left = mouse_state.left;
            if m.poll(&mut mouse_state) {
                aixos_gpu::erase_cursor(old_x, old_y);
                aixos_gpu::draw_cursor(mouse_state.x, mouse_state.y);
                if mouse_state.left && !prev_left {
                    handle_click(mouse_state.x, mouse_state.y);
                }
            }
        }
        if let Some(ev) = aixos_input::poll() {
            handle_key(&mut buf, ev.code, ev.ch);
        }
    }
}

fn handle_key(buf: &mut ShellBuf, code: u16, ch: Option<char>) {
    match code {
        28 => {
            uart_write("\n");
            let result = execute_cmd(buf);
            if !result.is_empty() {
                uart_write(result);
                uart_write("\n");
                aixos_gpu::desktop::render_command_result(result);
            }
            buf.clear();
            let mut d = 0u64;
            while d < 5_000_000 { d += 1; }
            aixos_gpu::desktop::render_input_line(b"", 0);
            uart_write("axos> ");
        }
        1 => {
            buf.clear();
            aixos_gpu::desktop::render_input_line(b"", 0);
            uart_write_byte(b'\r');
            uart_write("axos> ");
        }
        14 => {
            if buf.pop() {
                uart_write_byte(0x08);
                uart_write_byte(b' ');
                uart_write_byte(0x08);
                aixos_gpu::desktop::render_input_line(buf.as_slice(), buf.len);
            }
        }
        _ => {
            if let Some(c) = ch {
                let b = c as u8;
                if (0x20..0x7fu8).contains(&b) {
                    if buf.push(b) {
                        uart_write_byte(b);
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
