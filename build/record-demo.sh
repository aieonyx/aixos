#!/bin/sh
# Copyright (c) 2026 Edison Lepiten / AIEONYX
mkdir -p ~/nlnet-evidence
asciinema rec ~/nlnet-evidence/aixos-phoenix-lite-boot.cast \
    --command "bash build/run-qemu.sh" \
    --title "aiXos Phoenix Lite - Sovereign Boot Demo"
