#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unreachable_code)]
// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// RamfbCfg MUST be packed (28 bytes) to match QEMU QEMU_PACKED RAMFBCfg.
const FW_CFG_DMA_ADDR: *mut u64 = 0x0902_0010 as *mut u64;
const FW_CFG_DMA_CTL_ERROR:  u32 = 0x01;
const FW_CFG_DMA_CTL_SELECT: u32 = 0x08;
const FW_CFG_DMA_CTL_WRITE:  u32 = 0x10;
const FW_CFG_RAMFB_KEY: u16 = 0x0025;
const FORMAT_XR24: u32 = 0x3432_5258;

// seL4 PL-83: use gpu_dma region (0x4B000000, uncached, VA=PA)
// Layout in gpu_dma:
//   0x4B000000: Virtqueue (for virtio-gpu)
//   0x4B008000: RamfbCfg (28 bytes)
//   0x4B008020: FwCfgDma (16 bytes)
const RAMFB_CFG_ADDR: u64 = 0x4B008000;
const RAMFB_DMA_ADDR: u64 = 0x4B008020;

pub fn init(fb_addr: u64, width: u32, height: u32) -> bool {
    unsafe {
        // Write RamfbCfg directly to gpu_dma region
        // struct RamfbCfg: addr(8) fmt(4) flags(4) width(4) height(4) stride(4) = 28 bytes
        let cfg = RAMFB_CFG_ADDR as *mut u8;
        // addr (big-endian u64)
        let addr_be = fb_addr.to_be();
        core::ptr::copy_nonoverlapping(
            &addr_be as *const u64 as *const u8, cfg, 8);
        // fmt (big-endian u32) = FORMAT_XR24
        let fmt_be = FORMAT_XR24.to_be();
        core::ptr::copy_nonoverlapping(
            &fmt_be as *const u32 as *const u8, cfg.add(8), 4);
        // flags = 0
        core::ptr::write_bytes(cfg.add(12), 0, 4);
        // width
        let w_be = width.to_be();
        core::ptr::copy_nonoverlapping(
            &w_be as *const u32 as *const u8, cfg.add(16), 4);
        // height
        let h_be = height.to_be();
        core::ptr::copy_nonoverlapping(
            &h_be as *const u32 as *const u8, cfg.add(20), 4);
        // stride = width * 4
        let s_be = (width * 4).to_be();
        core::ptr::copy_nonoverlapping(
            &s_be as *const u32 as *const u8, cfg.add(24), 4);

        // Write FwCfgDma to gpu_dma region
        // struct FwCfgDma: control(4) length(4) address(8) = 16 bytes
        let dma = RAMFB_DMA_ADDR as *mut u8;
        // control
        let ctrl = (((FW_CFG_RAMFB_KEY as u32) << 16)
            | FW_CFG_DMA_CTL_SELECT | FW_CFG_DMA_CTL_WRITE).to_be();
        core::ptr::copy_nonoverlapping(
            &ctrl as *const u32 as *const u8, dma, 4);
        // length = size of RamfbCfg = 28
        let len_be = 28u32.to_be();
        core::ptr::copy_nonoverlapping(
            &len_be as *const u32 as *const u8, dma.add(4), 4);
        // address = physical address of cfg (VA=PA in gpu_dma)
        let addr_be2 = RAMFB_CFG_ADDR.to_be();
        core::ptr::copy_nonoverlapping(
            &addr_be2 as *const u64 as *const u8, dma.add(8), 8);

        #[cfg(target_arch = "aarch64")]
        core::arch::asm!("dsb sy", options(nostack));

        // Write physical address of dma struct to fw_cfg
        FW_CFG_DMA_ADDR.write_volatile(RAMFB_DMA_ADDR.to_be());

        #[cfg(target_arch = "aarch64")]
        core::arch::asm!("dsb sy", options(nostack));

        // Poll for completion (check control field = 0)
        let mut tries = 0u32;
        loop {
            let ctl = core::ptr::read_volatile(dma as *const u32).to_be();
            if ctl & FW_CFG_DMA_CTL_ERROR != 0 { return false; }
            if ctl == 0 { break; }
            tries += 1;
            if tries > 1_000_000 { return false; }
        }
    }
    true
}
