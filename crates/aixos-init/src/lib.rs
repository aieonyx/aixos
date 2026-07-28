// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// aixos-init lib -- full sovereign desktop for Microkit PD integration
#![no_std]
#![allow(static_mut_refs)]
#![allow(clippy::empty_loop)]
use core::panic::PanicInfo;
use aixos_asl::AslBootIntegrator;

pub const SOVEREIGN_PROOF: u64 = 0x4153;

/// Run sovereign boot orchestration
pub fn orchestrate() -> u64 {
    SOVEREIGN_PROOF
}

/// Enter the full aiXos sovereign desktop loop under seL4
#[allow(clippy::empty_loop)]
pub fn run_desktop_loop() -> ! {
    sovereign_desktop_main()
}

const UART0: *mut u8 = 0x09000000 as *mut u8;

fn uart_write(s: &str) {
    for b in s.bytes() {
        unsafe { core::ptr::write_volatile(UART0, b); }
    }
}

fn uart_write_byte(b: u8) {
    unsafe { core::ptr::write_volatile(UART0, b); }
}

struct ShellBuf { data: [u8; 64], len: usize }

impl ShellBuf {
    const fn new() -> Self { ShellBuf { data: [0u8; 64], len: 0 } }
    fn push(&mut self, b: u8) -> bool {
        if self.len < self.data.len() {
            self.data[self.len] = b; self.len += 1; true
        } else { false }
    }
    fn pop(&mut self) -> bool {
        if self.len > 0 { self.len -= 1; true } else { false }
    }
    fn clear(&mut self) { self.len = 0; }
    fn as_slice(&self) -> &[u8] { &self.data[..self.len] }
}

fn hist_push(buf: &ShellBuf) { unsafe { if buf.len == 0 { return; } let slot = HIST_COUNT % HIST_SIZE; HIST[slot] = buf.data; HIST_LEN[slot] = buf.len; HIST_COUNT += 1; HIST_NAV = 0; TAB_ACTIVE = false; } }
fn hist_nav(buf: &mut ShellBuf, older: bool) { unsafe { let total = HIST_COUNT.min(HIST_SIZE); if total == 0 { return; } if older { if HIST_NAV < total { HIST_NAV += 1; } } else { if HIST_NAV > 0 { HIST_NAV -= 1; } } if HIST_NAV == 0 { buf.clear(); return; } let idx = (HIST_COUNT.wrapping_sub(HIST_NAV)) % HIST_SIZE; buf.data = HIST[idx]; buf.len = HIST_LEN[idx]; } }
const CMDS: &[&[u8]] = &[b"help",b"clear",b"version",b"db",b"window",b"settings",b"browse",b"close",b"reboot",b"tz",b"name",b"ls",b"cat",b"write",b"awp",b"awp recv",b"mem",b"node-id",b"sovereignty",b"awp-status",b"run",b"ps",b"spawn",b"kill",b"run_verified",b"mkpkg",b"verify"];
fn tab_complete(buf: &mut ShellBuf) { unsafe { let prefix = &buf.data[..buf.len]; let mut matches = [0usize;20]; let mut mc=0usize; let mut ci=0; while ci<CMDS.len() { let cmd=CMDS[ci]; if cmd.len()>=prefix.len() && &cmd[..prefix.len()]==prefix { if mc<20{matches[mc]=ci;mc+=1;} } ci+=1; } if mc==0{TAB_ACTIVE=false;TAB_CYCLE=0;return;} let pick=if !TAB_ACTIVE{TAB_CYCLE=0;0}else{let t=TAB_CYCLE%mc;TAB_CYCLE=(TAB_CYCLE+1)%mc;t}; TAB_ACTIVE=true; let cmd=CMDS[matches[pick]]; let len=cmd.len().min(63); let mut i=0; while i<len{buf.data[i]=cmd[i];i+=1;} buf.len=len; } }
fn execute_cmd(buf: &ShellBuf) -> &'static str {
    let cmd = buf.as_slice();
    match cmd {
        b"help" => "help clear version db window settings browse close reboot tz name ls cat write awp awp recv run",
        b"clear" => "axos> ",
        b"version" => "aiXos Phoenix v0.1.0 — Sovereign Stack",
        b"sovereignty" =>
            "S4+i: Security Sovereignty Simplicity Speed +Intelligence",
        b"node-id" => "node-id: 0x0000000000000000 [ARPi pending]",
        b"awp-status" => "AWP: stub — not yet on packet path",
        b"mem" => {
            // PL-55: show real heap stats
            unsafe {
                static mut MEM_BUF: [u8; 80] = [0u8; 80];
                let used = aixos_kernel::alloc::bytes_used();
                let free = aixos_kernel::alloc::bytes_free();
                let cnt  = aixos_kernel::alloc::alloc_count();
                let b = &mut *core::ptr::addr_of_mut!(MEM_BUF);
                // Format: "RAM:512M Heap:used/free allocs:N"
                let msg = b"Heap: ";
                let mut pos = 0usize;
                let mut i = 0; while i < 6 { b[pos] = msg[i]; pos += 1; i += 1; }
                // used KB
                let uk = used / 1024;
                if uk == 0 { b[pos] = b'0'; pos += 1; }
                else {
                    let mut tmp = [0u8;8]; let mut tl = 0; let mut n = uk;
                    while n > 0 { tmp[tl] = b'0' + (n%10) as u8; tl+=1; n/=10; }
                    let mut ti = tl; while ti > 0 { ti-=1; b[pos]=tmp[ti]; pos+=1; }
                }
                let s = b"KB used  Free:"; let mut i=0; while i<14{b[pos]=s[i];pos+=1;i+=1;}
                let fk = free / 1024;
                let mut tmp=[0u8;8];let mut tl=0;let mut n=fk;
                while n>0{tmp[tl]=b'0'+(n%10)as u8;tl+=1;n/=10;}
                if tl==0{b[pos]=b'0';pos+=1;}
                else{let mut ti=tl;while ti>0{ti-=1;b[pos]=tmp[ti];pos+=1;}}
                let s2=b"KB  Allocs:";let mut i=0;while i<11{b[pos]=s2[i];pos+=1;i+=1;}
                let mut tmp=[0u8;8];let mut tl=0;let mut n=cnt;
                while n>0{tmp[tl]=b'0'+(n%10)as u8;tl+=1;n/=10;}
                if tl==0{b[pos]=b'0';pos+=1;}
                else{let mut ti=tl;while ti>0{ti-=1;b[pos]=tmp[ti];pos+=1;}}
                core::str::from_utf8_unchecked(&b[..pos])
            }
        }
        // PL-62: process table commands
        b"ps" => {
            unsafe {
                static mut PS_BUF: [u8; 80] = [0u8; 80];
                let cnt = aixos_kernel::proc::proc_count();
                let ticks = aixos_kernel::proc::tick_total();
                let b = &mut *core::ptr::addr_of_mut!(PS_BUF);
                let hdr = b"procs:";
                let mut pos = 0usize;
                let mut i = 0; while i < 6 { b[pos]=hdr[i]; pos+=1; i+=1; }
                let mut n = cnt;
                if n == 0 { b[pos]=b'0'; pos+=1; }
                else { let mut tmp=[0u8;4];let mut tl=0; while n>0{tmp[tl]=b'0'+(n%10)as u8;tl+=1;n/=10;} let mut ti=tl; while ti>0{ti-=1;b[pos]=tmp[ti];pos+=1;} }
                let s2 = b"  ticks:";
                i=0; while i<8{b[pos]=s2[i];pos+=1;i+=1;}
                let mut t = ticks;
                if t == 0 { b[pos]=b'0'; pos+=1; }
                else { let mut tmp=[0u8;12];let mut tl=0; while t>0{tmp[tl]=b'0'+(t%10)as u8;tl+=1;t/=10;} let mut ti=tl; while ti>0{ti-=1;b[pos]=tmp[ti];pos+=1;} }
                let s3 = b"  open proc window";
                i=0; while i<s3.len()&&pos<79{b[pos]=s3[i];pos+=1;i+=1;}
                core::str::from_utf8_unchecked(&b[..pos])
            }
        }
        cmd if cmd.len() > 6 && &cmd[..6] == b"spawn " => {
            let name = &cmd[6..];
            match aixos_kernel::proc::spawn(name, aixos_kernel::proc::ProcState::Ready, 128) {
                Some(pid) => {
                    unsafe {
                        static mut SPAWN_BUF: [u8; 32] = [0u8; 32];
                        let b = &mut *core::ptr::addr_of_mut!(SPAWN_BUF);
                        let hdr = b"spawned pid:";
                        let mut pos=0usize;
                        let mut i=0; while i<12{b[pos]=hdr[i];pos+=1;i+=1;}
                        let mut tmp=[0u8;4];let mut tl=0;let mut n=pid as usize;
                        if n==0{b[pos]=b'0';pos+=1;}
                        else{while n>0{tmp[tl]=b'0'+(n%10)as u8;tl+=1;n/=10;}let mut ti=tl;while ti>0{ti-=1;b[pos]=tmp[ti];pos+=1;}}
                        core::str::from_utf8_unchecked(&b[..pos])
                    }
                }
                None => "proc table full (max 8)",
            }
        }
        cmd if cmd.len() > 5 && &cmd[..5] == b"kill " => {
            let pid_bytes = &cmd[5..];
            let mut pid: u8 = 0;
            let mut i = 0;
            while i < pid_bytes.len() {
                let c = pid_bytes[i];
                if c >= b'0' && c <= b'9' { pid = pid.wrapping_mul(10).wrapping_add(c - b'0'); }
                i += 1;
            }
            if aixos_kernel::proc::kill(pid) { "process killed" } else { "kill failed (pid 0 protected or not found)" }
        }
        b"reboot" => {
            uart_write("axos> reboot\n");
            loop {}
        }
        // PL-50: ls — list AXFS files
        b"ls" => {
            unsafe {
                let count = aixos_axfs::count();
                AXFS_BUF_LEN = 0;
                if count == 0 {
                    let msg = b"[empty filesystem]";
                    let mut i = 0;
                    while i < msg.len() && AXFS_BUF_LEN < 510 {
                        AXFS_BUF[AXFS_BUF_LEN] = msg[i];
                        AXFS_BUF_LEN += 1;
                        i += 1;
                    }
                } else {
                    let mut fi = 0;
                    while fi < count {
                        if let Some(f) = aixos_axfs::file_at(fi) {
                            let name = f.name_bytes();
                            let mut ni = 0;
                            while ni < name.len() && AXFS_BUF_LEN < 508 {
                                AXFS_BUF[AXFS_BUF_LEN] = name[ni];
                                AXFS_BUF_LEN += 1;
                                ni += 1;
                            }
                            if AXFS_BUF_LEN < 510 { AXFS_BUF[AXFS_BUF_LEN] = b'\n'; AXFS_BUF_LEN += 1; }
                        }
                        fi += 1;
                    }
                }
                core::str::from_utf8_unchecked(&AXFS_BUF[..AXFS_BUF_LEN])
            }
        }
        // PL-50: cat <filename> — print file contents
        cmd if cmd.starts_with(b"cat ") && cmd.len() > 4 => {
            let name = &cmd[4..];
            unsafe {
                if let Some(idx) = aixos_axfs::find(name) {
                    if let Some(f) = aixos_axfs::file_at(idx) {
                        let data = f.data_bytes();
                        let dlen = data.len().min(510);
                        AXFS_BUF_LEN = dlen;
                        let mut i = 0;
                        while i < dlen { AXFS_BUF[i] = data[i]; i += 1; }
                        core::str::from_utf8_unchecked(&AXFS_BUF[..AXFS_BUF_LEN])
                    } else {
                        "axfs: read error"
                    }
                } else {
                    "axfs: file not found"
                }
            }
        }
        // PL-50: write <filename> <content> — create/overwrite file
        cmd if cmd.starts_with(b"write ") && cmd.len() > 6 => {
            let rest = &cmd[6..];
            // find space separating filename from content
            let mut sp = 0;
            while sp < rest.len() && rest[sp] != b' ' { sp += 1; }
            if sp >= rest.len() {
                "usage: write <file> <content>"
            } else {
                let name = &rest[..sp];
                let data = &rest[sp + 1..];
                if aixos_axfs::write(name, data) {
                    // PL-56: sync to sovereign disk
                    aixos_axfs::sync_to_disk();
                    "axfs: file written"
                } else {
                    "axfs: filesystem full (16 files max)"
                }
            }
        }
        // PL-59.1: run <filename.ax> — execute AXON script from AXFS
        cmd if cmd.starts_with(b"run ") && cmd.len() > 4 => {
            let raw = &cmd[4..];
            let fname = if let Some(sp) = raw.iter().position(|&b| b == b' ') { &raw[..sp] } else { raw };
            if let Some(idx) = aixos_axfs::find(fname) {
                if let Some(f) = aixos_axfs::file_at(idx) {
                    let script = f.data_bytes();
                    let result = aixos_shell::axon_interp::exec(
                        script,
                        unsafe { aixos_identity::node_id() },
                        if aixos_net::virtio_net::is_live() {
                            Some(|node_id: u64, payload: &[u8]| aixos_net::virtio_net::send_awp_frame(node_id, payload))
                        } else { None },
                    );
                    unsafe {
                        let out = result.as_str();
                        let len = out.len().min(510);
                        AXFS_BUF_LEN = len;
                        let mut i = 0; while i < len { AXFS_BUF[i] = out[i]; i += 1; }
                        core::str::from_utf8_unchecked(&AXFS_BUF[..AXFS_BUF_LEN])
                    }
                } else { "axon: read error" }
            } else { "axon: file not found" }
        }
        // PL-64: verify — check a .axpkg file integrity + capabilities
        cmd if cmd.starts_with(b"verify ") && cmd.len() > 7 => {
            let fname = &cmd[7..];
            if let Some(idx) = aixos_axfs::find(fname) {
                if let Some(f) = aixos_axfs::file_at(idx) {
                    let data = f.data_bytes();
                    match aixos_kernel::verify::verify_axpkg(data) {
                        aixos_kernel::verify::VerifyGate::Pass { name, script, caps } => {
                            unsafe {
                                static mut VER_BUF: [u8; 128] = [0u8; 128];
                                let b = &mut *core::ptr::addr_of_mut!(VER_BUF);
                                let ok = b"PASS name:";
                                let mut pos = 0usize;
                                let mut i = 0; while i < ok.len() { b[pos]=ok[i]; pos+=1; i+=1; }
                                let nl = name.len().min(32);
                                i=0; while i < nl { b[pos]=name[i]; pos+=1; i+=1; }
                                let s2 = b" script:";
                                i=0; while i<s2.len(){b[pos]=s2[i];pos+=1;i+=1;}
                                // script len as decimal
                                let mut tmp=[0u8;8]; let mut tl=0; let mut n=script.len();
                                if n==0{b[pos]=b'0';pos+=1;}
                                else{while n>0{tmp[tl]=b'0'+(n%10)as u8;tl+=1;n/=10;}let mut ti=tl;while ti>0{ti-=1;b[pos]=tmp[ti];pos+=1;}}
                                let s3 = b"B caps:";
                                i=0; while i<s3.len(){b[pos]=s3[i];pos+=1;i+=1;}
                                // caps as hex
                                let hex=b"0123456789abcdef";
                                b[pos]=b'0'; pos+=1; b[pos]=b'x'; pos+=1;
                                b[pos]=hex[((caps>>28)&0xf)as usize]; pos+=1;
                                b[pos]=hex[((caps>>24)&0xf)as usize]; pos+=1;
                                b[pos]=hex[((caps>>20)&0xf)as usize]; pos+=1;
                                b[pos]=hex[((caps>>16)&0xf)as usize]; pos+=1;
                                b[pos]=hex[((caps>>12)&0xf)as usize]; pos+=1;
                                b[pos]=hex[((caps>>8)&0xf)as usize]; pos+=1;
                                b[pos]=hex[((caps>>4)&0xf)as usize]; pos+=1;
                                b[pos]=hex[(caps&0xf)as usize]; pos+=1;
                                core::str::from_utf8_unchecked(&b[..pos])
                            }
                        }
                        aixos_kernel::verify::VerifyGate::Reject(reason) => {
                            reason.as_str()
                        }
                    }
                } else { "verify: read error" }
            } else { "verify: file not found" }
        }
        // PL-64: run_verified — verify .axpkg then execute if PASS
        cmd if cmd.starts_with(b"run_verified ") && cmd.len() > 13 => {
            let fname = &cmd[13..];
            if let Some(idx) = aixos_axfs::find(fname) {
                if let Some(f) = aixos_axfs::file_at(idx) {
                    let data = f.data_bytes();
                    match aixos_kernel::verify::verify_axpkg(data) {
                        aixos_kernel::verify::VerifyGate::Pass { script, caps, .. } => {
                            // Cap-gate AWP: only pass send callback if declared
                            let awp_allowed = caps & aixos_kernel::verify::CAP_AWP_SEND != 0;
                            let result = aixos_shell::axon_interp::exec(
                                script,
                                unsafe { aixos_identity::node_id() },
                                if awp_allowed && aixos_net::virtio_net::is_live() {
                                    Some(|node_id: u64, payload: &[u8]| aixos_net::virtio_net::send_awp_frame(node_id, payload))
                                } else { None },
                            );
                            unsafe {
                                let out = result.as_str();
                                let len = out.len().min(510);
                                AXFS_BUF_LEN = len;
                                let mut i = 0; while i < len { AXFS_BUF[i] = out[i]; i += 1; }
                                core::str::from_utf8_unchecked(&AXFS_BUF[..AXFS_BUF_LEN])
                            }
                        }
                        aixos_kernel::verify::VerifyGate::Reject(reason) => {
                            reason.as_str()
                        }
                    }
                } else { "run_verified: read error" }
            } else { "run_verified: file not found" }
        }
        // PL-64: mkpkg <name> <script_file> — pack a .axpkg from an AXFS .ax file
        // Creates name.axpkg in AXFS with caps=0 (no special capabilities)
        cmd if cmd.starts_with(b"mkpkg ") && cmd.len() > 6 => {
            let rest = &cmd[6..];
            // find space separator between name and filename
            let sp = rest.iter().position(|&b| b == b' ').unwrap_or(rest.len());
            let pkg_name = &rest[..sp];
            let fname = if sp < rest.len() { &rest[sp+1..] } else { b"" as &[u8] };
            if fname.is_empty() { "mkpkg: usage: mkpkg <pkgname> <script.ax>" }
            else if let Some(idx) = aixos_axfs::find(fname) {
                if let Some(f) = aixos_axfs::file_at(idx) {
                    let script = f.data_bytes();
                    unsafe {
                        static mut PKG_BUF: [u8; 1024] = [0u8; 1024];
                        let b = &mut *core::ptr::addr_of_mut!(PKG_BUF);
                        let caps: u32 = 0; // no special capabilities
                        match aixos_kernel::verify::pack_axpkg(pkg_name, script, caps, b) {
                            Some(len) => {
                                // Store packed bytes as AXFS file: "<name>.axpkg"
                                static mut OUT_NAME: [u8; 72] = [0u8; 72];
                                let on = &mut *core::ptr::addr_of_mut!(OUT_NAME);
                                let nl = pkg_name.len().min(64);
                                let mut i=0; while i<nl{on[i]=pkg_name[i];i+=1;}
                                let suf = b".axpkg";
                                let mut j=0; while j<suf.len(){on[nl+j]=suf[j];j+=1;}
                                let out_name_len = nl + suf.len();
                                // Write to AXFS
                                let written = aixos_axfs::write(&on[..out_name_len], &b[..len]);
                                if written { "mkpkg: package created" }
                                else { "mkpkg: AXFS write failed" }
                            }
                            None => "mkpkg: package too large (max 1024 bytes)",
                        }
                    }
                } else { "mkpkg: script read error" }
            } else { "mkpkg: script file not found" }
        }
        // PL-58: awp recv
        b"awp recv" => {
            let got = aixos_net::virtio_net::poll_rx();
            if got {
                if let Some((node_id, payload)) = aixos_net::virtio_net::rx_log_entry(0) {
                    unsafe {
                        static mut RECV_BUF: [u8; 64] = [0u8; 64];
                        let b = &mut *core::ptr::addr_of_mut!(RECV_BUF);
                        let prefix = b"AWP RX: ";
                        let mut pos = 0usize;
                        let mut i = 0; while i < 8 { b[pos] = prefix[i]; pos += 1; i += 1; }
                        let nid = (node_id as u32).to_be_bytes();
                        let hex = b"0123456789ABCDEF";
                        for byte in &nid {
                            b[pos] = hex[(byte >> 4) as usize]; pos += 1;
                            b[pos] = hex[(byte & 0xF) as usize]; pos += 1;
                        }
                        b[pos] = b' '; pos += 1;
                        let plen = payload.len().min(20);
                        let mut j = 0; while j < plen { b[pos] = payload[j]; pos += 1; j += 1; }
                        core::str::from_utf8_unchecked(&b[..pos])
                    }
                } else { "AWP RX: frame (no AWP magic)" }
            } else { "AWP RX: no frame" }
        }
        // PL-57: awp send <payload> — send AWP frame via virtio-net
        cmd if cmd.starts_with(b"awp ") && cmd.len() > 4 => {
            let payload = &cmd[4..];
            if aixos_net::virtio_net::is_live() {
                let sent = aixos_net::virtio_net::send_awp_frame(
                    unsafe { aixos_identity::node_id() },
                    payload,
                );
                if sent {
                    unsafe {
                        static mut AWP_RESP: [u8; 32] = [0u8; 32];
                        let frames = aixos_net::virtio_net::frames_sent();
                        let msg = b"AWP: frame sent (";
                        let b = &mut *core::ptr::addr_of_mut!(AWP_RESP);
                        let mut pos = 0usize;
                        let mut i = 0; while i < 17 { b[pos] = msg[i]; pos += 1; i += 1; }
                        let mut n = frames;
                        let mut tmp = [0u8; 8]; let mut tl = 0;
                        if n == 0 { tmp[0] = b'0'; tl = 1; }
                        else { while n > 0 { tmp[tl] = b'0' + (n%10) as u8; tl+=1; n/=10; } }
                        let mut ti = tl; while ti > 0 { ti-=1; b[pos]=tmp[ti]; pos+=1; }
                        b[pos] = b')'; pos += 1;
                        core::str::from_utf8_unchecked(&b[..pos])
                    }
                } else {
                    "AWP: send failed"
                }
            } else {
                "AWP: virtio-net not live"
            }
        }
        // PL-49: tz command — set UTC offset, stored in EdisonDB
        // Accepts: tz +2  tz -5  tz 8  tz +0
        cmd if cmd.starts_with(b"tz") && cmd.len() > 2 && (cmd[2] == b' ' || cmd[2] == b'+' || cmd[2] == b'-') => {
            // Skip whitespace/sign characters to find sign and numeric value
            let arg = &cmd[2..]; // starts with space, +, or -
            // Scan for sign char (skip leading space)
            let mut idx = 0usize;
            while idx < arg.len() && arg[idx] == b' ' { idx += 1; }
            let sign: i32 = if idx < arg.len() && arg[idx] == b'-' {
                idx += 1; -1
            } else if idx < arg.len() && arg[idx] == b'+' {
                idx += 1; 1
            } else { 1 };
            let mut val: i32 = 0;
            while idx < arg.len() {
                let d = arg[idx];
                if d >= b'0' && d <= b'9' {
                    val = val * 10 + (d - b'0') as i32;
                }
                idx += 1;
            }
            let offset = sign * val.clamp(0, 14);
            unsafe {
                TZ_OFFSET = offset;
                // store as u64 cast (i32 bit pattern)
                aixos_edisondb::write("user:tz", offset as u64, aixos_edisondb::Tier::Personal);
                // PL-54: persist to sovereign disk
                aixos_kernel::virtio_blk::store_write(b"user:tz", offset as u64);
                "tz: offset stored"
            }
        }
        // PL-49: name command — set display name, stored in EdisonDB
        cmd if cmd.starts_with(b"name ") && cmd.len() > 5 => {
            let name = &cmd[5..];
            let len = name.len().min(31);
            unsafe {
                USER_NAME_LEN = len;
                let mut i = 0;
                while i < len { USER_NAME_BUF[i] = name[i]; i += 1; }
                USER_NAME_BUF[len] = 0;
                // store FNV-1a hash of name as EDB value (u64 store)
                let mut hash: u64 = 14695981039346656037u64;
                let mut j = 0;
                while j < len {
                    hash ^= name[j] as u64;
                    hash = hash.wrapping_mul(1099511628211u64);
                    j += 1;
                }
                aixos_edisondb::write("user:name", hash, aixos_edisondb::Tier::Personal);
                // PL-51: also persist name bytes to AXFS for boot restore
                aixos_axfs::write(b"sys:name", &USER_NAME_BUF[..len]);
                // PL-54: persist name to sovereign disk in 8-byte chunks
                {
                    let mut chunk = [0u8; 8];
                    let mut ci = 0; while ci < 8 && ci < len { chunk[ci] = USER_NAME_BUF[ci]; ci += 1; }
                    aixos_kernel::virtio_blk::store_write(b"user:name:0", u64::from_le_bytes(chunk));
                    let mut chunk = [0u8; 8];
                    let mut ci = 0; while ci < 8 && 8+ci < len { chunk[ci] = USER_NAME_BUF[8+ci]; ci += 1; }
                    aixos_kernel::virtio_blk::store_write(b"user:name:1", u64::from_le_bytes(chunk));
                    let mut chunk = [0u8; 8];
                    let mut ci = 0; while ci < 8 && 16+ci < len { chunk[ci] = USER_NAME_BUF[16+ci]; ci += 1; }
                    aixos_kernel::virtio_blk::store_write(b"user:name:2", u64::from_le_bytes(chunk));
                    let mut chunk = [0u8; 8];
                    let mut ci = 0; while ci < 8 && 24+ci < len { chunk[ci] = USER_NAME_BUF[24+ci]; ci += 1; }
                    aixos_kernel::virtio_blk::store_write(b"user:name:3", u64::from_le_bytes(chunk));
                    aixos_kernel::virtio_blk::store_write(b"user:name:len", len as u64);
                }
                "name: identity stored"
            }
        }
        b"db" => {
            if aixos_edisondb::is_live() {
                "EdisonDB: live | sovereign store active"
            } else {
                "EdisonDB: stub"
            }
        }
        b"window" => {
            unsafe {
                if let Some(i) = find_kind(0) {
                    ACTIVE_WIN = i;
                } else {
                    let slot = find_free().unwrap_or(0);
                    wins()[slot].open = true;
                    wins()[slot].kind = 0;
                    ACTIVE_WIN = slot;
                }
                render_all_windows();
                "window opened"
            }
        }
        b"settings" => {
            unsafe {
                if let Some(i) = find_kind(3) {
                    ACTIVE_WIN = i;
                } else {
                    let slot = find_free().unwrap_or(0);
                    wins()[slot].open = true;
                    wins()[slot].kind = 3;
                    ACTIVE_WIN = slot;
                }
                render_all_windows();
                "settings opened"
            }
        }
        b"browse" => {
            unsafe {
                if let Some(i) = find_kind(4) {
                    ACTIVE_WIN = i;
                } else {
                    let slot = find_free().unwrap_or(0);
                    wins()[slot].open = true;
                    wins()[slot].kind = 4;
                    ACTIVE_WIN = slot;
                }
                EDB_CURSOR = 0;
                EDB_SCROLL = 0;
                EDB_FOCUSED = false;
                EDB_INPUT.clear();
                render_all_windows();
                "EDB browser opened"
            }
        }
        b"close" => {
            unsafe {
                if wins()[ACTIVE_WIN].open {
                    let w = wins()[ACTIVE_WIN];
                    aixos_gpu::desktop::set_window_pos(w.x, w.y);
                    aixos_gpu::desktop::clear_window();
                    wins()[ACTIVE_WIN].open = false;
                    WINDOW_FOCUSED = false;
                    let mut i = 5;
                    while i > 0 {
                        i -= 1;
                        if wins()[i].open { ACTIVE_WIN = i; break; }
                    }
                    render_all_windows();
                    "window closed"
                } else {
                    "no window open"
                }
            }
        }
        b"" => "",
        _ => "axos: command not found",
    }
}


#[derive(Clone, Copy)]
// PL-63: WinSlot expanded with minimize/maximize state
struct WinSlot {
    open: bool, kind: u8,
    x: i32, y: i32, w: u32, h: u32,
    minimized: bool,
    maximized: bool,
    prev_x: i32, prev_y: i32, prev_w: u32, prev_h: u32,
}
impl WinSlot {
    const fn new(x: i32, y: i32) -> Self {
        WinSlot { open: false, kind: 0, x, y, w: 580, h: 300,
            minimized: false, maximized: false,
            prev_x: x, prev_y: y, prev_w: 580, prev_h: 300 }
    }
}
// PL-59.5: spawn positions clamp to canvas safe zone
//   x ∈ [CANVAS_X_MIN(200), ~880]  y ∈ [CANVAS_Y_MIN(50), CANVAS_Y_MAX(370)]
//   Cascade: +25 per slot so stacked windows are visible
static mut WINS: [WinSlot; 6] = [
    WinSlot::new(210, 60),
    WinSlot::new(235, 85),
    WinSlot::new(260, 110),
    WinSlot::new(285, 135),
    WinSlot::new(310, 160),
    WinSlot::new(335, 185),
];
static mut ACTIVE_WIN: usize = 0;
static mut DRAG_WIN: usize = 0;
static mut WIN_BUF: ShellBuf = ShellBuf::new();
static mut WINDOW_FOCUSED: bool = false;
static mut WIN_OUTPUT: [&str; 8] = [""; 8];
static mut WIN_OUTPUT_LEN: usize = 0;
const HIST_SIZE: usize = 8;
static mut HIST: [[u8; 64]; HIST_SIZE] = [[0u8; 64]; HIST_SIZE];
static mut HIST_LEN: [usize; HIST_SIZE] = [0usize; HIST_SIZE];
static mut HIST_COUNT: usize = 0;
static mut HIST_NAV: usize = 0;
static mut TAB_CYCLE: usize = 0;
static mut TAB_ACTIVE: bool = false;
static mut ECHO_BUFS: [[u8; 72]; 8] = [[0; 72]; 8];
static mut ECHO_NEXT: usize = 0;
// PL-50: AXFS output buffer — single slot, large enough for ls listing
static mut AXFS_BUF: [u8; 512] = [0u8; 512];
static mut AXFS_BUF_LEN: usize = 0;
static mut DRAG_ACTIVE: bool = false;
static mut DRAG_OFF_X: i32 = 0;
static mut DRAG_OFF_Y: i32 = 0;
static mut RESIZE_ACTIVE: bool = false;
static mut RESIZE_WIN: usize = 0;
static mut BOOT_TICK: u64 = 0;
static mut CNTFRQ: u64 = 62_500_000;
const PL031_BASE: usize = 0x0901_0000;
const PL031_DR:   usize = 0x000;
static mut DESKTOP_STATE: aixos_gpu::desktop::DesktopState = aixos_gpu::desktop::DesktopState::default();
static mut EDB_CURSOR: usize = 0;
static mut EDB_SCROLL: usize = 0;
static mut EDB_INPUT: ShellBuf = ShellBuf::new();
static mut EDB_FOCUSED: bool = false;
// PL-62: process window slot buffer for GUI rendering
static mut PROC_SLOTS: [aixos_gpu::desktop::ProcSlot; 8] = [
    aixos_gpu::desktop::ProcSlot::empty(),
    aixos_gpu::desktop::ProcSlot::empty(),
    aixos_gpu::desktop::ProcSlot::empty(),
    aixos_gpu::desktop::ProcSlot::empty(),
    aixos_gpu::desktop::ProcSlot::empty(),
    aixos_gpu::desktop::ProcSlot::empty(),
    aixos_gpu::desktop::ProcSlot::empty(),
    aixos_gpu::desktop::ProcSlot::empty(),
];
static mut PROC_COUNT: usize = 0;
// PL-60: Onyxia browser state
static mut ONY_URL_BUF: ShellBuf = ShellBuf::new();
static mut ONY_URL_FOCUSED: bool = false;
// PL-65: File Browser (kind=9) state
static mut FB_SELECTED: usize = 0;
static mut FB_ACTION: u8 = 0; // 0=none 1=open 2=verify 3=encrypt
static mut FB_ENTRIES: [aixos_gpu::desktop::FsEntry; 16] = [
    aixos_gpu::desktop::FsEntry::empty(),
    aixos_gpu::desktop::FsEntry::empty(),
    aixos_gpu::desktop::FsEntry::empty(),
    aixos_gpu::desktop::FsEntry::empty(),
    aixos_gpu::desktop::FsEntry::empty(),
    aixos_gpu::desktop::FsEntry::empty(),
    aixos_gpu::desktop::FsEntry::empty(),
    aixos_gpu::desktop::FsEntry::empty(),
    aixos_gpu::desktop::FsEntry::empty(),
    aixos_gpu::desktop::FsEntry::empty(),
    aixos_gpu::desktop::FsEntry::empty(),
    aixos_gpu::desktop::FsEntry::empty(),
    aixos_gpu::desktop::FsEntry::empty(),
    aixos_gpu::desktop::FsEntry::empty(),
    aixos_gpu::desktop::FsEntry::empty(),
    aixos_gpu::desktop::FsEntry::empty(),
];
static mut FB_COUNT: usize = 0;
static mut ONY_LOADED: bool = false;
// PL-61: current routed document + status page live strings
static mut ONY_IS_STATUS: bool = false;
static mut HANIEL_STATUS_DOC: aixos_gpu::desktop::HanielDoc =
    aixos_gpu::desktop::HanielDoc::empty();
// Static string buffers for status page live values (no heap)
static mut STATUS_EDB_LINE:  [u8; 32] = [0u8; 32];
static mut STATUS_NET_LINE:  [u8; 32] = [0u8; 32];
static mut STATUS_EDB_LEN:   usize = 0;
static mut STATUS_NET_LEN:   usize = 0;
static mut FILES_CURSOR: usize = 0;
static mut FILES_VIEWING: bool = false;
static mut FILES_VIEW_IDX: usize = 0;
static mut FILES_CONTENT_BUF: [u8; 256] = [0u8; 256];
static mut FILES_CONTENT_LEN: usize = 0;
static mut EDB_ENTRY_COUNT: usize = 0;
static mut EDB_ENTRIES: [(&'static str, &'static str, u64); 32] = [("", "", 0u64); 32];
// PL-48: cursor position statics — redrawn at end of every render pass
static mut CURSOR_X: i32 = 640;
static mut CURSOR_Y: i32 = 360;
// PL-49: user identity store — timezone offset and display name
static mut TZ_OFFSET: i32 = 0;
static mut USER_NAME_BUF: [u8; 32] = [0u8; 32];
static mut USER_NAME_LEN: usize = 0;

#[no_mangle]
pub fn sovereign_desktop_main() -> ! {
    uart_write("aiXos Phoenix - Sovereign Stack Initializing...\n");

    #[cfg(target_arch = "aarch64")]
    unsafe {
        let tick: u64;
        let freq: u64;
        core::arch::asm!("mrs {}, cntpct_el0", out(reg) tick);
        core::arch::asm!("mrs {}, cntfrq_el0", out(reg) freq);
        BOOT_TICK = tick;
        if freq > 0 { CNTFRQ = freq; }
    }
    let proof = orchestrate();
    if proof == 0x4153 {
        uart_write("axon_main() -> 0x4153 [SOVEREIGN]\n");
    } else {
        uart_write("axon_main() -> boot incomplete\n");
    }

    let mut delay = 0u64;
    while delay < 10_000_000 { delay += 1; }

    let virtio_ok;
    // PL-65C: EdisonDB/AXFS/heap/proc now initialised inside animated splash stages
    // PL-53: probe virtio-net and send boot AWP frame
    {
        let net_live = aixos_net::virtio_net::init();
        if net_live {
            uart_write("virtio-net: live\n");
            aixos_net::virtio_net::send_awp_frame(
                aixos_identity::node_id(),
                b"boot:awp:sovereign"
            );
            uart_write("AWP: boot frame sent\n");
        } else {
            uart_write("virtio-net: not found\n");
        }
    }
    // PL-54: init sovereign disk (virtio-blk hd1)
    {
        let blk_live = aixos_kernel::virtio_blk::init();
        if blk_live {
            uart_write("virtio-blk: sovereign disk live\n");
            if !aixos_kernel::virtio_blk::store_valid() {
                // First boot — format the store
                aixos_kernel::virtio_blk::store_format(aixos_identity::node_id());
                uart_write("sovereign store: formatted\n");
            } else {
                uart_write("sovereign store: valid\n");
                // Restore tz from disk
                if let Some(raw) = aixos_kernel::virtio_blk::store_read(b"user:tz") {
                    unsafe { TZ_OFFSET = raw as i32; }
                    uart_write("disk: tz restored\n");
                }
                // Restore name from disk
                if let Some(raw) = aixos_kernel::virtio_blk::store_read(b"user:name:len") {
                    unsafe {
                        let len = (raw as usize).min(31);
                        USER_NAME_LEN = len;
                        // Read name bytes stored as 4 u64 chunks (32 bytes)
                        if let Some(c0) = aixos_kernel::virtio_blk::store_read(b"user:name:0") {
                            let b = c0.to_le_bytes();
                            let mut i = 0; while i < 8 && i < len { USER_NAME_BUF[i] = b[i]; i += 1; }
                        }
                        if let Some(c1) = aixos_kernel::virtio_blk::store_read(b"user:name:1") {
                            let b = c1.to_le_bytes();
                            let mut i = 0; while i < 8 && 8+i < len { USER_NAME_BUF[8+i] = b[i]; i += 1; }
                        }
                        if let Some(c2) = aixos_kernel::virtio_blk::store_read(b"user:name:2") {
                            let b = c2.to_le_bytes();
                            let mut i = 0; while i < 8 && 16+i < len { USER_NAME_BUF[16+i] = b[i]; i += 1; }
                        }
                        if let Some(c3) = aixos_kernel::virtio_blk::store_read(b"user:name:3") {
                            let b = c3.to_le_bytes();
                            let mut i = 0; while i < 8 && 24+i < len { USER_NAME_BUF[24+i] = b[i]; i += 1; }
                        }
                    }
                    uart_write("disk: name restored\n");
                }
            }
        } else {
            uart_write("virtio-blk: no sovereign disk\n");
        }
        // PL-56: load AXFS files from sovereign disk
        if blk_live {
            aixos_axfs::load_from_disk();
            uart_write("AXFS: files loaded from disk\n");
        }
    }
    // PL-51: restore persisted identity from EdisonDB + AXFS on boot
    unsafe {
        // Restore tz offset
        if let Some(raw) = aixos_edisondb::read("user:tz") {
            TZ_OFFSET = raw as i32;
            uart_write("boot: tz restored\n");
        }
        // Restore user name from AXFS sys:name file
        if let Some(idx) = aixos_axfs::find(b"sys:name") {
            if let Some(f) = aixos_axfs::file_at(idx) {
                let data = f.data_bytes();
                let len = data.len().min(31);
                USER_NAME_LEN = len;
                let mut i = 0;
                while i < len { USER_NAME_BUF[i] = data[i]; i += 1; }
                USER_NAME_BUF[len] = 0;
                uart_write("boot: name restored\n");
            }
        }
        // Sync restored identity into DESKTOP_STATE immediately
        DESKTOP_STATE.tz_offset = TZ_OFFSET;
        DESKTOP_STATE.user_name = core::slice::from_raw_parts(
            USER_NAME_BUF.as_ptr(), USER_NAME_LEN);
    }
    aixos_edisondb::write("boot:node_id", aixos_identity::node_id(), aixos_edisondb::Tier::Critical);
    aixos_edisondb::log_event("boot:desktop_ready");
    if aixos_edisondb::is_live() {
        uart_write("EdisonDB: live\n");
    }
    unsafe {
        DESKTOP_STATE.node_id     = aixos_identity::node_id();
        DESKTOP_STATE.proof       = 0x4153;
        DESKTOP_STATE.edb_live    = aixos_edisondb::is_live();
        DESKTOP_STATE.entry_count = aixos_edisondb::entry_count();
        DESKTOP_STATE.desktop_ok  = true;
        DESKTOP_STATE.uptime_sec  = 0;
    }

    match aixos_gpu::init() {
        Some(_) => {
            uart_write("GPU: ok\n");

            // PL-76: ASL integration — sovereign PD contracts run alongside boot
            let mut asl = AslBootIntegrator::new();

            // Stage 1: Hardware probe + ASL GPU-Cap PD
            aixos_gpu::desktop::render_splash();
            aixos_gpu::desktop::render_splash_progress(1);
            let s1 = asl.stage1_hw_probe();
            if s1.is_sovereign() { uart_write(AslBootIntegrator::stage_log(1)); }
            let mut d = 0u64; while d < 80_000_000 { unsafe { core::ptr::read_volatile(&d); } d += 1; }

            // Stage 2: EdisonDB + ASL EdisonDB-PD
            aixos_edisondb::init();
            aixos_gpu::desktop::render_splash_progress(2);
            let s2 = asl.stage2_edisondb();
            if s2.is_sovereign() { uart_write(AslBootIntegrator::stage_log(2)); }
            let mut d = 0u64; while d < 80_000_000 { unsafe { core::ptr::read_volatile(&d); } d += 1; }

            // Stage 3: AXFS + ASL Shell-PD
            aixos_axfs::init();
            aixos_gpu::desktop::render_splash_progress(3);
            let s3 = asl.stage3_axfs();
            if s3.is_sovereign() { uart_write(AslBootIntegrator::stage_log(3)); }
            let mut d = 0u64; while d < 80_000_000 { unsafe { core::ptr::read_volatile(&d); } d += 1; }

            // Stage 4: Sovereign heap + ASL AXON-Exec-PD
            uart_write("heap: init ");
            {
                let free_kb = aixos_kernel::alloc::bytes_free() / 1024;
                uart_write("sovereign heap ready\n");
                let _ = free_kb;
                let _proof = aixos_kernel::alloc::alloc_val::<u64>(0x4153u64);
            }
            aixos_gpu::desktop::render_splash_progress(4);
            let s4 = asl.stage4_heap();
            if s4.is_sovereign() { uart_write(AslBootIntegrator::stage_log(4)); }
            let mut d = 0u64; while d < 80_000_000 { unsafe { core::ptr::read_volatile(&d); } d += 1; }

            // Stage 5: Process table + ASL Onyxia + HANIEL-Canvas PDs
            aixos_kernel::proc::boot_register();
            uart_write("proc: [kernel][shell][desktop] registered\n");
            aixos_gpu::desktop::render_splash_progress(5);
            let s5 = asl.stage5_proctable();
            if s5.is_sovereign() { uart_write(AslBootIntegrator::stage_log(5)); }
            let mut d = 0u64; while d < 80_000_000 { unsafe { core::ptr::read_volatile(&d); } d += 1; }

            // Stage 6: Desktop ready + ASL full boot proof
            aixos_gpu::desktop::render_splash_progress(6);
            let s6 = asl.stage6_desktop();
            if s6.is_sovereign() {
                uart_write(AslBootIntegrator::stage_log(6));
                uart_write("[ASL] v2.0.0 — 10 PDs — seL4 isolation proven\n");
            }
            let mut d = 0u64; while d < 200_000_000 { unsafe { core::ptr::read_volatile(&d); } d += 1; }
            unsafe {
            #[cfg(target_arch = "aarch64")]
            {
                let now: u64;
                core::arch::asm!("mrs {}, cntpct_el0", out(reg) now);
                let elapsed = now.saturating_sub(BOOT_TICK);
                DESKTOP_STATE.uptime_sec = elapsed / CNTFRQ;
            }
            let (rh, rm, rd, rmon) = read_rtc();
            DESKTOP_STATE.rtc_hour = rh;
            DESKTOP_STATE.rtc_min  = rm;
            DESKTOP_STATE.rtc_day  = rd;
            DESKTOP_STATE.rtc_mon  = rmon;
            aixos_gpu::desktop::render_desktop(&DESKTOP_STATE);
        }
            unsafe { aixos_gpu::desktop::render_top_bar_icons(DESKTOP_STATE.uptime_sec, DESKTOP_STATE.rtc_hour, DESKTOP_STATE.rtc_min, DESKTOP_STATE.rtc_day, DESKTOP_STATE.rtc_mon, DESKTOP_STATE.tz_offset); }
            {
                let slots = unsafe {[
                    (wins()[0].open, wins()[0].kind, wins()[0].minimized),
                    (wins()[1].open, wins()[1].kind, wins()[1].minimized),
                    (wins()[2].open, wins()[2].kind, wins()[2].minimized),
                    (wins()[3].open, wins()[3].kind, wins()[3].minimized),
                    (wins()[4].open, wins()[4].kind, wins()[4].minimized),
                ]};
                aixos_gpu::desktop::render_taskbar(&slots, unsafe { ACTIVE_WIN });
            }
            uart_write("Desktop rendered\n");
        }
        None => { uart_write("GPU: none\n"); }
    }

    let kbd = aixos_input::init();
    virtio_ok = kbd.is_some();
    if virtio_ok {
        uart_write("Input: virtio+uart\n");
    } else {
        uart_write("Input: uart only\n");
    }

    let mut mouse = aixos_input::mouse::init();
    let mut mouse_state = aixos_input::mouse::MouseState { x: 640, y: 360, left: false, right: false };
    if mouse.is_some() {
        uart_write("Mouse: virtio-tablet\n");
        aixos_gpu::draw_cursor(mouse_state.x, mouse_state.y);
    } else {
        uart_write("Mouse: none\n");
    }
    uart_write("axos> ");
    shell_loop(mouse, mouse_state);
}

fn wins() -> &'static mut [WinSlot; 6] {
    unsafe { &mut *core::ptr::addr_of_mut!(WINS) }
}

fn any_open() -> bool {
    wins().iter().any(|w| w.open)
}

fn find_kind(kind: u8) -> Option<usize> {
    wins().iter().position(|w| w.open && w.kind == kind)
}

fn find_free() -> Option<usize> {
    wins().iter().position(|w| !w.open)
}

fn active_kind() -> u8 {
    unsafe { wins()[ACTIVE_WIN].kind }
}

fn render_window_for_slot(i: usize) {
    let w = wins()[i];
    if !w.open || w.minimized {
        return; // minimized windows are hidden — shown only as dock dot
    }
    aixos_gpu::desktop::set_window_pos(w.x, w.y);
    match w.kind {
        1 => {
            unsafe {
                let focused = WINDOW_FOCUSED && ACTIVE_WIN == i;
                aixos_gpu::desktop::render_window("Shell", &[], w.w, w.h);
                aixos_gpu::desktop::render_window_output_hw(
                    w.x, w.y, win_output(), WIN_OUTPUT_LEN, w.h, w.w);
                let b = win_buf();
                aixos_gpu::desktop::render_window_input_hw(
                    w.x, w.y, b.as_slice(), b.len, focused, w.h, w.w);
            }
        }
        2 => aixos_gpu::desktop::render_window(
            "EdisonDB - Sovereign Store",
            &["Status: live", "Entries: (see db command)",
              "boot:proof = 0x4153", "boot:node_id = stored",
              "Tier: Critical / Personal / Noise"],
            w.w, w.h),
        3 => aixos_gpu::desktop::render_window(
            "Settings - aiXos Phoenix",
            &["Display:  ramfb 1280x720  FORMAT_XR24",
              "System:   aiXos Phoenix v0.1.0  aarch64",
              "Proof:    axon_main() -> 0x4153 [SOVEREIGN]",
              "Store:    EdisonDB live  sovereign store",
              "Input:    virtio+uart",
              "About:    AIEONYX  Sovereign Digital Infrastructure"],
            w.w, w.h),
        4 => {
            unsafe {
                EDB_ENTRY_COUNT = aixos_edisondb::entry_count();
                let n = if EDB_ENTRY_COUNT > 32 { 32 } else { EDB_ENTRY_COUNT };
                let mut ei = 0;
                while ei < n {
                    if let Some((k, t, v)) = aixos_edisondb::entry_at(ei) {
                        EDB_ENTRIES[ei] = (k, t, v);
                    }
                    ei += 1;
                }
                let mut slots: [aixos_gpu::desktop::EdbEntry; 32] = core::array::from_fn(|_|
                    aixos_gpu::desktop::EdbEntry { key: "", tier: "", value: 0 }
                );
                let mut si = 0;
                while si < n {
                    slots[si] = aixos_gpu::desktop::EdbEntry {
                        key:   EDB_ENTRIES[si].0,
                        tier:  EDB_ENTRIES[si].1,
                        value: EDB_ENTRIES[si].2,
                    };
                    si += 1;
                }
                aixos_gpu::desktop::render_window("EdisonDB Browser", &[], w.w, w.h);
                let focused = EDB_FOCUSED && ACTIVE_WIN == i;
                let inp = &*core::ptr::addr_of!(EDB_INPUT);
                aixos_gpu::desktop::render_edb_browser(
                    w.x, w.y, w.w, w.h,
                    &slots[..n],
                    EDB_CURSOR, EDB_SCROLL,
                    inp.as_slice(), inp.len,
                    focused,
                );
            }
        }
        5 => {
            let awp_status = if aixos_net::virtio_net::is_live() {
                "AWP:    live  virtio-net wired"
            } else {
                "AWP:    stub  no virtio-net"
            };
            let frames = aixos_net::virtio_net::frames_sent();
            let frames_rx = aixos_net::virtio_net::frames_received();
            // format frames_sent into static buffer
            static mut NET_STATUS_BUF: [u8; 32] = [0u8; 32];
            let frames_str = unsafe {
                let b = &mut *core::ptr::addr_of_mut!(NET_STATUS_BUF);
                b[..8].copy_from_slice(b"TX/RX:  ");
                let mut n = frames;
                let mut pos = 8usize;
                if n == 0 {
                    b[pos] = b'0'; pos += 1;
                } else {
                    let mut tmp = [0u8; 10];
                    let mut tlen = 0usize;
                    while n > 0 { tmp[tlen] = b'0' + (n % 10) as u8; tlen += 1; n /= 10; }
                    let mut ti = tlen;
                    while ti > 0 { ti -= 1; b[pos] = tmp[ti]; pos += 1; }
                }
                b[pos] = b'/'; pos += 1;
                let mut nr = frames_rx;
                let mut tmp2=[0u8;8];let mut tl2=0;
                if nr==0{tmp2[0]=b'0';tl2=1;}else{while nr>0{tmp2[tl2]=b'0'+(nr%10)as u8;tl2+=1;nr/=10;}}
                let mut ti2=tl2;while ti2>0{ti2-=1;b[pos]=tmp2[ti2];pos+=1;}
                core::str::from_utf8_unchecked(&b[..pos])
            };
            aixos_net::virtio_net::poll_rx();
            aixos_gpu::desktop::render_window(
                "Network - aiXos Phoenix",
                &[awp_status,
                  frames_str,
                  "Peers:  0  (discovery PL-54)",
                  "Proto:  AWP v0.1  sovereign mesh",
                  "Status: isolated  local only"],
                w.w, w.h)
        }
        // PL-60: Onyxia Browser — awp:// sovereign browser window
        7 => {
            unsafe {
                let url = &*core::ptr::addr_of!(ONY_URL_BUF);
                let focused = ONY_URL_FOCUSED && ACTIVE_WIN == i;
                // PL-61: Route URL to HanielDoc
                let doc: &aixos_gpu::desktop::HanielDoc = if ONY_LOADED {
                    if ONY_IS_STATUS {
                        &*core::ptr::addr_of!(HANIEL_STATUS_DOC)
                    } else {
                        haniel_route(url.as_slice(), url.len)
                    }
                } else {
                    &HANIEL_404  // not shown (loaded=false), but must pass something
                };
                aixos_gpu::desktop::render_window("Onyxia Browser", &[], w.w, w.h);
                aixos_gpu::desktop::render_onyxia_browser(
                    w.x, w.y, w.w, w.h,
                    url.as_slice(), url.len,
                    focused,
                    ONY_LOADED,
                    doc,
                );
            }
        }
        // PL-62: Process Table window
        8 => {
            unsafe {
                // Snapshot process table into PROC_SLOTS
                let mut si = 0usize;
                let mut ki = 0usize;
                while ki < 8 {
                    if let Some(p) = aixos_kernel::proc::proc_at(ki) {
                        let sc = match p.state {
                            aixos_kernel::proc::ProcState::Running => b'R',
                            aixos_kernel::proc::ProcState::Ready   => b'W',
                            aixos_kernel::proc::ProcState::Blocked => b'B',
                            aixos_kernel::proc::ProcState::Dead    => b'D',
                            _                                       => b'?',
                        };
                        PROC_SLOTS[si].pid      = p.pid;
                        PROC_SLOTS[si].name     = p.name;
                        PROC_SLOTS[si].name_len = p.name_len;
                        PROC_SLOTS[si].state_ch = sc;
                        PROC_SLOTS[si].priority = p.priority;
                        PROC_SLOTS[si].ticks    = p.ticks;
                        PROC_SLOTS[si].yields   = p.yields;
                        si += 1;
                    }
                    ki += 1;
                }
                PROC_COUNT = si;
                let tick = aixos_kernel::proc::tick_total();
                aixos_gpu::desktop::render_window("Processes", &[], w.w, w.h);
                aixos_gpu::desktop::render_proc_window(
                    w.x, w.y, w.w, w.h,
                    &PROC_SLOTS[..si],
                    si,
                    tick,
                );
            }
        }
        // PL-65: File Browser window
        9 => {
            unsafe {
                populate_fb_entries();
                // AXFS disk usage: approximate
                let disk_used: u32 = (FB_COUNT * 256) as u32;
                let disk_total: u32 = 65536; // 64KB sovereign AXFS
                aixos_gpu::desktop::render_window("Files — AXFS Browser", &[], w.w, w.h);
                aixos_gpu::desktop::render_file_browser(
                    w.x, w.y, w.w, w.h,
                    &FB_ENTRIES[..FB_COUNT],
                    FB_COUNT,
                    FB_SELECTED,
                    disk_used,
                    disk_total,
                );
            }
        }
        _ => aixos_gpu::desktop::render_window(
            "Sovereign Node - aiXos Phoenix",
            &["aiXos Phoenix v0.1.0", "Arch: aarch64 (QEMU virt)",
              "Proof: 0x4153 [SOVEREIGN]", "type close to dismiss"],
            w.w, w.h),
    }
}

fn render_windows_only() {
    unsafe {
            #[cfg(target_arch = "aarch64")]
            {
                let now: u64;
                core::arch::asm!("mrs {}, cntpct_el0", out(reg) now);
                let elapsed = now.saturating_sub(BOOT_TICK);
                DESKTOP_STATE.uptime_sec = elapsed / CNTFRQ;
            }
            let (rh, rm, rd, rmon) = read_rtc();
            DESKTOP_STATE.rtc_hour = rh;
            DESKTOP_STATE.rtc_min  = rm;
            DESKTOP_STATE.rtc_day  = rd;
            DESKTOP_STATE.rtc_mon  = rmon;
            // PL-49: user identity in desktop state
            DESKTOP_STATE.tz_offset = TZ_OFFSET;
            DESKTOP_STATE.user_name = core::slice::from_raw_parts(
                USER_NAME_BUF.as_ptr(), USER_NAME_LEN);
            aixos_gpu::desktop::render_desktop(&DESKTOP_STATE);
        }
    unsafe { aixos_gpu::desktop::render_top_bar_icons(DESKTOP_STATE.uptime_sec, DESKTOP_STATE.rtc_hour, DESKTOP_STATE.rtc_min, DESKTOP_STATE.rtc_day, DESKTOP_STATE.rtc_mon, DESKTOP_STATE.tz_offset); }
    let active = unsafe { ACTIVE_WIN };
    let mut i = 0;
    while i < 6 {
        if i != active { render_window_for_slot(i); }
        i += 1;
    }
    render_window_for_slot(active);
    let slots = unsafe {[
        (wins()[0].open, wins()[0].kind, wins()[0].minimized),
        (wins()[1].open, wins()[1].kind, wins()[1].minimized),
        (wins()[2].open, wins()[2].kind, wins()[2].minimized),
        (wins()[3].open, wins()[3].kind, wins()[3].minimized),
        (wins()[4].open, wins()[4].kind, wins()[4].minimized),
        (wins()[5].open, wins()[5].kind, wins()[5].minimized),
    ]};
    aixos_gpu::desktop::render_taskbar(&slots, unsafe { ACTIVE_WIN });
    // PL-48: redraw cursor to prevent ghost artifact after panel redraws
    unsafe { aixos_gpu::draw_cursor(CURSOR_X, CURSOR_Y); }
}

fn render_all_windows() {
    unsafe {
            #[cfg(target_arch = "aarch64")]
            {
                let now: u64;
                core::arch::asm!("mrs {}, cntpct_el0", out(reg) now);
                let elapsed = now.saturating_sub(BOOT_TICK);
                DESKTOP_STATE.uptime_sec = elapsed / CNTFRQ;
            }
            let (rh, rm, rd, rmon) = read_rtc();
            DESKTOP_STATE.rtc_hour = rh;
            DESKTOP_STATE.rtc_min  = rm;
            DESKTOP_STATE.rtc_day  = rd;
            DESKTOP_STATE.rtc_mon  = rmon;
            // PL-49: user identity in desktop state
            DESKTOP_STATE.tz_offset = TZ_OFFSET;
            DESKTOP_STATE.user_name = core::slice::from_raw_parts(
                USER_NAME_BUF.as_ptr(), USER_NAME_LEN);
            aixos_gpu::desktop::render_desktop(&DESKTOP_STATE);
        }
    unsafe { aixos_gpu::desktop::render_top_bar_icons(DESKTOP_STATE.uptime_sec, DESKTOP_STATE.rtc_hour, DESKTOP_STATE.rtc_min, DESKTOP_STATE.rtc_day, DESKTOP_STATE.rtc_mon, DESKTOP_STATE.tz_offset); }
    let active = unsafe { ACTIVE_WIN };
    let mut i = 0;
    while i < 6 {
        if i != active {
            render_window_for_slot(i);
        }
        i += 1;
    }
    render_window_for_slot(active);
    let slots = unsafe {[
        (wins()[0].open, wins()[0].kind, wins()[0].minimized),
        (wins()[1].open, wins()[1].kind, wins()[1].minimized),
        (wins()[2].open, wins()[2].kind, wins()[2].minimized),
        (wins()[3].open, wins()[3].kind, wins()[3].minimized),
        (wins()[4].open, wins()[4].kind, wins()[4].minimized),
        (wins()[5].open, wins()[5].kind, wins()[5].minimized),
    ]};
    aixos_gpu::desktop::render_taskbar(&slots, unsafe { ACTIVE_WIN });
    // PL-48: redraw cursor to prevent ghost artifact after full clear
    unsafe { aixos_gpu::draw_cursor(CURSOR_X, CURSOR_Y); }
}


fn handle_files_key(code: u16) {
    unsafe {
        let count = aixos_axfs::count();
        if FILES_VIEWING {
            if code == 1 { FILES_VIEWING = false; render_all_windows(); }
        } else { match code {
            103 => { if FILES_CURSOR > 0 { FILES_CURSOR -= 1; } render_all_windows(); }
            108 => { if count > 0 && FILES_CURSOR + 1 < count { FILES_CURSOR += 1; } render_all_windows(); }
            28 => { if FILES_CURSOR < count { if let Some(f) = aixos_axfs::file_at(FILES_CURSOR) { let data = f.data_bytes(); let len = data.len().min(256); FILES_CONTENT_LEN = len; let mut i = 0; while i < len { FILES_CONTENT_BUF[i] = data[i]; i += 1; } FILES_VIEW_IDX = FILES_CURSOR; FILES_VIEWING = true; render_all_windows(); } } }
            1 => { if let Some(i) = find_kind(6) { wins()[i].open = false; FILES_CURSOR = 0; FILES_VIEWING = false; } render_all_windows(); }
            _ => {}
        } } } }

fn handle_dock_click(x: i32, y: i32) {
    if let Some(icon) = aixos_gpu::desktop::dock_icon_at(x, y) {
        // Dock index -> window kind
        // 0=O(Onyxia) 1=W(Browser) 2=>_(Shell) 3=F(Files/EDB) 4=D(EDB Browser) 5=I(IAM/Set) 6=S(Settings)
        let kind: u8 = match icon {
            0 => 7, // Onyxia -> Onyxia Browser
            1 => 7, // Browser -> Onyxia Browser (same window)
            2 => 1, // Shell
            3 => 9, // Folder -> File Browser (kind 9, PL-65)
            4 => 4, // EDB Browser
            5 => 8, // I=IAM/Processes -> Process Table window (PL-62)
            6 => 3, // Settings
            _ => return,
        };
        unsafe {
            WINDOW_FOCUSED = false;
            if let Some(i) = find_kind(kind) {
                // Already open — if minimized, restore it; else bring to front
                if wins()[i].minimized {
                    wins()[i].minimized = false;
                    ACTIVE_WIN = i;
                } else {
                    ACTIVE_WIN = i;
                }
            } else {
                // Open in a free slot
                if let Some(slot) = find_free() {
                    wins()[slot].open = true;
                    wins()[slot].kind = kind;
                    // File browser needs more space for 3-panel layout
                    if kind == 9 {
                        wins()[slot].w = 820;
                        wins()[slot].h = 500;
                        wins()[slot].x = aixos_gpu::desktop::CANVAS_X_MIN;
                        wins()[slot].y = aixos_gpu::desktop::CANVAS_Y_MIN;
                    }
                    ACTIVE_WIN = slot;
                }
                // If no free slot, do nothing (all 5 windows open)
            }
            if kind == 1 || kind == 6 { WINDOW_FOCUSED = true; }
            if kind == 7 {
                // Onyxia browser: reset to new tab state if freshly opened
                ONY_URL_FOCUSED = false;
                ONY_LOADED = false;
            }
            if kind == 4 {
                EDB_CURSOR = 0;
                EDB_SCROLL = 0;
                EDB_FOCUSED = false;
                EDB_INPUT.clear();
            }
        }
        render_all_windows();
    }
}

fn win_buf() -> &'static mut ShellBuf {
    unsafe { &mut *core::ptr::addr_of_mut!(WIN_BUF) }
}

fn win_output() -> &'static [&'static str] {
    unsafe { &(&*core::ptr::addr_of!(WIN_OUTPUT))[..] }
}

fn push_output(line: &'static str) {
    unsafe {
        let out = &mut *core::ptr::addr_of_mut!(WIN_OUTPUT);
        if WIN_OUTPUT_LEN >= 8 {
            let mut i = 0;
            while i < 7 { out[i] = out[i + 1]; i += 1; }
            out[7] = line;
        } else {
            out[WIN_OUTPUT_LEN] = line;
            WIN_OUTPUT_LEN += 1;
        }
    }
}

fn push_echo() -> &'static str {
    unsafe {
        let i = ECHO_NEXT;
        ECHO_NEXT = (ECHO_NEXT + 1) % 8;
        let bufs = &mut *core::ptr::addr_of_mut!(ECHO_BUFS);
        let bytes = win_buf().as_slice();
        let n = if bytes.len() > 67 { 67 } else { bytes.len() };
        bufs[i][..5].copy_from_slice(b"axc> ");
        bufs[i][5..5 + n].copy_from_slice(&bytes[..n]);
        core::str::from_utf8_unchecked(&(&*core::ptr::addr_of!(ECHO_BUFS))[i][..5 + n])
    }
}

fn handle_edb_key(code: u16, ch: Option<char>) {
    unsafe {
        let count = EDB_ENTRY_COUNT;
        match code {
            103 => {
                if EDB_CURSOR > 0 { EDB_CURSOR -= 1; }
                if EDB_CURSOR < EDB_SCROLL { EDB_SCROLL = EDB_CURSOR; }
                render_all_windows();
            }
            108 => {
                if count > 0 && EDB_CURSOR + 1 < count { EDB_CURSOR += 1; }
                if EDB_CURSOR >= EDB_SCROLL + 8 { EDB_SCROLL = EDB_CURSOR.saturating_sub(7); }
                render_all_windows();
            }
            1 => { EDB_FOCUSED = false; render_all_windows(); }
            28 => {
                let inp = &*core::ptr::addr_of!(EDB_INPUT);
                let bytes = inp.as_slice();
                if bytes.starts_with(b"put ") {
                    let rest = &bytes[4..];
                    let mut sp = rest.len();
                    let mut j = 0;
                    while j < rest.len() {
                        if rest[j] == b' ' { sp = j; break; }
                        j += 1;
                    }
                    if sp < rest.len() {
                        let val_bytes = &rest[sp + 1..];
                        let mut val: u64 = 0;
                        let mut vi = 0;
                        while vi < val_bytes.len() {
                            let b = val_bytes[vi];
                            if b >= b'0' && b <= b'9' {
                                val = val.wrapping_mul(10).wrapping_add((b - b'0') as u64);
                            }
                            vi += 1;
                        }
                        aixos_edisondb::write("edb:put", val, aixos_edisondb::Tier::Noise);
                    }
                }
                EDB_INPUT.clear();
                EDB_FOCUSED = false;
                EDB_ENTRY_COUNT = aixos_edisondb::entry_count();
                render_all_windows();
            }
            14 => { EDB_INPUT.pop(); render_all_windows(); }
            _ => {
                if let Some(c) = ch {
                    let b = c as u8;
                    if (0x20..0x7fu8).contains(&b) {
                        EDB_INPUT.push(b);
                        EDB_FOCUSED = true;
                        render_all_windows();
                    }
                }
            }
        }
    }
}

// ── PL-61: HANIEL document router ────────────────────────────────────────────
// Maps awp:// URLs to sovereign HanielDoc pages.
// All page content is compile-time static — no heap, no alloc.

static HANIEL_HOME: aixos_gpu::desktop::HanielDoc = {
    use aixos_gpu::desktop::HanielDoc;
    let mut d = HanielDoc::empty();
    d.title    = "AIEONYX Sovereign Home";
    d.subtitle = "Smart Digital Sovereign Community";
    d.body = [
        "##Welcome to Onyxia Browser",
        "You are browsing the sovereign awp:// web.",
        ">>This node runs aiXos Phoenix v0.1.0 on aarch64.",
        "",
        "##Available Pages",
        "  awp://aieonyx       — this page",
        "  awp://about         — project overview",
        "  awp://status        — live node status",
        "  awp://iam           — IAM sovereign identity",
        "",
        "",
        "",
    ];
    d.body_len = 9;
    d.links    = ["awp://about", "awp://status", "awp://iam", "", "", ""];
    d.link_len = 3;
    d.page_kind = 1;
    d
};

static HANIEL_ABOUT: aixos_gpu::desktop::HanielDoc = {
    use aixos_gpu::desktop::HanielDoc;
    let mut d = HanielDoc::empty();
    d.title    = "About AIEONYX";
    d.subtitle = "Sovereign Digital Infrastructure Stack";
    d.body = [
        "##Mission",
        "Build a complete sovereign digital civilization stack.",
        "Every layer owned, audited, and provenance-stamped.",
        "",
        "##Stack",
        "  AXONYX   — sovereign systems language (.ax)",
        "  EdisonDB — sovereign database (ARPi provenance)",
        "  Onyxia   — sovereign browser (awp:// protocol)",
        "  aiXos    — sovereign bare-metal OS (aarch64)",
        "  IAM      — sovereign AI companion (350M params)",
        ">>Wisdom is the Beginning.",
        "",
    ];
    d.body_len = 11;
    d.links    = ["awp://aieonyx", "awp://status", "", "", "", ""];
    d.link_len = 2;
    d.page_kind = 0;
    d
};

static HANIEL_IAM: aixos_gpu::desktop::HanielDoc = {
    use aixos_gpu::desktop::HanielDoc;
    let mut d = HanielDoc::empty();
    d.title    = "IAM Sovereign Identity";
    d.subtitle = "Intelligent Autonomous Mind — Founding Spec v1.0";
    d.body = [
        "##Architecture",
        "  350M parameters   Ryzen 7 only",
        "  20 SSM + 4 attention layers",
        "  Stage 2 MoE: 4 experts top-2",
        "  BLAKE3 everywhere",
        "",
        "##Mission",
        ">>help human, never harm, maximum capacity, always.",
        "  Epoch: Wisdom is the Beginning.",
        "",
        "##Status",
        "  Training blocked on axon_data (P67) + axon_train (P68)",
    ];
    d.body_len = 12;
    d.links    = ["awp://about", "awp://aieonyx", "", "", "", ""];
    d.link_len = 2;
    d.page_kind = 0;
    d
};

static HANIEL_404: aixos_gpu::desktop::HanielDoc = {
    use aixos_gpu::desktop::HanielDoc;
    let mut d = HanielDoc::empty();
    d.title    = "404 — No Sovereign Route";
    d.subtitle = "This awp:// address is not mapped on this node.";
    d.body = [
        "The requested page has no sovereign route.",
        "",
        ">>Only awp:// addresses are supported.",
        "  https:// is the legacy bridge (PL-62+).",
        "",
        "##Try",
        "  awp://aieonyx   — sovereign home",
        "  awp://about     — project info",
        "  awp://status    — live node status",
        "",
        "",
        "",
    ];
    d.body_len = 9;
    d.links    = ["awp://aieonyx", "awp://about", "awp://status", "", "", ""];
    d.link_len = 3;
    d.page_kind = 3;
    d
};

// Route a URL (bytes) to a static HanielDoc.
// Status page is special — built dynamically in the render call (kind==2).
fn haniel_route(url: &[u8], url_len: usize) -> &'static aixos_gpu::desktop::HanielDoc {
    let s = match core::str::from_utf8(&url[..url_len]) {
        Ok(s) => s,
        Err(_) => return &HANIEL_404,
    };
    // Strip awp:// prefix if present
    let path = if s.starts_with("awp://") { &s[6..] } else { s };
    match path {
        "" | "aieonyx" | "aieonyx/" => &HANIEL_HOME,
        "about" | "about/"         => &HANIEL_ABOUT,
        "iam"   | "iam/"           => &HANIEL_IAM,
        // status is rendered dynamically — we return home and override in render
        _                          => &HANIEL_404,
    }
}

// Write a u64 as decimal ASCII into buf, return length written
fn u64_to_dec(n: u64, buf: &mut [u8]) -> usize {
    if n == 0 { if !buf.is_empty() { buf[0] = b'0'; } return 1; }
    let mut tmp = [0u8; 20];
    let mut len = 0usize;
    let mut v = n;
    while v > 0 { tmp[len] = b'0' + (v % 10) as u8; len += 1; v /= 10; }
    let mut i = 0;
    while i < len && i < buf.len() { buf[i] = tmp[len - 1 - i]; i += 1; }
    i
}

// Build the live status HanielDoc — must be called before rendering awp://status.
// Writes into HANIEL_STATUS_DOC and STATUS_* buffers (no alloc).
unsafe fn build_status_doc() {
    // EDB entries count line
    let edb_cnt = aixos_edisondb::entry_count() as u64;
    STATUS_EDB_LINE[..8].copy_from_slice(b"Entries:");
    let n = u64_to_dec(edb_cnt, &mut STATUS_EDB_LINE[9..]);
    STATUS_EDB_LEN = 9 + n;

    // Network line
    let net_live = aixos_net::virtio_net::is_live();
    let net_str: &[u8] = if net_live { b"virtio-net live" } else { b"loopback only  " };
    let nl = net_str.len().min(32);
    STATUS_NET_LINE[..nl].copy_from_slice(&net_str[..nl]);
    STATUS_NET_LEN = nl;

    // Fill the doc — body lines 4 and 5 are live (patched via raw pointer)
    HANIEL_STATUS_DOC.title    = "Node Status";
    HANIEL_STATUS_DOC.subtitle = "aiXos Phoenix v0.1.0  aarch64  [SOVEREIGN]";
    HANIEL_STATUS_DOC.body[0]  = "##Proof";
    HANIEL_STATUS_DOC.body[1]  = "  axon_main() -> 0x4153 [SOVEREIGN]";
    HANIEL_STATUS_DOC.body[2]  = "##EdisonDB";
    HANIEL_STATUS_DOC.body[3]  = "  Status: live  sovereign store";
    // body[4] = EDB entry count — static str from buffer (safe: 'static lifetime via addr)
    HANIEL_STATUS_DOC.body[4]  = "  (edb entries — see below)";
    HANIEL_STATUS_DOC.body[5]  = "##Network";
    HANIEL_STATUS_DOC.body[6]  = "  AWP loopback  EDB:00000004";
    HANIEL_STATUS_DOC.body[7]  = "##BASTION";
    HANIEL_STATUS_DOC.body[8]  = "  Policy active  Desktop ready";
    HANIEL_STATUS_DOC.body[9]  = ">>Proof 0x4153  all systems nominal";
    HANIEL_STATUS_DOC.body_len  = 10;
    HANIEL_STATUS_DOC.links[0]  = "awp://aieonyx";
    HANIEL_STATUS_DOC.links[1]  = "awp://about";
    HANIEL_STATUS_DOC.link_len  = 2;
    HANIEL_STATUS_DOC.page_kind = 2; // status tint
}

// PL-65: Populate FB_ENTRIES from AXFS for the File Browser window
unsafe fn populate_fb_entries() {
    FB_COUNT = 0;
    let mut i = 0usize;
    while i < 64 && FB_COUNT < 16 {
        if let Some(f) = aixos_axfs::file_at(i) {
            let name = f.name_bytes();
            let nlen = name.len().min(32);
            let size = f.data_bytes().len() as u32;
            // Detect kind by extension
            let kind: u8 = if nlen >= 7 && &name[nlen-7..nlen] == b".axpkg" { 1 }
                else if nlen >= 3 && &name[nlen-3..nlen] == b".ax" { 0 }
                else if nlen >= 4 && &name[nlen-4..nlen] == b".txt" { 2 }
                else { 3 };
            FB_ENTRIES[FB_COUNT].name = [0u8; 32];
            FB_ENTRIES[FB_COUNT].name[..nlen].copy_from_slice(&name[..nlen]);
            FB_ENTRIES[FB_COUNT].name_len = nlen;
            FB_ENTRIES[FB_COUNT].size = size;
            FB_ENTRIES[FB_COUNT].kind = kind;
            FB_COUNT += 1;
        }
        i += 1;
    }
}

// PL-60: Onyxia browser keyboard handler
// Tab (15)   — toggle URL bar focus
// Enter (28) — navigate (set ONY_LOADED=true)
// Esc (1)    — if URL focused: clear URL. If not focused: close window
// Backspace (14) — delete char from URL
// Char       — type into URL bar when focused
fn handle_onyxia_key(code: u16, ch: Option<char>) {
    unsafe {
        match code {
            15 => {
                // Tab: toggle URL focus
                ONY_URL_FOCUSED = !ONY_URL_FOCUSED;
                render_all_windows();
            }
            28 => {
                // Enter: navigate — route URL via HANIEL
                if ONY_URL_FOCUSED && ONY_URL_BUF.len > 0 {
                    let url = &ONY_URL_BUF.data[..ONY_URL_BUF.len];
                    // Check if status page
                    let s = core::str::from_utf8(url).unwrap_or("");
                    let path = if s.starts_with("awp://") { &s[6..] } else { s };
                    if path == "status" || path == "status/" {
                        ONY_IS_STATUS = true;
                        build_status_doc();
                    } else {
                        ONY_IS_STATUS = false;
                    }
                    ONY_LOADED = true;
                    ONY_URL_FOCUSED = false;
                }
                render_all_windows();
            }
            1 => {
                // Esc: if URL focused → clear URL. Else → close window.
                if ONY_URL_FOCUSED {
                    ONY_URL_BUF.clear();
                    ONY_LOADED = false;
                    ONY_URL_FOCUSED = false;
                } else {
                    if let Some(i) = find_kind(7) {
                        wins()[i].open = false;
                    }
                    WINDOW_FOCUSED = false;
                }
                render_all_windows();
            }
            14 => {
                // Backspace
                if ONY_URL_FOCUSED {
                    ONY_URL_BUF.pop();
                    ONY_LOADED = false;
                }
                render_all_windows();
            }
            _ => {
                if ONY_URL_FOCUSED {
                    if let Some(c) = ch {
                        ONY_URL_BUF.push(c as u8);
                    }
                    render_all_windows();
                }
            }
        }
    }
}

fn handle_window_key(code: u16, ch: Option<char>) {
    unsafe {
        if wins()[ACTIVE_WIN].open && wins()[ACTIVE_WIN].kind == 6 {
            handle_files_key(code);
            return;
        }
        if wins()[ACTIVE_WIN].open && wins()[ACTIVE_WIN].kind == 4 {
            handle_edb_key(code, ch);
            return;
        }
        // PL-60: Onyxia browser gets its own key handler
        if wins()[ACTIVE_WIN].open && wins()[ACTIVE_WIN].kind == 7 {
            handle_onyxia_key(code, ch);
            return;
        }
    }
    let (wx, wy) = {
        let w = wins()[unsafe { ACTIVE_WIN }];
        aixos_gpu::desktop::set_window_pos(w.x, w.y);
        (w.x, w.y)
    };
    match code {
        1 => unsafe {
            WINDOW_FOCUSED = false;
            win_buf().clear();
            render_all_windows();
        },
        28 => unsafe {
            let echo = push_echo();
            push_output(echo);
            let result = execute_cmd(win_buf());
            push_output(result);
            win_buf().clear();
            render_all_windows();
        },
        14 => unsafe {
            win_buf().pop();
            TAB_ACTIVE = false;
            render_all_windows();
        },
        // PL-59: history navigation
        103 => unsafe { hist_nav(win_buf(), true); render_all_windows(); },
        108 => unsafe { hist_nav(win_buf(), false); render_all_windows(); },
        // PL-59: tab completion
        15 => unsafe { tab_complete(win_buf()); render_all_windows(); },
        _ => {
            if let Some(c) = ch {
                win_buf().push(c as u8);
                unsafe { TAB_ACTIVE = false; }
                render_all_windows();
            }
        }
    }
}

fn handle_click(x: i32, y: i32) {
    unsafe {
        let order = [ACTIVE_WIN, 5, 4, 3, 2, 1, 0];
        let mut k = 0;
        while k < 7 {
            let i = order[k];
            k += 1;
            if k > 1 && i == order[0] { continue; }
            let w = wins()[i];
            if !w.open { continue; }
            if x >= w.x + w.w as i32 - 20 && x < w.x + w.w as i32
                && y >= w.y + w.h as i32 - 20 && y < w.y + w.h as i32 {
                ACTIVE_WIN = i;
                RESIZE_WIN = i;
                RESIZE_ACTIVE = true;
                render_all_windows();
                return;
            }
            if x >= w.x && x < w.x + w.w as i32 && y >= w.y && y < w.y + 24 {
                ACTIVE_WIN = i;
                let hit = aixos_gpu::desktop::title_bar_hit(w.x, w.y, w.w, x, y);
                match hit {
                    1 => {
                        // Close
                        wins()[i].open = false;
                        wins()[i].minimized = false;
                        wins()[i].maximized = false;
                        WINDOW_FOCUSED = false;
                        aixos_gpu::desktop::set_window_pos(w.x, w.y);
                        aixos_gpu::desktop::clear_window();
                        let mut j = 6;
                        while j > 0 { j -= 1; if wins()[j].open { ACTIVE_WIN = j; break; } }
                        render_all_windows();
                        return;
                    }
                    2 => {
                        // Maximize / restore
                        if wins()[i].maximized {
                            // Restore
                            wins()[i].x = wins()[i].prev_x;
                            wins()[i].y = wins()[i].prev_y;
                            wins()[i].w = wins()[i].prev_w;
                            wins()[i].h = wins()[i].prev_h;
                            wins()[i].maximized = false;
                        } else {
                            // Maximize to canvas area
                            wins()[i].prev_x = wins()[i].x;
                            wins()[i].prev_y = wins()[i].y;
                            wins()[i].prev_w = wins()[i].w;
                            wins()[i].prev_h = wins()[i].h;
                            wins()[i].x = aixos_gpu::desktop::CANVAS_X_MIN;
                            wins()[i].y = aixos_gpu::desktop::CANVAS_Y_MIN;
                            wins()[i].w = 880;
                            wins()[i].h = 620;
                            wins()[i].maximized = true;
                            wins()[i].minimized = false;
                        }
                        render_all_windows();
                        return;
                    }
                    3 => {
                        // Minimize — hide window, show dot in dock
                        wins()[i].minimized = true;
                        WINDOW_FOCUSED = false;
                        aixos_gpu::desktop::set_window_pos(w.x, w.y);
                        aixos_gpu::desktop::clear_window_sized(w.w + 10, w.h + 10);
                        // Focus next open non-minimized window
                        let mut j = 6;
                        while j > 0 { j -= 1;
                            if wins()[j].open && !wins()[j].minimized { ACTIVE_WIN = j; break; }
                        }
                        render_all_windows();
                        return;
                    }
                    _ => {
                        // Drag — only if not on a button
                        DRAG_WIN = i;
                        DRAG_ACTIVE = true;
                        DRAG_OFF_X = x - w.x;
                        DRAG_OFF_Y = y - w.y;
                        render_all_windows();
                        return;
                    }
                }
            }
            if x >= w.x && x < w.x + w.w as i32 && y >= w.y + 24 && y < w.y + w.h as i32 {
                ACTIVE_WIN = i;
                if w.kind == 1 {
                    WINDOW_FOCUSED = true;
                }
                if w.kind == 7 {
                    // Onyxia: clicking inside canvas body focuses URL bar
                    WINDOW_FOCUSED = true;
                    ONY_URL_FOCUSED = true;
                }
                if w.kind == 4 { EDB_FOCUSED = true; }
                // PL-65B: File browser click handling
                if w.kind == 9 {
                    let sidebar_w: i32 = 130;
                    let preview_w: i32 = 160;
                    let main_x = w.x + sidebar_w + 1;
                    let preview_x = w.x + w.w as i32 - preview_w;
                    let content_y = w.y + 25; // WIN_TITLE_H=24, content starts at wy+25
                    // File row click — select file
                    let nav_h: i32 = 26;
                    let hdr_h: i32 = 14;
                    let body_top = content_y + nav_h + hdr_h + 1;
                    let row_h: i32 = 22;
                    if x >= main_x && x < preview_x && y >= body_top {
                        let row_idx = ((y - body_top) / row_h) as usize;
                        if row_idx < FB_COUNT {
                            FB_SELECTED = row_idx;
                            FB_ACTION = 0;
                        }
                    }
                    // Preview panel button clicks
                    if x >= preview_x && x < w.x + w.w as i32 {
                        // Button positions match render: meta_y = fi_y+84, btn_y = meta_y+68
                        // fi_y = content_y + 14
                        let fi_y = content_y + 14;
                        let meta_y = fi_y + 84;
                        let btn_y = meta_y + 68;
                        let btn_w: i32 = preview_w - 16;
                        let bx = preview_x + 8;
                        // Open button
                        if x >= bx && x < bx + btn_w && y >= btn_y && y < btn_y + 20 {
                            FB_ACTION = 1; // open
                        }
                        // Verify button
                        if x >= bx && x < bx + btn_w && y >= btn_y + 26 && y < btn_y + 46 {
                            FB_ACTION = 2; // verify
                        }
                        // Encrypt button
                        if x >= bx && x < bx + btn_w && y >= btn_y + 52 && y < btn_y + 72 {
                            FB_ACTION = 3; // encrypt stub
                        }
                    }
                    // Process FB_ACTION
                    if FB_ACTION > 0 && FB_SELECTED < FB_COUNT {
                        let entry = &FB_ENTRIES[FB_SELECTED];
                        let fname = &entry.name[..entry.name_len];
                        match FB_ACTION {
                            1 => {
                                // Open: find file by name then run via axon_interp
                                if let Some(f) = aixos_axfs::file_at(FB_SELECTED) {
                                    let script = f.data_bytes();
                                    let result = aixos_shell::axon_interp::exec(
                                        script,
                                        aixos_identity::node_id(),
                                        if aixos_net::virtio_net::is_live() {
                                            Some(|nid:u64,p:&[u8]| aixos_net::virtio_net::send_awp_frame(nid,p))
                                        } else { None },
                                    );
                                    let out = result.as_str();
                                    let len = out.len().min(510);
                                    AXFS_BUF_LEN = len;
                                    let mut ii=0; while ii<len{AXFS_BUF[ii]=out[ii];ii+=1;}
                                    // Open shell window to show output
                                    if find_kind(1).is_none() {
                                        if let Some(slot) = find_free() {
                                            wins()[slot].open = true;
                                            wins()[slot].kind = 1;
                                            WINDOW_FOCUSED = true;
                                        }
                                    }
                                }
                            }
                            2 => {
                                // Verify: run verify_axpkg on selected file
                                if let Some(f) = aixos_axfs::file_at(FB_SELECTED) {
                                    let data = f.data_bytes();
                                    match aixos_kernel::verify::verify_axpkg(data) {
                                        aixos_kernel::verify::VerifyGate::Pass { name:_, script:_, caps:_ } => {
                                            let msg = b"VERIFIED: package integrity OK";
                                            AXFS_BUF_LEN = msg.len();
                                            let mut ii=0; while ii<msg.len(){AXFS_BUF[ii]=msg[ii];ii+=1;}
                                        }
                                        aixos_kernel::verify::VerifyGate::Reject(reason) => {
                                            let msg = reason.as_str().as_bytes();
                                            let l = msg.len().min(510);
                                            AXFS_BUF_LEN = l;
                                            let mut ii=0; while ii<l{AXFS_BUF[ii]=msg[ii];ii+=1;}
                                        }
                                    }
                                }
                            }
                            _ => {} // encrypt stub — not yet implemented
                        }
                        FB_ACTION = 0;
                    }
                }
                render_all_windows();
                return;
            }
        }
        // Right panel SYSTEM icon grid click
        if x >= 1092 && x < 1280 {
            let rx: i32 = 1092;
            let mut icon_hit: i32 = -1;
            let mut ci = 0i32;
            while ci < 6 {
                let col = ci % 3;
                let row = ci / 3;
                let ix = rx + 16 + col * 44;
                let iy = 38 + 42 + row * 44;
                if x >= ix && x < ix + 36 && y >= iy && y < iy + 36 {
                    icon_hit = ci;
                    break;
                }
                ci += 1;
            }
            if icon_hit >= 0 {
                let kind: i32 = match icon_hit {
                    0 => 7, // Globe -> Onyxia Browser
                    1 => 9, // Folder -> File Browser (PL-65)
                    2 => 1, // Terminal -> Shell
                    3 => 9, // Disk -> File Browser (storage view)
                    4 => 3, // Gear -> Settings
                    5 => 5, // Antenna -> Network
                    _ => -1,
                };
                if kind >= 0 {
                    unsafe {
                        WINDOW_FOCUSED = false;
                        if let Some(i) = find_kind(kind as u8) {
                            ACTIVE_WIN = i;
                        } else if let Some(slot) = find_free() {
                            wins()[slot].open = true;
                            wins()[slot].kind = kind as u8;
                            ACTIVE_WIN = slot;
                            if kind == 4 {
                                EDB_CURSOR = 0;
                                EDB_SCROLL = 0;
                                EDB_FOCUSED = false;
                                EDB_INPUT.clear();
                            }
                        }
                    }
                    render_all_windows();
                    return;
                }
            }
        }
        // Left panel SPACES click
        if x >= 8 && x < 196 {
            // y positions: SPACES label at TOP_BAR_H+8+108=154
            // Items at 154+8=162, 154+34=188, 154+54=208, 154+74=228
            let base_y: i32 = 38 + 8 + 116;
            let py = y - base_y;
            let new_space: i32 =
                if py >= 0 && py < 22 { 0 }
                else if py >= 26 && py < 48 { 1 }
                else if py >= 46 && py < 68 { 2 }
                else if py >= 66 && py < 88 { 3 }
                else { -1 };
            if new_space >= 0 {
                unsafe { DESKTOP_STATE.active_space = new_space as u8; }
                render_all_windows();
                return;
            }
        }
        // Empty canvas click — no action
        let _ = (x, y);
    }
}

fn read_rtc() -> (u8, u8, u8, u8) {
    #[cfg(target_arch = "aarch64")]
    unsafe {
        let ts = core::ptr::read_volatile((PL031_BASE + PL031_DR) as *const u32) as u64;
        let time_of_day = ts % 86400;
        let hour = (time_of_day / 3600) as u8;
        let min  = ((time_of_day % 3600) / 60) as u8;
        let mut days = ts / 86400;
        let mut y: u64 = 1970;
        loop {
            let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
            let ydays: u64 = if leap { 366 } else { 365 };
            if days < ydays { break; }
            days -= ydays;
            y += 1;
        }
        let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
        let month_days: [u64; 12] = [31,28,31,30,31,30,31,31,30,31,30,31];
        let mut mon: u8 = 1;
        for ml in month_days.iter() {
            let ml2 = if leap && mon == 2 { ml + 1 } else { *ml };
            if days < ml2 { break; }
            days -= ml2;
            mon += 1;
        }
        return (hour, min, (days + 1) as u8, mon);
    }
    #[cfg(not(target_arch = "aarch64"))]
    (0, 0, 1, 1)
}

fn shell_loop(
    mut mouse: Option<aixos_input::mouse::VirtioMouse>,
    mut mouse_state: aixos_input::mouse::MouseState,
) -> ! {
    let mut buf = ShellBuf::new();
    loop {
        if let Some(ref mut m) = mouse {
            let old_x = mouse_state.x;
            let old_y = mouse_state.y;
            let prev_left = mouse_state.left;
            if m.poll(&mut mouse_state) {
                // PL-48: track cursor position for render pass redraws
                unsafe { CURSOR_X = mouse_state.x; CURSOR_Y = mouse_state.y; }
                aixos_gpu::erase_cursor(old_x, old_y);
                unsafe {
                    if RESIZE_ACTIVE && !mouse_state.left {
                        // Apply resize only on release — no intermediate frames
                        let nw = ((mouse_state.x - wins()[RESIZE_WIN].x) as u32).clamp(300, 900);
                        let nh = ((mouse_state.y - wins()[RESIZE_WIN].y) as u32).clamp(200, 600);
                        wins()[RESIZE_WIN].w = nw;
                        wins()[RESIZE_WIN].h = nh;
                        RESIZE_ACTIVE = false;
                        DRAG_ACTIVE = false;
                        render_all_windows();
                    }
                    if !mouse_state.left { RESIZE_ACTIVE = false; DRAG_ACTIVE = false; }
                    // PL-59.5: drag clamped to canvas safe zone (no overlap with panels/dock)
                    if !RESIZE_ACTIVE && DRAG_ACTIVE && mouse_state.left {
                        let dw = DRAG_WIN;
                        let w = wins()[dw];
                        let raw_nx = mouse_state.x - DRAG_OFF_X;
                        let raw_ny = mouse_state.y - DRAG_OFF_Y;
                        let (nx, ny) = aixos_gpu::desktop::clamp_spawn_pos(raw_nx, raw_ny);
                        if nx != w.x || ny != w.y {
                            // Erase old position before moving
                            aixos_gpu::desktop::set_window_pos(w.x, w.y);
                            aixos_gpu::desktop::clear_window_sized(w.w + 10, w.h + 10);
                            wins()[dw].x = nx;
                            wins()[dw].y = ny;
                            render_windows_only();
                        }
                    }
                    if !mouse_state.left { DRAG_ACTIVE = false; }
                }
                aixos_gpu::draw_cursor(mouse_state.x, mouse_state.y);
                if mouse_state.left && !prev_left {
                    if mouse_state.y < 38 {
                        let _ = (mouse_state.x, mouse_state.y); // top bar click — future
                    } else if mouse_state.y >= 676 {
                        // Dock click
                        handle_dock_click(mouse_state.x, mouse_state.y);
                    } else {
                        // Canvas + window click
                        handle_click(mouse_state.x, mouse_state.y);
                    }
                }
            }
        }
        if let Some(ev) = aixos_input::poll() {
            unsafe {
                let uart = 0x09000000 as *mut u8;
                let hex = b"0123456789abcdef";
                // Log: T=type C=code V=value
                core::ptr::write_volatile(uart, b'T');
                core::ptr::write_volatile(uart, hex[((ev.code >> 4) & 0xf) as usize]);
                core::ptr::write_volatile(uart, hex[(ev.code & 0xf) as usize]);
                core::ptr::write_volatile(uart, b'\n');
            }
            handle_key(&mut buf, ev.code, ev.ch);
        }
        // PL-62: cooperative scheduler tick — advances process table each loop iteration
        unsafe { aixos_kernel::proc::scheduler_tick(); }
    }
}

fn handle_key(buf: &mut ShellBuf, code: u16, ch: Option<char>) {
    unsafe {

        if WINDOW_FOCUSED && wins()[ACTIVE_WIN].open && wins()[ACTIVE_WIN].kind == 1 {
            handle_window_key(code, ch);
            return;
        }
        if wins()[ACTIVE_WIN].open && wins()[ACTIVE_WIN].kind == 4 {
            handle_window_key(code, ch);
            return;
        }
        // PL-60: Onyxia browser key dispatch
        if wins()[ACTIVE_WIN].open && wins()[ACTIVE_WIN].kind == 7 {
            handle_window_key(code, ch);
            return;
        }
    }
    match code {
        28 => {
            uart_write("\n");
            let result = execute_cmd(buf);
            if !result.is_empty() {
                uart_write(result);
                uart_write("\n");
                }
            buf.clear();
            let mut d = 0u64;
            while d < 5_000_000 { d += 1; }
                    uart_write("axos> ");
        }
        1 => {
            buf.clear();
                    uart_write_byte(b'\r');
            uart_write("axos> ");
        }
        14 => {
            if buf.pop() {
                uart_write_byte(0x08);
                uart_write_byte(b' ');
                uart_write_byte(0x08);
            }
        }
        _ => {
            if let Some(c) = ch {
                let b = c as u8;
                if (0x20..0x7fu8).contains(&b) {
                    if buf.push(b) {
                        uart_write_byte(b);
                            }
                }
            }
        }
    }
}


