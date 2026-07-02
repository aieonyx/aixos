// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
#![cfg_attr(not(test), no_std)]

use aixos_haniel::{render_sovereign_surface, render_text};
use aixos_kernel::SovereignBoot;

const BOOT_BANNER: &str = "aiXos Phoenix - Sovereign Stack Initializing...";

pub struct SovereignShell;
impl SovereignShell { pub const fn new() -> Self { SovereignShell } }
impl Default for SovereignShell { fn default() -> Self { Self::new() } }

pub fn render_banner() -> bool {
    render_sovereign_surface();
    render_text(BOOT_BANNER)
}
pub fn render_node_id(id: u64) -> bool { let _ = id; render_text("node_id") }
pub fn render_awp_status(live: bool) -> bool {
    let label = if live { "awp: live" } else { "awp: down" };
    render_text(label)
}
pub fn render_proof(proof: u32) -> bool {
    let _ = proof;
    render_text("axon_main() -> 0x4153")
}
pub fn render_prompt() -> bool { render_text("axos> ") }

impl SovereignBoot for SovereignShell {
    fn handshake(&self) -> bool { render_sovereign_surface() }
    fn proof(&self) -> u32 { 0 }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shell_not_live_before_haniel_wiring() {
        assert!(!SovereignShell::new().handshake());
    }
    #[test]
    fn awp_status_stub_always_false() {
        assert!(!render_awp_status(false));
        assert!(!render_awp_status(true));
    }
}
