// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
#![cfg_attr(not(test), no_std)]

pub mod virtio_gpu;

pub const SOVEREIGN_PURPLE: u32 = 0xFF7B4FDB;

pub struct VirtioGpu { pub regs: virtio_gpu::VirtioGpuRegs }

pub fn init() -> Option<VirtioGpu> {
    #[cfg(not(test))]
    return virtio_gpu::probe().map(|regs| VirtioGpu { regs });
    #[cfg(test)]
    return None;
}

pub fn set_scanout(_width: u32, _height: u32) {}
pub fn flush(_x: u32, _y: u32, _w: u32, _h: u32) {}
pub fn fill_rect(_x: u32, _y: u32, _w: u32, _h: u32, _color: u32) {}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn init_returns_none_before_mmio_wiring() {
        assert!(init().is_none());
    }
    #[test]
    fn sovereign_purple_matches_brand_value() {
        assert_eq!(SOVEREIGN_PURPLE, 0xFF7B4FDB);
    }
}
