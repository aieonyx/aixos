// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0

#![cfg_attr(not(test), no_std)]

pub mod virtio_input;
pub mod uart_kbd;

pub use virtio_input::{VirtioInputEvent, EV_KEY, EV_VALUE_PRESS, EV_VALUE_REPEAT};

pub struct VirtioInput { base: usize }

impl VirtioInput {
    pub fn base(&self) -> usize { self.base }
    pub fn poll(&self) -> Option<KeyEvent> { poll() }
}

pub struct KeyEvent {
    pub code: u16,
    pub value: u32,
    pub ch: Option<char>,
}

pub fn init() -> Option<VirtioInput> {
    let (base, ver) = virtio_input::probe()?;
    if virtio_input::init_device(base, ver) { Some(VirtioInput { base }) } else { None }
}

pub fn poll() -> Option<KeyEvent> {
    // Try UART keyboard first (works with -serial stdio)
    if let Some(b) = uart_kbd::read_byte() {
        let ch = b as char;
        let code: u16 = match b {
            b'\r' | b'\n' => 28, // ENTER
            0x1b => 1,            // ESC
            0x7f | 0x08 => 14,   // BACKSPACE
            _ => 0,
        };
        return Some(KeyEvent {
            code,
            value: 1,
            ch: if b >= 0x20 && b < 0x7f { Some(ch) } else { None },
        });
    }
    // Fall back to virtio-input
    let ev = virtio_input::poll_event()?;
    Some(KeyEvent { code: ev.code, value: ev.value, ch: evdev_to_char(ev.code) })
}

pub fn evdev_to_char(code: u16) -> Option<char> {
    Some(match code {
        2=>'1', 3=>'2', 4=>'3', 5=>'4', 6=>'5',
        7=>'6', 8=>'7', 9=>'8', 10=>'9', 11=>'0',
        16=>'q', 17=>'w', 18=>'e', 19=>'r', 20=>'t',
        21=>'y', 22=>'u', 23=>'i', 24=>'o', 25=>'p',
        30=>'a', 31=>'s', 32=>'d', 33=>'f', 34=>'g',
        35=>'h', 36=>'j', 37=>'k', 38=>'l',
        44=>'z', 45=>'x', 46=>'c', 47=>'v', 48=>'b',
        49=>'n', 50=>'m', 57=>' ',
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn digits_map() {
        assert_eq!(evdev_to_char(2), Some('1'));
        assert_eq!(evdev_to_char(11), Some('0'));
    }
    #[test]
    fn letters_map() {
        assert_eq!(evdev_to_char(30), Some('a'));
        assert_eq!(evdev_to_char(44), Some('z'));
        assert_eq!(evdev_to_char(57), Some(' '));
    }
    #[test]
    fn control_keys_are_none() {
        assert_eq!(evdev_to_char(1), None);
        assert_eq!(evdev_to_char(14), None);
        assert_eq!(evdev_to_char(28), None);
    }
    #[test]
    fn unmapped_are_none() {
        assert_eq!(evdev_to_char(0), None);
        assert_eq!(evdev_to_char(u16::MAX), None);
    }
    #[test]
    fn event_struct_is_8_bytes() {
        assert_eq!(core::mem::size_of::<VirtioInputEvent>(), 8);
    }
}
