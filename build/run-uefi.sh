#!/bin/sh
# Copyright (c) 2026 Edison Lepiten / AIEONYX
# UEFI boot - firmware handles virtio init

qemu-system-aarch64 \
    -machine virt,highmem=off \
    -cpu cortex-a53 \
    -m 1G \
    -bios /usr/share/qemu-efi-aarch64/QEMU_EFI.fd \
    -device virtio-gpu-device \
    -device virtio-keyboard-device \
    -display sdl \
    -serial stdio \
    -drive if=none,file=boot/aixos.elf,id=hd0,format=raw \
    -device virtio-blk-device,drive=hd0 \
    -no-reboot
