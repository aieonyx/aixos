// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
use crate::{GenesisPd, SovereignBoot, AXON_PROOF};

pub struct IpcChannel(pub u8);

pub const GENESIS_CHANNEL: IpcChannel = IpcChannel(0);
pub const ARPI_CHANNEL: IpcChannel = IpcChannel(1);
pub const EDISONDB_CHANNEL: IpcChannel = IpcChannel(2);
pub const HANIEL_CHANNEL: IpcChannel = IpcChannel(3);

struct ArpiStage;
impl SovereignBoot for ArpiStage {
    fn handshake(&self) -> bool { false }
    fn proof(&self) -> u32 { 0 }
}

struct EdisonDbStage;
impl SovereignBoot for EdisonDbStage {
    fn handshake(&self) -> bool { false }
    fn proof(&self) -> u32 { 0 }
}

struct HanielStage;
impl SovereignBoot for HanielStage {
    fn handshake(&self) -> bool { false }
    fn proof(&self) -> u32 { 0 }
}

pub fn boot_sequence() -> u32 {
    let genesis = GenesisPd::new();
    let arpi = ArpiStage;
    let edisondb = EdisonDbStage;
    let haniel = HanielStage;
    let all_live = genesis.handshake()
        && arpi.handshake()
        && edisondb.handshake()
        && haniel.handshake();
    if all_live { AXON_PROOF } else { 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boot_sequence_returns_zero_before_any_pd_is_live() {
        assert_eq!(boot_sequence(), 0);
    }

    #[test]
    fn ipc_channels_are_distinct() {
        assert_ne!(GENESIS_CHANNEL.0, ARPI_CHANNEL.0);
        assert_ne!(ARPI_CHANNEL.0, EDISONDB_CHANNEL.0);
        assert_ne!(EDISONDB_CHANNEL.0, HANIEL_CHANNEL.0);
    }
}
