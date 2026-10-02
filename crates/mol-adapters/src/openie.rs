//! Adapter surface for `openie-runtime` (Z1→Z2→Z3 + Receipt).
//!
//! Default: offline zone map (no leapfrog compile). Optional `openie-path`
//! feature is reserved; live ask stays AdapterStub until a light path-dep compiles.

use mol_core::{CascadeTier, MolError, MolRequest, OpenIeZone, QueryKind, Result};
use mol_receipt::MolReceipt;
use serde::{Deserialize, Serialize};

/// Port toward openie-leapfrog runtime ask.
pub trait OpenIeRuntimePort: Send + Sync {
    /// Ask the OpenIE engine (offline map by default; live path deferred).
    fn ask(&self, req: &MolRequest) -> Result<OpenIeAskResult>;
}

/// Stub result mirroring OpenIE zones.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenIeAskResult {
    /// Zone that answered / would answer.
    pub zone: OpenIeZone,
    /// Mapped MoL cascade tier.
    pub mol_tier: CascadeTier,
    /// Answer text (offline note or live answer).
    pub answer: String,
    /// Optional MoL-shaped receipt bridge.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub receipt: Option<MolReceipt>,
    /// Integration note.
    pub note: String,
    /// True when this is offline map only (no leapfrog call).
    pub offline: bool,
}

/// Default stub — offline zone affinity; does not call leapfrog.
#[derive(Debug, Default, Clone)]
pub struct StubOpenIeRuntime;

impl StubOpenIeRuntime {
    /// Map MoL query kind → OpenIE zone + cascade tier (pattern from leapfrog).
    pub fn map_zone(kind: QueryKind) -> (OpenIeZone, CascadeTier) {
        match kind {
            QueryKind::UnitConvert | QueryKind::StackNavigate => {
                (OpenIeZone::Z1, CascadeTier::Lookup)
            }
            QueryKind::ClosedFormPhysics => (OpenIeZone::Z1, CascadeTier::Formula),
            QueryKind::LinearSolve | QueryKind::Settle => (OpenIeZone::Z2, CascadeTier::Solver),
            QueryKind::PrimitiveGap => (OpenIeZone::Z2, CascadeTier::Solver),
            QueryKind::FactualClaim => (OpenIeZone::Z2, CascadeTier::Lookup),
            QueryKind::Compose => (OpenIeZone::Z1, CascadeTier::Lookup),
            QueryKind::MemoryWrite => (OpenIeZone::Z1, CascadeTier::Lookup),
            QueryKind::MemoryRecall => (OpenIeZone::Z2, CascadeTier::Lookup),
            QueryKind::TicketClose => (OpenIeZone::Z1, CascadeTier::Lookup),
            QueryKind::FreeForm => (OpenIeZone::Z3, CascadeTier::Model),
        }
    }
}

impl OpenIeRuntimePort for StubOpenIeRuntime {
    fn ask(&self, req: &MolRequest) -> Result<OpenIeAskResult> {
        let (zone, mol_tier) = Self::map_zone(req.kind);
        Ok(OpenIeAskResult {
            zone,
            mol_tier,
            answer: format!(
                "(openie offline map) kind={:?} → {} / {}; MoL cascade owns close",
                req.kind,
                zone,
                mol_tier
            ),
            receipt: None,
            note: "offline zone map only; openie-leapfrog path-dep not linked (see BLUEPRINT §path to MoL)"
                .into(),
            offline: true,
        })
    }
}

/// Explicit live ask stub (returns AdapterStub).
pub fn live_ask_stub(_req: &MolRequest) -> Result<OpenIeAskResult> {
    Err(MolError::AdapterStub(
        "openie-runtime live ask not wired (enable future openie-path; sibling tree is heavy)"
            .into(),
    ))
}
