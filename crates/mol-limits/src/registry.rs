//! Named limit definitions and evaluation.

use mol_core::{
    Floor, FloorKind, Joules, LimitId, MolRequest, PeriodicStack, QueryKind,
    ValueOfInformation, VoiDecision,
};
use serde::{Deserialize, Serialize};

/// Result of evaluating one limit against a request.
#[derive(Debug, Clone, PartialEq)]
pub enum LimitEval {
    /// Limit does not bind — continue.
    Pass,
    /// Limit binds — stop with this floor.
    Bind(Floor),
}

/// A named limit in the registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamedLimit {
    /// Stable id.
    pub id: LimitId,
    /// Kind.
    pub kind: FloorKind,
    /// Human description.
    pub description: String,
    /// Optional static joule ceiling for this limit alone.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_j: Option<f64>,
    /// Optional VoI stop threshold (marginal utility).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub voi_stop_threshold: Option<f64>,
    /// When true, free-form / unknown grammar binds this limit (refuse escalate).
    #[serde(default)]
    pub refuse_unknown_grammar: bool,
    /// When true, missing allow_model on free-form binds.
    #[serde(default)]
    pub require_explicit_model: bool,
}

impl NamedLimit {
    /// Evaluate against request + optional estimated path cost so far.
    pub fn evaluate(&self, req: &MolRequest, path_cost_j: f64) -> LimitEval {
        match self.kind {
            FloorKind::Energy => {
                let ceiling = self.max_j.unwrap_or(req.budget.max_j.0);
                if path_cost_j > ceiling {
                    LimitEval::Bind(
                        Floor::new(
                            self.id.as_str(),
                            FloorKind::Energy,
                            format!(
                                "{}: path cost {path_cost_j:.3e} J exceeds ceiling {ceiling:.3e} J",
                                self.description
                            ),
                        )
                        .with_joules(Joules::new(ceiling)),
                    )
                } else if req.budget.max_j.0 <= 0.0 {
                    LimitEval::Bind(
                        Floor::new(
                            self.id.as_str(),
                            FloorKind::Energy,
                            "non-positive joule budget",
                        )
                        .with_joules(req.budget.max_j),
                    )
                } else {
                    LimitEval::Pass
                }
            }
            FloorKind::Latency => {
                if let Some(lat) = req.budget.max_latency {
                    if lat.max_ms == 0 {
                        return LimitEval::Bind(
                            Floor::new(
                                self.id.as_str(),
                                FloorKind::Latency,
                                "zero latency budget",
                            )
                            .with_latency_ms(0),
                        );
                    }
                }
                LimitEval::Pass
            }
            FloorKind::ValueOfInformation => {
                let thr = self.voi_stop_threshold.unwrap_or(0.05);
                // Heuristic: free-form with tiny budget → VoI stop before model.
                if matches!(req.kind, QueryKind::FreeForm) && !req.budget.allow_model {
                    let voi = ValueOfInformation::new(
                        0.01,
                        1e-3,
                        thr,
                        "free-form without allow_model: marginal utility of tokens ≈ 0 under MoL",
                    );
                    if voi.decide() == VoiDecision::Stop {
                        return LimitEval::Bind(Floor::new(
                            self.id.as_str(),
                            FloorKind::ValueOfInformation,
                            format!("{} ({})", self.description, voi.rationale),
                        ));
                    }
                }
                LimitEval::Pass
            }
            FloorKind::GrammarCoverage => {
                if self.refuse_unknown_grammar
                    && matches!(req.kind, QueryKind::FreeForm)
                    && !req.budget.allow_model
                {
                    LimitEval::Bind(Floor::new(
                        self.id.as_str(),
                        FloorKind::GrammarCoverage,
                        format!(
                            "{}: free-form not covered by Lookup/Formula/Solver grammar",
                            self.description
                        ),
                    ))
                } else {
                    LimitEval::Pass
                }
            }
            FloorKind::Safety => LimitEval::Pass, // checked in automate
            FloorKind::WcaRefuse => LimitEval::Pass, // wired via automate certify
            FloorKind::EfaCertificate => LimitEval::Pass, // wired via automate / close certify
            FloorKind::SettleRefuse => LimitEval::Pass, // cascade settle gear binds
            FloorKind::Information => LimitEval::Pass,
            // Ecosystem floors — evaluated in MixtureOfLimits::route fail-closed gate.
            FloorKind::Encapsulation => LimitEval::Pass,
            FloorKind::EnergyHonesty => LimitEval::Pass,
            FloorKind::ProvenanceMissing => LimitEval::Pass,
            FloorKind::AgentIsolation => LimitEval::Pass,
            FloorKind::PrimitiveGap => {
                // Registry-backed probe (Gap markers / absent names) — not string-only.
                let stack = PeriodicStack::subset();
                let probe = stack.probe_query(&req.query);
                if probe.is_gap() || matches!(req.kind, QueryKind::PrimitiveGap) {
                    let detail = match &probe {
                        mol_core::ProbeResult::Gap { name, family, reason } => {
                            format!(
                                "{}: probe name='{}' family={:?} — {}",
                                self.description, name, family, reason
                            )
                        }
                        _ => format!(
                            "{}: Periodic Stack primitive missing for '{}'",
                            self.description, req.query
                        ),
                    };
                    LimitEval::Bind(Floor::new(
                        self.id.as_str(),
                        FloorKind::PrimitiveGap,
                        detail,
                    ))
                } else {
                    LimitEval::Pass
                }
            }
            FloorKind::Satiation => {
                // Economic done: C(z)=1 → refuse further synthesis.
                if let Some(ref c) = req.completeness {
                    if c.is_complete() {
                        return LimitEval::Bind(Floor::new(
                            self.id.as_str(),
                            FloorKind::Satiation,
                            c.satiation_reason(),
                        ));
                    }
                }
                LimitEval::Pass
            }
        }
    }
}

/// Trait for custom limit checks (extensibility).
pub trait LimitCheck: Send + Sync {
    /// Limit id.
    fn id(&self) -> &str;
    /// Evaluate.
    fn check(&self, req: &MolRequest, path_cost_j: f64) -> LimitEval;
}

impl LimitCheck for NamedLimit {
    fn id(&self) -> &str {
        self.id.as_str()
    }

    fn check(&self, req: &MolRequest, path_cost_j: f64) -> LimitEval {
        self.evaluate(req, path_cost_j)
    }
}

/// Default registry — order matters (first binding wins).
pub fn default_registry() -> Vec<NamedLimit> {
    vec![
        NamedLimit {
            id: LimitId::new("safety"),
            kind: FloorKind::Safety,
            description: "safety / capability deny".into(),
            max_j: None,
            voi_stop_threshold: None,
            refuse_unknown_grammar: false,
            require_explicit_model: false,
        },
        NamedLimit {
            id: LimitId::new("satiation"),
            kind: FloorKind::Satiation,
            description: "economic satiation — C(z)=1 refuse further synthesis".into(),
            max_j: None,
            voi_stop_threshold: None,
            refuse_unknown_grammar: false,
            require_explicit_model: false,
        },

        NamedLimit {
            id: LimitId::new("energy"),
            kind: FloorKind::Energy,
            description: "joule budget floor (Landauer-aware estimates)".into(),
            max_j: None,
            voi_stop_threshold: None,
            refuse_unknown_grammar: false,
            require_explicit_model: false,
        },
        NamedLimit {
            id: LimitId::new("latency"),
            kind: FloorKind::Latency,
            description: "soft latency ceiling".into(),
            max_j: None,
            voi_stop_threshold: None,
            refuse_unknown_grammar: false,
            require_explicit_model: false,
        },
        NamedLimit {
            id: LimitId::new("voi"),
            kind: FloorKind::ValueOfInformation,
            description: "value-of-information stop".into(),
            max_j: None,
            voi_stop_threshold: Some(0.05),
            refuse_unknown_grammar: false,
            require_explicit_model: true,
        },
        NamedLimit {
            id: LimitId::new("grammar"),
            kind: FloorKind::GrammarCoverage,
            description: "grammar-coverage stop".into(),
            max_j: None,
            voi_stop_threshold: None,
            refuse_unknown_grammar: false,
            require_explicit_model: false,
        },
        NamedLimit {
            id: LimitId::new("primitive_gap"),
            kind: FloorKind::PrimitiveGap,
            description: "periodic-stack primitive gap".into(),
            max_j: None,
            voi_stop_threshold: None,
            refuse_unknown_grammar: false,
            require_explicit_model: false,
        },
        NamedLimit {
            id: LimitId::new("efa_certificate"),
            kind: FloorKind::EfaCertificate,
            description: "EFA-style energy-as-certificate refuse".into(),
            max_j: None,
            voi_stop_threshold: None,
            refuse_unknown_grammar: false,
            require_explicit_model: false,
        },
        NamedLimit {
            id: LimitId::new("settle_refuse"),
            kind: FloorKind::SettleRefuse,
            description: "Klere-style settle refuse".into(),
            max_j: None,
            voi_stop_threshold: None,
            refuse_unknown_grammar: false,
            require_explicit_model: false,
        },
        NamedLimit {
            id: LimitId::new("wca_refuse"),
            kind: FloorKind::WcaRefuse,
            description: "WCA refuse surface (wired via mol-automate)".into(),
            max_j: None,
            voi_stop_threshold: None,
            refuse_unknown_grammar: false,
            require_explicit_model: false,
        },
        NamedLimit {
            id: LimitId::new("information"),
            kind: FloorKind::Information,
            description: "information floor — more bits stop buying outcomes".into(),
            max_j: None,
            voi_stop_threshold: None,
            refuse_unknown_grammar: false,
            require_explicit_model: false,
        },
        NamedLimit {
            id: LimitId::new("encapsulation"),
            kind: FloorKind::Encapsulation,
            description: "WASM / capsule encapsulation boundary".into(),
            max_j: None,
            voi_stop_threshold: None,
            refuse_unknown_grammar: false,
            require_explicit_model: false,
        },
        NamedLimit {
            id: LimitId::new("energy_honesty"),
            kind: FloorKind::EnergyHonesty,
            description: "energy honesty — measured vs estimated vs modeled".into(),
            max_j: None,
            voi_stop_threshold: None,
            refuse_unknown_grammar: false,
            require_explicit_model: false,
        },
        NamedLimit {
            id: LimitId::new("provenance_missing"),
            kind: FloorKind::ProvenanceMissing,
            description: "fail-closed when required provenance / meter missing".into(),
            max_j: None,
            voi_stop_threshold: None,
            refuse_unknown_grammar: false,
            require_explicit_model: false,
        },
        NamedLimit {
            id: LimitId::new("agent_isolation"),
            kind: FloorKind::AgentIsolation,
            description: "Agent Lane isolation — no shared cookies/profile with opaque bots".into(),
            max_j: None,
            voi_stop_threshold: None,
            refuse_unknown_grammar: false,
            require_explicit_model: false,
        },
    ]
}
