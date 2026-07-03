// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0

const OFF_MAGIC_VALUE: usize = 0x000;
const OFF_VERSION: usize = 0x004;
const OFF_DEVICE_ID: usize = 0x008;
const OFF_STATUS: usize = 0x070;
const OFF_CONFIG_SPACE: usize = 0x100;

const VIRTIO_MAGIC: u32 = 0x74726976;
const VIRTIO_VERSION: u32 = 0x2;
const INPUT_DEVICE_ID: u32 = 0x12;

pub const CFG_SELECT_OFFSET: usize = 0x00;
pub const CFG_SUBSEL_OFFSET: usize = 0x01;
pub const CFG_SIZE_OFFSET: usize = 0x02;
pub const CFG_DATA_OFFSET: usize = 0x08;
pub const VIRTIO_INPUT_CFG_SELECT_EV_KEY: u8 = 0x01;

pub struct VirtioInputEvent { pub event_type: u16, pub code: u16, pub value: u32 }
pub struct VirtioInputRegs { base: usize }

impl VirtioInputRegs {
    unsafe fn read32(&self, offset: usize) -> u32 {
        core::ptr::read_volatile((self.base + offset) as *const u32)
    }
    pub fn status(&self) -> u32 { unsafe { self.read32(OFF_STATUS) } }
    pub fn config_space_base(&self) -> usize { self.base + OFF_CONFIG_SPACE }
}

pub fn probe(scan_base: usize, step: usize, slots: usize) -> Option<VirtioInputRegs> {
    let mut slot = 0;
    while slot < slots {
        let base = scan_base + slot * step;
        let regs = VirtioInputRegs { base };
        let magic = unsafe { regs.read32(OFF_MAGIC_VALUE) };
        let version = unsafe { regs.read32(OFF_VERSION) };
        let device_id = unsafe { regs.read32(OFF_DEVICE_ID) };
        if magic == VIRTIO_MAGIC && version == VIRTIO_VERSION && device_id == INPUT_DEVICE_ID {
            return Some(regs);
        }
        slot += 1;
    }
    None
}

pub fn poll_event(_regs: &VirtioInputRegs) -> Option<VirtioInputEvent> { None }
