#!/bin/sh
# Copyright (c) 2026 Edison Lepiten / AIEONYX
# SPDX-License-Identifier: Apache-2.0

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ISO="$REPO_ROOT/dist/aixos-phoenix-lite.iso"
EFI="/usr/share/qemu-efi-aarch64/QEMU_EFI.fd"

if [ ! -f "$ISO" ]; then
    echo "Image not found: $ISO"
    echo "Run: bash build/make-iso.sh"
    exit 1
fi

echo "Booting aiXos Phoenix Lite..."
qemu-system-aarch64 \
    -machine virt \
    -cpu cortex-a72 \
    -m 512M \
    -bios "$EFI" \
    -vga none \
    -device ramfb \
    -device virtio-keyboard-device \
    -device virtio-tablet-device \
    -device virtio-blk-device,drive=hd0 \
    -drive if=none,file="$ISO",format=raw,id=hd0,readonly=on \
    -display gtk \
    -serial pty \
    -no-reboot
