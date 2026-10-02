//! MoL energy harness desktop shell (`mol-desktop`).
//!
//! **Product stance:** not a chat clone of VS Code / Cursor / Zed / Grok Bot /
//! Claude Code. Dominance axes = joules-per-verified-decision, MoL
//! commit|refuse ownership, multi-fabric inventory, full transaction ledger,
//! machine-protocol-first UI (thin human confirm), energy harness UX.
//!
//! - **Default:** headless [`ShellSession`] API (prove / CI / MCP-class lane).
//! - **Feature `gui`:** egui/eframe native window (Linux + Mac; no Electron).
//!
//! See `docs/mol-desktop.md`.

#![deny(missing_docs)]

mod shell;
mod view;

#[cfg(feature = "gui")]
pub mod gui;

pub use shell::{
    AskOpts, EcosystemCertifyOpts, ShellCloseResult, ShellEcosystemResult, ShellError, ShellResult,
    ShellSession,
};
pub use view::{
    estimate_is_labeled, AgentLaneView, CascadeStepView, ComputeStepView, ConfirmRequest,
    ConfirmResponse, EncapsulationView, FabricView, JouleLedgerEntry, JouleLedgerSummary,
    ReceiptView,
};

/// Crate version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// True when compiled with `gui` feature.
pub const GUI_ENABLED: bool = cfg!(feature = "gui");
