// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
#![cfg_attr(not(test), no_std)]

pub mod bastion_pd;

use aixos_kernel::{GenesisPd, boot::boot_sequence};
use aixos_identity::ArpiCeremony;
use aixos_net::AwpLite;
use aixos_shell::{SovereignShell, render_banner, render_node_id,
    render_awp_status, render_proof, render_prompt};
use crate::bastion_pd::BastionPd;

pub const BOOT_BANNER: &str = "aiXos Phoenix - Sovereign Stack Initializing...";

pub fn orchestrate() -> u32 {
    let genesis = GenesisPd::new();
    let arpi = ArpiCeremony::new();
    let awp = AwpLite::new();
    let shell = SovereignShell::new();
    let bastion = BastionPd::new();
    let proof = boot_sequence(&[&genesis, &arpi, &awp, &shell, &bastion]);
    render_banner();
    render_node_id(0);
    render_awp_status(false);
    render_proof(proof);
    render_prompt();
    proof
}

pub struct ProofLine(pub u32);
pub fn proof_line(proof: u32) -> ProofLine { ProofLine(proof) }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn boot_banner_is_set() { assert!(!BOOT_BANNER.is_empty()); }
    #[test]
    fn orchestrate_returns_zero_before_pds_live() {
        assert_eq!(orchestrate(), 0);
    }
    #[test]
    fn proof_line_carries_value() {
        assert_eq!(proof_line(0x4153).0, 0x4153);
    }
}
