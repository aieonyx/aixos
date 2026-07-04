// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0

use crate::virtio_gpu::VirtioGpuRegs;

pub const QUEUE_SIZE: usize = 16;
pub const VIRTQ_DESC_F_WRITE: u16 = 2;
pub const VIRTQ_DESC_F_NEXT: u16 = 1;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct VirtqDesc {
    pub addr: u64,
    pub len: u32,
    pub flags: u16,
    pub next: u16,
}

#[repr(C)]
pub struct VirtqAvail {
    pub flags: u16,
    pub idx: u16,
    pub ring: [u16; QUEUE_SIZE],
    pub used_event: u16,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct VirtqUsedElem { pub id: u32, pub len: u32 }

#[repr(C)]
pub struct VirtqUsed {
    pub flags: u16,
    pub idx: u16,
    pub ring: [VirtqUsedElem; QUEUE_SIZE],
    pub avail_event: u16,
}

/// DMA-visible ring — must be page-aligned for legacy v1 QueuePFN.
/// QEMU legacy layout: desc at +0x000, avail at +0x100,
/// used at next 4096-aligned boundary after avail.
#[repr(C, align(4096))]
pub struct VirtqueueRing {
    pub desc: [VirtqDesc; QUEUE_SIZE],
    pub avail: VirtqAvail,
    _pad: [u8; 4096 - core::mem::size_of::<[VirtqDesc; QUEUE_SIZE]>()
               - core::mem::size_of::<VirtqAvail>()],
    pub used: VirtqUsed,
}

/// Bookkeeping separate from DMA ring.
pub struct Virtqueue {
    pub ring: VirtqueueRing,
    next_free: u16,
    last_used: u16,
}

const EMPTY_DESC: VirtqDesc = VirtqDesc { addr: 0, len: 0, flags: 0, next: 0 };
const EMPTY_USED: VirtqUsedElem = VirtqUsedElem { id: 0, len: 0 };

impl Virtqueue {
    pub const fn new() -> Self {
        Virtqueue {
            ring: VirtqueueRing {
                desc: [EMPTY_DESC; QUEUE_SIZE],
                avail: VirtqAvail { flags: 0, idx: 0, ring: [0; QUEUE_SIZE], used_event: 0 },
                _pad: [0u8; 4096 - core::mem::size_of::<[VirtqDesc; QUEUE_SIZE]>()
                           - core::mem::size_of::<VirtqAvail>()],
                used: VirtqUsed { flags: 0, idx: 0, ring: [EMPTY_USED; QUEUE_SIZE], avail_event: 0 },
            },
            next_free: 0,
            last_used: 0,
        }
    }

    pub fn add_buffer(&mut self, addr: u64, len: u32, write: bool) -> u16 {
        let idx = self.next_free % QUEUE_SIZE as u16;
        self.ring.desc[idx as usize] = VirtqDesc {
            addr, len,
            flags: if write { VIRTQ_DESC_F_WRITE } else { 0 },
            next: 0,
        };
        let slot = self.ring.avail.idx % QUEUE_SIZE as u16;
        self.ring.avail.ring[slot as usize] = idx;
        unsafe {
            core::arch::asm!("dsb sy", "isb", options(nostack, nomem));
        }
        self.ring.avail.idx = self.ring.avail.idx.wrapping_add(1);
        unsafe {
            core::arch::asm!("dsb sy", "isb", options(nostack, nomem));
        }
        self.next_free = self.next_free.wrapping_add(1);
        idx
    }


    /// Add two chained descriptors: cmd (read) -> resp (write)
    pub fn add_chained(&mut self, cmd_addr: u64, cmd_len: u32,
                        resp_addr: u64, resp_len: u32) {
        let cmd_idx = self.next_free % QUEUE_SIZE as u16;
        let resp_idx = (self.next_free + 1) % QUEUE_SIZE as u16;
        // Command descriptor points to response
        self.ring.desc[cmd_idx as usize] = VirtqDesc {
            addr: cmd_addr, len: cmd_len,
            flags: VIRTQ_DESC_F_NEXT,
            next: resp_idx,
        };
        // Response descriptor (device writes back)
        self.ring.desc[resp_idx as usize] = VirtqDesc {
            addr: resp_addr, len: resp_len,
            flags: VIRTQ_DESC_F_WRITE,
            next: 0,
        };
        // Add head (cmd) to available ring
        let slot = self.ring.avail.idx % QUEUE_SIZE as u16;
        self.ring.avail.ring[slot as usize] = cmd_idx;
        unsafe { core::arch::asm!("dsb sy", options(nostack, nomem)); }
        self.ring.avail.idx = self.ring.avail.idx.wrapping_add(1);
        unsafe { core::arch::asm!("dsb sy", options(nostack, nomem)); }
        self.next_free = self.next_free.wrapping_add(2);
    }

    pub fn notify(&self, regs: &VirtioGpuRegs, queue_idx: u32) {
        regs.notify_queue(queue_idx);
    }

    pub fn poll_used(&mut self) -> Option<u16> {
        unsafe { core::arch::asm!("dsb sy", "isb", options(nostack, nomem)); }
        if self.ring.used.idx == self.last_used { return None; }
        let slot = self.last_used % QUEUE_SIZE as u16;
        let id = self.ring.used.ring[slot as usize].id as u16;
        self.last_used = self.last_used.wrapping_add(1);
        Some(id)
    }
}

impl Default for Virtqueue {
    fn default() -> Self { Self::new() }
}
