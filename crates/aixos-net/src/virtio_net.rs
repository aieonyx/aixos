// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// PL-53: virtio-net MMIO probe + AWP frame TX
#![allow(dead_code)]

// ── MMIO layout (mirrors virtio-gpu, different device ID) ─────────────────────
const MMIO_SCAN_BASE: usize = 0x0a00_0000;
const MMIO_STEP:      usize = 0x200;
const MMIO_SLOTS:     usize = 32;

const OFF_MAGIC:       usize = 0x000;
const OFF_VERSION:     usize = 0x004;
const OFF_DEVICE_ID:   usize = 0x008;
const OFF_DEV_FEAT:    usize = 0x010;
const OFF_DRV_FEAT:    usize = 0x020;
const OFF_STATUS:      usize = 0x070;
const OFF_QUEUE_SEL:   usize = 0x030;
const OFF_QUEUE_MAX:   usize = 0x034;
const OFF_QUEUE_NUM:   usize = 0x038;
const OFF_QUEUE_ALIGN: usize = 0x03c;
const OFF_QUEUE_PFN:   usize = 0x040;
const OFF_QUEUE_READY: usize = 0x044;
const OFF_QUEUE_NOTIF: usize = 0x050;

const VIRTIO_MAGIC:   u32 = 0x74726976;
const VIRTIO_V1:      u32 = 0x1;
const NET_DEVICE_ID:  u32 = 0x1;   // virtio-net device ID

pub const STATUS_ACK:    u32 = 1;
pub const STATUS_DRIVER: u32 = 2;
pub const STATUS_DRV_OK: u32 = 4;
pub const STATUS_FAILED: u32 = 128;

// TX queue index for virtio-net (0=RX, 1=TX, 2=ctrl)
pub const TX_QUEUE: u32 = 1;

// ── Virtqueue (self-contained, no_std, 16-slot) ────────────────────────────────
pub const QUEUE_SIZE: usize = 16;

#[repr(C)]
#[derive(Clone, Copy)]
struct VirtqDesc { addr: u64, len: u32, flags: u16, next: u16 }

#[repr(C)]
struct VirtqAvail { flags: u16, idx: u16, ring: [u16; QUEUE_SIZE], used_event: u16 }

#[repr(C)]
#[derive(Clone, Copy)]
struct VirtqUsedElem { id: u32, len: u32 }

#[repr(C)]
struct VirtqUsed { flags: u16, idx: u16, ring: [VirtqUsedElem; QUEUE_SIZE], avail_event: u16 }

#[repr(C, align(4096))]
struct NetRing {
    desc:  [VirtqDesc; QUEUE_SIZE],
    avail: VirtqAvail,
    _pad:  [u8; 4096
               - core::mem::size_of::<[VirtqDesc; QUEUE_SIZE]>()
               - core::mem::size_of::<VirtqAvail>()],
    used:  VirtqUsed,
}

// ── AWP frame format ───────────────────────────────────────────────────────────
// Bytes: [0..8]  magic "AIEONYX\0"
//        [8..16] node_id (u64 LE)
//        [16..20] version (u32 LE)
//        [20..24] payload_len (u32 LE)
//        [24..] payload bytes
pub const AWP_MAGIC: [u8; 8] = *b"AIEONYX\0";
pub const AWP_VERSION: u32 = 1;
pub const AWP_FRAME_HDR: usize = 24;
pub const AWP_MAX_PAYLOAD: usize = 232; // total frame ≤ 256B
pub const AWP_FRAME_MAX: usize = AWP_FRAME_HDR + AWP_MAX_PAYLOAD;

// virtio-net requires a 12-byte header before the packet
#[repr(C)]
struct VirtioNetHdr {
    flags:       u8,
    gso_type:    u8,
    hdr_len:     u16,
    gso_size:    u16,
    csum_start:  u16,
    csum_offset: u16,
    num_buffers: u16,
    // 2 bytes padding to align to 12
    _pad: u16,
}

// ── Static TX state ────────────────────────────────────────────────────────────
static mut TX_RING: NetRing = NetRing {
    desc: [VirtqDesc { addr: 0, len: 0, flags: 0, next: 0 }; QUEUE_SIZE],
    avail: VirtqAvail { flags: 0, idx: 0, ring: [0; QUEUE_SIZE], used_event: 0 },
    _pad: [0u8; 4096
               - core::mem::size_of::<[VirtqDesc; QUEUE_SIZE]>()
               - core::mem::size_of::<VirtqAvail>()],
    used: VirtqUsed { flags: 0, idx: 0,
        ring: [VirtqUsedElem { id: 0, len: 0 }; QUEUE_SIZE], avail_event: 0 },
};
static mut TX_NEXT: u16 = 0;

// Two static buffers: virtio-net header + AWP frame
static mut NET_HDR_BUF: VirtioNetHdr = VirtioNetHdr {
    flags: 0, gso_type: 0, hdr_len: 0, gso_size: 0,
    csum_start: 0, csum_offset: 0, num_buffers: 1, _pad: 0,
};
static mut AWP_BUF: [u8; AWP_FRAME_MAX] = [0u8; AWP_FRAME_MAX];
static mut AWP_BUF_LEN: usize = 0;

// Probe result
static mut NET_BASE: usize = 0;
static mut NET_LIVE: bool = false;
static mut FRAMES_SENT: u32 = 0;

// ── MMIO helpers ──────────────────────────────────────────────────────────────
#[inline]
unsafe fn read32(base: usize, off: usize) -> u32 {
    core::ptr::read_volatile((base + off) as *const u32)
}
#[inline]
unsafe fn write32(base: usize, off: usize, v: u32) {
    core::ptr::write_volatile((base + off) as *mut u32, v);
}
#[inline]
fn dsb() {
    #[cfg(target_arch = "aarch64")]
    unsafe { core::arch::asm!("dsb sy", options(nostack, nomem)); }
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Probe MMIO for virtio-net device and initialise TX queue.
/// Returns true if a virtio-net device was found and set up.
pub fn init() -> bool {
    #[cfg(not(target_arch = "aarch64"))]
    return false;

    #[cfg(target_arch = "aarch64")]
    unsafe {
        let mut slot = 0usize;
        while slot < MMIO_SLOTS {
            let base = MMIO_SCAN_BASE + slot * MMIO_STEP;
            let magic     = read32(base, OFF_MAGIC);
            let version   = read32(base, OFF_VERSION);
            let device_id = read32(base, OFF_DEVICE_ID);
            if magic == VIRTIO_MAGIC && version == VIRTIO_V1 && device_id == NET_DEVICE_ID {
                // Virtio legacy v1 init sequence
                write32(base, OFF_STATUS, 0);                        // reset
                write32(base, OFF_STATUS, STATUS_ACK);
                write32(base, OFF_STATUS, STATUS_ACK | STATUS_DRIVER);
                let _feat = read32(base, OFF_DEV_FEAT);
                write32(base, OFF_DRV_FEAT, 0);                      // no features

                // Select TX queue (queue 1)
                write32(base, OFF_QUEUE_SEL, TX_QUEUE);
                let qmax = read32(base, OFF_QUEUE_MAX);
                if qmax == 0 {
                    slot += 1;
                    continue;
                }
                let qsize = (QUEUE_SIZE as u32).min(qmax);
                write32(base, OFF_QUEUE_NUM, qsize);
                write32(base, OFF_QUEUE_ALIGN, 4096);

                // Set PFN (legacy v1: page frame number)
                let ring_addr = core::ptr::addr_of_mut!(TX_RING) as u64;
                let pfn = (ring_addr >> 12) as u32;
                write32(base, OFF_QUEUE_PFN, pfn);

                write32(base, OFF_STATUS,
                    STATUS_ACK | STATUS_DRIVER | STATUS_DRV_OK);

                NET_BASE = base;
                NET_LIVE = true;
                return true;
            }
            slot += 1;
        }
        false
    }
}

pub fn is_live() -> bool { unsafe { NET_LIVE } }
pub fn frames_sent() -> u32 { unsafe { FRAMES_SENT } }

/// Build and transmit one AWP frame via virtio-net TX queue.
/// payload: up to AWP_MAX_PAYLOAD bytes.
/// Returns true if frame was enqueued.
pub fn send_awp_frame(node_id: u64, payload: &[u8]) -> bool {
    #[cfg(not(target_arch = "aarch64"))]
    { let _ = (node_id, payload); return false; }

    #[cfg(target_arch = "aarch64")]
    unsafe {
        if !NET_LIVE { return false; }
        let plen = payload.len().min(AWP_MAX_PAYLOAD);

        // Build AWP frame into static buffer
        let frame = &mut *core::ptr::addr_of_mut!(AWP_BUF);
        // Magic
        frame[0..8].copy_from_slice(&AWP_MAGIC);
        // node_id LE
        let nid = node_id.to_le_bytes();
        frame[8..16].copy_from_slice(&nid);
        // version LE
        let ver = AWP_VERSION.to_le_bytes();
        frame[16..20].copy_from_slice(&ver);
        // payload_len LE
        let pl = (plen as u32).to_le_bytes();
        frame[20..24].copy_from_slice(&pl);
        // payload
        let mut i = 0;
        while i < plen { frame[24 + i] = payload[i]; i += 1; }
        AWP_BUF_LEN = AWP_FRAME_HDR + plen;

        // Use two chained descriptors:
        //   desc[0]: virtio-net header (read, NEXT)
        //   desc[1]: AWP frame (read)
        let hdr_ptr = core::ptr::addr_of_mut!(NET_HDR_BUF);
        let hdr_addr = hdr_ptr as u64;
        let hdr_len  = core::mem::size_of::<VirtioNetHdr>() as u32;
        let frm_addr = core::ptr::addr_of!(AWP_BUF) as u64;
        let frm_len  = AWP_BUF_LEN as u32;

        let ring = &mut *core::ptr::addr_of_mut!(TX_RING);
        let d0 = (TX_NEXT as usize) % QUEUE_SIZE;
        let d1 = (TX_NEXT as usize + 1) % QUEUE_SIZE;

        ring.desc[d0] = VirtqDesc { addr: hdr_addr, len: hdr_len, flags: 0x1 /* NEXT */, next: d1 as u16 };
        ring.desc[d1] = VirtqDesc { addr: frm_addr, len: frm_len, flags: 0, next: 0 };

        let slot = (ring.avail.idx as usize) % QUEUE_SIZE;
        ring.avail.ring[slot] = d0 as u16;
        dsb();
        ring.avail.idx = ring.avail.idx.wrapping_add(1);
        dsb();

        // Notify device: queue 1 = TX
        write32(NET_BASE, OFF_QUEUE_NOTIF, TX_QUEUE);

        TX_NEXT = TX_NEXT.wrapping_add(2);
        FRAMES_SENT = FRAMES_SENT.wrapping_add(1);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn awp_magic_is_correct() {
        assert_eq!(&AWP_MAGIC, b"AIEONYX\0");
    }

    #[test]
    fn awp_frame_hdr_size() {
        assert_eq!(AWP_FRAME_HDR, 24);
    }

    #[test]
    fn init_returns_false_on_host() {
        // No virtio-net MMIO on host
        assert!(!init());
    }

    #[test]
    fn send_awp_frame_returns_false_when_not_live() {
        assert!(!send_awp_frame(0x4153, b"sovereign"));
    }

    #[test]
    fn frames_sent_starts_at_zero() {
        assert_eq!(frames_sent(), 0);
    }
}
