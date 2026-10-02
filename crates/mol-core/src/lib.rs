//! Mixture of Limits — core substrate types.
//!
//! Thesis (OpenIE / David Charlot): **Mixture of Limits** is the universal law of
//! computer intelligence — floors where more bits stop buying outcomes. Mathematical
//! compression / formulas beat excess tokens. Neural nets are a flawed default
//! construct for CI; navigation law over the Periodic Stack, not MoE inside the
//! generative corridor.
//!
//! Proof law: `E(x) ≥ θ(D)·μ(S,V)`.
//! MathGround cascade: Lookup → Closed-form → Sparse solver → Stochastic model LAST.
//!
//! Honesty: estimated joules ≠ measured RAPL/NVML; never invent sensor readings.
//! Ecosystem law: encapsulated WASM/capsules, energy+compute receipts, Agent Lane
//! isolation, fail-closed when provenance or meter required but missing.

#![deny(missing_docs)]

mod budget;
mod claims;
mod completeness;
mod episode;
mod phase1;
mod shunt;
mod capsule_runtime;
mod compute;
mod encapsulation;
mod error;
mod energy;
mod fabric;
mod kernel;
mod floor;
mod honesty;
mod agent_lane;
mod landauer;
mod meter;
mod memory;
mod mu;
mod replay;
mod request;
mod stack;
mod tier;
mod voi;

pub use budget::{Budget, LatencyBudget};
pub use agent_lane::{
    host_invoke, AgentIsolationPolicy, AgentLaneReceipt, AgentLaneSession, AppLane,
    GrantReceipt, HostCapability, HostInvokeDecision, HostInvokeRequest, KeywordConfirm,
    LanePartition, LaneProvenance, PartitionSurface,
};
pub use compute::{
    fabrics_from_inventory, ComputeFabric, ComputeStepReceipt, FabricAvailability, FabricId,
    ScheduleDecision,
};
pub use capsule_runtime::{
    certify_sealed_add_fixture, default_capsule_runtime, CapsuleBounds, CapsuleCertifyResult,
    CapsuleDecision, CapsuleGrant, CapsuleInvoke, CapsuleRuntime, StubCapsuleRuntime,
    FIXTURE_ADD_WASM, JOULES_PER_FUEL,
};
#[cfg(feature = "wasmtime")]
pub use capsule_runtime::{wasmtime_capsule_runtime, WasmtimeCapsuleRuntime};
pub use encapsulation::{
    CapsuleBoundary, CapsuleContext, CapsuleId, EncapsulationReceipt,
};
pub use honesty::{
    energy_pair_honest, require_measured_or_refuse, require_provenance_or_refuse,
    EnergyHonestyClass,
};
pub use completeness::{CompletenessClause, CompletenessSnapshot};
pub use episode::{EpisodeState, EpisodeStatus, EpisodeStore};
pub use phase1::{run_phase1, transduce, Phase1Config, Phase1Outcome, TypedAst};
pub use shunt::{FixtureShuntHal, ShuntCapability, ShuntHal, ShuntReading, StubShuntHal};
pub use claims::{
    looks_compose_ask, looks_factual_ask, parse_compose_requirements, seed_compose_recipes,
    seed_demo_claims, AxisLite, ClaimHit, ClaimId, ClaimStore, ClaimVersion, ComposeRecipe,
    ComposeResolve, KnowledgeClaim,
};
pub use energy::{EstimateKind, Joules, MeasureSource};
pub use fabric::{
    detect_adapter_probes, detect_inventory, inventory_for_schedule, route_fabric, schedule_fabric,
    AdapterBackendHint, AdapterDeviceClass, AdapterProbe, DeviceKind, DevicePresence,
    FabricDecision, FabricInventory, FabricTarget, InventorySource, DETECT_HONESTY_NOTE,
    FABRIC_DETECT_ENABLED,
};
pub use kernel::{
    checksum_f32, run_tiny_vector_add, run_tiny_vector_add_if_gpu, skip_vector_add, soft_vector_add,
    vector_add_reference, KernelMode, KernelRunResult, KERNEL_ESTIMATED_J, KERNEL_HONESTY_NOTE,
    TINY_VECTOR_ADD_N, WGPU_KERNEL_ENABLED,
};
pub use error::{MolError, Result};
pub use floor::{Floor, FloorKind, LimitId};
pub use memory::{
    looks_recall_ask, looks_remember_ask, parse_recall_key, parse_remember,
    BitemporalStore, MemoryFact, MemoryId, MemoryWriteProposal,
};
pub use landauer::{
    landauer_floor_from_bits, Landauer, LandauerFloor, BOLTZMANN_J_PER_K, LN2,
    ROOM_TEMPERATURE_KELVIN,
};
pub use meter::{
    measure_energy_during, measure_energy_window, meter_status_line, parse_powermetrics_output,
    parse_nvidia_smi_power_csv, probe_meter_capability, probe_nvml_capability, sample_from_nvml_energy_mj,
    sample_from_nvml_watts, sample_from_rapl_counters, sample_from_smc_pstr_watts, sample_nvml,
    ComponentJoules, MeterCapability, MeterComponent, MeterSample, RaplCounter,
    ENERGY_METER_ENABLED, MACOS_METER_HELP, METER_HONESTY_NOTE,
};
pub use mu::{
    impedance_mismatch_energy, ImpedanceEstimate, MuCatalog, MuSource, DEFAULT_THETA_BITS,
};
pub use replay::{
    Composed, Deterministic, ModelGenerated, ReplayClass, ReplayMarker, RetrievedCited,
    TypedAnswer,
};
pub use request::{FailClosedPolicy, MolRequest, QueryKind};
pub use stack::{
    CellStatus, GearKind, PeriodicStack, ProbeResult, StackFamily, StackPrimitive,
    FULL_TARGET_FAMILIES, FULL_TARGET_PRIMITIVES,
};
pub use tier::{CascadeTier, OpenIeZone, ThermoClass};
pub use voi::{ValueOfInformation, VoiDecision};

/// Crate version string.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Always false — software reference only (matches WCA / leapfrog honesty).
pub const BOARD_SYNTH_CLAIMED: bool = false;
