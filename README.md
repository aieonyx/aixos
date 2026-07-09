<p align="center">
  <img src="assets/phoenix.jpg" alt="aiXos Phoenix - The Sovereign Desktop OS" width="100%"/>
</p>

<h1 align="center">aiXos Phoenix</h1>

<h3 align="center">The Sovereign Desktop OS by AIEONYX</h3>

<p align="center">
  <strong>Built in Rust. No Linux. No POSIX. Sovereignty from first instruction.</strong>
</p>

<p align="center">
  <a href="https://github.com/aieonyx/aixos/releases/tag/v0.1.0-phoenix-lite">v0.1.0-phoenix-lite</a>
  •
  <a href="https://github.com/aieonyx/AXON">AXONYX Language</a>
  •
  <a href="https://github.com/aieonyx/edisondb">EdisonDB</a>
  •
  <a href="https://github.com/aieonyx/haniel">HANIEL</a>
  •
  <a href="LICENSE">Apache-2.0</a>
</p>

<p align="center">
  <img src="https://github.com/aieonyx/aixos/actions/workflows/ci.yml/badge.svg" alt="CI"/>
</p>

<p align="center">
  <strong>IDENTITY ESTABLISHED • PROOF EARNED • SOVEREIGN DESKTOP ON SCREEN</strong>
</p>

<hr>

## What is aiXos Phoenix?

**aiXos** is a sovereign desktop operating system developed by **AIEONYX** — built from scratch in Rust on AArch64, with no Linux kernel, no POSIX layer, and no external GUI toolkit.

**Phoenix** is the first era of aiXos: a bootable, security-focused desktop OS with a sovereign identity layer, live system panels, a working shell, and a floating window primitive — all rendered directly to the framebuffer.

> **The user should own the machine, the identity, the data, and the rules of execution.**

<hr>

## Current State — Phoenix Lite v0.1.0

aiXos Phoenix Lite boots on **QEMU aarch64** via EDK2 UEFI and displays a fully live sovereign desktop.

### What works today

| Feature | Status |
|---------|--------|
| Bare-metal AArch64 boot via EDK2 + PE/COFF stub | ✅ |
| ramfb framebuffer — 1280×720 FORMAT_XR24 | ✅ |
| Three-panel sovereign desktop (Identity / Canvas / System) | ✅ |
| Sovereign proof `0x4153 [SOVEREIGN]` — earned, not hardcoded | ✅ |
| ARPi identity ceremony — hardware-derived node ID | ✅ |
| AWP-Lite loopback confirmed | ✅ |
| All 5 Protection Domains live (GENESIS, ARPi, AWP, Shell, BASTION) | ✅ |
| virtio-input keyboard driver — GTK window + UART both working | ✅ |
| Live left panel — Node ID, ARPi state, Proof, Boot mode, Arch, Kernel | ✅ |
| Live right panel — AWP, EdisonDB, Input, Display, HANIEL, BASTION | ✅ |
| Shell with 10 commands | ✅ |
| Sovereign floating window primitive | ✅ |

### Shell commands

```
help  clear  version  sovereignty  node-id  awp-status  mem  window  close  reboot
```

### Boot output

```
aiXos Phoenix - Sovereign Stack Initializing...
axon_main() -> 0x4153 [SOVEREIGN]
GPU: ok
Desktop rendered
Input: virtio+uart
axos>
```

### On screen

**Status bar:** `aiXos Phoenix | axon_main() -> 0x4153 | Sovereign`

**Left panel (IDENTITY):**
- Node: `a1e04851 40100001` (hardware-derived from RAM base + fw_cfg)
- ARPi: `active`
- Proof: `0x4153 [OK]`
- Boot: `Live` | Arch: `aarch64` | Kernel: `Phoenix v0.1`

**Right panel (SYSTEM):**
- AWP: `lite-live` | EdisonDB: `stub` | Input: `virtio+uart`
- Display: `ramfb 1280x720` | HANIEL: `stub` | BASTION: `lite-live`

**Window primitive:** type `window` at the shell to open a floating sovereign node info window. Type `close` to dismiss.

<hr>

## Sovereign Stack

| Component | Role | Status |
|-----------|------|--------|
| aiXos Phoenix | Sovereign desktop OS | **Active — PL-20** |
| ARPi | Identity ceremony protocol | Hardware-derived node ID live |
| AWP-Lite | Sovereign network protocol | Loopback confirmed |
| BASTION | Policy enforcement daemon | Shell loop active |
| GENESIS PD | Kernel boot proof | Execution is proof |
| EdisonDB | Sovereign database layer | Stub — integration planned |
| HANIEL | Sovereign render engine | Stub — bare-metal port planned |
| AXONYX | Sovereign systems language | Compiler complete, OS integration planned |
| ASL-seL4 | Sovereign Linux + microkernel | v1.0.0-asl complete, separate repo |
| Onyxia | Sovereign browser | v1.1.0 complete, separate repo |

<hr>

## How to Run

### Requirements

- QEMU 8.2.2+
- Rust 1.94.1 (pinned via `rust-toolchain.toml`)
- `aarch64-linux-gnu` cross toolchain
- `mtools`, `gdisk`, `qemu-efi-aarch64`

### Build and run (ISO boot — recommended)

```bash
git clone https://github.com/aieonyx/aixos
cd aixos

# Build ELF + PE image
bash build/build-iso.sh

# Package ISO
bash build/make-iso.sh

# Boot via EDK2
bash build/run-iso.sh
```

Then type commands at the terminal. The GTK window is the display.

### Quick commands to try

```
version       — show OS version
sovereignty   — display S4+i doctrine
node-id       — show hardware-derived node identity
awp-status    — AWP protocol state
window        — open sovereign node info window
close         — dismiss window
mem           — memory map
```

<hr>

## Boot Sequence

```
EDK2 UEFI firmware
  └── BOOTAA64.EFI (PE/COFF EFI stub)
      ├── GetMemoryMap / ExitBootServices
      ├── Disable MMU/cache
      ├── Self-relocate to 0x40000000
      └── _start
          ├── Zero BSS
          └── aixos_main()
              ├── orchestrate() — 5 PD handshake sequence
              │   ├── GenesisPd::handshake()   → true (execution is proof)
              │   ├── ArpiCeremony::handshake() → true (node_id derived)
              │   ├── AwpLite::handshake()      → true (loopback confirmed)
              │   ├── SovereignShell::handshake() → true (desktop live)
              │   └── BastionPd::handshake()    → true (shell loop active)
              │   → proof = 0x4153 [SOVEREIGN]
              ├── aixos_gpu::init() — ramfb via fw_cfg DMA
              ├── render_desktop() + render_left_panel() + render_right_panel()
              ├── aixos_input::init() — virtio-input v1 legacy, slot 31
              └── shell_loop() — UART + virtio keyboard, both active
```

<hr>

## Key Technical Facts

| Property | Value |
|----------|-------|
| Architecture | AArch64 bare-metal (`no_std`) |
| Rust toolchain | 1.94.1 (pinned) |
| RAM base | `0x40000000` |
| Framebuffer | `0x44000000` — 1280×720 FORMAT_XR24 |
| fw_cfg key | `0x0025` (etc/ramfb) |
| virtio-input slot | 31 (`0x1f`), device ID `0x12` |
| virtio version | v1 legacy MMIO |
| Boot path | EDK2 → PE/COFF stub → `_start` → `aixos_main()` |
| Sovereign proof | `0x4153` = `AS` in ASCII (AIEONYX Sovereign) |
| Node ID | Hardware-derived: RAM base XOR fw_cfg XOR seed |
| Tests | 38 passing, 0 failing |

<hr>

## Known Gaps (honest)

- EdisonDB not yet integrated as bare-metal storage (std/tokio dependency)
- HANIEL bare-metal render engine not yet ported (Tauri dependency)
- AWP not yet on a real packet path (loopback only)
- Mouse/cursor not yet implemented
- x86_64 target not yet supported
- Node ID is hardware-derived constant, not a cryptographic Ed25519 keypair

<hr>

## Roadmap

### Phoenix Lite — v0.1 ✅
- Bare-metal boot, ramfb display, three-panel desktop
- virtio-input keyboard (GTK + UART)
- All 5 PDs wired — sovereign proof earned
- Live identity panels, shell, window primitive

### Phoenix v0.2 — In Progress
- ⬜ Mouse cursor (virtio-mouse driver)
- ⬜ EdisonDB bare-metal bridge
- ⬜ HANIEL bare-metal render surface
- ⬜ AWP packet path (virtio-net)
- ⬜ Multi-window management

### Phoenix v1.0 — Future
- ⬜ Real Ed25519 ARPi keypair ceremony
- ⬜ Onyxia browser integration
- ⬜ Full AXONYX application layer
- ⬜ x86_64 port
- ⬜ Liquid-glass sovereign desktop aesthetic

<hr>

## License

**Apache-2.0**

Copyright © 2026 Edison Lepiten / AIEONYX

<hr>

<div align="center">

**IDENTITY ESTABLISHED • PROOF EARNED • SOVEREIGN DESKTOP ON SCREEN**

*aiXos Phoenix — The Sovereign Desktop OS by AIEONYX*

</div>
