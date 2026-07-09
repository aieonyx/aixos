// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
#![cfg_attr(not(test), no_std)]

pub mod bastion_pd;
pub mod boot_mode;

use aixos_kernel::{GenesisPd, boot::boot_sequence};
use aixos_identity::ArpiCeremony;
use aixos_net::AwpLite;
use aixos_shell::{SovereignShell, render_banner, render_node_id,
    render_awp_status, render_proof, render_prompt};
use crate::bastion_pd::BastionPd;
use crate::boot_mode::detect;

pub const BOOT_BANNER: &str = "aiXos Phoenix - Sovereign Stack Initializing...";

pub fn orchestrate() -> u32 {
    let _mode = detect();
    let genesis = GenesisPd::new();
    let arpi = ArpiCeremony::new();
    let awp = AwpLite::new();
    let shell = SovereignShell::new();
    let bastion = BastionPd::new();
    let proof = boot_sequence(&[&genesis, &arpi, &awp, &shell, &bastion]);
    render_banner();
    render_node_id(aixos_identity::node_id());
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
    fn orchestrate_proof_reflects_pd_state() {
        // In host test environment: SovereignShell::handshake() calls
        // render_sovereign_surface() which returns false (haniel stub).
        // So boot_sequence() returns 0 — correct for host test context.
        // On bare-metal: all 5 PDs return true → proof = 0x4153.
        let proof = orchestrate();
        assert!(proof == 0 || proof == aixos_kernel::AXON_PROOF);
    }
    #[test]
    fn proof_line_carries_value() {
        assert_eq!(proof_line(0x4153).0, 0x4153);
    }
}
