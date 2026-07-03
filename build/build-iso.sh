#!/bin/sh
# Copyright (c) 2026 Edison Lepiten / AIEONYX
set -e

cargo build --release --target aarch64-unknown-none --bin aixos
cp target/aarch64-unknown-none/release/aixos boot/aixos.elf
echo "aiXos Phoenix Lite - build complete"
echo "ELF: boot/aixos.elf"
echo "Sovereign proof: 0x4153"
