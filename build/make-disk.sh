#!/bin/sh
# Copyright (c) 2026 Edison Lepiten / AIEONYX
# SPDX-License-Identifier: Apache-2.0
# PL-54: Create sovereign persistent disk image
set -e

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DIST="$REPO_ROOT/dist"
DISK="$DIST/aixos-sovereign.img"

mkdir -p "$DIST"

# 1MB raw disk — 2048 x 512-byte sectors
# Sector 0: sovereign store header
# Sectors 1-15: key-value store (32B key + 8B value = 40B per slot, 12 slots per sector)
dd if=/dev/zero of="$DISK" bs=512 count=2048 2>/dev/null

echo "Sovereign disk created: $DISK"
ls -lh "$DISK"
