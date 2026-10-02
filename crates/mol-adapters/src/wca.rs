//! Adapter surface for `wca-commit` propose/certify/commit|refuse.
//!
//! Software-reference stub certifies by default; refuses destructive /
//! uncertified patterns. Live MCP / in-proc path stays AdapterStub.

use mol_core::{Floor, FloorKind, MolError, Result, BOARD_SYNTH_CLAIMED};
use serde::{Deserialize, Serialize};

/// Port toward wca-lut-edge commit gate.
pub trait WcaCommitPort: Send + Sync {
    /// Certify a proposal (software-reference by default).
    fn certify(&self, proposal_summary: &str, estimated_j: f64) -> Result<WcaCertResult>;
}

/// Stub certificate result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WcaCertResult {
    /// allow | refuse
    pub decision: String,
    /// Reasons.
    pub reasons: Vec<String>,
    /// Surrogate joules.
    pub estimated_j: f64,
    /// Always false.
    pub board_synth_claimed: bool,
    /// Optional MoL floor when refused.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub floor: Option<Floor>,
    /// Note.
    pub note: String,
}

/// Default software-reference WCA commit port.
#[derive(Debug, Default, Clone)]
pub struct StubWcaCommit;

impl WcaCommitPort for StubWcaCommit {
    fn certify(&self, proposal_summary: &str, estimated_j: f64) -> Result<WcaCertResult> {
        Ok(software_reference_cert(
            proposal_summary,
            estimated_j,
            !looks_unsafe(proposal_summary),
        ))
    }
}

fn looks_unsafe(summary: &str) -> bool {
    let s = summary.to_ascii_lowercase();
    s.contains("rm -rf")
        || s.contains("drop table")
        || s.contains("format disk")
        || s.contains("wca refuse")
        || s.contains("uncertified")
}

/// Document + demo helper showing the expected honesty fields.
pub fn software_reference_cert(summary: &str, estimated_j: f64, allow: bool) -> WcaCertResult {
    if allow {
        WcaCertResult {
            decision: "allow".into(),
            reasons: vec![format!("software-reference WCA allow for: {summary}")],
            estimated_j,
            board_synth_claimed: BOARD_SYNTH_CLAIMED,
            floor: None,
            note: "catalog_surrogate joules; not RAPL/NVML".into(),
        }
    } else {
        let reason = format!("WCA certify refuse for: {summary}");
        WcaCertResult {
            decision: "refuse".into(),
            reasons: vec![reason.clone()],
            estimated_j,
            board_synth_claimed: BOARD_SYNTH_CLAIMED,
            floor: Some(Floor::new("wca_refuse", FloorKind::WcaRefuse, reason)),
            note: "catalog_surrogate joules; not RAPL/NVML; refuse over escalate".into(),
        }
    }
}

/// Explicit live-path stub.
pub fn live_wca_stub(_summary: &str, _estimated_j: f64) -> Result<WcaCertResult> {
    Err(MolError::AdapterStub(
        "wca-commit live path not wired in v0.1 (enable future wca-path; see BLUEPRINT.md)".into(),
    ))
}
