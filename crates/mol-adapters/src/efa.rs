//! EFA-style energy-as-certificate port (Physical AI BMI propose/certify/refuse).
//!
//! Software-reference stub: one scalar energy is both policy and a pre-commit
//! Lyapunov-style certificate. Refuse over commit when the loop would diverge.
//! Does **not** claim Ferric / MuJoCo / robot hardware. `board_synth_claimed=false`.
//! Estimated joules are catalog surrogates — never RAPL/NVML.

use mol_core::{Floor, FloorKind, MolError, Result, BOARD_SYNTH_CLAIMED};
use serde::{Deserialize, Serialize};

/// Port toward EFA certificate (BMI Energy First Architecture spirit).
pub trait EfaCertificatePort: Send + Sync {
    /// Certify a proposed act before commit.
    ///
    /// Returns allow or refuse with an optional MoL floor binding.
    fn certify(&self, proposal: &EfaProposal) -> Result<EfaCertResult>;
}

/// Proposal handed to the certificate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EfaProposal {
    /// Human summary of the act.
    pub summary: String,
    /// Estimated path joules (surrogate).
    pub estimated_j: f64,
    /// Optional energy residual / Lyapunov candidate (software reference).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub energy_residual: Option<f64>,
    /// Optional tag: "safe" | "diverge" | "spoof" | …
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
}

/// Certificate decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EfaDecision {
    /// Certificate holds — commit may proceed.
    Allow,
    /// Certificate refuses — no commit.
    Refuse,
}

/// Stub certificate result matching BMI propose/certify/refuse shape.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EfaCertResult {
    /// allow | refuse
    pub decision: EfaDecision,
    /// Reasons (machine + human).
    pub reasons: Vec<String>,
    /// Surrogate joules for the certify step itself (not RAPL).
    pub estimated_j: f64,
    /// Always false.
    pub board_synth_claimed: bool,
    /// Optional MoL floor when refused.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub floor: Option<Floor>,
    /// Note.
    pub note: String,
}

/// Default software-reference EFA certificate.
///
/// Heuristic (demo, not hardware):
/// - tag/summary containing `diverge`, `spoof`, `uncertified`, `false-safe` → refuse
/// - `energy_residual > 0` (ascent) → refuse
/// - otherwise allow with structural-identity note
#[derive(Debug, Default, Clone)]
pub struct StubEfaCertificate;

impl EfaCertificatePort for StubEfaCertificate {
    fn certify(&self, proposal: &EfaProposal) -> Result<EfaCertResult> {
        let s = proposal.summary.to_ascii_lowercase();
        let tag = proposal
            .tag
            .as_deref()
            .unwrap_or("")
            .to_ascii_lowercase();
        let blob = format!("{s} {tag}");

        let refuse_token = [
            "diverge",
            "spoof",
            "uncertified",
            "false-safe",
            "false_safe",
            "refuse certificate",
            "efa refuse",
        ]
        .iter()
        .any(|t| blob.contains(t));

        let ascent = proposal
            .energy_residual
            .map(|e| e > 1e-12)
            .unwrap_or(false);

        if refuse_token || ascent {
            let reason = if ascent {
                format!(
                    "EFA certificate refuse: energy residual {} > 0 (Lyapunov ascent; refuse over commit)",
                    proposal.energy_residual.unwrap_or(0.0)
                )
            } else {
                format!(
                    "EFA certificate refuse: proposal tagged unsafe / diverge ({})",
                    proposal.summary
                )
            };
            let floor = Floor::new("efa_certificate", FloorKind::EfaCertificate, &reason);
            return Ok(EfaCertResult {
                decision: EfaDecision::Refuse,
                reasons: vec![reason],
                estimated_j: proposal.estimated_j.min(1e-9),
                board_synth_claimed: BOARD_SYNTH_CLAIMED,
                floor: Some(floor),
                note: "software-reference EFA stub; not Ferric/MuJoCo; estimated ≠ measured"
                    .into(),
            });
        }

        Ok(EfaCertResult {
            decision: EfaDecision::Allow,
            reasons: vec![format!(
                "EFA certificate allow: structural identity holds for '{}'",
                proposal.summary
            )],
            estimated_j: proposal.estimated_j.min(1e-9),
            board_synth_claimed: BOARD_SYNTH_CLAIMED,
            floor: None,
            note: "software-reference EFA stub; Lyapunov spirit only; no robot claim".into(),
        })
    }
}

/// Explicit live-path stub (path-dep not wired).
pub fn live_efa_stub(_proposal: &EfaProposal) -> Result<EfaCertResult> {
    Err(MolError::AdapterStub(
        "efa live path not wired (Ferric/BMI runtime); use StubEfaCertificate software-reference"
            .into(),
    ))
}
