// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0

use crate::draw::{draw_rect, draw_border, draw_hline, blend_rect};
use crate::font::draw_str;

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

pub fn render_desktop() {
    draw_rect(0, 0, 1280, 720, DARK_BG);
    draw_rect(0, 0, 1280, 40, TOP_BAR);
    draw_hline(0, 40, 1280, PANEL_BORDER);
    draw_rect(0, 41, 200, 629, PANEL_BG);
    draw_border(0, 41, 200, 629, PANEL_BORDER);
    draw_rect(1080, 41, 200, 629, PANEL_BG);
    draw_border(1080, 41, 200, 629, PANEL_BORDER);
    draw_rect(201, 41, 878, 629, DARK_BG);
    blend_rect(201, 41, 878, 629, SOVEREIGN_PURPLE, 28);
    draw_rect(340, 670, 600, 48, DOCK_BG);
    draw_border(340, 670, 600, 48, PANEL_BORDER);
    draw_rect(624, 679, 32, 32, SOVEREIGN_PURPLE);
    draw_border(624, 679, 32, 32, PANEL_BORDER);
    // AIEONYX logo diamond
    let cx: u32 = 640;
    let cy: u32 = 355;
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
    draw_rect(1060, 14, 8, 8, ACCENT_TEAL);
}

pub fn render_status_bar(text: &str) {
    draw_str(16, 16, text, TEXT_WHITE);
}

pub fn render_dock() {
    // Node icon
    draw_rect(360, 674, 80, 36, ACCENT_TEAL);
    draw_str(376, 686, "Node", TEXT_WHITE);
    // Shell icon
    draw_rect(460, 674, 80, 36, ACCENT_AMBER);
    draw_str(476, 686, "Shell", TEXT_WHITE);
    // EDB icon
    draw_rect(560, 674, 80, 36, SOVEREIGN_PURPLE);
    draw_str(576, 686, "EDB", TEXT_WHITE);
    // prompt label
}

/// Left panel — Sovereign Identity Space
/// Shows node identity, ARPi ceremony state, boot proof.
pub fn render_left_panel(proof: u32, node_id: u64) {
    // Clear panel content area (leave border)
    draw_rect(1, 42, 198, 627, PANEL_BG);

    // Section header
    draw_str(8, 52, "IDENTITY", SOVEREIGN_PURPLE);
    draw_hline(1, 64, 198, PANEL_BORDER);

    // Node ID — hardware-derived from RAM base + fw_cfg constants
    draw_str(8, 72, "Node", TEXT_DIM);
    // Render node_id as two lines of 8 hex digits each
    let hi = (node_id >> 32) as u32;
    let lo = node_id as u32;
    crate::font::draw_hex32(8, 84, hi, TEXT_WHITE);
    crate::font::draw_hex32(76, 84, lo, TEXT_WHITE);

    draw_hline(1, 100, 198, PANEL_BORDER);

    // ARPi ceremony state
    draw_str(8, 108, "ARPi", TEXT_DIM);
    if proof == 0x4153 {
        draw_str(8, 120, "active", ACCENT_TEAL);
    } else {
        draw_str(8, 120, "pending", ACCENT_AMBER);
    }

    draw_hline(1, 136, 198, PANEL_BORDER);

    // Sovereign proof
    draw_str(8, 144, "Proof", TEXT_DIM);
    if proof == 0x4153 {
        draw_str(8, 156, "0x4153 [OK]", ACCENT_TEAL);
    } else {
        draw_str(8, 156, "incomplete", ACCENT_AMBER);
    }

    draw_hline(1, 172, 198, PANEL_BORDER);

    // Boot mode
    draw_str(8, 180, "Boot", TEXT_DIM);
    draw_str(8, 192, "Live", TEXT_WHITE);

    draw_hline(1, 208, 198, PANEL_BORDER);

    // Architecture
    draw_str(8, 216, "Arch", TEXT_DIM);
    draw_str(8, 228, "aarch64", TEXT_WHITE);

    draw_hline(1, 244, 198, PANEL_BORDER);

    // Kernel
    draw_str(8, 252, "Kernel", TEXT_DIM);
    draw_str(8, 264, "Phoenix v0.1", TEXT_WHITE);
}

/// Right panel — System Space
/// Shows AWP status, EdisonDB state, input driver, display.
pub fn render_right_panel() {
    draw_rect(1081, 42, 198, 627, PANEL_BG);

    draw_str(1088, 52, "SYSTEM", SOVEREIGN_PURPLE);
    draw_hline(1080, 64, 200, PANEL_BORDER);

    // AWP status — loopback confirmed
    draw_str(1088, 72, "AWP", TEXT_DIM);
    draw_str(1088, 84, "lite-live", ACCENT_TEAL);

    draw_hline(1080, 100, 200, PANEL_BORDER);

    // EdisonDB
    draw_str(1088, 108, "EdisonDB", TEXT_DIM);
    if aixos_edisondb::is_live() {
        draw_str(1088, 120, "live", ACCENT_TEAL);
    } else {
        draw_str(1088, 120, "stub", ACCENT_AMBER);
    }

    draw_hline(1080, 136, 200, PANEL_BORDER);

    // Input driver
    draw_str(1088, 144, "Input", TEXT_DIM);
    draw_str(1088, 156, "virtio+uart", ACCENT_TEAL);

    draw_hline(1080, 172, 200, PANEL_BORDER);

    // Display
    draw_str(1088, 180, "Display", TEXT_DIM);
    draw_str(1088, 192, "ramfb 1280x720", ACCENT_TEAL);

    draw_hline(1080, 208, 200, PANEL_BORDER);

    // HANIEL
    draw_str(1088, 216, "HANIEL", TEXT_DIM);
    draw_str(1088, 228, "stub", ACCENT_AMBER);

    draw_hline(1080, 244, 200, PANEL_BORDER);

    // BASTION — shell loop running proves daemon context active
    draw_str(1088, 252, "BASTION", TEXT_DIM);
    draw_str(1088, 264, "lite-live", ACCENT_TEAL);
}

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

static mut CUR_WIN_X: i32 = 340;
static mut CUR_WIN_Y: i32 = 110;

pub fn set_window_pos(x: i32, y: i32) {
    unsafe { CUR_WIN_X = x; CUR_WIN_Y = y; }
}
pub fn get_window_pos() -> (i32, i32) {
    unsafe { (CUR_WIN_X, CUR_WIN_Y) }
}

pub fn dock_icon_at(x: i32, y: i32) -> Option<u8> {
    if y < 674 || y >= 710 {
        return None;
    }
    if x >= 360 && x < 440 { return Some(0); }
    if x >= 460 && x < 540 { return Some(1); }
    if x >= 560 && x < 640 { return Some(2); }
    None
}

pub fn render_window(title: &str, lines: &[&str]) {
    let wx = unsafe { CUR_WIN_X as u32 };
    let wy = unsafe { CUR_WIN_Y as u32 };
    draw_rect(wx.saturating_sub(1), wy.saturating_sub(1), WIN_W + 2, WIN_H + 2, ACCENT_TEAL);
    draw_rect(wx, wy, WIN_W, WIN_TITLE_H, WIN_TITLE);
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
    draw_str(wx + WIN_W - 62, wy + 8, "[close]", 0x666688);
    draw_hline(wx, wy + WIN_TITLE_H, WIN_W, ACCENT_TEAL);
    draw_rect(wx, wy + WIN_TITLE_H + 1, WIN_W, WIN_H - WIN_TITLE_H - 1, WIN_BG);
    let mut row = 0u32;
    for line in lines.iter().take(10) {
        draw_str(wx + 12, wy + WIN_TITLE_H + 12 + row * 18, line, TEXT_WHITE);
        row += 1;
    }
}

pub fn render_window_output(wx: i32, wy: i32, lines: &[&'static str], count: usize) {
    draw_rect((wx + 4) as u32, (wy + 25) as u32, 572, 254, WIN_BG);
    let n = if count > 8 { 8 } else { count };
    let mut y = wy + 36;
    let mut idx = 0;
    while idx < n {
        draw_str((wx + 8) as u32, y as u32, lines[idx], TEXT_WHITE);
        y += 18;
        idx += 1;
    }
}

pub fn render_window_input(wx: i32, wy: i32, buf: &[u8], len: usize, focused: bool) {
    let y = wy + 280;
    draw_rect((wx + 4) as u32, (y - 2) as u32, 572, 18, WIN_BG);
    draw_str((wx + 8) as u32, y as u32, "win> ", ACCENT_TEAL);
    if let Ok(txt) = core::str::from_utf8(&buf[..len]) {
        draw_str((wx + 48) as u32, y as u32, txt, TEXT_WHITE);
    }
    draw_str((wx + 48 + (len as i32) * 8) as u32, y as u32, "_", TEXT_WHITE);
    if focused {
        draw_str((wx + 500) as u32, y as u32, "[focused]", TEXT_DIM);
    }
}

pub fn clear_window() {
    let wx = unsafe { CUR_WIN_X as u32 };
    let wy = unsafe { CUR_WIN_Y as u32 };
    draw_rect(wx.saturating_sub(2), wy.saturating_sub(2), WIN_W + 10, WIN_H + 4, DARK_BG);
    blend_rect(wx.saturating_sub(2), wy.saturating_sub(2), WIN_W + 10, WIN_H + 4, SOVEREIGN_PURPLE, 28);
}
