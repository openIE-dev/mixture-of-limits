//! Bitemporal agent memory — valid-time + transaction-time store.
//!
//! Clean-room in-tree store (not a full DB). Writes are **append-only** and are
//! intended to land only after MoL `close` COMMIT (caller/`MixtureOfLimits`
//! enforces that). Reads support as-of queries on both time axes.
//!
//! Honesty: no RAPL; memory facts are labeled estimates / cited recalls only.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Stable memory fact id (`memory:<key>#tx<n>`).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct MemoryId(pub String);

impl MemoryId {
    /// Construct.
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// As str.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for MemoryId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// One bitemporal memory fact (append-only version).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryFact {
    /// Citation id (`memory:key#txN`).
    pub id: MemoryId,
    /// Logical key.
    pub key: String,
    /// Stored value.
    pub value: String,
    /// Valid-time start (application time).
    pub valid_from: DateTime<Utc>,
    /// Valid-time end; `None` = still open / current.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_to: Option<DateTime<Utc>>,
    /// Transaction-time (when the write was committed to the store).
    pub tx_time: DateTime<Utc>,
    /// Monotonic transaction id.
    pub tx_id: u64,
    /// MoL receipt id that authorized the write (when known).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub receipt_id: Option<String>,
}

impl MemoryFact {
    /// Citation marker for receipts / answers.
    pub fn cite(&self) -> String {
        format!("[cite:{}]", self.id.as_str())
    }

    /// Answer line for recall.
    pub fn recalled_answer(&self) -> String {
        format!(
            "recalled {} = {} {} (valid_from={} tx_id={})",
            self.key,
            self.value,
            self.cite(),
            self.valid_from.to_rfc3339(),
            self.tx_id
        )
    }
}

/// Proposed write parsed from a remember/store query (not yet committed).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryWriteProposal {
    /// Key.
    pub key: String,
    /// Value.
    pub value: String,
    /// Optional explicit valid-from (else commit time).
    pub valid_from: Option<DateTime<Utc>>,
    /// Optional explicit valid-to.
    pub valid_to: Option<DateTime<Utc>>,
}

/// In-memory bitemporal store (valid-time + transaction-time).
#[derive(Debug, Clone, Default)]
pub struct BitemporalStore {
    facts: Vec<MemoryFact>,
    next_tx: u64,
}

impl BitemporalStore {
    /// Empty store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of fact versions (including superseded).
    pub fn len(&self) -> usize {
        self.facts.len()
    }

    /// Empty?
    pub fn is_empty(&self) -> bool {
        self.facts.is_empty()
    }

    /// Next transaction id (peek).
    pub fn next_tx_id(&self) -> u64 {
        self.next_tx
    }

    /// All facts (append order).
    pub fn facts(&self) -> &[MemoryFact] {
        &self.facts
    }

    /// Commit a write (append-only). Ends any open valid-time version for `key`
    /// at `valid_from` (or now). Call **only** after MoL close COMMIT.
    pub fn commit_write(
        &mut self,
        key: impl Into<String>,
        value: impl Into<String>,
        valid_from: Option<DateTime<Utc>>,
        valid_to: Option<DateTime<Utc>>,
        receipt_id: Option<String>,
    ) -> MemoryFact {
        let key = key.into();
        let value = value.into();
        let now = Utc::now();
        let valid_from = valid_from.unwrap_or(now);
        let tx_time = now;
        let tx_id = self.next_tx;
        self.next_tx += 1;

        // Close open versions for this key whose valid interval overlaps start.
        for f in &mut self.facts {
            if f.key == key && f.valid_to.is_none() && f.valid_from <= valid_from {
                f.valid_to = Some(valid_from);
            }
        }

        let id = MemoryId::new(format!("memory:{key}#tx{tx_id}"));
        let fact = MemoryFact {
            id,
            key,
            value,
            valid_from,
            valid_to,
            tx_time,
            tx_id,
            receipt_id,
        };
        self.facts.push(fact.clone());
        fact
    }

    /// Commit from a parsed proposal.
    pub fn commit_proposal(
        &mut self,
        proposal: &MemoryWriteProposal,
        receipt_id: Option<String>,
    ) -> MemoryFact {
        self.commit_write(
            proposal.key.clone(),
            proposal.value.clone(),
            proposal.valid_from,
            proposal.valid_to,
            receipt_id,
        )
    }

    /// Bitemporal recall: fact for `key` valid at `as_of_valid` and known as of
    /// transaction time `as_of_tx` (both default to now when `None`).
    pub fn recall(
        &self,
        key: &str,
        as_of_valid: Option<DateTime<Utc>>,
        as_of_tx: Option<DateTime<Utc>>,
    ) -> Option<&MemoryFact> {
        let now = Utc::now();
        let as_of_valid = as_of_valid.unwrap_or(now);
        let as_of_tx = as_of_tx.unwrap_or(now);
        self.facts
            .iter()
            .rev()
            .find(|f| {
                f.key == key
                    && f.tx_time <= as_of_tx
                    && f.valid_from <= as_of_valid
                    && f.valid_to.map(|t| as_of_valid < t).unwrap_or(true)
            })
    }

    /// Snapshot of current keys as-of both axes (latest open fact per key).
    pub fn as_of(
        &self,
        as_of_valid: Option<DateTime<Utc>>,
        as_of_tx: Option<DateTime<Utc>>,
    ) -> Vec<&MemoryFact> {
        let now = Utc::now();
        let as_of_valid = as_of_valid.unwrap_or(now);
        let as_of_tx = as_of_tx.unwrap_or(now);
        let mut keys: Vec<String> = self
            .facts
            .iter()
            .filter(|f| f.tx_time <= as_of_tx)
            .map(|f| f.key.clone())
            .collect();
        keys.sort();
        keys.dedup();
        keys.into_iter()
            .filter_map(|k| self.recall(&k, Some(as_of_valid), Some(as_of_tx)))
            .collect()
    }
}

/// True when query looks like a well-formed remember/store write (`key = value`).
pub fn looks_remember_ask(query: &str) -> bool {
    parse_remember(query).is_some()
}

/// True when query looks like a memory recall.
pub fn looks_recall_ask(query: &str) -> bool {
    parse_recall_key(query).is_some()
}

/// Parse `remember <key> = <value>` / `store <key>: <value>` / `memory write …`.
pub fn parse_remember(query: &str) -> Option<MemoryWriteProposal> {
    let raw = query.trim();
    let q = raw.to_ascii_lowercase();
    let rest = if let Some(r) = q.strip_prefix("memory write ") {
        r
    } else if let Some(r) = q.strip_prefix("remember ") {
        r
    } else if let Some(r) = q.strip_prefix("store memory ") {
        r
    } else if let Some(r) = q.strip_prefix("store ") {
        r
    } else {
        return None;
    };
    // Find separator in the lowercased rest, map back to original casing slice.
    let prefix_len = raw.len() - rest.len();
    let sep_pos = rest.find('=').or_else(|| rest.find(':'))?;
    let sep_char = rest.as_bytes()[sep_pos] as char;
    let key_l = rest[..sep_pos].trim();
    let val_l = rest[sep_pos + sep_char.len_utf8()..].trim();
    if key_l.is_empty() || val_l.is_empty() {
        return None;
    }
    // Reject free-form VoI bait that slipped past (value is only whitespace junk).
    if key_l.split_whitespace().count() > 4 {
        return None;
    }
    let orig_rest = raw[prefix_len..].trim();
    let orig_sep = orig_rest.find(sep_char)?;
    let key = orig_rest[..orig_sep].trim().to_string();
    let value = orig_rest[orig_sep + sep_char.len_utf8()..].trim().to_string();
    if key.is_empty() || value.is_empty() {
        return None;
    }
    Some(MemoryWriteProposal {
        key,
        value,
        valid_from: None,
        valid_to: None,
    })
}

/// Parse recall key from `recall <key>` / `what do you remember about <key>`.
pub fn parse_recall_key(query: &str) -> Option<String> {
    let raw = query.trim();
    let q = raw.to_ascii_lowercase();
    let key = if let Some(r) = q.strip_prefix("recall memory ") {
        r.trim()
    } else if let Some(r) = q.strip_prefix("recall ") {
        r.trim()
    } else if let Some(r) = q.strip_prefix("what do you remember about ") {
        r.trim().trim_end_matches('?').trim()
    } else if let Some(r) = q.strip_prefix("remember about ") {
        r.trim().trim_end_matches('?').trim()
    } else {
        return None;
    };
    if key.is_empty() || key.contains('=') {
        return None;
    }
    // Prefer original casing from the tail of the raw query.
    let lower_key = key.to_string();
    let orig = if let Some(i) = q.rfind(&lower_key) {
        raw[i..i + lower_key.len()].to_string()
    } else {
        lower_key
    };
    Some(orig.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn parse_remember_and_recall() {
        let p = parse_remember("remember landauer_note = E_min = kT ln2 per bit").unwrap();
        assert_eq!(p.key, "landauer_note");
        assert!(p.value.contains("kT"));
        assert!(looks_remember_ask("store foo: bar"));
        assert!(!looks_remember_ask("remember write a poem about GPUs"));
        assert_eq!(
            parse_recall_key("recall landauer_note").as_deref(),
            Some("landauer_note")
        );
        assert_eq!(
            parse_recall_key("what do you remember about landauer_note?").as_deref(),
            Some("landauer_note")
        );
    }

    #[test]
    fn bitemporal_write_recall_and_supersede() {
        let mut s = BitemporalStore::new();
        let t0 = Utc::now() - Duration::seconds(10);
        let f1 = s.commit_write("k", "v1", Some(t0), None, Some("r1".into()));
        assert_eq!(f1.tx_id, 0);
        assert!(f1.id.as_str().contains("memory:k#tx0"));
        let got = s.recall("k", None, None).unwrap();
        assert_eq!(got.value, "v1");

        let t1 = Utc::now();
        let f2 = s.commit_write("k", "v2", Some(t1), None, None);
        assert_eq!(f2.tx_id, 1);
        assert_eq!(s.recall("k", None, None).unwrap().value, "v2");
        // As-of valid time before supersede → v1
        assert_eq!(
            s.recall("k", Some(t0 + Duration::seconds(1)), None)
                .unwrap()
                .value,
            "v1"
        );
        // As-of (valid in open interval of v1, tx at v1) → v1
        assert_eq!(
            s.recall("k", Some(t0 + Duration::seconds(1)), Some(f1.tx_time))
                .unwrap()
                .value,
            "v1"
        );
    }

    #[test]
    fn unknown_recall_misses() {
        let s = BitemporalStore::new();
        assert!(s.recall("missing", None, None).is_none());
    }
}
