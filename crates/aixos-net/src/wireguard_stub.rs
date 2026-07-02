// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0

pub struct WgEnvelope { pub inner: u64 }

pub fn wrap(packet: u64) -> WgEnvelope {
    WgEnvelope { inner: packet }
}

pub fn unwrap(env: WgEnvelope) -> u64 {
    env.inner
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_unwrap_roundtrip() {
        assert_eq!(unwrap(wrap(0x4153)), 0x4153);
    }
}
