// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// PL-62: Sovereign Cooperative Process Primitives
//
// Design: single-core cooperative scheduler, no real context switching.
// Processes are cooperative — they yield voluntarily.
// The table holds up to MAX_PROCS slots; the scheduler advances a tick
// counter each loop iteration and selects the next Ready process.
//
// PID 0 = [kernel]    — always Running, never killed
// PID 1 = [shell]     — registered at boot
// PID 2 = [desktop]   — registered at boot

#![allow(dead_code)]

pub const MAX_PROCS: usize = 8;

/// Cooperative process state.
#[derive(Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum ProcState {
    Empty   = 0, // slot unused
    Ready   = 1, // waiting to run
    Running = 2, // currently executing
    Blocked = 3, // waiting on I/O or event
    Dead    = 4, // exited / killed
}

/// Sovereign process descriptor — no heap, fixed-size.
#[derive(Copy, Clone)]
pub struct ProcDesc {
    pub pid:       u8,
    pub state:     ProcState,
    pub priority:  u8,       // 0 = highest, 255 = lowest
    pub ticks:     u64,      // scheduler ticks consumed
    pub yields:    u32,      // voluntary yield count
    pub name:      [u8; 16], // process name (ASCII, null-padded)
    pub name_len:  usize,
}

impl ProcDesc {
    pub const fn empty() -> Self {
        ProcDesc {
            pid: 0, state: ProcState::Empty, priority: 128,
            ticks: 0, yields: 0,
            name: [0u8; 16], name_len: 0,
        }
    }
    pub fn name_str(&self) -> &str {
        core::str::from_utf8(&self.name[..self.name_len]).unwrap_or("?")
    }
}

/// The sovereign process table — static, no alloc.
pub struct ProcessTable {
    procs:      [ProcDesc; MAX_PROCS],
    count:      usize,   // number of non-Empty slots
    current:    usize,   // index of currently Running slot
    tick_total: u64,     // global scheduler tick counter
    next_pid:   u8,      // monotonic PID allocator
}

impl ProcessTable {
    pub const fn new() -> Self {
        ProcessTable {
            procs: [ProcDesc::empty(); MAX_PROCS],
            count: 0,
            current: 0,
            tick_total: 0,
            next_pid: 0,
        }
    }
}

static mut PTABLE: ProcessTable = ProcessTable::new();

/// Register a new process. Returns its PID, or None if table is full.
/// name: ASCII bytes. state: initial state (usually Ready).
pub fn spawn(name: &[u8], state: ProcState, priority: u8) -> Option<u8> {
    unsafe {
        // Find empty slot
        let mut slot = MAX_PROCS;
        let mut i = 0;
        while i < MAX_PROCS {
            if PTABLE.procs[i].state == ProcState::Empty {
                slot = i;
                break;
            }
            i += 1;
        }
        if slot == MAX_PROCS { return None; } // full

        let pid = PTABLE.next_pid;
        PTABLE.next_pid = PTABLE.next_pid.wrapping_add(1);

        let mut desc = ProcDesc::empty();
        desc.pid      = pid;
        desc.state    = state;
        desc.priority = priority;
        let nlen = name.len().min(15);
        desc.name[..nlen].copy_from_slice(&name[..nlen]);
        desc.name_len = nlen;

        PTABLE.procs[slot] = desc;
        PTABLE.count += 1;
        Some(pid)
    }
}

/// Kill a process by PID. PID 0 ([kernel]) cannot be killed.
pub fn kill(pid: u8) -> bool {
    if pid == 0 { return false; }
    unsafe {
        let mut i = 0;
        while i < MAX_PROCS {
            if PTABLE.procs[i].pid == pid
                && PTABLE.procs[i].state != ProcState::Empty {
                PTABLE.procs[i].state = ProcState::Dead;
                return true;
            }
            i += 1;
        }
    }
    false
}

/// Cooperative yield: mark current as Ready, advance to next Ready process.
/// Returns the PID of the newly Running process.
pub fn scheduler_tick() -> u8 {
    unsafe {
        PTABLE.tick_total += 1;

        // Increment tick count for current running process
        let cur = PTABLE.current;
        if PTABLE.procs[cur].state == ProcState::Running {
            PTABLE.procs[cur].ticks += 1;
        }

        // Find next Ready slot (round-robin after current)
        let mut next = cur;
        let mut tried = 0;
        loop {
            next = (next + 1) % MAX_PROCS;
            tried += 1;
            if tried > MAX_PROCS { break; } // no other Ready — stay on current
            let s = PTABLE.procs[next].state;
            if s == ProcState::Ready || s == ProcState::Running { break; }
        }

        if next != cur {
            // Yield current (unless kernel — always Running)
            if PTABLE.procs[cur].pid != 0
                && PTABLE.procs[cur].state == ProcState::Running {
                PTABLE.procs[cur].state = ProcState::Ready;
                PTABLE.procs[cur].yields += 1;
            }
            PTABLE.procs[next].state = ProcState::Running;
            PTABLE.current = next;
        }

        PTABLE.procs[PTABLE.current].pid
    }
}

/// Read-only snapshot of a slot (for GUI rendering).
pub fn proc_at(idx: usize) -> Option<ProcDesc> {
    unsafe {
        if idx >= MAX_PROCS { return None; }
        let p = PTABLE.procs[idx];
        if p.state == ProcState::Empty { None } else { Some(p) }
    }
}

/// Total number of non-empty slots.
pub fn proc_count() -> usize {
    unsafe { PTABLE.count }
}

/// Global tick counter.
pub fn tick_total() -> u64 {
    unsafe { PTABLE.tick_total }
}

/// Current running PID.
pub fn current_pid() -> u8 {
    unsafe { PTABLE.procs[PTABLE.current].pid }
}

/// Boot: register the three sovereign system processes.
/// Must be called once after PTABLE is initialised.
pub fn boot_register() {
    // PID 0 — [kernel] — always Running
    let _ = spawn(b"[kernel]", ProcState::Running, 0);
    // PID 1 — [shell]
    let _ = spawn(b"[shell]", ProcState::Ready, 64);
    // PID 2 — [desktop]
    let _ = spawn(b"[desktop]", ProcState::Ready, 64);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_and_count() {
        // Reset table for test
        unsafe { PTABLE = ProcessTable::new(); }
        boot_register();
        assert_eq!(proc_count(), 3);
    }

    #[test]
    fn test_scheduler_tick_advances() {
        unsafe { PTABLE = ProcessTable::new(); }
        boot_register();
        let t0 = tick_total();
        scheduler_tick();
        assert!(tick_total() > t0);
    }

    #[test]
    fn test_kill_non_kernel() {
        unsafe { PTABLE = ProcessTable::new(); }
        boot_register();
        // PID 1 = [shell] — can be killed
        assert!(kill(1));
        // PID 0 = [kernel] — cannot be killed
        assert!(!kill(0));
    }

    #[test]
    fn test_spawn_user_proc() {
        unsafe { PTABLE = ProcessTable::new(); }
        boot_register();
        let pid = spawn(b"myproc", ProcState::Ready, 128);
        assert!(pid.is_some());
        assert_eq!(proc_count(), 4);
    }

    #[test]
    fn test_proc_at_snapshot() {
        unsafe { PTABLE = ProcessTable::new(); }
        boot_register();
        let p = proc_at(0).unwrap();
        assert_eq!(p.name_str(), "[kernel]");
        assert_eq!(p.state, ProcState::Running);
        assert_eq!(p.priority, 0);
    }
}
