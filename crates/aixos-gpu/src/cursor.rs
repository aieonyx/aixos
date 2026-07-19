// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// framebuffer cursor: 12x20 arrow, save-under compositing

use core::ptr::{addr_of, addr_of_mut};

pub const CURSOR_W: usize = 12;
pub const CURSOR_H: usize = 20;

const WHITE: u32 = 0xFFFF_FFFF;
const SCREEN_W: i32 = 1280;
const SCREEN_H: i32 = 720;

// 1bpp rows, MSB-first in the top 12 bits of each u16
static ARROW: [u16; CURSOR_H] = [
    0x8000, 0xC000, 0xE000, 0xF000, 0xF800,
    0xFC00, 0xFE00, 0xFF00, 0xFF80, 0xFFC0,
    0xFFE0, 0xFE00, 0xEE00, 0xC700, 0x8700,
    0x0380, 0x0380, 0x0180, 0x0000, 0x0000,
];

static mut SAVED: [u32; CURSOR_W * CURSOR_H] = [0; CURSOR_W * CURSOR_H];

// Saves the full 12x20 background region, then paints opaque
// pixels. Caller must erase_cursor at the OLD position before
// drawing at a new one; erase must use the same x,y as the
// matching draw.
pub fn draw_cursor(fb: &mut [u32], stride: usize, x: i32, y: i32) {
    let saved = unsafe { &mut *addr_of_mut!(SAVED) };
    for row in 0..CURSOR_H {
        for col in 0..CURSOR_W {
            let px = x + col as i32;
            let py = y + row as i32;
            if px < 0 || px >= SCREEN_W || py < 0 || py >= SCREEN_H {
                continue;
            }
            let idx = py as usize * stride + px as usize;
            if idx >= fb.len() {
                continue;
            }
            saved[row * CURSOR_W + col] = fb[idx];
            if (ARROW[row] >> (15 - col)) & 1 == 1 {
                fb[idx] = WHITE;
            }
        }
    }
}

// Restores the saved background region captured by the last
// draw_cursor call. Same clipping as draw, so the restored
// region matches exactly.
pub fn erase_cursor(fb: &mut [u32], stride: usize, x: i32, y: i32) {
    let saved = unsafe { &*addr_of!(SAVED) };
    for row in 0..CURSOR_H {
        for col in 0..CURSOR_W {
            let px = x + col as i32;
            let py = y + row as i32;
            if px < 0 || px >= SCREEN_W || py < 0 || py >= SCREEN_H {
                continue;
            }
            let idx = py as usize * stride + px as usize;
            if idx >= fb.len() {
                continue;
            }
            fb[idx] = saved[row * CURSOR_W + col];
        }
    }
}
