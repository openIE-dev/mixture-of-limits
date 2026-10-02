//! Limit surfaces / floors — where more bits stop buying outcomes.

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::energy::Joules;

/// Named limit identifier (stable string for receipts / registry).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LimitId(pub String);

impl LimitId {
    /// Construct from static or owned string.
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// As str.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for LimitId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Kind of floor / limit surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FloorKind {
    /// Information-theoretic floor (bits already sufficient / Shannon).
    Information,
    /// Landauer / joule thermodynamic floor (collapse-intelligence bound;
    /// reversible-computing erase cost — estimate ≠ RAPL/NVML).
    Energy,
    /// Value-of-information stop (marginal bits worthless under budget).
    ValueOfInformation,
    /// Grammar coverage closed — escalate would not help.
    GrammarCoverage,
    /// WCA / automation refuse (unsafe or policy).
    WcaRefuse,
    /// EFA-style energy-as-certificate refuse (Lyapunov / diverge veto before commit).
    EfaCertificate,
    /// Klere-style settle refuse (energy landscape will not settle → report, don't invent).
    SettleRefuse,
    /// Missing primitive on the Periodic Stack (258/33).
    /// QI/thermo empty cells (e.g. documented `physical_settle`) surface here.
    PrimitiveGap,
    /// Soft latency ceiling.
    Latency,
    /// Safety / capability deny.
    Safety,
    /// WASM / capsule encapsulation boundary violated (unsealed or host-session share).
    Encapsulation,
    /// Energy honesty violated (measured/estimated/modeled mislabel; invent measured_j).
    EnergyHonesty,
    /// Fail-closed: required provenance (cite / capsule / meter) missing.
    ProvenanceMissing,
    /// Agent Lane isolation violated (shared cookies/profile with opaque bots).
    AgentIsolation,
    /// Economic satiation stop — written completeness C(z)=1; refuse further synthesis.
    /// Distinct from VoI / energy / certificate refuses (economic done, not physical/cert).
    Satiation,
}

impl FloorKind {
    /// Wire / registry label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Information => "information",
            Self::Energy => "energy",
            Self::ValueOfInformation => "voi",
            Self::GrammarCoverage => "grammar_coverage",
            Self::WcaRefuse => "wca_refuse",
            Self::EfaCertificate => "efa_certificate",
            Self::SettleRefuse => "settle_refuse",
            Self::PrimitiveGap => "primitive_gap",
            Self::Latency => "latency",
            Self::Safety => "safety",
            Self::Encapsulation => "encapsulation",
            Self::EnergyHonesty => "energy_honesty",
            Self::ProvenanceMissing => "provenance_missing",
            Self::AgentIsolation => "agent_isolation",
            Self::Satiation => "satiation",
        }
    }
}

impl fmt::Display for FloorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// A concrete floor binding: the limit that stopped escalation or commit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Floor {
    /// Registry id.
    pub id: LimitId,
    /// Kind.
    pub kind: FloorKind,
    /// Human reason.
    pub reason: String,
    /// Optional joule threshold that bound.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub joule_threshold: Option<Joules>,
    /// Optional latency threshold (ms).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latency_ms_threshold: Option<u64>,
}

impl Floor {
    /// Construct a binding floor.
    pub fn new(id: impl Into<String>, kind: FloorKind, reason: impl Into<String>) -> Self {
        Self {
            id: LimitId::new(id),
            kind,
            reason: reason.into(),
            joule_threshold: None,
            latency_ms_threshold: None,
        }
    }

    /// With joule threshold.
    pub fn with_joules(mut self, j: Joules) -> Self {
        self.joule_threshold = Some(j);
        self
    }

    /// With latency threshold.
    pub fn with_latency_ms(mut self, ms: u64) -> Self {
        self.latency_ms_threshold = Some(ms);
        self
    }
}
