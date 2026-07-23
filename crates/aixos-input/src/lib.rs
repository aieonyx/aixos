// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0

#![cfg_attr(not(test), no_std)]

pub mod mouse;
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

// Evdev codes for shift keys
const KEY_LEFTSHIFT:  u16 = 42;
const KEY_RIGHTSHIFT: u16 = 54;

// Shift state — set on press, cleared on release
static mut SHIFT_ACTIVE: bool = false;

/// Poll for a key event.
/// Priority: virtio-input first (GTK window), then UART (terminal).
pub fn poll() -> Option<KeyEvent> {
    // virtio-input path first — GTK window keyboard
    if virtio_input::is_initialized() {
        loop {
            let ev = match virtio_input::poll_event() {
                Some(e) => e,
                None => break,
            };
            if ev.code == KEY_LEFTSHIFT || ev.code == KEY_RIGHTSHIFT {
                unsafe {
                    SHIFT_ACTIVE = ev.value == EV_VALUE_PRESS || ev.value == EV_VALUE_REPEAT;
                }
                continue;
            }
            if ev.ev_type != EV_KEY { continue; }
            if ev.value != EV_VALUE_PRESS && ev.value != EV_VALUE_REPEAT { continue; }
            let shifted = unsafe { SHIFT_ACTIVE };
            return Some(KeyEvent {
                code: ev.code,
                value: ev.value,
                ch: evdev_to_char(ev.code, shifted),
            });
        }
    }

    // UART path — terminal keyboard (fallback)
    if let Some(b) = uart_kbd::read_byte() {
        let code: u16 = match b {
            b'\r' | b'\n' => 28,
            0x1b           => 1,
            0x7f | 0x08    => 14,
            _              => 0,
        };
        return Some(KeyEvent {
            code,
            value: 1,
            ch: if (0x20..0x7f).contains(&b) { Some(b as char) } else { None },
        });
    }

    None
}

pub fn evdev_to_char(code: u16, shift: bool) -> Option<char> {
    Some(if shift {
        match code {
            2=>'!', 3=>'@', 4=>'#', 5=>'$', 6=>'%',
            7=>'^', 8=>'&', 9=>'*', 10=>'(', 11=>')',
            16=>'Q', 17=>'W', 18=>'E', 19=>'R', 20=>'T',
            21=>'Y', 22=>'U', 23=>'I', 24=>'O', 25=>'P',
            30=>'A', 31=>'S', 32=>'D', 33=>'F', 34=>'G',
            35=>'H', 36=>'J', 37=>'K', 38=>'L',
            44=>'Z', 45=>'X', 46=>'C', 47=>'V', 48=>'B',
            49=>'N', 50=>'M', 57=>' ',
            12=>'_', 13=>'+', 26=>'{', 27=>'}', 39=>':',
            40=>'"', 41=>'~', 43=>'|', 51=>'<', 52=>'>',
            53=>'?',
            _ => return None,
        }
    } else {
        match code {
            2=>'1', 3=>'2', 4=>'3', 5=>'4', 6=>'5',
            7=>'6', 8=>'7', 9=>'8', 10=>'9', 11=>'0',
            16=>'q', 17=>'w', 18=>'e', 19=>'r', 20=>'t',
            21=>'y', 22=>'u', 23=>'i', 24=>'o', 25=>'p',
            30=>'a', 31=>'s', 32=>'d', 33=>'f', 34=>'g',
            35=>'h', 36=>'j', 37=>'k', 38=>'l',
            44=>'z', 45=>'x', 46=>'c', 47=>'v', 48=>'b',
            49=>'n', 50=>'m', 57=>' ',
            12=>'-', 13=>'=', 26=>'[', 27=>']', 39=>';',
            40=>'\'', 41=>'`', 43=>'\\', 51=>',', 52=>'.',
            53=>'/',
            _ => return None,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digits_map_unshifted() {
        assert_eq!(evdev_to_char(2, false), Some('1'));
        assert_eq!(evdev_to_char(11, false), Some('0'));
    }

    #[test]
    fn digits_map_shifted() {
        assert_eq!(evdev_to_char(2, true), Some('!'));
        assert_eq!(evdev_to_char(3, true), Some('@'));
        assert_eq!(evdev_to_char(9, true), Some('*'));
    }

    #[test]
    fn letters_unshifted() {
        assert_eq!(evdev_to_char(30, false), Some('a'));
        assert_eq!(evdev_to_char(44, false), Some('z'));
        assert_eq!(evdev_to_char(57, false), Some(' '));
    }

    #[test]
    fn letters_shifted() {
        assert_eq!(evdev_to_char(30, true), Some('A'));
        assert_eq!(evdev_to_char(44, true), Some('Z'));
        assert_eq!(evdev_to_char(57, true), Some(' '));
    }

    #[test]
    fn control_keys_are_none() {
        assert_eq!(evdev_to_char(1, false), None);
        assert_eq!(evdev_to_char(14, false), None);
        assert_eq!(evdev_to_char(28, false), None);
    }

    #[test]
    fn unmapped_are_none() {
        assert_eq!(evdev_to_char(0, false), None);
        assert_eq!(evdev_to_char(u16::MAX, false), None);
    }

    #[test]
    fn event_struct_is_8_bytes() {
        assert_eq!(core::mem::size_of::<VirtioInputEvent>(), 8);
    }

    #[test]
    fn virtio_not_initialized_at_startup() {
        assert!(!virtio_input::is_initialized());
    }

    #[test]
    fn uart_kbd_none_on_host() {
        assert_eq!(uart_kbd::read_byte(), None);
    }

    #[test]
    fn shift_gives_uppercase() {
        assert_eq!(evdev_to_char(18, true), Some('E'));
        assert_eq!(evdev_to_char(18, false), Some('e'));
    }

    #[test]
    fn symbols_unshifted_and_shifted() {
        assert_eq!(evdev_to_char(12, false), Some('-'));
        assert_eq!(evdev_to_char(12, true),  Some('_'));
    }
}

