// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// AXFS — Sovereign File System (PL-50 stub, PL-56 persistent)
// In-memory file table backed by virtio-blk sovereign disk sectors 16-31.
// Files persist across reboots when virtio-blk is live.
#![cfg_attr(not(test), no_std)]
#[cfg(target_arch = "aarch64")]
extern crate aixos_kernel;

/// Maximum files in the sovereign file table (matches AXFS_SECTOR_COUNT)
pub const MAX_FILES: usize = 16;
/// Maximum file name length (bytes)
pub const NAME_LEN: usize = 32;
/// Maximum file content size (bytes)
pub const DATA_LEN: usize = 256;

#[derive(Clone, Copy)]
pub struct AxFile {
    pub name: [u8; NAME_LEN],
    pub name_len: usize,
    pub data: [u8; DATA_LEN],
    pub data_len: usize,
}

impl AxFile {
    const fn empty() -> Self {
        AxFile {
            name: [0u8; NAME_LEN],
            name_len: 0,
            data: [0u8; DATA_LEN],
            data_len: 0,
        }
    }

    pub fn name_bytes(&self) -> &[u8] {
        &self.name[..self.name_len]
    }

    pub fn data_bytes(&self) -> &[u8] {
        &self.data[..self.data_len]
    }
}

static mut FILES: [AxFile; MAX_FILES] = [AxFile::empty(); MAX_FILES];
static mut FILE_COUNT: usize = 0;
static mut INITIALIZED: bool = false;

/// Initialize AXFS — seeds the boot readme file.
pub fn init() {
    unsafe {
        if INITIALIZED {
            return;
        }
        INITIALIZED = true;
        // Seed sovereign boot file
        write(b"readme.txt", b"aiXos Phoenix -- Sovereign OS\nAIEONYX | Build: green\n");
    }
}

pub fn is_live() -> bool {
    unsafe { INITIALIZED }
}

pub fn count() -> usize {
    unsafe { FILE_COUNT }
}

/// Write (or overwrite) a file. Returns false if table is full.
pub fn write(name: &[u8], data: &[u8]) -> bool {
    let nlen = name.len().min(NAME_LEN);
    let dlen = data.len().min(DATA_LEN);
    unsafe {
        let fc = FILE_COUNT;
        // Upsert: check if name already exists
        for i in 0..fc {
            if FILES[i].name_bytes() == &name[..nlen] {
                FILES[i].data_len = dlen;
                let mut j = 0;
                while j < dlen { FILES[i].data[j] = data[j]; j += 1; }
                return true;
            }
        }
        // Append
        if fc >= MAX_FILES {
            return false;
        }
        let mut f = AxFile::empty();
        f.name_len = nlen;
        let mut i = 0;
        while i < nlen { f.name[i] = name[i]; i += 1; }
        f.data_len = dlen;
        let mut j = 0;
        while j < dlen { f.data[j] = data[j]; j += 1; }
        FILES[fc] = f;
        FILE_COUNT = fc + 1;
        true
    }
}

/// Returns a reference to the file at index i, or None.
pub fn file_at(i: usize) -> Option<&'static AxFile> {
    unsafe {
        if i >= FILE_COUNT { return None; }
        Some(&FILES[i])
    }
}

/// Find file by name, return index or None.
pub fn find(name: &[u8]) -> Option<usize> {
    unsafe {
        for i in 0..FILE_COUNT {
            if FILES[i].name_bytes() == name {
                return Some(i);
            }
        }
        None
    }
}

/// Sync all in-memory AXFS files to sovereign disk (PL-56).
/// Called after every write() when virtio-blk is live.
#[cfg(target_arch = "aarch64")]
pub fn sync_to_disk() {
    unsafe {
        let count = FILE_COUNT;
        let mut slot = 0;
        while slot < count {
            let f = &FILES[slot];
            aixos_kernel::virtio_blk::axfs_write_file(
                slot,
                &f.name[..],
                f.name_len,
                &f.data[..],
                f.data_len,
            );
            slot += 1;
        }
        // Clear remaining slots
        while slot < aixos_kernel::virtio_blk::AXFS_SECTOR_COUNT {
            let empty_name = [0u8; 32];
            let empty_data = [0u8; 256];
            aixos_kernel::virtio_blk::axfs_write_file(slot, &empty_name, 0, &empty_data, 0);
            slot += 1;
        }
    }
}

#[cfg(not(target_arch = "aarch64"))]
pub fn sync_to_disk() {}

/// Load AXFS files from sovereign disk into memory (PL-56).
/// Called at boot after virtio-blk init.
#[cfg(target_arch = "aarch64")]
pub fn load_from_disk() {
    unsafe {
        FILE_COUNT = 0;
        let mut slot = 0;
        while slot < aixos_kernel::virtio_blk::AXFS_SECTOR_COUNT && slot < MAX_FILES {
            let mut name_buf = [0u8; 32];
            let mut data_buf = [0u8; 256];
            if let Some((nlen, dlen)) = aixos_kernel::virtio_blk::axfs_read_file(
                slot, &mut name_buf, &mut data_buf
            ) {
                let mut f = AxFile::empty();
                f.name_len = nlen;
                let mut i = 0; while i < nlen { f.name[i] = name_buf[i]; i += 1; }
                f.data_len = dlen;
                let mut j = 0; while j < dlen { f.data[j] = data_buf[j]; j += 1; }
                FILES[slot] = f;
                FILE_COUNT = slot + 1;
            } else {
                break; // empty slot — stop
            }
            slot += 1;
        }
    }
}

#[cfg(not(target_arch = "aarch64"))]
pub fn load_from_disk() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_seeds_readme() {
        init();
        assert!(is_live());
        assert!(count() >= 1);
        let f = file_at(0).unwrap();
        assert_eq!(f.name_bytes(), b"readme.txt");
    }

    #[test]
    fn write_and_find() {
        init();
        assert!(write(b"hello.txt", b"sovereign"));
        let idx = find(b"hello.txt").expect("file not found");
        let f = file_at(idx).unwrap();
        assert_eq!(f.data_bytes(), b"sovereign");
    }

    #[test]
    fn write_upserts_in_place() {
        init();
        write(b"uptest.txt", b"v1");
        let before = count();
        write(b"uptest.txt", b"v2");
        assert_eq!(count(), before);
        let idx = find(b"uptest.txt").unwrap();
        assert_eq!(file_at(idx).unwrap().data_bytes(), b"v2");
    }

    #[test]
    fn find_missing_returns_none() {
        init();
        assert!(find(b"ghost.txt").is_none());
    }
}
