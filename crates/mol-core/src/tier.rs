//! Cascade tiers mapped across MathGround, OpenIE zones, and thermo classes.

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::replay::ReplayClass;

/// MathGround / MoL cascade gear (cheapest first).
///
/// Lookup → Closed-form (Formula) → Sparse solver → Stochastic model LAST.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CascadeTier {
    /// Table / LUT / unit dictionary hit.
    Lookup,
    /// Closed-form formula / identity.
    Formula,
    /// Sparse / exact solver (linear algebra, SAT, …).
    /// Ising / adiabatic *energy-landscape settle* inspires this tier; v0.1 is classical.
    Solver,
    /// Stochastic / NN residual leaf — never substrate.
    Model,
}

impl CascadeTier {
    /// Routing order cheapest → hottest.
    pub const fn routing_order() -> [CascadeTier; 4] {
        [Self::Lookup, Self::Formula, Self::Solver, Self::Model]
    }

    /// Default OpenIE zone affinity.
    pub const fn openie_zone(self) -> OpenIeZone {
        match self {
            Self::Lookup | Self::Formula => OpenIeZone::Z1,
            Self::Solver => OpenIeZone::Z2,
            Self::Model => OpenIeZone::Z3,
        }
    }

    /// Default replay class when this tier answers.
    pub const fn default_replay(self) -> ReplayClass {
        match self {
            Self::Lookup | Self::Formula => ReplayClass::Deterministic,
            Self::Solver => ReplayClass::Composed,
            Self::Model => ReplayClass::ModelGenerated,
        }
    }

    /// Catalog impedance μ for this tier (ThermoClass mismatch order).
    pub const fn catalog_mu(self) -> f64 {
        self.thermo().mismatch_order()
    }

    /// Rough analytical joule estimate for a single tier attempt via `E ≈ θ·μ`.
    ///
    /// Surrogate from the clean-room μ catalog — **not** RAPL/NVML.
    pub fn surrogate_joules(self) -> f64 {
        crate::mu::MuCatalog::estimate_tier(self).estimated_j.0
    }

    /// Human label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Lookup => "lookup",
            Self::Formula => "formula",
            Self::Solver => "solver",
            Self::Model => "model",
        }
    }
}

impl fmt::Display for CascadeTier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// OpenIE synthesis zones Z1 → Z2 → Z3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum OpenIeZone {
    /// Z₁ — math = words. Bounded determinism.
    Z1,
    /// Z₂ — math ≈ words. Bounded inference / citation.
    Z2,
    /// Z₃ — unbounded generation. Statistical last resort.
    Z3,
}

impl OpenIeZone {
    /// Routing order.
    pub const fn routing_order() -> [OpenIeZone; 3] {
        [Self::Z1, Self::Z2, Self::Z3]
    }

    /// Label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Z1 => "Z1",
            Self::Z2 => "Z2",
            Self::Z3 => "Z3",
        }
    }
}

impl fmt::Display for OpenIeZone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// Coarse thermodynamic class (inherits leapfrog / compute-stack axis T).
///
/// Labels mismatch order-of-magnitude above Landauer — not measured RAPL/NVML.
/// Collapse-intelligence map: L0≈LUT/Formula, L1≈Solver settle, L2max≈Model residual.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ThermoClass {
    /// Near-Landauer / LUT / pure closed-form.
    L0,
    /// Deterministic silicon ops with modest mismatch.
    L1,
    /// Constrained inference / retrieval-bound.
    L2,
    /// Heavy statistical / generative.
    L2Max,
}

impl ThermoClass {
    /// Rough order-of-magnitude multiplier above Landauer.
    pub const fn mismatch_order(self) -> f64 {
        match self {
            Self::L0 => 1e3,
            Self::L1 => 1e9,
            Self::L2 => 1e12,
            Self::L2Max => 1e18,
        }
    }
}

impl fmt::Display for ThermoClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::L0 => write!(f, "L0"),
            Self::L1 => write!(f, "L1"),
            Self::L2 => write!(f, "L2"),
            Self::L2Max => write!(f, "L2max"),
        }
    }
}

impl CascadeTier {
    /// Default thermo class for the tier.
    pub const fn thermo(self) -> ThermoClass {
        match self {
            Self::Lookup => ThermoClass::L0,
            Self::Formula => ThermoClass::L0,
            Self::Solver => ThermoClass::L1,
            Self::Model => ThermoClass::L2Max,
        }
    }
}
