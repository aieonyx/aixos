// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
#![cfg_attr(not(test), no_std)]

use aixos_kernel::{GenesisPd, boot::boot_sequence};
use aixos_identity::ArpiCeremony;

pub const BOOT_BANNER: &str = "aiXos Phoenix - Sovereign Stack Initializing...";

pub fn orchestrate() -> u32 {
    let genesis = GenesisPd::new();
    let arpi = ArpiCeremony::new();
    boot_sequence(&[&genesis, &arpi])
}

pub struct ProofLine(pub u32);

pub fn proof_line(proof: u32) -> ProofLine {
    ProofLine(proof)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boot_banner_is_set() {
        assert!(!BOOT_BANNER.is_empty());
    }

    #[test]
    fn orchestrate_returns_zero_before_pds_live() {
        assert_eq!(orchestrate(), 0);
    }

    #[test]
    fn proof_line_carries_value() {
        assert_eq!(proof_line(0x4153).0, 0x4153);
    }
}
