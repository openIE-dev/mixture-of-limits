//! Compute receipts — fabric / step / joules (estimated vs measured honesty).
//!
//! Complements cascade step records: each compute step names fabric chosen and
//! joule brackets without inventing measured_j.
//!
//! Multi-fabric law (Ferric soft-ref semantics, clean-room): inventory every
//! known fabric; schedule cheapest sufficient; stamp [`ComputeStepReceipt`] with
//! fabric id + optional unavailable reason; **never** invent `measured_j`.

use serde::{Deserialize, Serialize};

use crate::energy::{EstimateKind, Joules, MeasureSource};
use crate::meter::MeterSample;
use crate::fabric::{DeviceKind, FabricDecision, FabricInventory};
use crate::honesty::EnergyHonestyClass;

/// Stable fabric id (wire label) — usually [`DeviceKind::label`].
pub type FabricId = String;

/// Soft availability of one fabric in an inventory snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FabricAvailability {
    /// Present and eligible for routing.
    Available,
    /// Known kind but soft-unavailable (absent adapter / feature / silicon).
    SoftUnavailable {
        /// Why this fabric is unavailable (honest; never invents joules).
        reason: String,
    },
}

impl FabricAvailability {
    /// True when Available.
    pub fn is_available(&self) -> bool {
        matches!(self, Self::Available)
    }

    /// Unavailable reason if any.
    pub fn unavailable_reason(&self) -> Option<&str> {
        match self {
            Self::Available => None,
            Self::SoftUnavailable { reason } => Some(reason.as_str()),
        }
    }
}

/// One compute fabric candidate (id + kind + availability).
///
/// Soft-link to Ferric `Context` / adapter enumerate: MoL names fabrics as law
/// without path-depending Ferric crates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComputeFabric {
    /// Stable id (e.g. `cpu`, `gpu_metal`).
    pub id: FabricId,
    /// Device kind.
    pub kind: DeviceKind,
    /// Soft availability.
    pub availability: FabricAvailability,
    /// Optional honesty / detect note.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

impl ComputeFabric {
    /// Available fabric from a device kind.
    pub fn available(kind: DeviceKind) -> Self {
        Self {
            id: kind.label().into(),
            kind,
            availability: FabricAvailability::Available,
            note: None,
        }
    }

    /// Soft-unavailable fabric (adapter / silicon absent).
    pub fn soft_unavailable(kind: DeviceKind, reason: impl Into<String>) -> Self {
        Self {
            id: kind.label().into(),
            kind,
            availability: FabricAvailability::SoftUnavailable {
                reason: reason.into(),
            },
            note: None,
        }
    }

    /// With note.
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }

    /// True when available for schedule.
    pub fn is_available(&self) -> bool {
        self.availability.is_available()
    }
}

/// Schedule / route decision across fabrics (chosen or refuse).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScheduleDecision {
    /// Cheapest sufficient fabric chosen; estimated joules only offline.
    Chosen {
        /// Chosen fabric.
        fabric: ComputeFabric,
        /// Estimated joules for the route step (catalog / analytical).
        estimated_j: Joules,
    },
    /// No sufficient fabric — refuse (`fabric_unavailable`).
    Unavailable {
        /// Kind that was preferred / attempted, if known.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        requested: Option<DeviceKind>,
        /// Honest refuse reason.
        reason: String,
        /// Estimated joules for the refuse stamp (never invents measured).
        estimated_j: Joules,
    },
}

impl ScheduleDecision {
    /// Chosen fabric if any.
    pub fn chosen_fabric(&self) -> Option<&ComputeFabric> {
        match self {
            Self::Chosen { fabric, .. } => Some(fabric),
            Self::Unavailable { .. } => None,
        }
    }

    /// Chosen device kind if any.
    pub fn chosen_kind(&self) -> Option<DeviceKind> {
        self.chosen_fabric().map(|f| f.kind)
    }

    /// True on unavailable refuse.
    pub fn is_unavailable(&self) -> bool {
        matches!(self, Self::Unavailable { .. })
    }

    /// Unavailable reason if refuse.
    pub fn unavailable_reason(&self) -> Option<&str> {
        match self {
            Self::Chosen { .. } => None,
            Self::Unavailable { reason, .. } => Some(reason.as_str()),
        }
    }

    /// Estimated joules on either arm.
    pub fn estimated_j(&self) -> Joules {
        match self {
            Self::Chosen { estimated_j, .. } | Self::Unavailable { estimated_j, .. } => {
                *estimated_j
            }
        }
    }

    /// Build from legacy [`FabricDecision`] + route-step estimated joules.
    pub fn from_fabric_decision(dec: FabricDecision, estimated_j: Joules) -> Self {
        match dec {
            FabricDecision::Chosen(kind) => Self::Chosen {
                fabric: ComputeFabric::available(kind)
                    .with_note("scheduled cheapest sufficient (MoL fabric law)"),
                estimated_j,
            },
            FabricDecision::Refuse { reason } => Self::Unavailable {
                requested: None,
                reason,
                estimated_j,
            },
        }
    }

    /// Stamp a [`ComputeStepReceipt`] for this decision (`fabric:route` label).
    pub fn to_compute_step(&self) -> ComputeStepReceipt {
        ComputeStepReceipt::from_schedule("fabric:route", self)
    }
}

/// Build [`ComputeFabric`] rows from an inventory (available + soft-unavailable).
pub fn fabrics_from_inventory(inv: &FabricInventory) -> Vec<ComputeFabric> {
    DeviceKind::all()
        .into_iter()
        .filter(|k| *k != DeviceKind::Unknown)
        .map(|kind| {
            let row = inv.devices.iter().find(|d| d.kind == kind);
            let note = row.and_then(|d| d.note.clone());
            if inv.is_present(kind) {
                let mut f = ComputeFabric::available(kind);
                f.note = note.or_else(|| Some("present in inventory".into()));
                f
            } else {
                let reason = note.unwrap_or_else(|| {
                    format!(
                        "soft-unavailable: {} absent (detect optional; no invent joules)",
                        kind.label()
                    )
                });
                ComputeFabric::soft_unavailable(kind, reason)
            }
        })
        .collect()
}

/// One fabric-aware compute step for receipts / ledgers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComputeStepReceipt {
    /// Step label (e.g. `cascade:lookup`, `capsule:invoke`, `fabric:route`).
    pub label: String,
    /// Fabric chosen for this step (if any).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fabric: Option<DeviceKind>,
    /// Stable fabric id (wire) — mirrors [`ComputeFabric::id`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fabric_id: Option<FabricId>,
    /// When the step refused for fabric unavailability.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unavailable_reason: Option<String>,
    /// Estimated joules (always present when step ran).
    pub estimated_j: Joules,
    /// Kind of estimate.
    #[serde(default)]
    pub estimate_kind: EstimateKind,
    /// Measured joules when a real meter existed — never invented.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub measured_j: Option<Joules>,
    /// Measure source honesty.
    pub measure_source: MeasureSource,
    /// Derived honesty class.
    pub honesty: EnergyHonestyClass,
    /// Optional capsule id that executed this step.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capsule_id: Option<String>,
    /// Optional execution proof (checksum / output sample) when a real or soft kernel ran.
    ///
    /// Example: `checksum=0x…;n=64;sample=[0,3,6];mode=live:metal;live=true`.
    /// Never invents joules — proof of compute only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution_proof: Option<String>,
}

impl ComputeStepReceipt {
    /// Estimated-only step (offline / WASM-safe floor).
    pub fn estimated(
        label: impl Into<String>,
        fabric: Option<DeviceKind>,
        estimated_j: Joules,
    ) -> Self {
        let fabric_id = fabric.map(|k| k.label().to_string());
        Self {
            label: label.into(),
            fabric,
            fabric_id,
            unavailable_reason: None,
            estimated_j,
            estimate_kind: EstimateKind::Analytical,
            measured_j: None,
            measure_source: MeasureSource::CascadeEstimate,
            honesty: EnergyHonestyClass::Estimated,
            capsule_id: None,
            execution_proof: None,
        }
    }

    /// Stamp from a [`ScheduleDecision`] (fabric route / refuse).
    pub fn from_schedule(label: impl Into<String>, decision: &ScheduleDecision) -> Self {
        match decision {
            ScheduleDecision::Chosen {
                fabric,
                estimated_j,
            } => Self {
                label: label.into(),
                fabric: Some(fabric.kind),
                fabric_id: Some(fabric.id.clone()),
                unavailable_reason: None,
                estimated_j: *estimated_j,
                estimate_kind: EstimateKind::Analytical,
                measured_j: None,
                measure_source: MeasureSource::CascadeEstimate,
                honesty: EnergyHonestyClass::Estimated,
                capsule_id: None,
                execution_proof: None,
            },
            ScheduleDecision::Unavailable {
                requested,
                reason,
                estimated_j,
            } => Self {
                label: label.into(),
                fabric: *requested,
                fabric_id: requested.map(|k| k.label().to_string()),
                unavailable_reason: Some(reason.clone()),
                estimated_j: *estimated_j,
                estimate_kind: EstimateKind::Analytical,
                measured_j: None,
                measure_source: MeasureSource::CascadeEstimate,
                honesty: EnergyHonestyClass::Estimated,
                capsule_id: None,
                execution_proof: None,
            },
        }
    }

    /// Attach capsule id.
    pub fn with_capsule(mut self, id: impl Into<String>) -> Self {
        self.capsule_id = Some(id.into());
        self
    }

    /// Attach unavailable reason (fabric refuse).
    pub fn with_unavailable_reason(mut self, reason: impl Into<String>) -> Self {
        self.unavailable_reason = Some(reason.into());
        self
    }

    /// Attach execution proof (kernel checksum / sample). Does not set measured_j.
    pub fn with_execution_proof(mut self, proof: impl Into<String>) -> Self {
        self.execution_proof = Some(proof.into());
        self
    }

    /// Stamp a real OS meter sample onto this step (package `measured_j` only).
    ///
    /// Never invents: dishonest / empty samples leave estimated honesty unchanged.
    /// Used when SMC/RAPL overlaps a live Metal kernel window.
    pub fn apply_meter_sample(&mut self, sample: &MeterSample) {
        if sample.records_measurement() && sample.measured_j.is_some() {
            self.measured_j = sample.measured_j;
            self.measure_source = sample.source;
            self.honesty = EnergyHonestyClass::from_measure(sample.source, sample.measured_j);
        }
    }

    /// Honesty ok for this step (pair law).
    pub fn honesty_ok(&self) -> bool {
        crate::honesty::energy_pair_honest(self.measured_j, self.measure_source)
            && self.honesty
                == EnergyHonestyClass::from_measure(self.measure_source, self.measured_j)
    }

    /// True when this step records a fabric-unavailable refuse.
    pub fn is_fabric_unavailable(&self) -> bool {
        self.unavailable_reason.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::budget::Budget;
    use crate::fabric::{route_fabric, FabricInventory};
    use crate::tier::CascadeTier;

    #[test]
    fn estimated_step_honesty_ok() {
        let s =
            ComputeStepReceipt::estimated("cascade:lookup", Some(DeviceKind::Cpu), Joules::new(1e-12));
        assert!(s.honesty_ok());
        assert_eq!(s.honesty, EnergyHonestyClass::Estimated);
        assert!(s.measured_j.is_none());
        assert_eq!(s.fabric_id.as_deref(), Some("cpu"));
    }

    #[test]
    fn schedule_cpu_commit_stamps_fabric_id() {
        let inv = FabricInventory::software_ref();
        let dec = route_fabric(CascadeTier::Formula, &inv, &Budget::coin_cell());
        let sched = ScheduleDecision::from_fabric_decision(dec, Joules::new(1e-12));
        assert!(!sched.is_unavailable());
        let step = sched.to_compute_step();
        assert_eq!(step.fabric, Some(DeviceKind::Cpu));
        assert_eq!(step.fabric_id.as_deref(), Some("cpu"));
        assert!(step.unavailable_reason.is_none());
        assert!(step.measured_j.is_none());
        assert!(step.honesty_ok());
    }

    #[test]
    fn schedule_unavailable_stamps_reason_no_measured() {
        let inv = FabricInventory::software_ref(); // no Gpu*
        let dec = route_fabric(CascadeTier::Model, &inv, &Budget::demo().allow_model());
        assert!(dec.is_refuse());
        let sched = ScheduleDecision::from_fabric_decision(dec, Joules::ZERO);
        assert!(sched.is_unavailable());
        let step = sched.to_compute_step();
        assert!(step.unavailable_reason.is_some());
        assert!(step.measured_j.is_none());
        assert!(step.honesty_ok());
    }

    #[test]
    fn inventory_lists_soft_unavailable() {
        let inv = FabricInventory::software_ref();
        let fabrics = fabrics_from_inventory(&inv);
        let cpu = fabrics.iter().find(|f| f.kind == DeviceKind::Cpu).unwrap();
        assert!(cpu.is_available());
        let metal = fabrics
            .iter()
            .find(|f| f.kind == DeviceKind::GpuMetal)
            .unwrap();
        assert!(!metal.is_available());
        assert!(metal.availability.unavailable_reason().is_some());
    }
}
