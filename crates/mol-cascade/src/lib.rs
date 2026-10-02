//! MoL cascade engine — MathGround gears with model demoted to residual leaf.
//!
//! ```text
//! Lookup → Formula → Z2 retrieve → Z1 compose → Solver/settle → Stochastic model LAST
//! ```
//!
//! Escalate **only** on grammar miss. Model tier returns [`MolError::ModelRefused`]
//! unless `budget.allow_model`. No trained weights; model is a labeled stub trait.
//! Settle (Klere-style) lives under Solver; refuse-when-it-will-not-settle binds
//! `settle_refuse` rather than inventing an answer.

#![deny(missing_docs)]

mod bloom;
mod engine;
mod grammar;
mod lut_gear;
mod tiers;
mod distill;
mod residual;

pub use engine::{CascadeEngine, CascadeResult};
pub use bloom::{BloomFilter, ResolutionLut};
pub use grammar::{GrammarCoverage, TierAnswer};
pub use lut_gear::{CompositeLookup, TicketResolutionLookup};
pub use tiers::{
    ClaimCompose, ClaimRetrieve, FormulaTier, LinearSolver, LookupTable, ModelStub,
    StubModelEndpoint, TernarySettle, UnitLookup,
};
pub use distill::{distill_certified_model_last, DistillEntry, DistillStore};
pub use residual::{residual_budget_ok, ResidualModelAdapter};

/// Crate version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
