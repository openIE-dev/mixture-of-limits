//! Tiny clean-room wgpu compute kernel (vector-add) for MoL certify receipts.
//!
//! Soft-ref / feature-off: CPU reference stub (same math) — prove stays green offline.
//! `fabric-detect`: live WGSL dispatch on the chosen Gpu* backend (Metal on Apple).
//! **Never invent `measured_j`.** Detection / dispatch ≠ joules.
//!
//! Ferric is reference semantics only — this module does not vendor or path-dep Ferric.

use serde::{Deserialize, Serialize};

use crate::compute::ComputeStepReceipt;
use crate::energy::{EstimateKind, Joules, MeasureSource};
use crate::fabric::{DeviceKind, FABRIC_DETECT_ENABLED};
use crate::honesty::EnergyHonestyClass;

/// Default element count for the tiny vector-add (one workgroup-friendly size).
pub const TINY_VECTOR_ADD_N: u32 = 64;

/// Estimated joules for the tiny kernel (analytical floor; not RAPL).
pub const KERNEL_ESTIMATED_J: Joules = Joules(1e-9);

/// Honesty note: kernel execution never invents package joules.
pub const KERNEL_HONESTY_NOTE: &str =
    "wgpu/CPU vector-add ≠ measured joules; measured_j stays None unless a real meter is attached";

/// Whether the optional live kernel path is compiled (`fabric-detect`).
pub const WGPU_KERNEL_ENABLED: bool = FABRIC_DETECT_ENABLED;

/// How the tiny kernel was satisfied.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KernelMode {
    /// Real wgpu compute dispatch on a Gpu* backend.
    Live {
        /// Backend token (`metal`, `vulkan`, …).
        backend: String,
    },
    /// CPU reference of the same math (offline / no adapter / feature off).
    SoftStub {
        /// Why live dispatch was not used.
        reason: String,
    },
    /// Explicitly skipped (e.g. Cpu fabric schedule — kernel not required).
    Skipped {
        /// Skip reason.
        reason: String,
    },
}

impl KernelMode {
    /// True when a live GPU dispatch ran.
    pub fn is_live(&self) -> bool {
        matches!(self, Self::Live { .. })
    }

    /// Wire label fragment.
    pub fn label_suffix(&self) -> &'static str {
        match self {
            Self::Live { .. } => "live",
            Self::SoftStub { .. } => "soft",
            Self::Skipped { .. } => "skip",
        }
    }
}

/// Result of [`run_tiny_vector_add`] / soft stub.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KernelRunResult {
    /// Step label (`kernel:vector_add`).
    pub label: String,
    /// Fabric the schedule chose (or Cpu for pure soft).
    pub fabric: DeviceKind,
    /// Live / soft / skip.
    pub mode: KernelMode,
    /// Element count.
    pub n: u32,
    /// First few output values (proof sample).
    pub output_sample: Vec<f32>,
    /// Stable checksum of the full output vector (`f32` bits FNV-1a style).
    pub checksum: u64,
    /// Estimated joules only (never invents measured).
    pub estimated_j: Joules,
    /// Operator / receipt note.
    pub note: String,
}

impl KernelRunResult {
    /// Compact execution proof string for [`ComputeStepReceipt::execution_proof`].
    pub fn execution_proof(&self) -> String {
        let sample: Vec<String> = self
            .output_sample
            .iter()
            .take(8)
            .map(|v| {
                if *v == v.trunc() && v.abs() < 1e9 {
                    format!("{}", *v as i64)
                } else {
                    format!("{v}")
                }
            })
            .collect();
        let live = self.mode.is_live();
        let mode = match &self.mode {
            KernelMode::Live { backend } => format!("live:{backend}"),
            KernelMode::SoftStub { reason } => format!("soft:{}", truncate(reason, 48)),
            KernelMode::Skipped { reason } => format!("skip:{}", truncate(reason, 48)),
        };
        format!(
            "checksum=0x{:016x};n={};sample=[{}];mode={};live={}",
            self.checksum,
            self.n,
            sample.join(","),
            mode,
            live
        )
    }

    /// Stamp a [`ComputeStepReceipt`] (estimated honesty; never invents measured_j).
    pub fn to_compute_step(&self) -> ComputeStepReceipt {
        let mut step = ComputeStepReceipt {
            label: self.label.clone(),
            fabric: Some(self.fabric),
            fabric_id: Some(self.fabric.label().to_string()),
            unavailable_reason: None,
            estimated_j: self.estimated_j,
            estimate_kind: EstimateKind::Analytical,
            measured_j: None,
            measure_source: MeasureSource::CascadeEstimate,
            honesty: EnergyHonestyClass::Estimated,
            capsule_id: None,
            execution_proof: Some(self.execution_proof()),
        };
        if let KernelMode::Skipped { reason } = &self.mode {
            step.unavailable_reason = Some(reason.clone());
        }
        step
    }

    /// True when honesty invariants hold for this result.
    pub fn honesty_ok(&self) -> bool {
        self.to_compute_step().honesty_ok()
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max.saturating_sub(1)])
    }
}

/// CPU-reference vector-add inputs/outputs used by soft stub and live verification.
pub fn vector_add_reference(n: u32) -> (Vec<f32>, Vec<f32>, Vec<f32>) {
    let a: Vec<f32> = (0..n).map(|i| i as f32).collect();
    let b: Vec<f32> = (0..n).map(|i| (i * 2) as f32).collect();
    let c: Vec<f32> = a.iter().zip(b.iter()).map(|(x, y)| x + y).collect();
    (a, b, c)
}

/// Stable checksum over `f32` bit patterns (FNV-1a 64).
pub fn checksum_f32(data: &[f32]) -> u64 {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;
    let mut h = FNV_OFFSET;
    for v in data {
        for byte in v.to_bits().to_le_bytes() {
            h ^= u64::from(byte);
            h = h.wrapping_mul(FNV_PRIME);
        }
    }
    h
}

/// Soft CPU stub of the tiny vector-add (always available; prove-safe).
pub fn soft_vector_add(fabric: DeviceKind, reason: impl Into<String>) -> KernelRunResult {
    let n = TINY_VECTOR_ADD_N;
    let (_a, _b, c) = vector_add_reference(n);
    let checksum = checksum_f32(&c);
    let sample: Vec<f32> = c.iter().take(8).copied().collect();
    KernelRunResult {
        label: "kernel:vector_add".into(),
        fabric,
        mode: KernelMode::SoftStub {
            reason: reason.into(),
        },
        n,
        output_sample: sample,
        checksum,
        estimated_j: KERNEL_ESTIMATED_J,
        note: format!("{KERNEL_HONESTY_NOTE}; mode=soft_stub"),
    }
}

/// Skip stamp (Cpu fabric — kernel not required for Formula soft-ref).
pub fn skip_vector_add(fabric: DeviceKind, reason: impl Into<String>) -> KernelRunResult {
    KernelRunResult {
        label: "kernel:vector_add".into(),
        fabric,
        mode: KernelMode::Skipped {
            reason: reason.into(),
        },
        n: 0,
        output_sample: Vec::new(),
        checksum: 0,
        estimated_j: Joules::ZERO,
        note: format!("{KERNEL_HONESTY_NOTE}; mode=skipped"),
    }
}

/// Run the tiny vector-add for a scheduled fabric.
///
/// - [`DeviceKind::Cpu`] / non-GPU → soft stub (or skip when `skip_on_cpu`).
/// - Gpu* + `fabric-detect` → live WGSL on matching backend; fall back to soft stub.
/// - Gpu* without feature → soft stub with honest reason.
///
/// **Never** sets measured joules.
pub fn run_tiny_vector_add(fabric: DeviceKind) -> KernelRunResult {
    if !fabric.is_gpu() {
        return soft_vector_add(
            fabric,
            "non-Gpu* fabric: CPU reference stub (kernel optional on Cpu schedule)",
        );
    }
    #[cfg(feature = "fabric-detect")]
    {
        match live_vector_add(fabric) {
            Ok(r) => r,
            Err(reason) => soft_vector_add(
                fabric,
                format!("live wgpu dispatch unavailable: {reason}"),
            ),
        }
    }
    #[cfg(not(feature = "fabric-detect"))]
    {
        soft_vector_add(
            fabric,
            "fabric-detect/wgpu-kernel feature off — CPU reference stub (prove offline)",
        )
    }
}

/// Run kernel only when schedule chose Gpu*; otherwise return a skip result.
pub fn run_tiny_vector_add_if_gpu(fabric: DeviceKind) -> KernelRunResult {
    if fabric.is_gpu() {
        run_tiny_vector_add(fabric)
    } else {
        skip_vector_add(
            fabric,
            "kernel skipped: schedule chose non-Gpu* (Formula/Lookup → Cpu)",
        )
    }
}

#[cfg(feature = "fabric-detect")]
fn live_vector_add(fabric: DeviceKind) -> Result<KernelRunResult, String> {
    let backend_want = fabric.backend_token().ok_or_else(|| {
        format!("fabric {} has no wgpu backend token", fabric.label())
    })?;

    let n = TINY_VECTOR_ADD_N;
    let (a, b, expected) = vector_add_reference(n);
    let expected_checksum = checksum_f32(&expected);

    let (device, queue, adapter_name, backend_got) =
        pollster::block_on(request_device_for_backend(backend_want))?;

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("mol_vector_add"),
        source: wgpu::ShaderSource::Wgsl(VECTOR_ADD_WGSL.into()),
    });

    let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("mol_vector_add_bgl"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("mol_vector_add_pl"),
        bind_group_layouts: &[&bgl],
        push_constant_ranges: &[],
    });

    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("mol_vector_add_pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: Some("main"),
        compilation_options: Default::default(),
        cache: None,
    });

    let a_bytes = f32_to_bytes(&a);
    let b_bytes = f32_to_bytes(&b);
    let out_size = (n as u64) * 4;

    let buf_a = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("mol_a"),
        size: out_size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let buf_b = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("mol_b"),
        size: out_size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let buf_c = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("mol_c"),
        size: out_size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let buf_read = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("mol_c_read"),
        size: out_size,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    queue.write_buffer(&buf_a, 0, &a_bytes);
    queue.write_buffer(&buf_b, 0, &b_bytes);

    let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("mol_vector_add_bg"),
        layout: &bgl,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: buf_a.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: buf_b.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: buf_c.as_entire_binding(),
            },
        ],
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("mol_vector_add_enc"),
    });
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("mol_vector_add_pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind, &[]);
        pass.dispatch_workgroups((n + 63) / 64, 1, 1);
    }
    encoder.copy_buffer_to_buffer(&buf_c, 0, &buf_read, 0, out_size);
    queue.submit(Some(encoder.finish()));

    let slice = buf_read.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    device.poll(wgpu::Maintain::Wait);
    rx.recv()
        .map_err(|e| format!("map channel: {e}"))?
        .map_err(|e| format!("map_async: {e}"))?;

    let data = slice.get_mapped_range();
    let out = bytes_to_f32(&data);
    drop(data);
    buf_read.unmap();

    if out.len() != expected.len() {
        return Err(format!(
            "output len {} != expected {}",
            out.len(),
            expected.len()
        ));
    }
    for (i, (got, exp)) in out.iter().zip(expected.iter()).enumerate() {
        if (got - exp).abs() > 1e-5 {
            return Err(format!("mismatch at {i}: got {got} expected {exp}"));
        }
    }
    let checksum = checksum_f32(&out);
    if checksum != expected_checksum {
        return Err(format!(
            "checksum mismatch live=0x{checksum:016x} ref=0x{expected_checksum:016x}"
        ));
    }

    let sample: Vec<f32> = out.iter().take(8).copied().collect();
    Ok(KernelRunResult {
        label: "kernel:vector_add".into(),
        fabric,
        mode: KernelMode::Live {
            backend: backend_got,
        },
        n,
        output_sample: sample,
        checksum,
        estimated_j: KERNEL_ESTIMATED_J,
        note: format!(
            "{KERNEL_HONESTY_NOTE}; mode=live adapter={adapter_name:?} fabric={}",
            fabric.label()
        ),
    })
}

#[cfg(feature = "fabric-detect")]
const VECTOR_ADD_WGSL: &str = r#"
@group(0) @binding(0) var<storage, read> a: array<f32>;
@group(0) @binding(1) var<storage, read> b: array<f32>;
@group(0) @binding(2) var<storage, read_write> c: array<f32>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if (i >= arrayLength(&c)) {
        return;
    }
    c[i] = a[i] + b[i];
}
"#;

#[cfg(feature = "fabric-detect")]
fn f32_to_bytes(v: &[f32]) -> Vec<u8> {
    v.iter().flat_map(|f| f.to_le_bytes()).collect()
}

#[cfg(feature = "fabric-detect")]
fn bytes_to_f32(bytes: &[u8]) -> Vec<f32> {
    bytes
        .chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}

#[cfg(feature = "fabric-detect")]
async fn request_device_for_backend(
    backend_want: &str,
) -> Result<(wgpu::Device, wgpu::Queue, String, String), String> {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });
    let adapters = instance.enumerate_adapters(wgpu::Backends::all());
    let mut matched = None;
    for adapter in adapters {
        let info = adapter.get_info();
        if matches!(info.device_type, wgpu::DeviceType::Cpu) {
            continue;
        }
        let token = match info.backend {
            wgpu::Backend::Metal => "metal",
            wgpu::Backend::Vulkan => "vulkan",
            wgpu::Backend::BrowserWebGpu => "webgpu",
            wgpu::Backend::Dx12 => "dx12",
            wgpu::Backend::Gl => "gl",
            wgpu::Backend::Empty => continue,
        };
        if token == backend_want {
            matched = Some((adapter, info.name, token.to_string()));
            break;
        }
    }
    let (adapter, name, token) = matched.ok_or_else(|| {
        format!("no non-CPU wgpu adapter with backend={backend_want}")
    })?;
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: Some("mol_tiny_kernel"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_defaults(),
                memory_hints: Default::default(),
            },
            None,
        )
        .await
        .map_err(|e| format!("request_device: {e}"))?;
    Ok((device, queue, name, token))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn soft_stub_checksum_stable_and_honest() {
        let r = soft_vector_add(DeviceKind::GpuMetal, "unit test");
        assert_eq!(r.n, TINY_VECTOR_ADD_N);
        assert!(!r.mode.is_live());
        let (_a, _b, c) = vector_add_reference(TINY_VECTOR_ADD_N);
        assert_eq!(r.checksum, checksum_f32(&c));
        assert_eq!(r.output_sample[0], 0.0);
        assert_eq!(r.output_sample[1], 3.0);
        assert_eq!(r.output_sample[2], 6.0);
        let step = r.to_compute_step();
        assert_eq!(step.label, "kernel:vector_add");
        assert_eq!(step.fabric_id.as_deref(), Some("gpu_metal"));
        assert!(step.measured_j.is_none());
        assert!(step.honesty_ok());
        assert!(step.execution_proof.as_ref().unwrap().contains("checksum=0x"));
        assert!(step.execution_proof.as_ref().unwrap().contains("live=false"));
    }

    #[test]
    fn skip_on_cpu_schedule() {
        let r = run_tiny_vector_add_if_gpu(DeviceKind::Cpu);
        assert!(matches!(r.mode, KernelMode::Skipped { .. }));
        assert!(r.honesty_ok());
        assert!(r.to_compute_step().measured_j.is_none());
    }

    #[test]
    fn gpu_path_soft_without_feature_or_fallback() {
        let r = run_tiny_vector_add(DeviceKind::GpuMetal);
        // Feature off → soft. Feature on without Metal → soft. Never invents measured_j.
        assert!(r.honesty_ok());
        assert!(r.to_compute_step().measured_j.is_none());
        assert_eq!(r.fabric, DeviceKind::GpuMetal);
        if !WGPU_KERNEL_ENABLED {
            assert!(!r.mode.is_live());
        }
    }
}
