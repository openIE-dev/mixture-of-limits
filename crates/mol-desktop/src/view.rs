//! Display DTOs for the energy harness UI / machine protocol lane.
//!
//! Thin, serializable views over [`mol_receipt::MolReceipt`] and fabric inventory —
//! optimized for **joules-per-verified-decision**, not token chat chrome.

use mol_core::{
    AgentLaneReceipt, ComputeStepReceipt, DeviceKind, EncapsulationReceipt, EstimateKind,
    FabricInventory, OpenIeZone, ReplayClass,
};
use mol_limits::CloseOutcome;
use mol_receipt::{CascadeStepOutcome, CascadeStepRecord, MolReceipt};
use serde::{Deserialize, Serialize};

/// Compact receipt surface for ask/close panels and protocol clients.
///
/// Honesty: `measured_j` is `Some` only when a real OS meter probe succeeded;
/// soft-ref / VM / no-permission keeps `None` (never invent).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptView {
    /// Certified commit vs refuse.
    pub commit: bool,
    /// `COMMIT` | `REFUSE`.
    pub decision: String,
    /// Original query.
    pub query: Option<String>,
    /// Answer text when committed.
    pub answer: Option<String>,
    /// OpenIE zone affinity.
    pub zone: Option<String>,
    /// Fabric chosen (cheapest sufficient device).
    pub fabric: Option<String>,
    /// Binding limit id when refuse/stop.
    pub limit: Option<String>,
    /// Limit reason when present.
    pub limit_reason: Option<String>,
    /// Cascade tier that answered.
    pub cascade_answered: Option<String>,
    /// Typed replay class.
    pub replay_class: Option<String>,
    /// Citation claim ids (cite / compose).
    pub citation_ids: Vec<String>,
    /// Compose sources when Z1 synthesis.
    pub composed_from: Vec<String>,
    /// Estimated joules (analytical / μ catalog — not RAPL).
    pub estimated_j: f64,
    /// Estimate kind label.
    pub estimate_kind: String,
    /// Catalog impedance μ when stamped.
    pub mu: Option<f64>,
    /// μ provenance (`catalog` in soft-ref).
    pub mu_source: String,
    /// Landauer floor ratio when available.
    pub landauer_floor_ratio: Option<f64>,
    /// Soft-measured joules when real probe succeeded; else `None`.
    pub measured_j: Option<f64>,
    /// How measurement was obtained.
    pub measure_source: String,
    /// Per-component measured joules (empty when the meter did not run or failed).
    pub component_measured: Vec<ComponentJouleView>,
    /// Board synth claim — always false in soft-ref.
    pub board_synth_claimed: bool,
    /// Cascade waterfall steps.
    pub cascade_steps: Vec<CascadeStepView>,
    /// Router / certify rationale.
    pub rationale: String,
    /// Receipt id.
    pub receipt_id: String,
    /// Energy honesty class: estimated / measured / modeled / unavailable.
    pub energy_honesty: String,
    /// Encapsulation stamp when a capsule was attached (missing → None / refuse stamp).
    pub encapsulation: Option<EncapsulationView>,
    /// Agent Lane isolation stamp when a policy was attached.
    pub agent_lane: Option<AgentLaneView>,
    /// Fabric/step/joules compute receipts (estimated vs measured honesty).
    pub compute_steps: Vec<ComputeStepView>,
}

/// One cascade step for the energy harness waterfall panel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CascadeStepView {
    /// Tier label.
    pub tier: String,
    /// Soft elapsed microseconds.
    pub us: u64,
    /// Outcome wire label.
    pub outcome: String,
    /// Surrogate joules for this step.
    pub estimated_j: f64,
}

fn step_outcome_label(o: &CascadeStepOutcome) -> String {
    match o {
        CascadeStepOutcome::Answered => "answered".into(),
        CascadeStepOutcome::Miss => "miss".into(),
        CascadeStepOutcome::Refused => "refused".into(),
        CascadeStepOutcome::Skipped => "skipped".into(),
    }
}

impl From<&CascadeStepRecord> for CascadeStepView {
    fn from(s: &CascadeStepRecord) -> Self {
        Self {
            tier: s.tier.label().to_string(),
            us: s.us,
            outcome: step_outcome_label(&s.outcome),
            estimated_j: s.estimated_j.0,
        }
    }
}

/// Encapsulation / WASM capsule surface on a receipt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncapsulationView {
    /// Capsule id.
    pub capsule_id: String,
    /// Boundary label (`wasm_component`, `native_host`, …).
    pub boundary: String,
    /// Sealed imports/exports.
    pub sealed: bool,
    /// Anti-pattern: shared host session.
    pub shares_host_session: bool,
}

impl From<&EncapsulationReceipt> for EncapsulationView {
    fn from(e: &EncapsulationReceipt) -> Self {
        Self {
            capsule_id: e.capsule_id.clone(),
            boundary: e.boundary.label().to_string(),
            sealed: e.sealed,
            shares_host_session: e.shares_host_session,
        }
    }
}

/// Agent Lane isolation surface on a receipt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentLaneView {
    /// `human` | `agent`.
    pub lane: String,
    /// Separate profile root.
    pub separate_profile: bool,
    /// Separate cookie jar.
    pub separate_cookies: bool,
    /// Separate web storage (may be false — honest gap).
    pub separate_storage: bool,
    /// Opaque-bot share (must be false for commit).
    pub allow_shared_opaque_bot: bool,
}

impl From<&AgentLaneReceipt> for AgentLaneView {
    fn from(a: &AgentLaneReceipt) -> Self {
        Self {
            lane: a.lane.label().to_string(),
            separate_profile: a.separate_profile,
            separate_cookies: a.separate_cookies,
            separate_storage: a.separate_storage,
            allow_shared_opaque_bot: a.allow_shared_opaque_bot,
        }
    }
}

/// One compute-step receipt for the harness (estimated vs measured labels).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeStepView {
    /// Step label (`fabric:route`, `kernel:vector_add`, `capsule:invoke`, …).
    pub label: String,
    /// Fabric chosen.
    pub fabric: Option<String>,
    /// Stable fabric id wire (`cpu`, `gpu_metal`, …) — mirrors ComputeStepReceipt.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fabric_id: Option<String>,
    /// When the step refused for fabric unavailability.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unavailable_reason: Option<String>,
    /// Estimated joules.
    pub estimated_j: f64,
    /// Estimate kind.
    pub estimate_kind: String,
    /// Measured joules when real meter — never invented.
    pub measured_j: Option<f64>,
    /// Measure source label.
    pub measure_source: String,
    /// Honesty class label.
    pub honesty: String,
    /// Capsule id when step ran under encapsulation.
    pub capsule_id: Option<String>,
    /// Optional kernel execution proof (checksum / sample).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution_proof: Option<String>,
}

impl From<&ComputeStepReceipt> for ComputeStepView {
    fn from(s: &ComputeStepReceipt) -> Self {
        Self {
            label: s.label.clone(),
            fabric: s.fabric.map(|d| d.label().to_string()),
            fabric_id: s.fabric_id.clone(),
            unavailable_reason: s.unavailable_reason.clone(),
            estimated_j: s.estimated_j.0,
            estimate_kind: estimate_kind_label(s.estimate_kind),
            measured_j: s.measured_j.map(|j| j.0),
            measure_source: s.measure_source.label().to_string(),
            honesty: s.honesty.label().to_string(),
            capsule_id: s.capsule_id.clone(),
            execution_proof: s.execution_proof.clone(),
        }
    }
}

/// One measured component rail on a receipt view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentJouleView {
    /// `cpu` / `gpu` / `ane` / `dram` / `package`.
    pub component: String,
    /// Joules over the meter window.
    pub joules: f64,
    /// Probe label (`powermetrics`, `ioreport`, `rapl`, …).
    pub measure_source: String,
}

impl ReceiptView {
    /// Build from a close outcome (commit|refuse + receipt).
    pub fn from_close(out: &CloseOutcome) -> Self {
        Self::from_receipt(out.is_commit(), out.receipt())
    }

    /// Build from commit flag + receipt.
    pub fn from_receipt(commit: bool, r: &MolReceipt) -> Self {
        Self {
            commit,
            decision: if commit {
                "COMMIT".into()
            } else {
                "REFUSE".into()
            },
            query: r.query.clone(),
            answer: r.answer.clone(),
            zone: r.zone.map(|z: OpenIeZone| z.label().to_string()),
            fabric: r.fabric_chosen.map(|d: DeviceKind| d.label().to_string()),
            limit: r.limit_fired.as_ref().map(|f| f.id.as_str().to_string()),
            limit_reason: r.limit_fired.as_ref().map(|f| f.reason.clone()),
            cascade_answered: r.cascade_answered.map(|t| t.label().to_string()),
            replay_class: r.replay_class.map(|c: ReplayClass| c.label().to_string()),
            citation_ids: r.citation_ids.clone(),
            composed_from: r.composed_from.clone(),
            estimated_j: r.estimated_j.0,
            estimate_kind: estimate_kind_label(r.estimate_kind),
            mu: r.mu,
            mu_source: r.mu_source.label().to_string(),
            landauer_floor_ratio: r.landauer_floor_ratio,
            measured_j: r.measured_j.map(|j| j.0),
            measure_source: r.measure_source.label().to_string(),
            component_measured: r
                .component_measured
                .iter()
                .map(|c| ComponentJouleView {
                    component: c.component.label().to_string(),
                    joules: c.joules.0,
                    measure_source: c.measure_source.label().to_string(),
                })
                .collect(),
            board_synth_claimed: r.board_synth_claimed,
            cascade_steps: r.cascade_steps.iter().map(CascadeStepView::from).collect(),
            rationale: r.rationale.clone(),
            receipt_id: r.id.clone(),
            energy_honesty: r.energy_honesty.label().to_string(),
            encapsulation: r.encapsulation.as_ref().map(EncapsulationView::from),
            agent_lane: r.agent_lane.as_ref().map(AgentLaneView::from),
            compute_steps: r.compute_steps.iter().map(ComputeStepView::from).collect(),
        }
    }

    /// Soft-ref honesty: measured absent and board_synth false.
    /// When a real meter stamped `Some`, still requires board_synth=false and labeled source.
    pub fn honesty_ok(&self) -> bool {
        if self.board_synth_claimed {
            return false;
        }
        if self.component_measured.iter().any(|c| {
            !c.joules.is_finite()
                || c.joules < 0.0
                || !matches!(
                    c.measure_source.as_str(),
                    "rapl" | "nvml" | "cpu_proxy" | "powermetrics" | "ioreport" | "smc" | "macos_energy" | "windows_energy"
                )
        }) {
            return false;
        }
        match self.measured_j {
            None => true,
            Some(_) => matches!(
                self.measure_source.as_str(),
                "rapl"
                    | "nvml"
                    | "cpu_proxy"
                    | "powermetrics"
                    | "ioreport"
                    | "smc"
                    | "macos_energy"
                    | "windows_energy"
            ),
        }
    }

    /// One-line harness summary: decision + joules + fabric + limit.
    pub fn harness_line(&self) -> String {
        let enc = self
            .encapsulation
            .as_ref()
            .map(|e| {
                format!(
                    "{}:{}{}",
                    e.boundary,
                    if e.sealed { "sealed" } else { "unsealed" },
                    if e.shares_host_session { "+host_share" } else { "" }
                )
            })
            .unwrap_or_else(|| "missing".into());
        let lane = self
            .agent_lane
            .as_ref()
            .map(|a| {
                format!(
                    "{}:cookies={}{}",
                    a.lane,
                    a.separate_cookies,
                    if a.allow_shared_opaque_bot { "+opaque_bot" } else { "" }
                )
            })
            .unwrap_or_else(|| "missing".into());
        format!(
            "{}  honesty={}  est_j={:.3e}  measured_j={}  landauer_ratio={}  fabric={}  zone={}  limit={}  enc={}  agent={}  compute_steps={}",
            self.decision,
            self.energy_honesty,
            self.estimated_j,
            self.measured_j
                .map(|j| format!("{j:.3e}"))
                .unwrap_or_else(|| "None".into()),
            self.landauer_floor_ratio
                .map(|r| format!("{r:.3e}"))
                .unwrap_or_else(|| "n/a".into()),
            self.fabric.as_deref().unwrap_or("-"),
            self.zone.as_deref().unwrap_or("-"),
            self.limit.as_deref().unwrap_or("-"),
            enc,
            lane,
            self.compute_steps.len(),
        )
    }
}

fn estimate_kind_label(k: EstimateKind) -> String {
    match k {
        EstimateKind::Analytical => "analytical".into(),
        EstimateKind::Calib => "calib".into(),
        EstimateKind::Fixture => "fixture".into(),
    }
}

/// Fabric inventory panel DTO.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FabricView {
    /// Inventory source label.
    pub source: String,
    /// Present device kinds.
    pub present: Vec<String>,
    /// Full inventory summary string.
    pub summary: String,
    /// Honesty note: detect ≠ measured joules.
    pub honesty_note: String,
}

impl FabricView {
    /// From inventory.
    pub fn from_inventory(inv: &FabricInventory) -> Self {
        Self {
            source: inv.source.label().to_string(),
            present: inv
                .present_kinds()
                .into_iter()
                .map(|k| k.label().to_string())
                .collect(),
            summary: inv.summary(),
            honesty_note: "detect ≠ measured joules; measured_j only when OS meter probe succeeds"
                .into(),
        }
    }
}

/// One row in the session joule ledger (every close is a verified decision).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JouleLedgerEntry {
    /// Sequence index (0-based).
    pub seq: u64,
    /// Query text.
    pub query: String,
    /// COMMIT / REFUSE.
    pub decision: String,
    /// Estimated joules for this act.
    pub estimated_j: f64,
    /// Landauer ratio when present.
    pub landauer_floor_ratio: Option<f64>,
    /// Running sum of estimated_j in this session.
    pub cumulative_estimated_j: f64,
    /// Fabric chosen.
    pub fabric: Option<String>,
    /// Limit id if refuse.
    pub limit: Option<String>,
    /// Receipt id.
    pub receipt_id: String,
    /// Soft-measured when probe succeeded.
    pub measured_j: Option<f64>,
    /// Energy honesty class label.
    pub energy_honesty: String,
    /// Encapsulation present / sealed / missing stamp.
    pub encapsulation: String,
    /// Agent lane stamp (`missing` | lane summary).
    pub agent_lane: String,
    /// Number of compute_steps on the receipt.
    pub compute_steps: usize,
}

/// Session-level ledger summary (joules-per-verified-decision surface).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JouleLedgerSummary {
    /// Number of close acts.
    pub acts: u64,
    /// Commits.
    pub commits: u64,
    /// Refuses.
    pub refuses: u64,
    /// Sum of estimated_j.
    pub total_estimated_j: f64,
    /// Mean estimated_j per act (0 if empty).
    pub mean_estimated_j: f64,
    /// Mean landauer ratio over acts that stamped one.
    pub mean_landauer_ratio: Option<f64>,
    /// True when every act with measured_j=None used unavailable/cascade sources,
    /// or every Some used a labeled measured source (never invented).
    pub measured_honest: bool,
}

/// Human confirm request for consequential acts (interface tax).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmRequest {
    /// Short act label.
    pub act: String,
    /// Why confirm is required.
    pub reason: String,
    /// Estimated joules if known.
    pub estimated_j: Option<f64>,
    /// Capability that would be exercised.
    pub capability: Option<String>,
}

/// Operator confirm response.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfirmResponse {
    /// Human approved consequential act.
    Approve,
    /// Human refused — MoL stays refuse-over-escalate.
    Refuse,
}

/// True when estimate kind is an explicit labeled variant.
pub fn estimate_is_labeled(kind: EstimateKind) -> bool {
    matches!(
        kind,
        EstimateKind::Analytical | EstimateKind::Calib | EstimateKind::Fixture
    )
}

