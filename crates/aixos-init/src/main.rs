// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
#![no_std]
#![no_main]
#![cfg(not(test))]
#![allow(clippy::empty_loop)]
use core::panic::PanicInfo;

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

fn execute_cmd(buf: &ShellBuf) -> &'static str {
    let cmd = buf.as_slice();
    match cmd {
        b"help" => "help clear version db window settings browse close reboot tz name ls cat write awp",
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
struct WinSlot { open: bool, kind: u8, x: i32, y: i32, w: u32, h: u32 }
static mut WINS: [WinSlot; 6] = [
    WinSlot { open: false, kind: 0, x: 60,  y: 80,  w: 580, h: 300 },
    WinSlot { open: false, kind: 0, x: 100, y: 100, w: 580, h: 300 },
    WinSlot { open: false, kind: 0, x: 140, y: 120, w: 580, h: 300 },
    WinSlot { open: false, kind: 0, x: 180, y: 140, w: 580, h: 300 },
    WinSlot { open: false, kind: 0, x: 220, y: 160, w: 580, h: 300 },
    WinSlot { open: false, kind: 0, x: 260, y: 180, w: 580, h: 300 },
];
static mut ACTIVE_WIN: usize = 0;
static mut DRAG_WIN: usize = 0;
static mut WIN_BUF: ShellBuf = ShellBuf::new();
static mut WINDOW_FOCUSED: bool = false;
static mut WIN_OUTPUT: [&str; 8] = [""; 8];
static mut WIN_OUTPUT_LEN: usize = 0;
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
pub extern "C" fn aixos_main() -> ! {
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
    let proof = aixos_init::orchestrate();
    if proof == 0x4153 {
        uart_write("axon_main() -> 0x4153 [SOVEREIGN]\n");
    } else {
        uart_write("axon_main() -> boot incomplete\n");
    }

    let mut delay = 0u64;
    while delay < 10_000_000 { delay += 1; }

    let virtio_ok;
    aixos_edisondb::init();
    // PL-50: AXFS init — seeds readme.txt
    aixos_axfs::init();
    // PL-55: sovereign heap init
    uart_write("heap: init ");
    {
        let free_kb = aixos_kernel::alloc::bytes_free() / 1024;
        // Just log — bump allocator needs no explicit init
        uart_write("sovereign heap ready\n");
        // Make a boot allocation as proof
        let _proof = aixos_kernel::alloc::alloc_val::<u64>(0x4153u64);
    }
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
            aixos_gpu::desktop::render_splash();
            // Volatile read prevents optimizer from eliminating the delay
            let mut splash_delay = 0u64;
            while splash_delay < 1_200_000_000 {
                unsafe { core::ptr::read_volatile(&splash_delay); }
                splash_delay += 1;
            }
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
                    (wins()[0].open, wins()[0].kind),
                    (wins()[1].open, wins()[1].kind),
                    (wins()[2].open, wins()[2].kind),
                    (wins()[3].open, wins()[3].kind),
                    (wins()[4].open, wins()[4].kind),
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
    if !w.open {
        return;
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
        (wins()[0].open, wins()[0].kind),
        (wins()[1].open, wins()[1].kind),
        (wins()[2].open, wins()[2].kind),
        (wins()[3].open, wins()[3].kind),
        (wins()[4].open, wins()[4].kind),
        (wins()[5].open, wins()[5].kind),
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
        (wins()[0].open, wins()[0].kind),
        (wins()[1].open, wins()[1].kind),
        (wins()[2].open, wins()[2].kind),
        (wins()[3].open, wins()[3].kind),
        (wins()[4].open, wins()[4].kind),
        (wins()[5].open, wins()[5].kind),
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
            0 => 0, // Onyxia -> Node window (placeholder)
            1 => 0, // Browser -> Node window (placeholder)
            2 => 1, // Shell
            3 => 6, // Files -> AXFS Files window
            4 => 4, // EDB Browser
            5 => 3, // IAM -> Settings (placeholder)
            6 => 3, // Settings
            _ => return,
        };
        unsafe {
            WINDOW_FOCUSED = false;
            if let Some(i) = find_kind(kind) {
                // Already open — bring to front
                ACTIVE_WIN = i;
            } else {
                // Open in a free slot
                if let Some(slot) = find_free() {
                    wins()[slot].open = true;
                    wins()[slot].kind = kind;
                    ACTIVE_WIN = slot;
                }
                // If no free slot, do nothing (all 5 windows open)
            }
            if kind == 1 || kind == 6 { WINDOW_FOCUSED = true; }
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
        bufs[i][..5].copy_from_slice(b"win> ");
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
            render_all_windows();
        },
        _ => {
            if let Some(c) = ch {
                win_buf().push(c as u8);
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
                if x >= w.x + w.w as i32 - 22 && x < w.x + w.w as i32 - 6 {
                    wins()[i].open = false;
                    WINDOW_FOCUSED = false;
                    aixos_gpu::desktop::set_window_pos(w.x, w.y);
                    aixos_gpu::desktop::clear_window();
                    let mut j = 6;
                    while j > 0 { j -= 1; if wins()[j].open { ACTIVE_WIN = j; break; } }
                    render_all_windows();
                    return;
                }
                DRAG_WIN = i;
                DRAG_ACTIVE = true;
                DRAG_OFF_X = x - w.x;
                DRAG_OFF_Y = y - w.y;
                render_all_windows();
                return;
            }
            if x >= w.x && x < w.x + w.w as i32 && y >= w.y + 24 && y < w.y + w.h as i32 {
                ACTIVE_WIN = i;
                if w.kind == 1 {
                    WINDOW_FOCUSED = true;
                }
                if w.kind == 4 { EDB_FOCUSED = true; }
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
                    0 => 0, // O -> Node
                    1 => 2, // F -> EDB Store
                    2 => 3, // S -> Settings
                    3 => 3, // A -> Settings placeholder
                    4 => 4, // D -> EDB Browser
                    5 => 5, // N -> Network
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
                    const DRAG_MIN_X: i32 = 0; const DRAG_MAX_X: i32 = 700;
                    if !RESIZE_ACTIVE && DRAG_ACTIVE && mouse_state.left {
                        let dw = DRAG_WIN;
                        let w = wins()[dw];
                        let nx = (mouse_state.x - DRAG_OFF_X).clamp(DRAG_MIN_X, DRAG_MAX_X);
                        let ny = (mouse_state.y - DRAG_OFF_Y).clamp(50, 580);
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

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    uart_write("aiXos: panic\n");
    loop {}
}
