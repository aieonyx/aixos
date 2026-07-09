<<p align="center">
  <img src="assets/phoenix.jpg" alt="aiXos Phoenix - The Sovereign Desktop OS" width="100%"/>
</p>

<h1 align="center">aiXos Phoenix</h1>

<h3 align="center">The Sovereign Desktop OS by AIEONYX</h3>

<p align="center">
  <strong>Built on seL4. Written for sovereignty. Pixels on screen.</strong>
</p>

<p align="center">
  <a href="https://github.com/aieonyx/aixos/releases/tag/v0.1.0-phoenix-lite">
    v0.1.0-phoenix-lite
  </a>
  •
  <a href="https://github.com/aieonyx/AXON">
    AXONYX Language
  </a>
  •
  <a href="https://github.com/aieonyx/edisondb">
    EdisonDB
  </a>
  •
  <a href="https://github.com/aieonyx/haniel">
    HANIEL
  </a>
  •
  <a href="LICENSE">
    Apache-2.0
  </a>
</p>

<p align="center">
  <img src="https://github.com/aieonyx/aixos/actions/workflows/ci.yml/badge.svg" alt="CI"/>
</p>

<p align="center">
  <strong>IDENTITY ESTABLISHED • POLICY ENFORCED • SOVEREIGN DESKTOP ON SCREEN</strong>
</p>

<hr>

</div>

<hr>

<h2>What is aiXos Phoenix?</h2>

<p>
  <b>aiXos</b> is a sovereign desktop operating system developed by
  <b>AIEONYX</b>.
</p>

<p>
  <b>Phoenix</b> is the first era of the AIEONYX aiXos family: a bootable,
  security-focused desktop OS built on the formally verified
  <b>seL4 microkernel</b>, with a growing application layer written in
  <b>AXONYX</b> — AIEONYX’s sovereign systems programming language.
</p>

<p>
  aiXos is not designed to be just another Linux distribution or desktop skin.
  It is an attempt to build a user-respecting operating system stack from the
  ground up, where identity, policy, rendering, storage, networking, and
  applications are designed around one principle:
</p>

<blockquote>
  <b>The user should own the machine, the identity, the data, and the rules of execution.</b>
</blockquote>

<hr>

<h2>Why aiXos Exists</h2>

<p>
  Modern computing is powerful, but much of it is no longer truly owned by the
  user. Identity is outsourced. Policy is hidden. Data is centralized. Trust is
  often assumed instead of verified.
</p>

<p>
  aiXos is AIEONYX’s answer to that problem.
</p>

<p>The project explores a different model of personal computing:</p>

<ul>
  <li><b>Sovereign identity</b> before user space begins</li>
  <li><b>Policy enforcement</b> before the session starts</li>
  <li><b>Local-first execution</b> with no required cloud dependency</li>
  <li><b>Framebuffer-rendered desktop surface</b> without an external GUI toolkit</li>
  <li><b>AXONYX-based application layer</b> designed specifically for sovereign systems</li>
  <li><b>EdisonDB storage layer</b> for user-owned persistent data</li>
  <li><b>HANIEL rendering engine</b> for direct pixel output</li>
  <li><b>AWP networking model</b> for future signed sovereign mesh communication</li>
</ul>

<p>
  <b>Phoenix Lite</b> is the first working foundation of that vision.
</p>

<hr>

<h2>Current State — Phoenix Lite</h2>

<p>
  <b>aiXos Phoenix Lite</b> currently boots on <b>QEMU aarch64</b> and displays
  a sovereign desktop surface rendered directly to the framebuffer.
</p>

<p>The current milestone includes:</p>

<ul>
  <li>Bootable aarch64 bare-metal OS image</li>
  <li>UEFI ISO boot path through EDK2 and PE/COFF EFI stub</li>
  <li>seL4-oriented boot structure</li>
  <li>ARPi identity ceremony</li>
  <li>AWP-Lite sovereign protocol layer</li>
  <li>BASTION policy enforcement path</li>
  <li>HANIEL-rendered desktop surface</li>
  <li>ramfb framebuffer output through QEMU <code>fw_cfg</code></li>
  <li>Three-panel sovereign desktop layout</li>
  <li>AIEONYX diamond logo rendered on screen</li>
  <li>8x8 bitmap font text rendering</li>
  <li>Bottom shell prompt: <code>axos&gt;</code></li>
  <li>virtio-input keyboard driver milestone initialized</li>
</ul>

<p><b>Expected screen output:</b></p>

<pre><code>aiXos Phoenix - Sovereign Stack Initializing...
axon_main() -&gt; 0x4153
GPU: ok
Desktop rendered
axos&gt;</code></pre>

<hr>

<h2>What You See on Screen</h2>

<p>
  Phoenix Lite renders a minimal sovereign desktop surface:
</p>

<table>
  <thead>
    <tr>
      <th>Area</th>
      <th>Description</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td>Top Bar</td>
      <td><code>aiXos Phoenix</code> status line with boot identity</td>
    </tr>
    <tr>
      <td>Left Panel</td>
      <td>User Space panel</td>
    </tr>
    <tr>
      <td>Center Canvas</td>
      <td>Sovereign purple desktop surface with AIEONYX diamond logo</td>
    </tr>
    <tr>
      <td>Right Panel</td>
      <td>System Space panel</td>
    </tr>
    <tr>
      <td>Bottom Dock</td>
      <td><code>axos&gt;</code> sovereign shell prompt</td>
    </tr>
  </tbody>
</table>

<p>
  The current visual direction is intentionally minimal: a clean sovereign
  desktop foundation before adding richer GUI effects in Phoenix Full.
</p>

<hr>

<h2>Boot Sequence</h2>

<h3>Development Boot Path</h3>

<p>Fast development loop using QEMU <code>-kernel</code>:</p>

<pre><code>_start
  └── BSS zero loop
      └── aixos_main()
          └── orchestrate()
              ├── ARPi identity ceremony
              ├── AWP-Lite protocol layer
              ├── BASTION policy path
              ├── HANIEL desktop surface
              ├── aixos_gpu::init()
              │   └── ramfb via fw_cfg DMA
              ├── render_desktop()
              │   ├── top status bar
              │   ├── left user panel
              │   ├── center canvas
              │   ├── right system panel
              │   └── bottom shell dock
              └── shell_loop()
                  └── axos&gt;</code></pre>

<h3>ISO Boot Path</h3>

<p>UEFI ISO boot chain:</p>

<pre><code>EDK2 UEFI firmware
  └── BOOTAA64.EFI
      └── PE/COFF EFI stub
          ├── GetMemoryMap
          ├── ExitBootServices
          ├── Disable MMU/cache
          ├── Self-relocate to 0x40000000
          └── Jump to _start
              └── aixos_main()
                  └── sovereign desktop</code></pre>

<hr>

<h2>AXONYX Source Layer</h2>

<p>
  <b>AXONYX</b> is the sovereign systems language layer powering the aiXos
  application model.
</p>

<p>
  Phoenix Lite ships AXONYX source files across identity, networking, policy,
  shell, layout, GPU, input, and boot selection areas.
</p>

<table>
  <thead>
    <tr>
      <th>File</th>
      <th>Purpose</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td><code>ceremony.ax</code></td>
      <td>ARPi identity ceremony</td>
    </tr>
    <tr>
      <td><code>awp_lite.ax</code></td>
      <td>AWP-Lite sovereign protocol</td>
    </tr>
    <tr>
      <td><code>sovereignty.ax</code></td>
      <td>OS build and sovereignty manifest</td>
    </tr>
    <tr>
      <td><code>shell.ax</code></td>
      <td>Sovereign shell core</td>
    </tr>
    <tr>
      <td><code>layout.ax</code></td>
      <td>HANIEL canvas layout model</td>
    </tr>
    <tr>
      <td><code>bastion.ax</code></td>
      <td>Policy and heartbeat logic</td>
    </tr>
    <tr>
      <td><code>boot/aixos-boot.ax</code></td>
      <td>Boot mode selection</td>
    </tr>
    <tr>
      <td><code>crates/aixos-gpu/src/desktop.ax</code></td>
      <td>Desktop layout constants</td>
    </tr>
    <tr>
      <td><code>crates/aixos-input/src/input.ax</code></td>
      <td>Input and key code definitions</td>
    </tr>
  </tbody>
</table>

<p>
  The long-term goal is to move the full aiXos application layer into AXONYX
  and reduce Rust stubs as the language matures.
</p>

<hr>

<h2>Sovereign Stack</h2>

<table>
  <thead>
    <tr>
      <th>Component</th>
      <th>Role</th>
      <th>Status</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td>seL4 + ASL</td>
      <td>Microkernel foundation</td>
      <td>Integrated</td>
    </tr>
    <tr>
      <td>AXONYX</td>
      <td>Sovereign systems language</td>
      <td>Integrated and expanding</td>
    </tr>
    <tr>
      <td>EdisonDB</td>
      <td>Sovereign database layer</td>
      <td>Planned integration</td>
    </tr>
    <tr>
      <td>HANIEL</td>
      <td>Sovereign render engine</td>
      <td>Rendering desktop surface</td>
    </tr>
    <tr>
      <td>Onyxia</td>
      <td>Sovereign browser</td>
      <td>Planned integration</td>
    </tr>
    <tr>
      <td>ARPi</td>
      <td>Identity ceremony protocol</td>
      <td>Built in</td>
    </tr>
    <tr>
      <td>AWP-Lite</td>
      <td>Sovereign network protocol layer</td>
      <td>Early implementation</td>
    </tr>
    <tr>
      <td>BASTION</td>
      <td>Policy enforcement daemon/path</td>
      <td>Built in</td>
    </tr>
    <tr>
      <td>EQL Keyboard Driver</td>
      <td>virtio-input keyboard path</td>
      <td>Driver milestone complete</td>
    </tr>
  </tbody>
</table>

<hr>

<h2>Roadmap</h2>

<h3>Phoenix Lite — v0.1</h3>

<p><b>Delivered foundation:</b></p>

<ul>
  <li>✅ seL4-oriented microkernel boot path</li>
  <li>✅ ARPi identity ceremony</li>
  <li>✅ AWP-Lite sovereign protocol layer</li>
  <li>✅ BASTION policy enforcement path</li>
  <li>✅ HANIEL sovereign desktop surface</li>
  <li>✅ AXONYX <code>.ax</code> files integrated into the OS tree</li>
  <li>✅ Bootable on QEMU aarch64</li>
  <li>✅ First pixels through ramfb</li>
  <li>✅ Sovereign purple desktop surface</li>
  <li>✅ Three-panel desktop layout</li>
  <li>✅ AIEONYX diamond logo rendered on screen</li>
  <li>✅ 8x8 bitmap font rendering</li>
  <li>✅ Shell prompt displayed on screen</li>
  <li>✅ virtio-input keyboard driver milestone</li>
  <li>✅ UEFI ISO boot path through EDK2 and PE/COFF stub</li>
</ul>

<h3>Phoenix Full — v1.0</h3>

<p><b>Next major target:</b></p>

<ul>
  <li>⬜ Keyboard input delivery into the shell</li>
  <li>⬜ QEMU input routing fix for virtio keyboard events</li>
  <li>⬜ x86_64 port for Intel/AMD machines</li>
  <li>⬜ Full AXONYX application layer with fewer Rust stubs</li>
  <li>⬜ Onyxia browser integration</li>
  <li>⬜ EdisonDB persistent sovereign storage</li>
  <li>⬜ Full AWP sovereign mesh networking</li>
  <li>⬜ IAM — Intelligent Assistant to Man</li>
  <li>⬜ Liquid-glass desktop direction for Phoenix Full</li>
  <li>⬜ Improved desktop interaction model</li>
</ul>

<h3>aiXos v2.0</h3>

<p><b>Future direction:</b></p>

<ul>
  <li>⬜ AXIOM/SOMA hardware identity binding</li>
  <li>⬜ Aegis collective defense model</li>
  <li>⬜ Multi-node sovereign mesh</li>
  <li>⬜ Sovereign application ecosystem</li>
  <li>⬜ Hardware-backed user identity and policy chain</li>
  <li>⬜ Local-first intelligent computing environment</li>
</ul>

<hr>

<h2>How to Run</h2>

<h3>Requirements</h3>

<p>You will need:</p>

<ul>
  <li>QEMU 8.2.2 or newer</li>
  <li>Rust with <code>aarch64-unknown-none</code> target</li>
  <li><code>aarch64-linux-gnu</code> toolchain</li>
  <li><code>xorriso</code></li>
  <li><code>mtools</code></li>
  <li><code>gdisk</code></li>
  <li><code>qemu-efi-aarch64</code></li>
</ul>

<h3>Build</h3>

<pre><code>git clone https://github.com/aieonyx/aixos
cd aixos
git submodule update --init --recursive

cargo build --release --target aarch64-unknown-none --bin aixos
cp target/aarch64-unknown-none/release/aixos boot/aixos.elf</code></pre>

<h3>Run Development Boot</h3>

<p>Fast boot path for development:</p>

<pre><code>bash build/run-qemu.sh</code></pre>

<h3>Run ISO Boot</h3>

<p>Full UEFI ISO boot path:</p>

<pre><code>aarch64-linux-gnu-objcopy -O binary boot/aixos.elf boot/aixos.img

bash build/make-iso.sh
bash build/run-iso.sh</code></pre>

<p><b>Expected output:</b></p>

<pre><code>aiXos Phoenix - Sovereign Stack Initializing...
axon_main() -&gt; 0x4153
GPU: ok
Desktop rendered
axos&gt;</code></pre>

<hr>

<h2>Key Technical Facts</h2>

<table>
  <thead>
    <tr>
      <th>Property</th>
      <th>Value</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td>Architecture</td>
      <td>aarch64 bare-metal</td>
    </tr>
    <tr>
      <td>RAM Base</td>
      <td><code>0x40000000</code></td>
    </tr>
    <tr>
      <td>Framebuffer</td>
      <td><code>0x44000000</code></td>
    </tr>
    <tr>
      <td>Display Mode</td>
      <td><code>1280×720</code></td>
    </tr>
    <tr>
      <td>Pixel Format</td>
      <td><code>FORMAT_XR24</code></td>
    </tr>
    <tr>
      <td>ramfb Source</td>
      <td>QEMU <code>fw_cfg</code></td>
    </tr>
    <tr>
      <td>fw_cfg Key</td>
      <td><code>0x0025 = etc/ramfb</code></td>
    </tr>
    <tr>
      <td>virtio-input Slot</td>
      <td><code>31</code></td>
    </tr>
    <tr>
      <td>virtio-input Mode</td>
      <td>v1 legacy</td>
    </tr>
    <tr>
      <td>virtio-input Device ID</td>
      <td><code>0x12</code></td>
    </tr>
    <tr>
      <td>Boot Format</td>
      <td>ELF through QEMU <code>-kernel</code></td>
    </tr>
    <tr>
      <td>ISO Boot Format</td>
      <td>PE/COFF EFI through EDK2</td>
    </tr>
    <tr>
      <td>QEMU Machine</td>
      <td><code>virt</code></td>
    </tr>
    <tr>
      <td>CPU</td>
      <td><code>cortex-a72</code></td>
    </tr>
    <tr>
      <td>Memory</td>
      <td><code>512M</code></td>
    </tr>
  </tbody>
</table>

<hr>

<h2>Contributing</h2>

<p>
  aiXos is a civilizational open-source project built for people who believe
  ordinary users deserve digital sovereignty.
</p>

<p>Contributions are welcome from:</p>

<ul>
  <li>Systems programmers</li>
  <li>Rust developers</li>
  <li>OS developers</li>
  <li>seL4 researchers</li>
  <li>Compiler and language developers</li>
  <li>GUI and rendering engineers</li>
  <li>Security engineers</li>
  <li>Designers interested in sovereign desktop UX</li>
  <li>Builders who believe computing should be user-owned again</li>
</ul>

<p><b>Areas where help is especially valuable:</b></p>

<table>
  <thead>
    <tr>
      <th>Area</th>
      <th>Needed Work</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td>Keyboard Input</td>
      <td>Complete QEMU GTK routing and shell delivery</td>
    </tr>
    <tr>
      <td>AXONYX Coverage</td>
      <td>Replace Rust stubs with AXONYX source</td>
    </tr>
    <tr>
      <td>x86_64 Port</td>
      <td>Bring aiXos to Intel/AMD hardware</td>
    </tr>
    <tr>
      <td>HANIEL GUI</td>
      <td>Improve desktop rendering, layout, and interaction</td>
    </tr>
    <tr>
      <td>Liquid Glass UI</td>
      <td>Add Phoenix Full visual direction</td>
    </tr>
    <tr>
      <td>AWP Mesh</td>
      <td>Implement real signed sovereign networking</td>
    </tr>
    <tr>
      <td>EdisonDB Integration</td>
      <td>Add persistent local sovereign storage</td>
    </tr>
    <tr>
      <td>Onyxia Integration</td>
      <td>Bring the sovereign browser into aiXos</td>
    </tr>
  </tbody>
</table>

<hr>

<h2>Known Gaps</h2>

<p>
  Phoenix Lite is real and bootable, but it is still early-stage OS work.
</p>

<p><b>Current known gaps:</b></p>

<ul>
  <li>Keyboard events are not yet fully delivered from QEMU GTK to the shell</li>
  <li>Boot mode selection is currently hardcoded to Live mode</li>
  <li>AWP-Lite loopback is not yet wired to a real packet path</li>
  <li>Ed25519 signing stubs are not yet connected to real key material</li>
  <li>x86_64 target is not yet supported</li>
  <li>EdisonDB is not yet integrated as persistent system storage</li>
  <li>Onyxia browser is not yet integrated into the desktop</li>
  <li>Full AXONYX application-layer replacement is still in progress</li>
</ul>

<p>
  This project is intentionally public while it is being built. The goal is not
  to pretend the system is finished, but to show the path clearly and build it
  in the open.
</p>

<hr>

<h2>Project Philosophy</h2>

<p>
  aiXos is built on a simple belief:
</p>

<blockquote>
  <b>A computer should serve its owner first.</b>
</blockquote>

<p>That means:</p>

<ul>
  <li>Local-first by design</li>
  <li>No forced cloud dependency</li>
  <li>Transparent boot and policy chain</li>
  <li>User-owned identity</li>
  <li>User-owned data</li>
  <li>Open source foundation</li>
  <li>Security as architecture, not an afterthought</li>
  <li>Sovereignty as a system property, not a marketing word</li>
</ul>

<p>
  Phoenix Lite is the first visible proof of that direction: a bootable
  sovereign desktop with pixels on screen.
</p>

<hr>

<h2>License</h2>

<p>
  <b>Apache-2.0</b>
</p>

<p>
  Copyright © 2026 Edison Lepiten / AIEONYX
</p>

<p>
  All code released under Apache-2.0 remains available under the terms of that
  license.
</p>

<p>
  AIEONYX’s project pledge is to keep aiXos open, user-respecting, and aligned
  with digital sovereignty.
</p>

<hr>

<div align="center">

<p>
  <b>IDENTITY ESTABLISHED • POLICY ENFORCED • SOVEREIGN DESKTOP ON SCREEN</b>
</p>

<p>
  <sub>aiXos Phoenix — The Sovereign Desktop OS by AIEONYX</sub>
</p>

</div>
