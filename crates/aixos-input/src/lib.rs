// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
#![cfg_attr(not(test), no_std)]

pub mod virtio_input;

pub const KEY_ESC: u16 = 1;
pub const KEY_BACKSPACE: u16 = 14;
pub const KEY_ENTER: u16 = 28;
pub const KEY_A: u16 = 30;
pub const KEY_B: u16 = 48;
pub const KEY_C: u16 = 46;
pub const KEY_D: u16 = 32;
pub const KEY_E: u16 = 18;
pub const KEY_S: u16 = 31;
pub const KEY_Z: u16 = 44;

pub struct KeyEvent { pub code: u16, pub value: u32 }
pub struct VirtioInput;

pub fn init() -> Option<VirtioInput> { None }
pub fn poll() -> Option<KeyEvent> { None }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn init_returns_none_before_mmio_wiring() { assert!(init().is_none()); }
    #[test]
    fn poll_returns_none_before_queue_wiring() { assert!(poll().is_none()); }
    #[test]
    fn key_a_is_correct_evdev_code() { assert_eq!(KEY_A, 30); }
    #[test]
    fn key_enter_is_correct_evdev_code() { assert_eq!(KEY_ENTER, 28); }
}
