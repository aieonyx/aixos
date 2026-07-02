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

pub fn ceremony_ready() -> bool { false }

pub fn node_id() -> u64 { 0 }

impl SovereignBoot for ArpiCeremony {
    fn handshake(&self) -> bool { ceremony_ready() }
    fn proof(&self) -> u32 { 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ceremony_not_ready_before_ax_wiring() {
        assert!(!ceremony_ready());
    }

    #[test]
    fn node_id_stub_is_zero() {
        assert_eq!(node_id(), 0);
    }

    #[test]
    fn arpi_ceremony_handshake_matches_ready_state() {
        let pd = ArpiCeremony::new();
        assert!(!pd.handshake());
        assert_eq!(pd.proof(), 0);
    }
}
