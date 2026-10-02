//! Proposed acts and capabilities.

use serde::{Deserialize, Serialize};

/// Kind of automation act.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActKind {
    /// Read-only sense (always softer).
    Sense,
    /// Propose an answer / plan (no side effect).
    Propose,
    /// Write / mutate (irreversible).
    Mutate,
    /// Shell / network (irreversible, highest).
    External,
}

/// Capability flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    /// Allow sense.
    Sense,
    /// Allow propose.
    Propose,
    /// Allow mutate.
    Mutate,
    /// Allow external.
    External,
}

/// Set of granted capabilities (default empty = deny all irreversible).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilitySet {
    /// Granted caps.
    pub granted: Vec<Capability>,
}

impl CapabilitySet {
    /// Empty (default deny).
    pub fn deny_all() -> Self {
        Self::default()
    }

    /// Sense + propose only (safe demo).
    pub fn sense_propose() -> Self {
        Self {
            granted: vec![Capability::Sense, Capability::Propose],
        }
    }

    /// Check grant.
    pub fn allows(&self, cap: Capability) -> bool {
        self.granted.contains(&cap)
    }

    /// Capability required for an act kind.
    pub fn required(kind: ActKind) -> Capability {
        match kind {
            ActKind::Sense => Capability::Sense,
            ActKind::Propose => Capability::Propose,
            ActKind::Mutate => Capability::Mutate,
            ActKind::External => Capability::External,
        }
    }
}

/// A proposed act.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Act {
    /// Kind.
    pub kind: ActKind,
    /// Human summary.
    pub summary: String,
    /// Estimated joules to execute (surrogate).
    pub estimated_j: f64,
    /// Optional payload (JSON).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload: Option<serde_json::Value>,
}

impl Act {
    /// Construct.
    pub fn new(kind: ActKind, summary: impl Into<String>, estimated_j: f64) -> Self {
        Self {
            kind,
            summary: summary.into(),
            estimated_j,
            payload: None,
        }
    }
}
