// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
#![cfg_attr(not(test), no_std)]

pub mod commands;
pub mod draw;
pub mod desktop;
pub mod font;
pub mod ramfb;
pub mod framebuffer;
pub mod virtio_gpu;
pub mod virtqueue;

pub const SOVEREIGN_PURPLE: u32 = 0xFF7B4FDB;
const RESOURCE_ID: u32 = 1;
#[allow(dead_code)]
const SCANOUT_ID: u32 = 0;

static mut CTRLQ: virtqueue::Virtqueue = virtqueue::Virtqueue::new();
static mut RESP_BUF: commands::GpuCtrlResp = commands::GpuCtrlResp {
    hdr: commands::GpuCtrlHdr { cmd_type: 0, flags: 0, fence_id: 0, ctx_id: 0, padding: 0 }
};
static mut ACTIVE_REGS: Option<virtio_gpu::VirtioGpuRegs> = None;
static mut LAST_WAIT: u32 = 0;
static mut RAMFB_ACTIVE: bool = false;

pub struct VirtioGpu { pub regs: virtio_gpu::VirtioGpuRegs }

#[allow(dead_code)]
impl VirtioGpu {
    fn submit(&self, ptr: *mut u8, len: u32) {
        unsafe {
            let ctrlq = &mut *core::ptr::addr_of_mut!(CTRLQ);
            let resp = core::ptr::addr_of_mut!(RESP_BUF);
            // Clear response
            core::ptr::write_volatile(
                resp as *mut u32,
                0u32
            );
            // Chained: cmd -> resp
            ctrlq.add_chained(
                ptr as u64, len,
                resp as u64,
                core::mem::size_of::<commands::GpuCtrlResp>() as u32
            );
            ctrlq.notify(&self.regs, 0);
            let mut tries: u32 = 0;
            loop {
                if ctrlq.poll_used().is_some() { break; }
                tries = tries.wrapping_add(1);
                if tries > 1_000_000 { break; }
            }
            LAST_WAIT = tries;
        }
    }

    fn resource_create_2d(&self, resource_id: u32, format: u32, width: u32, height: u32) {
        let mut cmd = commands::GpuResourceCreate2d {
            hdr: commands::GpuCtrlHdr { cmd_type: virtio_gpu::CMD_RESOURCE_CREATE_2D,
                flags: 0, fence_id: 0, ctx_id: 0, padding: 0 },
            resource_id, format, width, height,
        };
        self.submit(&mut cmd as *mut _ as *mut u8,
            core::mem::size_of::<commands::GpuResourceCreate2d>() as u32);
    }

    fn attach_backing(&self, resource_id: u32, addr: u64, length: u32) {
        let mut cmd = commands::GpuResourceAttachBacking {
            hdr: commands::GpuCtrlHdr { cmd_type: virtio_gpu::CMD_RESOURCE_ATTACH_BACKING,
                flags: 0, fence_id: 0, ctx_id: 0, padding: 0 },
            resource_id, nr_entries: 1,
            entry: commands::GpuMemEntry { addr, length, padding: 0 },
        };
        self.submit(&mut cmd as *mut _ as *mut u8,
            core::mem::size_of::<commands::GpuResourceAttachBacking>() as u32);
    }

    fn set_scanout_cmd(&self, scanout_id: u32, resource_id: u32, width: u32, height: u32) {
        let mut cmd = commands::GpuSetScanout {
            hdr: commands::GpuCtrlHdr { cmd_type: virtio_gpu::CMD_SET_SCANOUT,
                flags: 0, fence_id: 0, ctx_id: 0, padding: 0 },
            r: commands::GpuRect { x: 0, y: 0, width, height },
            scanout_id, resource_id,
        };
        self.submit(&mut cmd as *mut _ as *mut u8,
            core::mem::size_of::<commands::GpuSetScanout>() as u32);
    }

    fn transfer_to_host_2d(&self, resource_id: u32, x: u32, y: u32, w: u32, h: u32) {
        let mut cmd = commands::GpuTransferToHost2d {
            hdr: commands::GpuCtrlHdr { cmd_type: virtio_gpu::CMD_TRANSFER_TO_HOST_2D,
                flags: 0, fence_id: 0, ctx_id: 0, padding: 0 },
            r: commands::GpuRect { x, y, width: w, height: h },
            offset: 0, resource_id, padding: 0,
        };
        self.submit(&mut cmd as *mut _ as *mut u8,
            core::mem::size_of::<commands::GpuTransferToHost2d>() as u32);
    }

    fn resource_flush(&self, resource_id: u32, x: u32, y: u32, w: u32, h: u32) {
        let mut cmd = commands::GpuResourceFlush {
            hdr: commands::GpuCtrlHdr { cmd_type: virtio_gpu::CMD_RESOURCE_FLUSH,
                flags: 0, fence_id: 0, ctx_id: 0, padding: 0 },
            r: commands::GpuRect { x, y, width: w, height: h },
            resource_id, padding: 0,
        };
        self.submit(&mut cmd as *mut _ as *mut u8,
            core::mem::size_of::<commands::GpuResourceFlush>() as u32);
    }
}

pub fn init() -> Option<VirtioGpu> {
    #[cfg(test)] return None;
    #[cfg(not(test))] {
    // Try ramfb first
    let fb = framebuffer::fb_addr();
    if ramfb::init(fb, framebuffer::WIDTH as u32, framebuffer::HEIGHT as u32) {
        unsafe { RAMFB_ACTIVE = true; }
        return Some(VirtioGpu { regs: virtio_gpu::VirtioGpuRegs { base: 0 } });
    }
    // Fall back to virtio-gpu
    let regs = virtio_gpu::probe()?;
    // virtio-mmio v1 legacy init sequence
    regs.set_status(0);
    regs.set_status(virtio_gpu::STATUS_ACKNOWLEDGE);
    regs.set_status(virtio_gpu::STATUS_ACKNOWLEDGE | virtio_gpu::STATUS_DRIVER);
    let features = regs.device_features();
    regs.set_driver_features(features);
    regs.select_queue(0);
    if regs.queue_num_max() == 0 { return None; }
    regs.set_queue_num(virtqueue::QUEUE_SIZE as u32);
    // v1 legacy: use QueuePFN (page frame number) not descriptor addresses
    unsafe {
        let q = core::ptr::addr_of_mut!(CTRLQ);
        let ring_addr = core::ptr::addr_of_mut!((*q).ring) as u64;
        let pfn = (ring_addr >> 12) as u32;
        regs.set_queue_align(4096);
        regs.set_queue_pfn(pfn);
    }
    regs.set_status(virtio_gpu::STATUS_ACKNOWLEDGE | virtio_gpu::STATUS_DRIVER
        | virtio_gpu::STATUS_DRIVER_OK);
    let gpu = VirtioGpu { regs };
    gpu.resource_create_2d(RESOURCE_ID, virtio_gpu::FORMAT_B8G8R8X8_UNORM,
        framebuffer::WIDTH as u32, framebuffer::HEIGHT as u32);
    gpu.attach_backing(RESOURCE_ID, framebuffer::fb_addr(), framebuffer::fb_size());
    gpu.set_scanout_cmd(SCANOUT_ID, RESOURCE_ID,
        framebuffer::WIDTH as u32, framebuffer::HEIGHT as u32);
    unsafe { ACTIVE_REGS = Some(regs); }
    Some(gpu)
    }
}

pub fn set_scanout(_width: u32, _height: u32) {}

pub fn fill_rect(x: u32, y: u32, w: u32, h: u32, color: u32) {
    framebuffer::fill_rect(x, y, w, h, color);
    framebuffer::cache_flush();
}

pub fn flush(_x: u32, _y: u32, _w: u32, _h: u32) {
    // ramfb: framebuffer is directly memory-mapped, no flush needed
    // virtio-gpu: flush via transfer+resource_flush
    unsafe {
        if RAMFB_ACTIVE { return; }
        if let Some(regs) = ACTIVE_REGS {
            let gpu = VirtioGpu { regs };
            gpu.transfer_to_host_2d(RESOURCE_ID, _x, _y, _w, _h);
            gpu.resource_flush(RESOURCE_ID, _x, _y, _w, _h);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn init_returns_none_in_test_build() { assert!(init().is_none()); }
    #[test]
    fn sovereign_purple_matches_brand_value() { assert_eq!(SOVEREIGN_PURPLE, 0xFF7B4FDB); }
    #[test]
    fn flush_is_noop_without_active_device() { flush(0, 0, 1280, 720); }
}

#[cfg(not(test))]
pub fn dump_slots() {
    use virtio_gpu::{MMIO_SCAN_BASE, MMIO_STEP, MMIO_SCAN_SLOTS};
    let uart = 0x09000000 as *mut u8;
    let nibbles = b"0123456789abcdef";
    let mut slot = 0usize;
    while slot < MMIO_SCAN_SLOTS {
        let base = MMIO_SCAN_BASE + slot * MMIO_STEP;
        let magic = unsafe { core::ptr::read_volatile(base as *const u32) };
        if magic == 0x74726976 {
            let ver = unsafe { core::ptr::read_volatile((base + 0x004) as *const u32) };
            let dev = unsafe { core::ptr::read_volatile((base + 0x008) as *const u32) };
            unsafe {
                core::ptr::write_volatile(uart, b'[');
                core::ptr::write_volatile(uart, nibbles[slot / 16]);
                core::ptr::write_volatile(uart, nibbles[slot % 16]);
                core::ptr::write_volatile(uart, b'v');
                core::ptr::write_volatile(uart, nibbles[(ver & 0xf) as usize]);
                core::ptr::write_volatile(uart, b'=');
                core::ptr::write_volatile(uart, nibbles[((dev >> 4) & 0xf) as usize]);
                core::ptr::write_volatile(uart, nibbles[(dev & 0xf) as usize]);
                core::ptr::write_volatile(uart, b']');
            }
        }
        slot += 1;
    }
    unsafe { core::ptr::write_volatile(uart, b'\n'); }
}

#[cfg(test)]
pub fn dump_slots() {}

#[cfg(not(test))]
pub fn last_wait() -> u32 {
    unsafe { LAST_WAIT }
}
#[cfg(test)]
pub fn last_wait() -> u32 { 0 }

#[cfg(not(test))]
pub fn read_status() -> u32 {
    unsafe {
        if let Some(regs) = ACTIVE_REGS {
            return regs.status();
        }
    }
    0
}
#[cfg(test)]
pub fn read_status() -> u32 { 0 }
