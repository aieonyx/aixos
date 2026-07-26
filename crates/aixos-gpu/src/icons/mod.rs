// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// PL-65A: Sovereign icon set — 7 x 32x32 ARGB bitmaps

pub mod aieonyx;
pub mod globe;
pub mod terminal;
pub mod folder;
pub mod disk;
pub mod gear;
pub mod antenna;

pub const ICON_W: u32 = 32;
pub const ICON_H: u32 = 32;

/// Blit icon by index:
/// 0=aieonyx 1=globe 2=terminal 3=folder 4=disk 5=gear 6=antenna
pub fn blit_icon(idx: u8, x: u32, y: u32) {
    match idx {
        0 => aieonyx::blit(x, y),
        1 => globe::blit(x, y),
        2 => terminal::blit(x, y),
        3 => folder::blit(x, y),
        4 => disk::blit(x, y),
        5 => gear::blit(x, y),
        6 => antenna::blit(x, y),
        _ => {}
    }
}
