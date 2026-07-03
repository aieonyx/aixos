// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0

pub const MMIO_SCAN_BASE: usize = 0x0a00_0000;
pub const MMIO_STEP: usize = 0x200;
pub const MMIO_SCAN_SLOTS: usize = 8;

const OFF_MAGIC_VALUE: usize = 0x000;
const OFF_VERSION: usize = 0x004;
const OFF_DEVICE_ID: usize = 0x008;
const OFF_STATUS: usize = 0x070;
const OFF_QUEUE_SEL: usize = 0x030;
const OFF_QUEUE_NUM_MAX: usize = 0x034;
const OFF_QUEUE_NUM: usize = 0x038;
const OFF_QUEUE_READY: usize = 0x044;
const OFF_QUEUE_NOTIFY: usize = 0x050;
const OFF_QUEUE_DESC_LOW: usize = 0x080;
const OFF_QUEUE_DRIVER_LOW: usize = 0x090;
const OFF_QUEUE_DEVICE_LOW: usize = 0x0a0;
const OFF_CONFIG_SPACE: usize = 0x100;

const VIRTIO_MAGIC: u32 = 0x74726976;
const VIRTIO_VERSION: u32 = 0x2;
const GPU_DEVICE_ID: u32 = 0x10;

pub const CMD_GET_DISPLAY_INFO: u32 = 0x0100;
pub const CMD_RESOURCE_CREATE_2D: u32 = 0x0101;
pub const CMD_SET_SCANOUT: u32 = 0x0103;
pub const CMD_RESOURCE_FLUSH: u32 = 0x0104;
pub const CMD_TRANSFER_TO_HOST_2D: u32 = 0x0105;
pub const CMD_RESOURCE_ATTACH_BACKING: u32 = 0x0106;
pub const RESP_OK_NODATA: u32 = 0x1100;
pub const FORMAT_B8G8R8X8_UNORM: u32 = 2;

pub struct VirtioGpuRegs { base: usize }

impl VirtioGpuRegs {
    unsafe fn read32(&self, offset: usize) -> u32 {
        core::ptr::read_volatile((self.base + offset) as *const u32)
    }
    unsafe fn write32(&self, offset: usize, value: u32) {
        core::ptr::write_volatile((self.base + offset) as *mut u32, value)
    }
    pub fn status(&self) -> u32 { unsafe { self.read32(OFF_STATUS) } }
    pub fn set_status(&self, v: u32) { unsafe { self.write32(OFF_STATUS, v) } }
    pub fn select_queue(&self, i: u32) { unsafe { self.write32(OFF_QUEUE_SEL, i) } }
    pub fn queue_num_max(&self) -> u32 { unsafe { self.read32(OFF_QUEUE_NUM_MAX) } }
    pub fn set_queue_num(&self, n: u32) { unsafe { self.write32(OFF_QUEUE_NUM, n) } }
    pub fn set_queue_ready(&self, r: u32) { unsafe { self.write32(OFF_QUEUE_READY, r) } }
    pub fn notify_queue(&self, i: u32) { unsafe { self.write32(OFF_QUEUE_NOTIFY, i) } }
    pub fn set_queue_desc_low(&self, a: u32) { unsafe { self.write32(OFF_QUEUE_DESC_LOW, a) } }
    pub fn set_queue_driver_low(&self, a: u32) { unsafe { self.write32(OFF_QUEUE_DRIVER_LOW, a) } }
    pub fn set_queue_device_low(&self, a: u32) { unsafe { self.write32(OFF_QUEUE_DEVICE_LOW, a) } }
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
