//! Grammar coverage trait — does this tier cover the request?

use mol_core::{CascadeTier, MolRequest, OpenIeZone, ReplayClass, Result};

/// Rich tier answer (text + optional Z2 citation / Z1 compose metadata).
#[derive(Debug, Clone, PartialEq)]
pub struct TierAnswer {
    /// Answer text.
    pub text: String,
    /// Citation claim ids (RetrievedCited / Composed paths).
    pub citation_ids: Vec<String>,
    /// Optional replay-class override (e.g. RetrievedCited / Composed).
    pub replay_override: Option<ReplayClass>,
    /// Optional OpenIE zone override (e.g. Z2 for cite, Z1 for compose).
    pub zone_override: Option<OpenIeZone>,
    /// Source claim ids when composition happened (≥2 → ReplayClass::Composed).
    pub composed_from: Vec<String>,
}

impl TierAnswer {
    /// Text-only answer (default Deterministic path via tier defaults).
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            citation_ids: Vec::new(),
            replay_override: None,
            zone_override: None,
            composed_from: Vec::new(),
        }
    }

    /// Retrieved+cited answer for OpenIE Z2 (single authoritative retrieve — not composition).
    pub fn retrieved_cited(text: impl Into<String>, citation_ids: Vec<String>) -> Self {
        Self {
            text: text.into(),
            citation_ids,
            replay_override: Some(ReplayClass::RetrievedCited),
            zone_override: Some(OpenIeZone::Z2),
            composed_from: Vec::new(),
        }
    }

    /// Z1 compose/synthesis from ≥2 cited claims — always ReplayClass::Composed.
    ///
    /// Never launder as Deterministic or RetrievedCited alone when composition happened.
    pub fn composed(text: impl Into<String>, composed_from: Vec<String>) -> Self {
        debug_assert!(
            composed_from.len() >= 2,
            "compose requires ≥2 source claim ids"
        );
        let citation_ids = composed_from.clone();
        Self {
            text: text.into(),
            citation_ids,
            replay_override: Some(ReplayClass::Composed),
            zone_override: Some(OpenIeZone::Z1),
            composed_from,
        }
    }
}

impl From<String> for TierAnswer {
    fn from(text: String) -> Self {
        Self::text(text)
    }
}

impl From<&str> for TierAnswer {
    fn from(text: &str) -> Self {
        Self::text(text)
    }
}

/// A cascade gear that may cover a request's grammar.
pub trait GrammarCoverage: Send + Sync {
    /// Which tier this gear implements.
    fn tier(&self) -> CascadeTier;

    /// True if this gear's grammar covers the request (cheap pre-check).
    fn covers(&self, req: &MolRequest) -> bool;

    /// Attempt to answer. Return `Err(NotCovered)` on miss so the engine escalates.
    fn try_answer(&self, req: &MolRequest) -> Result<TierAnswer>;
}
