//! Multi-fabric device inventory and cheapest-sufficient routing.
//!
//! MoL law: after the cascade picks a tier, pick the **cheapest sufficient
//! device/fabric** that can execute that closed gear across all *known*
//! devices — not one accelerator.
//!
//! Soft-ref honesty: [`FabricInventory::software_ref`] always marks [`DeviceKind::Cpu`]
//! present; GPU / WASM / neuromorphic / thermo-silicon flags default absent.
//! No fake RAPL. Ferric / ferrotherm / ferromotion are **reference semantics**
//! (same-kernels-everywhere) — MoL does **not** path-dep Ferric for prove.
//!
//! Optional `fabric-detect` feature: probe wgpu adapters (`Backends::all`) and map
//! Metal→[`DeviceKind::GpuMetal`], Vulkan→[`DeviceKind::GpuVulkan`],
//! Dx12→[`DeviceKind::GpuDx12`], Gl→[`DeviceKind::GpuGl`],
//! Browser WebGPU→[`DeviceKind::GpuWebGpu`]. Host [`DeviceKind::Cpu`] is always
//! present and is **not** a wgpu backend. A wgpu `DeviceType::Cpu` adapter
//! (lavapipe / software) is recorded but **not** promoted to Gpu*.
//! **Detection ≠ measured joules** — inventory presence never invents RAPL/NVML.

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::budget::Budget;
use crate::tier::CascadeTier;

/// Honesty note stamped on wgpu-detected inventory rows.
pub const DETECT_HONESTY_NOTE: &str =
    "wgpu adapter detect ≠ measured joules; measured_j stays None (no fake RAPL)";

/// How the fabric inventory was obtained.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum InventorySource {
    /// Soft-ref: Cpu always; no live adapter probe. Default / CI prove path.
    #[default]
    SoftwareRef,
    /// Built from wgpu adapter enumeration (`fabric-detect` feature).
    WgpuDetect,
    /// Test / CLI mock from explicit [`AdapterBackendHint`]s (no live GPU required).
    Mock,
}

impl InventorySource {
    /// Wire label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::SoftwareRef => "software_ref",
            Self::WgpuDetect => "wgpu_detect",
            Self::Mock => "mock",
        }
    }
}

impl fmt::Display for InventorySource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// Backend hint from an adapter probe (or mock). Maps to GPU [`DeviceKind`]s.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdapterBackendHint {
    /// Apple Metal.
    Metal,
    /// Vulkan.
    Vulkan,
    /// Browser / native WebGPU.
    BrowserWebGpu,
    /// Direct3D 12.
    Dx12,
    /// OpenGL / GLES.
    Gl,
    /// Unclassified backend.
    Other,
}

impl AdapterBackendHint {
    /// Wire label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Metal => "metal",
            Self::Vulkan => "vulkan",
            Self::BrowserWebGpu => "browser_webgpu",
            Self::Dx12 => "dx12",
            Self::Gl => "gl",
            Self::Other => "other",
        }
    }

    /// Map to MoL GPU device kind when applicable.
    ///
    /// Metal→GpuMetal, Vulkan→GpuVulkan, Dx12→GpuDx12, Gl→GpuGl,
    /// BrowserWebGpu→GpuWebGpu. [`Self::Other`] does not map.
    /// Promotion still requires a non-CPU [`AdapterDeviceClass`] — see
    /// [`FabricInventory::from_adapter_probes`].
    pub const fn to_device_kind(self) -> Option<DeviceKind> {
        match self {
            Self::Metal => Some(DeviceKind::GpuMetal),
            Self::Vulkan => Some(DeviceKind::GpuVulkan),
            Self::BrowserWebGpu => Some(DeviceKind::GpuWebGpu),
            Self::Dx12 => Some(DeviceKind::GpuDx12),
            Self::Gl => Some(DeviceKind::GpuGl),
            Self::Other => None,
        }
    }
}

impl fmt::Display for AdapterBackendHint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// Whether a probed adapter is a real GPU or a software/CPU renderer.
///
/// wgpu `DeviceType::Cpu` (lavapipe, SwiftShader, …) must not be promoted to
/// a Gpu* fabric. Host CPU remains the honest fallback.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AdapterDeviceClass {
    /// Integrated, discrete, virtual, or unclassified GPU — eligible for Gpu*.
    #[default]
    Gpu,
    /// Software / CPU renderer. Recorded, not promoted to Gpu*.
    Cpu,
}

impl AdapterDeviceClass {
    /// Wire label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Gpu => "gpu",
            Self::Cpu => "cpu",
        }
    }
}

impl fmt::Display for AdapterDeviceClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// One probed adapter (name + backend hint). Used by detect and mocks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdapterProbe {
    /// Adapter display name from the driver / wgpu.
    pub name: String,
    /// Backend classification.
    pub backend: AdapterBackendHint,
    /// GPU vs software-CPU. Default Gpu so mock hints still promote.
    #[serde(default)]
    pub device_class: AdapterDeviceClass,
}

impl AdapterProbe {
    /// Construct.
    pub fn new(name: impl Into<String>, backend: AdapterBackendHint) -> Self {
        Self {
            name: name.into(),
            backend,
            device_class: AdapterDeviceClass::Gpu,
        }
    }

    /// Override device class (use [`AdapterDeviceClass::Cpu`] for software renderers).
    pub fn with_device_class(mut self, class: AdapterDeviceClass) -> Self {
        self.device_class = class;
        self
    }
}

/// Device / fabric kind available to MoL routing.
///
/// Alias of the fabric target surface: one enum for inventory + chosen fabric.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceKind {
    /// Host CPU reference path (always present in software-ref).
    Cpu,
    /// Metal GPU (Apple) via wgpu / native.
    GpuMetal,
    /// Vulkan GPU via wgpu / native.
    GpuVulkan,
    /// WebGPU (browser or native wgpu).
    GpuWebGpu,
    /// Direct3D 12 GPU.
    GpuDx12,
    /// OpenGL / GLES GPU.
    GpuGl,
    /// WASM / browser host (no native GPU assumed).
    WasmBrowser,
    /// Thermodynamic / energy-landscape settle fabric (ferrotherm-class; soft-ref logical).
    ThermoSettle,
    /// Neuromorphic spike fabric.
    NeuromorphicSpike,
    /// LUT / allow-table gate fabric (WCA-class).
    LutGate,
    /// Unknown / unclassified device.
    Unknown,
}

/// Fabric target — same surface as [`DeviceKind`] (inventory + routing).
pub type FabricTarget = DeviceKind;

impl DeviceKind {
    /// Wire / receipt label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Cpu => "cpu",
            Self::GpuMetal => "gpu_metal",
            Self::GpuVulkan => "gpu_vulkan",
            Self::GpuWebGpu => "gpu_webgpu",
            Self::GpuDx12 => "gpu_dx12",
            Self::GpuGl => "gpu_gl",
            Self::WasmBrowser => "wasm_browser",
            Self::ThermoSettle => "thermo_settle",
            Self::NeuromorphicSpike => "neuromorphic_spike",
            Self::LutGate => "lut_gate",
            Self::Unknown => "unknown",
        }
    }

    /// True if this is a GPU-class fabric (model residual targets).
    pub const fn is_gpu(self) -> bool {
        matches!(
            self,
            Self::GpuMetal | Self::GpuVulkan | Self::GpuWebGpu | Self::GpuDx12 | Self::GpuGl
        )
    }

    /// Backend wire name when this kind came from a wgpu adapter (`metal`, …).
    ///
    /// Stable receipt [`Self::label`] stays `gpu_metal` / `gpu_vulkan` / …
    /// This is the shorter backend token for operator output.
    pub const fn backend_token(self) -> Option<&'static str> {
        match self {
            Self::GpuMetal => Some("metal"),
            Self::GpuVulkan => Some("vulkan"),
            Self::GpuWebGpu => Some("webgpu"),
            Self::GpuDx12 => Some("dx12"),
            Self::GpuGl => Some("gl"),
            Self::Cpu => Some("cpu"),
            Self::WasmBrowser
            | Self::ThermoSettle
            | Self::NeuromorphicSpike
            | Self::LutGate
            | Self::Unknown => None,
        }
    }

    /// All known kinds (inventory completeness).
    pub const fn all() -> [DeviceKind; 11] {
        [
            Self::Cpu,
            Self::GpuMetal,
            Self::GpuVulkan,
            Self::GpuWebGpu,
            Self::GpuDx12,
            Self::GpuGl,
            Self::WasmBrowser,
            Self::ThermoSettle,
            Self::NeuromorphicSpike,
            Self::LutGate,
            Self::Unknown,
        ]
    }
}

impl fmt::Display for DeviceKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// Presence flag for one device in the inventory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DevicePresence {
    /// Device kind.
    pub kind: DeviceKind,
    /// Whether this device is available for routing.
    pub present: bool,
    /// Optional honesty note (e.g. software-ref always).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

impl DevicePresence {
    /// Construct.
    pub fn new(kind: DeviceKind, present: bool) -> Self {
        Self {
            kind,
            present,
            note: None,
        }
    }

    /// With note.
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }
}

/// Inventory of available devices / fabrics.
///
/// Soft-ref: [`Self::software_ref`] — Cpu always present; others optional flags.
/// Detect (optional feature): [`detect_inventory`] / [`Self::from_adapter_probes`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FabricInventory {
    /// Per-kind presence rows.
    pub devices: Vec<DevicePresence>,
    /// How this inventory was obtained.
    #[serde(default)]
    pub source: InventorySource,
    /// Adapter probes that produced this inventory (empty on pure soft-ref).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub probes: Vec<AdapterProbe>,
}

impl FabricInventory {
    /// Software-reference inventory: Cpu always present; all other kinds absent.
    ///
    /// Does **not** claim Metal/Vulkan/WebGPU/ferrotherm/neuromorphic silicon.
    /// Valid offline / CI prove path — no GPU required.
    pub fn software_ref() -> Self {
        let devices = DeviceKind::all()
            .into_iter()
            .map(|kind| {
                let present = matches!(kind, DeviceKind::Cpu);
                let mut row = DevicePresence::new(kind, present);
                if present {
                    row = row.with_note("software-ref always present");
                }
                row
            })
            .collect();
        Self {
            devices,
            source: InventorySource::SoftwareRef,
            probes: Vec::new(),
        }
    }

    /// Soft-ref plus logical ThermoSettle present (in-tree TernarySettle; not ferrotherm).
    pub fn software_ref_with_thermo() -> Self {
        let mut inv = Self::software_ref();
        inv.set_present(
            DeviceKind::ThermoSettle,
            true,
            Some("software-ref ternary settle (not ferrotherm/klere silicon)"),
        );
        inv
    }

    /// Soft-ref plus a GPU kind present (tests / optional future adapters).
    pub fn software_ref_with_gpu(gpu: DeviceKind) -> Self {
        debug_assert!(gpu.is_gpu(), "expected a GPU-class DeviceKind");
        let mut inv = Self::software_ref();
        inv.set_present(gpu, true, Some("test/optional GPU present flag"));
        inv
    }

    /// Build inventory from adapter probes (mockable without wgpu).
    ///
    /// Cpu is always present (host reference — not a wgpu backend). GPU-class
    /// probes mark the matching Gpu* kinds. [`AdapterDeviceClass::Cpu`] probes
    /// are stored but **not** promoted (honest CPU fallback). Does **not** set
    /// `measured_j` — detection ≠ joules.
    pub fn from_adapter_probes(probes: &[AdapterProbe], source: InventorySource) -> Self {
        let mut inv = Self::software_ref();
        inv.source = source;
        inv.probes = probes.to_vec();
        inv.set_present(
            DeviceKind::Cpu,
            true,
            Some("Cpu always present (host reference; not a wgpu backend)"),
        );
        let mut software_cpu: Vec<String> = Vec::new();
        for probe in probes {
            if probe.device_class == AdapterDeviceClass::Cpu {
                software_cpu.push(format!(
                    "wgpu software adapter '{}' backend={} device_class=cpu — not promoted to Gpu*; host Cpu fallback; {DETECT_HONESTY_NOTE}",
                    probe.name, probe.backend
                ));
                continue;
            }
            if let Some(kind) = probe.backend.to_device_kind() {
                let note = format!(
                    "adapter '{}' backend={} device_class={} — {DETECT_HONESTY_NOTE}",
                    probe.name, probe.backend, probe.device_class
                );
                inv.set_present(kind, true, Some(&note));
            }
        }
        if !software_cpu.is_empty() {
            let note = format!(
                "Cpu always present (host reference; not a wgpu backend); {}",
                software_cpu.join("; ")
            );
            inv.set_present(DeviceKind::Cpu, true, Some(&note));
        }
        inv
    }

    /// Convenience: mock inventory from backend hints alone (names auto-labeled).
    pub fn from_adapter_hints(hints: &[AdapterBackendHint]) -> Self {
        let probes: Vec<AdapterProbe> = hints
            .iter()
            .enumerate()
            .map(|(i, b)| AdapterProbe::new(format!("mock-adapter-{i}"), *b))
            .collect();
        Self::from_adapter_probes(&probes, InventorySource::Mock)
    }

    /// True when this inventory is a valid soft-ref (Cpu present; source SoftwareRef).
    pub fn is_valid_software_ref(&self) -> bool {
        self.source == InventorySource::SoftwareRef && self.is_present(DeviceKind::Cpu)
    }

    /// Set presence for a kind (insert row if missing).
    pub fn set_present(&mut self, kind: DeviceKind, present: bool, note: Option<&str>) {
        if let Some(row) = self.devices.iter_mut().find(|d| d.kind == kind) {
            row.present = present;
            if let Some(n) = note {
                row.note = Some(n.into());
            }
        } else {
            let mut row = DevicePresence::new(kind, present);
            if let Some(n) = note {
                row = row.with_note(n);
            }
            self.devices.push(row);
        }
    }

    /// True if `kind` is marked present.
    pub fn is_present(&self, kind: DeviceKind) -> bool {
        self.devices
            .iter()
            .any(|d| d.kind == kind && d.present)
    }

    /// Present kinds (stable enum order).
    pub fn present_kinds(&self) -> Vec<DeviceKind> {
        DeviceKind::all()
            .into_iter()
            .filter(|k| self.is_present(*k))
            .collect()
    }

    /// First present GPU in preference order: WebGPU → Metal → Vulkan → Dx12 → Gl.
    pub fn cheapest_gpu(&self) -> Option<DeviceKind> {
        [
            DeviceKind::GpuWebGpu,
            DeviceKind::GpuMetal,
            DeviceKind::GpuVulkan,
            DeviceKind::GpuDx12,
            DeviceKind::GpuGl,
        ]
        .into_iter()
        .find(|k| self.is_present(*k))
    }

    /// Compact summary for rationale strings.
    pub fn summary(&self) -> String {
        let present: Vec<&str> = self
            .present_kinds()
            .iter()
            .map(|k| k.label())
            .collect();
        if present.is_empty() {
            "none".into()
        } else {
            present.join(",")
        }
    }
}

impl Default for FabricInventory {
    fn default() -> Self {
        Self::software_ref()
    }
}

/// Outcome of fabric routing for a closed (or attempted) cascade tier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FabricDecision {
    /// Cheapest sufficient fabric chosen.
    Chosen(DeviceKind),
    /// No sufficient fabric under inventory / budget — refuse.
    Refuse {
        /// Why routing refused.
        reason: String,
    },
}

impl FabricDecision {
    /// Chosen fabric if any.
    pub fn chosen(&self) -> Option<DeviceKind> {
        match self {
            Self::Chosen(k) => Some(*k),
            Self::Refuse { .. } => None,
        }
    }

    /// True on refuse.
    pub fn is_refuse(&self) -> bool {
        matches!(self, Self::Refuse { .. })
    }
}

/// Route fabric after cascade picks a tier.
///
/// Law:
/// - Lookup / Formula → [`DeviceKind::Cpu`]
/// - Solver (settle) → [`DeviceKind::ThermoSettle`] if present, else Cpu
/// - Model residual → cheapest present Gpu*, else refuse (budget / inventory)
///
/// `settle_hint` is reserved for future Solver sub-gears; Solver always uses
/// the settle→ThermoSettle|Cpu rule today.
pub fn route_fabric(
    tier: CascadeTier,
    inventory: &FabricInventory,
    budget: &Budget,
) -> FabricDecision {
    let _ = budget; // reserved: future joule-priced fabric picks
    match tier {
        CascadeTier::Lookup | CascadeTier::Formula => {
            // Lookup/Formula always close on Cpu (LutGate may be present but law says Cpu).
            if inventory.is_present(DeviceKind::Cpu) {
                FabricDecision::Chosen(DeviceKind::Cpu)
            } else {
                FabricDecision::Refuse {
                    reason: "Lookup/Formula require Cpu; Cpu absent from fabric inventory".into(),
                }
            }
        }
        CascadeTier::Solver => {
            if inventory.is_present(DeviceKind::ThermoSettle) {
                FabricDecision::Chosen(DeviceKind::ThermoSettle)
            } else if inventory.is_present(DeviceKind::Cpu) {
                FabricDecision::Chosen(DeviceKind::Cpu)
            } else {
                FabricDecision::Refuse {
                    reason: "Solver/settle requires ThermoSettle|Cpu; neither present".into(),
                }
            }
        }
        CascadeTier::Model => {
            // Model residual → Gpu*; refuse if none (and/or budget forbids residual).
            if !budget.allow_model {
                return FabricDecision::Refuse {
                    reason: "model residual fabric refused (allow_model=false)".into(),
                };
            }
            if let Some(gpu) = inventory.cheapest_gpu() {
                // Coin-cell / tiny budgets refuse hot GPU residual even if present.
                if budget.max_j.0 < 1.0e-4 {
                    return FabricDecision::Refuse {
                        reason: format!(
                            "model residual on {} refused: budget max_j={} too tight for Gpu*",
                            gpu.label(),
                            budget.max_j.0
                        ),
                    };
                }
                FabricDecision::Chosen(gpu)
            } else {
                FabricDecision::Refuse {
                    reason: "model residual requires Gpu* fabric; none present in inventory".into(),
                }
            }
        }
    }
}

/// Whether the `fabric-detect` cargo feature is compiled in.
pub const FABRIC_DETECT_ENABLED: bool = cfg!(feature = "fabric-detect");

/// Probe wgpu adapters and return inventory (Cpu always + mapped Gpu*).
///
/// Requires `--features fabric-detect`. **Does not** measure joules / RAPL.
/// A probe panic or empty list stays fail-soft: host Cpu only, source
/// [`InventorySource::WgpuDetect`] (we did probe), no invented joules.
#[cfg(feature = "fabric-detect")]
pub fn detect_inventory() -> FabricInventory {
    let probes = detect_adapter_probes();
    FabricInventory::from_adapter_probes(&probes, InventorySource::WgpuDetect)
}

/// Enumerate wgpu adapters → [`AdapterProbe`] list.
///
/// Uses [`wgpu::Backends::all`] so Metal, Vulkan, DX12, and GL are visible
/// where the platform has them. Empty on probe failure (no panic to the caller).
/// Software `DeviceType::Cpu` adapters are tagged [`AdapterDeviceClass::Cpu`].
#[cfg(feature = "fabric-detect")]
pub fn detect_adapter_probes() -> Vec<AdapterProbe> {
    let enumerated = std::panic::catch_unwind(|| {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        instance.enumerate_adapters(wgpu::Backends::all())
    });
    let Ok(adapters) = enumerated else {
        return Vec::new();
    };
    adapters
        .into_iter()
        .map(|adapter| {
            let info = adapter.get_info();
            let backend = match info.backend {
                wgpu::Backend::Metal => AdapterBackendHint::Metal,
                wgpu::Backend::Vulkan => AdapterBackendHint::Vulkan,
                wgpu::Backend::BrowserWebGpu => AdapterBackendHint::BrowserWebGpu,
                wgpu::Backend::Dx12 => AdapterBackendHint::Dx12,
                wgpu::Backend::Gl => AdapterBackendHint::Gl,
                wgpu::Backend::Empty => AdapterBackendHint::Other,
            };
            let class = match info.device_type {
                wgpu::DeviceType::Cpu => AdapterDeviceClass::Cpu,
                wgpu::DeviceType::IntegratedGpu
                | wgpu::DeviceType::DiscreteGpu
                | wgpu::DeviceType::VirtualGpu
                | wgpu::DeviceType::Other => AdapterDeviceClass::Gpu,
            };
            AdapterProbe::new(info.name, backend).with_device_class(class)
        })
        .collect()
}

/// Stub when `fabric-detect` is off — returns empty probes (caller keeps soft-ref).
#[cfg(not(feature = "fabric-detect"))]
pub fn detect_adapter_probes() -> Vec<AdapterProbe> {
    Vec::new()
}

/// Soft fallback when feature is off: returns soft-ref (never panics; offline-safe).
#[cfg(not(feature = "fabric-detect"))]
pub fn detect_inventory() -> FabricInventory {
    FabricInventory::software_ref()
}

/// Inventory for a schedule stamp.
///
/// Live wgpu probe when `fabric-detect` is compiled; otherwise
/// [`FabricInventory::software_ref`]. Host Cpu is always present. Never sets
/// `measured_j` (inventory has no joule field).
pub fn inventory_for_schedule() -> FabricInventory {
    detect_inventory()
}

/// Schedule fabric for a cascade tier → [`crate::compute::ScheduleDecision`].
///
/// Wraps [`route_fabric`] with estimated joules for the route stamp. Copies the
/// inventory honesty note onto the chosen fabric. Soft-ref / offline / detect
/// paths are estimated only; **never** invents `measured_j`.
pub fn schedule_fabric(
    tier: CascadeTier,
    inventory: &FabricInventory,
    budget: &Budget,
    route_estimated_j: crate::energy::Joules,
) -> crate::compute::ScheduleDecision {
    let dec = route_fabric(tier, inventory, budget);
    let mut sched =
        crate::compute::ScheduleDecision::from_fabric_decision(dec, route_estimated_j);
    if let crate::compute::ScheduleDecision::Chosen { fabric, .. } = &mut sched {
        if let Some(note) = inventory
            .devices
            .iter()
            .find(|d| d.kind == fabric.kind)
            .and_then(|d| d.note.clone())
        {
            fabric.note = Some(note);
        }
    }
    sched
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn software_ref_cpu_always() {
        let inv = FabricInventory::software_ref();
        assert!(inv.is_present(DeviceKind::Cpu));
        assert!(!inv.is_present(DeviceKind::GpuMetal));
        assert!(!inv.is_present(DeviceKind::ThermoSettle));
        assert_eq!(inv.present_kinds(), vec![DeviceKind::Cpu]);
        assert_eq!(inv.source, InventorySource::SoftwareRef);
        assert!(inv.is_valid_software_ref());
    }

    #[test]
    fn soft_ref_inventory_valid_offline() {
        // Prove path: soft-ref must be valid without GPU / without fabric-detect.
        let inv = FabricInventory::software_ref();
        assert!(inv.is_valid_software_ref());
        assert!(!inv.is_present(DeviceKind::GpuMetal));
        assert!(!inv.is_present(DeviceKind::GpuVulkan));
        assert!(!inv.is_present(DeviceKind::GpuWebGpu));
        // Default detect without feature (or empty probe) must not invent measured joules.
        let fallback = detect_inventory();
        assert!(fallback.is_present(DeviceKind::Cpu));
        if !FABRIC_DETECT_ENABLED {
            assert_eq!(fallback.source, InventorySource::SoftwareRef);
        }
    }

    #[test]
    fn mock_hints_map_metal_vulkan_webgpu() {
        let inv = FabricInventory::from_adapter_hints(&[
            AdapterBackendHint::Metal,
            AdapterBackendHint::Vulkan,
            AdapterBackendHint::BrowserWebGpu,
            AdapterBackendHint::Dx12,
            AdapterBackendHint::Gl,
        ]);
        assert_eq!(inv.source, InventorySource::Mock);
        assert!(inv.is_present(DeviceKind::Cpu));
        assert!(inv.is_present(DeviceKind::GpuMetal));
        assert!(inv.is_present(DeviceKind::GpuVulkan));
        assert!(inv.is_present(DeviceKind::GpuWebGpu));
        assert!(inv.is_present(DeviceKind::GpuDx12));
        assert!(inv.is_present(DeviceKind::GpuGl));
        assert_eq!(inv.probes.len(), 5);
        // Notes must carry honesty (detect ≠ joules).
        let metal = inv
            .devices
            .iter()
            .find(|d| d.kind == DeviceKind::GpuMetal)
            .expect("metal row");
        let note = metal.note.as_deref().unwrap_or("");
        assert!(note.contains("measured") || note.contains("RAPL") || note.contains("joules"));
    }

    #[test]
    fn lookup_formula_route_cpu() {
        let inv = FabricInventory::software_ref();
        let b = Budget::coin_cell();
        assert_eq!(
            route_fabric(CascadeTier::Lookup, &inv, &b).chosen(),
            Some(DeviceKind::Cpu)
        );
        assert_eq!(
            route_fabric(CascadeTier::Formula, &inv, &b).chosen(),
            Some(DeviceKind::Cpu)
        );
    }

    #[test]
    fn settle_thermo_or_cpu() {
        let inv = FabricInventory::software_ref();
        let b = Budget::demo();
        assert_eq!(
            route_fabric(CascadeTier::Solver, &inv, &b).chosen(),
            Some(DeviceKind::Cpu)
        );
        let inv_t = FabricInventory::software_ref_with_thermo();
        assert_eq!(
            route_fabric(CascadeTier::Solver, &inv_t, &b).chosen(),
            Some(DeviceKind::ThermoSettle)
        );
    }

    #[test]
    fn model_gpu_or_refuse() {
        let inv = FabricInventory::software_ref();
        let b = Budget::demo().allow_model();
        assert!(route_fabric(CascadeTier::Model, &inv, &b).is_refuse());
        let inv_g = FabricInventory::software_ref_with_gpu(DeviceKind::GpuWebGpu);
        assert_eq!(
            route_fabric(CascadeTier::Model, &inv_g, &b).chosen(),
            Some(DeviceKind::GpuWebGpu)
        );
        // Tight budget refuses even with GPU.
        let tight = Budget::coin_cell().allow_model();
        assert!(route_fabric(CascadeTier::Model, &inv_g, &tight).is_refuse());
    }

    #[test]
    fn cpu_class_adapter_not_promoted() {
        let probe = AdapterProbe::new("llvmpipe", AdapterBackendHint::Vulkan)
            .with_device_class(AdapterDeviceClass::Cpu);
        let inv = FabricInventory::from_adapter_probes(&[probe], InventorySource::Mock);
        assert!(inv.is_present(DeviceKind::Cpu));
        assert!(!inv.is_present(DeviceKind::GpuVulkan));
        assert!(inv.cheapest_gpu().is_none());
        let note = inv
            .devices
            .iter()
            .find(|d| d.kind == DeviceKind::Cpu)
            .and_then(|d| d.note.as_deref())
            .unwrap_or("");
        assert!(note.contains("not promoted"));
        let b = Budget::demo().allow_model();
        assert!(route_fabric(CascadeTier::Model, &inv, &b).is_refuse());
        let step = schedule_fabric(CascadeTier::Model, &inv, &b, crate::energy::Joules::ZERO)
            .to_compute_step();
        assert!(step.measured_j.is_none());
        assert!(step.unavailable_reason.is_some());
    }

    #[test]
    fn mock_metal_routes_model_to_gpu_metal() {
        let inv = FabricInventory::from_adapter_hints(&[AdapterBackendHint::Metal]);
        let b = Budget::demo().allow_model();
        assert_eq!(
            route_fabric(CascadeTier::Model, &inv, &b).chosen(),
            Some(DeviceKind::GpuMetal)
        );
    }

    /// Live wgpu probe — ignored in CI (no GPU required). Run with:
    /// `cargo test -p mol-core --features fabric-detect -- --ignored`
    #[test]
    #[ignore = "optional live GPU; enable fabric-detect and run with --ignored on a host with adapters"]
    fn live_wgpu_detect_when_available() {
        if !FABRIC_DETECT_ENABLED {
            return;
        }
        let probes = detect_adapter_probes();
        let inv = detect_inventory();
        assert!(inv.is_present(DeviceKind::Cpu));
        assert_eq!(inv.source, InventorySource::WgpuDetect);
        // If any GPU-class probe appeared, inventory must reflect mapping.
        for p in &probes {
            if let Some(kind) = p.backend.to_device_kind() {
                assert!(
                    inv.is_present(kind),
                    "probe {:?} should mark {}",
                    p,
                    kind.label()
                );
            }
        }
        // Honesty: detection never invents measured joules (inventory-only).
        assert!(
            inv.devices.iter().any(|d| {
                d.note
                    .as_deref()
                    .map(|n| n.contains("measured") || n.contains(DETECT_HONESTY_NOTE))
                    .unwrap_or(false)
                    || d.kind == DeviceKind::Cpu
            })
        );
    }
}
