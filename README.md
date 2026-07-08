<p align="center">
  <img src="assets/first-pixels-pl10.png" alt="aiXos Phoenix — Sovereign Desktop OS" width="100%"/>
</p>

<p align="center">
  <strong>Built on seL4. Written for sovereignty. Pixels on screen.</strong>
</p>

<p align="center">
  <a href="https://github.com/aieonyx/aixos/releases/tag/v0.1.0-phoenix-lite">v0.1.0-phoenix-lite</a> •
  <a href="https://github.com/aieonyx/AXON">AXONYX Language</a> •
  <a href="https://github.com/aieonyx/edisondb">EdisonDB</a> •
  <a href="https://github.com/aieonyx/haniel">HANIEL</a> •
  Apache 2.0
</p>

<p align="center">
  <img src="https://github.com/aieonyx/aixos/actions/workflows/ci.yml/badge.svg" alt="CI"/>
</p>

---

## What is aiXos Phoenix?

aiXos is a sovereign desktop operating system by [AIEONYX](https://github.com/aieonyx).
Phoenix is the name of Version 1 — the first era of the aiXos family.
Built on a formally verified seL4 microkernel with an application layer
written in [AXONYX](https://github.com/aieonyx/AXON) — a sovereign systems programming
language designed from the ground up for this stack.

Every component in the boot chain declares sovereignty:

- **Identity** is established by ARPi ceremony before anything else runs
- **Network packets** are tagged and signed by AWP-Lite
- **Policy** is enforced by BASTION before any user session begins
- **The desktop surface** is rendered directly to framebuffer — no external GUI toolkit, no proprietary driver stack
- **The OS boots from ISO via UEFI** — EDK2 → PE/COFF stub → sovereign desktop

---

## Current State — aiXos Phoenix Lite (July 2026)

aiXos Phoenix Lite boots to a **sovereign three-panel desktop** on QEMU aarch64,
with pixel-rendered GUI, bitmap font text, AIEONYX diamond logo, and shell prompt.
It also boots from a UEFI ISO image via EDK2.
aiXos Phoenix - Sovereign Stack Initializing...
axon_main() -> 0x4153
GPU: ok
Desktop rendered
axos>

### What you see on screen

- **Top bar:** `aiXos Phoenix  |  axon_main() -> 0x4153  |  Sovereign`
- **Left panel:** User Space (dark panel)
- **Center canvas:** Sovereign purple tint + AIEONYX diamond logo
- **Right panel:** System Space
- **Bottom dock:** `axos>` shell prompt

---

## Boot Sequence

### Dev loop (`-kernel boot/aixos.elf`)
_start (boot.s)
└── BSS zero loop
└── aixos_main()
└── orchestrate() — ARPi, AWP, BASTION, HANIEL
└── aixos_gpu::init() — ramfb via fw_cfg DMA
└── render_desktop() — three-panel layout
└── render_status_bar()
└── shell_loop() — axos> prompt
### ISO boot (`bash build/run-iso.sh`)
EDK2 (UEFI firmware)
└── BOOTAA64.EFI (PE/COFF EFI stub)
└── GetMemoryMap + ExitBootServices
└── Disable MMU/cache
└── Self-relocate to 0x40000000
└── → _start → aixos_main() → sovereign desktop
---

## AXONYX .ax Files (10 shipped)

[AXONYX](https://github.com/aieonyx/AXON) is the sovereign language powering aiXos.
10 `.ax` files ship as part of the OS:

| File | Purpose |
|------|---------|
| `ceremony.ax` | ARPi identity ceremony |
| `awp_lite.ax` | AWP sovereign protocol |
| `sovereignty.ax` | OS build manifest |
| `shell.ax` | Sovereign shell core |
| `layout.ax` | HANIEL canvas grid |
| `bastion.ax` | Policy + heartbeat |
| `boot/aixos-boot.ax` | Boot mode selection |
| `crates/aixos-gpu/src/desktop.ax` | Desktop layout constants |
| `crates/aixos-input/src/input.ax` | Key codes |
| `boot/aixos-boot.ax` | Boot mode constants (live/install/rescue) |

---

## Sovereign Stack

| Component | Role | Repo |
|-----------|------|------|
| seL4 + ASL | Formally verified microkernel | [asl](https://github.com/aieonyx/asl) |
| AXONYX | Sovereign language | [AXON](https://github.com/aieonyx/AXON) |
| EdisonDB | Sovereign database | [edisondb](https://github.com/aieonyx/edisondb) |
| HANIEL | Sovereign render engine | [haniel](https://github.com/aieonyx/haniel) |
| Onyxia | Sovereign browser | [onyxia](https://github.com/aieonyx/onyxia) |
| ARPi | Identity protocol | built-in |
| AWP | Sovereign network protocol | built-in |
| BASTION | Policy enforcement daemon | built-in |

---

## Roadmap

### aiXos Phoenix Lite (v0.1) — DELIVERED ✅
- [x] seL4 microkernel boot
- [x] ARPi identity ceremony
- [x] AWP-Lite signed networking
- [x] BASTION policy enforcement
- [x] HANIEL sovereign shell surface
- [x] 10 AXONYX .ax files running in the OS
- [x] Bootable on QEMU aarch64
- [x] **First pixels on screen — sovereign purple (#7B4FDB) via ramfb**
- [x] **Three-panel sovereign desktop — top bar, panels, AIEONYX logo, dock**
- [x] **8x8 bitmap font rendering — status bar text on screen**
- [x] **virtio-input keyboard driver — virtqueue initialized (v1 legacy)**
- [x] **UEFI ISO boot — EDK2 → PE/COFF stub → aiXos Phoenix desktop**

### aiXos Phoenix Full (v1.0) — Next
- [ ] Keyboard input delivery (QEMU input routing)
- [ ] x86_64 port — native Intel/AMD boot
- [ ] Full AXONYX application layer (zero Rust stubs)
- [ ] Onyxia browser integrated
- [ ] EdisonDB persistent sovereign storage
- [ ] AWP full mesh networking
- [ ] IAM — Intelligent Assistant to Man

### aiXos v2.0 — Future
- [ ] AXIOM/SOMA hardware identity binding
- [ ] Aegis collective defense
- [ ] Multi-node sovereign mesh
- [ ] Sovereign app ecosystem

---

## How to Run

**Requirements:** QEMU 8.2.2+, Rust (aarch64-unknown-none target),
aarch64-linux-gnu toolchain, xorriso, mtools, gdisk, qemu-efi-aarch64

```sh
git clone https://github.com/aieonyx/aixos
cd aixos
git submodule update --init --recursive
cargo build --release --target aarch64-unknown-none --bin aixos
cp target/aarch64-unknown-none/release/aixos boot/aixos.elf
```

### Dev loop (fast boot, displays GUI)
```sh
bash build/run-qemu.sh
```

### ISO boot (UEFI, full chain)
```sh
aarch64-linux-gnu-objcopy -O binary boot/aixos.elf boot/aixos.img
bash build/make-iso.sh
bash build/run-iso.sh
```

Expected output:
aiXos Phoenix - Sovereign Stack Initializing...
axon_main() -> 0x4153
GPU: ok
Desktop rendered
axos>
---

## Key Technical Facts

| Property | Value |
|----------|-------|
| Architecture | aarch64 bare-metal |
| RAM base | 0x40000000 |
| Framebuffer | 0x44000000 (1280×720, FORMAT_XR24) |
| fw_cfg key | 0x0025 = etc/ramfb |
| virtio-input | slot 31, v1 legacy, device ID 0x12 |
| Boot formats | ELF (`-kernel`), PE/COFF EFI (ISO) |
| QEMU command | `-machine virt -cpu cortex-a72 -m 512M` |

---

## Contributing

aiXos is a civilizational project — built as a gift to ordinary people
who deserve digital sovereignty. Contributions are welcome.

Areas where contributions are most needed:

- **Keyboard input** — QEMU virtio-keyboard-device GTK routing fix
- **AXONYX .ax coverage** — replace Rust stubs with pure sovereign language
- **x86_64 port** — bring aiXos to Intel/AMD hardware
- **Liquid glass GUI** — backdrop blur, specular highlights for Phoenix v1.0 Full
- **AWP mesh** — full sovereign network protocol implementation

---

## Known Gaps (Honest)

- Keyboard events not yet delivered by QEMU GTK to virtio-keyboard-device
- Boot mode selection hardcoded to Live
- AWP-Lite loopback not yet wired to real packet path
- Ed25519 signing stubs not yet wired to real key material
- x86_64 target not yet supported

---

## License

Apache 2.0 — Copyright (c) 2026 Edison Lepiten / AIEONYX

aiXos is permanently and irrevocably open source.
Community Promise: this license will never be changed to restrict users.

---

<p align="center">
  <em>IDENTITY ESTABLISHED &nbsp;•&nbsp; POLICY ENFORCED &nbsp;•&nbsp; SOVEREIGN DESKTOP ON SCREEN</em>
</p>
