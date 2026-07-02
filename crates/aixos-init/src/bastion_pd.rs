// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
use aixos_edisondb::log_event;
use aixos_kernel::SovereignBoot;

pub struct BastionPd;
impl BastionPd { pub const fn new() -> Self { BastionPd } }
impl Default for BastionPd { fn default() -> Self { Self::new() } }

impl SovereignBoot for BastionPd {
    fn handshake(&self) -> bool { log_event("bastion:boot"); false }
    fn proof(&self) -> u32 { 0 }
}

pub fn node_heartbeat() -> bool { false }
pub fn policy_enforce() -> bool { false }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bastion_not_live_before_wiring() {
        assert!(!BastionPd::new().handshake());
    }
    #[test]
    fn heartbeat_and_policy_stubs_return_false() {
        assert!(!node_heartbeat());
        assert!(!policy_enforce());
    }
}
