// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

use crate::draw::{draw_rect, draw_border, draw_hline, draw_vline, blend_rect, draw_rounded_rect, draw_rounded_border};
use crate::font::{draw_str, draw_str_2x, draw_str_clipped, draw_hex32};
use crate::font16::{draw_str_16, draw_str_16_clipped};

const DARK_BG:          u32 = 0x0D0B1F;
const DARK_BG2:         u32 = 0x1A0E2E;
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
const BROWSE_GREEN:     u32 = 0x2A7A4A;
const TOP_BAR_H:  u32 = 38;
const DOCK_Y:     u32 = 676;
const DOCK_H:     u32 = 44;
const PANEL_W:    u32 = 180;
const TASKBAR_Y:  u32 = 676;
const TASKBAR_H:  u32 = 44;
const CANVAS_Y:   u32 = 38;
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
    // tagline removed
    draw_str(516, 440, "aiXos Phoenix  v0.1.0  aarch64", TEXT_WHITE);
    draw_str(504, 460, "axon_main() -> 0x4153  [SOVEREIGN]", ACCENT_TEAL);
    draw_rect(390, 500, 500, 12, PANEL_BG);
    draw_rect(390, 500, 500, 12, PANEL_BORDER);
    draw_rect(392, 502, 496, 8, ACCENT_TEAL);
}

pub fn render_desktop(state: &DesktopState) {
    // ── PL-59.2: Rich gradient background ────────────────────────────────────
    // Top: deep cosmic purple, bottom: near-black navy
    // Horizontal sweep: subtle purple glow from center
    let mut by: u32 = 0;
    while by < 720 {
        let ty = by * 255 / 720;
        // Vertical: dark purple top -> deep navy bottom
        let r_base = 0x0Du32.saturating_sub(ty / 20);
        let g_base = 0x08u32;
        let b_base = 0x1Fu32.saturating_add(ty / 40);
        // Horizontal sweep per row: brighter toward center x=640
        let mut bx: u32 = 0;
        while bx < 1280 {
            let tx = if bx < 640 { bx * 255 / 640 } else { (1280 - bx) * 255 / 640 };
            let glow = tx / 20; // subtle center brightening
            let r = (r_base + glow / 3).min(0x18) as u8;
            let g = (g_base + glow / 8).min(0x10) as u8;
            let b = (b_base + glow).min(0x38) as u8;
            let color = ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
            // Only draw on canvas (not over panels — save perf)
            if bx >= 188 && bx <= 1092 {
                use crate::framebuffer::{FRAMEBUFFER, WIDTH};
                unsafe {
                    let fb = core::ptr::addr_of_mut!(FRAMEBUFFER) as *mut u32;
                    let offset = by as usize * WIDTH + bx as usize;
                    core::ptr::write_volatile(fb.add(offset), color);
                }
            }
            bx += 1;
        }
        by += 1;
    }
    // Fill panel areas with flat dark color (faster)
    draw_rect(0, 0, 188, 720, 0x0A0816);
    draw_rect(1092, 0, 188, 720, 0x0A0816);

    // ── Stars: varied sizes and brightness ───────────────────────────────────
    let stars: [(u32, u32, u32, u32); 24] = [
        // (x, y, size, color)
        (220,65,1,0x8888AA),(380,42,2,0xCCCCEE),(540,88,1,0x9999BB),
        (720,55,2,0xDDDDFF),(850,30,1,0x7777AA),(960,75,1,0xAAAACC),
        (290,180,1,0x6666AA),(480,210,2,0xBBBBDD),(600,155,1,0x8888BB),
        (780,195,1,0x9999CC),(1020,120,2,0xCCCCEE),(1060,200,1,0x7777AA),
        (240,340,1,0x6666AA),(500,380,1,0x8888CC),(730,360,2,0xAAAADD),
        (880,310,1,0x7777BB),(1000,380,1,0x6666AA),(1050,450,2,0xBBBBCC),
        (210,500,1,0x8888AA),(420,480,2,0xCCCCDD),(680,510,1,0x7777AA),
        (820,490,1,0x9999BB),(1010,520,2,0xBBBBDD),(1070,560,1,0x8888AA),
    ];
    for (sx, sy, sz, sc) in stars.iter() {
        draw_rect(*sx, *sy, *sz, *sz, *sc);
    }
    // Extra tiny pixel stars
    let tiny: [(u32,u32); 16] = [
        (310,95),(455,130),(615,200),(850,240),(930,180),(1045,310),
        (270,265),(560,290),(700,400),(810,430),(950,390),(1060,470),
        (230,440),(470,510),(740,540),(890,560),
    ];
    for (sx, sy) in tiny.iter() {
        draw_rect(*sx, *sy, 1, 1, 0x555577);
    }

    // ── PL-65: Enhanced radial glow — soft sovereign presence ────────────────
    let logo_x: u32 = 640u32.saturating_sub(crate::logo::LOGO_W / 2);
    let logo_y: u32 = 348;

    // Outer diffuse halo
    let gc = logo_x + crate::logo::LOGO_W / 2;
    let gy = logo_y + crate::logo::LOGO_H / 2;
    blend_rect(gc.saturating_sub(120), gy.saturating_sub(120), 240, 240, 0x4B2F80, 6);
    blend_rect(gc.saturating_sub(80),  gy.saturating_sub(80),  160, 160, 0x5B3A9A, 10);
    blend_rect(gc.saturating_sub(60),  gy.saturating_sub(60),  120, 120, 0x6B3FA0, 16);
    blend_rect(gc.saturating_sub(48), gy.saturating_sub(48), 96, 96, 0x6B3FA0, 22);
    blend_rect(gc.saturating_sub(36), gy.saturating_sub(36), 72, 72, 0x7040C0, 32);
    blend_rect(gc.saturating_sub(24), gy.saturating_sub(24), 48, 48, 0x8840FF, 44);

    // Blit the pixel logo on top
    crate::logo::blit_logo(logo_x, logo_y);

    // AIEONYX wordmark — brighter, 2x size
    draw_str_2x(585, logo_y + crate::logo::LOGO_H + 8, "AIEONYX", 0x3A3A6A);
    // Sovereign tagline — subtle
    // tagline removed

    // Left glass panel
    draw_rounded_rect(8, TOP_BAR_H + 8, PANEL_W, 720 - TOP_BAR_H - DOCK_H - 16, 8, GLASS_PANEL);
    draw_rounded_border(8, TOP_BAR_H + 8, PANEL_W, 720 - TOP_BAR_H - DOCK_H - 16, 8, GLASS_BORDER);
    draw_hline(9, TOP_BAR_H + 9, PANEL_W - 2, 0x3A3860);
    draw_str_16(20, TOP_BAR_H + 24, "IDENTITY", 0x44446A);
    // PL-65: Avatar circle — 40x40 purple circle with "E" initial
    let av_x: u32 = 20;
    let av_y: u32 = TOP_BAR_H + 38;
    let av_r: u32 = 20;
    // Draw filled circle via concentric hlines
    let mut ay = 0u32;
    while ay < av_r * 2 {
        let dy = if ay < av_r { av_r - ay } else { ay - av_r };
        let dx = {
            let r2 = av_r * av_r;
            let d2 = dy * dy;
            if d2 > r2 { 0 } else {
                let s = r2 - d2;
                // integer sqrt
                let mut root = s;
                let mut iter = 0;
                while iter < 16 { root = (root + s / (root + 1)) / 2; iter += 1; }
                root
            }
        };
        if dx > 0 {
            draw_hline(av_x + av_r - dx, av_y + ay, dx * 2, SOVEREIGN_PURPLE);
        }
        ay += 1;
    }
    // Highlight top of circle
    blend_rect(av_x, av_y, av_r * 2, av_r, 0xFFFFFF, 25);
    // "E" initial — centred in circle
    let init_x = av_x + av_r - 4;
    let init_y = av_y + av_r - 8;
    if !state.user_name.is_empty() {
        // Use first letter of user name
        if let Ok(s) = core::str::from_utf8(state.user_name) {
            let first = &s[..s.len().min(1)];
            draw_str_16(init_x, init_y, first, TEXT_WHITE);
            // Name below avatar
            draw_str_16(av_x + av_r * 2 + 8, av_y + 6, s, TEXT_WHITE);
            draw_str(av_x + av_r * 2 + 8, av_y + 24, "Sovereign", 0x44446A);
        }
    } else {
        draw_str_16(init_x, init_y, "E", TEXT_WHITE);
        draw_str_16(av_x + av_r * 2 + 8, av_y + 6, "Edison", TEXT_WHITE);
        draw_str(av_x + av_r * 2 + 8, av_y + 24, "Sovereign", 0x44446A);
    }
    draw_hline(16, TOP_BAR_H + 90, PANEL_W - 16, GLASS_BORDER);
    draw_str_16(20, TOP_BAR_H + 104, "SPACES", 0x44446A);
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
    draw_str_16(rx + 12, TOP_BAR_H + 24, "SYSTEM", 0x44446A);
    // PL-65: 6 refined right-panel icons — 3x2 grid, 36x36 each
    // Row 0: Globe (Onyxia), Folder (Files), Terminal (Shell)
    // Row 1: Disk (Storage), Gear (Settings), Antenna (Network)
    let icon_colors: [u32; 6] = [
        0x1850A0,      // Globe — blue
        ACCENT_AMBER,  // Folder — amber
        0x1A6A1A,      // Terminal — green
        0x4A4A6A,      // Disk — slate
        SETTINGS_BLUE, // Gear — blue
        ACCENT_TEAL,   // Antenna — teal
    ];
    let mut ii = 0u32;
    while ii < 6 {
        let col = ii % 3;
        let row = ii / 3;
        let ix = rx + 16 + col * 44;
        let iy = TOP_BAR_H + 42 + row * 44;
        // Button background
        draw_rounded_rect(ix, iy, 36, 36, 6, icon_colors[ii as usize]);
        blend_rect(ix, iy, 36, 36, 0x000000, 150);
        blend_rect(ix, iy, 36, 18, 0xFFFFFF, 18);
        draw_rounded_border(ix, iy, 36, 36, 6, 0x33334A);
        // Sprite origin: 10x10 centred in 36x36 → offset (13,13)
        let sx = ix + 8;
        let sy = iy + 8;
        match ii {
            0 => {
                // Globe (Onyxia browser)
                draw_hline(sx + 3, sy,     14, 0x99BBFF);
                draw_hline(sx + 1, sy + 2, 18, 0x99BBFF);
                draw_hline(sx,     sy + 4, 20, 0x99BBFF);
                draw_hline(sx,     sy + 7, 20, 0x99BBFF);
                draw_hline(sx,     sy + 10, 20, 0x99BBFF);
                draw_hline(sx + 1, sy + 13, 18, 0x99BBFF);
                draw_hline(sx + 3, sy + 15, 14, 0x99BBFF);
                // Meridians
                draw_vline(sx + 9,  sy, 16, 0x3366CC);
                draw_vline(sx + 10, sy, 16, 0x3366CC);
                draw_hline(sx,      sy + 7, 20, 0x3366CC);
            }
            1 => {
                // Folder (Files)
                draw_rect(sx, sy + 2, 8, 3, ACCENT_AMBER);   // tab
                draw_rect(sx, sy + 4, 20, 12, ACCENT_AMBER); // body
                blend_rect(sx, sy + 4, 20, 12, 0x000000, 80);
                draw_border(sx, sy + 4, 20, 12, 0xFFCC44);
                draw_hline(sx + 2, sy + 7,  14, TEXT_WHITE);
                draw_hline(sx + 2, sy + 10, 10, TEXT_WHITE);
            }
            2 => {
                // Terminal `>_`
                // > chevron
                draw_hline(sx,     sy + 2, 5, TEXT_WHITE);
                draw_hline(sx + 1, sy + 4, 5, TEXT_WHITE);
                draw_hline(sx + 2, sy + 6, 5, TEXT_WHITE);
                draw_hline(sx + 1, sy + 8, 5, TEXT_WHITE);
                draw_hline(sx,     sy + 10, 5, TEXT_WHITE);
                // _ underline
                draw_hline(sx + 10, sy + 13, 9, ACCENT_TEAL);
                draw_rect(sx + 10, sy + 9, 4, 3, ACCENT_TEAL);
            }
            3 => {
                // Disk / Storage
                // Top ellipse
                draw_hline(sx + 4, sy,     12, 0xAAAACC);
                draw_hline(sx + 2, sy + 1, 16, 0xAAAACC);
                draw_hline(sx + 2, sy + 3, 16, 0xAAAACC);
                draw_hline(sx + 4, sy + 4, 12, 0xAAAACC);
                // Cylinder sides
                draw_vline(sx + 2,  sy + 2, 10, 0xAAAACC);
                draw_vline(sx + 17, sy + 2, 10, 0xAAAACC);
                // Bottom ellipse
                draw_hline(sx + 4, sy + 12, 12, 0xAAAACC);
                draw_hline(sx + 2, sy + 13, 16, 0xAAAACC);
                draw_hline(sx + 4, sy + 14, 12, 0xAAAACC);
                // Label slot
                draw_rect(sx + 5, sy + 6, 8, 3, 0x222240);
                draw_hline(sx + 6, sy + 7, 5, ACCENT_TEAL);
            }
            4 => {
                // Gear (Settings)
                draw_rect(sx + 6, sy + 2,  8, 16, SETTINGS_BLUE);
                draw_rect(sx + 2, sy + 6, 16,  8, SETTINGS_BLUE);
                // Teeth
                draw_rect(sx,     sy + 5,  3,  3, SETTINGS_BLUE);
                draw_rect(sx + 17,sy + 5,  3,  3, SETTINGS_BLUE);
                draw_rect(sx,     sy + 12, 3,  3, SETTINGS_BLUE);
                draw_rect(sx + 17,sy + 12, 3,  3, SETTINGS_BLUE);
                draw_rect(sx + 5, sy,      3,  3, SETTINGS_BLUE);
                draw_rect(sx + 12,sy,      3,  3, SETTINGS_BLUE);
                draw_rect(sx + 5, sy + 17, 3,  3, SETTINGS_BLUE);
                draw_rect(sx + 12,sy + 17, 3,  3, SETTINGS_BLUE);
                // Centre hole
                draw_rounded_rect(sx + 7, sy + 7, 6, 6, 3, 0x0A0818);
            }
            _ => {
                // Antenna / Network
                // Mast
                draw_vline(sx + 9,  sy + 6, 14, ACCENT_TEAL);
                draw_vline(sx + 10, sy + 6, 14, ACCENT_TEAL);
                // Signal arcs (3 concentric)
                draw_hline(sx + 5, sy + 2, 10, ACCENT_TEAL);
                draw_hline(sx + 3, sy + 4, 14, ACCENT_TEAL);
                draw_hline(sx + 1, sy + 6, 18, ACCENT_TEAL);
                // Base
                draw_hline(sx + 5, sy + 18, 10, ACCENT_TEAL);
            }
        }
        ii += 1;
    }
    draw_hline(rx + 8, TOP_BAR_H + 138, PANEL_W - 16, GLASS_BORDER);
    draw_str(rx + 16, TOP_BAR_H + 156, "RESOURCES", 0x44446A);
    let bar_w: u32 = PANEL_W - 48;
    let bar_x: u32 = rx + 16;
    // PL-65: CPU bar (30% sovereign idle) + MEM bar
    let cpu_pct: u32 = 30;
    draw_str(rx + 16, TOP_BAR_H + 172, "CPU", 0x888899);
    draw_str(rx + PANEL_W - 36, TOP_BAR_H + 172, "30%", 0x888899);
    draw_rect(bar_x, TOP_BAR_H + 182, bar_w, 5, 0x22224A);
    draw_rect(bar_x, TOP_BAR_H + 182, bar_w * cpu_pct / 100, 5, SOVEREIGN_PURPLE);
    let mem_pct: u32 = 55;
    draw_str(rx + 16, TOP_BAR_H + 194, "MEM", 0x888899);
    draw_str(rx + PANEL_W - 36, TOP_BAR_H + 194, "55%", 0x888899);
    draw_rect(bar_x, TOP_BAR_H + 204, bar_w, 5, 0x22224A);
    draw_rect(bar_x, TOP_BAR_H + 204, bar_w * mem_pct / 100, 5, ACCENT_TEAL);
    // Network section
    draw_hline(rx + 8, TOP_BAR_H + 218, PANEL_W - 16, GLASS_BORDER);
    draw_str(rx + 16, TOP_BAR_H + 234, "NETWORK", 0x44446A);
    let awp_col = if state.edb_live { ACCENT_TEAL } else { 0x444444 };
    draw_rounded_rect(rx + 16, TOP_BAR_H + 248, 8, 8, 4, awp_col);
    draw_str(rx + 30, TOP_BAR_H + 256, "AWP mesh active", 0x888899);
    draw_str(rx + 16, TOP_BAR_H + 270, "0 peers · local only", 0x444466);


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
    draw_str_16(582, 8, "aiXos Phoenix", 0x555588);
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


pub fn render_taskbar(slots: &[(bool, u8, bool)], active: usize) {
    draw_rect(0, DOCK_Y, 1280, DOCK_H, 0x0A0818);
    draw_hline(0, DOCK_Y, 1280, 0x1A1830);
    // PL-65: 7 icons x 34px + 6px gap + 20px padding = 314px
    // Full dock, no axos> prompt (removed as redundant)
    let dock_w: u32 = 314;
    let dock_x: u32 = (1280 - dock_w) / 2;
    let dock_py: u32 = DOCK_Y + 4;
    draw_rounded_rect(dock_x, dock_py, dock_w, 36, 10, 0x100E20);
    draw_rounded_border(dock_x, dock_py, dock_w, 36, 10, 0x2A2848);
    draw_hline(dock_x + 10, dock_py + 1, dock_w - 20, 0x3A3858);
    // PL-59.5: 7 app icons with 16×16 pixel sprites
    // Icons: 0=Onyxia 1=Browser 2=Shell 3=Files 4=EDB 5=IAM 6=Settings
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
        // Button background
        draw_rounded_rect(ix, iy, icon_w, 26, 4, colors[di as usize]);
        blend_rect(ix, iy, icon_w, 26, 0x000000, 110);
        blend_rect(ix, iy, icon_w, 13, 0xFFFFFF, 22);
        draw_rounded_border(ix, iy, icon_w, 26, 4, 0x33334A);
        // Sprite origin: center 16x16 in the 30x26 button → offset (7, 5)
        let sx = ix + 7;
        let sy = iy + 5;
        match di {
            0 => {
                // Onyxia: sovereign diamond — 4 triangles forming ◇
                // Top triangle  (rows 0-4): 1→9 wide
                draw_hline(sx + 7, sy,     2, TEXT_WHITE);
                draw_hline(sx + 5, sy + 1, 6, TEXT_WHITE);
                draw_hline(sx + 3, sy + 2, 10, TEXT_WHITE);
                draw_hline(sx + 1, sy + 3, 14, TEXT_WHITE);
                // Middle row
                draw_hline(sx,     sy + 4, 16, 0xCCBBFF);
                // Bottom triangle (rows 5-9)
                draw_hline(sx + 1, sy + 5, 14, TEXT_WHITE);
                draw_hline(sx + 3, sy + 6, 10, TEXT_WHITE);
                draw_hline(sx + 5, sy + 7, 6,  TEXT_WHITE);
                draw_hline(sx + 7, sy + 8, 2,  TEXT_WHITE);
                // Inner glow
                draw_hline(sx + 6, sy + 3, 4, 0xFFFFFF);
                draw_hline(sx + 5, sy + 4, 6, 0xDDCCFF);
            }
            1 => {
                // Browser: globe — circle outline + meridians
                draw_hline(sx + 4, sy,     8, 0x99BBFF);
                draw_hline(sx + 2, sy + 1, 12, 0x99BBFF);
                draw_hline(sx + 1, sy + 2, 14, 0x99BBFF);
                draw_hline(sx,     sy + 3, 16, 0x99BBFF);
                draw_hline(sx,     sy + 4, 16, 0x99BBFF);
                draw_hline(sx,     sy + 5, 16, 0x99BBFF);
                draw_hline(sx + 1, sy + 6, 14, 0x99BBFF);
                draw_hline(sx + 2, sy + 7, 12, 0x99BBFF);
                draw_hline(sx + 4, sy + 8, 8,  0x99BBFF);
                // Meridian cross: vertical + horizontal lines in darker blue
                draw_vline(sx + 7, sy,     9, 0x3366CC);
                draw_vline(sx + 8, sy,     9, 0x3366CC);
                draw_hline(sx,     sy + 4, 16, 0x3366CC);
            }
            2 => {
                // Shell: >_ prompt icon
                // > chevron (left half)
                draw_hline(sx,     sy + 1, 4, TEXT_WHITE);
                draw_hline(sx + 1, sy + 2, 4, TEXT_WHITE);
                draw_hline(sx + 2, sy + 3, 4, TEXT_WHITE);
                draw_hline(sx + 2, sy + 4, 4, TEXT_WHITE);
                draw_hline(sx + 1, sy + 5, 4, TEXT_WHITE);
                draw_hline(sx,     sy + 6, 4, TEXT_WHITE);
                // _ underline (right half, bottom)
                draw_hline(sx + 9, sy + 7, 7, ACCENT_TEAL);
                draw_hline(sx + 9, sy + 8, 7, ACCENT_TEAL);
                // Cursor blink block
                draw_rect(sx + 9, sy + 4, 3, 3, ACCENT_TEAL);
            }
            3 => {
                // Files / AXFS: folder icon
                // Tab (top-left of folder)
                draw_rect(sx, sy, 6, 2, ACCENT_AMBER);
                // Folder body
                draw_rect(sx, sy + 2, 16, 7, ACCENT_AMBER);
                blend_rect(sx, sy + 2, 16, 7, 0x000000, 80);
                // Folder body border
                draw_border(sx, sy + 2, 16, 7, 0xFFCC44);
                // Lines inside (files)
                draw_hline(sx + 2, sy + 4, 11, TEXT_WHITE);
                draw_hline(sx + 2, sy + 6, 8,  TEXT_WHITE);
            }
            4 => {
                // EDB Browser: database cylinder stack
                // Top ellipse
                draw_hline(sx + 3, sy,     10, BROWSE_GREEN);
                draw_hline(sx + 1, sy + 1, 14, BROWSE_GREEN);
                draw_hline(sx + 1, sy + 2, 14, BROWSE_GREEN);
                draw_hline(sx + 3, sy + 3, 10, BROWSE_GREEN);
                // Cylinder body sides (3 rows each for 2 tiers)
                draw_vline(sx + 1,  sy + 2, 7, BROWSE_GREEN);
                draw_vline(sx + 14, sy + 2, 7, BROWSE_GREEN);
                // Mid-tier ellipse separator
                draw_hline(sx + 3, sy + 4, 10, ACCENT_TEAL);
                draw_hline(sx + 1, sy + 5, 14, ACCENT_TEAL);
                // Bottom ellipse
                draw_hline(sx + 3, sy + 8, 10, BROWSE_GREEN);
                draw_hline(sx + 1, sy + 7, 14, BROWSE_GREEN);
            }
            5 => {
                // IAM: person silhouette
                // Head
                draw_rect(sx + 5, sy,     6, 5, 0xCCBBFF);
                // Shoulders / body
                draw_hline(sx + 2, sy + 6, 12, 0xCCBBFF);
                draw_hline(sx + 1, sy + 7, 14, 0xCCBBFF);
                draw_rect(sx + 2, sy + 7, 12, 2, 0xCCBBFF);
                // ID badge dot
                draw_rect(sx + 6, sy + 2, 4, 3, 0x7B4FDB);
                draw_hline(sx + 7, sy + 2, 2, TEXT_WHITE);
            }
            _ => {
                // Settings (6): gear — cross body + 4 notch bumps
                // Central cross
                draw_rect(sx + 5, sy + 2, 6, 12, SETTINGS_BLUE);
                draw_rect(sx + 2, sy + 5, 12, 6, SETTINGS_BLUE);
                // Corner notches (gear teeth)
                draw_rect(sx + 1, sy + 4, 2, 2, SETTINGS_BLUE);
                draw_rect(sx + 13, sy + 4, 2, 2, SETTINGS_BLUE);
                draw_rect(sx + 1, sy + 10, 2, 2, SETTINGS_BLUE);
                draw_rect(sx + 13, sy + 10, 2, 2, SETTINGS_BLUE);
                // Centre hole
                draw_rect(sx + 6, sy + 6, 4, 4, 0x0D0D22);
            }
        }
        di += 1;
    }
    // Separator
    // PL-65: no axos> prompt — dock is icons only
    // Open window indicators — teal dot = open, amber dot = minimized
    let mut wi = 0usize;
    while wi < slots.len() {
        if slots[wi].0 {
            let kind = slots[wi].1 as u32;
            let minimized = slots[wi].2;
            // Map window kind to dock icon index
            let dock_idx: u32 = match kind {
                1 => 2, // Shell -> >_
                2 => 4, // EDB  -> D
                3 => 6, // Set  -> S
                4 => 4, // EDB browser -> D
                7 => 0, // Onyxia browser -> O (diamond icon)
                8 => 5, // Process window -> I (person/IAM icon)
                9 => 3, // File Browser -> F (folder icon)
                _ => 0,
            };
            let dot_x = dock_x + 10 + dock_idx * (icon_w + icon_gap) + icon_w / 2 - 3;
            let dot_col = if minimized { ACCENT_AMBER } else { ACCENT_TEAL };
            draw_rect(dot_x, dock_py + 2, 6, 2, dot_col);
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
    // PL-65: axos> prompt removed
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
// PL-59.5: Canvas safe-zone — windows must not overlap left panel (ends ~196)
//           or dock (starts 676) or top bar (ends 38) or right panel (starts 1092)
pub const CANVAS_X_MIN: i32 = 200;   // clear of left panel right edge (180+8+12)
pub const CANVAS_Y_MIN: i32 = 50;    // clear of top bar (38)
pub const CANVAS_X_MAX: i32 = 880;   // right panel starts at 1092; 880+580+8 < 1084 if narrow
pub const CANVAS_Y_MAX: i32 = 370;   // y+300 ≤ 670 keeps window bottom above dock (676)
const WIN_X: u32 = 340;
const WIN_Y: u32 = 110;
const WIN_W: u32 = 580;
const WIN_H: u32 = 300;
const WIN_TITLE_H: u32 = 24;
const WIN_BG:    u32 = 0x0D0D22;
const WIN_TITLE: u32 = 0x1A1A3A;
const GLASS_HI:  u32 = 0x3A3A5A;
const GLASS_MID: u32 = 0x1E1E38;
const GLASS_LOW: u32 = 0x111128;
const SHADOW:    u32 = 0x000008;
const CLOSE_RED: u32 = 0xC0392B;

static mut CUR_WIN_X: i32 = 200;
static mut CUR_WIN_Y: i32 = 80;

pub fn set_window_pos(x: i32, y: i32) {
    unsafe { CUR_WIN_X = x; CUR_WIN_Y = y; }
}
pub fn get_window_pos() -> (i32, i32) {
    unsafe { (CUR_WIN_X, CUR_WIN_Y) }
}

/// PL-59.5: Clamp a window spawn position to the canvas safe zone.
/// Returns (clamped_x, clamped_y) — always inside left panel right edge and above dock.
pub fn clamp_spawn_pos(x: i32, y: i32) -> (i32, i32) {
    let cx = if x < CANVAS_X_MIN { CANVAS_X_MIN } else if x > CANVAS_X_MAX { CANVAS_X_MAX } else { x };
    let cy = if y < CANVAS_Y_MIN { CANVAS_Y_MIN } else if y > CANVAS_Y_MAX { CANVAS_Y_MAX } else { y };
    (cx, cy)
}

pub fn dock_icon_at(x: i32, y: i32) -> Option<u8> {
    let dy = DOCK_Y as i32;
    if y < dy || y > dy + 44 { return None; }
    let dock_x: i32 = (1280 - 314) / 2; // matches new dock_w=314
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
    draw_rounded_border(wx.saturating_sub(1), wy.saturating_sub(1), w + 2, h + 2, 6, ACCENT_TEAL);
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
    draw_str_16_clipped(wx + 24, wy + 4, title, TEXT_WHITE, wx + w - 52);
    // PL-65: macOS-style filled circle controls — right side
    // Red=close, Amber=minimize, Green=maximize
    let cy = wy + WIN_TITLE_H / 2;
    // Close (red) — rightmost
    let cx = wx + w - 14;
    draw_rounded_rect(cx - 5, cy - 5, 10, 10, 5, 0xC0392B);
    blend_rect(cx - 5, cy - 5, 10, 5, 0xFFFFFF, 30);
    // Minimize (amber) — middle
    let mnx = cx - 18;
    draw_rounded_rect(mnx - 5, cy - 5, 10, 10, 5, 0xE67E22);
    blend_rect(mnx - 5, cy - 5, 10, 5, 0xFFFFFF, 30);
    // Maximize (green) — left of amber
    let mxx = mnx - 18;
    draw_rounded_rect(mxx - 5, cy - 5, 10, 10, 5, 0x27AE60);
    blend_rect(mxx - 5, cy - 5, 10, 5, 0xFFFFFF, 30);
    draw_hline(wx, wy + WIN_TITLE_H, w, ACCENT_TEAL);
    draw_rect(wx, wy + WIN_TITLE_H + 1, w, h - WIN_TITLE_H - 1, WIN_BG);
    blend_rect(wx, wy + WIN_TITLE_H + 1, w, h - WIN_TITLE_H - 1, SOVEREIGN_PURPLE, 12);
    let mut row = 0u32;
    let max_rows = if h > WIN_TITLE_H + 20 { (h - WIN_TITLE_H - 20) / 18 } else { 0 };
    for line in lines.iter().take(max_rows as usize) {
        draw_str_clipped(wx + 12, wy + WIN_TITLE_H + 12 + row * 18, line, TEXT_WHITE, wx + w - 8);
        row += 1;
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
        // PL-59.4: 8x16 font for shell output
        draw_str_16_clipped((wx + 8) as u32, y as u32, lines[idx], TEXT_WHITE,
            (wx as u32).saturating_add(ww).saturating_sub(12));
        y += 20;
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
    draw_str_16((wx + 8) as u32, (y - 2) as u32, "axc> ", ACCENT_TEAL);
    if let Ok(txt) = core::str::from_utf8(&buf[..len]) {
        draw_str((wx + 48) as u32, y as u32, txt, TEXT_WHITE);
    }
    draw_str((wx + 48 + (len as i32) * 9) as u32, y as u32, "_", TEXT_WHITE);
    if focused {
        // Draw [focused] inside window right edge
        let fx = (wx as u32 + ww).saturating_sub(80);
        draw_str_16(fx, (y - 2) as u32, "[focused]", TEXT_DIM);
    }
}


// ── PL-52: AXFS Files Window ─────────────────────────────────────────────────

/// Render the sovereign AXFS file browser window.
/// mode=0: file list  mode=1: file content view
pub fn render_files_window(
    wx: i32, wy: i32, w: u32, h: u32,
    // file list: names as (ptr, len) pairs
    file_names: &[(*const u8, usize)],
    file_count: usize,
    cursor: usize,
    // content view
    content: &[u8],
    content_len: usize,
    viewing: bool,
) {
    let wx_u = wx as u32;
    let wy_u = wy as u32;
    // Window chrome
    draw_rect(wx_u, wy_u, w, h, 0x0A0818);
    draw_border(wx_u, wy_u, w, h, GLASS_BORDER);
    draw_rect(wx_u, wy_u, w, 20, GLASS_PANEL);
    draw_hline(wx_u, wy_u + 20, w, SOVEREIGN_PURPLE);
    // Title
    if viewing {
        draw_str(wx_u + 8, wy_u + 14, "AXFS - File View", 0x888899);
    } else {
        draw_str(wx_u + 8, wy_u + 14, "AXFS - Files", 0x888899);
        draw_str(wx_u + w - 120, wy_u + 14, "[Enter=open Esc=back]", 0x44446A);
    }

    let row_h: u32 = 18;
    let start_y = wy_u + 28;

    if viewing {
        // Render file content line by line
        let mut line_start = 0usize;
        let mut row = 0u32;
        let max_rows = (h.saturating_sub(36)) / row_h;
        let mut i = 0;
        while i <= content_len && row < max_rows {
            let end = i == content_len || content[i] == b'\n';
            if end {
                let line = &content[line_start..i];
                if let Ok(s) = core::str::from_utf8(line) {
                    draw_str_clipped(wx_u + 8, start_y + row * row_h, s, TEXT_WHITE, wx_u + w - 8);
                }
                row += 1;
                line_start = i + 1;
            }
            i += 1;
        }
        draw_str(wx_u + 8, wy_u + h - 14, "Esc: back to list", 0x44446A);
    } else {
        // Render file list
        let max_rows = (h.saturating_sub(36)) / row_h;
        let mut fi = 0usize;
        while fi < file_count && (fi as u32) < max_rows {
            let (ptr, len) = file_names[fi];
            let row_y = start_y + fi as u32 * row_h;
            let is_selected = fi == cursor;
            if is_selected {
                draw_rect(wx_u + 4, row_y - 2, w - 8, row_h, SOVEREIGN_PURPLE);
            }
            let col = if is_selected { TEXT_WHITE } else { 0x888899 };
            draw_str(wx_u + 12, row_y + 10, if is_selected { ">" } else { " " }, ACCENT_TEAL);
            // Draw filename from raw ptr
            let name_bytes = unsafe { core::slice::from_raw_parts(ptr, len) };
            if let Ok(s) = core::str::from_utf8(name_bytes) {
                draw_str_clipped(wx_u + 24, row_y + 10, s, col, wx_u + w - 8);
            }
            fi += 1;
        }
        if file_count == 0 {
            draw_str(wx_u + 12, start_y + 10, "[empty filesystem]", 0x44446A);
        }
        draw_str(wx_u + 8, wy_u + h - 14, "arrows: navigate  Enter: open  Esc: close", 0x44446A);
    }
}

// ── PL-32: EDB Browser Window ────────────────────────────────────────────────

pub struct EdbEntry {
    pub key:   &'static str,
    pub tier:  &'static str,
    pub value: u64,
}

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
    draw_str_16(wx_u + 8, input_row_y, "edb>", ACCENT_TEAL);
    let buf_x = wx_u + 40;
    if let Ok(txt) = core::str::from_utf8(&input_buf[..input_len]) {
        // PL-59.4: 8x16 input line
    draw_str_16(buf_x, input_row_y, txt, TEXT_WHITE);
    }
    draw_str_16(buf_x + (input_len as u32) * 9, input_row_y, "_", ACCENT_TEAL);
    if focused { draw_str_16(wx_u + w - 80, input_row_y, "[focused]", TEXT_DIM); }
    draw_rect(wx_u + w - 12, (wy + h as i32 - 12) as u32, 12, 12, ACCENT_TEAL);
    draw_rect(wx_u + w - 8,  (wy + h as i32 - 8) as u32,  4,  4,  TEXT_WHITE);
}

pub fn clear_window_sized(w: u32, h: u32) {
    let wx = unsafe { CUR_WIN_X as u32 };
    let wy = unsafe { CUR_WIN_Y as u32 };
    draw_rect(wx.saturating_sub(2), wy.saturating_sub(2), w + 4, h + 4, DARK_BG);
}

/// PL-63: Query which title bar control was clicked.
/// Returns: 0=none, 1=close, 2=maximize, 3=minimize
pub fn title_bar_hit(wx: i32, wy: i32, w: u32, click_x: i32, click_y: i32) -> u8 {
    let title_h = WIN_TITLE_H as i32;
    if click_y < wy || click_y > wy + title_h { return 0; }
    let cy = wy + (WIN_TITLE_H / 2) as i32;
    // Circle centres (right side):
    let cx  = wx + w as i32 - 14; // close (red)
    let mnx = cx - 18;             // minimize (amber)
    let mxx = mnx - 18;            // maximize (green)
    let r: i32 = 6; // hit radius
    let in_y = (click_y - cy).abs() <= r;
    if in_y && (click_x - cx).abs()  <= r { return 1; } // close
    if in_y && (click_x - mnx).abs() <= r { return 3; } // minimize
    if in_y && (click_x - mxx).abs() <= r { return 2; } // maximize
    0
}

pub fn clear_window() {
    let wx = unsafe { CUR_WIN_X as u32 };
    let wy = unsafe { CUR_WIN_Y as u32 };
    draw_rect(wx.saturating_sub(2), wy.saturating_sub(2), WIN_W + 10, WIN_H + 4, DARK_BG);
}

// ── PL-60: Onyxia Browser Window ─────────────────────────────────────────────
// Sovereign awp:// browser — URL bar + page canvas placeholder.
// url_buf/url_len: the current URL being typed or displayed.
// url_focused: true when the URL bar has keyboard focus.
// loaded: true after Enter pressed — shows "page loaded" state.
// ── PL-61: HANIEL Canvas Compositor ──────────────────────────────────────────
// Sovereign document model for awp:// pages rendered inside Onyxia Browser.
// All fields are &'static str — pages are compile-time sovereign content.

pub const HANIEL_MAX_BODY:  usize = 12;
pub const HANIEL_MAX_LINKS: usize = 6;

pub struct HanielDoc {
    pub title:      &'static str,
    pub subtitle:   &'static str,
    pub body:       [&'static str; HANIEL_MAX_BODY],
    pub body_len:   usize,
    pub links:      [&'static str; HANIEL_MAX_LINKS],
    pub link_len:   usize,
    /// 0 = normal, 1 = home/index, 2 = status/live, 3 = error
    pub page_kind:  u8,
}

impl HanielDoc {
    pub const fn empty() -> Self {
        HanielDoc {
            title: "", subtitle: "",
            body: [""; HANIEL_MAX_BODY], body_len: 0,
            links: [""; HANIEL_MAX_LINKS], link_len: 0,
            page_kind: 0,
        }
    }
}

/// Render a HanielDoc into the Onyxia Browser page canvas.
/// canvas_top / canvas_h define the drawable region inside the window.
pub fn render_haniel_canvas(
    wx_u: u32, canvas_top: u32, w: u32, canvas_h: u32,
    doc: &HanielDoc,
) {
    // ── Page background ───────────────────────────────────────────────────────
    draw_rect(wx_u + 1, canvas_top, w - 2, canvas_h, WIN_BG);

    // Kind-specific accent tint
    let tint: u32 = match doc.page_kind {
        1 => SOVEREIGN_PURPLE, // home
        2 => ACCENT_TEAL,      // status
        3 => 0xA02020,         // error
        _ => 0x181830,         // normal
    };
    blend_rect(wx_u + 1, canvas_top, w - 2, canvas_h, tint, 14);

    // Top accent line
    draw_hline(wx_u + 1, canvas_top, w - 2, tint);

    // ── Page header ───────────────────────────────────────────────────────────
    let hdr_y = canvas_top + 8;

    // AWP badge
    draw_rounded_rect(wx_u + 10, hdr_y, 28, 13, 2, 0x0D2A1A);
    draw_border(wx_u + 10, hdr_y, 28, 13, ACCENT_TEAL);
    draw_str(wx_u + 13, hdr_y + 8, "awp", ACCENT_TEAL);

    // Title
    let title_col: u32 = match doc.page_kind {
        3 => 0xCC4444,
        _ => TEXT_WHITE,
    };
    draw_str_16_clipped(wx_u + 44, hdr_y, doc.title, title_col, wx_u + w - 10);

    // Subtitle
    if !doc.subtitle.is_empty() {
        draw_str_clipped(wx_u + 44, hdr_y + 18, doc.subtitle, TEXT_DIM, wx_u + w - 10);
    }

    // Header separator
    let sep_y = hdr_y + 32;
    draw_hline(wx_u + 10, sep_y, w - 20, PANEL_BORDER);

    // ── Body text ─────────────────────────────────────────────────────────────
    let body_y0 = sep_y + 6;
    let row_h: u32 = 16;
    let mut bi = 0usize;
    while bi < doc.body_len {
        let line = doc.body[bi];
        let ly = body_y0 + bi as u32 * row_h;
        // Section headers: lines starting with "##" render in teal, no prefix
        if line.len() >= 2 && &line[..2] == "##" {
            draw_str_clipped(wx_u + 10, ly, &line[2..], ACCENT_TEAL, wx_u + w - 10);
        } else if line.len() >= 2 && &line[..2] == ">>" {
            // Highlighted line: amber
            draw_str_clipped(wx_u + 14, ly, &line[2..], ACCENT_AMBER, wx_u + w - 10);
        } else {
            draw_str_clipped(wx_u + 14, ly, line, TEXT_WHITE, wx_u + w - 10);
        }
        bi += 1;
    }

    // ── Link list ─────────────────────────────────────────────────────────────
    if doc.link_len > 0 {
        let links_y0 = if doc.body_len > 0 {
            body_y0 + doc.body_len as u32 * row_h + 6
        } else {
            body_y0
        };
        // Separator before links
        draw_hline(wx_u + 10, links_y0 - 3, w - 20, PANEL_BORDER);
        let mut li = 0usize;
        while li < doc.link_len {
            let ly = links_y0 + li as u32 * 15;
            // Link bullet
            draw_rect(wx_u + 12, ly + 3, 4, 4, SOVEREIGN_PURPLE);
            draw_str_clipped(wx_u + 20, ly, doc.links[li], SOVEREIGN_PURPLE, wx_u + w - 60);
            draw_str(wx_u + w - 54, ly, "[Tab+Enter]", TEXT_DIM);
            li += 1;
        }
    }

    // ── Sovereign footer ──────────────────────────────────────────────────────
    let foot_y = canvas_top + canvas_h - 14;
    draw_hline(wx_u + 10, foot_y - 2, w - 20, PANEL_BORDER);
    draw_str_clipped(wx_u + 12, foot_y, "AIEONYX Sovereign Digital Infrastructure", 0x222240, wx_u + w / 2);
    draw_str_clipped(wx_u + w / 2 + 10, foot_y, "HANIEL compositor  PL-61", 0x222240, wx_u + w - 4);
}

pub fn render_onyxia_browser(
    wx: i32, wy: i32, w: u32, h: u32,
    url_buf: &[u8], url_len: usize,
    url_focused: bool,
    loaded: bool,
    doc: &HanielDoc,
) {
    let wx_u = wx as u32;
    let wy_u = wy as u32;

    // ── URL bar row (below title bar) ────────────────────────────────────────
    let bar_y = wy_u + WIN_TITLE_H + 2;
    let bar_h: u32 = 22;

    // URL bar background — dark panel
    draw_rect(wx_u + 1, bar_y, w - 2, bar_h, 0x0A0818);

    // Back / forward stub buttons (◀ ▶) — purely decorative at PL-60
    draw_rounded_rect(wx_u + 4,  bar_y + 3, 14, 16, 2, 0x1A1A3A);
    draw_rounded_rect(wx_u + 20, bar_y + 3, 14, 16, 2, 0x1A1A3A);
    draw_str(wx_u + 7,  bar_y + 11, "<", TEXT_DIM);
    draw_str(wx_u + 23, bar_y + 11, ">", TEXT_DIM);

    // Reload button stub (circle arrow ↺)
    draw_rounded_rect(wx_u + 38, bar_y + 3, 14, 16, 2, 0x1A1A3A);
    draw_str(wx_u + 41, bar_y + 11, "o", TEXT_DIM);

    // awp:// scheme badge
    let badge_x = wx_u + 56;
    draw_rounded_rect(badge_x, bar_y + 4, 30, 14, 2, 0x0D2A1A);
    draw_border(badge_x, bar_y + 4, 30, 14, ACCENT_TEAL);
    draw_str(badge_x + 3, bar_y + 12, "awp", ACCENT_TEAL);

    // URL input field
    let field_x = badge_x + 34;
    let field_w = if w > field_x - wx_u + 8 { w - (field_x - wx_u) - 8 } else { 4 };
    let border_col = if url_focused { ACCENT_TEAL } else { 0x2A2848 };
    draw_rect(field_x, bar_y + 3, field_w, 16, 0x06060F);
    draw_border(field_x, bar_y + 3, field_w, 16, border_col);

    // URL text
    let text_x = field_x + 4;
    let text_y = bar_y + 11;
    if url_len > 0 {
        if let Ok(txt) = core::str::from_utf8(&url_buf[..url_len]) {
            draw_str_clipped(text_x, text_y, txt, TEXT_WHITE, field_x + field_w - 4);
        }
    } else if !url_focused {
        draw_str_clipped(text_x, text_y, "awp://", TEXT_DIM, field_x + field_w - 4);
    }
    // Cursor blink
    if url_focused {
        let cur_x = text_x + (url_len as u32) * 8;
        draw_rect(cur_x.min(field_x + field_w - 6), text_y - 1, 2, 12, ACCENT_TEAL);
    }

    // Separator under URL bar
    let sep_y = bar_y + bar_h;
    draw_hline(wx_u + 1, sep_y, w - 2, ACCENT_TEAL);

    // ── Page canvas area ─────────────────────────────────────────────────────
    let canvas_top = sep_y + 1;
    let canvas_h = if h > canvas_top - wy_u + 2 { h - (canvas_top - wy_u) - 2 } else { 4 };

    // Page background
    draw_rect(wx_u + 1, canvas_top, w - 2, canvas_h, WIN_BG);
    blend_rect(wx_u + 1, canvas_top, w - 2, canvas_h, SOVEREIGN_PURPLE, 8);

    if loaded && url_len > 0 {
        // ── PL-61: HANIEL compositor renders the routed document ──────────────
        render_haniel_canvas(wx_u, canvas_top, w, canvas_h, doc);
    } else {
        // ── Unloaded state: new tab sovereign splash ──────────────────────────
        // AIEONYX wordmark centred in canvas
        let mid_y = canvas_top + canvas_h / 2;

        // Diamond logo (small, centred) — 5-row hline ◇
        let logo_x = wx_u + w / 2 - 8;
        let logo_y = mid_y - 30;
        draw_hline(logo_x + 7, logo_y,     2, SOVEREIGN_PURPLE);
        draw_hline(logo_x + 5, logo_y + 1, 6, SOVEREIGN_PURPLE);
        draw_hline(logo_x + 3, logo_y + 2, 10, SOVEREIGN_PURPLE);
        draw_hline(logo_x + 5, logo_y + 3, 6, SOVEREIGN_PURPLE);
        draw_hline(logo_x + 7, logo_y + 4, 2, SOVEREIGN_PURPLE);

        // Tagline
        draw_str_clipped(wx_u + w / 2 - 38, mid_y - 14,
            "Onyxia Browser", TEXT_DIM, wx_u + w - 8);
        draw_str_16_clipped(wx_u + w / 2 - 54, mid_y,
            "awp:// sovereign web", SOVEREIGN_PURPLE, wx_u + w - 8);

        // Hint line
        draw_str_clipped(wx_u + w / 2 - 76, mid_y + 22,
            "Tab = focus URL  Enter = navigate  Esc = clear", 0x2A2848, wx_u + w - 8);
    }

    // Resize handle
    draw_rect(wx_u + w - 12, wy_u + h - 12, 12, 12, ACCENT_TEAL);
    draw_rect(wx_u + w - 8,  wy_u + h - 8,  4,  4,  TEXT_WHITE);

    // [focused] badge if URL bar active
    if url_focused {
        draw_str_16(wx_u + w - 80, bar_y + 11, "[focused]", TEXT_DIM);
    }
}

// ── PL-62: Process Window ─────────────────────────────────────────────────────
// Renders the sovereign process table inside a standard window chrome.
// proc_slots: array of (name, state_char, priority, ticks, pid) tuples.
// count: number of valid slots.
// tick_total: global scheduler tick counter for header display.

pub struct ProcSlot {
    pub pid:      u8,
    pub name:     [u8; 16],
    pub name_len: usize,
    /// 'R'=Running 'W'=Ready 'B'=Blocked 'D'=Dead
    pub state_ch: u8,
    pub priority: u8,
    pub ticks:    u64,
    pub yields:   u32,
}

impl ProcSlot {
    pub const fn empty() -> Self {
        ProcSlot { pid:0, name:[0u8;16], name_len:0, state_ch:0, priority:0, ticks:0, yields:0 }
    }
    pub fn name_str(&self) -> &str {
        core::str::from_utf8(&self.name[..self.name_len]).unwrap_or("?")
    }
}

pub fn render_proc_window(
    wx: i32, wy: i32, w: u32, h: u32,
    slots: &[ProcSlot],
    count: usize,
    tick_total: u64,
) {
    let wx_u = wx as u32;
    let wy_u = wy as u32;
    let _ = wy_u;

    // ── Column header row ─────────────────────────────────────────────────────
    let hdr_y = (wy + WIN_TITLE_H as i32 + 4) as u32;
    draw_rect(wx_u + 1, hdr_y, w - 2, 14, 0x0A0A1A);
    draw_str(wx_u + 6,  hdr_y + 4, "PID", TEXT_DIM);
    draw_str(wx_u + 28, hdr_y + 4, "NAME            ", TEXT_DIM);
    draw_str(wx_u + 148, hdr_y + 4, "ST", TEXT_DIM);
    draw_str(wx_u + 168, hdr_y + 4, "PRI", TEXT_DIM);
    draw_str(wx_u + 196, hdr_y + 4, "TICKS", TEXT_DIM);
    draw_str(wx_u + 252, hdr_y + 4, "YIELDS", TEXT_DIM);

    // Tick total in header right
    draw_str(wx_u + w - 120, hdr_y + 4, "tick:", TEXT_DIM);
    draw_hex32(wx_u + w - 84, hdr_y + 4, tick_total as u32, ACCENT_TEAL);

    let sep_y = hdr_y + 14;
    draw_hline(wx_u + 4, sep_y, w - 8, PANEL_BORDER);

    // ── Process rows ──────────────────────────────────────────────────────────
    let row_h: u32 = 16;
    let body_top = sep_y + 2;
    let max_rows = if h > (body_top - (wy as u32)) + 20 {
        ((h - (body_top - wy as u32) - 20) / row_h) as usize
    } else { 0 };

    draw_rect(wx_u + 1, body_top, w - 2,
        (max_rows as u32 * row_h + 4).min(h.saturating_sub(body_top - wy as u32)),
        WIN_BG);

    let n = count.min(max_rows).min(slots.len());
    let mut row = 0usize;
    while row < n {
        let s = &slots[row];
        let ry = body_top + row as u32 * row_h;

        // Row highlight for Running
        let is_running = s.state_ch == b'R';
        if is_running {
            draw_rect(wx_u + 2, ry, w - 4, row_h - 1, 0x0A1A12);
        }

        // PID
        let pid_col = if is_running { ACCENT_TEAL } else { TEXT_DIM };
        draw_hex32(wx_u + 6, ry + 4, s.pid as u32, pid_col);

        // Name (clipped to column width)
        let name_col = if is_running { TEXT_WHITE } else { 0x9999BB };
        draw_str_clipped(wx_u + 28, ry + 4, s.name_str(), name_col, wx_u + 148);

        // State char
        let st_col = match s.state_ch {
            b'R' => ACCENT_TEAL,
            b'W' => ACCENT_AMBER,
            b'B' => 0x6666AA,
            b'D' => 0x664444,
            _    => TEXT_DIM,
        };
        let st_str: &str = match s.state_ch {
            b'R' => "R",
            b'W' => "W",
            b'B' => "B",
            b'D' => "D",
            _    => "?",
        };
        draw_str(wx_u + 148, ry + 4, st_str, st_col);

        // Priority
        draw_hex32(wx_u + 168, ry + 4, s.priority as u32, TEXT_DIM);

        // Ticks (low 32 bits)
        draw_hex32(wx_u + 196, ry + 4, s.ticks as u32, TEXT_DIM);

        // Yields
        draw_hex32(wx_u + 252, ry + 4, s.yields, TEXT_DIM);

        row += 1;
    }

    if count == 0 {
        draw_str(wx_u + 12, body_top + 6, "[no processes registered]", TEXT_DIM);
    }

    // ── Footer ────────────────────────────────────────────────────────────────
    let foot_y = (wy + h as i32 - 18) as u32;
    draw_hline(wx_u + 4, foot_y - 2, w - 8, PANEL_BORDER);
    draw_str(wx_u + 8,  foot_y + 2, "PL-62 cooperative scheduler", TEXT_DIM);
    draw_str(wx_u + w - 90, foot_y + 2, "R=run W=wait B=blk D=dead", 0x333355);

    // Resize handle
    draw_rect(wx_u + w - 12, (wy + h as i32 - 12) as u32, 12, 12, ACCENT_TEAL);
    draw_rect(wx_u + w - 8,  (wy + h as i32 - 8)  as u32, 4,  4,  TEXT_WHITE);
}

// ── PL-65: File Browser Window (kind=9) ──────────────────────────────────────
// Shows AXFS file listing with name, size, type columns.
// files: array of (name, size_bytes, kind) — kind: 0=.ax 1=.axpkg 2=.txt 3=other
pub struct FsEntry {
    pub name:      [u8; 32],
    pub name_len:  usize,
    pub size:      u32,
    pub kind:      u8, // 0=.ax 1=.axpkg 2=.txt 3=other
}

impl FsEntry {
    pub const fn empty() -> Self {
        FsEntry { name: [0u8; 32], name_len: 0, size: 0, kind: 3 }
    }
    pub fn name_str(&self) -> &str {
        core::str::from_utf8(&self.name[..self.name_len]).unwrap_or("?")
    }
}

pub fn render_file_browser(
    wx: i32, wy: i32, w: u32, h: u32,
    entries: &[FsEntry],
    count: usize,
    selected: usize,
    disk_used: u32,
    disk_total: u32,
) {
    let wx_u = wx as u32;
    let wy_u = wy as u32;

    // ── Column header ─────────────────────────────────────────────────────────
    let hdr_y = wy_u + WIN_TITLE_H + 2;
    draw_rect(wx_u + 1, hdr_y, w - 2, 14, 0x0A0A1A);
    draw_str(wx_u + 8,  hdr_y + 4, "NAME",      TEXT_DIM);
    draw_str(wx_u + 200, hdr_y + 4, "TYPE",     TEXT_DIM);
    draw_str(wx_u + 260, hdr_y + 4, "SIZE",     TEXT_DIM);
    let sep_y = hdr_y + 14;
    draw_hline(wx_u + 4, sep_y, w - 8, PANEL_BORDER);

    // ── File rows ─────────────────────────────────────────────────────────────
    let row_h: u32 = 16;
    let body_top = sep_y + 2;
    let avail_h = h.saturating_sub(body_top - wy_u).saturating_sub(40);
    let max_rows = (avail_h / row_h) as usize;
    draw_rect(wx_u + 1, body_top, w - 2,
        avail_h.min(max_rows as u32 * row_h + 4), WIN_BG);

    let n = count.min(max_rows).min(entries.len());
    let mut row = 0usize;
    while row < n {
        let e = &entries[row];
        let ry = body_top + row as u32 * row_h;
        let is_sel = row == selected;

        // Row highlight
        if is_sel {
            draw_rect(wx_u + 2, ry, w - 4, row_h - 1, 0x1A1A38);
            draw_hline(wx_u + 2, ry, w - 4, SOVEREIGN_PURPLE);
        }

        // Kind icon + colour
        let (type_str, name_col) = match e.kind {
            0 => (".ax",    ACCENT_TEAL),
            1 => (".axpkg", ACCENT_AMBER),
            2 => (".txt",   TEXT_WHITE),
            _ => ("file",   TEXT_DIM),
        };

        // Kind dot
        let dot_col = match e.kind {
            0 => ACCENT_TEAL,
            1 => ACCENT_AMBER,
            2 => 0x8888FF,
            _ => TEXT_DIM,
        };
        draw_rect(wx_u + 6, ry + 5, 5, 5, dot_col);

        // Name
        draw_str_clipped(wx_u + 16, ry + 4, e.name_str(), name_col, wx_u + 196);

        // Type
        draw_str(wx_u + 200, ry + 4, type_str, TEXT_DIM);

        // Size
        // Format bytes as "NNN B" or "N.N K"
        if e.size >= 1024 {
            let kb = e.size / 1024;
            let hex = [b'0' + (kb / 10) as u8, b'0' + (kb % 10) as u8, b' ', b'K', b'B'];
            if let Ok(s) = core::str::from_utf8(&hex) {
                draw_str(wx_u + 260, ry + 4, s, TEXT_DIM);
            }
        } else {
            let b1 = if e.size >= 100 { b'0' + (e.size / 100) as u8 } else { b' ' };
            let b2 = if e.size >= 10  { b'0' + ((e.size / 10) % 10) as u8 } else { b' ' };
            let b3 = b'0' + (e.size % 10) as u8;
            let sz = [b1, b2, b3, b' ', b'B'];
            if let Ok(s) = core::str::from_utf8(&sz) {
                draw_str(wx_u + 260, ry + 4, s, TEXT_DIM);
            }
        }
        row += 1;
    }

    if count == 0 {
        draw_str(wx_u + 16, body_top + 8, "[empty filesystem]", TEXT_DIM);
    }

    // ── Disk usage footer ─────────────────────────────────────────────────────
    let foot_y = wy_u + h - 32;
    draw_hline(wx_u + 4, foot_y, w - 8, PANEL_BORDER);
    // "Disk 1: NNN/NNN B"
    draw_str(wx_u + 8, foot_y + 6, "Disk 1", ACCENT_TEAL);
    draw_str(wx_u + 52, foot_y + 6, "sovereign AXFS", TEXT_DIM);
    // Usage bar
    let used_pct = if disk_total > 0 { disk_used * 100 / disk_total } else { 0 };
    let ubar_w = w.saturating_sub(20);
    draw_rect(wx_u + 8, foot_y + 18, ubar_w, 6, 0x22224A);
    draw_rect(wx_u + 8, foot_y + 18, ubar_w * used_pct / 100, 6, SOVEREIGN_PURPLE);
    // Pct label
    let p1 = b'0' + (used_pct / 10) as u8;
    let p2 = b'0' + (used_pct % 10) as u8;
    let pct_str = [p1, p2, b'%', b' ', b'u', b's', b'e', b'd'];
    if let Ok(s) = core::str::from_utf8(&pct_str) {
        draw_str(wx_u + w - 60, foot_y + 6, s, TEXT_DIM);
    }

    // Resize handle
    draw_rect(wx_u + w - 12, wy_u + h - 12, 12, 12, ACCENT_TEAL);
    draw_rect(wx_u + w - 8,  wy_u + h - 8,   4,  4, TEXT_WHITE);
}
