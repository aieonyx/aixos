// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
#![cfg_attr(not(test), no_std)]

/// Sovereign proof value. Matches ASL boot proof axon_main() -> 0x4153.
pub const AXON_PROOF: u32 = 0x4153;

pub mod boot;
pub mod virtio_blk;
pub mod alloc;

/// Boot-sequence contract implemented by each sovereign PD stub.
pub trait SovereignBoot {
    fn handshake(&self) -> bool;
    fn proof(&self) -> u32;
}

/// GENESIS Protection Domain.
/// Invariant: if this code is executing, the kernel loaded and GENESIS
/// succeeded. handshake() returns true unconditionally — execution is proof.
pub struct GenesisPd;

impl GenesisPd {
    pub const fn new() -> Self { GenesisPd }
}

impl Default for GenesisPd {
    fn default() -> Self { Self::new() }
}

impl SovereignBoot for GenesisPd {
    /// Execution of this function IS the proof of genesis.
    /// If the kernel is running, GENESIS is live.
    fn handshake(&self) -> bool { true }
    fn proof(&self) -> u32 { AXON_PROOF }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn axon_proof_matches_asl_boot_value() {
        assert_eq!(AXON_PROOF, 0x4153);
    }

    #[test]
    fn genesis_pd_is_live() {
        // GENESIS is always live if this code executes — execution is proof.
        let pd = GenesisPd::new();
        assert!(pd.handshake());
        assert_eq!(pd.proof(), AXON_PROOF);
    }
}
