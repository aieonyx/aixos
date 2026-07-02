// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
#![cfg_attr(not(test), no_std)]

/// Sovereign proof value. Matches ASL boot proof axon_main() -> 0x4153.
pub const AXON_PROOF: u32 = 0x4153;

pub mod boot;

/// Boot-sequence contract implemented by each sovereign PD stub.
pub trait SovereignBoot {
    fn handshake(&self) -> bool;
    fn proof(&self) -> u32;
}

/// GENESIS PD stub. Real wiring lands once asl is a path dependency.
pub struct GenesisPd;

impl GenesisPd {
    pub const fn new() -> Self { GenesisPd }
}

impl Default for GenesisPd {
    fn default() -> Self { Self::new() }
}

impl SovereignBoot for GenesisPd {
    fn handshake(&self) -> bool { false }
    fn proof(&self) -> u32 { 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn axon_proof_matches_asl_boot_value() {
        assert_eq!(AXON_PROOF, 0x4153);
    }

    #[test]
    fn genesis_pd_stub_not_yet_live() {
        let pd = GenesisPd::new();
        assert!(!pd.handshake());
        assert_eq!(pd.proof(), 0);
    }
}
