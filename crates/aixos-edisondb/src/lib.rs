// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// sovereign in-memory tiered store (bare-metal EdisonDB bridge)
#![cfg_attr(not(test), no_std)]

const MAX_ENTRIES: usize = 32;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tier {
    Critical,
    Personal,
    Noise,
}

#[derive(Clone, Copy)]
struct Entry {
    key: &'static str,
    value: u64,
    tier: Tier,
}

static mut STORE: [Option<Entry>; MAX_ENTRIES] = [None; MAX_ENTRIES];
static mut STORE_LEN: usize = 0;
static mut INITIALIZED: bool = false;

pub fn init() {
    unsafe {
        if INITIALIZED {
            return;
        }
        INITIALIZED = true;
    }
    write("boot:proof", 0x4153, Tier::Critical);
}

pub fn write(key: &'static str, value: u64, tier: Tier) -> bool {
    unsafe {
        let len = STORE_LEN;
        for i in 0..len {
            if let Some(ref mut e) = STORE[i] {
                if e.key == key {
                    e.value = value;
                    e.tier = tier;
                    return true;
                }
            }
        }
        if len >= MAX_ENTRIES {
            return false;
        }
        STORE[len] = Some(Entry { key, value, tier });
        STORE_LEN = len + 1;
        true
    }
}

pub fn read(key: &'static str) -> Option<u64> {
    unsafe {
        for i in 0..STORE_LEN {
            if let Some(ref e) = STORE[i] {
                if e.key == key {
                    return Some(e.value);
                }
            }
        }
        None
    }
}

pub fn count() -> usize {
    unsafe { STORE_LEN }
}

pub fn is_live() -> bool {
    unsafe { INITIALIZED }
}

pub fn log_event(event: &'static str) -> bool {
    write(event, count() as u64, Tier::Noise)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_is_live_and_seeds_proof() {
        init();
        assert!(is_live());
        assert_eq!(read("boot:proof"), Some(0x4153));
    }

    #[test]
    fn write_appends_and_read_finds() {
        let before = count();
        assert!(write("test:alpha", 7, Tier::Personal));
        assert_eq!(count(), before + 1);
        assert_eq!(read("test:alpha"), Some(7));
    }

    #[test]
    fn write_upserts_in_place() {
        assert!(write("test:upsert", 1, Tier::Noise));
        let after_first = count();
        assert!(write("test:upsert", 2, Tier::Critical));
        assert_eq!(count(), after_first);
        assert_eq!(read("test:upsert"), Some(2));
    }

    #[test]
    fn read_missing_returns_none() {
        assert_eq!(read("test:missing"), None);
    }

    #[test]
    fn log_event_is_idempotent_on_space() {
        assert!(log_event("test:event"));
        let after_first = count();
        assert!(log_event("test:event"));
        assert_eq!(count(), after_first);
    }
}
