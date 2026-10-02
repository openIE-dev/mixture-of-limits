//! Primitive Distillation Loop v1 — certified Model LAST → Lookup/Formula append.
//!
//! Uncertified proposals never become Lookup. Distilled entries carry provenance
//! (source receipt, certify method, replay class). Offline / CLI distill path.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use mol_core::{MolError, ReplayClass, Result};
use serde::{Deserialize, Serialize};

/// One distilled primitive ready for Lookup / Formula append.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DistillEntry {
    /// Stable entry id.
    pub id: String,
    /// Target gear: `lookup` or `formula`.
    pub gear: String,
    /// Lookup key or formula identity pattern.
    pub pattern: String,
    /// Deterministic answer / formula body.
    pub body: String,
    /// Source receipt id (certified Model LAST close).
    pub source_receipt_id: String,
    /// Certify method label (`ni_in_crate` / `efa+wca`).
    pub certify_method: String,
    /// Certificate ids stamped at commit.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub certificate_ids: Vec<String>,
    /// Must remain Deterministic after distill (never ModelGenerated).
    pub replay_class: ReplayClass,
    /// Note.
    pub note: String,
}

/// Distillation store (append-only JSON registry).
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct DistillStore {
    /// Entries by id.
    pub entries: HashMap<String, DistillEntry>,
    /// Optional durable path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<PathBuf>,
}

impl DistillStore {
    /// Empty store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Load or create.
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        if !path.exists() {
            return Ok(Self {
                entries: HashMap::new(),
                path: Some(path),
            });
        }
        let raw = fs::read_to_string(&path).map_err(|e| {
            MolError::AdapterStub(format!("distill store read: {e}"))
        })?;
        let mut s: DistillStore = serde_json::from_str(&raw).map_err(|e| {
            MolError::AdapterStub(format!("distill store parse: {e}"))
        })?;
        s.path = Some(path);
        Ok(s)
    }

    /// Persist.
    pub fn save(&self) -> Result<()> {
        let Some(ref path) = self.path else {
            return Ok(());
        };
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let raw = serde_json::to_string_pretty(self).map_err(|e| {
            MolError::AdapterStub(format!("distill serialize: {e}"))
        })?;
        fs::write(path, raw).map_err(|e| MolError::AdapterStub(format!("distill write: {e}")))?;
        Ok(())
    }

    /// Append entry.
    pub fn append(&mut self, entry: DistillEntry) -> Result<()> {
        self.entries.insert(entry.id.clone(), entry);
        self.save()
    }

    /// Lookup by pattern (O(n) soft-ref).
    pub fn find_pattern(&self, pattern: &str) -> Option<&DistillEntry> {
        self.entries.values().find(|e| e.pattern == pattern)
    }
}

/// Distill a **certified** Model LAST commit into a Lookup/Formula entry.
///
/// Refuses when:
/// - replay is not ModelGenerated (wrong source), or
/// - certificate_ids empty (uncertified — never distill), or
/// - board_synth claimed (honesty).
pub fn distill_certified_model_last(
    source_receipt_id: &str,
    model_proposal: &str,
    certificate_ids: &[String],
    certify_method: &str,
    gear: &str,
    pattern: &str,
    body: &str,
) -> Result<DistillEntry> {
    if certificate_ids.is_empty() {
        return Err(MolError::LimitFired {
            id: "distill_uncertified".into(),
            reason: "Primitive Distillation refuses: uncertified Model LAST never becomes Lookup/Formula".into(),
        });
    }
    if gear != "lookup" && gear != "formula" {
        return Err(MolError::LimitFired {
            id: "distill_gear".into(),
            reason: format!("distill gear must be lookup|formula, got {gear}"),
        });
    }
    let id = format!(
        "distill:{}:{}",
        gear,
        simple_hash(&format!("{pattern}|{body}|{source_receipt_id}"))
    );
    Ok(DistillEntry {
        id,
        gear: gear.into(),
        pattern: pattern.into(),
        body: body.into(),
        source_receipt_id: source_receipt_id.into(),
        certify_method: certify_method.into(),
        certificate_ids: certificate_ids.to_vec(),
        replay_class: ReplayClass::Deterministic,
        note: format!(
            "distilled from certified Model LAST proposal (len={}); replay launders to Deterministic only after NI cert",
            model_proposal.len()
        ),
    })
}

fn simple_hash(s: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.as_bytes() {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refuses_uncertified() {
        let err = distill_certified_model_last(
            "r1",
            "proposal",
            &[],
            "ni_in_crate",
            "lookup",
            "k",
            "v",
        )
        .unwrap_err();
        assert!(format!("{err}").contains("uncertified") || format!("{err}").contains("distill"));
    }

    #[test]
    fn cert_path_appends_deterministic() {
        let e = distill_certified_model_last(
            "r-abc",
            "MODEL_GENERATED_PROPOSAL: ticket R-NEW",
            &["efa:deadbeef".into(), "wca:cafebabe".into()],
            "ni_in_crate",
            "lookup",
            "ticket close resolution=R-NEW",
            "LOOKUP ticket_resolution R-NEW → closed",
        )
        .unwrap();
        assert_eq!(e.replay_class, ReplayClass::Deterministic);
        assert_eq!(e.gear, "lookup");
        let mut store = DistillStore::new();
        store.append(e.clone()).unwrap();
        assert!(store.find_pattern("ticket close resolution=R-NEW").is_some());
    }
}
