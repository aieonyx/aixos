// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0

pub const MMIO_SCAN_BASE: usize = 0x0a00_0000;
pub const MMIO_STEP: usize = 0x200;
pub const MMIO_SCAN_SLOTS: usize = 32;

const OFF_MAGIC_VALUE: usize = 0x000;
const OFF_VERSION: usize = 0x004;
const OFF_DEVICE_ID: usize = 0x008;
const OFF_DEVICE_FEATURES: usize = 0x010;
const OFF_DEVICE_FEATURES_SEL: usize = 0x014;
const OFF_DRIVER_FEATURES: usize = 0x020;
const OFF_DRIVER_FEATURES_SEL: usize = 0x024;
const OFF_STATUS: usize = 0x070;
const OFF_QUEUE_SEL: usize = 0x030;
const OFF_QUEUE_NUM_MAX: usize = 0x034;
const OFF_QUEUE_NUM: usize = 0x038;
const OFF_QUEUE_ALIGN: usize = 0x03c;
const OFF_QUEUE_READY: usize = 0x044;
const OFF_QUEUE_NOTIFY: usize = 0x050;
const OFF_QUEUE_DESC_LOW: usize = 0x080;
const OFF_QUEUE_DESC_HIGH: usize = 0x084;
const OFF_QUEUE_DRIVER_LOW: usize = 0x090;
const OFF_QUEUE_DRIVER_HIGH: usize = 0x094;
const OFF_QUEUE_DEVICE_LOW: usize = 0x0a0;
const OFF_QUEUE_DEVICE_HIGH: usize = 0x0a4;
const OFF_CONFIG_SPACE: usize = 0x100;
const OFF_QUEUE_PFN: usize = 0x040;
const QUEUE_PAGE_SHIFT: u32 = 12;
const QUEUE_PAGE_SIZE: usize = 4096;

const VIRTIO_MAGIC: u32 = 0x74726976;
const VIRTIO_VERSION: u32 = 0x1;
const GPU_DEVICE_ID: u32 = 0x10;

pub const CMD_GET_DISPLAY_INFO: u32 = 0x0100;
pub const CMD_RESOURCE_CREATE_2D: u32 = 0x0101;
pub const CMD_SET_SCANOUT: u32 = 0x0103;
pub const CMD_RESOURCE_FLUSH: u32 = 0x0104;
pub const CMD_TRANSFER_TO_HOST_2D: u32 = 0x0105;
pub const CMD_RESOURCE_ATTACH_BACKING: u32 = 0x0106;
pub const RESP_OK_NODATA: u32 = 0x1100;
pub const FORMAT_B8G8R8X8_UNORM: u32 = 2;

pub const STATUS_ACKNOWLEDGE: u32 = 1;
pub const STATUS_DRIVER: u32 = 2;
pub const STATUS_DRIVER_OK: u32 = 4;
pub const STATUS_FEATURES_OK: u32 = 8;
pub const STATUS_FAILED: u32 = 128;

#[derive(Clone, Copy)]
pub struct VirtioGpuRegs { pub base: usize }

impl VirtioGpuRegs {
    unsafe fn read32(&self, offset: usize) -> u32 {
        core::ptr::read_volatile((self.base + offset) as *const u32)
    }
    unsafe fn write32(&self, offset: usize, value: u32) {
        core::ptr::write_volatile((self.base + offset) as *mut u32, value);
    }
    pub fn status(&self) -> u32 { unsafe { self.read32(OFF_STATUS) } }
    pub fn set_status(&self, v: u32) { unsafe { self.write32(OFF_STATUS, v) } }
    pub fn device_features(&self) -> u32 { unsafe { self.read32(OFF_DEVICE_FEATURES) } }
    pub fn set_device_features_sel(&self, v: u32) { unsafe { self.write32(OFF_DEVICE_FEATURES_SEL, v) } }
    pub fn set_driver_features(&self, v: u32) { unsafe { self.write32(OFF_DRIVER_FEATURES, v) } }
    pub fn set_driver_features_sel(&self, v: u32) { unsafe { self.write32(OFF_DRIVER_FEATURES_SEL, v) } }
    pub fn select_queue(&self, i: u32) { unsafe { self.write32(OFF_QUEUE_SEL, i) } }
    pub fn queue_num_max(&self) -> u32 { unsafe { self.read32(OFF_QUEUE_NUM_MAX) } }
    pub fn set_queue_num(&self, n: u32) { unsafe { self.write32(OFF_QUEUE_NUM, n) } }
    pub fn set_queue_ready(&self, r: u32) { unsafe { self.write32(OFF_QUEUE_READY, r) } }
    pub fn notify_queue(&self, i: u32) { unsafe { self.write32(OFF_QUEUE_NOTIFY, i) } }
    pub fn set_queue_desc_low(&self, v: u32) { unsafe { self.write32(OFF_QUEUE_DESC_LOW, v) } }
    pub fn set_queue_desc_high(&self, v: u32) { unsafe { self.write32(OFF_QUEUE_DESC_HIGH, v) } }
    pub fn set_queue_driver_low(&self, v: u32) { unsafe { self.write32(OFF_QUEUE_DRIVER_LOW, v) } }
    pub fn set_queue_driver_high(&self, v: u32) { unsafe { self.write32(OFF_QUEUE_DRIVER_HIGH, v) } }
    pub fn set_queue_device_low(&self, v: u32) { unsafe { self.write32(OFF_QUEUE_DEVICE_LOW, v) } }
    pub fn set_queue_device_high(&self, v: u32) { unsafe { self.write32(OFF_QUEUE_DEVICE_HIGH, v) } }
    pub fn set_queue_align(&self, v: u32) { unsafe { self.write32(OFF_QUEUE_ALIGN, v) } }
    pub fn set_queue_pfn(&self, pfn: u32) { unsafe { self.write32(OFF_QUEUE_PFN, pfn) } }
    pub fn config_space_base(&self) -> usize { self.base + OFF_CONFIG_SPACE }
}

pub fn probe() -> Option<VirtioGpuRegs> {
    let mut slot = 0;
    while slot < MMIO_SCAN_SLOTS {
        let base = MMIO_SCAN_BASE + slot * MMIO_STEP;
        let regs = VirtioGpuRegs { base };
        let magic = unsafe { regs.read32(OFF_MAGIC_VALUE) };
        let version = unsafe { regs.read32(OFF_VERSION) };
        let device_id = unsafe { regs.read32(OFF_DEVICE_ID) };
        if magic == VIRTIO_MAGIC && version == VIRTIO_VERSION && device_id == GPU_DEVICE_ID {
            return Some(regs);
        }
        slot += 1;
    }
    None
}

pub fn probe_pci() -> Option<VirtioGpuRegs> {
    // PCI MMIO BAR scan for virtio-gpu-pci on QEMU virt machine
    // PCI config space at 0x3f000000, BAR0 gives MMIO address
    // Try known virtio-pci MMIO regions
    let candidates: [usize; 4] = [
        0x10000000,
        0x10001000,
        0x10002000,
        0x10003000,
    ];
    let mut i = 0;
    while i < 4 {
        let base = candidates[i];
        let regs = VirtioGpuRegs { base };
        let magic = unsafe { regs.read32(OFF_MAGIC_VALUE) };
        let version = unsafe { regs.read32(OFF_VERSION) };
        let device_id = unsafe { regs.read32(OFF_DEVICE_ID) };
        if magic == VIRTIO_MAGIC && version == VIRTIO_VERSION && device_id == GPU_DEVICE_ID {
            return Some(regs);
        }
        i += 1;
    }
    None
}
