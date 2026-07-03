# aiXos Phoenix Lite — v0.1.0-phoenix-lite

First public release of aiXos Phoenix Lite by AIEONYX.

## What is included

A bootable sovereign desktop OS binary for aarch64, built on a
formally verified seL4 microkernel, with application logic written
in AXONYX — a sovereign systems programming language.

## AXONYX .ax files shipped

- ceremony.ax — ARPi identity ceremony
- awp_lite.ax — AWP sovereign protocol
- sovereignty.ax — OS build manifest
- shell.ax — sovereign shell core
- layout.ax — HANIEL canvas grid
- bastion.ax — policy enforcement + heartbeat
- boot/aixos-boot.ax — boot mode selection

## Boot sequence

GENESIS -> ARPi identity -> AWP-Lite -> Sovereign Shell
-> BASTION policy -> render_banner -> render_proof -> axos>

## How to run

```sh
bash build/build-iso.sh
bash build/run-qemu.sh
# Press Ctrl+A then X to exit QEMU
```

## Known gaps

- HANIEL pixel output pending full submodule wiring
- Boot mode selection hardcoded to Live
- Ed25519 signing stubs not yet wired to real key material
- AWP-Lite loopback not yet wired to real packet path

## License

Apache 2.0 — Copyright (c) 2026 Edison Lepiten / AIEONYX
