// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
#![cfg_attr(not(test), no_std)]

pub mod commands;
pub mod font16;
pub mod logo;
pub mod aieonyx_logo; // PL-65C: official 128x128 ARGB logo
pub mod icons; // PL-65A: 32x32 ARGB sovereign icon bitmaps
pub mod cursor;
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


// seL4 PL-83: GPU command buffers in phoenix_heap (VA=PA guaranteed)
// Layout: 0x4A000000=Virtqueue, 0x4A010000=cmd bufs
const GPU_CMD_BASE: u64 = 0x4B010000;
const GPU_CMD_CREATE_OFFSET:   u64 = 0x000;
const GPU_CMD_ATTACH_OFFSET:   u64 = 0x040;
const GPU_CMD_SCANOUT_OFFSET:  u64 = 0x080;
const GPU_CMD_TRANSFER_OFFSET: u64 = 0x0c0;
const GPU_CMD_FLUSH_OFFSET:    u64 = 0x100;
const GPU_RESP_OFFSET:         u64 = 0x140;
// seL4 PL-83: static GPU command buffers (VA=PA, DMA-safe)
static mut CMD_CREATE_2D: commands::GpuResourceCreate2d =
    commands::GpuResourceCreate2d { hdr: commands::GpuCtrlHdr { cmd_type:0,flags:0,fence_id:0,ctx_id:0,padding:0 }, resource_id:0,format:0,width:0,height:0 };
static mut CMD_ATTACH: commands::GpuResourceAttachBacking =
    commands::GpuResourceAttachBacking { hdr: commands::GpuCtrlHdr { cmd_type:0,flags:0,fence_id:0,ctx_id:0,padding:0 }, resource_id:0,nr_entries:0,entry:commands::GpuMemEntry{addr:0,length:0,padding:0} };
static mut CMD_SCANOUT: commands::GpuSetScanout =
    commands::GpuSetScanout { hdr: commands::GpuCtrlHdr { cmd_type:0,flags:0,fence_id:0,ctx_id:0,padding:0 }, r:commands::GpuRect{x:0,y:0,width:0,height:0},scanout_id:0,resource_id:0 };
static mut CMD_TRANSFER: commands::GpuTransferToHost2d =
    commands::GpuTransferToHost2d { hdr: commands::GpuCtrlHdr { cmd_type:0,flags:0,fence_id:0,ctx_id:0,padding:0 }, r:commands::GpuRect{x:0,y:0,width:0,height:0},offset:0,resource_id:0,padding:0 };
static mut CMD_FLUSH: commands::GpuResourceFlush =
    commands::GpuResourceFlush { hdr: commands::GpuCtrlHdr { cmd_type:0,flags:0,fence_id:0,ctx_id:0,padding:0 }, r:commands::GpuRect{x:0,y:0,width:0,height:0},resource_id:0,padding:0 };

impl VirtioGpu {
    fn submit(&self, ptr: *mut u8, len: u32) {
        unsafe {
            // seL4 PL-83: use heap queue (VA=PA guaranteed at 0x4A000000)
            let ctrlq = &mut *(0x4B000000u64 as *mut virtqueue::Virtqueue);
            // Response buffer at 0x4A010140 (in phoenix_heap, VA=PA)
            let resp = (GPU_CMD_BASE + GPU_RESP_OFFSET) as *mut commands::GpuCtrlResp;
            // Clear response
            core::ptr::write_volatile(resp as *mut u32, 0u32);
            // Chained: cmd -> resp
            ctrlq.add_chained(
                ptr as u64, len,
                resp as u64,
                core::mem::size_of::<commands::GpuCtrlResp>() as u32
            );
            ctrlq.notify(&self.regs, 0);
            // seL4 PL-83: fire-and-forget — no blocking wait
            // QEMU processes virtio-gpu commands asynchronously
        }
    }

    fn resource_create_2d(&self, resource_id: u32, format: u32, width: u32, height: u32) {
        // seL4 PL-83: use static buffer (VA=PA, safe for DMA)
        unsafe {
            CMD_CREATE_2D = commands::GpuResourceCreate2d {
                hdr: commands::GpuCtrlHdr { cmd_type: virtio_gpu::CMD_RESOURCE_CREATE_2D,
                    flags: 0, fence_id: 0, ctx_id: 0, padding: 0 },
                resource_id, format, width, height,
            };
            self.submit(core::ptr::addr_of_mut!(CMD_CREATE_2D) as *mut u8,
                core::mem::size_of::<commands::GpuResourceCreate2d>() as u32);
        }
    }

    fn attach_backing(&self, resource_id: u32, addr: u64, length: u32) {
        unsafe {
            CMD_ATTACH = commands::GpuResourceAttachBacking {
                hdr: commands::GpuCtrlHdr { cmd_type: virtio_gpu::CMD_RESOURCE_ATTACH_BACKING,
                    flags: 0, fence_id: 0, ctx_id: 0, padding: 0 },
                resource_id, nr_entries: 1,
                entry: commands::GpuMemEntry { addr, length, padding: 0 },
            };
            self.submit(core::ptr::addr_of_mut!(CMD_ATTACH) as *mut u8,
                core::mem::size_of::<commands::GpuResourceAttachBacking>() as u32);
        }
    }

    fn set_scanout_cmd(&self, scanout_id: u32, resource_id: u32, width: u32, height: u32) {
        unsafe {
            CMD_SCANOUT = commands::GpuSetScanout {
                hdr: commands::GpuCtrlHdr { cmd_type: virtio_gpu::CMD_SET_SCANOUT,
                    flags: 0, fence_id: 0, ctx_id: 0, padding: 0 },
                r: commands::GpuRect { x: 0, y: 0, width, height },
                scanout_id, resource_id,
            };
            self.submit(core::ptr::addr_of_mut!(CMD_SCANOUT) as *mut u8,
                core::mem::size_of::<commands::GpuSetScanout>() as u32);
        }
    }

    fn transfer_to_host_2d(&self, resource_id: u32, x: u32, y: u32, w: u32, h: u32) {
        unsafe {
            CMD_TRANSFER = commands::GpuTransferToHost2d {
                hdr: commands::GpuCtrlHdr { cmd_type: virtio_gpu::CMD_TRANSFER_TO_HOST_2D,
                    flags: 0, fence_id: 0, ctx_id: 0, padding: 0 },
                r: commands::GpuRect { x, y, width: w, height: h },
                offset: 0, resource_id, padding: 0,
            };
            self.submit(core::ptr::addr_of_mut!(CMD_TRANSFER) as *mut u8,
                core::mem::size_of::<commands::GpuTransferToHost2d>() as u32);
        }
    }

    fn resource_flush(&self, resource_id: u32, x: u32, y: u32, w: u32, h: u32) {
        unsafe {
            CMD_FLUSH = commands::GpuResourceFlush {
                hdr: commands::GpuCtrlHdr { cmd_type: virtio_gpu::CMD_RESOURCE_FLUSH,
                    flags: 0, fence_id: 0, ctx_id: 0, padding: 0 },
                r: commands::GpuRect { x, y, width: w, height: h },
                resource_id, padding: 0,
            };
            self.submit(core::ptr::addr_of_mut!(CMD_FLUSH) as *mut u8,
                core::mem::size_of::<commands::GpuResourceFlush>() as u32);
        }
    }
}

pub fn init() -> Option<VirtioGpu> {
    #[cfg(test)] return None;
    #[cfg(not(test))] {
    // seL4 PL-83: try ramfb first (fw_cfg DMA at 0x4B008020, VA=PA)
    let fb = framebuffer::fb_addr();
    if ramfb::init(fb, framebuffer::WIDTH as u32, framebuffer::HEIGHT as u32) {
        unsafe { RAMFB_ACTIVE = true; }
        unsafe { let u = 0x09000000 as *mut u8; for b in b"GPU: ramfb ok\n" { u.write_volatile(*b); } }
        return Some(VirtioGpu { regs: virtio_gpu::VirtioGpuRegs { base: 0 } });
    }
    unsafe { let u = 0x09000000 as *mut u8; for b in b"GPU: ramfb fail\n" { u.write_volatile(*b); } }
    // Fall back to virtio-gpu
    let regs = virtio_gpu::probe()?;
    // seL4 PL-83: log GPU base address
    unsafe {
        let u = 0x09000000 as *mut u8;
        u.write_volatile(b'G');
        // Print base address nibble
        u.write_volatile(b'0' + ((regs.base >> 8) & 0xf) as u8);
        u.write_volatile(b'\n');
    }
    // virtio-mmio v1 legacy init sequence
    regs.set_status(0);
    regs.set_status(virtio_gpu::STATUS_ACKNOWLEDGE);
    regs.set_status(virtio_gpu::STATUS_ACKNOWLEDGE | virtio_gpu::STATUS_DRIVER);
    // v2 modern: negotiate features then set FEATURES_OK
    let features = regs.device_features() & 0x1;  // keep only VIRGL feature
    regs.set_driver_features(features);
    regs.set_status(virtio_gpu::STATUS_ACKNOWLEDGE | virtio_gpu::STATUS_DRIVER
        | virtio_gpu::STATUS_FEATURES_OK);
    // Verify FEATURES_OK was accepted
    let feat_status = regs.status();
    if feat_status & virtio_gpu::STATUS_FEATURES_OK == 0 { return None; }
    regs.select_queue(0);
    if regs.queue_num_max() == 0 { return None; }
    regs.set_queue_num(virtqueue::QUEUE_SIZE as u32);
    // v2 modern: use absolute physical addresses
    // phoenix_heap at 0x4A000000 has guaranteed VA=PA (system file identity map)
    // Place CTRLQ at start of heap
    unsafe {
        let heap_base = 0x4B000000u64;
        let q = heap_base as *mut virtqueue::Virtqueue;
        // Zero the queue memory
        core::ptr::write_bytes(q as *mut u8, 0, core::mem::size_of::<virtqueue::Virtqueue>());
        let q = &mut *q;
        // Descriptor table address
        let desc_addr = core::ptr::addr_of!(q.ring.desc) as u64;
        regs.set_queue_desc_low(desc_addr as u32);
        regs.set_queue_desc_high((desc_addr >> 32) as u32);
        // Available ring address
        let avail_addr = core::ptr::addr_of!(q.ring.avail) as u64;
        // Check if virtio_gpu has set_queue_avail
        regs.set_queue_avail_low(avail_addr as u32);
        regs.set_queue_avail_high((avail_addr >> 32) as u32);
        // Used ring address
        let used_addr = core::ptr::addr_of!(q.ring.used) as u64;
        regs.set_queue_used_low(used_addr as u32);
        regs.set_queue_used_high((used_addr >> 32) as u32);
        // Enable queue
        regs.set_queue_ready(1);
    }
    let final_status = regs.status();
    unsafe {
        let u = 0x09000000 as *mut u8;
        u.write_volatile(b'S');
        u.write_volatile(b'0' + (final_status & 0xf) as u8);
        u.write_volatile(b'\n');
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

#[cfg(not(test))]
fn fb_slice() -> &'static mut [u32] {
    unsafe {
        core::slice::from_raw_parts_mut(
            framebuffer::fb_addr() as *mut u32,
            (framebuffer::WIDTH as usize) * (framebuffer::HEIGHT as usize),
        )
    }
}

#[cfg(not(test))]
pub fn draw_cursor(x: i32, y: i32) {
    cursor::draw_cursor(fb_slice(), framebuffer::WIDTH as usize, x, y);
    framebuffer::cache_flush();
}
#[cfg(test)]
pub fn draw_cursor(_x: i32, _y: i32) {}

#[cfg(not(test))]
pub fn erase_cursor(x: i32, y: i32) {
    cursor::erase_cursor(fb_slice(), framebuffer::WIDTH as usize, x, y);
    framebuffer::cache_flush();
}
#[cfg(test)]
pub fn erase_cursor(_x: i32, _y: i32) {}

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

pub use logo::{blit_logo, blit_raw, LOGO_W, LOGO_H};

pub use font16::{draw_str_16, draw_str_16_clipped, draw_char_16};
