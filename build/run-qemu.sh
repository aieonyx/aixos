#!/bin/sh
# Copyright (c) 2026 Edison Lepiten / AIEONYX
# SPDX-License-Identifier: Apache-2.0
#
# PL-14: UART keyboard is the primary input channel.
# Run from a terminal emulator — type commands at the terminal,
# they are piped through -serial stdio to the shell_loop.
# virtio-keyboard-device remains for future GTK binding work.
echo "aiXos Phoenix - Sovereign Stack Initializing..."
echo "[PL-14] Type commands in THIS terminal. GTK window = display only."
echo ""

qemu-system-aarch64 \
    -machine virt \
    -cpu cortex-a72 \
    -m 512M \
    -vga none \
    -device ramfb \
    -display gtk \
    -serial stdio \
    -kernel boot/aixos.elf \
    -no-reboot \
    -d guest_errors
