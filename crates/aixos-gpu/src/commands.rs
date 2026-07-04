// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0

#[repr(C)]
pub struct GpuCtrlHdr {
    pub cmd_type: u32, pub flags: u32,
    pub fence_id: u64, pub ctx_id: u32, pub padding: u32,
}

#[repr(C)]
pub struct GpuResourceCreate2d {
    pub hdr: GpuCtrlHdr,
    pub resource_id: u32, pub format: u32,
    pub width: u32, pub height: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GpuRect { pub x: u32, pub y: u32, pub width: u32, pub height: u32 }

#[repr(C)]
pub struct GpuSetScanout {
    pub hdr: GpuCtrlHdr,
    pub r: GpuRect, pub scanout_id: u32, pub resource_id: u32,
}

#[repr(C)]
pub struct GpuResourceFlush {
    pub hdr: GpuCtrlHdr,
    pub r: GpuRect, pub resource_id: u32, pub padding: u32,
}

#[repr(C)]
pub struct GpuTransferToHost2d {
    pub hdr: GpuCtrlHdr,
    pub r: GpuRect, pub offset: u64,
    pub resource_id: u32, pub padding: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GpuMemEntry { pub addr: u64, pub length: u32, pub padding: u32 }

#[repr(C)]
pub struct GpuResourceAttachBacking {
    pub hdr: GpuCtrlHdr,
    pub resource_id: u32, pub nr_entries: u32,
    pub entry: GpuMemEntry,
}

#[repr(C)]
pub struct GpuCtrlResp { pub hdr: GpuCtrlHdr }
