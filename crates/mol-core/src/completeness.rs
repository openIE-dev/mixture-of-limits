//! Written completeness C(z) — economic satiation stop.
//!
//! When all clauses are true, C(z)=1 and further synthesis refuses with
//! [`FloorKind::Satiation`] (economic reason code). Distinct from VoI / cert / energy.

use serde::{Deserialize, Serialize};

/// One named completeness clause.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompletenessClause {
    /// Stable clause id (e.g. `resolution_code_set`).
    pub id: String,
    /// Whether the clause currently holds.
    pub holds: bool,
}

impl CompletenessClause {
    /// Construct.
    pub fn new(id: impl Into<String>, holds: bool) -> Self {
        Self {
            id: id.into(),
            holds,
        }
    }
}

/// Snapshot of written completeness C(z) for an episode / chore.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompletenessSnapshot {
    /// Predicate id (e.g. `C_ticket_close`).
    pub id: String,
    /// Clauses; C(z)=1 iff every clause holds.
    pub clauses: Vec<CompletenessClause>,
}

impl CompletenessSnapshot {
    /// Construct.
    pub fn new(id: impl Into<String>, clauses: Vec<CompletenessClause>) -> Self {
        Self {
            id: id.into(),
            clauses,
        }
    }

    /// Support-desk MVP: all three ticket-close clauses.
    pub fn ticket_close(resolution_set: bool, ack_or_sla: bool, no_blocker: bool) -> Self {
        Self::new(
            "C_ticket_close",
            vec![
                CompletenessClause::new("resolution_code_set", resolution_set),
                CompletenessClause::new("customer_ack_or_sla_elapsed", ack_or_sla),
                CompletenessClause::new("no_open_blocker", no_blocker),
            ],
        )
    }

    /// Financial risk sketch: three publish clauses.
    pub fn risk_score(features: bool, in_bounds: bool, cert: bool) -> Self {
        Self::new(
            "C_risk_score",
            vec![
                CompletenessClause::new("required_features_present", features),
                CompletenessClause::new("score_within_policy_bounds", in_bounds),
                CompletenessClause::new("certificate_for_publish", cert),
            ],
        )
    }

    /// C(z)=1 when every clause holds (empty ⇒ false — never silent satiation).
    pub fn is_complete(&self) -> bool {
        !self.clauses.is_empty() && self.clauses.iter().all(|c| c.holds)
    }

    /// Economic refuse rationale.
    pub fn satiation_reason(&self) -> String {
        let held: Vec<&str> = self
            .clauses
            .iter()
            .filter(|c| c.holds)
            .map(|c| c.id.as_str())
            .collect();
        format!(
            "economic satiation: C(z)=1 for predicate '{}' (clauses true: {:?}); refuse further synthesis — distinct from VoI/energy/certificate refuse",
            self.id, held
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn incomplete_until_all_hold() {
        let c = CompletenessSnapshot::ticket_close(true, true, false);
        assert!(!c.is_complete());
        let c2 = CompletenessSnapshot::ticket_close(true, true, true);
        assert!(c2.is_complete());
    }
}
