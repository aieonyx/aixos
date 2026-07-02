# aiXos Phoenix

**Sovereign Desktop OS by AIEONYX**

## What it is

aiXos Phoenix is a sovereign desktop operating system built on a
formally verified seL4 microkernel. It is the first OS where the
application layer is written in AXONYX — a sovereign systems
programming language designed from the ground up for this stack.

## What it runs on

aarch64 hardware. Boots from USB in three modes:
Live, Persistent USB, and Install-to-Disk.
QEMU aarch64 verified during development.

## What makes it sovereign

Every component in the boot chain declares sovereignty:
identity is established by ARPi ceremony before anything else runs,
network packets are tagged and signed by AWP-Lite,
policy is enforced by BASTION before any user session begins,
and the desktop surface is rendered by HANIEL — no external
GUI toolkit, no proprietary driver stack.

## Components

- [AXONYX](https://github.com/aieonyx/AXON) — sovereign language
- [EdisonDB](https://github.com/aieonyx/edisondb) — sovereign database
- [HANIEL](https://github.com/aieonyx/haniel) — sovereign render engine

## License

Apache 2.0 — Copyright (c) 2026 Edison Lepiten / AIEONYX
