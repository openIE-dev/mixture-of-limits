//! In-crate live NI / WCA / EFA certify path (default / fallback).
//!
//! Issues real certificate ids and commit|refuse receipts. Not soft-ref-only
//! heuristics without provenance: every allow stamps `certificate_id`s that
//! bind the irreversible act. Optional live HTTP/MCP is in [`crate::ni_http`];
//! close prefers env when set and falls back here. FPGA Stage C / Ferric
//! hardware meters stay `measured_j=None`, `board_synth_claimed=false`,
//! `stage_c_measured=false`.

use mol_core::{Floor, FloorKind, Result, BOARD_SYNTH_CLAIMED};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::efa::{EfaCertResult, EfaCertificatePort, EfaDecision, EfaProposal, StubEfaCertificate};
use crate::wca::{StubWcaCommit, WcaCertResult, WcaCommitPort};

/// Where the NI certificate was obtained.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CertifySource {
    /// Default in-crate EFA+WCA software-reference path.
    #[default]
    InCrate,
    /// Live HTTP POST `/v1/certify`.
    Http,
    /// Live MCP-shaped JSON-RPC `tools/call`.
    Mcp,
    /// Live endpoint configured but failed; fell back to in-crate.
    InCrateFallback,
}

impl std::fmt::Display for CertifySource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InCrate => write!(f, "in_crate"),
            Self::Http => write!(f, "http"),
            Self::Mcp => write!(f, "mcp"),
            Self::InCrateFallback => write!(f, "in_crate_fallback"),
        }
    }
}


/// One NI certificate with durable id (commit gate provenance).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NiCertificate {
    /// Stable certificate id (`ni:<uuid>`).
    pub certificate_id: String,
    /// EFA sub-certificate id.
    pub efa_id: String,
    /// WCA sub-certificate id.
    pub wca_id: String,
    /// `commit` | `refuse`.
    pub decision: String,
    /// Reasons.
    pub reasons: Vec<String>,
    /// Always false on soft-ref / in-crate path (FPGA Stage C optional unmetered).
    pub board_synth_claimed: bool,
    /// FPGA / Stage C measured claim — always false here.
    pub stage_c_measured: bool,
    /// Estimated certify joules (catalog surrogate).
    pub estimated_j: f64,
    /// Provenance of this certificate (in-crate / http / mcp / fallback).
    #[serde(default)]
    pub source: CertifySource,
}

impl NiCertificate {
    /// True when commit is allowed.
    pub fn allows_commit(&self) -> bool {
        self.decision == "commit"
    }
}

/// Combined live certify outcome for close.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveCertOutcome {
    /// NI certificate (always present after certify attempt).
    pub ni: NiCertificate,
    /// EFA raw result.
    pub efa: EfaCertResult,
    /// WCA raw result.
    pub wca: WcaCertResult,
    /// Optional floor when refused.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub floor: Option<Floor>,
}

/// In-crate NI certify path — real commit/refuse/receipt with certificate ids.
///
/// Uses software-reference EFA+WCA logic but **mints durable certificate ids**
/// and a unified NI receipt. Live HTTP/MCP is optional via env
/// (`MOL_NI_CERTIFY_URL` / `MOL_WCA_CERTIFY_URL`); this path is the default
/// and the fallback when live is unset or fails. Ferric / FPGA Stage C meters
/// remain stubs (`stage_c_measured=false`).
#[derive(Debug, Default, Clone)]
pub struct InCrateNiCertify {
    efa: StubEfaCertificate,
    wca: StubWcaCommit,
}

impl InCrateNiCertify {
    /// Construct.
    pub fn new() -> Self {
        Self::default()
    }

    /// Certify a proposal end-to-end: EFA → WCA → NI receipt with ids.
    pub fn certify_live(
        &self,
        summary: &str,
        estimated_j: f64,
        efa_tag: Option<String>,
        energy_residual: Option<f64>,
    ) -> Result<LiveCertOutcome> {
        let efa_res = self.efa.certify(&EfaProposal {
            summary: summary.into(),
            estimated_j,
            energy_residual,
            tag: efa_tag,
        })?;

        let efa_id = format!("efa:{}", short_uuid());
        let wca_id = format!("wca:{}", short_uuid());
        let ni_id = format!("ni:{}", short_uuid());

        if efa_res.decision == EfaDecision::Refuse {
            let floor = efa_res.floor.clone().unwrap_or_else(|| {
                Floor::new(
                    "efa_certificate",
                    FloorKind::EfaCertificate,
                    "EFA certificate refuse",
                )
            });
            let ni = NiCertificate {
                certificate_id: ni_id,
                efa_id,
                wca_id,
                decision: "refuse".into(),
                reasons: efa_res.reasons.clone(),
                board_synth_claimed: BOARD_SYNTH_CLAIMED,
                stage_c_measured: false,
                estimated_j: efa_res.estimated_j,
                source: CertifySource::InCrate,
            };
            // Still run WCA for receipt completeness (decision already refuse).
            let wca_res = self.wca.certify(summary, estimated_j)?;
            return Ok(LiveCertOutcome {
                ni,
                efa: efa_res,
                wca: wca_res,
                floor: Some(floor),
            });
        }

        let wca_res = self.wca.certify(summary, estimated_j)?;
        if wca_res.decision != "allow" {
            let floor = wca_res.floor.clone().unwrap_or_else(|| {
                Floor::new("wca_refuse", FloorKind::WcaRefuse, "WCA certify refuse")
            });
            let ni = NiCertificate {
                certificate_id: ni_id,
                efa_id,
                wca_id,
                decision: "refuse".into(),
                reasons: wca_res.reasons.clone(),
                board_synth_claimed: BOARD_SYNTH_CLAIMED,
                stage_c_measured: false,
                estimated_j,
                source: CertifySource::InCrate,
            };
            return Ok(LiveCertOutcome {
                ni,
                efa: efa_res,
                wca: wca_res,
                floor: Some(floor),
            });
        }

        let ni = NiCertificate {
            certificate_id: ni_id.clone(),
            efa_id: efa_id.clone(),
            wca_id: wca_id.clone(),
            decision: "commit".into(),
            reasons: vec![
                format!("NI commit allow; efa={efa_id}; wca={wca_id}"),
                format!("summary_ok: {summary}"),
            ],
            board_synth_claimed: BOARD_SYNTH_CLAIMED,
            stage_c_measured: false,
            estimated_j,
            source: CertifySource::InCrate,
        };
        Ok(LiveCertOutcome {
            ni,
            efa: efa_res,
            wca: wca_res,
            floor: None,
        })
    }
}

impl EfaCertificatePort for InCrateNiCertify {
    fn certify(&self, proposal: &EfaProposal) -> Result<EfaCertResult> {
        // Delegate; live path prefers certify_live for full NI receipt.
        self.efa.certify(proposal)
    }
}

impl WcaCommitPort for InCrateNiCertify {
    fn certify(&self, proposal_summary: &str, estimated_j: f64) -> Result<WcaCertResult> {
        self.wca.certify(proposal_summary, estimated_j)
    }
}

fn short_uuid() -> String {
    Uuid::new_v4().simple().to_string()[..12].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn live_commit_stamps_ids() {
        let ni = InCrateNiCertify::new();
        let out = ni
            .certify_live("convert 1 celsius", 1e-9, None, None)
            .unwrap();
        assert!(out.ni.allows_commit());
        assert!(out.ni.certificate_id.starts_with("ni:"));
        assert!(out.ni.efa_id.starts_with("efa:"));
        assert!(out.ni.wca_id.starts_with("wca:"));
        assert!(!out.ni.board_synth_claimed);
        assert!(!out.ni.stage_c_measured);
        assert_eq!(out.ni.source, CertifySource::InCrate);
    }

    #[test]
    fn live_refuse_diverge() {
        let ni = InCrateNiCertify::new();
        let out = ni
            .certify_live("act", 1e-9, Some("diverge".into()), None)
            .unwrap();
        assert!(!out.ni.allows_commit());
        assert!(out.floor.is_some());
        assert!(!out.ni.stage_c_measured);
    }
}
