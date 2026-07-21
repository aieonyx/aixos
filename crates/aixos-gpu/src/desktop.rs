// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0

use crate::draw::{draw_rect, draw_border, draw_hline, blend_rect};
use crate::framebuffer::cache_flush;
use crate::font::{draw_str, draw_str_2x, draw_str_clipped};

const DARK_BG:          u32 = 0x0A0A1A;
const PANEL_BG:         u32 = 0x141428;
const PANEL_BORDER:     u32 = 0x2A2A4A;
const TEXT_WHITE:       u32 = 0xEEEEFF;
const TEXT_DIM:         u32 = 0x666688;
const TOP_BAR:          u32 = 0x080818;
const DOCK_BG:          u32 = 0x0D0D20;
const SOVEREIGN_PURPLE: u32 = 0x7B4FDB;
const ACCENT_TEAL:      u32 = 0x1BAF7A;
const ACCENT_AMBER:     u32 = 0xD4A017;
const SETTINGS_BLUE:    u32 = 0x1B7FC4;
const TOP_BAR_H: u32 = 50;
const TASKBAR_Y: u32 = 670;
const TASKBAR_H: u32 = 50;
const CANVAS_Y: u32 = 50;
const CANVAS_H: u32 = 620;

pub fn render_desktop() {
    // Full canvas — clear, flush cache, then redraw
    draw_rect(0, 0, 1280, 720, DARK_BG);
    cache_flush();
    draw_rect(0, 0, 1280, 720, DARK_BG);
    blend_rect(0, CANVAS_Y, 1280, CANVAS_H, SOVEREIGN_PURPLE, 28);
    // Top bar
    draw_rect(0, 0, 1280, TOP_BAR_H, TOP_BAR);
    draw_hline(0, TOP_BAR_H, 1280, PANEL_BORDER);
    // Taskbar
    draw_rect(0, TASKBAR_Y, 1280, TASKBAR_H, DOCK_BG);
    draw_hline(0, TASKBAR_Y, 1280, PANEL_BORDER);
    // AIEONYX diamond centered at (640, 360)
    let cx: u32 = 640;
    let cy: u32 = 360;
    let mut i: u32 = 0;
    while i <= 20 {
        let w = i * 2 + 1;
        let x = cx.saturating_sub(i);
        let y = cy.saturating_sub(20).saturating_add(i);
        draw_hline(x, y, w, SOVEREIGN_PURPLE);
        i += 1;
    }
    let mut i: u32 = 1;
    while i <= 20 {
        let w = (20 - i) * 2 + 1;
        let x = cx.saturating_sub(20 - i);
        let y = cy + i;
        draw_hline(x, y, w, SOVEREIGN_PURPLE);
        i += 1;
    }
    // Teal indicator dot top-right
    draw_rect(1260, 18, 8, 8, ACCENT_TEAL);
}

pub fn render_status_bar(text: &str) {
    draw_str_2x(430, 16, text, TEXT_WHITE);
}

pub fn render_top_bar_icons() {
    // Node icon
    draw_rect(8, 8, 60, 34, ACCENT_TEAL);
    draw_str(16, 18, "Node", TEXT_WHITE);
    // Shell icon
    draw_rect(76, 8, 60, 34, ACCENT_AMBER);
    draw_str(82, 18, "Shell", TEXT_WHITE);
    // EDB icon
    draw_rect(144, 8, 60, 34, SOVEREIGN_PURPLE);
    draw_str(155, 18, "EDB", TEXT_WHITE);
    // Settings icon
    draw_rect(212, 8, 60, 34, SETTINGS_BLUE);
    draw_str(222, 18, "Set", TEXT_WHITE);
}

pub fn render_taskbar(slots: &[(bool, u8)], active: usize) {
    draw_rect(0, TASKBAR_Y, 1280, TASKBAR_H, DOCK_BG);
    draw_hline(0, TASKBAR_Y, 1280, PANEL_BORDER);
    let names = ["Node", "Shell", "EDB", "Set"];
    let mut btn_x: u32 = 8;
    let mut i = 0;
    while i < 4 {
        if slots[i].0 {
            let kind = slots[i].1 as usize;
            let name = if kind < 4 { names[kind] } else { "Win" };
            let color = if i == active { ACCENT_TEAL } else { PANEL_BG };
            draw_rect(btn_x, TASKBAR_Y + 8, 110, 34, color);
            draw_border(btn_x, TASKBAR_Y + 8, 110, 34, PANEL_BORDER);
            draw_str(btn_x + 8, TASKBAR_Y + 20, name, TEXT_WHITE);
            btn_x += 118;
        }
        i += 1;
    }
    // axos> prompt at right
    draw_str(1100, TASKBAR_Y + 20, "axos>", TEXT_DIM);
}

/// Left panel — Sovereign Identity Space
/// Shows node identity, ARPi ceremony state, boot proof.


/// Right panel — System Space
/// Shows AWP status, EdisonDB state, input driver, display.


/// Update right panel input driver status after virtio init
pub fn render_right_panel_input(virtio_ok: bool) {
    draw_rect(1088, 152, 180, 12, PANEL_BG);
    if virtio_ok {
        draw_str(1088, 156, "virtio+uart", ACCENT_TEAL);
    } else {
        draw_str(1088, 156, "uart only", ACCENT_AMBER);
    }
}

pub fn render_input_line(buf: &[u8], len: usize) {
    draw_rect(340, 710, 600, 10, DOCK_BG);
    draw_str(348, 712, "axos> ", TEXT_DIM);
    let n = if len < buf.len() { len } else { buf.len() };
    crate::font::draw_bytes(398, 682, &buf[..n], TEXT_WHITE);
}

pub fn render_command_result(msg: &str) {
    draw_rect(340, 710, 600, 10, DOCK_BG);
    draw_str(348, 712, msg, ACCENT_TEAL);
}

#[allow(dead_code)]
const TEXT_DIM_2: u32 = 0x666688;

// ── PL-20: Sovereign Window Primitive ────────────────────────────────────────
const WIN_X: u32 = 340;
const WIN_Y: u32 = 110;
const WIN_W: u32 = 580;
const WIN_H: u32 = 300;
const WIN_TITLE_H: u32 = 24;
const WIN_BG:    u32 = 0x0D0D22;
const WIN_TITLE: u32 = 0x1A1A3A;

static mut CUR_WIN_X: i32 = 200;
static mut CUR_WIN_Y: i32 = 80;

pub fn set_window_pos(x: i32, y: i32) {
    unsafe { CUR_WIN_X = x; CUR_WIN_Y = y; }
}
pub fn get_window_pos() -> (i32, i32) {
    unsafe { (CUR_WIN_X, CUR_WIN_Y) }
}

pub fn dock_icon_at(x: i32, y: i32) -> Option<u8> {
    if y < 8 || y > 42 { return None; }
    if x >= 8 && x < 68 { return Some(0); }
    if x >= 76 && x < 136 { return Some(1); }
    if x >= 144 && x < 204 { return Some(2); }
    if x >= 212 && x < 272 { return Some(3); }
    None
}

pub fn render_window(title: &str, lines: &[&str], w: u32, h: u32) {
    let wx = unsafe { CUR_WIN_X as u32 };
    let wy = unsafe { CUR_WIN_Y as u32 };
    draw_rect(wx.saturating_sub(1), wy.saturating_sub(1), w + 2, h + 2, ACCENT_TEAL);
    draw_rect(wx, wy, w, WIN_TITLE_H, WIN_TITLE);
    let tx = wx + 8;
    let ty = wy + 12;
    draw_hline(tx,                   ty.saturating_sub(4), 1, SOVEREIGN_PURPLE);
    draw_hline(tx.saturating_sub(1), ty.saturating_sub(3), 3, SOVEREIGN_PURPLE);
    draw_hline(tx.saturating_sub(2), ty.saturating_sub(2), 5, SOVEREIGN_PURPLE);
    draw_hline(tx.saturating_sub(3), ty.saturating_sub(1), 7, SOVEREIGN_PURPLE);
    draw_hline(tx.saturating_sub(4), ty,                   9, SOVEREIGN_PURPLE);
    draw_hline(tx.saturating_sub(3), ty + 1,               7, SOVEREIGN_PURPLE);
    draw_hline(tx.saturating_sub(2), ty + 2,               5, SOVEREIGN_PURPLE);
    draw_hline(tx.saturating_sub(1), ty + 3,               3, SOVEREIGN_PURPLE);
    draw_hline(tx,                   ty + 4,               1, SOVEREIGN_PURPLE);
    draw_str(wx + 22, wy + 8, title, TEXT_WHITE);
    draw_str(wx + w - 62, wy + 8, "[close]", 0x666688);
    draw_hline(wx, wy + WIN_TITLE_H, w, ACCENT_TEAL);
    draw_rect(wx, wy + WIN_TITLE_H + 1, w, h - WIN_TITLE_H - 1, WIN_BG);
    let mut row = 0u32;
    for line in lines.iter().take(10) {
        draw_str(wx + 12, wy + WIN_TITLE_H + 12 + row * 18, line, TEXT_WHITE);
        row += 1;
    }
    draw_rect(wx + w - 12, wy + h - 12, 12, 12, ACCENT_TEAL);
    draw_rect(wx + w - 8, wy + h - 8, 4, 4, TEXT_WHITE);
}

pub fn render_window_output(wx: i32, wy: i32, lines: &[&'static str], count: usize) {
    render_window_output_h(wx, wy, lines, count, WIN_H);
}
pub fn render_window_output_h(wx: i32, wy: i32, lines: &[&'static str], count: usize, wh: u32) {
    render_window_output_hw(wx, wy, lines, count, wh, 578);
}
pub fn render_window_output_hw(wx: i32, wy: i32, lines: &[&'static str], count: usize, wh: u32, ww: u32) {
    let body_h = if wh > 45 { wh - 45 } else { 1 };
    draw_rect((wx + 1) as u32, (wy + 25) as u32, ww.saturating_sub(4), body_h, WIN_BG);
    let n = if count > 8 { 8 } else { count };
    let mut y = wy + 30;
    let mut idx = 0;
    while idx < n {
        draw_str_clipped((wx + 8) as u32, y as u32, lines[idx], TEXT_WHITE, (wx + 572) as u32);
        y += 18;
        idx += 1;
    }
}

pub fn render_window_input(wx: i32, wy: i32, buf: &[u8], len: usize, focused: bool) {
    render_window_input_h(wx, wy, buf, len, focused, WIN_H);
}
pub fn render_window_input_h(wx: i32, wy: i32, buf: &[u8], len: usize, focused: bool, wh: u32) {
    render_window_input_hw(wx, wy, buf, len, focused, wh, 580);
}
pub fn render_window_input_hw(wx: i32, wy: i32, buf: &[u8], len: usize, focused: bool, wh: u32, ww: u32) {
    let y = wy + wh as i32 - 20;
    let y = if y < wy + 30 { wy + 30 } else { y };
    draw_rect((wx + 4) as u32, (y - 2) as u32, ww.saturating_sub(8), 18, WIN_BG);
    draw_str((wx + 8) as u32, y as u32, "win> ", ACCENT_TEAL);
    if let Ok(txt) = core::str::from_utf8(&buf[..len]) {
        draw_str((wx + 48) as u32, y as u32, txt, TEXT_WHITE);
    }
    draw_str((wx + 48 + (len as i32) * 8) as u32, y as u32, "_", TEXT_WHITE);
    if focused {
        // Draw [focused] inside window right edge
        let fx = (wx as u32 + ww).saturating_sub(80);
        draw_str(fx, y as u32, "[focused]", TEXT_DIM);
    }
}

pub fn clear_window_sized(w: u32, h: u32) {
    let wx = unsafe { CUR_WIN_X as u32 };
    let wy = unsafe { CUR_WIN_Y as u32 };
    draw_rect(wx.saturating_sub(2), wy.saturating_sub(2), w + 4, h + 4, DARK_BG);
    blend_rect(wx.saturating_sub(2), wy.saturating_sub(2), w + 4, h + 4, SOVEREIGN_PURPLE, 28);
}

pub fn clear_window() {
    let wx = unsafe { CUR_WIN_X as u32 };
    let wy = unsafe { CUR_WIN_Y as u32 };
    draw_rect(wx.saturating_sub(2), wy.saturating_sub(2), WIN_W + 10, WIN_H + 4, DARK_BG);
    blend_rect(wx.saturating_sub(2), wy.saturating_sub(2), WIN_W + 10, WIN_H + 4, SOVEREIGN_PURPLE, 28);
}
