//! Shared error type for the Mixture of Limits workspace.

use thiserror::Error;

/// Fallible MoL operations.
pub type Result<T> = std::result::Result<T, MolError>;

/// Errors that can surface across MoL crates.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum MolError {
    /// Cascade tier could not cover the query.
    #[error("not covered by grammar: {0}")]
    NotCovered(String),

    /// Model tier refused (default: model demoted; need `allow_model`).
    #[error("model refused: {0}")]
    ModelRefused(String),

    /// A named limit fired and stopped escalation / commit.
    #[error("limit fired ({id}): {reason}")]
    LimitFired {
        /// Limit identifier.
        id: String,
        /// Human-readable reason.
        reason: String,
    },

    /// Budget exhausted (joules or latency).
    #[error("budget exceeded: {0}")]
    BudgetExceeded(String),

    /// Automation policy refused the act.
    #[error("refuse: {0}")]
    Refuse(String),

    /// Replay-class coercion forbidden (e.g. ModelGenerated → Deterministic).
    #[error("replay class coercion forbidden: {0}")]
    ReplayCoercion(String),

    /// Receipt verification failed.
    #[error("receipt verify: {0}")]
    ReceiptVerify(String),

    /// Adapter / integration stub not wired.
    #[error("adapter stub: {0}")]
    AdapterStub(String),

    /// I/O.
    #[error("io: {0}")]
    Io(String),

    /// Generic message.
    #[error("{0}")]
    Msg(String),
}

impl From<std::io::Error> for MolError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e.to_string())
    }
}
