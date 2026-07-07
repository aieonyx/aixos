#!/bin/sh
# Copyright (c) 2026 Edison Lepiten / AIEONYX
echo "aiXos Phoenix - Sovereign Stack Initializing..."

qemu-system-aarch64 \
    -machine virt \
    -cpu cortex-a72 \
    -m 512M \
    -vga none \
    -device ramfb \
    -device virtio-keyboard-device \
    -display gtk \
    -serial stdio \
    -kernel boot/aixos.elf \
    -no-reboot
