#!/bin/bash
# Copyright (c) 2026 Edison Lepiten / AIEONYX
# SPDX-License-Identifier: Apache-2.0
# build-seL4.sh — Build aiXos Phoenix under ASL-seL4 mKernel

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MICROKIT_SDK="${MICROKIT_SDK:-}"

# Use standalone asl repo (has correct .cargo/config.toml for Microkit)
# Falls back to submodule if standalone not present
if [ -d "$HOME/asl" ] && [ -f "$HOME/asl/build-phoenix-v2.sh" ]; then
    ASL_DIR="$HOME/asl"
else
    ASL_DIR="$SCRIPT_DIR/asl"
fi

RED='\033[0;31m'; GREEN='\033[0;32m'; CYAN='\033[0;36m'
GOLD='\033[0;33m'; NC='\033[0m'

info() { echo -e "${CYAN}[BUILD]${NC} $*"; }
ok()   { echo -e "${GREEN}[OK]${NC}    $*"; }
gold() { echo -e "${GOLD}[AIEONYX]${NC} $*"; }
die()  { echo -e "${RED}[FAIL]${NC}  $*" >&2; exit 1; }

echo ""
gold "aiXos Phoenix — Sovereign Build under ASL-seL4 mKernel"
gold "S4+i: Security Sovereignty Simplicity Speed +Intelligence"
echo ""

[ -z "$MICROKIT_SDK" ] && die "MICROKIT_SDK not set."
[ ! -f "$MICROKIT_SDK/bin/microkit" ] && die "Microkit tool not found."

# Step 1: Build Phoenix-Desktop PD
info "Step 1: Building Phoenix-Desktop PD (real aiXos sovereign desktop)..."
cd "$SCRIPT_DIR"
bash build-phoenix-pd.sh
ok "Phoenix-Desktop PD built: $(ls -lh target/aarch64-unknown-none/microkit/aixos-phoenix-pd | awk '{print $5}')"

# Step 2: Build ASL-seL4 PDs + assemble image
info "Step 2: Building ASL-seL4 mKernel via $ASL_DIR..."
cd "$ASL_DIR"
export MICROKIT_SDK
bash build-phoenix-v2.sh
ok "Microkit image: $ASL_DIR/build-v2/phoenix-v2.img ($(ls -lh $ASL_DIR/build-v2/phoenix-v2.img | awk '{print $5}'))"

# Store image path for run-seL4.sh
echo "$ASL_DIR/build-v2/phoenix-v2.img" > "$SCRIPT_DIR/.seL4-image-path"

echo ""
gold "Build complete — aiXos Phoenix under ASL-seL4 mKernel"
gold "Boot: bash run-seL4.sh"
echo ""
