//! Ferric soft-ref inventory / adapter — reference semantics only.
//!
//! Discovers sibling Ferric / ferrix checkouts when present (`MOL_FERRIC_ROOT`
//! or documented Mac paths). Never path-deps Ferric crates. Never invents
//! `measured_j`. On-device EFA / MuJoCo / robot meters remain OUT OF PROOF SCOPE.
//! `stage_c_measured=false`, `board_synth_claimed=false`.

use mol_core::{MolError, Result, BOARD_SYNTH_CLAIMED};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Env override for Ferric / ferrix workspace root.
pub const ENV_FERRIC_ROOT: &str = "MOL_FERRIC_ROOT";

/// Soft-ref Ferric inventory (crate / docs presence ≠ measured joules).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FerricInventory {
    /// Resolved root if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root: Option<String>,
    /// Relative paths / crate names discovered.
    pub artifacts: Vec<String>,
    /// True when a Ferric-shaped tree was found.
    pub artifacts_present: bool,
    /// Always false — no on-device Ferric meter.
    pub stage_c_measured: bool,
    /// Always false.
    pub board_synth_claimed: bool,
    /// Soft-ref EFA cert available in-tree (StubEfa / hw_gaps ferric_efa_cert).
    pub soft_ref_efa: bool,
    /// Honesty note.
    pub note: String,
}

impl FerricInventory {
    /// Unavailable / missing checkout (still honest).
    pub fn unavailable(reason: impl Into<String>) -> Self {
        Self {
            root: None,
            artifacts: vec![],
            artifacts_present: false,
            stage_c_measured: false,
            board_synth_claimed: BOARD_SYNTH_CLAIMED,
            soft_ref_efa: true,
            note: format!(
                "{}; soft-ref EFA still available in-tree; stage_c_measured=false; board_synth_claimed=false; estimates≠measured_j",
                reason.into()
            ),
        }
    }
}

/// Soft-ref Ferric certify result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FerricCertResult {
    /// allow | refuse | stub
    pub decision: String,
    /// Inventory snapshot.
    pub inventory: FerricInventory,
    /// Always false.
    pub stage_c_measured: bool,
    /// Always false.
    pub board_synth_claimed: bool,
    /// Note.
    pub note: String,
}

fn candidate_roots() -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(p) = std::env::var(ENV_FERRIC_ROOT) {
        let t = p.trim();
        if !t.is_empty() {
            out.push(PathBuf::from(t));
        }
    }
    for p in [
        "/Users/dcharlot/vibe-coding/ferric",
        "/Users/dcharlot/data-share/vibe-coding/ferric",
        "/Users/dcharlot/data-share/vibe-coding/ferrix",
        "/workspace/ferric",
        "/workspace/ferrix",
    ] {
        out.push(PathBuf::from(p));
    }
    out
}

fn looks_like_ferric_tree(root: &Path) -> bool {
    root.join("Cargo.toml").is_file()
        || root.join("crates").is_dir()
        || root.join("ferric-core").is_dir()
        || root.join("README.md").is_file()
}

fn collect_artifacts(root: &Path) -> Vec<String> {
    let mut found = Vec::new();
    let interesting = [
        "ferric-core",
        "ferric-tensor",
        "ferric-onnx",
        "ferric-load",
        "ferric-llama",
        "ferric-web",
        "ferrotherm",
        "ferromotion",
        "Cargo.toml",
        "README.md",
    ];
    // Top-level hits.
    if let Ok(rd) = std::fs::read_dir(root) {
        for ent in rd.flatten() {
            let name = ent.file_name().to_string_lossy().to_string();
            let lower = name.to_ascii_lowercase();
            if interesting.iter().any(|k| lower.contains(&k.to_ascii_lowercase())) {
                found.push(name);
            }
        }
    }
    // crates/* children.
    let crates = root.join("crates");
    if crates.is_dir() {
        if let Ok(rd) = std::fs::read_dir(&crates) {
            for ent in rd.flatten() {
                let name = ent.file_name().to_string_lossy().to_string();
                let lower = name.to_ascii_lowercase();
                if lower.contains("ferric") || lower.contains("ferro") {
                    found.push(format!("crates/{name}"));
                }
            }
        }
    }
    found.sort();
    found.dedup();
    found
}

/// Probe Ferric soft-ref inventory. Never sets measured flags.
pub fn probe_ferric() -> FerricInventory {
    for root in candidate_roots() {
        if !root.is_dir() || !looks_like_ferric_tree(&root) {
            continue;
        }
        let artifacts = collect_artifacts(&root);
        let present = !artifacts.is_empty();
        return FerricInventory {
            root: Some(root.display().to_string()),
            artifacts,
            artifacts_present: present,
            stage_c_measured: false,
            board_synth_claimed: BOARD_SYNTH_CLAIMED,
            soft_ref_efa: true,
            note: if present {
                "Ferric/ferrix tree discovered; soft-ref inventory only — no path-dep; on-device EFA/MuJoCo meters NOT wired; stage_c_measured=false"
                    .into()
            } else {
                "Ferric root present but no named crates matched; soft-ref EFA still in-tree; stage_c_measured=false"
                    .into()
            },
        };
    }
    FerricInventory::unavailable(
        "Ferric/ferrix root not found (set MOL_FERRIC_ROOT); soft-ref EFA remains in-tree",
    )
}

/// Soft-ref Ferric certify: inventories + EFA-shaped allow/refuse; never meters.
pub fn certify_ferric_soft(summary: &str) -> FerricCertResult {
    let inv = probe_ferric();
    let lower = summary.to_ascii_lowercase();
    let decision = if lower.contains("board meter claim")
        || lower.contains("stage_c_measured=true")
        || lower.contains("measured_j")
        || lower.contains("claim silicon")
    {
        "refuse"
    } else if lower.contains("diverge") || lower.contains("spoof") || lower.contains("uncertified")
    {
        "refuse"
    } else if inv.artifacts_present {
        "allow"
    } else {
        "stub"
    };
    FerricCertResult {
        decision: decision.into(),
        note: format!(
            "Ferric soft-ref for '{summary}'; artifacts_present={}; soft_ref_efa=true; stage_c_measured=false; on-device Ferric OUT OF PROOF SCOPE",
            inv.artifacts_present
        ),
        inventory: inv,
        stage_c_measured: false,
        board_synth_claimed: BOARD_SYNTH_CLAIMED,
    }
}

/// Live on-device Ferric / MuJoCo path — reserved AdapterStub.
pub fn live_ferric_meter_stub() -> Result<FerricCertResult> {
    Err(MolError::AdapterStub(
        "on-device Ferric EFA / MuJoCo robot meter not wired; stage_c_measured stays false".into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn probe_never_claims_measured() {
        let inv = probe_ferric();
        assert!(!inv.stage_c_measured);
        assert!(!inv.board_synth_claimed);
        assert!(inv.soft_ref_efa);
    }

    #[test]
    fn certify_refuses_fake_meter() {
        let r = certify_ferric_soft("stage_c_measured=true board meter claim");
        assert_eq!(r.decision, "refuse");
        assert!(!r.stage_c_measured);
    }

    #[test]
    fn live_stub_errors() {
        assert!(live_ferric_meter_stub().is_err());
    }
}
