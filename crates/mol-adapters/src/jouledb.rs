//! Adapter surface for JouleDB cascade tiers Lookup→Formula→Extract→Aggregate→Reason.

use mol_core::{CascadeTier, MolError, MolRequest, QueryKind, Result};
use serde::{Deserialize, Serialize};

/// Port toward jouledb energy-metered cascade.
pub trait JouleDbCascadePort: Send + Sync {
    /// Explain which JouleDB-aligned tier would handle the query.
    fn explain(&self, req: &MolRequest) -> Result<JouleDbExplain>;
}

/// Explain result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JouleDbExplain {
    /// Mapped MoL tier.
    pub mol_tier: CascadeTier,
    /// JouleDB tier label.
    pub jouledb_tier: String,
    /// Note.
    pub note: String,
}

/// Default stub with local mapping documentation (no jouledb link).
#[derive(Debug, Default, Clone)]
pub struct StubJouleDbCascade;

impl JouleDbCascadePort for StubJouleDbCascade {
    fn explain(&self, req: &MolRequest) -> Result<JouleDbExplain> {
        let (mol_tier, jdb) = match req.kind {
            QueryKind::UnitConvert | QueryKind::StackNavigate => (CascadeTier::Lookup, "Lookup"),
            QueryKind::ClosedFormPhysics => (CascadeTier::Formula, "Formula"),
            QueryKind::LinearSolve => (CascadeTier::Solver, "Extract/Aggregate"),
            QueryKind::Settle => (CascadeTier::Solver, "Aggregate"),
            QueryKind::PrimitiveGap => (CascadeTier::Solver, "Extract"),
            QueryKind::FactualClaim => (CascadeTier::Lookup, "Lookup/Cite"),
            QueryKind::Compose => (CascadeTier::Lookup, "Lookup/Compose"),
            QueryKind::MemoryWrite | QueryKind::MemoryRecall => (CascadeTier::Lookup, "Lookup"),
            QueryKind::TicketClose => (CascadeTier::Lookup, "Lookup"),
            QueryKind::FreeForm => (CascadeTier::Model, "Reason"),
        };
        Ok(JouleDbExplain {
            mol_tier,
            jouledb_tier: jdb.into(),
            note: format!(
                "offline mapping only (jouledb-path not linked); query kind {:?}",
                req.kind
            ),
        })
    }
}

/// Explicit stub error path when a live call is requested.
pub fn live_query_stub(_req: &MolRequest) -> Result<()> {
    Err(MolError::AdapterStub(
        "jouledb live query not wired in v0.1 (see BLUEPRINT.md §Integration)".into(),
    ))
}
