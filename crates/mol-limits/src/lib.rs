//! Named limits registry + [`MixtureOfLimits`] router that owns close.
//!
//! Evaluate limits in order; stop at the first binding floor; emit a receipt
//! naming which limit fired. Cascade answers only when no floor binds first.
//! [`MixtureOfLimits::close`] runs propose → certify → commit|refuse → receipt.

#![deny(missing_docs)]

mod registry;
mod router;

pub use registry::{default_registry, LimitCheck, LimitEval, NamedLimit};
pub use router::{CloseOutcome, MixtureOfLimits, MolOutcome};

// Re-export stack navigator used by primitive_gap probes.
pub use mol_core::{
    CellStatus, PeriodicStack, ProbeResult, StackFamily, StackPrimitive,
    FULL_TARGET_FAMILIES, FULL_TARGET_PRIMITIVES,
};

/// Crate version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
