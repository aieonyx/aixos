// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0

use crate::draw::{draw_rect, draw_border, draw_hline, blend_rect};
use crate::font::draw_str;

const DARK_BG:       u32 = 0x0A0A1A;
const PANEL_BG:      u32 = 0x141428;
const PANEL_BORDER:  u32 = 0x2A2A4A;
const TEXT_WHITE:    u32 = 0xEEEEFF;
const TOP_BAR:       u32 = 0x080818;
const DOCK_BG:       u32 = 0x0D0D20;
const SOVEREIGN_PURPLE: u32 = 0x7B4FDB;
const ACCENT_TEAL:   u32 = 0x1BAF7A;

pub fn render_desktop() {
    // Background
    draw_rect(0, 0, 1280, 720, DARK_BG);
    // Top bar
    draw_rect(0, 0, 1280, 40, TOP_BAR);
    draw_hline(0, 40, 1280, PANEL_BORDER);
    // Left panel
    draw_rect(0, 41, 200, 629, PANEL_BG);
    draw_border(0, 41, 200, 629, PANEL_BORDER);
    // Right panel
    draw_rect(1080, 41, 200, 629, PANEL_BG);
    draw_border(1080, 41, 200, 629, PANEL_BORDER);
    // Center canvas — purple tint over dark
    draw_rect(201, 41, 878, 629, DARK_BG);
    blend_rect(201, 41, 878, 629, SOVEREIGN_PURPLE, 28);
    // Dock
    draw_rect(340, 670, 600, 48, DOCK_BG);
    draw_border(340, 670, 600, 48, PANEL_BORDER);
    // Home button — 32x32 sovereign purple square centered in dock
    draw_rect(624, 679, 32, 32, SOVEREIGN_PURPLE);
    draw_border(624, 679, 32, 32, PANEL_BORDER);
    // AIEONYX logo diamond — center canvas
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
    // Status dot
    draw_rect(1060, 14, 8, 8, ACCENT_TEAL);
}

pub fn render_status_bar(text: &str) {
    draw_str(16, 16, text, TEXT_WHITE);
}

pub fn render_dock() {
    draw_str(350, 682, "axos>", TEXT_WHITE);
}

use crate::draw::{draw_rect as fill, draw_border as db};
use crate::font::draw_bytes;

const TEXT_DIM_2: u32 = 0x666688;

pub fn render_input_line(buf: &[u8], len: usize) {
    draw_rect(340, 670, 600, 50, DOCK_BG);
    draw_border(340, 670, 600, 50, PANEL_BORDER);
    draw_str(348, 682, "axos> ", 0x666688);
    let n = if len < buf.len() { len } else { buf.len() };
    crate::font::draw_bytes(398, 682, &buf[..n], TEXT_WHITE);
}

pub fn render_command_result(msg: &str) {
    draw_rect(340, 670, 600, 50, DOCK_BG);
    draw_border(340, 670, 600, 50, PANEL_BORDER);
    draw_str(348, 682, msg, ACCENT_TEAL);
}

// ── PL-20: Sovereign Window Primitive ────────────────────────────────────────

const WIN_X: u32 = 340;
const WIN_Y: u32 = 110;
const WIN_W: u32 = 580;
const WIN_H: u32 = 300;
const WIN_TITLE_H: u32 = 24;
const WIN_BG:    u32 = 0x0D0D22;
const WIN_TITLE: u32 = 0x1A1A3A;

/// Render a sovereign floating window on the center canvas.
/// title: window title text
/// lines: content lines (up to 10, 18px apart)
pub fn render_window(title: &str, lines: &[&str]) {
    // Outer teal border
    draw_rect(WIN_X.saturating_sub(1), WIN_Y.saturating_sub(1),
              WIN_W + 2, WIN_H + 2, ACCENT_TEAL);
    // Title bar
    draw_rect(WIN_X, WIN_Y, WIN_W, WIN_TITLE_H, WIN_TITLE);
    // Mini diamond icon in title bar
    let tx = WIN_X + 8;
    let ty = WIN_Y + 12;
    draw_hline(tx,                    ty.saturating_sub(4), 1, SOVEREIGN_PURPLE);
    draw_hline(tx.saturating_sub(1),  ty.saturating_sub(3), 3, SOVEREIGN_PURPLE);
    draw_hline(tx.saturating_sub(2),  ty.saturating_sub(2), 5, SOVEREIGN_PURPLE);
    draw_hline(tx.saturating_sub(3),  ty.saturating_sub(1), 7, SOVEREIGN_PURPLE);
    draw_hline(tx.saturating_sub(4),  ty,                   9, SOVEREIGN_PURPLE);
    draw_hline(tx.saturating_sub(3),  ty + 1,               7, SOVEREIGN_PURPLE);
    draw_hline(tx.saturating_sub(2),  ty + 2,               5, SOVEREIGN_PURPLE);
    draw_hline(tx.saturating_sub(1),  ty + 3,               3, SOVEREIGN_PURPLE);
    draw_hline(tx,                    ty + 4,               1, SOVEREIGN_PURPLE);
    // Title text and close hint
    draw_str(WIN_X + 22, WIN_Y + 8, title, TEXT_WHITE);
    draw_str(WIN_X + WIN_W - 52, WIN_Y + 8, "[close]", 0x666688);
    // Title bar bottom separator
    draw_hline(WIN_X, WIN_Y + WIN_TITLE_H, WIN_W, ACCENT_TEAL);
    // Content area background
    draw_rect(WIN_X, WIN_Y + WIN_TITLE_H + 1,
              WIN_W, WIN_H - WIN_TITLE_H - 1, WIN_BG);
    // Content lines — 18px row height, 12px left margin
    let mut row = 0u32;
    for line in lines.iter().take(10) {
        draw_str(WIN_X + 12,
                 WIN_Y + WIN_TITLE_H + 12 + row * 18,
                 line, TEXT_WHITE);
        row += 1;
    }
}

/// Clear the window — restore sovereign canvas background.
pub fn clear_window() {
    draw_rect(WIN_X.saturating_sub(1), WIN_Y.saturating_sub(1),
              WIN_W + 2, WIN_H + 2, DARK_BG);
    blend_rect(WIN_X.saturating_sub(1), WIN_Y.saturating_sub(1),
               WIN_W + 2, WIN_H + 2, SOVEREIGN_PURPLE, 28);
}
