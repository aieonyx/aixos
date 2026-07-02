// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
pub enum BootMode { Live, Persistent, Install }

pub fn detect() -> BootMode { BootMode::Live }

pub fn is_persistent(mode: &BootMode) -> bool {
    matches!(mode, BootMode::Persistent)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn detect_defaults_to_live() {
        assert!(matches!(detect(), BootMode::Live));
    }
    #[test]
    fn live_is_not_persistent() {
        assert!(!is_persistent(&BootMode::Live));
    }
}
