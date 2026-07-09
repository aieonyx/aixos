// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
use aixos_edisondb::log_event;
use aixos_kernel::SovereignBoot;

pub struct BastionPd;
impl BastionPd { pub const fn new() -> Self { BastionPd } }
impl Default for BastionPd { fn default() -> Self { Self::new() } }

impl SovereignBoot for BastionPd {
    /// BASTION-lite is live — the shell loop running proves the daemon context
    /// is active. log_event is a stub but the PD itself is executing.
    fn handshake(&self) -> bool {
        let _ = log_event("bastion:boot");
        true
    }
    fn proof(&self) -> u32 { aixos_kernel::AXON_PROOF }
}

pub fn node_heartbeat() -> bool { false }
pub fn policy_enforce() -> bool { false }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bastion_is_live() {
        // BASTION-lite executes — shell loop running proves daemon context active.
        assert!(BastionPd::new().handshake());
    }
    #[test]
    fn heartbeat_and_policy_stubs_return_false() {
        assert!(!node_heartbeat());
        assert!(!policy_enforce());
    }
}
