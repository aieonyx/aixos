#!/bin/sh
# Copyright (c) 2026 Edison Lepiten / AIEONYX
set -e

BOOT_MODE="${BOOT_MODE:-live}"

echo "aiXos Phoenix - Sovereign Stack Initializing..."
echo "Boot mode: $BOOT_MODE"
echo "Sovereign proof: 0x4153"

qemu-system-aarch64 \
    -machine virt,virtualization=on,highmem=off \
    -cpu cortex-a53 \
    -m 2G \
    -smp 4 \
    -nographic \
    -kernel target/aarch64-unknown-none/release/aixos.elf
