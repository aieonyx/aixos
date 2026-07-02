// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
#![cfg_attr(not(test), no_std)]

pub struct PixelSurface { pub width: u32, pub height: u32 }
pub struct RenderRequest { pub surface: PixelSurface, pub frame_id: u64 }
pub struct HanielPd;

impl HanielPd {
    pub const fn new() -> Self { HanielPd }
    pub fn submit(&self, _request: RenderRequest) -> bool { false }
}
impl Default for HanielPd { fn default() -> Self { Self::new() } }

pub fn render_text(_text: &str) -> bool { false }
pub fn render_sovereign_surface() -> bool { false }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn submit_returns_false_before_pipeline_wiring() {
        let pd = HanielPd::new();
        let req = RenderRequest {
            surface: PixelSurface { width: 1280, height: 720 },
            frame_id: 0
        };
        assert!(!pd.submit(req));
    }
    #[test]
    fn render_text_stub_returns_false() { assert!(!render_text("axos> ")); }
    #[test]
    fn render_surface_stub_returns_false() { assert!(!render_sovereign_surface()); }
}
