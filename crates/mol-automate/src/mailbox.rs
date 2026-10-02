//! Thin agent mailbox — Goal / Message / Act proposals + close transcript.
//!
//! Clean-room (no path-dep on openie-leapfrog). The mailbox is the sense surface
//! for the agent loop: pending items are classified into Acts, closed through
//! AutomateGate / MoL, and commit|refuse receipts are appended to the transcript.

use serde::{Deserialize, Serialize};

use crate::act::Act;
use mol_receipt::MolReceipt;

/// Kind of pending mailbox item.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MailItem {
    /// Operator / planner goal text (becomes a Propose act with query payload).
    Goal {
        /// Goal text (treated as MoL query when classified).
        text: String,
    },
    /// Session message envelope (body becomes the query).
    Message {
        /// Sender label.
        from: String,
        /// Recipient label.
        to: String,
        /// Body text.
        body: String,
    },
    /// Explicit act proposal already shaped for AutomateGate.
    ActProposal {
        /// Proposed act.
        act: Act,
    },
}

impl MailItem {
    /// Human summary for transcript / CLI.
    pub fn summary(&self) -> String {
        match self {
            Self::Goal { text } => format!("goal:{text}"),
            Self::Message { from, to, body } => format!("message:{from}->{to}:{body}"),
            Self::ActProposal { act } => format!("act:{:?}:{}", act.kind, act.summary),
        }
    }

    /// Text used as MoL query when classifying (if any).
    pub fn query_text(&self) -> Option<&str> {
        match self {
            Self::Goal { text } => Some(text.as_str()),
            Self::Message { body, .. } => Some(body.as_str()),
            Self::ActProposal { act } => act
                .payload
                .as_ref()
                .and_then(|v| v.get("query"))
                .and_then(|v| v.as_str())
                .or(Some(act.summary.as_str())),
        }
    }
}

/// One envelope in the mailbox.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MailEnvelope {
    /// Monotonic id.
    pub id: u64,
    /// Payload.
    pub item: MailItem,
    /// True after the agent loop has closed it.
    pub handled: bool,
}

/// One recorded agent-loop step (sense → classify → close → record).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTranscriptEntry {
    /// Mailbox envelope id that was sensed.
    pub mail_id: u64,
    /// Sensed item summary.
    pub sensed: String,
    /// Classified query text (if any).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub classified_query: Option<String>,
    /// Act kind used at the gate.
    pub act_kind: String,
    /// True iff AutomateGate / MoL committed.
    pub commit: bool,
    /// Binding limit id on refuse (if any).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit_id: Option<String>,
    /// Cascade answering tier label (if any).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cascade_answered: Option<String>,
    /// Replay class label (if any).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replay_class: Option<String>,
    /// True if any cascade Model step Answered (must stay false under !allow_model).
    pub model_answered: bool,
    /// Close / gate receipt.
    pub receipt: MolReceipt,
}

impl AgentTranscriptEntry {
    /// Short CLI line.
    pub fn display_line(&self) -> String {
        format!(
            "#{} {} → {} limit={:?} tier={:?} replay={:?} model_answered={}",
            self.mail_id,
            self.sensed,
            if self.commit { "COMMIT" } else { "REFUSE" },
            self.limit_id,
            self.cascade_answered,
            self.replay_class,
            self.model_answered
        )
    }
}

/// In-process agent mailbox + close transcript.
#[derive(Debug, Default)]
pub struct AgentMailbox {
    next_id: u64,
    /// Pending + handled envelopes (order preserved).
    envelopes: Vec<MailEnvelope>,
    /// Close transcript (append-only).
    transcript: Vec<AgentTranscriptEntry>,
}

impl AgentMailbox {
    /// Empty mailbox.
    pub fn new() -> Self {
        Self::default()
    }

    /// Next unused id (peek).
    pub fn next_id(&self) -> u64 {
        self.next_id
    }

    /// All envelopes.
    pub fn envelopes(&self) -> &[MailEnvelope] {
        &self.envelopes
    }

    /// Close transcript.
    pub fn transcript(&self) -> &[AgentTranscriptEntry] {
        &self.transcript
    }

    /// Count of unhandled envelopes.
    pub fn pending_count(&self) -> usize {
        self.envelopes.iter().filter(|e| !e.handled).count()
    }

    /// Post a goal.
    pub fn post_goal(&mut self, text: impl Into<String>) -> u64 {
        self.push(MailItem::Goal { text: text.into() })
    }

    /// Post a message.
    pub fn post_message(
        &mut self,
        from: impl Into<String>,
        to: impl Into<String>,
        body: impl Into<String>,
    ) -> u64 {
        self.push(MailItem::Message {
            from: from.into(),
            to: to.into(),
            body: body.into(),
        })
    }

    /// Post an explicit act proposal.
    pub fn post_act(&mut self, act: Act) -> u64 {
        self.push(MailItem::ActProposal { act })
    }

    fn push(&mut self, item: MailItem) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.envelopes.push(MailEnvelope {
            id,
            item,
            handled: false,
        });
        id
    }

    /// First unhandled envelope (sense).
    pub fn sense(&self) -> Option<&MailEnvelope> {
        self.envelopes.iter().find(|e| !e.handled)
    }

    /// Mark envelope handled and append transcript entry.
    pub fn record(&mut self, mail_id: u64, entry: AgentTranscriptEntry) -> bool {
        let Some(env) = self.envelopes.iter_mut().find(|e| e.id == mail_id) else {
            return false;
        };
        env.handled = true;
        self.transcript.push(entry);
        true
    }

    /// Clear pending + transcript (demo reset).
    pub fn clear(&mut self) {
        self.envelopes.clear();
        self.transcript.clear();
        self.next_id = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::act::{Act, ActKind};

    #[test]
    fn post_goal_and_sense() {
        let mut mb = AgentMailbox::new();
        let id = mb.post_goal("landauer joules per bit");
        assert_eq!(id, 0);
        assert_eq!(mb.pending_count(), 1);
        let sensed = mb.sense().unwrap();
        assert_eq!(sensed.id, 0);
        assert!(matches!(sensed.item, MailItem::Goal { .. }));
    }

    #[test]
    fn post_message_and_act() {
        let mut mb = AgentMailbox::new();
        mb.post_message("user", "agent", "convert 0 celsius to fahrenheit");
        mb.post_act(Act::new(ActKind::Propose, "ask", 1e-12));
        assert_eq!(mb.pending_count(), 2);
    }
}
