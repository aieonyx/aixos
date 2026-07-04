// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// RamfbCfg MUST be packed (28 bytes) to match QEMU QEMU_PACKED RAMFBCfg.
// repr(C) without packed pads to 32 bytes; fw_cfg rejects length mismatch.

const FW_CFG_DMA_ADDR: *mut u64 = 0x0902_0010 as *mut u64;
const FW_CFG_DMA_CTL_ERROR:  u32 = 0x01;
const FW_CFG_DMA_CTL_SELECT: u32 = 0x08;
const FW_CFG_DMA_CTL_WRITE:  u32 = 0x10;
const FW_CFG_RAMFB_KEY: u16 = 0x0025;
const FORMAT_XR24: u32 = 0x3432_5258;

#[repr(C, align(8))]
struct FwCfgDma { control: u32, length: u32, address: u64 }

#[repr(C, packed)]
struct RamfbCfg {
    addr: u64, fmt: u32, flags: u32,
    width: u32, height: u32, stride: u32,
}

const _: () = assert!(core::mem::size_of::<RamfbCfg>() == 28);
const _: () = assert!(core::mem::size_of::<FwCfgDma>() == 16);

pub fn init(fb_addr: u64, width: u32, height: u32) -> bool {
    let cfg = RamfbCfg {
        addr:   fb_addr.to_be(),
        fmt:    FORMAT_XR24.to_be(),
        flags:  0u32.to_be(),
        width:  width.to_be(),
        height: height.to_be(),
        stride: (width * 4).to_be(),
    };
    let dma = FwCfgDma {
        control: (((FW_CFG_RAMFB_KEY as u32) << 16)
            | FW_CFG_DMA_CTL_SELECT | FW_CFG_DMA_CTL_WRITE).to_be(),
        length:  (core::mem::size_of::<RamfbCfg>() as u32).to_be(),
        address: (core::ptr::addr_of!(cfg) as u64).to_be(),
    };
    unsafe {
        core::arch::asm!("dsb sy", "isb", options(nostack));
        FW_CFG_DMA_ADDR.write_volatile(
            (core::ptr::addr_of!(dma) as u64).to_be()
        );
        core::arch::asm!("dsb sy", "isb", options(nostack));
        let mut tries = 0u32;
        loop {
            let ctl = core::ptr::read_volatile(
                core::ptr::addr_of!(dma.control) as *const u32
            ).to_be();
            if ctl & FW_CFG_DMA_CTL_ERROR != 0 { return false; }
            if ctl == 0 { break; }
            tries += 1;
            if tries > 1_000_000 { return false; }
        }
    }
    true
}
