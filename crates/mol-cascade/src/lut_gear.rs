//! Lookup gears: O(1) resolution LUT + composite (LUT then UnitLookup).
//!
//! A1: grammar/Bloom hit → cascade closes at Lookup; model never invoked.

use mol_core::{CascadeTier, MolError, MolRequest, QueryKind, Result};

use crate::bloom::ResolutionLut;
use crate::grammar::{GrammarCoverage, TierAnswer};
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

/// Composite Lookup: TicketResolution LUT first, then UnitLookup (units/stack).
///
/// Ensures O(1) grammar hits never fall through to model.
#[derive(Debug, Clone)]
pub struct CompositeLookup {
    /// Resolution / band LUT.
    pub ticket: TicketResolutionLookup,
    /// Unit / stack navigator.
    pub units: UnitLookup,
}

impl Default for CompositeLookup {
    fn default() -> Self {
        Self {
            ticket: TicketResolutionLookup::default(),
            units: UnitLookup,
        }
    }
}

impl GrammarCoverage for CompositeLookup {
    fn tier(&self) -> CascadeTier {
        CascadeTier::Lookup
    }

    fn covers(&self, req: &MolRequest) -> bool {
        self.ticket.covers(req) || self.units.covers(req)
    }

    fn try_answer(&self, req: &MolRequest) -> Result<TierAnswer> {
        if self.ticket.covers(req) {
            match self.ticket.try_answer(req) {
                Ok(a) => return Ok(a),
                Err(MolError::NotCovered(_)) => {}
                Err(e) => return Err(e),
            }
        }
        self.units.try_answer(req)
    }
}
