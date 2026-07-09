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

/// Poll for a key event.
/// Priority: UART first (primary — reliable via -serial stdio),
/// then virtio-input (secondary — GTK window focus required).
pub fn poll() -> Option<KeyEvent> {
    // UART path: works whenever -serial stdio is active and the terminal
    // has focus. This is the PL-14 primary path.
    if let Some(b) = uart_kbd::read_byte() {
        let code: u16 = match b {
            b'\r' | b'\n' => 28, // ENTER
            0x1b           => 1,  // ESC
            0x7f | 0x08    => 14, // BACKSPACE
            _              => 0,
        };
        return Some(KeyEvent {
            code,
            value: 1,
            ch: if b >= 0x20 && b < 0x7f { Some(b as char) } else { None },
        });
    }
    // virtio-input path: active only if DRIVER_OK handshake succeeded and
    // QEMU routes GTK window events to the virtio-keyboard-device.
    if !virtio_input::is_initialized() { return None; }
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

    // PL-14: virtio not initialized before init_device() runs on host tests
    #[test]
    fn virtio_not_initialized_at_startup() {
        assert!(!virtio_input::is_initialized(),
            "is_initialized() must return false before init_device() succeeds");
    }

    // PL-14: UART read_byte() is cfg-gated on aarch64; on host it always returns None.
    #[test]
    fn uart_kbd_none_on_host() {
        // cfg-gate ensures no hardware access on x86_64 test host.
        // read_byte() returns None unconditionally on non-aarch64 targets.
        assert_eq!(uart_kbd::read_byte(), None);
    }
}

