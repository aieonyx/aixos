# aiXos Phoenix — v1.0.0

**First sovereign desktop OS release by AIEONYX.**

Bare-metal AArch64. No Linux. No POSIX. Sovereignty from first instruction.

---

## What ships in v1.0.0

### Boot splash
- Full-screen AIEONYX identity splash on every boot
- Large sovereign diamond, wordmark, version, proof hash, teal progress bar
- ~2.5s hold then transitions to desktop

### Sovereign desktop
- Unified top bar: `[Node] [Shell] [EDB] [Set] [Brw]` — five sovereign icons
- Status bar: `aiXos Phoenix : axon_main() -> 0x4153 : Sovereign`
- AIEONYX diamond centered on canvas
- Taskbar with live window indicators

### Window system — 5 simultaneous windows
| Kind | Title | Access |
|------|-------|--------|
| 0 | Sovereign Node | `[Node]` icon or `window` command |
| 1 | Shell | `[Shell]` icon |
| 2 | EdisonDB Store | `[EDB]` icon or `db` command |
| 3 | Settings | `[Set]` icon or `settings` command |
| 4 | EDB Browser | `[Brw]` icon or `browse` command |

All windows: drag, resize, close, focus. Shell window: full typed input.

### EDB Browser (kind=4)
- Live navigable list of all EdisonDB entries
- Tier badge (C/P/N), key label, hex value per row
- Up/Down arrow cursor navigation with scroll
- `edb>` input line: type `put <key> <value>` to write entries

### Shell commands
### EdisonDB — live bare-metal store
- 32-entry sovereign key-value store, three tiers: Critical / Personal / Noise
- Boot entries: `boot:proof`, `boot:node_id`, `boot:desktop_ready`
- `entry_count()` + `entry_at(i)` API for browser

### Sovereign proof
`0x4153` = `AS` in ASCII — AIEONYX Sovereign. Earned, not hardcoded.

---

## How to run

```bash
git clone https://github.com/aieonyx/aixos
cd aixos
bash build/build-iso.sh
bash build/make-iso.sh
bash build/run-iso.sh
```

Requirements: QEMU 8.2+, Rust 1.94.1, `aarch64-linux-gnu` toolchain,
`mtools`, `gdisk`, `qemu-efi-aarch64`.

---

## Build gate

- Rust toolchain: 1.94.1 (pinned)
- Target: `aarch64-unknown-none`
- Tests: 42 passing, 0 failing
- CI: green

---

## Known gaps (honest)

- AWP not yet on a real packet path (loopback only)
- Node ID is hardware-derived, not Ed25519 keypair
- x86_64 target not yet supported
- Liquid-glass aesthetic deferred to v1.1

---

## License

Apache-2.0 — Copyright © 2026 Edison Lepiten / AIEONYX
