//! Thin agent mailbox loop — sense → classify → cascade/floors → certify → commit|refuse → record.
//!
//! MoL owns close via [`AutomateGate`] / `MixtureOfLimits::close`. Model stays demoted:
//! budgets keep `allow_model=false`; the loop never silently escalates to Model.

use serde::{Deserialize, Serialize};

use mol_core::{looks_remember_ask, Budget, CascadeTier};
use mol_receipt::CascadeStepOutcome;

use crate::act::{Act, ActKind};
use crate::gate::{AutomateGate, CommitDecision};
use crate::mailbox::{AgentMailbox, AgentTranscriptEntry, MailItem};

/// Outcome of one agent-loop step.
#[derive(Debug, Clone)]
pub struct AgentStepOutcome {
    /// Transcript entry recorded for this step.
    pub entry: AgentTranscriptEntry,
}

/// Report after draining the mailbox.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRunReport {
    /// Steps executed.
    pub steps: usize,
    /// How many committed.
    pub commits: usize,
    /// How many refused.
    pub refuses: usize,
    /// True iff any step had model Answered (must be false under default policy).
    pub any_model_answered: bool,
    /// True iff board_synth stayed false on every receipt.
    pub board_synth_honest: bool,
    /// True iff measured_j stayed None on every receipt.
    pub measured_j_honest: bool,
    /// Transcript display lines.
    pub lines: Vec<String>,
}

impl AgentRunReport {
    /// Pretty block for CLI.
    pub fn display_block(&self) -> String {
        let mut out = String::new();
        out.push_str("═══ Agent mailbox loop ═══\n");
        out.push_str(&format!(
            "steps={} commits={} refuses={} model_answered={} measured_j_honest={} board_synth_honest={}\n",
            self.steps,
            self.commits,
            self.refuses,
            self.any_model_answered,
            self.measured_j_honest,
            self.board_synth_honest
        ));
        for line in &self.lines {
            out.push_str(line);
            out.push('\n');
        }
        out.push_str("══════════════════════════\n");
        out
    }

    /// Proof / demo success under honesty + no model escalate.
    pub fn ok_no_model(&self) -> bool {
        self.steps > 0
            && !self.any_model_answered
            && self.measured_j_honest
            && self.board_synth_honest
    }
}

/// Demo goal expected fingerprint (commit/refuse + optional limit id).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DemoGoalExpect {
    /// Goal / query text posted to mailbox.
    pub goal: &'static str,
    /// Expected commit.
    pub commit: bool,
    /// Expected limit id when refused (None = any/none).
    pub limit_id: Option<&'static str>,
}

/// Canonical demo goals: formula, cite, compose, VoI refuse, settle refuse.
pub fn demo_goal_expects() -> &'static [DemoGoalExpect] {
    &[
        DemoGoalExpect {
            goal: "landauer joules per bit",
            commit: true,
            limit_id: None,
        },
        DemoGoalExpect {
            goal: "what is the landauer principle",
            commit: true,
            limit_id: None,
        },
        DemoGoalExpect {
            goal: "compose claim:landauer.principle and claim:mol.law",
            commit: true,
            limit_id: None,
        },
        DemoGoalExpect {
            goal: "write a poem about GPUs",
            commit: false,
            limit_id: Some("voi"),
        },
        DemoGoalExpect {
            goal: "settle refuse will not settle [1,1,1]",
            commit: false,
            limit_id: Some("settle_refuse"),
        },
    ]
}

/// Thin agent: mailbox + AutomateGate.
///
/// Loop invariant: `allow_model` stays false — no silent model escalate.
pub struct AgentLoop {
    /// Mailbox (sense surface + transcript).
    pub mailbox: AgentMailbox,
    /// Capability-gated close path.
    pub gate: AutomateGate,
    /// Max joules for classified Propose acts.
    pub max_j: f64,
    /// Must stay false for proof / demo (no silent Model escalate).
    pub allow_model: bool,
}

impl Default for AgentLoop {
    fn default() -> Self {
        Self {
            mailbox: AgentMailbox::new(),
            gate: AutomateGate::default(),
            max_j: Budget::demo().max_j.0,
            allow_model: false,
        }
    }
}

impl AgentLoop {
    /// Construct with default sense+propose caps and demoted model.
    pub fn new() -> Self {
        Self::default()
    }

    /// Post a goal into the mailbox.
    pub fn post_goal(&mut self, text: impl Into<String>) -> u64 {
        self.mailbox.post_goal(text)
    }

    /// Post a message into the mailbox.
    pub fn post_message(
        &mut self,
        from: impl Into<String>,
        to: impl Into<String>,
        body: impl Into<String>,
    ) -> u64 {
        self.mailbox.post_message(from, to, body)
    }

    /// Post an explicit act proposal.
    pub fn post_act(&mut self, act: Act) -> u64 {
        self.mailbox.post_act(act)
    }

    /// Seed demo goals (formula / cite / compose / voi / settle refuse).
    pub fn seed_demo_goals(&mut self) {
        for g in demo_goal_expects() {
            self.post_goal(g.goal);
        }
    }

    /// Grant Mutate capability (needed for bitemporal memory writes).
    pub fn grant_mutate(&mut self) {
        use crate::act::Capability;
        if !self.gate.caps.allows(Capability::Mutate) {
            self.gate.caps.granted.push(Capability::Mutate);
        }
    }

    /// Run bitemporal memory demo: capability refuse → grant → write → recall cite → VoI refuse.
    ///
    /// Returns `(capability_refused, write_committed, recall_committed, citation_id)`.
    pub fn run_memory_demo(&mut self) -> Result<(bool, bool, bool, String), String> {
        // 1) Remember without Mutate → capability refuse; store empty.
        self.mailbox.clear();
        self.gate.caps = crate::act::CapabilitySet::sense_propose();
        // Reset store for demo isolation.
        *self.gate.mol.memory_lock() = mol_core::BitemporalStore::new();
        self.post_goal("remember landauer_note = E_min = kT ln2 per bit");
        let step1 = self.step().ok_or("no step for remember without cap")?;
        let cap_refused = !step1.entry.commit;
        if !cap_refused {
            return Err("expected capability refuse on remember without Mutate".into());
        }
        if !self.gate.mol.memory_lock().is_empty() {
            return Err("store must stay empty on capability refuse".into());
        }

        // 2) Grant Mutate, remember → COMMIT + store write
        self.grant_mutate();
        self.mailbox.clear();
        self.post_goal("remember landauer_note = E_min = kT ln2 per bit");
        let step2 = self.step().ok_or("no step for remember with Mutate")?;
        if !step2.entry.commit {
            return Err(format!(
                "expected COMMIT remember with Mutate, limit={:?}",
                step2.entry.limit_id
            ));
        }
        if self.gate.mol.memory_lock().is_empty() {
            return Err("store empty after memory COMMIT".into());
        }
        let write_ok = true;

        // 3) Recall → COMMIT RetrievedCited + memory: citation
        self.post_goal("recall landauer_note");
        let step3 = self.step().ok_or("no step for recall")?;
        if !step3.entry.commit {
            return Err(format!(
                "expected COMMIT recall, limit={:?}",
                step3.entry.limit_id
            ));
        }
        if step3.entry.receipt.replay_class != Some(mol_core::ReplayClass::RetrievedCited) {
            return Err(format!(
                "expected RetrievedCited, got {:?}",
                step3.entry.receipt.replay_class
            ));
        }
        let cite = step3
            .entry
            .receipt
            .citation_ids
            .first()
            .cloned()
            .ok_or("recall missing citation_ids")?;
        if !cite.starts_with("memory:") {
            return Err(format!("expected memory: cite, got {cite}"));
        }
        if step3.entry.model_answered || step3.entry.receipt.measured_j.is_some() {
            return Err("honesty violated on recall".into());
        }
        if step3.entry.receipt.board_synth_claimed {
            return Err("board_synth claimed on recall".into());
        }

        // 4) VoI: free-form remember bait (no key=value) → REFUSE voi
        self.post_goal("remember write a poem about GPUs");
        let step4 = self.step().ok_or("no step for voi remember")?;
        if step4.entry.commit {
            return Err("free-form remember must REFUSE".into());
        }
        if step4.entry.limit_id.as_deref() != Some("voi") {
            return Err(format!(
                "expected voi refuse, got {:?}",
                step4.entry.limit_id
            ));
        }

        Ok((cap_refused, write_ok, step3.entry.commit, cite))
    }

    /// Classify a mail item into an Act for AutomateGate.
    ///
    /// Goals and messages become `Propose` with `payload.query`. Act proposals
    /// pass through (estimated_j clamped to gate budget).
    pub fn classify(&self, item: &MailItem) -> Act {
        match item {
            MailItem::Goal { text } => {
                if looks_remember_ask(text) {
                    // Irreversible memory write — requires Mutate capability; MoL close owns commit.
                    let mut act = Act::new(ActKind::Mutate, format!("remember:{text}"), 1e-12);
                    act.payload = Some(serde_json::json!({ "query": text, "memory_write": true }));
                    act
                } else {
                    let mut act = Act::new(ActKind::Propose, format!("goal:{text}"), 1e-12);
                    act.payload = Some(serde_json::json!({ "query": text }));
                    act
                }
            }
            MailItem::Message { from, to, body } => {
                if looks_remember_ask(body) {
                    let mut act = Act::new(
                        ActKind::Mutate,
                        format!("remember:{from}->{to}"),
                        1e-12,
                    );
                    act.payload = Some(serde_json::json!({ "query": body, "memory_write": true }));
                    act
                } else {
                    let mut act = Act::new(
                        ActKind::Propose,
                        format!("message:{from}->{to}"),
                        1e-12,
                    );
                    act.payload = Some(serde_json::json!({ "query": body }));
                    act
                }
            }
            MailItem::ActProposal { act } => {
                let mut a = act.clone();
                if a.estimated_j > self.max_j {
                    a.estimated_j = self.max_j;
                }
                // Ensure Propose/Sense with query when summary looks like a query and payload missing.
                if a.payload.is_none()
                    && matches!(a.kind, ActKind::Propose | ActKind::Sense)
                    && !a.summary.is_empty()
                {
                    a.payload = Some(serde_json::json!({ "query": a.summary }));
                }
                a
            }
        }
    }

    /// One loop step: sense → classify → gate (close/certify) → record.
    ///
    /// Returns `None` when the mailbox has no pending items.
    ///
    /// Policy: `allow_model` must stay false — AutomateGate builds
    /// `Budget::joules(max_j)` with `allow_model=false`, so Model never answers.
    pub fn step(&mut self) -> Option<AgentStepOutcome> {
        let (mail_id, item) = {
            let env = self.mailbox.sense()?;
            (env.id, env.item.clone())
        };
        let sensed = item.summary();
        let classified_query = item.query_text().map(|s| s.to_string());
        let act = self.classify(&item);

        // Hard policy: thin agent loop does not open the model leaf.
        // AutomateGate::gate → mol.close uses Budget::joules(max_j) with allow_model=false.
        let out = self.gate.gate(&act);

        let commit = matches!(out.decision, CommitDecision::Commit);
        let limit_id = out
            .receipt
            .limit_fired
            .as_ref()
            .map(|f| f.id.as_str().to_string());
        let cascade_answered = out
            .receipt
            .cascade_answered
            .map(|t| t.label().to_string());
        let replay_class = out.receipt.replay_class.map(|r| r.to_string());
        let model_answered = out.receipt.cascade_steps.iter().any(|s| {
            s.tier == CascadeTier::Model && matches!(s.outcome, CascadeStepOutcome::Answered)
        });

        let entry = AgentTranscriptEntry {
            mail_id,
            sensed,
            classified_query,
            act_kind: format!("{:?}", act.kind).to_ascii_lowercase(),
            commit,
            limit_id,
            cascade_answered,
            replay_class,
            model_answered,
            receipt: out.receipt,
        };
        self.mailbox.record(mail_id, entry.clone());
        Some(AgentStepOutcome { entry })
    }

    /// Drain all pending mailbox items.
    pub fn run(&mut self) -> AgentRunReport {
        let mut lines = Vec::new();
        let mut commits = 0usize;
        let mut refuses = 0usize;
        let mut any_model = false;
        let mut measured_ok = true;
        let mut board_ok = true;
        let mut steps = 0usize;

        while let Some(out) = self.step() {
            steps += 1;
            if out.entry.commit {
                commits += 1;
            } else {
                refuses += 1;
            }
            if out.entry.model_answered {
                any_model = true;
            }
            if out.entry.receipt.measured_j.is_some() {
                measured_ok = false;
            }
            if out.entry.receipt.board_synth_claimed {
                board_ok = false;
            }
            lines.push(out.entry.display_line());
        }

        AgentRunReport {
            steps,
            commits,
            refuses,
            any_model_answered: any_model,
            board_synth_honest: board_ok,
            measured_j_honest: measured_ok,
            lines,
        }
    }

    /// Seed demo goals and run; returns report + transcript snapshot.
    pub fn run_demo(&mut self) -> AgentRunReport {
        self.mailbox.clear();
        self.seed_demo_goals();
        self.run()
    }

    /// Verify demo expectations against the current transcript (after run_demo).
    pub fn demo_expectations_met(&self) -> Result<(), String> {
        let expects = demo_goal_expects();
        let t = self.mailbox.transcript();
        if t.len() != expects.len() {
            return Err(format!(
                "transcript len {} != demo expects {}",
                t.len(),
                expects.len()
            ));
        }
        for (entry, exp) in t.iter().zip(expects.iter()) {
            if entry.model_answered {
                return Err(format!("model answered on '{}'", exp.goal));
            }
            if entry.commit != exp.commit {
                return Err(format!(
                    "goal '{}' commit={} expected {}",
                    exp.goal, entry.commit, exp.commit
                ));
            }
            if let Some(want) = exp.limit_id {
                let got = entry.limit_id.as_deref().unwrap_or("");
                if got != want {
                    return Err(format!(
                        "goal '{}' limit_id='{got}' expected '{want}'",
                        exp.goal
                    ));
                }
            }
            if entry.receipt.measured_j.is_some() {
                return Err(format!("measured_j set on '{}'", exp.goal));
            }
            if entry.receipt.board_synth_claimed {
                return Err(format!("board_synth claimed on '{}'", exp.goal));
            }
            // Classified query should match goal text for Goal items.
            if entry.classified_query.as_deref() != Some(exp.goal) {
                return Err(format!(
                    "classified_query mismatch for '{}': {:?}",
                    exp.goal, entry.classified_query
                ));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::act::{Act, ActKind};
    use crate::gate::CommitDecision;

    #[test]
    fn demo_loop_hits_formula_cite_compose_voi_settle() {
        let mut agent = AgentLoop::new();
        assert!(!agent.allow_model);
        let report = agent.run_demo();
        assert!(report.ok_no_model(), "{report:?}");
        assert_eq!(report.steps, 5);
        assert_eq!(report.commits, 3);
        assert_eq!(report.refuses, 2);
        agent.demo_expectations_met().expect("demo expects");
    }

    #[test]
    fn message_path_closes_formula() {
        let mut agent = AgentLoop::new();
        agent.post_message("user", "mol", "convert 100 celsius to fahrenheit");
        let report = agent.run();
        assert_eq!(report.steps, 1);
        assert_eq!(report.commits, 1);
        assert!(!report.any_model_answered);
        let e = &agent.mailbox.transcript()[0];
        assert!(e.commit);
        assert!(e.cascade_answered.is_some());
    }

    #[test]
    fn act_proposal_mutate_refused_by_default() {
        let mut agent = AgentLoop::new();
        agent.post_act(Act::new(ActKind::Mutate, "touch /tmp/x", 1e-12));
        let out = agent.step().expect("step");
        assert!(!out.entry.commit);
        assert!(!out.entry.model_answered);
        // Capability deny path
        assert!(matches!(
            agent.gate.gate(&Act::new(ActKind::Mutate, "touch /tmp/x", 1e-12)).decision,
            CommitDecision::Refuse(_)
        ));
    }

    #[test]
    #[test]
    fn memory_demo_bitemporal_cite() {
        let mut agent = AgentLoop::new();
        let (cap_r, write_c, recall_c, cite) = agent.run_memory_demo().expect("memory demo");
        assert!(cap_r);
        assert!(write_c);
        assert!(recall_c);
        assert!(cite.starts_with("memory:"));
        assert!(!agent.gate.mol.memory_lock().is_empty());
    }

    fn no_silent_model_escalate_on_freeform() {
        let mut agent = AgentLoop::new();
        agent.post_goal("write a poem about GPUs");
        let report = agent.run();
        assert!(!report.any_model_answered);
        assert_eq!(report.refuses, 1);
        assert_eq!(
            agent.mailbox.transcript()[0].limit_id.as_deref(),
            Some("voi")
        );
    }
}
