//! Klere-style ternary settle-or-refuse port.
//!
//! Software-reference stub of settle-to-silence: iterate a tiny ternary
//! attractor; if it will not settle, **refuse** (report, don't invent).
//! Priced surrogate joules only — never claim FPGA pJ/accumulate as `measured_j`.
//! `board_synth_claimed=false`. Does not ship klere-vm / FPGA SDK.

use mol_core::{Floor, FloorKind, MolError, Result, BOARD_SYNTH_CLAIMED};
use serde::{Deserialize, Serialize};

/// Ternary symbol {-1, 0, +1}.
pub type Ternary = i8;

/// Port toward Klere settle fabric / klere-vm twin.
pub trait KlereSettlePort: Send + Sync {
    /// Attempt settle; return answer or refuse.
    fn settle(&self, job: &KlereSettleJob) -> Result<KlereSettleResult>;
}

/// Settle job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlereSettleJob {
    /// Human label.
    pub summary: String,
    /// Initial ternary state.
    pub state: Vec<Ternary>,
    /// Max restore steps.
    pub max_steps: u32,
    /// When true, force refuse path (demo).
    #[serde(default)]
    pub force_refuse: bool,
}

/// Settle decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KlereDecision {
    /// Settled to silence (fixed point).
    Settled,
    /// Will not settle — refuse.
    Refuse,
}

/// Settle result with honesty fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlereSettleResult {
    /// settled | refuse
    pub decision: KlereDecision,
    /// Final state when settled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub final_state: Option<Vec<Ternary>>,
    /// Steps taken.
    pub steps: u32,
    /// Surrogate priced joules (catalog; not FPGA measured on this host).
    pub estimated_j: f64,
    /// Always None on this stub — never fake FPGA meter.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub measured_j: Option<f64>,
    /// Always false.
    pub board_synth_claimed: bool,
    /// Optional MoL floor on refuse.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub floor: Option<Floor>,
    /// Note.
    pub note: String,
}

/// Default software-reference Klere settle.
///
/// Dynamics (tiny demo, not EPU): each step, set s_i ← sign(Σ_j W_ij s_j) with
/// a fixed circulant weight; 0 stays 0. If force_refuse, or state contains a
/// `will_not_settle` marker pattern (all +1 with max_steps==0), refuse.
#[derive(Debug, Default, Clone)]
pub struct StubKlereSettle;

impl KlereSettlePort for StubKlereSettle {
    fn settle(&self, job: &KlereSettleJob) -> Result<KlereSettleResult> {
        // Surrogate price: ~0.16 pJ/accumulate × |state| × steps (Klere-reported
        // order; labelled estimate, not measured on this host).
        const PJ_PER_ACC: f64 = 0.1596e-12;

        if job.force_refuse
            || job.summary.to_ascii_lowercase().contains("will not settle")
            || job.summary.to_ascii_lowercase().contains("settle refuse")
            || job.max_steps == 0
        {
            let reason = format!(
                "Klere settle refuse: landscape will not settle ({})",
                job.summary
            );
            let floor = Floor::new("settle_refuse", FloorKind::SettleRefuse, &reason);
            return Ok(KlereSettleResult {
                decision: KlereDecision::Refuse,
                final_state: None,
                steps: 0,
                estimated_j: PJ_PER_ACC * job.state.len().max(1) as f64,
                measured_j: None,
                board_synth_claimed: BOARD_SYNTH_CLAIMED,
                floor: Some(floor),
                note: "software-reference Klere stub; priced ≠ FPGA measured; board_synth=false"
                    .into(),
            });
        }

        let n = job.state.len();
        if n == 0 {
            let reason = "Klere settle refuse: empty state";
            let floor = Floor::new("settle_refuse", FloorKind::SettleRefuse, reason);
            return Ok(KlereSettleResult {
                decision: KlereDecision::Refuse,
                final_state: None,
                steps: 0,
                estimated_j: PJ_PER_ACC,
                measured_j: None,
                board_synth_claimed: BOARD_SYNTH_CLAIMED,
                floor: Some(floor),
                note: "empty state cannot settle".into(),
            });
        }

        let mut s = job.state.clone();
        // Clamp to ternary.
        for v in &mut s {
            *v = match *v {
                x if x > 0 => 1,
                x if x < 0 => -1,
                _ => 0,
            };
        }

        let mut steps = 0u32;
        for _ in 0..job.max_steps.max(1) {
            let mut next = s.clone();
            for i in 0..n {
                if s[i] == 0 {
                    continue; // silence stays
                }
                // Circulant neighbour sum.
                let left = s[(i + n - 1) % n];
                let right = s[(i + 1) % n];
                let field = left as i32 + right as i32;
                next[i] = if field > 0 {
                    1
                } else if field < 0 {
                    -1
                } else {
                    0
                };
            }
            steps += 1;
            if next == s {
                let estimated_j = PJ_PER_ACC * n as f64 * steps as f64;
                return Ok(KlereSettleResult {
                    decision: KlereDecision::Settled,
                    final_state: Some(s),
                    steps,
                    estimated_j,
                    measured_j: None,
                    board_synth_claimed: BOARD_SYNTH_CLAIMED,
                    floor: None,
                    note: "software-reference settle-to-silence; estimated priced joules only"
                        .into(),
                });
            }
            s = next;
        }

        let reason = format!(
            "Klere settle refuse: no fixed point in {} steps ({})",
            job.max_steps, job.summary
        );
        let floor = Floor::new("settle_refuse", FloorKind::SettleRefuse, &reason);
        Ok(KlereSettleResult {
            decision: KlereDecision::Refuse,
            final_state: Some(s),
            steps,
            estimated_j: PJ_PER_ACC * n as f64 * steps as f64,
            measured_j: None,
            board_synth_claimed: BOARD_SYNTH_CLAIMED,
            floor: Some(floor),
            note: "did not settle; refuse over invent; measured_j=None".into(),
        })
    }
}

/// Explicit live-path stub (klere-vm / FPGA not wired).
pub fn live_klere_stub(_job: &KlereSettleJob) -> Result<KlereSettleResult> {
    Err(MolError::AdapterStub(
        "klere live path not wired (klere-vm/FPGA); use StubKlereSettle software-reference".into(),
    ))
}

/// Parse a ternary vector from query text like `settle ternary [1, 0, -1, 1]`.
pub fn parse_ternary_state(query: &str) -> Option<Vec<Ternary>> {
    let start = query.find('[')?;
    let end = query[start..].find(']')? + start;
    let inner = &query[start + 1..end];
    let mut out = Vec::new();
    for tok in inner.split(|c: char| c == ',' || c.is_whitespace()) {
        let t = tok.trim();
        if t.is_empty() {
            continue;
        }
        let v: i8 = t.parse().ok()?;
        out.push(match v {
            x if x > 0 => 1,
            x if x < 0 => -1,
            _ => 0,
        });
    }
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}
