// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
#![cfg_attr(not(test), no_std)]

/// A pixel surface handed to CANVAS for presentation.
pub struct PixelSurface {
    pub width: u32,
    pub height: u32,
}

/// A single render request submitted to the HANIEL PD.
pub struct RenderRequest {
    pub surface: PixelSurface,
    pub frame_id: u64,
}

/// HANIEL Protection Domain interface.
pub struct HanielPd;

impl HanielPd {
    pub const fn new() -> Self {
        HanielPd
    }

    pub fn submit(&self, _request: RenderRequest) -> bool {
        false
    }
}

impl Default for HanielPd {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn submit_returns_false_before_pipeline_wiring() {
        let pd = HanielPd::new();
        let request = RenderRequest {
            surface: PixelSurface { width: 1280, height: 720 },
            frame_id: 0,
        };
        assert!(!pd.submit(request));
    }
}
