//! Lookup gears: O(1) resolution LUT + composite (LUT then UnitLookup).
//!
//! A1: grammar/Bloom hit → cascade closes at Lookup; model never invoked.

use mol_core::{CascadeTier, MolError, MolRequest, QueryKind, Result};

use crate::bloom::ResolutionLut;
use crate::distill::DistillStore;
use crate::grammar::{GrammarCoverage, TierAnswer};
use crate::catalog_gear::CatalogLookup;
use crate::tiers::UnitLookup;

/// Support-desk / risk-band O(1) Bloom+HashMap Lookup gear.
#[derive(Debug, Clone)]
pub struct TicketResolutionLookup {
    /// Exact LUT with Bloom prefilter.
    pub lut: ResolutionLut,
}

impl Default for TicketResolutionLookup {
    fn default() -> Self {
        Self {
            lut: ResolutionLut::support_desk_demo(),
        }
    }
}

impl TicketResolutionLookup {
    /// Support-desk demo rows.
    pub fn support_desk() -> Self {
        Self::default()
    }

    /// Risk-score sketch rows.
    pub fn risk_score() -> Self {
        Self {
            lut: ResolutionLut::risk_score_demo(),
        }
    }
}

impl GrammarCoverage for TicketResolutionLookup {
    fn tier(&self) -> CascadeTier {
        CascadeTier::Lookup
    }

    fn covers(&self, req: &MolRequest) -> bool {
        matches!(req.kind, QueryKind::TicketClose)
            || self.lut.lookup_in_query(&req.query).is_some()
    }

    fn try_answer(&self, req: &MolRequest) -> Result<TierAnswer> {
        match self.lut.lookup_in_query(&req.query) {
            Some((code, ans)) => Ok(TierAnswer::text(format!(
                "{ans} [lut_code={code}; O(1) Bloom+HashMap; model never invoked]"
            ))),
            None => Err(MolError::NotCovered(format!(
                "ticket/resolution LUT miss: {}",
                req.query
            ))),
        }
    }
}

/// Composite Lookup: distilled primitives → TicketResolution LUT → UnitLookup.
///
/// Ensures O(1) grammar hits never fall through to model. After Primitive
/// Distillation, a **second pass** matching a distilled pattern closes here
/// without opening Model LAST.
#[derive(Debug, Clone)]
pub struct CompositeLookup {
    /// Resolution / band LUT.
    pub ticket: TicketResolutionLookup,
    /// Unit / stack navigator.
    pub units: UnitLookup,
    /// Live Periodic Stack Lookup catalog.
    pub catalog: CatalogLookup,
    /// Optional certified Model LAST → Lookup distill registry.
    pub distilled: Option<DistillStore>,
}

impl Default for CompositeLookup {
    fn default() -> Self {
        Self {
            ticket: TicketResolutionLookup::default(),
            units: UnitLookup,
            catalog: CatalogLookup,
            distilled: None,
        }
    }
}

impl CompositeLookup {
    /// Attach a distill store (Lookup gear entries only).
    pub fn with_distill_store(mut self, store: DistillStore) -> Self {
        self.distilled = Some(store);
        self
    }
}

impl GrammarCoverage for CompositeLookup {
    fn tier(&self) -> CascadeTier {
        CascadeTier::Lookup
    }

    fn covers(&self, req: &MolRequest) -> bool {
        if let Some(store) = &self.distilled {
            if let Some(e) = store.match_query(&req.query) {
                if e.gear.eq_ignore_ascii_case("lookup") {
                    return true;
                }
            }
        }
        self.ticket.covers(req) || self.catalog.covers(req) || self.units.covers(req)
    }

    fn try_answer(&self, req: &MolRequest) -> Result<TierAnswer> {
        if let Some(store) = &self.distilled {
            if let Some(e) = store.match_query(&req.query) {
                if e.gear.eq_ignore_ascii_case("lookup") {
                    return Ok(TierAnswer::text(format!(
                        "{} [distilled_lookup id={}; pattern={}; model never invoked on second pass]",
                        e.body, e.id, e.pattern
                    )));
                }
            }
        }
        if self.ticket.covers(req) {
            match self.ticket.try_answer(req) {
                Ok(a) => return Ok(a),
                Err(MolError::NotCovered(_)) => {}
                Err(e) => return Err(e),
            }
        }
        if self.catalog.covers(req) {
            match self.catalog.try_answer(req) {
                Ok(a) => return Ok(a),
                Err(MolError::NotCovered(_)) => {}
                Err(e) => return Err(e),
            }
        }
        self.units.try_answer(req)
    }
}


/// Formula gear overlay: distilled certified Model LAST → Formula identity.
///
/// Soft-ref: pattern match only (no symbolic algebra). Used ahead of [`FormulaTier`]
/// so second-pass closes without model.
#[derive(Debug, Clone, Default)]
pub struct DistilledFormula {
    /// Optional distill registry (formula gear entries).
    pub distilled: Option<DistillStore>,
}

impl DistilledFormula {
    /// Empty overlay.
    pub fn new() -> Self {
        Self::default()
    }

    /// Attach distill store.
    pub fn with_distill_store(mut self, store: DistillStore) -> Self {
        self.distilled = Some(store);
        self
    }
}

impl GrammarCoverage for DistilledFormula {
    fn tier(&self) -> CascadeTier {
        CascadeTier::Formula
    }

    fn covers(&self, req: &MolRequest) -> bool {
        self.distilled
            .as_ref()
            .and_then(|s| s.match_query(&req.query))
            .is_some_and(|e| e.gear.eq_ignore_ascii_case("formula"))
    }

    fn try_answer(&self, req: &MolRequest) -> Result<TierAnswer> {
        match self
            .distilled
            .as_ref()
            .and_then(|s| s.match_query(&req.query))
            .filter(|e| e.gear.eq_ignore_ascii_case("formula"))
        {
            Some(e) => Ok(TierAnswer::text(format!(
                "{} [distilled_formula id={}; pattern={}; model never invoked on second pass]",
                e.body, e.id, e.pattern
            ))),
            None => Err(MolError::NotCovered(format!(
                "distilled formula miss: {}",
                req.query
            ))),
        }
    }
}
