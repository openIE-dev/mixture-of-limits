//! Close-step transcripts for receipt replay (JSONL or in-memory).
//!
//! A transcript records commit/refuse + limit id (+ answering tier) per ask.
//! Replay re-closes each entry and checks the fingerprint matches **without**
//! requiring model escalation (`allow_model` stays false; Model must not Answer).

use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;

use mol_core::{Budget, CascadeTier, MolError, Result};

use crate::receipt::MolReceipt;

/// Fingerprint of one close outcome (commit/refuse + limit id + tier).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloseFingerprint {
    /// True iff close committed.
    pub commit: bool,
    /// Binding limit id when refused (or stop).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit_id: Option<String>,
    /// Answering cascade tier when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cascade_answered: Option<String>,
}

impl CloseFingerprint {
    /// Build from a receipt + commit flag.
    pub fn from_receipt(commit: bool, r: &MolReceipt) -> Self {
        Self {
            commit,
            limit_id: r.limit_fired.as_ref().map(|f| f.id.as_str().to_string()),
            cascade_answered: r.cascade_answered.map(|t| t.label().to_string()),
        }
    }
}

impl fmt::Display for CloseFingerprint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "commit={} limit={:?} tier={:?}",
            self.commit, self.limit_id, self.cascade_answered
        )
    }
}

/// One recorded close step in a transcript.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CloseTranscriptEntry {
    /// Original query text.
    pub query: String,
    /// Budget max joules (estimated).
    pub max_j: f64,
    /// Whether model leaf was allowed (replay proof keeps this false).
    #[serde(default)]
    pub allow_model: bool,
    /// Soft latency ms if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_latency_ms: Option<u64>,
    /// Expected fingerprint at record time.
    pub expected: CloseFingerprint,
}

impl CloseTranscriptEntry {
    /// Budget reconstructed from the entry.
    pub fn budget(&self) -> Budget {
        let mut b = Budget::joules(self.max_j);
        if let Some(ms) = self.max_latency_ms {
            b = b.with_latency_ms(ms);
        }
        if self.allow_model {
            b = b.allow_model();
        }
        b
    }

    /// Record from query + budget + outcome.
    pub fn record(query: impl Into<String>, budget: Budget, commit: bool, receipt: &MolReceipt) -> Self {
        Self {
            query: query.into(),
            max_j: budget.max_j.0,
            allow_model: budget.allow_model,
            max_latency_ms: budget.max_latency.map(|l| l.max_ms),
            expected: CloseFingerprint::from_receipt(commit, receipt),
        }
    }
}

/// Ordered transcript of close steps.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CloseTranscript {
    /// Schema tag.
    #[serde(default = "CloseTranscript::default_schema")]
    pub schema: String,
    /// Entries in order.
    pub entries: Vec<CloseTranscriptEntry>,
}

impl CloseTranscript {
    /// Schema version.
    pub const SCHEMA: &'static str = "mol.transcript.v1";

    fn default_schema() -> String {
        Self::SCHEMA.into()
    }

    /// Empty transcript.
    pub fn new() -> Self {
        Self {
            schema: Self::SCHEMA.into(),
            entries: Vec::new(),
        }
    }

    /// Push a recorded close.
    pub fn push(&mut self, entry: CloseTranscriptEntry) {
        self.entries.push(entry);
    }

    /// Serialize as JSONL (one entry object per line).
    pub fn to_jsonl(&self) -> Result<String> {
        let mut out = String::new();
        for e in &self.entries {
            let line = serde_json::to_string(e)
                .map_err(|e| MolError::Msg(format!("serde: {e}")))?;
            out.push_str(&line);
            out.push('\n');
        }
        Ok(out)
    }

    /// Parse JSONL (one [`CloseTranscriptEntry`] per non-empty line).
    pub fn from_jsonl(text: &str) -> Result<Self> {
        let mut entries = Vec::new();
        for (i, line) in text.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let e: CloseTranscriptEntry = serde_json::from_str(line).map_err(|e| {
                MolError::Msg(format!("transcript JSONL line {}: {e}", i + 1))
            })?;
            entries.push(e);
        }
        Ok(Self {
            schema: Self::SCHEMA.into(),
            entries,
        })
    }

    /// Load JSONL from a path.
    pub fn load_jsonl(path: impl AsRef<Path>) -> Result<Self> {
        let text = std::fs::read_to_string(path.as_ref())
            .map_err(|e| MolError::Io(format!("read transcript: {e}")))?;
        Self::from_jsonl(&text)
    }

    /// Write JSONL to a path.
    pub fn save_jsonl(&self, path: impl AsRef<Path>) -> Result<()> {
        let text = self.to_jsonl()?;
        if let Some(parent) = path.as_ref().parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| MolError::Io(format!("mkdir transcript: {e}")))?;
            }
        }
        std::fs::write(path.as_ref(), text)
            .map_err(|e| MolError::Io(format!("write transcript: {e}")))
    }
}

/// Per-entry replay result.
#[derive(Debug, Clone)]
pub struct ReplayEntryReport {
    /// 0-based index.
    pub index: usize,
    /// Query.
    pub query: String,
    /// True when fingerprint matched and no model answer.
    pub ok: bool,
    /// Detail lines.
    pub checks: Vec<String>,
}

/// Aggregate replay report.
#[derive(Debug, Clone)]
pub struct ReplayReport {
    /// Total entries.
    pub total: usize,
    /// Passed.
    pub passed: usize,
    /// Failed / drifted.
    pub failed: usize,
    /// Per-entry.
    pub entries: Vec<ReplayEntryReport>,
}

impl ReplayReport {
    /// True when every entry passed and transcript non-empty.
    pub fn ok(&self) -> bool {
        self.failed == 0 && self.total > 0
    }

    /// Multi-line CLI display.
    pub fn display_block(&self) -> String {
        let mut out = String::new();
        for e in &self.entries {
            out.push_str(&format!(
                "── replay [{}] {} — {}\n",
                e.index,
                if e.ok { "PASS" } else { "FAIL" },
                e.query
            ));
            for c in &e.checks {
                out.push_str("   ");
                out.push_str(c);
                out.push('\n');
            }
        }
        out.push_str(&format!(
            "═══ replay summary: {} total, {} pass, {} fail ═══\n",
            self.total, self.passed, self.failed
        ));
        out
    }
}

/// Outcome of a single close used by [`replay_transcript`].
#[derive(Debug, Clone)]
pub struct ReplayCloseOutcome {
    /// Commit vs refuse.
    pub commit: bool,
    /// Receipt after close.
    pub receipt: MolReceipt,
}

/// Replay a transcript by re-closing each entry with `closer`.
///
/// Checks:
/// 1. Fingerprint (commit + limit_id + cascade_answered) matches recorded expected.
/// 2. Model tier did not Answer (no re-escalation to model).
/// 3. Entry `allow_model` is false on the proof path (warn/fail if true).
pub fn replay_transcript<F>(transcript: &CloseTranscript, mut closer: F) -> Result<ReplayReport>
where
    F: FnMut(&str, Budget) -> Result<ReplayCloseOutcome>,
{
    let mut entries = Vec::new();
    let mut passed = 0usize;
    let mut failed = 0usize;

    for (index, entry) in transcript.entries.iter().enumerate() {
        let mut checks = Vec::new();
        let mut ok = true;

        if entry.allow_model {
            ok = false;
            checks.push("FAIL: allow_model=true (replay proof requires model demoted)".into());
        }

        let out = match closer(&entry.query, entry.budget()) {
            Ok(o) => o,
            Err(e) => {
                failed += 1;
                entries.push(ReplayEntryReport {
                    index,
                    query: entry.query.clone(),
                    ok: false,
                    checks: vec![format!("FAIL: close error: {e}")],
                });
                continue;
            }
        };

        let got = CloseFingerprint::from_receipt(out.commit, &out.receipt);
        if got != entry.expected {
            ok = false;
            checks.push(format!(
                "FAIL: fingerprint drift expected={} got={}",
                entry.expected, got
            ));
        } else {
            checks.push(format!("OK: fingerprint {got}"));
        }

        // Model must not have answered.
        let model_answered = out.receipt.cascade_answered == Some(CascadeTier::Model)
            || out.receipt.cascade_steps.iter().any(|s| {
                s.tier == CascadeTier::Model
                    && matches!(
                        s.outcome,
                        crate::receipt::CascadeStepOutcome::Answered
                    )
            });
        if model_answered {
            ok = false;
            checks.push("FAIL: model tier answered (replay must not re-escalate to model)".into());
        } else {
            checks.push("OK: model not answered".into());
        }

        // Honesty: no fake measured_j on replay path.
        if out.receipt.measured_j.is_some() {
            ok = false;
            checks.push("FAIL: measured_j present on soft-ref replay".into());
        }

        if ok {
            passed += 1;
        } else {
            failed += 1;
        }
        entries.push(ReplayEntryReport {
            index,
            query: entry.query.clone(),
            ok,
            checks,
        });
    }

    Ok(ReplayReport {
        total: transcript.entries.len(),
        passed,
        failed,
        entries,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use mol_core::{EstimateKind, MeasureSource, MuSource};
    use crate::receipt::ReceiptBuilder;

    fn dummy_receipt(tier: Option<CascadeTier>, limit: Option<&str>) -> MolReceipt {
        let mut b = ReceiptBuilder::new()
            .estimated_j(Joules_zero())
            .estimate_kind(EstimateKind::Analytical)
            .measure_source(MeasureSource::CascadeEstimate)
            .mu(1e3, MuSource::Catalog)
            .rationale("test");
        if let Some(t) = tier {
            b = b.answered(t).answer("ok");
        }
        if let Some(id) = limit {
            b = b.limit_fired(mol_core::Floor::new(id, mol_core::FloorKind::ValueOfInformation, "x"));
        }
        b.build()
    }

    fn Joules_zero() -> mol_core::Joules {
        mol_core::Joules::ZERO
    }

    #[test]
    fn jsonl_roundtrip() {
        let r = dummy_receipt(Some(CascadeTier::Formula), None);
        let mut t = CloseTranscript::new();
        t.push(CloseTranscriptEntry::record(
            "landauer joules per bit",
            Budget::coin_cell(),
            true,
            &r,
        ));
        let jsonl = t.to_jsonl().unwrap();
        let back = CloseTranscript::from_jsonl(&jsonl).unwrap();
        assert_eq!(back.entries.len(), 1);
        assert_eq!(back.entries[0].query, "landauer joules per bit");
        assert!(back.entries[0].expected.commit);
    }
}
