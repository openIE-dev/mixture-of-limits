//! Automate gate: propose → LimitCheck → certify → Commit|Refuse → receipt.

use mol_adapters::{
    EfaCertificatePort, EfaDecision, EfaProposal, StubEfaCertificate, StubWcaCommit,
    WcaCommitPort,
};
use mol_core::{
    looks_remember_ask, Budget, EstimateKind, Floor, FloorKind, Joules, MeasureSource,
    MolRequest, MuSource, BOARD_SYNTH_CLAIMED,
};
use mol_limits::{CloseOutcome, MixtureOfLimits};
use mol_receipt::{MolReceipt, ReceiptBuilder};
use serde::{Deserialize, Serialize};

use crate::act::{Act, ActKind, CapabilitySet};

/// Machine-readable refuse reason.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum RefuseReason {
    /// Capability not granted.
    CapabilityDenied {
        /// Detail.
        detail: String,
    },
    /// MoL limit fired.
    LimitFired {
        /// Limit id.
        limit_id: String,
        /// Detail.
        detail: String,
    },
    /// Budget exceeded.
    BudgetExceeded {
        /// Detail.
        detail: String,
    },
    /// Certificate refuse (EFA / WCA).
    CertificateRefuse {
        /// Detail.
        detail: String,
    },
    /// Policy / safety.
    Policy {
        /// Detail.
        detail: String,
    },
}

/// Commit decision.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommitDecision {
    /// Allowed to execute.
    Commit,
    /// Refused — no side effect.
    Refuse(RefuseReason),
}

/// Outcome of the automate gate.
#[derive(Debug, Clone)]
pub struct AutomateOutcome {
    /// Decision.
    pub decision: CommitDecision,
    /// Whether side effect ran (always false on refuse; true only if caller executes after Commit).
    pub executed: bool,
    /// Receipt.
    pub receipt: MolReceipt,
    /// Always false.
    pub board_synth_claimed: bool,
}

/// Capability-gated automate loop with certify ports.
pub struct AutomateGate {
    /// Capabilities.
    pub caps: CapabilitySet,
    /// MoL router for ask-shaped acts.
    pub mol: MixtureOfLimits,
    /// Max joules for an act.
    pub max_j: f64,
}

impl Default for AutomateGate {
    fn default() -> Self {
        Self {
            caps: CapabilitySet::sense_propose(),
            mol: MixtureOfLimits::new(),
            max_j: 1e-3,
        }
    }
}

impl AutomateGate {
    /// Construct with caps.
    pub fn new(caps: CapabilitySet) -> Self {
        Self {
            caps,
            mol: MixtureOfLimits::new(),
            max_j: 1e-3,
        }
    }

    /// Propose → LimitCheck → certify (EFA+WCA) → Commit|Refuse → receipt.
    ///
    /// Does **not** perform irreversible I/O itself; on `Commit` the caller may
    /// execute. Mutate/External without capability → refuse.
    pub fn gate(&self, act: &Act) -> AutomateOutcome {
        self.gate_with(act, &StubEfaCertificate, &StubWcaCommit)
    }

    /// Gate with injected certify ports.
    pub fn gate_with(
        &self,
        act: &Act,
        efa: &dyn EfaCertificatePort,
        wca: &dyn WcaCommitPort,
    ) -> AutomateOutcome {
        let required = CapabilitySet::required(act.kind);
        if !self.caps.allows(required) {
            let reason = RefuseReason::CapabilityDenied {
                detail: format!("{:?} not granted for {:?}", required, act.kind),
            };
            return self.refuse(act, reason, None);
        }

        if act.estimated_j > self.max_j {
            let reason = RefuseReason::BudgetExceeded {
                detail: format!(
                    "act estimated_j {:.3e} > gate max_j {:.3e}",
                    act.estimated_j, self.max_j
                ),
            };
            return self.refuse(act, reason, None);
        }

        // For Propose/Sense acts that look like queries, run MoL close.
        if matches!(act.kind, ActKind::Propose | ActKind::Sense) {
            if let Some(q) = act
                .payload
                .as_ref()
                .and_then(|v| v.get("query"))
                .and_then(|v| v.as_str())
            {
                let req = MolRequest::new(q, Budget::joules(self.max_j));
                match self.mol.close_with(&req, efa, wca) {
                    Ok(CloseOutcome::Commit { receipt, .. }) => {
                        let mut receipt = receipt;
                        receipt.executed = Some(false);
                        return AutomateOutcome {
                            decision: CommitDecision::Commit,
                            executed: false,
                            receipt,
                            board_synth_claimed: BOARD_SYNTH_CLAIMED,
                        };
                    }
                    Ok(CloseOutcome::Refuse { floor, receipt }) => {
                        let reason = RefuseReason::LimitFired {
                            limit_id: floor.id.as_str().into(),
                            detail: floor.reason.clone(),
                        };
                        return AutomateOutcome {
                            decision: CommitDecision::Refuse(reason),
                            executed: false,
                            receipt,
                            board_synth_claimed: BOARD_SYNTH_CLAIMED,
                        };
                    }
                    Err(e) => {
                        let reason = RefuseReason::Policy {
                            detail: e.to_string(),
                        };
                        return self.refuse(act, reason, None);
                    }
                }
            }
        }

        // Memory write via Mutate: MoL close owns commit; store write lands only on COMMIT.
        // Capability already checked above — without Mutate this path is unreachable.
        if matches!(act.kind, ActKind::Mutate) {
            let mem_q = act
                .payload
                .as_ref()
                .and_then(|v| v.get("query"))
                .and_then(|v| v.as_str())
                .filter(|q| looks_remember_ask(q))
                .or_else(|| {
                    if looks_remember_ask(&act.summary) {
                        Some(act.summary.as_str())
                    } else {
                        None
                    }
                });
            if let Some(q) = mem_q {
                let req = MolRequest::new(q, Budget::joules(self.max_j));
                match self.mol.close_with(&req, efa, wca) {
                    Ok(CloseOutcome::Commit { receipt, .. }) => {
                        return AutomateOutcome {
                            decision: CommitDecision::Commit,
                            executed: receipt.executed.unwrap_or(true),
                            receipt,
                            board_synth_claimed: BOARD_SYNTH_CLAIMED,
                        };
                    }
                    Ok(CloseOutcome::Refuse { floor, receipt }) => {
                        let reason = RefuseReason::LimitFired {
                            limit_id: floor.id.as_str().into(),
                            detail: floor.reason.clone(),
                        };
                        return AutomateOutcome {
                            decision: CommitDecision::Refuse(reason),
                            executed: false,
                            receipt,
                            board_synth_claimed: BOARD_SYNTH_CLAIMED,
                        };
                    }
                    Err(e) => {
                        let reason = RefuseReason::Policy {
                            detail: e.to_string(),
                        };
                        return self.refuse(act, reason, None);
                    }
                }
            }
        }

        // Mutate/External: soft safety then certify.
        if matches!(act.kind, ActKind::Mutate | ActKind::External) {
            let s = act.summary.to_ascii_lowercase();
            if s.contains("rm -rf") || s.contains("drop table") || s.contains("format disk") {
                let reason = RefuseReason::Policy {
                    detail: "destructive pattern refused (refuse over escalate)".into(),
                };
                return self.refuse(act, reason, None);
            }
        }

        // Certify before commit certificate.
        let efa_res = match efa.certify(&EfaProposal {
            summary: act.summary.clone(),
            estimated_j: act.estimated_j,
            energy_residual: act
                .payload
                .as_ref()
                .and_then(|v| v.get("energy_residual"))
                .and_then(|v| v.as_f64()),
            tag: act
                .payload
                .as_ref()
                .and_then(|v| v.get("tag"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        }) {
            Ok(r) => r,
            Err(e) => {
                return self.refuse(
                    act,
                    RefuseReason::Policy {
                        detail: e.to_string(),
                    },
                    None,
                );
            }
        };
        if efa_res.decision == EfaDecision::Refuse {
            let floor = efa_res.floor.clone().unwrap_or_else(|| {
                Floor::new(
                    "efa_certificate",
                    FloorKind::EfaCertificate,
                    "EFA certificate refuse",
                )
            });
            return self.refuse(
                act,
                RefuseReason::CertificateRefuse {
                    detail: efa_res.reasons.join("; "),
                },
                Some(floor),
            );
        }

        let wca_res = match wca.certify(&act.summary, act.estimated_j) {
            Ok(r) => r,
            Err(e) => {
                return self.refuse(
                    act,
                    RefuseReason::Policy {
                        detail: e.to_string(),
                    },
                    None,
                );
            }
        };
        if wca_res.decision != "allow" {
            let floor = wca_res.floor.clone().unwrap_or_else(|| {
                Floor::new("wca_refuse", FloorKind::WcaRefuse, "WCA certify refuse")
            });
            return self.refuse(
                act,
                RefuseReason::CertificateRefuse {
                    detail: wca_res.reasons.join("; "),
                },
                Some(floor),
            );
        }

        let receipt = ReceiptBuilder::new()
            .query(&act.summary)
            .estimated_j(Joules::new(act.estimated_j))
            .estimate_kind(EstimateKind::Analytical)
            .measure_source(MeasureSource::CatalogSurrogate)
            .mu_source(MuSource::Catalog)
            .executed(false)
            .rationale(format!(
                "automate gate COMMIT for {:?} after EFA+WCA certify: {}",
                act.kind, act.summary
            ))
            .build();
        AutomateOutcome {
            decision: CommitDecision::Commit,
            executed: false,
            receipt,
            board_synth_claimed: BOARD_SYNTH_CLAIMED,
        }
    }

    fn refuse(&self, act: &Act, reason: RefuseReason, floor: Option<Floor>) -> AutomateOutcome {
        let detail = match &reason {
            RefuseReason::CapabilityDenied { detail }
            | RefuseReason::BudgetExceeded { detail }
            | RefuseReason::Policy { detail }
            | RefuseReason::CertificateRefuse { detail } => detail.clone(),
            RefuseReason::LimitFired { detail, .. } => detail.clone(),
        };
        let floor = floor.unwrap_or_else(|| {
            Floor::new("wca_refuse", FloorKind::WcaRefuse, detail.clone())
        });
        let receipt = ReceiptBuilder::new()
            .query(&act.summary)
            .estimated_j(Joules::new(act.estimated_j))
            .estimate_kind(EstimateKind::Analytical)
            .measure_source(MeasureSource::CatalogSurrogate)
            .mu_source(MuSource::Catalog)
            .limit_fired(floor)
            .executed(false)
            .rationale(format!("automate REFUSE: {detail}"))
            .build();
        AutomateOutcome {
            decision: CommitDecision::Refuse(reason),
            executed: false,
            receipt,
            board_synth_claimed: BOARD_SYNTH_CLAIMED,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::act::{Act, ActKind, Capability, CapabilitySet};

    #[test]
    fn mutate_denied_by_default() {
        let g = AutomateGate::default();
        let act = Act::new(ActKind::Mutate, "write file", 1e-9);
        let out = g.gate(&act);
        assert!(matches!(out.decision, CommitDecision::Refuse(_)));
        assert!(!out.executed);
        assert!(!out.board_synth_claimed);
    }

    #[test]
    fn propose_formula_commits() {
        let g = AutomateGate::default();
        let mut act = Act::new(ActKind::Propose, "ask landauer", 1e-12);
        act.payload = Some(serde_json::json!({"query": "landauer joules per bit"}));
        let out = g.gate(&act);
        assert!(matches!(out.decision, CommitDecision::Commit));
    }

    #[test]
    #[test]
    fn memory_write_requires_mutate_cap() {
        let g = AutomateGate::default(); // sense+propose only
        let mut act = Act::new(ActKind::Mutate, "remember k = v", 1e-12);
        act.payload = Some(serde_json::json!({"query": "remember k = v"}));
        let out = g.gate(&act);
        assert!(matches!(
            out.decision,
            CommitDecision::Refuse(RefuseReason::CapabilityDenied { .. })
        ));
        assert!(g.mol.memory_lock().is_empty());
    }

    #[test]
    fn memory_write_with_mutate_commits_store() {
        let mut caps = CapabilitySet::sense_propose();
        caps.granted.push(Capability::Mutate);
        let g = AutomateGate::new(caps);
        let mut act = Act::new(ActKind::Mutate, "remember note = hello", 1e-12);
        act.payload = Some(serde_json::json!({"query": "remember note = hello"}));
        let out = g.gate(&act);
        assert!(matches!(out.decision, CommitDecision::Commit));
        assert_eq!(g.mol.memory_lock().len(), 1);
        assert!(!out.board_synth_claimed);
        assert!(out.receipt.measured_j.is_none());
    }

    fn mutate_with_cap_efa_refuses_diverge() {
        let mut caps = CapabilitySet::sense_propose();
        caps.granted.push(Capability::Mutate);
        let g = AutomateGate::new(caps);
        let mut act = Act::new(ActKind::Mutate, "arm swing", 1e-9);
        act.payload = Some(serde_json::json!({"tag": "diverge"}));
        let out = g.gate(&act);
        assert!(matches!(
            out.decision,
            CommitDecision::Refuse(RefuseReason::CertificateRefuse { .. })
        ));
        assert_eq!(
            out.receipt.limit_fired.as_ref().map(|f| f.kind),
            Some(FloorKind::EfaCertificate)
        );
    }
}
