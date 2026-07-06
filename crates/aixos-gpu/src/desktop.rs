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
