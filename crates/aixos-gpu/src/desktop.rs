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
    draw_str(350, 682, "axos>", TEXT_WHITE);
}

/// Left panel — Sovereign Identity Space
/// Shows node identity, ARPi ceremony state, boot proof.
pub fn render_left_panel(proof: u32) {
    // Clear panel content area (leave border)
    draw_rect(1, 42, 198, 627, PANEL_BG);

    // Section header
    draw_str(8, 52, "IDENTITY", SOVEREIGN_PURPLE);
    draw_hline(1, 64, 198, PANEL_BORDER);

    // Node ID (stub — zero until ARPi wired)
    draw_str(8, 72, "Node", TEXT_DIM);
    draw_str(8, 84, "0x0000000000000000", TEXT_WHITE);

    draw_hline(1, 100, 198, PANEL_BORDER);

    // ARPi ceremony state
    draw_str(8, 108, "ARPi", TEXT_DIM);
    draw_str(8, 120, "pending", ACCENT_AMBER);

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

    // AWP status
    draw_str(1088, 72, "AWP", TEXT_DIM);
    draw_str(1088, 84, "stub", ACCENT_AMBER);

    draw_hline(1080, 100, 200, PANEL_BORDER);

    // EdisonDB
    draw_str(1088, 108, "EdisonDB", TEXT_DIM);
    draw_str(1088, 120, "stub", ACCENT_AMBER);

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

    // BASTION
    draw_str(1088, 252, "BASTION", TEXT_DIM);
    draw_str(1088, 264, "stub", ACCENT_AMBER);
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
    draw_rect(340, 670, 600, 50, DOCK_BG);
    draw_border(340, 670, 600, 50, PANEL_BORDER);
    draw_str(348, 682, "axos> ", TEXT_DIM);
    let n = if len < buf.len() { len } else { buf.len() };
    crate::font::draw_bytes(398, 682, &buf[..n], TEXT_WHITE);
}

pub fn render_command_result(msg: &str) {
    draw_rect(340, 670, 600, 50, DOCK_BG);
    draw_border(340, 670, 600, 50, PANEL_BORDER);
    draw_str(348, 682, msg, ACCENT_TEAL);
}

#[allow(dead_code)]
const TEXT_DIM_2: u32 = 0x666688;
