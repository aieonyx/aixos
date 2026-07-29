#!/bin/bash
# Copyright (c) 2026 Edison Lepiten / AIEONYX
# SPDX-License-Identifier: Apache-2.0
# run-seL4.sh — Boot aiXos Phoenix under ASL-seL4 mKernel

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

GOLD='\033[0;33m'; RED='\033[0;31m'; NC='\033[0m'
gold() { echo -e "${GOLD}[AIEONYX]${NC} $*"; }
die()  { echo -e "${RED}[FAIL]${NC}  $*" >&2; exit 1; }

if [ -f "$SCRIPT_DIR/.seL4-image-path" ]; then
    IMAGE=$(cat "$SCRIPT_DIR/.seL4-image-path")
elif [ -f "$HOME/asl/build-v2/phoenix-v2.img" ]; then
    IMAGE="$HOME/asl/build-v2/phoenix-v2.img"
elif [ -f "$SCRIPT_DIR/asl/build-v2/phoenix-v2.img" ]; then
    IMAGE="$SCRIPT_DIR/asl/build-v2/phoenix-v2.img"
else
    die "Image not found — run: bash build-seL4.sh"
fi

echo ""
gold "aiXos Phoenix — Booting under ASL-seL4 mKernel"
gold "seL4 15.0.0 + Microkit 1.4.1 + 16 Protection Domains"
gold "Sovereign proof invariant: 0x4153"
gold "Image: $IMAGE"
echo "Exit: Ctrl-A then X"
echo ""

exec qemu-system-aarch64 \
  -machine virt,virtualization=on \
  -cpu cortex-a53 \
  -m 2G \
  -nographic \
  -chardev stdio,id=char0,mux=on,signal=off \
  -serial chardev:char0 \
  -mon chardev=char0,mode=readline \
  -device virtio-gpu-device \
  -device virtio-net-device,netdev=net0 \
  -netdev user,id=net0 \
  -device virtio-blk-device,drive=hd0 \
  -drive file=/tmp/sovereign.img,format=raw,id=hd0,if=none \
  -device loader,file="$IMAGE",addr=0x70000000,cpu-num=0
