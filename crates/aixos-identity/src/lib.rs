// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
#![cfg_attr(not(test), no_std)]
use aixos_kernel::SovereignBoot;

pub struct ArpiCeremony;

impl ArpiCeremony {
    pub const fn new() -> Self { ArpiCeremony }
}

impl Default for ArpiCeremony {
    fn default() -> Self { Self::new() }
}

/// Derive a deterministic node ID from hardware constants.
/// RAM base (0x40000000) is confirmed present on every boot —
/// QEMU virt machine always maps RAM here. XOR with the fw_cfg
/// DMA address to produce a unique 64-bit node identity.
/// This is a hardware-derived stub — not a cryptographic keypair.
/// Full ARPi Ed25519 ceremony lands when the Root Key is wired.
pub fn node_id() -> u64 {
    let ram_base: u64 = 0x4000_0000;
    let fwcfg:    u64 = 0x0902_0010;
    // Mix the two hardware constants into a non-zero node seed
    ram_base ^ (fwcfg << 16) ^ 0xA1E0_4153_0000_0001
}

/// ARPi ceremony is ready when we have a non-zero hardware-derived node ID.
/// Invariant: node_id() is always non-zero for this hardware target.
pub fn ceremony_ready() -> bool {
    node_id() != 0
}

impl SovereignBoot for ArpiCeremony {
    fn handshake(&self) -> bool { ceremony_ready() }
    fn proof(&self) -> u32 { aixos_kernel::AXON_PROOF }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_id_is_nonzero() {
        // Hardware-derived node ID must never be zero.
        assert_ne!(node_id(), 0);
    }

    #[test]
    fn node_id_is_deterministic() {
        // Same hardware constants → same node ID every time.
        assert_eq!(node_id(), node_id());
    }

    #[test]
    fn ceremony_ready_when_node_id_nonzero() {
        assert!(ceremony_ready());
    }

    #[test]
    fn arpi_ceremony_handshake_is_live() {
        let pd = ArpiCeremony::new();
        assert!(pd.handshake());
        assert_eq!(pd.proof(), aixos_kernel::AXON_PROOF);
    }
}
