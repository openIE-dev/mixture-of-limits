//! Value of Information stop — marginal bits that no longer buy outcomes.

use serde::{Deserialize, Serialize};

use crate::energy::Joules;

/// Value-of-information assessment for escalating to a hotter tier.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValueOfInformation {
    /// Estimated marginal utility of more bits / hotter tier (0..1 heuristic).
    pub marginal_utility: f64,
    /// Estimated marginal joule cost of escalation.
    pub marginal_cost_j: Joules,
    /// Threshold below which VoI says stop.
    pub stop_threshold: f64,
    /// Short rationale.
    pub rationale: String,
}

impl ValueOfInformation {
    /// Construct.
    pub fn new(
        marginal_utility: f64,
        marginal_cost_j: f64,
        stop_threshold: f64,
        rationale: impl Into<String>,
    ) -> Self {
        Self {
            marginal_utility,
            marginal_cost_j: Joules::new(marginal_cost_j),
            stop_threshold,
            rationale: rationale.into(),
        }
    }

    /// True when marginal utility is at or below the stop threshold.
    pub fn should_stop(&self) -> bool {
        self.marginal_utility <= self.stop_threshold
    }

    /// Utility per joule (heuristic).
    pub fn utility_per_joule(&self) -> f64 {
        if self.marginal_cost_j.0 <= 0.0 {
            f64::INFINITY
        } else {
            self.marginal_utility / self.marginal_cost_j.0
        }
    }
}

/// Decision from a VoI gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VoiDecision {
    /// Escalate to next cheaper-sufficient tier still within budget.
    Escalate,
    /// Stop — VoI floor binding.
    Stop,
}

impl ValueOfInformation {
    /// Map to decision.
    pub fn decide(&self) -> VoiDecision {
        if self.should_stop() {
            VoiDecision::Stop
        } else {
            VoiDecision::Escalate
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stop_when_utility_low() {
        let v = ValueOfInformation::new(0.01, 1e-3, 0.05, "diminishing");
        assert_eq!(v.decide(), VoiDecision::Stop);
    }
}
