// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0

pub const WIDTH: usize = 1280;
pub const HEIGHT: usize = 720;

#[link_section = ".fb"]
pub static mut FRAMEBUFFER: [u32; WIDTH * HEIGHT] = [0; WIDTH * HEIGHT];

pub fn fb_addr() -> u64 {
    core::ptr::addr_of!(FRAMEBUFFER) as u64
}

pub fn fb_size() -> u32 { (WIDTH * HEIGHT * 4) as u32 }

pub fn cache_flush() {
    unsafe {
        { #[cfg(target_arch = "aarch64")] { core::arch::asm!("dsb sy", options(nostack, nomem)); } }
    }
}

pub fn fill(color: u32) {
    unsafe {
        let ptr = core::ptr::addr_of_mut!(FRAMEBUFFER) as *mut u32;
        let mut i = 0;
        while i < WIDTH * HEIGHT {
            core::ptr::write_volatile(ptr.add(i), color);
            i += 1;
        }
    }
}

pub fn fill_rect(x: u32, y: u32, w: u32, h: u32, color: u32) {
    unsafe {
        let ptr = core::ptr::addr_of_mut!(FRAMEBUFFER) as *mut u32;
        let mut row = 0;
        while row < h {
            let py = y + row;
            if (py as usize) < HEIGHT {
                let mut col = 0;
                while col < w {
                    let px = x + col;
                    if (px as usize) < WIDTH {
                        let offset = (py as usize) * WIDTH + (px as usize);
                        core::ptr::write_volatile(ptr.add(offset), color);
                    }
                    col += 1;
                }
            }
            row += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fb_size_matches_1280x720_argb() { assert_eq!(fb_size(), 1280 * 720 * 4); }
    #[test]
    fn fb_addr_is_nonzero() { assert_ne!(fb_addr(), 0); }
    #[test]
    fn fill_rect_out_of_bounds_does_not_panic() {
        fill_rect(1275, 715, 100, 100, 0xFFFFFFFF);
    }
}
