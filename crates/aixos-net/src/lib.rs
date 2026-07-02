// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
#![cfg_attr(not(test), no_std)]

use aixos_kernel::SovereignBoot;

pub mod wireguard_stub;

pub struct AwpLite;

impl AwpLite {
    pub const fn new() -> Self { AwpLite }
}

impl Default for AwpLite {
    fn default() -> Self { Self::new() }
}

pub fn send(node_id: u64, version: u32) -> u64 {
    let _ = (node_id, version);
    0
}

pub fn loopback_test() -> bool {
    let tag = send(0, 1);
    tag != 0
}

impl SovereignBoot for AwpLite {
    fn handshake(&self) -> bool { loopback_test() }
    fn proof(&self) -> u32 { 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn send_is_stub_zero() { assert_eq!(send(1, 1), 0); }

    #[test]
    fn loopback_fails_before_wiring() { assert!(!loopback_test()); }

    #[test]
    fn awp_lite_handshake_matches_loopback() {
        assert!(!AwpLite::new().handshake());
    }
}
