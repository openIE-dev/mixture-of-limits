//! Durable episode state machine for written completeness C(z).
//!
//! Episodes persist CompletenessSnapshot across closes. When C(z)=1 the
//! episode saturates and further synthesis refuses (economic satiation).

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::completeness::CompletenessSnapshot;
use crate::error::{MolError, Result};

/// Episode lifecycle status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EpisodeStatus {
    /// Open — C(z)<1; further closes allowed under floors.
    #[default]
    Open,
    /// C(z)=1 — satiation stop; further synthesis refuses.
    Satiated,
    /// Terminal refuse (capability / safety); not economic satiation.
    ClosedRefuse,
}

/// One episode's durable state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpisodeState {
    /// Stable episode id.
    pub id: String,
    /// Written completeness snapshot.
    pub completeness: CompletenessSnapshot,
    /// Number of closes observed.
    pub close_count: u32,
    /// Last receipt id stamped on commit/refuse.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_receipt_id: Option<String>,
    /// Lifecycle.
    pub status: EpisodeStatus,
}

impl EpisodeState {
    /// New open episode from a completeness snapshot.
    pub fn open(id: impl Into<String>, completeness: CompletenessSnapshot) -> Self {
        let mut s = Self {
            id: id.into(),
            completeness,
            close_count: 0,
            last_receipt_id: None,
            status: EpisodeStatus::Open,
        };
        s.refresh_satiation();
        s
    }

    /// Refresh status from C(z).
    pub fn refresh_satiation(&mut self) {
        if self.completeness.is_complete() {
            self.status = EpisodeStatus::Satiated;
        } else if self.status == EpisodeStatus::Satiated {
            // Clauses cleared → reopen (rare; allow explicit reset).
            self.status = EpisodeStatus::Open;
        }
    }

    /// True when further synthesis must refuse (economic).
    pub fn must_refuse_synthesis(&self) -> bool {
        matches!(self.status, EpisodeStatus::Satiated) || self.completeness.is_complete()
    }

    /// Record a close; update completeness and receipt id.
    pub fn record_close(
        &mut self,
        completeness: Option<CompletenessSnapshot>,
        receipt_id: impl Into<String>,
    ) {
        if let Some(c) = completeness {
            self.completeness = c;
        }
        self.close_count = self.close_count.saturating_add(1);
        self.last_receipt_id = Some(receipt_id.into());
        self.refresh_satiation();
    }
}

/// Durable store of episodes (in-memory + optional JSON file).
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct EpisodeStore {
    /// Episodes by id.
    pub episodes: HashMap<String, EpisodeState>,
    /// Optional durable path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<PathBuf>,
}

impl EpisodeStore {
    /// Empty in-memory store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Load from JSON path (creates empty if missing).
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        if !path.exists() {
            return Ok(Self {
                episodes: HashMap::new(),
                path: Some(path),
            });
        }
        let raw = fs::read_to_string(&path).map_err(|e| {
            MolError::AdapterStub(format!("episode store read {}: {e}", path.display()))
        })?;
        let mut store: EpisodeStore = serde_json::from_str(&raw).map_err(|e| {
            MolError::AdapterStub(format!("episode store parse {}: {e}", path.display()))
        })?;
        store.path = Some(path);
        Ok(store)
    }

    /// Persist to path when set.
    pub fn save(&self) -> Result<()> {
        let Some(ref path) = self.path else {
            return Ok(());
        };
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let raw = serde_json::to_string_pretty(self).map_err(|e| {
            MolError::AdapterStub(format!("episode store serialize: {e}"))
        })?;
        fs::write(path, raw).map_err(|e| {
            MolError::AdapterStub(format!("episode store write {}: {e}", path.display()))
        })?;
        Ok(())
    }

    /// Get or create an episode.
    pub fn get_or_open(
        &mut self,
        id: impl Into<String>,
        completeness: CompletenessSnapshot,
    ) -> &mut EpisodeState {
        let id = id.into();
        self.episodes
            .entry(id.clone())
            .or_insert_with(|| EpisodeState::open(id, completeness))
    }

    /// Borrow episode if present.
    pub fn get(&self, id: &str) -> Option<&EpisodeState> {
        self.episodes.get(id)
    }

    /// Mutable borrow.
    pub fn get_mut(&mut self, id: &str) -> Option<&mut EpisodeState> {
        self.episodes.get_mut(id)
    }

    /// Record close and optionally save.
    pub fn record_close(
        &mut self,
        id: &str,
        completeness: Option<CompletenessSnapshot>,
        receipt_id: impl Into<String>,
    ) -> Result<&EpisodeState> {
        let ep = self
            .episodes
            .get_mut(id)
            .ok_or_else(|| MolError::AdapterStub(format!("episode '{id}' not found")))?;
        ep.record_close(completeness, receipt_id);
        let status = ep.status;
        let _ = status;
        self.save()?;
        Ok(self.episodes.get(id).expect("just inserted"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::completeness::CompletenessSnapshot;

    #[test]
    fn satiates_when_c1() {
        let mut ep = EpisodeState::open("e1", CompletenessSnapshot::ticket_close(true, true, false));
        assert!(!ep.must_refuse_synthesis());
        ep.record_close(
            Some(CompletenessSnapshot::ticket_close(true, true, true)),
            "r1",
        );
        assert!(ep.must_refuse_synthesis());
        assert_eq!(ep.status, EpisodeStatus::Satiated);
        assert_eq!(ep.close_count, 1);
    }

    #[test]
    fn durable_roundtrip() {
        let dir = std::env::temp_dir().join(format!("mol-ep-{}", std::process::id()));
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("episodes.json");
        let mut store = EpisodeStore::load(&path).unwrap();
        store.get_or_open("ep-a", CompletenessSnapshot::ticket_close(true, true, true));
        store
            .record_close("ep-a", None, "receipt-1")
            .unwrap();
        let loaded = EpisodeStore::load(&path).unwrap();
        let ep = loaded.get("ep-a").unwrap();
        assert!(ep.must_refuse_synthesis());
        assert_eq!(ep.last_receipt_id.as_deref(), Some("receipt-1"));
        let _ = fs::remove_dir_all(&dir);
    }
}
