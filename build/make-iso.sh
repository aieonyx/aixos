#!/bin/sh
# Copyright (c) 2026 Edison Lepiten / AIEONYX
# SPDX-License-Identifier: Apache-2.0
# PL-13: Build aiXos Phoenix Lite bootable image
set -e

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DIST="$REPO_ROOT/dist"
ISO="$DIST/aixos-phoenix-lite.iso"
IMG="$REPO_ROOT/boot/aixos.img"

echo "=== aiXos Phoenix Lite ISO Build ==="
mkdir -p "$DIST"

# Create a raw disk image with GPT + FAT32 ESP
# Layout: 1MB gap + 64MB ESP partition
DISK="$DIST/disk.img"
dd if=/dev/zero of="$DISK" bs=1M count=68 2>/dev/null

# Create GPT partition table with one EFI partition
sgdisk -n 1:2048:131071 -t 1:ef00 -c 1:"EFI System" "$DISK"

# Format the ESP partition as FAT32
# Extract partition offset (sector 2048 * 512 = 1048576 bytes)
mkfs.fat -F 32 --offset 2048 "$DISK" $((131071 - 2048 + 1))

# Install EFI application
mmd -i "$DISK"@@1048576 ::/EFI
mmd -i "$DISK"@@1048576 ::/EFI/BOOT
mcopy -i "$DISK"@@1048576 "$IMG" ::/EFI/BOOT/BOOTAA64.EFI
echo "EFI application installed"

# Copy to ISO name (it is actually a raw disk image)
cp "$DISK" "$ISO"
rm -f "$DISK"

echo ""
echo "=== Image built ==="
ls -lh "$ISO"
sha256sum "$ISO"
sgdisk -p "$ISO" 2>/dev/null || true
