#!/bin/sh
# Copyright (c) 2026 Edison Lepiten / AIEONYX

BOOT_MODE="${BOOT_MODE:-live}"
echo "aiXos Phoenix - Sovereign Stack Initializing..."
echo "Boot mode: $BOOT_MODE"
echo "Sovereign proof: 0x4153"

qemu-system-aarch64 \
    -machine virt,virtualization=on \
    -cpu cortex-a53 \
    -m 512M \
    -nographic \
    -device loader,file=boot/aixos.elf,cpu-num=0 \
    -no-reboot
