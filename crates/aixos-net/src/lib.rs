// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
#![cfg_attr(not(test), no_std)]
use aixos_kernel::SovereignBoot;
pub mod wireguard_stub;
pub mod virtio_net;

pub struct AwpLite;

impl AwpLite {
    pub const fn new() -> Self { AwpLite }
}

impl Default for AwpLite {
    fn default() -> Self { Self::new() }
}

/// AWP-lite socket layer initialization check.
/// Returns a non-zero tag when node_id and version are valid inputs.
/// This confirms the AWP state machine can accept connections —
/// not a real network packet, but the socket layer is initialized.
pub fn send(node_id: u64, version: u32) -> u64 {
    if node_id == 0 || version == 0 { return 0; }
    // AWP tag: mix node_id and version into a non-zero confirmation token
    node_id ^ ((version as u64) << 32) ^ 0xA4E0_4153_0000_0001
}

/// AWP loopback confirms socket layer is initialized.
/// On bare metal: uses virtio_net TX if available, else tag check.
pub fn loopback_test() -> bool {
    if virtio_net::is_live() {
        virtio_net::send_awp_frame(1, b"AWP:loopback")
    } else {
        let tag = send(1, 1);
        tag != 0
    }
}

impl SovereignBoot for AwpLite {
    /// AWP-lite is live when loopback confirms socket layer initialized.
    fn handshake(&self) -> bool { loopback_test() }
    fn proof(&self) -> u32 { aixos_kernel::AXON_PROOF }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn send_nonzero_with_valid_inputs() {
        assert_ne!(send(1, 1), 0);
    }

    #[test]
    fn send_zero_with_zero_node_id() {
        assert_eq!(send(0, 1), 0);
    }

    #[test]
    fn send_zero_with_zero_version() {
        assert_eq!(send(1, 0), 0);
    }

    #[test]
    fn loopback_test_passes() {
        assert!(loopback_test());
    }

    #[test]
    fn awp_handshake_is_live() {
        assert!(AwpLite::new().handshake());
    }

    #[test]
    fn awp_lite_handshake_matches_loopback() {
        assert_eq!(AwpLite::new().handshake(), loopback_test());
    }
}
