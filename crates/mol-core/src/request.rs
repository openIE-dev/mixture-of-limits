//! MoL request surface.

use serde::{Deserialize, Serialize};

use crate::agent_lane::AgentIsolationPolicy;
use crate::budget::Budget;
use crate::encapsulation::CapsuleContext;
use crate::completeness::CompletenessSnapshot;
use crate::meter::MeterSample;
use crate::stack::PeriodicStack;

/// Fail-closed gates: when set, missing provenance / meter / capsule → refuse.
///
/// Soft-ref default is all-false (estimation floor remains universal). Turn gates
/// on for consequential acts that must not proceed without evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct FailClosedPolicy {
    /// Refuse commit when measured_j is absent (never invent to satisfy).
    #[serde(default)]
    pub require_measured: bool,
    /// Refuse when no sealed capsule context is attached.
    #[serde(default)]
    pub require_capsule: bool,
    /// Refuse when citation provenance is empty on cite/compose paths.
    #[serde(default)]
    pub require_citations: bool,
    /// Refuse when Agent Lane isolation policy is missing or fails check.
    #[serde(default)]
    pub require_agent_isolation: bool,
}

impl FailClosedPolicy {
    /// No fail-closed gates (default soft-ref / WASM estimate path).
    pub const fn open() -> Self {
        Self {
            require_measured: false,
            require_capsule: false,
            require_citations: false,
            require_agent_isolation: false,
        }
    }

    /// Strict ecosystem posture: capsule + agent isolation required.
    pub const fn encapsulated_agent() -> Self {
        Self {
            require_measured: false,
            require_capsule: true,
            require_citations: false,
            require_agent_isolation: true,
        }
    }

    /// Meter-required posture (consequential energy-certified act).
    pub const fn meter_required() -> Self {
        Self {
            require_measured: true,
            require_capsule: false,
            require_citations: false,
            require_agent_isolation: false,
        }
    }

    /// Any gate enabled?
    pub const fn any(self) -> bool {
        self.require_measured
            || self.require_capsule
            || self.require_citations
            || self.require_agent_isolation
    }
}

/// Coarse query kind for grammar classification (demo / v0.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryKind {
    /// Unit conversion dictionary.
    UnitConvert,
    /// Closed-form physics / Landauer / E=mc² / Shannon rate style.
    ClosedFormPhysics,
    /// Small dense linear solve.
    LinearSolve,
    /// Ternary / energy-landscape settle (Klere-style Solver gear).
    Settle,
    /// Named Periodic Stack primitive gap probe (real registry miss/Gap).
    PrimitiveGap,
    /// Periodic Stack subset navigate (family / primitive / scale).
    StackNavigate,
    /// Factual knowledge ask — Z2 retrieve+cite (refuse if unknown).
    FactualClaim,
    /// Z1 compose/synthesis from ≥2 cited claims (ReplayClass::Composed).
    Compose,
    /// Bitemporal memory write proposal (lands only on MoL close COMMIT).
    MemoryWrite,
    /// Bitemporal memory recall (cited).
    MemoryRecall,
    /// Free-form / unknown — may hit model leaf or refuse.
    FreeForm,
    /// Support-desk ticket close / resolution-code Lookup (O(1) Bloom/trie path).
    TicketClose,
}

impl QueryKind {
    /// Heuristic classify from query text (demo classifier; not an NN).
    ///
    /// `PrimitiveGap` uses the in-tree [`PeriodicStack`] probe (Gap markers /
    /// absent names), not string heuristics alone.
    pub fn classify(query: &str) -> Self {
        let q = query.to_ascii_lowercase();
        let stack = PeriodicStack::subset();

        // Real stack gap probe first (registry-backed).
        if stack.probe_query(query).is_gap() {
            return Self::PrimitiveGap;
        }

        if PeriodicStack::is_navigate_query(query) {
            return Self::StackNavigate;
        }

        // Memory write/recall before physics/formula heuristics (e.g. remember … landauer …).
        if crate::memory::looks_remember_ask(query) {
            return Self::MemoryWrite;
        }
        if crate::memory::looks_recall_ask(query) {
            return Self::MemoryRecall;
        }

        if q.contains("settle")
            || q.contains("ternary")
            || q.contains("attractor")
            || q.contains("klere settle")
        {
            Self::Settle
        } else if q.contains("convert")
            || q.contains("celsius")
            || q.contains("fahrenheit")
            || q.contains("meters")
            || q.contains("feet")
            || q.contains("electronvolt")
            || q.contains(" electron")
            || (q.contains(" to ")
                && (q.contains("kg")
                    || q.contains("lb")
                    || q.contains("km")
                    || q.contains("joule")
                    || q.contains("ev")))
            || q.contains("unit cascade")
        {
            Self::UnitConvert
        } else if (
            q.contains("landauer")
                && (q.contains("joules")
                    || q.contains("per bit")
                    || q.contains("k_b")
                    || q.contains("kbt")
                    || q.contains("ln2")
                    || q.contains("ln 2")
                    || q.contains("kelvin")
                    || q.contains("floor"))
        )
            || q.contains("e=mc")
            || q.contains("e = mc")
            || q.contains("rest energy")
            || q.contains("joules per bit")
            || q.contains("shannon capacity")
            || q.contains("shannon rate")
            || q.contains("nyquist")
            || q.contains("max bits")
            || q.contains("bit bound")
            || (q.contains("log2") && (q.contains("snr") || q.contains("bandwidth")))
            || ((q.contains("k_b") || q.contains("kbt") || q.contains("ln2") || q.contains("ln 2"))
                && (q.contains("bit") || q.contains("landauer") || q.contains("joule")))
        {
            Self::ClosedFormPhysics
        } else if q.contains("solve")
            || q.contains("linear system")
            || q.contains("2x2")
            || q.contains("matrix")
        {
            Self::LinearSolve
        } else if crate::claims::looks_compose_ask(query) {
            Self::Compose
        } else if crate::claims::looks_factual_ask(query) {
            Self::FactualClaim
        } else if q.contains("resolution")
            || q.contains("ticket close")
            || q.contains("ticket_close")
            || q.contains("resolve ticket")
            || q.starts_with("r-")
            || q.contains(" resolution=")
            || q.contains("resolution_code")
            || q.contains("risk score")
            || q.contains("risk-")
            || q.contains("band=")
            || q.contains("risk_score")
        {
            Self::TicketClose
        } else {
            Self::FreeForm
        }
    }
}

/// A Mixture of Limits request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MolRequest {
    /// Natural-language or structured query text.
    pub query: String,
    /// Classified kind (may be overridden).
    pub kind: QueryKind,
    /// Joule / latency / model budget.
    pub budget: Budget,
    /// Optional WASM / capsule encapsulation context.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capsule: Option<CapsuleContext>,
    /// Optional Agent Lane isolation policy.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_lane: Option<AgentIsolationPolicy>,
    /// Fail-closed gates for provenance / meter / capsule / agent isolation.
    #[serde(default)]
    pub fail_closed: FailClosedPolicy,
    /// Optional pre-sampled OS meter evidence for `require_measured` / receipt stamp.
    ///
    /// Attach only real [`MeterSample`] values from a probe (or honest fixtures).
    /// Never invent `measured_j` to satisfy fail-closed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meter_sample: Option<MeterSample>,
    /// Written completeness C(z) for satiation stop. When `is_complete()`, route
    /// refuses with FloorKind::Satiation before further synthesis.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completeness: Option<CompletenessSnapshot>,
}

impl MolRequest {
    /// Construct with auto-classified kind.
    pub fn new(query: impl Into<String>, budget: Budget) -> Self {
        let query = query.into();
        let kind = QueryKind::classify(&query);
        Self {
            query,
            kind,
            budget,
            capsule: None,
            agent_lane: None,
            fail_closed: FailClosedPolicy::open(),
            meter_sample: None,
            completeness: None,
        }
    }

    /// Override kind.
    pub fn with_kind(mut self, kind: QueryKind) -> Self {
        self.kind = kind;
        self
    }

    /// Attach capsule context.
    pub fn with_capsule(mut self, capsule: CapsuleContext) -> Self {
        self.capsule = Some(capsule);
        self
    }

    /// Attach agent isolation policy.
    pub fn with_agent_lane(mut self, policy: AgentIsolationPolicy) -> Self {
        self.agent_lane = Some(policy);
        self
    }

    /// Set fail-closed policy.
    pub fn with_fail_closed(mut self, policy: FailClosedPolicy) -> Self {
        self.fail_closed = policy;
        self
    }

    /// Attach a real OS meter sample (or honest fixture). Never invent joules.
    pub fn with_meter_sample(mut self, sample: MeterSample) -> Self {
        self.meter_sample = Some(sample);
        self
    }

    /// Attach written completeness C(z) for satiation gate.
    pub fn with_completeness(mut self, c: CompletenessSnapshot) -> Self {
        self.completeness = Some(c);
        self
    }
}
