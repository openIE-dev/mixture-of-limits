//! WASM / capsule encapsulation boundaries — law for fully encapsulated compute.
//!
//! Borrowed pattern (not clone) from Lux `lux-wasm-runtime` (WASI Preview 2 /
//! Component Model) and Nova Agent Lane isolation: opaque frontier-vendor bot
//! loops that share host state with human sessions are an anti-pattern.
//!
//! MoL never claims a live Wasmtime host here — soft-ref proves sealed vs
//! unsealed capsule contexts offline.

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::floor::{Floor, FloorKind};

/// Stable capsule / component identity (content-address or session id).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CapsuleId(pub String);

impl CapsuleId {
    /// Construct.
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// As str.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CapsuleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Where compute is encapsulated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapsuleBoundary {
    /// In-process native host (no WASM boundary).
    NativeHost,
    /// WASM Component Model (WASI Preview 2) — Lux `lux-wasm-runtime` shape.
    WasmComponent,
    /// Classic wasm32 module (e.g. lux-engine scroll WASM).
    WasmModule,
    /// OS process / sandbox-exec style boundary.
    ProcessSandbox,
    /// Agent Lane–class isolated profile (Nova pattern).
    AgentLane,
}

impl CapsuleBoundary {
    /// Wire label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::NativeHost => "native_host",
            Self::WasmComponent => "wasm_component",
            Self::WasmModule => "wasm_module",
            Self::ProcessSandbox => "process_sandbox",
            Self::AgentLane => "agent_lane",
        }
    }

    /// True when this boundary is a real encapsulation surface (not bare host).
    pub const fn is_encapsulated(self) -> bool {
        !matches!(self, Self::NativeHost)
    }
}

impl fmt::Display for CapsuleBoundary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// Capsule context attached to a MoL request / receipt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapsuleContext {
    /// Capsule id.
    pub id: CapsuleId,
    /// Boundary kind.
    pub boundary: CapsuleBoundary,
    /// True when imports/exports are sealed (no host escape / no shared opaque bot state).
    pub sealed: bool,
    /// Anti-pattern flag: capsule shares cookies/profile/storage with human or opaque bot.
    #[serde(default)]
    pub shares_host_session: bool,
}

impl CapsuleContext {
    /// Sealed WASM component capsule (happy path).
    pub fn sealed_wasm_component(id: impl Into<String>) -> Self {
        Self {
            id: CapsuleId::new(id),
            boundary: CapsuleBoundary::WasmComponent,
            sealed: true,
            shares_host_session: false,
        }
    }

    /// Unsealed / leaky capsule (must refuse under fail-closed encapsulation).
    pub fn leaky_native(id: impl Into<String>) -> Self {
        Self {
            id: CapsuleId::new(id),
            boundary: CapsuleBoundary::NativeHost,
            sealed: false,
            shares_host_session: true,
        }
    }

    /// Encapsulation law check — returns binding floor when violated.
    pub fn check(&self, require_sealed: bool) -> Result<(), Floor> {
        // Anti-pattern: any capsule that shares host session with opaque bots / human lane.
        if self.shares_host_session {
            return Err(Floor::new(
                "encapsulation_host_share",
                FloorKind::Encapsulation,
                format!(
                    "capsule '{}' boundary={} shares host session — opaque bot / shared-storage pattern refused",
                    self.id, self.boundary
                ),
            ));
        }
        if require_sealed && !self.sealed {
            return Err(Floor::new(
                "encapsulation_unsealed",
                FloorKind::Encapsulation,
                format!(
                    "capsule '{}' boundary={} is not sealed; refuse opaque host escape",
                    self.id, self.boundary
                ),
            ));
        }
        if require_sealed && !self.boundary.is_encapsulated() {
            return Err(Floor::new(
                "encapsulation_not_encapsulated",
                FloorKind::Encapsulation,
                format!(
                    "capsule '{}' boundary={} is not an encapsulation surface",
                    self.id, self.boundary
                ),
            ));
        }
        Ok(())
    }
}

/// Receipt projection of encapsulation state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncapsulationReceipt {
    /// Capsule id.
    pub capsule_id: String,
    /// Boundary label.
    pub boundary: CapsuleBoundary,
    /// Sealed.
    pub sealed: bool,
    /// Shared host session (anti-pattern when true for agents).
    pub shares_host_session: bool,
}

impl From<&CapsuleContext> for EncapsulationReceipt {
    fn from(c: &CapsuleContext) -> Self {
        Self {
            capsule_id: c.id.as_str().to_string(),
            boundary: c.boundary,
            sealed: c.sealed,
            shares_host_session: c.shares_host_session,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sealed_wasm_passes() {
        let c = CapsuleContext::sealed_wasm_component("lux:runtime@0.1.0");
        assert!(c.check(true).is_ok());
        assert!(c.boundary.is_encapsulated());
    }

    #[test]
    fn host_share_refused() {
        let c = CapsuleContext::leaky_native("opaque-bot");
        let err = c.check(false).unwrap_err();
        assert_eq!(err.kind, FloorKind::Encapsulation);
        assert_eq!(err.id.as_str(), "encapsulation_host_share");
    }

    #[test]
    fn unsealed_refused_when_required() {
        let mut c = CapsuleContext::sealed_wasm_component("x");
        c.sealed = false;
        let err = c.check(true).unwrap_err();
        assert_eq!(err.id.as_str(), "encapsulation_unsealed");
    }
}
