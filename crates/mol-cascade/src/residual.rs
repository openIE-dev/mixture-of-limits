//! Residual Model LAST leaf adapter.
//!
//! Opens only under VoI>0 + `allow_model` + budget headroom. Proposals are
//! always `ModelGenerated`. Uncertified proposals never commit (close gate).

use mol_core::{
    Budget, MolError, MolRequest, QueryKind, Result, ValueOfInformation, VoiDecision,
};

use crate::tiers::ModelStub;

/// Residual Model LAST adapter — labeled generative leaf, no weights.
///
/// Gate order (caller + this leaf):
/// 1. `budget.allow_model`
/// 2. VoI > 0 (escalate)
/// 3. budget joule headroom (enforced by cascade engine)
/// 4. NI certify-before-commit (enforced by MixtureOfLimits::close)
#[derive(Debug, Default, Clone)]
pub struct ResidualModelAdapter {
    /// When true, free-form residual asks produce a proposal (still ModelGenerated).
    pub propose_free_form: bool,
}

impl ResidualModelAdapter {
    /// Default residual leaf (propose on residual markers + optional free-form).
    pub fn new() -> Self {
        Self {
            propose_free_form: true,
        }
    }

    /// Assess VoI for opening the model leaf.
    pub fn voi_for(req: &MolRequest) -> ValueOfInformation {
        match req.kind {
            QueryKind::FreeForm => {
                // Free-form without grammar: positive VoI only when allow_model
                // (otherwise registry already refused). Marginal utility modest.
                if req.budget.allow_model {
                    ValueOfInformation::new(
                        0.4,
                        1e-4,
                        0.05,
                        "residual free-form: VoI>0 under allow_model; Model LAST may propose",
                    )
                } else {
                    ValueOfInformation::new(
                        0.0,
                        1e-4,
                        0.05,
                        "free-form without allow_model: VoI≈0",
                    )
                }
            }
            QueryKind::TicketClose
            | QueryKind::UnitConvert
            | QueryKind::ClosedFormPhysics
            | QueryKind::LinearSolve
            | QueryKind::Settle
            | QueryKind::StackNavigate
            | QueryKind::FactualClaim
            | QueryKind::Compose
            | QueryKind::MemoryWrite
            | QueryKind::MemoryRecall => ValueOfInformation::new(
                0.0,
                1e-9,
                0.05,
                "grammar-covered coordinate: VoI≈0 for model escalation",
            ),
            QueryKind::PrimitiveGap => ValueOfInformation::new(
                0.0,
                1e-9,
                0.05,
                "primitive_gap: refuse invent; VoI≈0 for hallucinated coverage",
            ),
        }
    }

    /// True when this adapter may generate under VoI + budget policy.
    pub fn may_generate(req: &MolRequest) -> bool {
        if !req.budget.allow_model {
            return false;
        }
        let voi = Self::voi_for(req);
        voi.decide() == VoiDecision::Escalate
    }
}

impl ModelStub for ResidualModelAdapter {
    fn generate(&self, req: &MolRequest) -> Result<String> {
        if !req.budget.allow_model {
            return Err(MolError::ModelRefused(
                "Residual Model LAST refused: allow_model=false".into(),
            ));
        }
        let voi = Self::voi_for(req);
        if voi.decide() != VoiDecision::Escalate {
            return Err(MolError::ModelRefused(format!(
                "Residual Model LAST refused: VoI stop ({})",
                voi.rationale
            )));
        }

        let q = req.query.to_ascii_lowercase();
        // Explicit residual / model_fallback markers (product A4 path).
        if q.contains("residual propose")
            || q.contains("model propose")
            || q.contains("model_fallback")
            || q.contains("uncertified")
        {
            return Ok(format!(
                "MODEL_GENERATED_PROPOSAL (residual adapter; no weights): residual for '{}'; commit requires NI/EFA certificate — never launders to Deterministic; voi_mu={:.3}",
                req.query, voi.marginal_utility
            ));
        }

        if self.propose_free_form && matches!(req.kind, QueryKind::FreeForm) {
            return Ok(format!(
                "MODEL_GENERATED_PROPOSAL (residual adapter; free-form under VoI>0+budget): '{}'; NI cert required to commit; voi_mu={:.3}",
                req.query, voi.marginal_utility
            ));
        }

        Err(MolError::ModelRefused(format!(
            "Residual Model LAST: no residual pattern for '{}'",
            req.query
        )))
    }
}

/// Budget helper: coin-cell never opens model; demo+allow_model may.
pub fn residual_budget_ok(budget: &Budget, spent_j: f64, model_cost_j: f64) -> bool {
    budget.allow_model && spent_j + model_cost_j <= budget.max_j.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use mol_core::Budget;

    #[test]
    fn refuses_without_allow_model() {
        let a = ResidualModelAdapter::new();
        let req = MolRequest::new("residual propose x", Budget::coin_cell());
        assert!(a.generate(&req).is_err());
    }

    #[test]
    fn proposes_when_voi_and_allow() {
        let a = ResidualModelAdapter::new();
        let mut b = Budget::demo().allow_model();
        b.max_j = mol_core::Joules::new(1.0);
        let req = MolRequest::new("residual propose ticket summary", b);
        let s = a.generate(&req).unwrap();
        assert!(s.contains("MODEL_GENERATED_PROPOSAL"));
    }

    #[test]
    fn grammar_covered_voi_zero() {
        let req = MolRequest::new("convert 100 celsius to fahrenheit", Budget::demo().allow_model());
        assert!(!ResidualModelAdapter::may_generate(&req));
    }
}
