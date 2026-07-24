// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// PL-54: virtio-blk MMIO probe + sovereign store read/write
#![allow(dead_code)]

// ── MMIO layout ───────────────────────────────────────────────────────────────
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
const OFF_QUEUE_NOTIF: usize = 0x050;

const VIRTIO_MAGIC:  u32 = 0x74726976;
const VIRTIO_V1:     u32 = 0x1;
const BLK_DEVICE_ID: u32 = 0x2;  // virtio-blk device ID

const STATUS_ACK:    u32 = 1;
const STATUS_DRIVER: u32 = 2;
const STATUS_DRV_OK: u32 = 4;

// ── Sovereign store layout (512-byte sectors) ─────────────────────────────────
// Sector 0: Header
//   [0..8]   magic b"AXSOV001"
//   [8..16]  node_id (u64 LE)
//   [16..20] entry_count (u32 LE)
//   [20..512] reserved
// Sectors 1-15: Key-value entries
//   Each entry: 32B key (null-padded) + 8B value (u64 LE) = 40B
//   Per sector: 12 entries (480B used, 32B reserved)
//   Total: 15 * 12 = 180 entries max

pub const SECTOR_SIZE: usize = 512;
pub const SOV_MAGIC: [u8; 8] = *b"AXSOV001";
pub const MAX_ENTRIES: usize = 180;
pub const ENTRY_SIZE: usize = 40;
pub const KEY_SIZE: usize = 32;
pub const ENTRIES_PER_SECTOR: usize = 12;
pub const DATA_SECTOR_START: usize = 1;
pub const DATA_SECTOR_COUNT: usize = 15;

// ── virtqueue (16-slot, self-contained) ───────────────────────────────────────
const QUEUE_SIZE: usize = 16;

#[repr(C)]
#[derive(Clone, Copy)]
struct VirtqDesc { addr: u64, len: u32, flags: u16, next: u16 }

#[repr(C)]
struct VirtqAvail { flags: u16, idx: u16, ring: [u16; QUEUE_SIZE], used_event: u16 }

#[repr(C)]
#[derive(Clone, Copy)]
struct VirtqUsedElem { id: u32, len: u32 }

#[repr(C)]
struct VirtqUsed {
    flags: u16, idx: u16,
    ring: [VirtqUsedElem; QUEUE_SIZE],
    avail_event: u16,
}

#[repr(C, align(4096))]
struct BlkRing {
    desc:  [VirtqDesc; QUEUE_SIZE],
    avail: VirtqAvail,
    _pad:  [u8; 4096
               - core::mem::size_of::<[VirtqDesc; QUEUE_SIZE]>()
               - core::mem::size_of::<VirtqAvail>()],
    used:  VirtqUsed,
}

// ── virtio-blk request header ─────────────────────────────────────────────────
const VIRTIO_BLK_T_IN:  u32 = 0; // read
const VIRTIO_BLK_T_OUT: u32 = 1; // write

#[repr(C)]
struct BlkReqHdr { req_type: u32, reserved: u32, sector: u64 }

// ── Static state ──────────────────────────────────────────────────────────────
static mut BLK_RING: BlkRing = BlkRing {
    desc:  [VirtqDesc { addr: 0, len: 0, flags: 0, next: 0 }; QUEUE_SIZE],
    avail: VirtqAvail { flags: 0, idx: 0, ring: [0; QUEUE_SIZE], used_event: 0 },
    _pad:  [0u8; 4096
               - core::mem::size_of::<[VirtqDesc; QUEUE_SIZE]>()
               - core::mem::size_of::<VirtqAvail>()],
    used:  VirtqUsed { flags: 0, idx: 0,
        ring: [VirtqUsedElem { id: 0, len: 0 }; QUEUE_SIZE], avail_event: 0 },
};
static mut BLK_NEXT: u16 = 0;
static mut BLK_BASE: usize = 0;
static mut BLK_LIVE: bool = false;

// Static DMA buffers
static mut BLK_REQ:    BlkReqHdr = BlkReqHdr { req_type: 0, reserved: 0, sector: 0 };
static mut BLK_BUF:    [u8; SECTOR_SIZE] = [0u8; SECTOR_SIZE];
static mut BLK_STATUS: u8 = 0xFF;

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
#[inline]
fn delay(n: u32) {
    let mut i = 0u32;
    while i < n { unsafe { core::arch::asm!("nop"); } i += 1; }
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Probe MMIO for a virtio-blk device that is NOT the boot ISO (hd0).
/// QEMU assigns MMIO slots in device order: hd0=slot N, net=slot M, hd1=slot P.
/// We probe all slots and pick the SECOND virtio-blk device found.
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
            if magic == VIRTIO_MAGIC && version == VIRTIO_V1 && device_id == BLK_DEVICE_ID {
                if setup(base) {
                    BLK_BASE = base;
                    BLK_LIVE = true;
                    // Test if writable: try writing to sector 0 and check status
                    // sovereign disk starts as zeros; ISO has GPT header
                    // Read sector 0 — if it's all zeros or has SOV_MAGIC, this is our disk
                    BLK_LIVE = true;
                    if let Some(sec) = read_sector(0) {
                        let is_gpt  = sec[0] == 0x45 && sec[1] == 0x46 && sec[2] == 0x49; // "EFI"
                        let is_sov  = &sec[0..8] == &SOV_MAGIC;
                        let is_zero = sec[0] == 0 && sec[1] == 0 && sec[2] == 0;
                        if is_sov || is_zero {
                            // This is the sovereign disk
                            return true;
                        }
                        if is_gpt {
                            // This is the ISO — skip
                            BLK_LIVE = false;
                            // Reset device
                            write32(base, OFF_STATUS, 0);
                        }
                    }
                }
            }
            slot += 1;
        }
        BLK_LIVE = false;
        false
    }
}

#[cfg(target_arch = "aarch64")]
unsafe fn setup(base: usize) -> bool {
    write32(base, OFF_STATUS, 0);
    write32(base, OFF_STATUS, STATUS_ACK);
    write32(base, OFF_STATUS, STATUS_ACK | STATUS_DRIVER);
    let _feat = read32(base, OFF_DEV_FEAT);
    write32(base, OFF_DRV_FEAT, 0);
    write32(base, OFF_QUEUE_SEL, 0);
    let qmax = read32(base, OFF_QUEUE_MAX);
    if qmax == 0 { return false; }
    let qsize = (QUEUE_SIZE as u32).min(qmax);
    write32(base, OFF_QUEUE_NUM, qsize);
    write32(base, OFF_QUEUE_ALIGN, 4096);
    let ring_addr = core::ptr::addr_of_mut!(BLK_RING) as u64;
    let pfn = (ring_addr >> 12) as u32;
    write32(base, OFF_QUEUE_PFN, pfn);
    write32(base, OFF_STATUS, STATUS_ACK | STATUS_DRIVER | STATUS_DRV_OK);
    true
}

pub fn is_live() -> bool { unsafe { BLK_LIVE } }

/// Read one 512-byte sector into BLK_BUF. Returns true on success.
pub fn read_sector(sector: u64) -> Option<&'static [u8; SECTOR_SIZE]> {
    #[cfg(not(target_arch = "aarch64"))]
    { let _ = sector; return None; }

    #[cfg(target_arch = "aarch64")]
    unsafe {
        if !BLK_LIVE { return None; }
        submit_request(VIRTIO_BLK_T_IN, sector);
        Some(&*core::ptr::addr_of!(BLK_BUF))
    }
}

/// Write BLK_BUF content to one 512-byte sector. Returns true on success.
pub fn write_sector(sector: u64, data: &[u8; SECTOR_SIZE]) -> bool {
    #[cfg(not(target_arch = "aarch64"))]
    { let _ = (sector, data); return false; }

    #[cfg(target_arch = "aarch64")]
    unsafe {
        if !BLK_LIVE { return false; }
        let buf = &mut *core::ptr::addr_of_mut!(BLK_BUF);
        buf.copy_from_slice(data);
        submit_request(VIRTIO_BLK_T_OUT, sector);
        true
    }
}

#[cfg(target_arch = "aarch64")]
unsafe fn submit_request(req_type: u32, sector: u64) {
    let ring = &mut *core::ptr::addr_of_mut!(BLK_RING);

    // Reset status
    BLK_STATUS = 0xFF;

    // Fill request header
    let req = &mut *core::ptr::addr_of_mut!(BLK_REQ);
    req.req_type = req_type;
    req.reserved = 0;
    req.sector   = sector;

    // 3 chained descriptors: [req_hdr(R)] -> [data(R/W)] -> [status(W)]
    let d0 = (BLK_NEXT as usize)     % QUEUE_SIZE;
    let d1 = (BLK_NEXT as usize + 1) % QUEUE_SIZE;
    let d2 = (BLK_NEXT as usize + 2) % QUEUE_SIZE;

    let is_write = req_type == VIRTIO_BLK_T_OUT;
    let data_flags: u16 = if is_write { 0x1 /* NEXT */ } else { 0x1 | 0x2 /* NEXT|WRITE */ };

    ring.desc[d0] = VirtqDesc {
        addr: core::ptr::addr_of!(BLK_REQ) as u64,
        len: core::mem::size_of::<BlkReqHdr>() as u32,
        flags: 0x1, // NEXT
        next: d1 as u16,
    };
    ring.desc[d1] = VirtqDesc {
        addr: core::ptr::addr_of!(BLK_BUF) as u64,
        len: SECTOR_SIZE as u32,
        flags: data_flags,
        next: d2 as u16,
    };
    ring.desc[d2] = VirtqDesc {
        addr: core::ptr::addr_of!(BLK_STATUS) as u64,
        len: 1,
        flags: 0x2, // WRITE
        next: 0,
    };

    let slot = (ring.avail.idx as usize) % QUEUE_SIZE;
    ring.avail.ring[slot] = d0 as u16;
    dsb();
    ring.avail.idx = ring.avail.idx.wrapping_add(1);
    dsb();

    write32(BLK_BASE, OFF_QUEUE_NOTIF, 0);

    // Poll for completion — read status volatile to prevent optimizer caching
    let mut timeout = 0u32;
    loop {
        dsb();
        let status = core::ptr::read_volatile(core::ptr::addr_of!(BLK_STATUS));
        if status != 0xFF || timeout >= 2_000_000 { break; }
        timeout += 1;
    }

    BLK_NEXT = BLK_NEXT.wrapping_add(3);
}

// ── Sovereign Store API ───────────────────────────────────────────────────────

/// Check if sovereign store header is valid on disk.
pub fn store_valid() -> bool {
    #[cfg(not(target_arch = "aarch64"))]
    return false;
    #[cfg(target_arch = "aarch64")]
    unsafe {
        if let Some(sec) = read_sector(0) {
            return &sec[0..8] == &SOV_MAGIC;
        }
        false
    }
}

/// Write the sovereign store header to sector 0.
pub fn store_format(node_id: u64) -> bool {
    #[cfg(not(target_arch = "aarch64"))]
    { let _ = node_id; return false; }
    #[cfg(target_arch = "aarch64")]
    unsafe {
        let mut buf = [0u8; SECTOR_SIZE];
        buf[0..8].copy_from_slice(&SOV_MAGIC);
        buf[8..16].copy_from_slice(&node_id.to_le_bytes());
        buf[16..20].copy_from_slice(&0u32.to_le_bytes());
        write_sector(0, &buf)
    }
}

/// Read a value from the sovereign store by key. Returns None if not found.
pub fn store_read(key: &[u8]) -> Option<u64> {
    #[cfg(not(target_arch = "aarch64"))]
    { let _ = key; return None; }
    #[cfg(target_arch = "aarch64")]
    unsafe {
        let klen = key.len().min(KEY_SIZE);
        for s in 0..DATA_SECTOR_COUNT {
            if let Some(sec) = read_sector((DATA_SECTOR_START + s) as u64) {
                for e in 0..ENTRIES_PER_SECTOR {
                    let off = e * ENTRY_SIZE;
                    if off + ENTRY_SIZE > SECTOR_SIZE { break; }
                    let k = &sec[off..off + KEY_SIZE];
                    if k[..klen] == key[..klen] && (klen == KEY_SIZE || k[klen] == 0) {
                        let vb: [u8; 8] = [
                            sec[off+KEY_SIZE], sec[off+KEY_SIZE+1],
                            sec[off+KEY_SIZE+2], sec[off+KEY_SIZE+3],
                            sec[off+KEY_SIZE+4], sec[off+KEY_SIZE+5],
                            sec[off+KEY_SIZE+6], sec[off+KEY_SIZE+7],
                        ];
                        return Some(u64::from_le_bytes(vb));
                    }
                    // Empty slot — stop searching this sector
                    if k[0] == 0 { break; }
                }
            }
        }
        None
    }
}

/// Write a key-value pair to the sovereign store.
/// Upserts in place if key exists, otherwise appends.
pub fn store_write(key: &[u8], value: u64) -> bool {
    #[cfg(not(target_arch = "aarch64"))]
    { let _ = (key, value); return false; }
    #[cfg(target_arch = "aarch64")]
    unsafe {
        let klen = key.len().min(KEY_SIZE);
        for s in 0..DATA_SECTOR_COUNT {
            let sector_idx = (DATA_SECTOR_START + s) as u64;
            if let Some(sec_ref) = read_sector(sector_idx) {
                let mut buf = [0u8; SECTOR_SIZE];
                buf.copy_from_slice(sec_ref);
                for e in 0..ENTRIES_PER_SECTOR {
                    let off = e * ENTRY_SIZE;
                    if off + ENTRY_SIZE > SECTOR_SIZE { break; }
                    let k = &buf[off..off + KEY_SIZE];
                    let is_match = k[..klen] == key[..klen] && (klen == KEY_SIZE || k[klen] == 0);
                    let is_empty = k[0] == 0;
                    if is_match || is_empty {
                        // Write key
                        let mut i = 0;
                        while i < klen { buf[off + i] = key[i]; i += 1; }
                        while i < KEY_SIZE { buf[off + i] = 0; i += 1; }
                        // Write value
                        let vb = value.to_le_bytes();
                        let mut j = 0;
                        while j < 8 { buf[off + KEY_SIZE + j] = vb[j]; j += 1; }
                        return write_sector(sector_idx, &buf);
                    }
                }
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sov_magic_correct() {
        assert_eq!(&SOV_MAGIC, b"AXSOV001");
    }

    #[test]
    fn sector_size_is_512() {
        assert_eq!(SECTOR_SIZE, 512);
    }

    #[test]
    fn entry_size_fits_sector() {
        assert!(ENTRIES_PER_SECTOR * ENTRY_SIZE <= SECTOR_SIZE);
    }

    #[test]
    fn init_returns_false_on_host() {
        assert!(!init());
    }

    #[test]
    fn store_read_returns_none_on_host() {
        assert!(store_read(b"user:tz").is_none());
    }
}
