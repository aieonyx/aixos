#!/usr/bin/env bash
# Copyright (c) 2026 Edison Lepiten / AIEONYX
# SPDX-License-Identifier: Apache-2.0
# aiXos Phoenix Lite - QEMU aarch64 boot script.
set -e

IMAGE="${1:-target/aarch64-unknown-none/release/aixos-init}"

echo "aiXos Phoenix - Sovereign Stack Initializing..."
echo "axon_main() -> 0x4153"

qemu-system-aarch64 \
  -machine virt,virtualization=on,highmem=off \
  -cpu cortex-a53 \
  -m 2G \
  -smp 4 \
  -nographic \
  -device loader,file="$IMAGE",addr=0x70000000,cpu-num=0
