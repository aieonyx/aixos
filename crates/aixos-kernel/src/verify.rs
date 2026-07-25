// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// PL-64: Sovereign .axpkg verify-before-run gate
//
// Architectural invariant: NO .ax package executes without passing verify().
// Plain .ax files may run via `run` (dev/sovereign mode).
// .axpkg files must pass this gate before script bytes are handed to axon_interp.
//
// .axpkg wire format (bare-metal subset — no serde, no heap):
//   [0..4]   magic:        b"AXPK"
//   [4]      version:      u8 (1)
//   [5..9]   name_len:     u32 LE  (package name length)
//   [9..N]   name:         ASCII package name
//   [N..N+4] caps_mask:    u32 LE  (capability bitmask)
//   [N+4..M] script:       .ax source bytes (remainder minus hash trailer)
//   [M..M+8] fnv64_hash:   u64 LE  FNV-1a hash of name+caps_mask+script
//
// Capability bitmask (deny-by-default):
//   bit 0 = AWP_SEND    — script may send AWP frames
//   bit 1 = FS_READ     — script may read AXFS files
//   bit 2 = FS_WRITE    — script may write AXFS files
//   bit 3 = DB_READ     — script may read EdisonDB
//   bit 4 = DB_WRITE    — script may write EdisonDB
//   bit 5 = SPAWN       — script may spawn processes
//
// Verification steps (no heap, no alloc):
//   1. Magic check: bytes 0..4 == b"AXPK"
//   2. Version check: byte 4 == 1
//   3. Length sanity: minimum viable package size
//   4. FNV-1a hash check: recompute over name+caps+script, compare stored
//   5. Capability audit: if script uses `awp`, AWP_SEND bit must be set
//
// Result: VerifyGate::Pass(script_slice, caps) or VerifyGate::Reject(reason)

#![allow(dead_code)]

/// Capability flags — deny by default, declared in package header
pub const CAP_AWP_SEND:  u32 = 1 << 0;
pub const CAP_FS_READ:   u32 = 1 << 1;
pub const CAP_FS_WRITE:  u32 = 1 << 2;
pub const CAP_DB_READ:   u32 = 1 << 3;
pub const CAP_DB_WRITE:  u32 = 1 << 4;
pub const CAP_SPAWN:     u32 = 1 << 5;

pub const AXPKG_MAGIC:   &[u8; 4] = b"AXPK";
pub const AXPKG_VERSION: u8 = 1;
pub const HASH_TRAILER:  usize = 8; // FNV-64 stored as u64 LE

/// Reason a package was rejected
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum RejectReason {
    TooShort,
    InvalidMagic,
    VersionMismatch,
    NameTooLong,
    HashMismatch,
    CapabilityViolation, // script uses feature not declared in caps_mask
}

impl RejectReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::TooShort           => "package too short",
            Self::InvalidMagic       => "invalid magic (not AXPK)",
            Self::VersionMismatch    => "unsupported version",
            Self::NameTooLong        => "package name too long (max 64)",
            Self::HashMismatch       => "FNV-64 hash mismatch — tampered",
            Self::CapabilityViolation=> "capability violation — undeclared feature used",
        }
    }
}

/// Result of verify_axpkg()
pub enum VerifyGate<'a> {
    /// Package is verified — contains script bytes and declared caps
    Pass {
        name:    &'a [u8],
        script:  &'a [u8],
        caps:    u32,
    },
    Reject(RejectReason),
}

impl<'a> VerifyGate<'a> {
    pub fn is_pass(&self) -> bool { matches!(self, Self::Pass { .. }) }
    pub fn reject_reason(&self) -> Option<RejectReason> {
        match self { Self::Reject(r) => Some(*r), _ => None }
    }
}

/// FNV-1a 64-bit hash — sovereign integrity primitive
/// Same algorithm as EdisonDB embedding spec
pub fn fnv64(data: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in data {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01B3);
    }
    hash
}

/// Verify a raw .axpkg byte slice.
/// Returns VerifyGate::Pass with zero-copy slices into `data`,
/// or VerifyGate::Reject with the rejection reason.
///
/// No heap allocation — all slices point into the input buffer.
pub fn verify_axpkg(data: &[u8]) -> VerifyGate<'_> {
    // Minimum: 4 magic + 1 ver + 4 name_len + 0 name + 4 caps + 0 script + 8 hash = 21
    const MIN_LEN: usize = 4 + 1 + 4 + 4 + HASH_TRAILER;
    if data.len() < MIN_LEN {
        return VerifyGate::Reject(RejectReason::TooShort);
    }

    // Magic
    if &data[0..4] != AXPKG_MAGIC {
        return VerifyGate::Reject(RejectReason::InvalidMagic);
    }

    // Version
    if data[4] != AXPKG_VERSION {
        return VerifyGate::Reject(RejectReason::VersionMismatch);
    }

    // Name length
    let name_len = u32::from_le_bytes([data[5], data[6], data[7], data[8]]) as usize;
    if name_len > 64 {
        return VerifyGate::Reject(RejectReason::NameTooLong);
    }

    let name_start = 9;
    let name_end   = name_start + name_len;
    let caps_start = name_end;
    let caps_end   = caps_start + 4;
    let script_start = caps_end;

    if data.len() < caps_end + HASH_TRAILER {
        return VerifyGate::Reject(RejectReason::TooShort);
    }

    let script_end = data.len() - HASH_TRAILER;
    if script_end < script_start {
        return VerifyGate::Reject(RejectReason::TooShort);
    }

    let name   = &data[name_start..name_end];
    let caps   = u32::from_le_bytes([data[caps_start], data[caps_start+1],
                                     data[caps_start+2], data[caps_start+3]]);
    let script = &data[script_start..script_end];

    // FNV-64 integrity check
    // Hash covers: name + caps_mask bytes + script
    let stored_hash = u64::from_le_bytes([
        data[script_end],   data[script_end+1],
        data[script_end+2], data[script_end+3],
        data[script_end+4], data[script_end+5],
        data[script_end+6], data[script_end+7],
    ]);

    // Recompute: hash name, then caps bytes, then script
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in name {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01B3);
    }
    let caps_bytes = caps.to_le_bytes();
    for &b in &caps_bytes {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01B3);
    }
    for &b in script {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01B3);
    }

    if hash != stored_hash {
        return VerifyGate::Reject(RejectReason::HashMismatch);
    }

    // Capability audit — scan script for undeclared features
    if script_uses_awp(script) && (caps & CAP_AWP_SEND == 0) {
        return VerifyGate::Reject(RejectReason::CapabilityViolation);
    }

    VerifyGate::Pass { name, script, caps }
}

/// Check if script contains `awp ` statement (requires CAP_AWP_SEND)
fn script_uses_awp(script: &[u8]) -> bool {
    let needle = b"awp ";
    if script.len() < needle.len() { return false; }
    let mut i = 0;
    while i + needle.len() <= script.len() {
        if &script[i..i+needle.len()] == needle { return true; }
        i += 1;
    }
    false
}

/// Pack a .axpkg from components — for creating test packages in-kernel.
/// Writes into `out` buffer. Returns bytes written, or None if too small.
pub fn pack_axpkg(
    name: &[u8],
    script: &[u8],
    caps: u32,
    out: &mut [u8],
) -> Option<usize> {
    let name_len = name.len().min(64);
    let total = 4 + 1 + 4 + name_len + 4 + script.len() + HASH_TRAILER;
    if out.len() < total { return None; }

    out[0..4].copy_from_slice(AXPKG_MAGIC);
    out[4] = AXPKG_VERSION;
    let nl = name_len as u32;
    out[5..9].copy_from_slice(&nl.to_le_bytes());
    out[9..9+name_len].copy_from_slice(&name[..name_len]);

    let caps_off = 9 + name_len;
    out[caps_off..caps_off+4].copy_from_slice(&caps.to_le_bytes());

    let script_off = caps_off + 4;
    out[script_off..script_off+script.len()].copy_from_slice(script);

    let hash_off = script_off + script.len();

    // Compute hash
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in &name[..name_len] {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01B3);
    }
    for &b in &caps.to_le_bytes() {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01B3);
    }
    for &b in script {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01B3);
    }
    out[hash_off..hash_off+8].copy_from_slice(&hash.to_le_bytes());

    Some(total)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_pkg(name: &[u8], script: &[u8], caps: u32) -> [u8; 512] {
        let mut buf = [0u8; 512];
        pack_axpkg(name, script, caps, &mut buf).unwrap();
        buf
    }

    #[test]
    fn test_valid_package() {
        let buf = make_pkg(b"hello", b"print \"hello world\"", 0);
        let result = verify_axpkg(&buf[..4+1+4+5+4+19+8]);
        assert!(result.is_pass());
    }

    #[test]
    fn test_invalid_magic() {
        let mut buf = make_pkg(b"test", b"print \"x\"", 0);
        buf[0] = b'X'; // corrupt magic
        let result = verify_axpkg(&buf[..4+1+4+4+4+9+8]);
        assert_eq!(result.reject_reason(), Some(RejectReason::InvalidMagic));
    }

    #[test]
    fn test_hash_mismatch() {
        let mut buf = make_pkg(b"test", b"print \"hello\"", 0);
        // Corrupt a script byte
        let script_start = 9 + 4 + 4; // after magic+ver+name_len+name+caps
        buf[script_start] ^= 0xFF;
        let result = verify_axpkg(&buf[..4+1+4+4+4+13+8]);
        assert_eq!(result.reject_reason(), Some(RejectReason::HashMismatch));
    }

    #[test]
    fn test_awp_capability_violation() {
        // Script uses awp but caps don't declare AWP_SEND
        let buf = make_pkg(b"bad", b"awp hello", 0); // no CAP_AWP_SEND
        let result = verify_axpkg(&buf[..4+1+4+3+4+9+8]);
        assert_eq!(result.reject_reason(), Some(RejectReason::CapabilityViolation));
    }

    #[test]
    fn test_awp_capability_declared() {
        // Script uses awp and caps declare AWP_SEND — should pass
        let buf = make_pkg(b"ok", b"awp hello", CAP_AWP_SEND);
        let result = verify_axpkg(&buf[..4+1+4+2+4+9+8]);
        assert!(result.is_pass());
    }

    #[test]
    fn test_too_short() {
        let result = verify_axpkg(b"AXP");
        assert_eq!(result.reject_reason(), Some(RejectReason::TooShort));
    }

    #[test]
    fn test_pack_verify_roundtrip() {
        let name   = b"sovereign";
        let script = b"let x = 42\nprint x";
        let caps   = CAP_FS_READ | CAP_DB_READ;
        let mut buf = [0u8; 512];
        let len = pack_axpkg(name, script, caps, &mut buf).unwrap();
        let result = verify_axpkg(&buf[..len]);
        assert!(result.is_pass());
        if let VerifyGate::Pass { name: n, script: s, caps: c } = result {
            assert_eq!(n, name);
            assert_eq!(s, script);
            assert_eq!(c, caps);
        }
    }
}
