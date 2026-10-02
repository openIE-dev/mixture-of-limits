//! Capability-gated automation — propose → certify → Commit|Refuse → receipt.
//!
//! Mirrors WCA / EFA spirit (`wca-lut-edge` / Physical AI BMI):
//! - Default deny capabilities.
//! - Irreversible acts require explicit allow flags.
//! - Certify (EFA + WCA software-reference) before commit.
//! - Refuse over escalate when unsafe.
//! - `board_synth_claimed = false`.
//! - Surrogate joules only unless a real probe is wired later.
//!
//! End-to-end ecosystem certify ([`EcosystemCertify`]): Agent Lane + fabric +
//! WASM capsule + energy honesty + GrantReceipt → one receipt.
//!
//! Thin agent surface (clean-room, no sibling path-deps):
//! - [`AgentMailbox`] — Goal / Message / Act proposals + close transcript
//! - [`AgentLoop`] — sense → classify → cascade/floors → certify → commit|refuse → record
//!   (model demoted; no silent escalate)

#![deny(missing_docs)]

mod act;
mod agent;
mod ecosystem;
mod gate;
mod mailbox;

pub use act::{Act, ActKind, Capability, CapabilitySet};
pub use agent::{
    demo_goal_expects, AgentLoop, AgentRunReport, AgentStepOutcome, DemoGoalExpect,
};
pub use gate::{AutomateGate, AutomateOutcome, CommitDecision, RefuseReason};
pub use ecosystem::{
    run_ecosystem_e2e_certify, EcosystemCertify, EcosystemCertifyConfig,
    EcosystemCertifyOutcome,
};
pub use mailbox::{AgentMailbox, AgentTranscriptEntry, MailEnvelope, MailItem};

/// Crate version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Always false — software reference.
pub const BOARD_SYNTH_CLAIMED: bool = mol_core::BOARD_SYNTH_CLAIMED;
