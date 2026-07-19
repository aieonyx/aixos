// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// virtio-mmio v1 absolute pointer driver

use core::ptr::{addr_of, addr_of_mut, read_volatile, write_volatile};

const MMIO_BASE: usize = 0x0a00_0000;
const SLOT_SIZE: usize = 0x200;
const MAGIC: u32 = 0x7472_6976;
const DEV_INPUT: u32 = 0x12;

const R_MAGIC: usize = 0x000;
const R_VERSION: usize = 0x004;
const R_DEVICE: usize = 0x008;
const R_GUEST_PAGE: usize = 0x028;
const R_QUEUE_SEL: usize = 0x030;
const R_QUEUE_NUM: usize = 0x038;
const R_QUEUE_PFN: usize = 0x040;
const R_NOTIFY: usize = 0x050;
const R_STATUS: usize = 0x070;
const R_CFG_SEL: usize = 0x100;
const R_CFG_SUBSEL: usize = 0x101;
const R_CFG_SIZE: usize = 0x102;
const R_CFG_DATA: usize = 0x108;

const CFG_EV_BITS: u8 = 0x11;
const EV_SYN: u16 = 0x00;
const EV_KEY: u16 = 0x01;
const EV_ABS: u16 = 0x03;
const ABS_X: u16 = 0x00;
const ABS_Y: u16 = 0x01;
const BTN_LEFT: u16 = 0x110;
const BTN_RIGHT: u16 = 0x111;

const QUEUE_SIZE: usize = 16;
const AVAIL_OFF: usize = QUEUE_SIZE * 16;
const USED_OFF: usize = 4096;

const SCREEN_W: i32 = 1280;
const SCREEN_H: i32 = 720;

#[derive(Clone, Copy)]
pub struct MouseState {
    pub x: i32,
    pub y: i32,
    pub left: bool,
    pub right: bool,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct InputEvent {
    ev_type: u16,
    code: u16,
    value: u32,
}

#[repr(align(4096))]
struct Ring([u8; 8192]);
static mut RING: Ring = Ring([0; 8192]);
static mut EVENTS: [InputEvent; QUEUE_SIZE] =
    [InputEvent { ev_type: 0, code: 0, value: 0 }; QUEUE_SIZE];

pub struct VirtioMouse {
    base: usize,
    last_used: u16,
    avail_idx: u16,
}

fn r32(base: usize, off: usize) -> u32 {
    unsafe { read_volatile((base + off) as *const u32) }
}
fn w32(base: usize, off: usize, v: u32) {
    unsafe { write_volatile((base + off) as *mut u32, v) }
}
fn r8(base: usize, off: usize) -> u8 {
    unsafe { read_volatile((base + off) as *const u8) }
}
fn w8(base: usize, off: usize, v: u8) {
    unsafe { write_volatile((base + off) as *mut u8, v) }
}

fn has_ev_abs(base: usize) -> bool {
    w8(base, R_CFG_SEL, CFG_EV_BITS);
    w8(base, R_CFG_SUBSEL, EV_ABS as u8);
    if r8(base, R_CFG_SIZE) == 0 {
        return false;
    }
    r8(base, R_CFG_DATA) & 0x03 != 0
}

pub fn init() -> Option<VirtioMouse> {
    for slot in (0..32).rev() {
        let base = MMIO_BASE + slot * SLOT_SIZE;
        if r32(base, R_MAGIC) != MAGIC
            || r32(base, R_VERSION) != 1
            || r32(base, R_DEVICE) != DEV_INPUT
            || !has_ev_abs(base)
        {
            continue;
        }
        return Some(setup(base));
    }
    None
}

fn setup(base: usize) -> VirtioMouse {
    let ring = unsafe { addr_of_mut!(RING.0) as *mut u8 };
    let evs = unsafe { addr_of_mut!(EVENTS) as *mut InputEvent };
    w32(base, R_STATUS, 0);
    w32(base, R_STATUS, 1);
    w32(base, R_STATUS, 3);
    w32(base, R_GUEST_PAGE, 4096);
    w32(base, R_QUEUE_SEL, 0);
    w32(base, R_QUEUE_NUM, QUEUE_SIZE as u32);
    unsafe {
        for i in 0..QUEUE_SIZE {
            let d = ring.add(i * 16);
            write_volatile(d as *mut u64, evs.add(i) as u64);
            write_volatile(d.add(8) as *mut u32, 8);
            write_volatile(d.add(12) as *mut u16, 2);
            write_volatile(d.add(14) as *mut u16, 0);
        }
        let avail = ring.add(AVAIL_OFF);
        write_volatile(avail as *mut u16, 0);
        for i in 0..QUEUE_SIZE {
            write_volatile(avail.add(4 + i * 2) as *mut u16, i as u16);
        }
        write_volatile(avail.add(2) as *mut u16, QUEUE_SIZE as u16);
    }
    w32(base, R_QUEUE_PFN, ((unsafe { addr_of!(RING) } as usize) >> 12) as u32);
    w32(base, R_STATUS, 7);
    w32(base, R_NOTIFY, 0);
    VirtioMouse { base, last_used: 0, avail_idx: QUEUE_SIZE as u16 }
}

impl VirtioMouse {
    fn ring_base(&self) -> usize {
        unsafe { addr_of!(RING) as usize }
    }

    fn used_idx(&self) -> u16 {
        unsafe { read_volatile((self.ring_base() + USED_OFF + 2) as *const u16) }
    }

    fn used_id(&self, n: u16) -> usize {
        let e = self.ring_base() + USED_OFF + 4 + ((n as usize) % QUEUE_SIZE) * 8;
        (unsafe { read_volatile(e as *const u32) }) as usize % QUEUE_SIZE
    }

    fn recycle(&mut self, id: usize) {
        let avail = self.ring_base() + AVAIL_OFF;
        unsafe {
            let slot = avail + 4 + ((self.avail_idx as usize) % QUEUE_SIZE) * 2;
            write_volatile(slot as *mut u16, id as u16);
            self.avail_idx = self.avail_idx.wrapping_add(1);
            write_volatile((avail + 2) as *mut u16, self.avail_idx);
        }
        w32(self.base, R_NOTIFY, 0);
    }

    pub fn poll(&mut self, state: &mut MouseState) -> bool {
        if self.used_idx() == self.last_used {
            return false;
        }
        let mut changed = false;
        loop {
            while self.used_idx() == self.last_used {
                core::hint::spin_loop();
            }
            let id = self.used_id(self.last_used);
            let ev = unsafe { read_volatile((addr_of!(EVENTS) as *const InputEvent).add(id)) };
            self.last_used = self.last_used.wrapping_add(1);
            self.recycle(id);
            match ev.ev_type {
                EV_ABS if ev.code == ABS_X => {
                    state.x = (ev.value as i32) * 1279 / 32767;
                    changed = true;
                }
                EV_ABS if ev.code == ABS_Y => {
                    state.y = (ev.value as i32) * 719 / 32767;
                    changed = true;
                }
                EV_KEY if ev.code == BTN_LEFT => {
                    state.left = ev.value != 0;
                    changed = true;
                }
                EV_KEY if ev.code == BTN_RIGHT => {
                    state.right = ev.value != 0;
                    changed = true;
                }
                EV_SYN => break,
                _ => {}
            }
        }
        if state.x < 0 { state.x = 0; }
        if state.x >= SCREEN_W { state.x = SCREEN_W - 1; }
        if state.y < 0 { state.y = 0; }
        if state.y >= SCREEN_H { state.y = SCREEN_H - 1; }
        changed
    }
}
