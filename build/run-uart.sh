#!/bin/sh
# Copyright (c) 2026 Edison Lepiten / AIEONYX

qemu-system-aarch64 \
    -machine virt \
    -cpu cortex-a72 \
    -m 512M \
    -device ramfb \
    -nographic \
    -kernel boot/aixos.elf \
    -no-reboot
