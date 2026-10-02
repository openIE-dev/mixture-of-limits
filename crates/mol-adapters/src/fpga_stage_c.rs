//! FPGA Stage C / WCA LUT soft-ref inventory — wire what exists, never invent meters.
//!
//! Discovers sibling `openie-fpga` Stage C artifacts (bitstreams / RTL / `.fpga`
//! packs) when `MOL_OPENIE_FPGA_ROOT` is set or a documented sibling path exists.
//! **Always** stamps `stage_c_measured=false` and `board_synth_claimed=false`.
//! In-proc WCA LUT edge and board package joules remain OUT OF PROOF SCOPE.

use mol_core::{MolError, Result, BOARD_SYNTH_CLAIMED};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Environment override for the openie-fpga workspace root.
pub const ENV_OPENIE_FPGA_ROOT: &str = "MOL_OPENIE_FPGA_ROOT";

/// Soft-ref Stage C inventory (artifact presence ≠ measured joules).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageCInventory {
    /// Resolved openie-fpga root (if any).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root: Option<String>,
    /// Relative artifact paths found under Stage C.
    pub artifacts: Vec<String>,
    /// True when at least one Stage C bitstream / RTL artifact was found.
    pub artifacts_present: bool,
    /// Always false — no board package meter wired.
    pub stage_c_measured: bool,
    /// Always false on soft-ref.
    pub board_synth_claimed: bool,
    /// Honesty note.
    pub note: String,
}

impl StageCInventory {
    /// Empty / unavailable inventory (still honest).
    pub fn unavailable(reason: impl Into<String>) -> Self {
        Self {
            root: None,
            artifacts: vec![],
            artifacts_present: false,
            stage_c_measured: false,
            board_synth_claimed: BOARD_SYNTH_CLAIMED,
            note: format!(
                "{}; stage_c_measured=false; board_synth_claimed=false; estimates≠measured_j",
                reason.into()
            ),
        }
    }
}

/// Stage C certify probe — inventories artifacts; never claims silicon meters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageCCertResult {
    /// allow | refuse | stub
    pub decision: String,
    /// Inventory snapshot.
    pub inventory: StageCInventory,
    /// Always false.
    pub stage_c_measured: bool,
    /// Always false.
    pub board_synth_claimed: bool,
    /// Note.
    pub note: String,
}

fn candidate_roots() -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(p) = std::env::var(ENV_OPENIE_FPGA_ROOT) {
        let t = p.trim();
        if !t.is_empty() {
            out.push(PathBuf::from(t));
        }
    }
    // Documented sibling paths (Mac / data-share) — optional presence only.
    for p in [
        "/Users/dcharlot/data-share/vibe-coding/openie-fpga",
        "/Users/dcharlot/vibe-coding/openie-fpga",
        "/workspace/openie-fpga",
    ] {
        out.push(PathBuf::from(p));
    }
    out
}

fn collect_artifacts(root: &Path) -> Vec<String> {
    let mut found = Vec::new();
    let stage_c = root.join("artifacts/stage-c");
    let search_dirs = [stage_c.as_path(), root];
    let interesting = [
        "wca_commit_gate",
        "wca_seed1",
        "wca_allow_lut",
        "wca_ph_energy",
        "wca_cbf_energy",
        "wca_tlmm_synth",
    ];
    for dir in search_dirs {
        if !dir.is_dir() {
            continue;
        }
        let Ok(rd) = std::fs::read_dir(dir) else {
            continue;
        };
        for ent in rd.flatten() {
            let name = ent.file_name().to_string_lossy().to_string();
            let lower = name.to_ascii_lowercase();
            let hit = interesting.iter().any(|k| lower.contains(k))
                && (lower.ends_with(".bit")
                    || lower.ends_with(".v")
                    || lower.ends_with(".fpga")
                    || lower.ends_with(".xdc")
                    || lower.ends_with(".json")
                    || !lower.contains('.'));
            if hit {
                let rel = ent
                    .path()
                    .strip_prefix(root)
                    .map(|p| p.display().to_string())
                    .unwrap_or(name);
                found.push(rel);
            }
        }
    }
    // Top-level .fpga packs at repo root.
    if let Ok(rd) = std::fs::read_dir(root) {
        for ent in rd.flatten() {
            let name = ent.file_name().to_string_lossy().to_string();
            let lower = name.to_ascii_lowercase();
            if lower.contains("wca") && lower.ends_with(".fpga") {
                found.push(name);
            }
        }
    }
    found.sort();
    found.dedup();
    found
}

/// Probe Stage C artifact inventory (soft-ref). Never sets measured flags.
pub fn probe_stage_c() -> StageCInventory {
    for root in candidate_roots() {
        if !root.is_dir() {
            continue;
        }
        let artifacts = collect_artifacts(&root);
        let present = !artifacts.is_empty();
        return StageCInventory {
            root: Some(root.display().to_string()),
            artifacts,
            artifacts_present: present,
            stage_c_measured: false,
            board_synth_claimed: BOARD_SYNTH_CLAIMED,
            note: if present {
                "openie-fpga Stage C artifacts discovered; in-proc WCA LUT / board meter NOT wired; stage_c_measured=false; board_synth_claimed=false"
                    .into()
            } else {
                "openie-fpga root present but no Stage C WCA artifacts matched; stage_c_measured=false"
                    .into()
            },
        };
    }
    StageCInventory::unavailable(
        "openie-fpga root not found (set MOL_OPENIE_FPGA_ROOT); Stage C stays stub",
    )
}

/// Soft-ref Stage C certify: inventories only; refuses to claim board meters.
pub fn certify_stage_c_soft(summary: &str) -> StageCCertResult {
    let inv = probe_stage_c();
    let decision = if summary.to_ascii_lowercase().contains("board meter claim")
        || summary.to_ascii_lowercase().contains("stage_c_measured=true")
    {
        "refuse"
    } else {
        "stub"
    };
    StageCCertResult {
        decision: decision.into(),
        note: format!(
            "Stage C soft-ref for '{summary}'; artifacts_present={}; stage_c_measured=false; Ferric/FPGA package meters OUT OF PROOF SCOPE",
            inv.artifacts_present
        ),
        inventory: inv,
        stage_c_measured: false,
        board_synth_claimed: BOARD_SYNTH_CLAIMED,
    }
}

/// Live in-proc / UART board path — reserved AdapterStub (honesty preserved).
pub fn live_stage_c_meter_stub() -> Result<StageCCertResult> {
    Err(MolError::AdapterStub(
        "FPGA Stage C board meter / wca-lut-edge in-proc not wired; stage_c_measured stays false"
            .into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn probe_never_claims_measured() {
        let inv = probe_stage_c();
        assert!(!inv.stage_c_measured);
        assert!(!inv.board_synth_claimed);
    }

    #[test]
    fn certify_refuses_fake_meter_claim() {
        let r = certify_stage_c_soft("stage_c_measured=true board meter claim");
        assert_eq!(r.decision, "refuse");
        assert!(!r.stage_c_measured);
        assert!(!r.board_synth_claimed);
    }

    #[test]
    fn live_stub_errors_honestly() {
        assert!(live_stage_c_meter_stub().is_err());
    }
}
