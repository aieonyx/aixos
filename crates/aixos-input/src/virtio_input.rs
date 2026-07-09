// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
use core::mem::{offset_of, size_of};
use core::ptr::{addr_of, addr_of_mut, read_volatile, write_volatile};

const MMIO_SCAN_BASE: usize = 0x0a00_0000;
const MMIO_SLOT_STEP: usize = 0x200;
const MMIO_SLOT_COUNT: usize = 32;
const VIRT_MAGIC: u32 = 0x7472_6976;
const DEVICE_ID_INPUT: u32 = 0x12;
const OFF_MAGIC: usize = 0x000;
const OFF_VERSION: usize = 0x004;
const OFF_DEVICE_ID: usize = 0x008;
const OFF_DRIVER_FEATURES: usize = 0x020;
const OFF_GUEST_PAGE_SIZE: usize = 0x028;
const OFF_QUEUE_SEL: usize = 0x030;
const OFF_QUEUE_NUM_MAX: usize = 0x034;
const OFF_QUEUE_NUM: usize = 0x038;
const OFF_QUEUE_ALIGN: usize = 0x03c;
const OFF_QUEUE_PFN: usize = 0x040;
const OFF_QUEUE_READY: usize = 0x044;
const OFF_QUEUE_NOTIFY: usize = 0x050;
const OFF_STATUS: usize = 0x070;
const OFF_QUEUE_DESC_LOW: usize = 0x080;
const OFF_QUEUE_DESC_HIGH: usize = 0x084;
const OFF_QUEUE_DRIVER_LOW: usize = 0x090;
const OFF_QUEUE_DRIVER_HIGH: usize = 0x094;
const OFF_QUEUE_DEVICE_LOW: usize = 0x0a0;
const OFF_QUEUE_DEVICE_HIGH: usize = 0x0a4;
const STATUS_ACKNOWLEDGE: u32 = 1;
const STATUS_DRIVER: u32 = 2;
const STATUS_DRIVER_OK: u32 = 4;
const STATUS_FEATURES_OK: u32 = 8;
const QUEUE_SIZE: usize = 16;
const VIRTQ_DESC_F_WRITE: u16 = 2;
const VIRTQ_AVAIL_F_NO_INTERRUPT: u16 = 1;
pub const EV_KEY: u16 = 0x01;
pub const EV_VALUE_PRESS: u32 = 1;
pub const EV_VALUE_REPEAT: u32 = 2;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct VirtioInputEvent { pub ev_type: u16, pub code: u16, pub value: u32 }

#[derive(Clone, Copy)]
#[repr(C)]
struct VirtqDesc { addr: u64, len: u32, flags: u16, next: u16 }

#[repr(C)]
struct VirtqAvail { flags: u16, idx: u16, ring: [u16; QUEUE_SIZE], used_event: u16 }

#[derive(Clone, Copy)]
#[repr(C)]
struct VirtqUsedElem { id: u32, len: u32 }

#[repr(C)]
struct VirtqUsed { flags: u16, idx: u16, ring: [VirtqUsedElem; QUEUE_SIZE], avail_event: u16 }

#[repr(C, align(4096))]
struct InputRing {
    desc: [VirtqDesc; QUEUE_SIZE],
    avail: VirtqAvail,
    _pad: [u8; 4096 - 294],
    used: VirtqUsed,
}

const _: () = assert!(size_of::<VirtioInputEvent>() == 8);
const _: () = assert!(size_of::<InputRing>() == 8192);
const _: () = assert!(offset_of!(InputRing, used) == 4096);

const ZERO_EVENT: VirtioInputEvent = VirtioInputEvent { ev_type: 0, code: 0, value: 0 };
static mut EVENT_BUF: [VirtioInputEvent; QUEUE_SIZE] = [ZERO_EVENT; QUEUE_SIZE];
static mut RING: InputRing = InputRing {
    desc: [VirtqDesc { addr: 0, len: 0, flags: 0, next: 0 }; QUEUE_SIZE],
    avail: VirtqAvail { flags: 0, idx: 0, ring: [0; QUEUE_SIZE], used_event: 0 },
    _pad: [0; 4096 - 294],
    used: VirtqUsed { flags: 0, idx: 0,
        ring: [VirtqUsedElem { id: 0, len: 0 }; QUEUE_SIZE], avail_event: 0 },
};
pub static mut DEVICE_BASE: usize = 0;
static mut DEVICE_VER: u32 = 0;
static mut LAST_SEEN: u16 = 0;
static mut AVAIL_IDX: u16 = 0;
// PL-14: track whether driver_ok was successfully written
static mut DRIVER_OK_WRITTEN: bool = false;

fn dsb() {
    #[cfg(target_arch = "aarch64")]
    unsafe { core::arch::asm!("dsb sy", options(nostack)); }
    #[cfg(not(target_arch = "aarch64"))]
    core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
}

unsafe fn mmio_read(base: usize, off: usize) -> u32 {
    read_volatile((base + off) as *const u32)
}
unsafe fn mmio_write(base: usize, off: usize, val: u32) {
    write_volatile((base + off) as *mut u32, val);
}

pub fn is_initialized() -> bool {
    // PL-14 fix: report true only after driver_ok handshake succeeded
    unsafe { DRIVER_OK_WRITTEN && DEVICE_BASE != 0 }
}

#[inline(never)]
pub fn probe() -> Option<(usize, u32)> {
    let uart = 0x09000000 as *mut u8;
    unsafe { write_volatile(uart, b'P'); }
    for slot in 0..MMIO_SLOT_COUNT {
        let base = MMIO_SCAN_BASE + slot * MMIO_SLOT_STEP;
        unsafe {
            let magic = read_volatile(base as *const u32);
            if magic != VIRT_MAGIC { continue; }
            let ver = read_volatile((base + OFF_VERSION) as *const u32);
            let dev = read_volatile((base + OFF_DEVICE_ID) as *const u32);
            let nb = b"0123456789abcdef";
            write_volatile(uart, nb[(slot >> 4) & 0xf]);
            write_volatile(uart, nb[slot & 0xf]);
            write_volatile(uart, b':');
            write_volatile(uart, nb[(ver & 0xf) as usize]);
            write_volatile(uart, b':');
            write_volatile(uart, nb[((dev >> 4) & 0xf) as usize]);
            write_volatile(uart, nb[(dev & 0xf) as usize]);
            write_volatile(uart, b' ');
            if (ver == 1 || ver == 2) && dev == DEVICE_ID_INPUT {
                DEVICE_VER = ver;
                write_volatile(uart, b'F');
                write_volatile(uart, b'\n');
                return Some((base, ver));
            }
        }
    }
    unsafe { write_volatile(uart, b'N'); write_volatile(uart, b'\n'); }
    None
}

#[inline(never)]
pub fn init_device(base: usize, ver: u32) -> bool {
    unsafe {
        DEVICE_VER = ver;
        let uart = 0x09000000 as *mut u8;
        write_volatile(uart, b'I');
        write_volatile(uart, if ver == 2 { b'2' } else { b'1' });
        mmio_write(base, OFF_STATUS, 0);
        mmio_write(base, OFF_STATUS, STATUS_ACKNOWLEDGE);
        mmio_write(base, OFF_STATUS, STATUS_ACKNOWLEDGE | STATUS_DRIVER);
        mmio_write(base, OFF_DRIVER_FEATURES, 0);
        if ver == 2 {
            mmio_write(base, OFF_STATUS,
                STATUS_ACKNOWLEDGE | STATUS_DRIVER | STATUS_FEATURES_OK);
            if mmio_read(base, OFF_STATUS) & STATUS_FEATURES_OK == 0 {
                write_volatile(uart, b'X'); return false;
            }
        }
        // PL-14 fix: do NOT set FEATURES_OK for v1 legacy (it's undefined for v1
        // and QEMU may silently reject the status transition, leaving active=false)
        mmio_write(base, OFF_QUEUE_SEL, 0);
        let max = mmio_read(base, OFF_QUEUE_NUM_MAX);
        if max == 0 { write_volatile(uart, b'Q'); return false; }
        mmio_write(base, OFF_QUEUE_NUM, QUEUE_SIZE as u32);
        let ring = addr_of_mut!(RING);
        if ver == 1 {
            mmio_write(base, OFF_GUEST_PAGE_SIZE, 4096);
            mmio_write(base, OFF_QUEUE_ALIGN, 4096);
            mmio_write(base, OFF_QUEUE_PFN, (ring as usize >> 12) as u32);
        } else { // ver == 2
            let desc = addr_of!((*ring).desc) as u64;
            let avail = addr_of!((*ring).avail) as u64;
            let used = addr_of!((*ring).used) as u64;
            mmio_write(base, OFF_QUEUE_DESC_LOW,    desc as u32);
            mmio_write(base, OFF_QUEUE_DESC_HIGH,   (desc >> 32) as u32);
            mmio_write(base, OFF_QUEUE_DRIVER_LOW,  avail as u32);
            mmio_write(base, OFF_QUEUE_DRIVER_HIGH, (avail >> 32) as u32);
            mmio_write(base, OFF_QUEUE_DEVICE_LOW,  used as u32);
            mmio_write(base, OFF_QUEUE_DEVICE_HIGH, (used >> 32) as u32);
            mmio_write(base, OFF_QUEUE_READY, 1);
        }
        let events = addr_of_mut!(EVENT_BUF) as *mut VirtioInputEvent;
        let mut i = 0;
        while i < QUEUE_SIZE {
            write_volatile(addr_of_mut!((*ring).desc[i]),
                VirtqDesc { addr: events.add(i) as u64,
                    len: size_of::<VirtioInputEvent>() as u32,
                    flags: VIRTQ_DESC_F_WRITE, next: 0 });
            write_volatile(addr_of_mut!((*ring).avail.ring[i]), i as u16);
            i += 1;
        }
        write_volatile(addr_of_mut!((*ring).avail.flags), VIRTQ_AVAIL_F_NO_INTERRUPT);
        dsb();
        write_volatile(addr_of_mut!((*ring).avail.idx), QUEUE_SIZE as u16);
        AVAIL_IDX = QUEUE_SIZE as u16;
        LAST_SEEN = 0;
        dsb();
        // PL-14 fix: v1 final status = ACK | DRIVER | DRIVER_OK only (no FEATURES_OK)
        // v2 final status = ACK | DRIVER | FEATURES_OK | DRIVER_OK
        let final_status = if ver == 1 {
            STATUS_ACKNOWLEDGE | STATUS_DRIVER | STATUS_DRIVER_OK
        } else {
            STATUS_ACKNOWLEDGE | STATUS_DRIVER | STATUS_FEATURES_OK | STATUS_DRIVER_OK
        };
        mmio_write(base, OFF_STATUS, final_status);
        dsb();
        // Verify DRIVER_OK is reflected back (QEMU mirrors STATUS register)
        let status_rb = mmio_read(base, OFF_STATUS);
        if status_rb & STATUS_DRIVER_OK == 0 {
            write_volatile(uart, b'D'); // DRIVER_OK not reflected — device rejected
            return false;
        }
        mmio_write(base, OFF_QUEUE_NOTIFY, 0);
        DEVICE_BASE = base;
        DRIVER_OK_WRITTEN = true;
        write_volatile(uart, b'K');
        write_volatile(uart, b'\n');
    }
    true
}

pub fn poll_event() -> Option<VirtioInputEvent> {
    unsafe {
        let base = DEVICE_BASE;
        if base == 0 { return None; }
        let ring = addr_of_mut!(RING);
        loop {
            dsb();
            let used_idx = read_volatile(addr_of!((*ring).used.idx));
            if used_idx == LAST_SEEN { return None; }
            let used_slot = (LAST_SEEN as usize) % QUEUE_SIZE;
            let id = read_volatile(addr_of!((*ring).used.ring[used_slot].id)) as usize;
            let slot = id % QUEUE_SIZE;
            let events = addr_of!(EVENT_BUF) as *const VirtioInputEvent;
            let ev = read_volatile(events.add(slot));
            let avail_slot = (AVAIL_IDX as usize) % QUEUE_SIZE;
            write_volatile(addr_of_mut!((*ring).avail.ring[avail_slot]), slot as u16);
            dsb();
            AVAIL_IDX = AVAIL_IDX.wrapping_add(1);
            write_volatile(addr_of_mut!((*ring).avail.idx), AVAIL_IDX);
            mmio_write(base, OFF_QUEUE_NOTIFY, 0);
            LAST_SEEN = LAST_SEEN.wrapping_add(1);
            if ev.ev_type == EV_KEY
               && (ev.value == EV_VALUE_PRESS || ev.value == EV_VALUE_REPEAT) {
                return Some(ev);
            }
        }
    }
}

pub fn debug_snapshot() -> (u32, u16, u16) {
    unsafe {
        let base = DEVICE_BASE;
        if base == 0 { return (0, 0, 0); }
        let ring = addr_of!(RING);
        dsb();
        let used_idx = read_volatile(addr_of!((*ring).used.idx));
        (mmio_read(base, OFF_STATUS), AVAIL_IDX, used_idx)
    }
}
