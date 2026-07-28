// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// ════════════════════════════════════════════════════════════════════════════
// crates/aixos-asl/src/lib.rs
// PL-76: ASL-seL4 Integration Bridge (bare-metal subset)
// ════════════════════════════════════════════════════════════════════════════
//
// Wires the no_std/no_alloc ASL PD contracts into the aiXos Phoenix boot path.
// asl-haniel-canvas-pd and asl-sovereign-boot-proof are excluded here because
// asl-haniel uses Vec (requires global allocator not available on bare-metal).
// Those PDs are proven in the ASL repo (v2.0.0-asl) and integrated via the
// full seL4 build in the next integration sprint.
//
// What is wired here (6 PDs, all no_std/no_alloc):
//   Stage 1: GPU-Cap PD — maps ramfb, FramebufferWrite capability
//   Stage 2: EdisonDB-PD — sovereign store, ARPi auth
//   Stage 3: Shell-PD — axc> shell IPC routing
//   Stage 4: AXON-Exec-PD — scripts in isolated PD
//   Stage 5: Onyxia-PD — browser, http:// blocked
//   Stage 6: Phoenix-Desktop-PD — render loop proof
//
// UART evidence visible in boot log:
//   [ASL] Stage N: <PD> — <proof>
//   [ASL] Stage 6: SOVEREIGN DESKTOP LIVE — proof=0x4153
//
// Post Doctrine: P1 ✓ P2 ✓ P3 ✓ P4 ✓ P5 ✓

#![no_std]
#![forbid(unsafe_code)]

#[cfg(kani)]
extern crate kani;

use asl_arpi_ipc::AXON_PROOF;
use asl_phoenix_desktop::{PhoenixDesktopPd, GpuCapPd, FramebufDesc};
use asl_shell_pd::ShellPd;
use asl_edisondb_pd::EdisonDbPd;
use asl_onyxia_pd::OnyxiaPd;
use asl_axon_exec_pd::AxonExecPd;

// ── Constants ─────────────────────────────────────────────────────────────────

pub const SOVEREIGN_PROOF: u64 = AXON_PROOF;

// ── Integration result ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AslStageResult {
    Proven { stage: u8, proof: u64 },
    Failed,
}

impl AslStageResult {
    pub fn is_sovereign(&self) -> bool {
        matches!(self, AslStageResult::Proven { proof, .. } if *proof == SOVEREIGN_PROOF)
    }
}

// ── ASL boot integrator ───────────────────────────────────────────────────────

/// Runs sovereign PD contracts alongside each aiXos splash stage.
pub struct AslBootIntegrator {
    pub gpu_cap:   GpuCapPd,
    pub desktop:   PhoenixDesktopPd,
    pub shell:     ShellPd,
    pub edb:       EdisonDbPd,
    pub onyxia:    OnyxiaPd,
    pub axon_exec: AxonExecPd,
    pub stages_proven: u8,
    pub boot_complete: bool,
}

impl AslBootIntegrator {
    pub fn new() -> Self {
        AslBootIntegrator {
            gpu_cap:   GpuCapPd::new(),
            desktop:   PhoenixDesktopPd::new(),
            shell:     ShellPd::new(),
            edb:       EdisonDbPd::new(),
            onyxia:    OnyxiaPd::new(),
            axon_exec: AxonExecPd::new(),
            stages_proven: 0,
            boot_complete: false,
        }
    }

    /// Stage 1 — Hardware probe: GPU-Cap PD maps ramfb
    pub fn stage1_hw_probe(&mut self) -> AslStageResult {
        match self.gpu_cap.map_ramfb(0x44000000) {
            Ok(_) => {
                self.stages_proven += 1;
                AslStageResult::Proven { stage: 1, proof: SOVEREIGN_PROOF }
            }
            Err(_) => AslStageResult::Failed,
        }
    }

    /// Stage 2 — EdisonDB-PD boots with ARPi auth
    pub fn stage2_edisondb(&mut self) -> AslStageResult {
        match self.edb.on_boot_signal() {
            Ok(_) => {
                self.stages_proven += 1;
                AslStageResult::Proven { stage: 2, proof: SOVEREIGN_PROOF }
            }
            Err(_) => AslStageResult::Failed,
        }
    }

    /// Stage 3 — Shell-PD boots, AXFS IPC route proven
    pub fn stage3_axfs(&mut self) -> AslStageResult {
        match self.shell.on_boot_signal() {
            Ok(_) => {
                let route = asl_shell_pd::classify_cmd(b"ls");
                if route == asl_shell_pd::CmdRoute::Axfs {
                    self.stages_proven += 1;
                    AslStageResult::Proven { stage: 3, proof: SOVEREIGN_PROOF }
                } else {
                    AslStageResult::Failed
                }
            }
            Err(_) => AslStageResult::Failed,
        }
    }

    /// Stage 4 — AXON-Exec-PD boots, scripts isolated
    pub fn stage4_heap(&mut self) -> AslStageResult {
        match self.axon_exec.on_boot_signal() {
            Ok(_) => {
                self.stages_proven += 1;
                AslStageResult::Proven { stage: 4, proof: SOVEREIGN_PROOF }
            }
            Err(_) => AslStageResult::Failed,
        }
    }

    /// Stage 5 — Onyxia-PD boots, http:// blocked
    pub fn stage5_proctable(&mut self) -> AslStageResult {
        match self.onyxia.on_boot_signal() {
            Ok(_) => {
                // Verify http:// is blocked even under seL4
                let blocked = self.onyxia.navigate(b"http://evil.com").is_err();
                if blocked {
                    self.stages_proven += 1;
                    AslStageResult::Proven { stage: 5, proof: SOVEREIGN_PROOF }
                } else {
                    AslStageResult::Failed
                }
            }
            Err(_) => AslStageResult::Failed,
        }
    }

    /// Stage 6 — Phoenix-Desktop-PD receives GPU cap, render loop proven
    pub fn stage6_desktop(&mut self) -> AslStageResult {
        let fb_desc = FramebufDesc {
            vaddr:  0x44000000,
            width:  1280,
            height: 720,
            bpp:    4,
            stride: 1280 * 4,
        };
        let ok1 = self.desktop.on_boot_signal().is_ok();
        let ok2 = self.desktop.on_fb_cap_grant(fb_desc).is_ok();
        let proof_ok =
            self.edb.proof      == SOVEREIGN_PROOF &&
            self.onyxia.proof   == SOVEREIGN_PROOF &&
            self.axon_exec.proof == SOVEREIGN_PROOF;

        if ok1 && ok2 && proof_ok && self.stages_proven >= 5 {
            self.boot_complete = true;
            self.stages_proven += 1;
            AslStageResult::Proven { stage: 6, proof: SOVEREIGN_PROOF }
        } else {
            AslStageResult::Failed
        }
    }

    /// Run all 6 stages — returns true if fully sovereign
    pub fn run_all_stages(&mut self) -> bool {
        let s1 = self.stage1_hw_probe();
        let s2 = self.stage2_edisondb();
        let s3 = self.stage3_axfs();
        let s4 = self.stage4_heap();
        let s5 = self.stage5_proctable();
        let s6 = self.stage6_desktop();
        s1.is_sovereign() && s2.is_sovereign() && s3.is_sovereign() &&
        s4.is_sovereign() && s5.is_sovereign() && s6.is_sovereign() &&
        self.boot_complete
    }

    /// UART log message for each stage
    pub fn stage_log(stage: u8) -> &'static str {
        match stage {
            1 => "[ASL] Stage 1: GPU-Cap PD — ramfb mapped\n",
            2 => "[ASL] Stage 2: EdisonDB-PD — ARPi auth active\n",
            3 => "[ASL] Stage 3: Shell-PD — AXFS route proven\n",
            4 => "[ASL] Stage 4: AXON-Exec-PD — scripts isolated\n",
            5 => "[ASL] Stage 5: Onyxia-PD — http:// blocked\n",
            6 => "[ASL] Stage 6: SOVEREIGN DESKTOP LIVE — proof=0x4153\n",
            _ => "[ASL] Unknown stage\n",
        }
    }
}

impl Default for AslBootIntegrator { fn default() -> Self { Self::new() } }

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_stages_sovereign() {
        let mut i = AslBootIntegrator::new();
        assert!(i.run_all_stages());
        assert_eq!(i.stages_proven, 6);
        assert!(i.boot_complete);
    }

    #[test]
    fn test_stage1_hw_probe() {
        let mut i = AslBootIntegrator::new();
        assert!(i.stage1_hw_probe().is_sovereign());
    }

    #[test]
    fn test_stage2_edisondb() {
        let mut i = AslBootIntegrator::new();
        i.stage1_hw_probe();
        assert!(i.stage2_edisondb().is_sovereign());
    }

    #[test]
    fn test_stage3_shell_axfs() {
        let mut i = AslBootIntegrator::new();
        i.stage1_hw_probe();
        i.stage2_edisondb();
        assert!(i.stage3_axfs().is_sovereign());
    }

    #[test]
    fn test_stage4_axon_exec() {
        let mut i = AslBootIntegrator::new();
        i.stage1_hw_probe();
        i.stage2_edisondb();
        i.stage3_axfs();
        assert!(i.stage4_heap().is_sovereign());
    }

    #[test]
    fn test_stage5_onyxia_http_blocked() {
        let mut i = AslBootIntegrator::new();
        i.stage1_hw_probe();
        i.stage2_edisondb();
        i.stage3_axfs();
        i.stage4_heap();
        assert!(i.stage5_proctable().is_sovereign());
    }

    #[test]
    fn test_stage6_desktop_sovereign() {
        let mut i = AslBootIntegrator::new();
        i.run_all_stages();
        assert!(i.boot_complete);
    }

    #[test]
    fn test_stage_log_messages() {
        assert!(AslBootIntegrator::stage_log(1).contains("GPU-Cap"));
        assert!(AslBootIntegrator::stage_log(6).contains("SOVEREIGN"));
    }

    #[test]
    fn test_sovereign_proof_constant() {
        assert_eq!(SOVEREIGN_PROOF, 0x4153);
    }
}
