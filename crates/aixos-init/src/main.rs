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
        b"help" => "help clear version db window settings close reboot",
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
        b"db" => {
            if aixos_edisondb::is_live() {
                "EdisonDB: live | sovereign store active"
            } else {
                "EdisonDB: stub"
            }
        }
        b"window" => {
            unsafe {
                if let Some(i) = find_kind(0) {
                    ACTIVE_WIN = i;
                } else {
                    let slot = find_free().unwrap_or(0);
                    wins()[slot].open = true;
                    wins()[slot].kind = 0;
                    ACTIVE_WIN = slot;
                }
                render_all_windows();
                "window opened"
            }
        }
        b"settings" => {
            unsafe {
                if let Some(i) = find_kind(3) {
                    ACTIVE_WIN = i;
                } else {
                    let slot = find_free().unwrap_or(0);
                    wins()[slot].open = true;
                    wins()[slot].kind = 3;
                    ACTIVE_WIN = slot;
                }
                render_all_windows();
                "settings opened"
            }
        }
        b"close" => {
            unsafe {
                if wins()[ACTIVE_WIN].open {
                    let w = wins()[ACTIVE_WIN];
                    aixos_gpu::desktop::set_window_pos(w.x, w.y);
                    aixos_gpu::desktop::clear_window();
                    wins()[ACTIVE_WIN].open = false;
                    WINDOW_FOCUSED = false;
                    let mut i = 4;
                    while i > 0 {
                        i -= 1;
                        if wins()[i].open { ACTIVE_WIN = i; break; }
                    }
                    render_all_windows();
                    "window closed"
                } else {
                    "no window open"
                }
            }
        }
        b"" => "",
        _ => "axos: command not found",
    }
}


#[derive(Clone, Copy)]
struct WinSlot { open: bool, kind: u8, x: i32, y: i32, w: u32, h: u32 }
static mut WINS: [WinSlot; 4] = [
    WinSlot { open: false, kind: 0, x: 60,  y: 80,  w: 580, h: 300 },
    WinSlot { open: false, kind: 0, x: 100, y: 100, w: 580, h: 300 },
    WinSlot { open: false, kind: 0, x: 140, y: 120, w: 580, h: 300 },
    WinSlot { open: false, kind: 0, x: 180, y: 140, w: 580, h: 300 },
];
static mut ACTIVE_WIN: usize = 0;
static mut DRAG_WIN: usize = 0;
static mut WIN_BUF: ShellBuf = ShellBuf::new();
static mut WINDOW_FOCUSED: bool = false;
static mut WIN_OUTPUT: [&str; 8] = [""; 8];
static mut WIN_OUTPUT_LEN: usize = 0;
static mut ECHO_BUFS: [[u8; 72]; 8] = [[0; 72]; 8];
static mut ECHO_NEXT: usize = 0;
static mut DRAG_ACTIVE: bool = false;
static mut DRAG_OFF_X: i32 = 0;
static mut DRAG_OFF_Y: i32 = 0;
static mut RESIZE_ACTIVE: bool = false;
static mut RESIZE_WIN: usize = 0;

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
    aixos_edisondb::init();
    aixos_edisondb::write("boot:node_id", aixos_identity::node_id(), aixos_edisondb::Tier::Critical);
    aixos_edisondb::log_event("boot:desktop_ready");
    if aixos_edisondb::is_live() {
        uart_write("EdisonDB: live\n");
    }

    match aixos_gpu::init() {
        Some(_) => {
            uart_write("GPU: ok\n");
            aixos_gpu::desktop::render_desktop();
            aixos_gpu::desktop::render_top_bar_icons();
            {
                let slots = unsafe {[
                    (wins()[0].open, wins()[0].kind),
                    (wins()[1].open, wins()[1].kind),
                    (wins()[2].open, wins()[2].kind),
                    (wins()[3].open, wins()[3].kind),
                ]};
                aixos_gpu::desktop::render_taskbar(&slots, unsafe { ACTIVE_WIN });
            }
            aixos_gpu::desktop::render_status_bar("aiXos Phoenix : axon_main() -> 0x4153 : Sovereign");
            uart_write("Desktop rendered\n");
        }
        None => { uart_write("GPU: none\n"); }
    }

    let kbd = aixos_input::init();
    virtio_ok = kbd.is_some();
    if virtio_ok {
        uart_write("Input: virtio+uart\n");
    } else {
        uart_write("Input: uart only\n");
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

fn wins() -> &'static mut [WinSlot; 4] {
    unsafe { &mut *core::ptr::addr_of_mut!(WINS) }
}

fn any_open() -> bool {
    wins().iter().any(|w| w.open)
}

fn find_kind(kind: u8) -> Option<usize> {
    wins().iter().position(|w| w.open && w.kind == kind)
}

fn find_free() -> Option<usize> {
    wins().iter().position(|w| !w.open)
}

fn active_kind() -> u8 {
    unsafe { wins()[ACTIVE_WIN].kind }
}

fn render_window_for_slot(i: usize) {
    let w = wins()[i];
    if !w.open {
        return;
    }
    aixos_gpu::desktop::set_window_pos(w.x, w.y);
    match w.kind {
        1 => {
            aixos_gpu::desktop::render_window(
                "Shell - aiXos Phoenix",
                &["axos> sovereign shell", "type commands below", "",
                  "PL-24: shell window stub"],
                w.w, w.h);
            unsafe {
                let focused = WINDOW_FOCUSED && ACTIVE_WIN == i;
                aixos_gpu::desktop::render_window_output_hw(w.x, w.y, win_output(), WIN_OUTPUT_LEN, w.h, w.w);
                let b = win_buf();
                aixos_gpu::desktop::render_window_input_hw(w.x, w.y, b.as_slice(), b.len, focused, w.h, w.w);
            }
        }
        2 => aixos_gpu::desktop::render_window(
            "EdisonDB - Sovereign Store",
            &["Status: live", "Entries: (see db command)",
              "boot:proof = 0x4153", "boot:node_id = stored",
              "Tier: Critical / Personal / Noise"],
            w.w, w.h),
        3 => aixos_gpu::desktop::render_window(
            "Settings - aiXos Phoenix",
            &["Display:  ramfb 1280x720  FORMAT_XR24",
              "System:   aiXos Phoenix v0.1.0  aarch64",
              "Proof:    axon_main() -> 0x4153 [SOVEREIGN]",
              "Store:    EdisonDB live  sovereign store",
              "Input:    virtio+uart",
              "About:    AIEONYX  Sovereign Digital Infrastructure"],
            w.w, w.h),
        _ => aixos_gpu::desktop::render_window(
            "Sovereign Node - aiXos Phoenix",
            &["aiXos Phoenix v0.1.0", "Arch: aarch64 (QEMU virt)",
              "Proof: 0x4153 [SOVEREIGN]", "type close to dismiss"],
            w.w, w.h),
    }
}

fn render_windows_only() {
    let active = unsafe { ACTIVE_WIN };
    let mut i = 0;
    while i < 4 {
        if i != active { render_window_for_slot(i); }
        i += 1;
    }
    render_window_for_slot(active);
    let slots = unsafe {[
        (wins()[0].open, wins()[0].kind),
        (wins()[1].open, wins()[1].kind),
        (wins()[2].open, wins()[2].kind),
        (wins()[3].open, wins()[3].kind),
    ]};
    aixos_gpu::desktop::render_taskbar(&slots, unsafe { ACTIVE_WIN });
}

fn render_all_windows() {
    aixos_gpu::desktop::render_desktop();
    aixos_gpu::desktop::render_top_bar_icons();
    aixos_gpu::desktop::render_status_bar("aiXos Phoenix : axon_main() -> 0x4153 : Sovereign");
    let active = unsafe { ACTIVE_WIN };
    let mut i = 0;
    while i < 4 {
        if i != active {
            render_window_for_slot(i);
        }
        i += 1;
    }
    render_window_for_slot(active);
    let slots = unsafe {[
        (wins()[0].open, wins()[0].kind),
        (wins()[1].open, wins()[1].kind),
        (wins()[2].open, wins()[2].kind),
        (wins()[3].open, wins()[3].kind),
    ]};
    aixos_gpu::desktop::render_taskbar(&slots, unsafe { ACTIVE_WIN });
}


fn handle_dock_click(x: i32, y: i32) {
    if let Some(icon) = aixos_gpu::desktop::dock_icon_at(x, y) {
        unsafe {
            WINDOW_FOCUSED = false;
            if let Some(i) = find_kind(icon) {
                ACTIVE_WIN = i;
            } else {
                let slot = find_free().unwrap_or(0);
                wins()[slot].open = true;
                wins()[slot].kind = icon;
                ACTIVE_WIN = slot;
            }
        }
        render_all_windows();
    }
}

fn win_buf() -> &'static mut ShellBuf {
    unsafe { &mut *core::ptr::addr_of_mut!(WIN_BUF) }
}

fn win_output() -> &'static [&'static str] {
    unsafe { &(&*core::ptr::addr_of!(WIN_OUTPUT))[..] }
}

fn push_output(line: &'static str) {
    unsafe {
        let out = &mut *core::ptr::addr_of_mut!(WIN_OUTPUT);
        if WIN_OUTPUT_LEN >= 8 {
            let mut i = 0;
            while i < 7 { out[i] = out[i + 1]; i += 1; }
            out[7] = line;
        } else {
            out[WIN_OUTPUT_LEN] = line;
            WIN_OUTPUT_LEN += 1;
        }
    }
}

fn push_echo() -> &'static str {
    unsafe {
        let i = ECHO_NEXT;
        ECHO_NEXT = (ECHO_NEXT + 1) % 8;
        let bufs = &mut *core::ptr::addr_of_mut!(ECHO_BUFS);
        let bytes = win_buf().as_slice();
        let n = if bytes.len() > 67 { 67 } else { bytes.len() };
        bufs[i][..5].copy_from_slice(b"win> ");
        bufs[i][5..5 + n].copy_from_slice(&bytes[..n]);
        core::str::from_utf8_unchecked(&(&*core::ptr::addr_of!(ECHO_BUFS))[i][..5 + n])
    }
}

fn handle_window_key(code: u16, ch: Option<char>) {
    let (wx, wy) = {
        let w = wins()[unsafe { ACTIVE_WIN }];
        aixos_gpu::desktop::set_window_pos(w.x, w.y);
        (w.x, w.y)
    };
    match code {
        1 => unsafe {
            WINDOW_FOCUSED = false;
            win_buf().clear();
            aixos_gpu::desktop::render_window_input(wx, wy, &[], 0, false);
        },
        28 => unsafe {
            let echo = push_echo();
            push_output(echo);
            let result = execute_cmd(win_buf());
            push_output(result);
            win_buf().clear();
            if wins()[ACTIVE_WIN].open && wins()[ACTIVE_WIN].kind == 1 {
                aixos_gpu::desktop::render_window_output(wx, wy, win_output(), WIN_OUTPUT_LEN);
                aixos_gpu::desktop::render_window_input(wx, wy, &[], 0, WINDOW_FOCUSED);
            }
        },
        14 => unsafe {
            win_buf().pop();
            let b = win_buf();
            aixos_gpu::desktop::render_window_input(wx, wy, b.as_slice(), b.len, WINDOW_FOCUSED);
        },
        _ => {
            if let Some(c) = ch {
                win_buf().push(c as u8);
                let b = win_buf();
                unsafe {
                    aixos_gpu::desktop::render_window_input(wx, wy, b.as_slice(), b.len, WINDOW_FOCUSED);
                }
            }
        }
    }
}

fn handle_click(x: i32, y: i32) {
    unsafe {
        let order = [ACTIVE_WIN, 3, 2, 1, 0];
        let mut k = 0;
        while k < 5 {
            let i = order[k];
            k += 1;
            if k > 1 && i == order[0] { continue; }
            let w = wins()[i];
            if !w.open { continue; }
            if x >= w.x + w.w as i32 - 20 && x < w.x + w.w as i32
                && y >= w.y + w.h as i32 - 20 && y < w.y + w.h as i32 {
                ACTIVE_WIN = i;
                RESIZE_WIN = i;
                RESIZE_ACTIVE = true;
                render_all_windows();
                return;
            }
            if x >= w.x && x < w.x + w.w as i32 && y >= w.y && y < w.y + 24 {
                ACTIVE_WIN = i;
                if x >= w.x + w.w as i32 - 62 && x < w.x + w.w as i32 {
                    wins()[i].open = false;
                    WINDOW_FOCUSED = false;
                    aixos_gpu::desktop::set_window_pos(w.x, w.y);
                    aixos_gpu::desktop::clear_window();
                    let mut j = 4;
                    while j > 0 { j -= 1; if wins()[j].open { ACTIVE_WIN = j; break; } }
                    render_all_windows();
                    return;
                }
                DRAG_WIN = i;
                DRAG_ACTIVE = true;
                DRAG_OFF_X = x - w.x;
                DRAG_OFF_Y = y - w.y;
                render_all_windows();
                return;
            }
            if x >= w.x && x < w.x + w.w as i32 && y >= w.y + 24 && y < w.y + w.h as i32 {
                ACTIVE_WIN = i;
                if w.kind == 1 {
                    WINDOW_FOCUSED = true;
                }
                render_all_windows();
                return;
            }
        }
        if y > 50 && y < 670 && x > 0 && x < 1280 {
            WINDOW_FOCUSED = false;
            if let Some(i) = find_kind(0) {
                ACTIVE_WIN = i;
            } else {
                let slot = find_free().unwrap_or(0);
                wins()[slot].open = true;
                wins()[slot].kind = 0;
                ACTIVE_WIN = slot;
            }
            render_all_windows();
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
                unsafe {
                    if RESIZE_ACTIVE && !mouse_state.left {
                        // Apply resize only on release — no intermediate frames
                        let nw = ((mouse_state.x - wins()[RESIZE_WIN].x) as u32).clamp(300, 900);
                        let nh = ((mouse_state.y - wins()[RESIZE_WIN].y) as u32).clamp(200, 600);
                        wins()[RESIZE_WIN].w = nw;
                        wins()[RESIZE_WIN].h = nh;
                        RESIZE_ACTIVE = false;
                        DRAG_ACTIVE = false;
                        render_all_windows();
                    }
                    if !mouse_state.left { RESIZE_ACTIVE = false; DRAG_ACTIVE = false; }
                    const DRAG_MIN_X: i32 = 0; const DRAG_MAX_X: i32 = 700;
                    if !RESIZE_ACTIVE && DRAG_ACTIVE && mouse_state.left {
                        let dw = DRAG_WIN;
                        let w = wins()[dw];
                        let nx = (mouse_state.x - DRAG_OFF_X).clamp(DRAG_MIN_X, DRAG_MAX_X);
                        let ny = (mouse_state.y - DRAG_OFF_Y).clamp(50, 580);
                        if nx != w.x || ny != w.y {
                            // Erase old position before moving
                            aixos_gpu::desktop::set_window_pos(w.x, w.y);
                            aixos_gpu::desktop::clear_window_sized(w.w + 10, w.h + 10);
                            wins()[dw].x = nx;
                            wins()[dw].y = ny;
                            render_windows_only();
                        }
                    }
                    if !mouse_state.left { DRAG_ACTIVE = false; }
                }
                aixos_gpu::draw_cursor(mouse_state.x, mouse_state.y);
                if mouse_state.left && !prev_left {
                    if mouse_state.y < 50 {
                        handle_dock_click(mouse_state.x, mouse_state.y);
                    } else {
                        handle_click(mouse_state.x, mouse_state.y);
                    }
                }
            }
        }
        if let Some(ev) = aixos_input::poll() {
            handle_key(&mut buf, ev.code, ev.ch);
        }
    }
}

fn handle_key(buf: &mut ShellBuf, code: u16, ch: Option<char>) {
    unsafe {
        if WINDOW_FOCUSED && wins()[ACTIVE_WIN].open && wins()[ACTIVE_WIN].kind == 1 {
            handle_window_key(code, ch);
            return;
        }
    }
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
