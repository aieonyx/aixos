// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0

use crate::{SovereignBoot, AXON_PROOF};

pub struct IpcChannel(pub u8);

pub const GENESIS_CHANNEL: IpcChannel = IpcChannel(0);
pub const ARPI_CHANNEL: IpcChannel = IpcChannel(1);
pub const EDISONDB_CHANNEL: IpcChannel = IpcChannel(2);
pub const HANIEL_CHANNEL: IpcChannel = IpcChannel(3);

pub fn boot_sequence(stages: &[&dyn SovereignBoot]) -> u32 {
    let mut all_live = true;
    for stage in stages {
        if !stage.handshake() {
            all_live = false;
        }
    }
    if all_live { AXON_PROOF } else { 0 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GenesisPd;

    struct AlwaysLive;
    impl SovereignBoot for AlwaysLive {
        fn handshake(&self) -> bool { true }
        fn proof(&self) -> u32 { AXON_PROOF }
    }

    #[test]
    fn proof_with_genesis_only() {
        // GenesisPd::handshake() is true — execution is proof.
        // A single live stage returns AXON_PROOF.
        let genesis = GenesisPd::new();
        assert_eq!(boot_sequence(&[&genesis]), AXON_PROOF);
    }

    #[test]
    fn proof_when_all_stages_live() {
        let live = AlwaysLive;
        assert_eq!(boot_sequence(&[&live]), AXON_PROOF);
    }

    #[test]
    fn ipc_channels_are_distinct() {
        assert_ne!(GENESIS_CHANNEL.0, ARPI_CHANNEL.0);
    }
}
