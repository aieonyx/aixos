// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0

use crate::framebuffer::{self, WIDTH, HEIGHT};

pub fn draw_rect(x: u32, y: u32, w: u32, h: u32, color: u32) {
    framebuffer::fill_rect(x, y, w, h, color);
}

pub fn draw_hline(x: u32, y: u32, len: u32, color: u32) {
    if y as usize >= HEIGHT { return; }
    framebuffer::fill_rect(x, y, len.min((WIDTH as u32).saturating_sub(x)), 1, color);
}

pub fn draw_vline(x: u32, y: u32, len: u32, color: u32) {
    if x as usize >= WIDTH { return; }
    framebuffer::fill_rect(x, y, 1, len.min((HEIGHT as u32).saturating_sub(y)), color);
}

pub fn draw_border(x: u32, y: u32, w: u32, h: u32, color: u32) {
    draw_hline(x, y, w, color);
    draw_hline(x, y + h.saturating_sub(1), w, color);
    draw_vline(x, y, h, color);
    draw_vline(x + w.saturating_sub(1), y, h, color);
}

pub fn blend_rect(x: u32, y: u32, w: u32, h: u32, fg: u32, alpha: u8) {
    let fx = x as usize;
    let fy = y as usize;
    let fw = (w as usize).min(WIDTH.saturating_sub(fx));
    let fh = (h as usize).min(HEIGHT.saturating_sub(fy));
    let a = alpha as u32;
    let fg_r = (fg >> 16) & 0xFF;
    let fg_g = (fg >> 8)  & 0xFF;
    let fg_b =  fg        & 0xFF;
    unsafe {
        let ptr = core::ptr::addr_of_mut!(framebuffer::FRAMEBUFFER) as *mut u32;
        let mut row = 0usize;
        while row < fh {
            let mut col = 0usize;
            while col < fw {
                let off = (fy + row) * WIDTH + (fx + col);
                let bg = core::ptr::read_volatile(ptr.add(off));
                let bg_r = (bg >> 16) & 0xFF;
                let bg_g = (bg >> 8)  & 0xFF;
                let bg_b =  bg        & 0xFF;
                let r = (fg_r * a + bg_r * (255 - a)) / 255;
                let g = (fg_g * a + bg_g * (255 - a)) / 255;
                let b = (fg_b * a + bg_b * (255 - a)) / 255;
                core::ptr::write_volatile(ptr.add(off), (r << 16) | (g << 8) | b);
                col += 1;
            }
            row += 1;
        }
    }
}
