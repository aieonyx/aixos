// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0

use crate::draw::{draw_rect, draw_border, draw_hline, blend_rect, draw_rounded_rect, draw_rounded_border};
// cache_flush imported when needed
use crate::font::{draw_str, draw_str_2x, draw_str_clipped, draw_hex32, draw_str_15x, draw_str_15x_clipped};

const DARK_BG:          u32 = 0x0D0B1F;
#[allow(dead_code)]
const DARK_BG2:         u32 = 0x1A0E2E;
const PANEL_BG:         u32 = 0x141428;
const PANEL_BORDER:     u32 = 0x2A2A4A;
const TEXT_WHITE:       u32 = 0xEEEEFF;
#[allow(dead_code)]
const TEXT_DIM:         u32 = 0x666688;
#[allow(dead_code)]
const TOP_BAR:          u32 = 0x080818;
const DOCK_BG:          u32 = 0x0D0D20;
#[allow(dead_code)]
const SOVEREIGN_PURPLE: u32 = 0x7B4FDB;
#[allow(dead_code)]
const ACCENT_TEAL:      u32 = 0x1BAF7A;
const ACCENT_AMBER:     u32 = 0xD4A017;
const SETTINGS_BLUE:    u32 = 0x1B7FC4;
const BROWSE_GREEN:     u32 = 0x2A7A4A;
const TOP_BAR_H:  u32 = 38;
#[allow(dead_code)]
const DOCK_Y:     u32 = 676;
const DOCK_H:     u32 = 44;
const PANEL_W:    u32 = 180;
#[allow(dead_code)]
const TASKBAR_Y:  u32 = 676;
#[allow(dead_code)]
const TASKBAR_H:  u32 = 44;
#[allow(dead_code)]
const CANVAS_Y:   u32 = 38;
#[allow(dead_code)]
const CANVAS_H:   u32 = 638;
const GLASS_PANEL: u32 = 0x0F0D22;
const GLASS_BORDER: u32 = 0x2A2840;

pub struct DesktopState {
    pub node_id:     u64,
    pub proof:       u64,
    pub edb_live:    bool,
    pub entry_count: usize,
    pub desktop_ok:  bool,
    pub uptime_sec:  u64,
    pub rtc_hour:    u8,
    pub rtc_min:     u8,
    pub rtc_day:     u8,
    pub rtc_mon:     u8,
    pub active_space: u8,
    // PL-49: user identity fields
    pub tz_offset:   i32,
    pub user_name:   &'static [u8],
}
impl DesktopState {
    pub const fn default() -> Self {
        DesktopState {
            node_id: 0, proof: 0x4153, edb_live: false,
            entry_count: 0, desktop_ok: false, uptime_sec: 0, active_space: 0,
            rtc_hour: 0, rtc_min: 0, rtc_day: 0, rtc_mon: 0,
            tz_offset: 0, user_name: b"",
        }
    }
}


// ── PL-33: Boot Splash Screen ────────────────────────────────────────────────

pub fn render_splash() {
    draw_rect(0, 0, 1280, 720, DARK_BG);
    let cx: u32 = 640;
    let cy: u32 = 280;
    let mut i: u32 = 0;
    while i <= 48 {
        let w = i * 2 + 1;
        let x = cx.saturating_sub(i);
        let y = cy.saturating_sub(48).saturating_add(i);
        draw_hline(x, y, w, SOVEREIGN_PURPLE);
        i += 1;
    }
    let mut i: u32 = 1;
    while i <= 48 {
        let w = (48 - i) * 2 + 1;
        let x = cx.saturating_sub(48 - i);
        let y = cy + i;
        draw_hline(x, y, w, SOVEREIGN_PURPLE);
        i += 1;
    }
    draw_str_2x(584, 370, "AIEONYX", ACCENT_TEAL);
    draw_str(512, 408, "Sovereign Digital Infrastructure", TEXT_DIM);
    draw_str(516, 440, "aiXos Phoenix  v0.1.0  aarch64", TEXT_WHITE);
    draw_str(504, 460, "axon_main() -> 0x4153  [SOVEREIGN]", ACCENT_TEAL);
    draw_rect(390, 500, 500, 12, PANEL_BG);
    draw_rect(390, 500, 500, 12, PANEL_BORDER);
    draw_rect(392, 502, 496, 8, ACCENT_TEAL);
}

pub fn render_desktop(state: &DesktopState) {
    let mut by: u32 = 0;
    while by < 720 {
        let t = (by * 255 / 720) as u8;
        let r = 0x0Du8.saturating_add((((0x1Au32.saturating_sub(0x0D)) * t as u32) / 255) as u8);
        let g = 0x0Bu8.saturating_add((((0x0Eu32.saturating_sub(0x0B)) * t as u32) / 255) as u8);
        let b = 0x1Fu8.saturating_add((((0x2Eu32.saturating_sub(0x1F)) * t as u32) / 255) as u8);
        let color = ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
        draw_hline(0, by, 1280, color);
        by += 1;
    }
    let stars: [(u32, u32); 12] = [
        (120,80),(340,40),(580,90),(700,50),(50,200),(750,300),
        (200,420),(650,440),(900,150),(1100,80),(450,350),(1050,500),
    ];
    for (sx, sy) in stars.iter() {
        draw_rect(*sx, *sy, 2, 2, 0xCCCCDD);
    }
    // ── Center canvas sovereign anchor (PL-47) ──────────────────────────────
    // Small diamond at canvas center (between left panel x=188 and right panel x=1092)
    // Center: x=640, y=390 — subtle, dim, sovereign presence
    let cx: u32 = 640;
    let cy: u32 = 390;
    let r: u32 = 20; // radius
    let mut di: u32 = 0;
    while di <= r {
        let dw = di * 2 + 1;
        let dx = cx.saturating_sub(di);
        let dy = cy.saturating_sub(r).saturating_add(di);
        draw_hline(dx, dy, dw, 0x2A1A4A);
        di += 1;
    }
    let mut di: u32 = 1;
    while di <= r {
        let dw = (r - di) * 2 + 1;
        let dx = cx.saturating_sub(r - di);
        let dy = cy + di;
        draw_hline(dx, dy, dw, 0x2A1A4A);
        di += 1;
    }
    // AIEONYX wordmark below diamond — very dim, ambient
    draw_str(612, cy + r + 8, "AIEONYX", 0x1A1A2A);

    // Left glass panel
    draw_rounded_rect(8, TOP_BAR_H + 8, PANEL_W, 720 - TOP_BAR_H - DOCK_H - 16, 8, GLASS_PANEL);
    draw_rounded_border(8, TOP_BAR_H + 8, PANEL_W, 720 - TOP_BAR_H - DOCK_H - 16, 8, GLASS_BORDER);
    draw_hline(9, TOP_BAR_H + 9, PANEL_W - 2, 0x3A3860);
    draw_str(24, TOP_BAR_H + 28, "IDENTITY", 0x44446A);
    draw_rect(20, TOP_BAR_H + 42, 32, 32, SOVEREIGN_PURPLE);
    blend_rect(20, TOP_BAR_H + 42, 32, 32, 0xFFFFFF, 20);
    // PL-49: show user name if set, else fallback to "E" + hex node_id
    if !state.user_name.is_empty() {
        if let Ok(s) = core::str::from_utf8(state.user_name) {
            draw_str(30, TOP_BAR_H + 63, s, TEXT_WHITE);
        }
        draw_str(30, TOP_BAR_H + 73, "Sovereign", 0x44446A);
    } else {
        draw_str(30, TOP_BAR_H + 63, "E", TEXT_WHITE);
        draw_hex32(60, TOP_BAR_H + 55, state.node_id as u32, TEXT_WHITE);
        draw_str(60, TOP_BAR_H + 68, "Sovereign", 0x44446A);
    }
    draw_hline(16, TOP_BAR_H + 90, PANEL_W - 16, GLASS_BORDER);
    draw_str(24, TOP_BAR_H + 108, "SPACES", 0x44446A);
    let space_labels: [&str; 4] = ["Desktop", "Files", "Onyxia", "EdisonDB"];
    let space_y: [u32; 4] = [
        TOP_BAR_H + 116, TOP_BAR_H + 142,
        TOP_BAR_H + 162, TOP_BAR_H + 182,
    ];
    let mut si = 0u32;
    while si < 4 {
        let sy = space_y[si as usize];
        let is_active = si == state.active_space as u32;
        if is_active {
            draw_rect(16, sy, PANEL_W - 16, 22, SOVEREIGN_PURPLE);
            blend_rect(16, sy, PANEL_W - 16, 22, 0x000000, 180);
            draw_rect(24, sy + 5, 3, 12, SOVEREIGN_PURPLE);
            draw_str(34, sy + 13, space_labels[si as usize], TEXT_WHITE);
        } else {
            draw_rect(24, sy + 6, 3, 10, 0x33334A);
            draw_str(34, sy + 13, space_labels[si as usize], 0x55556A);
        }
        si += 1;
    }
    draw_hline(16, TOP_BAR_H + 218, PANEL_W - 16, GLASS_BORDER);
    draw_str(24, TOP_BAR_H + 234, "BASTION STATUS", 0x44446A);
    let pol_col = if state.edb_live { ACCENT_TEAL } else { 0x444444 };
    draw_rect(24, TOP_BAR_H + 256, 8, 8, pol_col);
    draw_str(36, TOP_BAR_H + 256, "Policy active", 0x888899);
    let desk_col = if state.desktop_ok { ACCENT_TEAL } else { 0x444444 };
    draw_rect(24, TOP_BAR_H + 272, 8, 8, desk_col);
    draw_str(36, TOP_BAR_H + 272, "Desktop ready", 0x888899);
    let proof_col = if state.proof == 0x4153 { SOVEREIGN_PURPLE } else { 0x444444 };
    draw_rect(24, TOP_BAR_H + 288, 8, 8, proof_col);
    draw_str(36, TOP_BAR_H + 288, "Proof 0x4153", 0x888899);
    // Right glass panel
    let rx: u32 = 1280 - PANEL_W - 8;
    draw_rounded_rect(rx, TOP_BAR_H + 8, PANEL_W, 720 - TOP_BAR_H - DOCK_H - 16, 8, GLASS_PANEL);
    draw_rounded_border(rx, TOP_BAR_H + 8, PANEL_W, 720 - TOP_BAR_H - DOCK_H - 16, 8, GLASS_BORDER);
    draw_hline(rx + 1, TOP_BAR_H + 9, PANEL_W - 2, 0x3A3860);
    draw_str(rx + 16, TOP_BAR_H + 28, "SYSTEM", 0x44446A);
    let icon_labels: [&str; 6] = ["O","F","S","A","D","N"];
    let icon_colors: [u32; 6] = [SOVEREIGN_PURPLE,0x1850A0,SETTINGS_BLUE,0x331A4A,BROWSE_GREEN,ACCENT_TEAL]; // A dimmed=stub, N=teal(Network wired)
    let mut ii = 0u32;
    while ii < 6 {
        let col = ii % 3;
        let row = ii / 3;
        let ix = rx + 16 + col * 44;
        let iy = TOP_BAR_H + 42 + row * 44;
        draw_rounded_rect(ix, iy, 36, 36, 6, icon_colors[ii as usize]);
        blend_rect(ix, iy, 36, 36, 0x000000, 160);
        blend_rect(ix, iy, 36, 18, 0xFFFFFF, 15);
        draw_rounded_border(ix, iy, 36, 36, 6, 0x44446A);
        draw_str(ix + 12, iy + 22, icon_labels[ii as usize], TEXT_WHITE);
        ii += 1;
    }
    draw_hline(rx + 8, TOP_BAR_H + 138, PANEL_W - 16, GLASS_BORDER);
    draw_str(rx + 16, TOP_BAR_H + 156, "RESOURCES", 0x44446A);
    let edb_pct = if state.entry_count > 0 { (state.entry_count * 100 / 32) as u32 } else { 0 };
    let bar_w: u32 = PANEL_W - 32;
    let bar_x: u32 = rx + 16;
    // EDB label then bar below
    draw_str(rx + 16, TOP_BAR_H + 172, "EDB fill", 0x888899);
    draw_rect(bar_x, TOP_BAR_H + 182, bar_w, 5, 0x22224A);
    draw_rect(bar_x, TOP_BAR_H + 182, bar_w * edb_pct / 100, 5, SOVEREIGN_PURPLE);
    // SIG label then bar below
    let proof_ok = state.proof == 0x4153;
    draw_str(rx + 16, TOP_BAR_H + 196, "SIG verify", 0x888899);
    draw_rect(bar_x, TOP_BAR_H + 206, bar_w, 5, 0x22224A);
    draw_rect(bar_x, TOP_BAR_H + 206, if proof_ok { bar_w } else { 0 }, 5, ACCENT_TEAL);
    // OK status shown by full bar color — no separate label needed
    draw_hline(rx + 8, TOP_BAR_H + 220, PANEL_W - 16, GLASS_BORDER);
    draw_str(rx + 16, TOP_BAR_H + 238, "NETWORK", 0x44446A);
    let awp_col = if state.edb_live { ACCENT_TEAL } else { 0x444444 };
    draw_rect(rx + 16, TOP_BAR_H + 252, 8, 8, awp_col);
    draw_str(rx + 30, TOP_BAR_H + 260, "AWP  loopback", 0x888899);
    draw_str(rx + 16, TOP_BAR_H + 276, "EDB:", 0x33334A);
    draw_hex32(rx + 48, TOP_BAR_H + 276, state.entry_count as u32, 0x44446A);


}


pub fn render_top_bar_icons(uptime_sec: u64, rtc_hour: u8, rtc_min: u8, rtc_day: u8, rtc_mon: u8, tz_offset: i32) {
    draw_rect(0, 0, 1280, TOP_BAR_H, 0x08060F);
    draw_hline(0, 0, 1280, 0x2A2848);
    draw_hline(0, TOP_BAR_H - 1, 1280, 0x1A1830);
    draw_rect(12, 13, 14, 2, TEXT_WHITE);
    draw_rect(12, 18, 10, 2, TEXT_WHITE);
    draw_rect(12, 23, 12, 2, TEXT_WHITE);
    // aiXos Phoenix centered — left cleared
    // Centered sovereign wordmark — IAM search deferred to future phase
    draw_str_15x(494, 10, "aiXos Phoenix", 0x444466);
    // Clock drawn in render_desktop() where state is in scope
    // Real date+time from PL031 RTC
    let digs = b"0123456789";
    let _ = uptime_sec;
    let months = ["   ","Jan","Feb","Mar","Apr","May","Jun",
                  "Jul","Aug","Sep","Oct","Nov","Dec"];
    let mon_str = if (rtc_mon as usize) < 13 { months[rtc_mon as usize] } else { "???" };
    draw_str(1080, 15, mon_str, 0x888899);
    crate::font::draw_char(1112, 15, digs[((rtc_day / 10) % 10) as usize] as char, 0x888899);
    crate::font::draw_char(1120, 15, digs[(rtc_day % 10) as usize] as char, 0x888899);
    draw_str(1130, 15, " ", 0x666688);
    // PL-49: apply timezone offset to clock display
    let local_hour = ((rtc_hour as i32 + tz_offset).rem_euclid(24)) as u8;
    crate::font::draw_char(1138, 15, digs[((local_hour / 10) % 10) as usize] as char, 0x888899);
    crate::font::draw_char(1146, 15, digs[(local_hour % 10) as usize] as char, 0x888899);
    draw_str(1154, 15, ":", 0x666688);
    crate::font::draw_char(1162, 15, digs[((rtc_min / 10) % 10) as usize] as char, 0x888899);
    crate::font::draw_char(1170, 15, digs[(rtc_min % 10) as usize] as char, 0x888899);
    // Show UTC+N or UTC-N label
    if tz_offset == 0 {
        draw_str(1190, 15, "UTC", 0x444466);
    } else if tz_offset > 0 {
        draw_str(1190, 15, "UTC+", 0x4466AA);
        let tz_abs = tz_offset as u8;
        crate::font::draw_char(1222, 15, digs[(tz_abs % 10) as usize] as char, 0x4466AA);
    } else {
        draw_str(1190, 15, "UTC-", 0x6644AA);
        let tz_abs = (-tz_offset) as u8;
        crate::font::draw_char(1222, 15, digs[(tz_abs % 10) as usize] as char, 0x6644AA);
    }
    draw_rect(1230, 15, 6, 6, ACCENT_TEAL);
}


pub fn render_taskbar(slots: &[(bool, u8)], active: usize) {
    draw_rect(0, DOCK_Y, 1280, DOCK_H, 0x0A0818);
    draw_hline(0, DOCK_Y, 1280, 0x1A1830);
    // 7 icons x 34px + 6px gap = 280px icons
    // + 10px left pad + 10px right pad + separator + axos> = ~420px total
    let dock_w: u32 = 420;
    let dock_x: u32 = (1280 - dock_w) / 2;
    let dock_py: u32 = DOCK_Y + 4;
    draw_rounded_rect(dock_x, dock_py, dock_w, 36, 10, 0x100E20);
    draw_rounded_border(dock_x, dock_py, dock_w, 36, 10, 0x2A2848);
    draw_hline(dock_x + 10, dock_py + 1, dock_w - 20, 0x3A3858);
    // 7 app icons, 34x26 each, 6px gap, start at dock_x+10
    let labels: [&str; 7] = ["O", "W", ">_", "F", "D", "I", "S"];
    let colors: [u32; 7] = [
        SOVEREIGN_PURPLE, 0x1850A0, ACCENT_AMBER,
        0x2A6A3A, BROWSE_GREEN, 0x8B4FDB, SETTINGS_BLUE,
    ];
    let icon_w: u32 = 30;
    let icon_gap: u32 = 6;
    let mut di = 0u32;
    while di < 7 {
        let ix = dock_x + 10 + di * (icon_w + icon_gap);
        let iy = dock_py + 5;
        draw_rounded_rect(ix, iy, icon_w, 26, 4, colors[di as usize]);
        blend_rect(ix, iy, icon_w, 26, 0x000000, 120);
        blend_rect(ix, iy, icon_w, 13, 0xFFFFFF, 20);
        draw_rounded_border(ix, iy, icon_w, 26, 4, 0x33334A);
        draw_str(ix + 9, iy + 17, labels[di as usize], TEXT_WHITE);
        di += 1;
    }
    // Separator
    let sep_x = dock_x + 10 + 7 * (icon_w + icon_gap) + 4;
    draw_rect(sep_x, dock_py + 8, 1, 20, 0x2A2848);
    // axos> prompt — right of separator, vertically centered
    draw_str(sep_x + 8, dock_py + 22, "axos>", 0x555570);
    draw_rect(sep_x + 52, dock_py + 13, 5, 12, SOVEREIGN_PURPLE);
    // Open window indicators — teal dot above icon
    let mut wi = 0usize;
    while wi < slots.len() {
        if slots[wi].0 {
            let kind = slots[wi].1 as u32;
            // Map window kind to dock icon index
            let dock_idx: u32 = match kind {
                1 => 2, // Shell -> >_
                2 => 4, // EDB  -> D
                3 => 6, // Set  -> S
                4 => 4, // EDB browser -> D
                _ => 0,
            };
            let dot_x = dock_x + 10 + dock_idx * (icon_w + icon_gap) + icon_w / 2 - 3;
            draw_rect(dot_x, dock_py + 2, 6, 2, ACCENT_TEAL);
        }
        wi += 1;
    }
    let _ = active;
}


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
#[allow(dead_code)]
const WIN_X: u32 = 340;
#[allow(dead_code)]
const WIN_Y: u32 = 110;
const WIN_W: u32 = 580;
const WIN_H: u32 = 300;
const WIN_TITLE_H: u32 = 24;
const WIN_BG:    u32 = 0x0D0D22;
#[allow(dead_code)]
const WIN_TITLE: u32 = 0x1A1A3A;
#[allow(dead_code)]
const GLASS_HI:  u32 = 0x3A3A5A;
#[allow(dead_code)]
const GLASS_MID: u32 = 0x1E1E38;
#[allow(dead_code)]
const GLASS_LOW: u32 = 0x111128;
#[allow(dead_code)]
const SHADOW:    u32 = 0x000008;
#[allow(dead_code)]
const CLOSE_RED: u32 = 0xC0392B;

static mut CUR_WIN_X: i32 = 200;
static mut CUR_WIN_Y: i32 = 80;

pub fn set_window_pos(x: i32, y: i32) {
    unsafe { CUR_WIN_X = x; CUR_WIN_Y = y; }
}
pub fn get_window_pos() -> (i32, i32) {
    unsafe { (CUR_WIN_X, CUR_WIN_Y) }
}

pub fn dock_icon_at(x: i32, y: i32) -> Option<u8> {
    let dy = DOCK_Y as i32;
    if y < dy || y > dy + 44 { return None; }
    let dock_x: i32 = (1280 - 420) / 2;
    let icon_w: i32 = 30;
    let icon_gap: i32 = 6;
    let mut i = 0u8;
    while i < 7 {
        let ix = dock_x + 10 + (i as i32) * (icon_w + icon_gap);
        if x >= ix && x < ix + icon_w { return Some(i); }
        i += 1;
    }
    None
}

pub fn render_window(title: &str, lines: &[&str], w: u32, h: u32) {
    let wx = unsafe { CUR_WIN_X as u32 };
    let wy = unsafe { CUR_WIN_Y as u32 };
    blend_rect(wx + 3, wy + 3, w + 2, h + 2, SHADOW, 100);
    draw_rounded_border(wx, wy, w, h, 5, 0x2A1A4A);
    let band = WIN_TITLE_H / 4;
    draw_rect(wx, wy,            w, band,                   GLASS_HI);
    draw_rect(wx, wy + band,     w, band,                   GLASS_MID);
    draw_rect(wx, wy + band * 2, w, band,                   GLASS_LOW);
    draw_rect(wx, wy + band * 3, w, WIN_TITLE_H - band * 3, WIN_TITLE);
    draw_hline(wx + 2, wy, w - 4, 0x6060A0);
    blend_rect(wx, wy, w, WIN_TITLE_H, 0xFFFFFF, 8);
    let tx = wx + 10;
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
    draw_str_15x_clipped(wx + 24, wy + 4, title, TEXT_WHITE, wx + w - 24);
    let cx = wx + w - 18;
    let cy = wy + 7;
    draw_rect(cx, cy, 10, 10, CLOSE_RED);
    blend_rect(cx, cy, 10, 5, 0xFFFFFF, 40);
    draw_border(cx, cy, 10, 10, 0x8B1A1A);
    draw_str(cx + 2, cy + 1, "x", TEXT_WHITE);
    draw_hline(wx, wy + WIN_TITLE_H, w, ACCENT_TEAL);
    draw_rect(wx, wy + WIN_TITLE_H + 1, w, h - WIN_TITLE_H - 1, WIN_BG);

    blend_rect(wx, wy + WIN_TITLE_H + 1, w, h - WIN_TITLE_H - 1, SOVEREIGN_PURPLE, 12);
    let max_rows = if h > WIN_TITLE_H + 20 { (h - WIN_TITLE_H - 20) / 18 } else { 0 };
    for (row, line) in lines.iter().take(max_rows as usize).enumerate() {
        draw_str_clipped(wx + 12, wy + WIN_TITLE_H + 12 + row as u32 * 18, line, TEXT_WHITE, wx + w - 8);
    }
    draw_rect(wx + w - 12, wy + h - 12, 12, 12, ACCENT_TEAL);
    blend_rect(wx + w - 12, wy + h - 12, 12, 6, 0xFFFFFF, 30);
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
    draw_str((wx + 48 + (len as i32) * 9) as u32, y as u32, "_", TEXT_WHITE);
    if focused {
        // Draw [focused] inside window right edge
        let fx = (wx as u32 + ww).saturating_sub(80);
        draw_str(fx, y as u32, "[focused]", TEXT_DIM);
    }
}


// ── PL-52: AXFS Files Window ─────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
pub fn render_files_window(
    wx: i32, wy: i32, w: u32, h: u32,
    file_names: &[(*const u8, usize)],
    file_count: usize,
    cursor: usize,
    content: &[u8],
    content_len: usize,
    viewing: bool,
) {
    let wx_u = wx as u32;
    let wy_u = wy as u32;
    // Content area starts below title bar (render_window already drew chrome)
    let start_y = wy_u + WIN_TITLE_H + 2;
    let row_h: u32 = 16;
    let max_rows = (h.saturating_sub(WIN_TITLE_H + 20)) / row_h;

    if viewing {
        // Draw "< back" hint in title area
        draw_str(wx_u + w - 100, wy_u + 14, "Esc=back", 0x44446A);
        // Render content line by line
        let mut line_start = 0usize;
        let mut row = 0u32;
        let mut i = 0;
        while i <= content_len && row < max_rows {
            if i == content_len || content[i] == b'\n' {
                let line = &content[line_start..i];
                if let Ok(s) = core::str::from_utf8(line) {
                    draw_str_clipped(wx_u + 8, start_y + row * row_h + 4, s, TEXT_WHITE, wx_u + w - 8);
                }
                row += 1;
                line_start = i + 1;
            }
            i += 1;
        }
        draw_str(wx_u + 8, wy_u + h - 12, "Esc: back to list", 0x44446A);
    } else {
        // Hint in title area
        // hints shown at bottom only
        // Render file list
        let mut fi = 0usize;
        while fi < file_count && (fi as u32) < max_rows {
            let (ptr, len) = file_names[fi];
            let row_y = start_y + fi as u32 * row_h;
            let is_sel = fi == cursor;
            if is_sel {
                draw_rect(wx_u + 2, row_y, w - 4, row_h, SOVEREIGN_PURPLE);
            }
            let col = if is_sel { TEXT_WHITE } else { 0x888899 };
            draw_str(wx_u + 8, row_y + 4, if is_sel { ">" } else { " " }, ACCENT_TEAL);
            let name_bytes = unsafe { core::slice::from_raw_parts(ptr, len) };
            if let Ok(s) = core::str::from_utf8(name_bytes) {
                draw_str_clipped(wx_u + 20, row_y + 4, s, col, wx_u + w - 8);
            }
            fi += 1;
        }
        if file_count == 0 {
            draw_str(wx_u + 8, start_y + 4, "[empty filesystem]", 0x44446A);
        }
        draw_str(wx_u + 8, wy_u + h - 12, "arrows: navigate  Enter: open  Esc: close", 0x44446A);
    }
}

// ── PL-32: EDB Browser Window ────────────────────────────────────────────────

pub struct EdbEntry {
    pub key:   &'static str,
    pub tier:  &'static str,
    pub value: u64,
}

#[allow(clippy::too_many_arguments)]
pub fn render_edb_browser(
    wx: i32, wy: i32, w: u32, h: u32,
    entries: &[EdbEntry],
    cursor: usize, scroll: usize,
    input_buf: &[u8], input_len: usize,
    focused: bool,
) {
    let wx_u = wx as u32;
    let count = entries.len();
    let hdr_y = (wy + WIN_TITLE_H as i32 + 4) as u32;
    draw_rect(wx_u + 1, hdr_y, w - 2, 16, WIN_BG);
    draw_str(wx_u + 8, hdr_y + 2, "Entries:", TEXT_DIM);
    draw_hex32(wx_u + 72, hdr_y + 2, count as u32, ACCENT_TEAL);
    draw_str(wx_u + 120, hdr_y + 2, "/ 32", TEXT_DIM);
    let sep_y = hdr_y + 17;
    draw_hline(wx_u + 4, sep_y, w - 8, PANEL_BORDER);
    let input_row_y = (wy + h as i32 - 22) as u32;
    let body_top = sep_y + 3;
    let row_h: u32 = 16;
    let max_visible = if input_row_y > body_top {
        ((input_row_y - body_top) / row_h) as usize
    } else { 0 };
    if input_row_y > body_top {
        draw_rect(wx_u + 1, body_top, w - 2, input_row_y - body_top, WIN_BG);
    }
    let mut row = 0usize;
    while row < max_visible {
        let ei = scroll + row;
        if ei >= count { break; }
        let ry = body_top + row as u32 * row_h;
        let is_cur = ei == cursor;
        if is_cur {
            draw_rect(wx_u + 2, ry, w - 4, row_h - 1, 0x0D2A20);
            draw_str(wx_u + 4, ry + 4, ">", ACCENT_TEAL);
        } else {
            draw_str(wx_u + 4, ry + 4, " ", TEXT_DIM);
        }
        let tier_col = if is_cur { ACCENT_TEAL } else { TEXT_DIM };
        draw_str(wx_u + 14, ry + 4, entries[ei].tier, tier_col);
        draw_str_clipped(wx_u + 28, ry + 4, entries[ei].key, TEXT_WHITE, wx_u + w - 100);
        draw_hex32(wx_u + w - 96, ry + 4, entries[ei].value as u32, ACCENT_AMBER);
        row += 1;
    }
    draw_hline(wx_u + 4, input_row_y - 3, w - 8, PANEL_BORDER);
    draw_rect(wx_u + 4, input_row_y - 1, w - 8, 18, WIN_BG);
    draw_str(wx_u + 8, input_row_y + 2, "edb>", ACCENT_TEAL);
    let buf_x = wx_u + 40;
    if let Ok(txt) = core::str::from_utf8(&input_buf[..input_len]) {
        draw_str(buf_x, input_row_y + 2, txt, TEXT_WHITE);
    }
    draw_str(buf_x + (input_len as u32) * 9, input_row_y + 2, "_", TEXT_WHITE);
    if focused { draw_str(wx_u + w - 80, input_row_y + 2, "[focused]", TEXT_DIM); }
    draw_rect(wx_u + w - 12, (wy + h as i32 - 12) as u32, 12, 12, ACCENT_TEAL);
    draw_rect(wx_u + w - 8,  (wy + h as i32 - 8) as u32,  4,  4,  TEXT_WHITE);
}

pub fn clear_window_sized(w: u32, h: u32) {
    let wx = unsafe { CUR_WIN_X as u32 };
    let wy = unsafe { CUR_WIN_Y as u32 };
    draw_rect(wx.saturating_sub(2), wy.saturating_sub(2), w + 4, h + 4, DARK_BG);
}

pub fn clear_window() {
    let wx = unsafe { CUR_WIN_X as u32 };
    let wy = unsafe { CUR_WIN_Y as u32 };
    draw_rect(wx.saturating_sub(2), wy.saturating_sub(2), WIN_W + 10, WIN_H + 4, DARK_BG);
}
