//! Receipt structure.

use chrono::{DateTime, Utc};
use mol_core::{
    landauer_floor_from_bits, AgentLaneReceipt, BOARD_SYNTH_CLAIMED, Budget, CascadeTier,
    ComponentJoules, ComputeStepReceipt, DeviceKind, EncapsulationReceipt, EnergyHonestyClass,
    EstimateKind, FabricInventory, Floor, Joules, MeasureSource, MeterSample, MuSource,
    OpenIeZone, ReplayClass,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Outcome of one cascade tier attempt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CascadeStepOutcome {
    /// Tier answered.
    Answered,
    /// Grammar miss — escalate.
    Miss,
    /// Explicit refuse (budget / model / safety).
    Refused,
    /// Skipped (e.g. model without allow_model).
    Skipped,
}

/// One tier attempt in the cascade waterfall (soft wall-clock; not RAPL).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CascadeStepRecord {
    /// Tier attempted.
    pub tier: CascadeTier,
    /// Soft elapsed microseconds for this attempt.
    pub us: u64,
    /// Outcome.
    pub outcome: CascadeStepOutcome,
    /// Surrogate joules charged for this step (estimate).
    pub estimated_j: Joules,
}

/// Synthesis / compose receipt fields (Z1 fuse of ≥2 cited claims).
///
/// Present when [`ReplayClass::Composed`] answered via claim composition — never
/// omit `composed_from` or launder as Deterministic / RetrievedCited alone.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SynthesisReceipt {
    /// Source claim ids fused into the answer (≥2).
    pub composed_from: Vec<String>,
    /// Number of claims composed.
    pub claim_count: u32,
    /// Synthesis kind label (`compose`).
    pub kind: String,
}

impl SynthesisReceipt {
    /// Build from composed-from claim ids.
    pub fn composed(composed_from: Vec<String>) -> Self {
        let claim_count = composed_from.len() as u32;
        Self {
            composed_from,
            claim_count,
            kind: "compose".into(),
        }
    }
}

/// Emitted for every MoL ask / automate decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MolReceipt {
    /// Stable receipt id.
    pub id: String,
    /// Schema tag.
    pub schema: String,
    /// Original query (when captured).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    /// Tier that answered (if any).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cascade_answered: Option<CascadeTier>,
    /// OpenIE zone affinity of the answering tier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zone: Option<OpenIeZone>,
    /// Replay class of the answer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replay_class: Option<ReplayClass>,
    /// Answer text / payload summary.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub answer: Option<String>,
    /// Citation claim ids when ReplayClass is RetrievedCited or Composed.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub citation_ids: Vec<String>,
    /// Z1 compose/synthesis receipt (composed_from, claim_count, kind).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub synthesis: Option<SynthesisReceipt>,
    /// Convenience mirror of `synthesis.composed_from` when present.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub composed_from: Vec<String>,
    /// Estimated joules (analytical / cascade table — not RAPL).
    pub estimated_j: Joules,
    /// Kind of estimate.
    pub estimate_kind: EstimateKind,
    /// Catalog impedance μ used for `E ≈ θ·μ` (when estimated via μ catalog).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mu: Option<f64>,
    /// Provenance of μ / estimated_j — soft-ref always `catalog` (not RAPL).
    #[serde(default)]
    pub mu_source: MuSource,
    /// Landauer θ joules (`bits × k_B T ln2`) when annotated.
    #[serde(rename = "theta_J", default, skip_serializing_if = "Option::is_none")]
    pub theta_j: Option<Joules>,
    /// Package/combined joules when a real probe succeeded (`None` otherwise — never faked).
    ///
    /// Not a sum of component rails. Per-rail numbers live in [`Self::component_measured`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub measured_j: Option<Joules>,
    /// How measurement was obtained (`unavailable` / `rapl` / `powermetrics` / `ioreport` / …).
    pub measure_source: MeasureSource,
    /// Per-component measured joules (CPU, GPU, ANE, DRAM, package) when a meter read them.
    ///
    /// Empty in software-ref and when the probe failed. A missing component is omitted, not zero.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub component_measured: Vec<ComponentJoules>,
    /// Binding floor / limit that fired (if refuse or stop).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit_fired: Option<Floor>,
    /// Cascade waterfall.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cascade_steps: Vec<CascadeStepRecord>,
    /// Budget snapshot.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub budget: Option<Budget>,
    /// Landauer floor joules (estimate).
    #[serde(rename = "landauer_floor_J", default, skip_serializing_if = "Option::is_none")]
    pub landauer_floor_j: Option<Joules>,
    /// Bits for Landauer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub landauer_bits: Option<f64>,
    /// `estimated_j / landauer_floor_J` when both positive (≈ catalog μ).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub landauer_floor_ratio: Option<f64>,
    /// Fabric chosen for the answering (or attempted) tier — cheapest sufficient device.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fabric_chosen: Option<DeviceKind>,
    /// Snapshot of available-device inventory at close (Cpu always in soft-ref).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fabric_inventory: Option<FabricInventory>,
    /// Derived energy honesty class (estimated / measured / modeled / unavailable).
    #[serde(default)]
    pub energy_honesty: EnergyHonestyClass,
    /// Encapsulation / WASM capsule receipt when a capsule context was attached.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encapsulation: Option<EncapsulationReceipt>,
    /// Agent Lane isolation receipt when an agent policy was attached.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_lane: Option<AgentLaneReceipt>,
    /// Fabric/step/joules compute receipts (complement cascade_steps).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub compute_steps: Vec<ComputeStepReceipt>,
    /// Always false — software reference.
    pub board_synth_claimed: bool,
    /// True iff an irreversible side effect ran (automate).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub executed: Option<bool>,
    /// UTC timestamp.
    pub timestamp: DateTime<Utc>,
    /// Classifier / router rationale.
    pub rationale: String,
    /// Optional HMAC hex signature over canonical body.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature_hex: Option<String>,
    /// Signature algorithm label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature_alg: Option<String>,
}

impl MolReceipt {
    /// Schema version string.
    pub const SCHEMA: &'static str = "mol.receipt.v1";

    /// Refresh [`Self::energy_honesty`] from measure_source + measured_j (pair law).
    pub fn refresh_energy_honesty(&mut self) {
        self.energy_honesty =
            EnergyHonestyClass::from_measure(self.measure_source, self.measured_j);
    }

    /// True when measured_j / measure_source pair is honest (never invent).
    pub fn energy_honesty_ok(&self) -> bool {
        mol_core::energy_pair_honest(self.measured_j, self.measure_source)
    }

    /// Annotate Landauer floor from inferred bit erasures and refresh floor ratio.
    pub fn annotate_landauer(&mut self, bits: f64) {
        let floor = landauer_floor_from_bits(bits);
        if floor.bits <= 0.0 || !floor.floor_j.is_finite() || floor.floor_j <= 0.0 {
            self.landauer_floor_j = None;
            self.landauer_bits = None;
            self.landauer_floor_ratio = None;
            self.theta_j = None;
            return;
        }
        self.landauer_floor_j = Some(Joules::new(floor.floor_j));
        self.landauer_bits = Some(floor.bits);
        self.theta_j = Some(Joules::new(floor.floor_j));
        self.refresh_landauer_ratio();
    }

    /// Stamp an OS meter sample.
    ///
    /// On success: `measured_j` (package only), `measure_source`, and `component_measured`.
    /// On failure: `measured_j=None`, `measure_source=unavailable`, components cleared.
    /// Never invents a number the sample does not carry.
    pub fn apply_meter_sample(&mut self, sample: &MeterSample) {
        if sample.records_measurement() {
            self.measured_j = sample.measured_j;
            self.measure_source = sample.source;
            self.component_measured = sample.components.clone();
        } else {
            self.measured_j = None;
            self.measure_source = MeasureSource::Unavailable;
            self.component_measured.clear();
        }
        self.refresh_energy_honesty();
    }

    /// Recompute `landauer_floor_ratio = estimated_j / landauer_floor_J` when possible.
    pub fn refresh_landauer_ratio(&mut self) {
        match self.landauer_floor_j {
            Some(floor) if floor.0 > 0.0 && floor.0.is_finite() && self.estimated_j.is_valid() => {
                self.landauer_floor_ratio = Some(self.estimated_j.0 / floor.0);
            }
            _ => {
                self.landauer_floor_ratio = None;
            }
        }
    }

    /// JSON serialize.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Parse JSON.
    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }
}

/// Builder for [`MolReceipt`].
#[derive(Debug, Default)]
pub struct ReceiptBuilder {
    query: Option<String>,
    cascade_answered: Option<CascadeTier>,
    zone: Option<OpenIeZone>,
    replay_class: Option<ReplayClass>,
    answer: Option<String>,
    citation_ids: Vec<String>,
    synthesis: Option<SynthesisReceipt>,
    composed_from: Vec<String>,
    estimated_j: Joules,
    estimate_kind: EstimateKind,
    mu: Option<f64>,
    mu_source: MuSource,
    measured_j: Option<Joules>,
    measure_source: MeasureSource,
    component_measured: Vec<ComponentJoules>,
    limit_fired: Option<Floor>,
    cascade_steps: Vec<CascadeStepRecord>,
    budget: Option<Budget>,
    landauer_bits: Option<f64>,
    fabric_chosen: Option<DeviceKind>,
    fabric_inventory: Option<FabricInventory>,
    energy_honesty: EnergyHonestyClass,
    encapsulation: Option<EncapsulationReceipt>,
    agent_lane: Option<AgentLaneReceipt>,
    compute_steps: Vec<ComputeStepReceipt>,
    executed: Option<bool>,
    rationale: String,
}

impl ReceiptBuilder {
    /// Start building.
    pub fn new() -> Self {
        Self {
            estimated_j: Joules::ZERO,
            estimate_kind: EstimateKind::Analytical,
            mu: None,
            mu_source: MuSource::Catalog,
            measure_source: MeasureSource::Unavailable,
            component_measured: Vec::new(),
            energy_honesty: EnergyHonestyClass::Estimated,
            encapsulation: None,
            agent_lane: None,
            compute_steps: Vec::new(),
            rationale: String::new(),
            ..Default::default()
        }
    }

    /// Set query.
    pub fn query(mut self, q: impl Into<String>) -> Self {
        self.query = Some(q.into());
        self
    }

    /// Set answering tier.
    pub fn answered(mut self, tier: CascadeTier) -> Self {
        self.cascade_answered = Some(tier);
        self.zone = Some(tier.openie_zone());
        self.replay_class = Some(tier.default_replay());
        self
    }

    /// Set answer text.
    pub fn answer(mut self, a: impl Into<String>) -> Self {
        self.answer = Some(a.into());
        self
    }

    /// Set estimated joules.
    pub fn estimated_j(mut self, j: Joules) -> Self {
        self.estimated_j = j;
        self
    }

    /// Estimate kind.
    pub fn estimate_kind(mut self, k: EstimateKind) -> Self {
        self.estimate_kind = k;
        self
    }

    /// Catalog impedance μ + source (soft-ref: `MuSource::Catalog`).
    pub fn mu(mut self, mu: f64, source: MuSource) -> Self {
        self.mu = Some(mu);
        self.mu_source = source;
        self
    }

    /// Mu source only.
    pub fn mu_source(mut self, source: MuSource) -> Self {
        self.mu_source = source;
        self
    }

    /// Soft measured (only when real).
    pub fn measured(mut self, j: Joules, src: MeasureSource) -> Self {
        debug_assert!(
            src.is_measured() || src == MeasureSource::Unavailable,
            "do not label estimates as measured"
        );
        self.measured_j = Some(j);
        self.measure_source = src;
        self
    }

    /// Measure source without a value (unavailable / estimate path).
    pub fn measure_source(mut self, src: MeasureSource) -> Self {
        self.measure_source = src;
        self
    }

    /// Per-component measured joules (only real probe rows).
    pub fn component_measured(mut self, rows: Vec<ComponentJoules>) -> Self {
        self.component_measured = rows;
        self
    }

    /// Limit that fired.
    pub fn limit_fired(mut self, floor: Floor) -> Self {
        self.limit_fired = Some(floor);
        self
    }

    /// Push cascade step.
    pub fn step(mut self, step: CascadeStepRecord) -> Self {
        self.cascade_steps.push(step);
        self
    }

    /// Steps from vec.
    pub fn steps(mut self, steps: Vec<CascadeStepRecord>) -> Self {
        self.cascade_steps = steps;
        self
    }

    /// Budget snapshot.
    pub fn budget(mut self, b: Budget) -> Self {
        self.budget = Some(b);
        self
    }

    /// Landauer bits.
    pub fn landauer_bits(mut self, bits: f64) -> Self {
        self.landauer_bits = Some(bits);
        self
    }

    /// Executed flag.
    pub fn executed(mut self, e: bool) -> Self {
        self.executed = Some(e);
        self
    }

    /// Rationale.
    pub fn rationale(mut self, r: impl Into<String>) -> Self {
        self.rationale = r.into();
        self
    }

    /// OpenIE zone override (e.g. Z2 for retrieve+cite).
    pub fn zone(mut self, z: OpenIeZone) -> Self {
        self.zone = Some(z);
        self
    }

    /// Citation claim ids (Z2 RetrievedCited / Z1 Composed).
    pub fn citations(mut self, ids: Vec<String>) -> Self {
        self.citation_ids = ids;
        self
    }

    /// Stamp Z1 compose/synthesis from claim ids (≥2).
    pub fn synthesis_composed(mut self, composed_from: Vec<String>) -> Self {
        self.composed_from = composed_from.clone();
        self.synthesis = Some(SynthesisReceipt::composed(composed_from));
        self
    }

    /// Full synthesis receipt.
    pub fn synthesis(mut self, s: SynthesisReceipt) -> Self {
        self.composed_from = s.composed_from.clone();
        self.synthesis = Some(s);
        self
    }

    /// Replay class override.
    pub fn replay_class(mut self, c: ReplayClass) -> Self {
        self.replay_class = Some(c);
        self
    }


    /// Energy honesty class.
    pub fn energy_honesty(mut self, h: EnergyHonestyClass) -> Self {
        self.energy_honesty = h;
        self
    }

    /// Encapsulation receipt.
    pub fn encapsulation(mut self, e: EncapsulationReceipt) -> Self {
        self.encapsulation = Some(e);
        self
    }

    /// Agent lane receipt.
    pub fn agent_lane(mut self, a: AgentLaneReceipt) -> Self {
        self.agent_lane = Some(a);
        self
    }

    /// Compute step receipts.
    pub fn compute_steps(mut self, steps: Vec<ComputeStepReceipt>) -> Self {
        self.compute_steps = steps;
        self
    }

    /// Push one compute step.
    pub fn compute_step(mut self, step: ComputeStepReceipt) -> Self {
        self.compute_steps.push(step);
        self
    }

    /// Fabric chosen + inventory snapshot (multi-fabric law).
    pub fn fabric(mut self, chosen: Option<DeviceKind>, inventory: FabricInventory) -> Self {
        self.fabric_chosen = chosen;
        self.fabric_inventory = Some(inventory);
        self
    }

    /// Fabric inventory only (e.g. refuse before a fabric was chosen).
    pub fn fabric_inventory(mut self, inventory: FabricInventory) -> Self {
        self.fabric_inventory = Some(inventory);
        self
    }

    /// Chosen fabric only.
    pub fn fabric_chosen(mut self, kind: DeviceKind) -> Self {
        self.fabric_chosen = Some(kind);
        self
    }

    /// Build unsigned receipt.
    pub fn build(self) -> MolReceipt {
        let mut r = MolReceipt {
            id: Uuid::new_v4().to_string(),
            schema: MolReceipt::SCHEMA.into(),
            query: self.query,
            cascade_answered: self.cascade_answered,
            zone: self.zone,
            replay_class: self.replay_class,
            answer: self.answer,
            citation_ids: self.citation_ids,
            synthesis: self.synthesis,
            composed_from: self.composed_from,
            estimated_j: self.estimated_j,
            estimate_kind: self.estimate_kind,
            mu: self.mu,
            mu_source: self.mu_source,
            theta_j: None,
            measured_j: self.measured_j,
            measure_source: self.measure_source,
            component_measured: self.component_measured,
            limit_fired: self.limit_fired,
            cascade_steps: self.cascade_steps,
            budget: self.budget,
            landauer_floor_j: None,
            landauer_bits: None,
            landauer_floor_ratio: None,
            fabric_chosen: self.fabric_chosen,
            fabric_inventory: self.fabric_inventory,
            energy_honesty: self.energy_honesty,
            encapsulation: self.encapsulation,
            agent_lane: self.agent_lane,
            compute_steps: self.compute_steps,
            board_synth_claimed: BOARD_SYNTH_CLAIMED,
            executed: self.executed,
            timestamp: Utc::now(),
            rationale: self.rationale,
            signature_hex: None,
            signature_alg: None,
        };
        if let Some(bits) = self.landauer_bits {
            r.annotate_landauer(bits);
        }
        r.refresh_energy_honesty();
        r
    }
}

