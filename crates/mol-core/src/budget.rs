//! Request budgets: max joules and max latency.

use serde::{Deserialize, Serialize};

use crate::energy::Joules;

/// Soft latency budget (wall-clock ms; not RAPL).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LatencyBudget {
    /// Maximum soft wall-clock milliseconds.
    pub max_ms: u64,
}

impl LatencyBudget {
    /// Construct.
    pub const fn new(max_ms: u64) -> Self {
        Self { max_ms }
    }
}

/// Combined joule + latency budget for a MoL request.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Budget {
    /// Maximum estimated joules allowed for the cascade path.
    pub max_j: Joules,
    /// Optional soft latency ceiling.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_latency: Option<LatencyBudget>,
    /// When false (default), model tier returns ModelRefused / NotCovered.
    #[serde(default)]
    pub allow_model: bool,
}

impl Budget {
    /// Construct with joule ceiling only.
    pub fn joules(max_j: f64) -> Self {
        Self {
            max_j: Joules::new(max_j),
            max_latency: None,
            allow_model: false,
        }
    }

    /// With latency.
    pub fn with_latency_ms(mut self, ms: u64) -> Self {
        self.max_latency = Some(LatencyBudget::new(ms));
        self
    }

    /// Explicitly allow the demoted model leaf.
    pub fn allow_model(mut self) -> Self {
        self.allow_model = true;
        self
    }

    /// Coin-cell / edge default: tiny joule budget, no model.
    pub fn coin_cell() -> Self {
        Self::joules(1.0e-6).with_latency_ms(5)
    }

    /// Desktop demo default.
    pub fn demo() -> Self {
        Self::joules(1.0e-3).with_latency_ms(100)
    }
}

impl Default for Budget {
    fn default() -> Self {
        Self::demo()
    }
}
