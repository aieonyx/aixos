// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// PL-55: Sovereign bump allocator
// Heap: _stack_top → 0x43F00000 (~3MB on QEMU virt 256MB RAM)
// Strategy: bump-pointer allocation, no free (arena style).
// Future: slab allocator on top for small objects.
#![allow(dead_code)]

// Heap bounds — just above stack, well below framebuffer at 0x44000000
pub const HEAP_START: usize = 0x4100_0000; // 16MB mark — safe above BSS+stack
pub const HEAP_END:   usize = 0x43F0_0000; // 1MB below framebuffer
pub const HEAP_SIZE:  usize = HEAP_END - HEAP_START; // ~47MB available

static mut BUMP_PTR: usize = HEAP_START;
static mut ALLOC_COUNT: usize = 0;
static mut BYTES_ALLOCATED: usize = 0;

/// Allocate `size` bytes aligned to `align`. Returns null on OOM.
/// align must be a power of two.
pub fn alloc(size: usize, align: usize) -> *mut u8 {
    if size == 0 { return core::ptr::null_mut(); }
    unsafe {
        // Round up bump pointer to alignment
        let aligned = (BUMP_PTR + align - 1) & !(align - 1);
        let new_ptr = aligned + size;
        if new_ptr > HEAP_END {
            return core::ptr::null_mut(); // OOM
        }
        BUMP_PTR = new_ptr;
        ALLOC_COUNT += 1;
        BYTES_ALLOCATED += size;
        aligned as *mut u8
    }
}

/// Allocate `size` bytes with default alignment (8 bytes).
pub fn alloc8(size: usize) -> *mut u8 {
    alloc(size, 8)
}

/// Allocate a zeroed region of `size` bytes.
pub fn alloc_zeroed(size: usize, align: usize) -> *mut u8 {
    let ptr = alloc(size, align);
    if !ptr.is_null() {
        unsafe {
            core::ptr::write_bytes(ptr, 0, size);
        }
    }
    ptr
}

/// Bump allocator does not support individual frees.
/// Call reset() to free the entire arena (useful for scratch allocations).
pub fn reset() {
    unsafe {
        BUMP_PTR = HEAP_START;
        ALLOC_COUNT = 0;
        BYTES_ALLOCATED = 0;
    }
}

/// Bytes used so far.
pub fn bytes_used() -> usize {
    unsafe { BUMP_PTR - HEAP_START }
}

/// Bytes remaining.
pub fn bytes_free() -> usize {
    unsafe {
        if BUMP_PTR > HEAP_END { 0 }
        else { HEAP_END - BUMP_PTR }
    }
}

/// Number of allocations made.
pub fn alloc_count() -> usize {
    unsafe { ALLOC_COUNT }
}

/// Total bytes requested (may differ from used due to alignment padding).
pub fn bytes_requested() -> usize {
    unsafe { BYTES_ALLOCATED }
}

// ── Typed allocation helpers ──────────────────────────────────────────────────

/// Allocate space for a value of type T and write it. Returns a raw pointer.
pub fn alloc_val<T>(val: T) -> *mut T {
    let ptr = alloc(core::mem::size_of::<T>(), core::mem::align_of::<T>()) as *mut T;
    if !ptr.is_null() {
        unsafe { core::ptr::write(ptr, val); }
    }
    ptr
}

/// Allocate a slice of `count` elements of type T (uninitialized).
pub fn alloc_slice<T>(count: usize) -> *mut T {
    alloc(
        core::mem::size_of::<T>() * count,
        core::mem::align_of::<T>(),
    ) as *mut T
}

/// Allocate a byte buffer and copy `data` into it. Returns pointer + len.
pub fn alloc_bytes(data: &[u8]) -> (*mut u8, usize) {
    let ptr = alloc(data.len(), 1);
    if !ptr.is_null() {
        unsafe {
            core::ptr::copy_nonoverlapping(data.as_ptr(), ptr, data.len());
        }
    }
    (ptr, data.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heap_constants_sane() {
        assert!(HEAP_START < HEAP_END);
        assert!(HEAP_SIZE > 1024 * 1024); // at least 1MB
        assert!(HEAP_END < 0x44000000);   // below framebuffer
    }

    #[test]
    #[cfg(target_arch = "aarch64")]
    fn alloc_returns_aligned_ptr() {
        reset();
        let p = alloc(16, 8);
        assert!(!p.is_null());
        assert_eq!(p as usize % 8, 0);
    }

    #[test]
    #[cfg(target_arch = "aarch64")]
    fn alloc_zero_returns_null() {
        reset();
        let p = alloc(0, 8);
        assert!(p.is_null());
    }

    #[test]
    #[cfg(target_arch = "aarch64")]
    fn two_allocs_dont_overlap() {
        reset();
        let p1 = alloc(64, 8);
        let p2 = alloc(64, 8);
        assert!(!p1.is_null());
        assert!(!p2.is_null());
        // p2 must be at least 64 bytes after p1
        let diff = p2 as usize - p1 as usize;
        assert!(diff >= 64);
    }

    #[test]
    #[cfg(target_arch = "aarch64")]
    fn bytes_used_tracks_allocations() {
        reset();
        let before = bytes_used();
        alloc(128, 8);
        let after = bytes_used();
        assert!(after >= before + 128);
    }

    #[test]
    #[cfg(target_arch = "aarch64")]
    fn alloc_zeroed_is_zero() {
        reset();
        let p = alloc_zeroed(32, 8);
        assert!(!p.is_null());
        for i in 0..32 {
            assert_eq!(unsafe { *p.add(i) }, 0);
        }
    }

    #[test]
    #[cfg(target_arch = "aarch64")]
    fn reset_clears_state() {
        reset();
        alloc(256, 8);
        reset();
        assert_eq!(bytes_used(), 0);
        assert_eq!(alloc_count(), 0);
    }

    #[test]
    #[cfg(target_arch = "aarch64")]
    fn alloc_val_roundtrip() {
        reset();
        let p = alloc_val::<u64>(0x4153);
        assert!(!p.is_null());
        assert_eq!(unsafe { *p }, 0x4153);
    }
}
