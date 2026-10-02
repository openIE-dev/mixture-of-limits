//! Cascade engine: escalate cheapest sufficient; model last / refused by default.

use std::time::Instant;

use mol_core::{
    schedule_fabric, CascadeTier, EstimateKind, FabricInventory, Floor, FloorKind, Joules,
    MeasureSource, MolError, MolRequest, MuCatalog, MuSource, ReplayClass, Result,
    ScheduleDecision, DEFAULT_THETA_BITS,
};
use mol_receipt::{
    CascadeStepOutcome, CascadeStepRecord, MolReceipt, ReceiptBuilder,
};

use crate::grammar::{GrammarCoverage, TierAnswer};
use crate::lut_gear::CompositeLookup;
use crate::tiers::{
    ClaimCompose, ClaimRetrieve, FormulaTier, LinearSolver, ModelStub, StubModelEndpoint,
    TernarySettle,
};

/// Composite Solver gear: linear 2×2 then ternary settle.
#[derive(Debug, Default)]
struct SolverGear {
    linear: LinearSolver,
    settle: TernarySettle,
}

impl GrammarCoverage for SolverGear {
    fn tier(&self) -> CascadeTier {
        CascadeTier::Solver
    }

    fn covers(&self, req: &MolRequest) -> bool {
        self.linear.covers(req) || self.settle.covers(req)
    }

    fn try_answer(&self, req: &MolRequest) -> Result<TierAnswer> {
        // Prefer linear when it covers; else settle; else NotCovered.
        if self.linear.covers(req) {
            match self.linear.try_answer(req) {
                Ok(a) => return Ok(a),
                Err(MolError::NotCovered(_)) => {}
                Err(e) => return Err(e),
            }
        }
        if self.settle.covers(req) {
            return self.settle.try_answer(req);
        }
        Err(MolError::NotCovered(format!("solver miss: {}", req.query)))
    }
}

/// Result of running the cascade.
#[derive(Debug, Clone)]
pub struct CascadeResult {
    /// Answer text when closed.
    pub answer: Option<String>,
    /// Receipt.
    pub receipt: MolReceipt,
    /// True if cascade closed with an answer.
    pub closed: bool,
}

/// Cascade engine with pluggable gears.
pub struct CascadeEngine {
    lookup: Box<dyn GrammarCoverage>,
    formula: Box<dyn GrammarCoverage>,
    /// Z2 retrieve+cite (claims corpus).
    retrieve: Box<dyn GrammarCoverage>,
    /// Z1 compose/synthesis from ≥2 cited claims (after retrieve).
    compose: Box<dyn GrammarCoverage>,
    solver: Box<dyn GrammarCoverage>,
    model: Box<dyn ModelStub>,
    /// Available-device inventory (soft-ref: Cpu always).
    pub fabric: FabricInventory,
}

impl Default for CascadeEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl CascadeEngine {
    /// Default demo gears (Lookup → Formula → Solver/settle → Model LAST).
    pub fn new() -> Self {
        Self {
            lookup: Box::new(CompositeLookup::default()),
            formula: Box::new(FormulaTier),
            retrieve: Box::new(ClaimRetrieve::default()),
            compose: Box::new(ClaimCompose::default()),
            solver: Box::new(SolverGear::default()),
            model: Box::new(StubModelEndpoint),
            fabric: FabricInventory::software_ref(),
        }
    }

    /// With custom fabric inventory (tests / optional GPU / thermo flags).
    pub fn with_fabric(mut self, fabric: FabricInventory) -> Self {
        self.fabric = fabric;
        self
    }

    /// Run MathGround cascade under request budget.
    pub fn run(&self, req: &MolRequest) -> Result<CascadeResult> {
        let mut steps = Vec::new();
        let mut spent = Joules::ZERO;
        // Lookup → Formula → Z2 retrieve+cite → Z1 compose → Solver. Model last (below).
        // Gear flags: (is_retrieve, is_compose)
        let gears: [(&dyn GrammarCoverage, CascadeTier, bool, bool); 5] = [
            (self.lookup.as_ref(), CascadeTier::Lookup, false, false),
            (self.formula.as_ref(), CascadeTier::Formula, false, false),
            (self.retrieve.as_ref(), CascadeTier::Lookup, true, false), // priced Lookup; Z2 cite meta
            (self.compose.as_ref(), CascadeTier::Lookup, false, true), // priced Lookup; Z1 Composed
            (self.solver.as_ref(), CascadeTier::Solver, false, false),
        ];

        for (gear, tier, is_retrieve, is_compose) in gears {
            let est = MuCatalog::estimate_tier(tier);
            let cost = est.estimated_j;
            if spent.saturating_add(cost).0 > req.budget.max_j.0 {
                let floor = Floor::new(
                    "energy_budget",
                    FloorKind::Energy,
                    format!(
                        "estimated spend {} + tier {} would exceed max_j {}",
                        spent, cost, req.budget.max_j
                    ),
                )
                .with_joules(req.budget.max_j);
                let receipt = self.refuse_receipt(req, steps, spent, floor, "energy budget floor");
                return Ok(CascadeResult {
                    answer: None,
                    receipt,
                    closed: false,
                });
            }

            if !gear.covers(req) {
                steps.push(CascadeStepRecord {
                    tier,
                    us: 0,
                    outcome: CascadeStepOutcome::Miss,
                    estimated_j: Joules::ZERO,
                });
                continue;
            }

            let t0 = Instant::now();
            match gear.try_answer(req) {
                Ok(ans) => {
                    let us = t0.elapsed().as_micros() as u64;
                    spent = spent.saturating_add(cost);
                    steps.push(CascadeStepRecord {
                        tier,
                        us,
                        outcome: CascadeStepOutcome::Answered,
                        estimated_j: cost,
                    });
                    let zone = ans
                        .zone_override
                        .unwrap_or_else(|| tier.openie_zone());
                    let replay = ans
                        .replay_override
                        .unwrap_or_else(|| tier.default_replay());
                    let rationale = if is_compose || replay == ReplayClass::Composed {
                        format!(
                            "cascade closed at Z1 compose/synthesis (tier={tier}, zone={zone}, replay={replay}); composed_from={:?}; cites={:?}; never laundered as Deterministic/RetrievedCited alone; E≈θ·μ catalog μ={:.3e}; model never invoked",
                            ans.composed_from, ans.citation_ids, est.mu
                        )
                    } else if is_retrieve || replay == ReplayClass::RetrievedCited {
                        format!(
                            "cascade closed at Z2 retrieve+cite (tier={tier}, zone={zone}, replay={replay}); cites={:?}; E≈θ·μ catalog μ={:.3e}; model never invoked",
                            ans.citation_ids, est.mu
                        )
                    } else {
                        format!(
                            "cascade closed at {tier} (OpenIE {zone}); E≈θ·μ catalog μ={:.3e}; model never invoked",
                            est.mu
                        )
                    };
                    let sched = schedule_fabric(
                        tier,
                        &self.fabric,
                        &req.budget,
                        Joules::new(cost.0 * 0.01),
                    );
                    let route_step = sched.to_compute_step();
                    if let ScheduleDecision::Unavailable { reason, .. } = &sched {
                        let floor = Floor::new(
                            "fabric_unavailable",
                            FloorKind::Energy,
                            reason.clone(),
                        );
                        let receipt = self.refuse_receipt_with_compute(
                            req,
                            steps,
                            spent,
                            floor,
                            &format!("fabric refuse after {tier}: {reason}"),
                            vec![route_step],
                            None,
                        );
                        return Ok(CascadeResult {
                            answer: None,
                            receipt,
                            closed: false,
                        });
                    }
                    let chosen = sched.chosen_kind();
                    let rationale = format!(
                        "{rationale}; fabric_chosen={} inventory=[{}]",
                        chosen.map(|k| k.label()).unwrap_or("none"),
                        self.fabric.summary()
                    );
                    let mut rb = ReceiptBuilder::new()
                        .query(&req.query)
                        .answered(tier)
                        .zone(zone)
                        .replay_class(replay)
                        .answer(&ans.text)
                        .citations(ans.citation_ids.clone())
                        .estimated_j(spent)
                        .estimate_kind(EstimateKind::Analytical)
                        .measure_source(MeasureSource::CatalogSurrogate)
                        .mu(est.mu, MuSource::Catalog)
                        .steps(steps)
                        .budget(req.budget)
                        .landauer_bits(DEFAULT_THETA_BITS)
                        .fabric(chosen, self.fabric.clone())
                        .compute_step(route_step)
                        .rationale(rationale);
                    if is_compose || !ans.composed_from.is_empty() {
                        rb = rb.synthesis_composed(ans.composed_from.clone());
                    }
                    let receipt = rb.build();
                    return Ok(CascadeResult {
                        answer: Some(ans.text),
                        receipt,
                        closed: true,
                    });
                }
                Err(MolError::NotCovered(_)) => {
                    let us = t0.elapsed().as_micros() as u64;
                    let miss_cost = Joules::new(cost.0 * 0.1);
                    spent = spent.saturating_add(miss_cost);
                    steps.push(CascadeStepRecord {
                        tier,
                        us,
                        outcome: CascadeStepOutcome::Miss,
                        estimated_j: miss_cost,
                    });
                }
                Err(MolError::LimitFired { id, reason }) => {
                    let us = t0.elapsed().as_micros() as u64;
                    steps.push(CascadeStepRecord {
                        tier,
                        us,
                        outcome: CascadeStepOutcome::Refused,
                        estimated_j: Joules::ZERO,
                    });
                    let kind = if id == "settle_refuse" {
                        FloorKind::SettleRefuse
                    } else if id == "efa_certificate" {
                        FloorKind::EfaCertificate
                    } else if id == "claim_unknown" || id == "compose_missing" {
                        FloorKind::Information
                    } else {
                        FloorKind::WcaRefuse
                    };
                    let floor = Floor::new(&id, kind, &reason);
                    let receipt = self.refuse_receipt(req, steps, spent, floor, &reason);
                    return Ok(CascadeResult {
                        answer: None,
                        receipt,
                        closed: false,
                    });
                }
                Err(e) => return Err(e),
            }
        }

        // Model leaf — demoted; fabric law: Gpu*|refuse if budget/inventory.
        let model_tier = CascadeTier::Model;
        let model_est = MuCatalog::estimate_tier(model_tier);
        let model_cost = model_est.estimated_j;
        if !req.budget.allow_model {
            steps.push(CascadeStepRecord {
                tier: model_tier,
                us: 0,
                outcome: CascadeStepOutcome::Skipped,
                estimated_j: Joules::ZERO,
            });
            let floor = Floor::new(
                "model_demoted",
                FloorKind::GrammarCoverage,
                "model tier skipped (allow_model=false); residual leaf not substrate",
            );
            let receipt = self.refuse_receipt(
                req,
                steps,
                spent,
                floor,
                "cascade exhausted deterministic gears; model refused by policy",
            );
            return Ok(CascadeResult {
                answer: None,
                receipt,
                closed: false,
            });
        }

        // Fabric routing for model residual BEFORE generate.
        let sched = schedule_fabric(model_tier, &self.fabric, &req.budget, Joules::ZERO);
        let route_step = sched.to_compute_step();
        if let ScheduleDecision::Unavailable { reason, .. } = &sched {
            steps.push(CascadeStepRecord {
                tier: model_tier,
                us: 0,
                outcome: CascadeStepOutcome::Refused,
                estimated_j: Joules::ZERO,
            });
            let floor = Floor::new("fabric_unavailable", FloorKind::Energy, reason.clone());
            let receipt = self.refuse_receipt_with_compute(
                req,
                steps,
                spent,
                floor,
                &format!("model residual fabric refuse: {reason}"),
                vec![route_step],
                None,
            );
            return Ok(CascadeResult {
                answer: None,
                receipt,
                closed: false,
            });
        }
        let model_fabric = sched.chosen_kind();

        if spent.saturating_add(model_cost).0 > req.budget.max_j.0 {
            let floor = Floor::new(
                "energy_budget",
                FloorKind::Energy,
                "model tier would exceed joule budget",
            )
            .with_joules(req.budget.max_j);
            // Fabric was already chosen; keep the stamp on the joule refuse.
            let receipt = self.refuse_receipt_with_compute(
                req,
                steps,
                spent,
                floor,
                &format!(
                    "energy budget before model; fabric_chosen={} (no measured_j)",
                    model_fabric.map(|k| k.label()).unwrap_or("none")
                ),
                vec![route_step],
                model_fabric,
            );
            return Ok(CascadeResult {
                answer: None,
                receipt,
                closed: false,
            });
        }

        let t0 = Instant::now();
        match self.model.generate(req) {
            Ok(answer) => {
                let us = t0.elapsed().as_micros() as u64;
                spent = spent.saturating_add(model_cost);
                steps.push(CascadeStepRecord {
                    tier: model_tier,
                    us,
                    outcome: CascadeStepOutcome::Answered,
                    estimated_j: model_cost,
                });
                let receipt = ReceiptBuilder::new()
                    .query(&req.query)
                    .answered(model_tier)
                    .answer(&answer)
                    .estimated_j(spent)
                    .estimate_kind(EstimateKind::Analytical)
                    .measure_source(MeasureSource::CatalogSurrogate)
                    .mu(model_est.mu, MuSource::Catalog)
                    .steps(steps)
                    .budget(req.budget)
                    .landauer_bits(DEFAULT_THETA_BITS)
                    .fabric(model_fabric, self.fabric.clone())
                        .compute_step(route_step.clone())
                    .rationale(format!(
                        "cascade closed at model leaf (explicitly allowed); fabric_chosen={}; E≈θ·μ catalog μ={:.3e}",
                        model_fabric.map(|k| k.label()).unwrap_or("none"),
                        model_est.mu
                    ))
                    .build();
                Ok(CascadeResult {
                    answer: Some(answer),
                    receipt,
                    closed: true,
                })
            }
            Err(MolError::ModelRefused(msg)) => {
                let us = t0.elapsed().as_micros() as u64;
                steps.push(CascadeStepRecord {
                    tier: model_tier,
                    us,
                    outcome: CascadeStepOutcome::Refused,
                    estimated_j: Joules::ZERO,
                });
                let floor = Floor::new("model_refused", FloorKind::GrammarCoverage, &msg);
                // Fabric was already chosen from inventory. Keep the stamp on the
                // refuse (weights missing ≠ fabric missing). measured_j stays None.
                let receipt = self.refuse_receipt_with_compute(
                    req,
                    steps,
                    spent,
                    floor,
                    &format!(
                        "{msg}; fabric_chosen={} (schedule stamp kept; no measured_j)",
                        model_fabric.map(|k| k.label()).unwrap_or("none")
                    ),
                    vec![route_step],
                    model_fabric,
                );
                Ok(CascadeResult {
                    answer: None,
                    receipt,
                    closed: false,
                })
            }
            Err(e) => Err(e),
        }
    }

    fn refuse_receipt(
        &self,
        req: &MolRequest,
        steps: Vec<CascadeStepRecord>,
        spent: Joules,
        floor: Floor,
        rationale: &str,
    ) -> MolReceipt {
        self.refuse_receipt_with_compute(req, steps, spent, floor, rationale, Vec::new(), None)
    }

    fn refuse_receipt_with_compute(
        &self,
        req: &MolRequest,
        steps: Vec<CascadeStepRecord>,
        spent: Joules,
        floor: Floor,
        rationale: &str,
        compute_steps: Vec<mol_core::ComputeStepReceipt>,
        fabric_chosen: Option<mol_core::DeviceKind>,
    ) -> MolReceipt {
        ReceiptBuilder::new()
            .query(&req.query)
            .estimated_j(spent)
            .estimate_kind(EstimateKind::Analytical)
            .measure_source(MeasureSource::CatalogSurrogate)
            .mu_source(MuSource::Catalog)
            .limit_fired(floor)
            .steps(steps)
            .budget(req.budget)
            .fabric(fabric_chosen, self.fabric.clone())
            .compute_steps(compute_steps)
            .executed(false)
            .rationale(format!(
                "{rationale}; fabric_inventory=[{}]",
                self.fabric.summary()
            ))
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mol_core::Budget;

    #[test]
    fn formula_closes_without_model() {
        let eng = CascadeEngine::new();
        let req = MolRequest::new("landauer joules per bit", Budget::coin_cell());
        let r = eng.run(&req).unwrap();
        assert!(r.closed);
        assert_eq!(r.receipt.cascade_answered, Some(CascadeTier::Formula));
        assert!(r.receipt.limit_fired.is_none());
        assert_eq!(r.receipt.fabric_chosen, Some(mol_core::DeviceKind::Cpu));
        assert!(r.receipt.fabric_inventory.as_ref().unwrap().is_present(mol_core::DeviceKind::Cpu));
    }

    #[test]
    fn freeform_refuses_model() {
        let eng = CascadeEngine::new();
        let req = MolRequest::new("write a poem about transistors", Budget::demo());
        let r = eng.run(&req).unwrap();
        assert!(!r.closed);
        assert!(r.receipt.limit_fired.is_some());
    }

    #[test]
    fn settle_closes_or_refuses() {
        let eng = CascadeEngine::new();
        let ok = eng
            .run(&MolRequest::new(
                "settle ternary [1, 1, 1, 1]",
                Budget::demo(),
            ))
            .unwrap();
        assert!(ok.closed);
        assert_eq!(ok.receipt.cascade_answered, Some(CascadeTier::Solver));

        let no = eng
            .run(&MolRequest::new(
                "settle refuse will not settle [1,1,1]",
                Budget::demo(),
            ))
            .unwrap();
        assert!(!no.closed);
        let floor = no.receipt.limit_fired.as_ref().unwrap();
        assert_eq!(floor.id.as_str(), "settle_refuse");
        assert_eq!(floor.kind, FloorKind::SettleRefuse);
    }
}
