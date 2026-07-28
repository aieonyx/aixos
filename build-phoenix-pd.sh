#!/bin/bash
# Copyright (c) 2026 Edison Lepiten / AIEONYX
# Build aixos-phoenix-pd with Microkit linker script
set -e

SDK="/home/edisonbl/aixos/microkit-sdk-1.4.1/board/qemu_virt_aarch64/release/lib"
LD="/home/edisonbl/aixos/microkit-phoenix.ld"
OUT="target/aarch64-unknown-none/microkit/aixos-phoenix-pd"

echo "[PD] Building Phoenix-Desktop PD with Microkit linker..."

# Temporarily use microkit config (no aixos-boot.ld)
cp .cargo/config.toml .cargo/config.toml.bak

cat > .cargo/config.toml << TOML
# Temporary — Microkit build for Phoenix-Desktop PD
[build]
target = "aarch64-unknown-none"

[target.aarch64-unknown-none]
rustflags = [
    "-C", "relocation-model=static",
]
TOML

# Build
cargo build \
    --manifest-path crates/aixos-phoenix-pd/Cargo.toml \
    --target aarch64-unknown-none \
    --profile microkit 2>&1 | \
    grep -E "^error|Compiling aixos-phoenix|Finished"

# Restore workspace config
cp .cargo/config.toml.bak .cargo/config.toml
rm .cargo/config.toml.bak

echo "[PD] Done: $OUT"
ls -la "$OUT"
