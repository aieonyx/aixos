// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
#![cfg_attr(not(test), no_std)]

pub struct StorageRequest { pub key: &'static str, pub tier: StorageTier }
pub enum StorageTier { Critical, Personal, Noise }
pub struct EqlQuery { pub text: &'static str }
pub struct EdisonDbPd { wal_ready: bool, mvcc_ready: bool }

impl EdisonDbPd {
    pub const fn new() -> Self { EdisonDbPd { wal_ready: false, mvcc_ready: false } }
    pub fn write(&self, _request: StorageRequest) -> bool { self.wal_ready }
    pub fn query(&self, _query: EqlQuery) -> bool { self.mvcc_ready }
}
impl Default for EdisonDbPd { fn default() -> Self { Self::new() } }

pub fn log_event(_event: &'static str) -> bool { false }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn write_and_query_return_false_before_wiring() {
        let pd = EdisonDbPd::new();
        assert!(!pd.write(StorageRequest { key: "k", tier: StorageTier::Personal }));
        assert!(!pd.query(EqlQuery { text: "SELECT * FROM test" }));
    }
    #[test]
    fn log_event_stub_returns_false() { assert!(!log_event("bastion:boot")); }
}
