#!/bin/sh
# Copyright (c) 2026 Edison Lepiten / AIEONYX
# SPDX-License-Identifier: Apache-2.0
set -e

cargo build --release --target aarch64-unknown-none --bin aixos
cp target/aarch64-unknown-none/release/aixos boot/aixos.elf

# Extract PE image (head + text sections only — no BSS/FB bloat)
aarch64-linux-gnu-objcopy \
    -O binary \
    --only-section=.head \
    --only-section=.text \
    boot/aixos.elf boot/aixos.img

echo "aiXos Phoenix Lite - build complete"
echo "ELF: boot/aixos.elf"
echo "IMG: boot/aixos.img"
echo "Sovereign proof: 0x4153"
