# MoL multi-fabric compute receipt path

Clean-room **optimized compute across fabrics** — not one accelerator. MoL law
inventories devices, schedules the cheapest sufficient fabric for a closed
cascade tier, and stamps `ComputeStepReceipt` with fabric id + joules honesty.
**Never invent `measured_j`.**

Ferric ([github.com/dcharlot-physicalai-bmi/ferric](https://github.com/dcharlot-physicalai-bmi/ferric))
is **reference semantics only** (same kernels Metal/Vulkan/WebGPU/CPU; wgpu
`Context::enumerate` / `for_adapter`). MoL does **not** path-dep or vendor Ferric.

## Law

| Rule | Behavior |
|---|---|
| Inventory | Soft-ref: **Cpu always**; GpuMetal/Vulkan/WebGPU/Thermo/… soft-unavailable OK offline |
| Optional detect | `--features fabric-detect` wgpu `Backends::all` maps Metal→`gpu_metal` (backend `metal`), Vulkan→`gpu_vulkan`, Dx12→`gpu_dx12`, Gl→`gpu_gl`, WebGPU→`gpu_webgpu`. Host Cpu is always present and is not a wgpu backend. `DeviceType::Cpu` software adapters are recorded and **not** promoted. **detect ≠ joules** |
| Live schedule | `mol ask --detect`, `mol fabric --detect`, `mol ecosystem-certify --detect --tier model` feed [`inventory_for_schedule`] into `schedule_fabric`. Formula stamps `cpu`. Model stamps cheapest Gpu* (`gpu_metal` on Apple) or REFUSE `fabric_unavailable`. Coin-cell budget refuses hot Gpu* even when present. |
| Schedule | Lookup/Formula → Cpu; Settle → ThermoSettle\|Cpu; Model residual → Gpu* or **REFUSE** `fabric_unavailable` |
| Receipt stamp | `fabric:route` [`ComputeStepReceipt`] with `fabric_id`, optional `unavailable_reason`, `estimated_j` only offline |
| Honesty | `measured_j=None` until a real meter; never invent from detect / schedule |

## Soft-link Ferric patterns (not vendored)

| Ferric | MoL clean-room |
|---|---|
| `Context::new` / `enumerate` / `for_adapter` | [`FabricInventory`] + optional `detect_inventory` / mock hints |
| Backend Metal/Vulkan/WebGPU | [`AdapterBackendHint`] → [`DeviceKind`] Gpu* |
| Same kernel everywhere | Cascade gear closes once; fabric is where it runs |
| Heterogeneous scheduler (roadmap) | [`schedule_fabric`] / [`ScheduleDecision`] cheapest-sufficient |
| CPU reference validation | Soft-ref Cpu always present; joules still estimated unless metered |

Mac checkout path documented in PLAN (`/Users/dcharlot/vibe-coding/ferric`) is
optional read-only when that machine is online — **not** required for prove.

## API sketch

```rust
use mol_core::{
    fabrics_from_inventory, schedule_fabric, Budget, CascadeTier, DeviceKind,
    FabricInventory, ScheduleDecision,
};

let inv = FabricInventory::software_ref(); // Cpu always; Gpu* soft-unavailable
let fabrics = fabrics_from_inventory(&inv);

let sched = schedule_fabric(CascadeTier::Formula, &inv, &Budget::coin_cell(), /*est*/ Default::default());
assert_eq!(sched.chosen_kind(), Some(DeviceKind::Cpu));
let step = sched.to_compute_step(); // label fabric:route, fabric_id=cpu, measured_j=None

let refuse = schedule_fabric(
    CascadeTier::Model,
    &inv,
    &Budget::demo().allow_model(),
    Default::default(),
);
assert!(matches!(refuse, ScheduleDecision::Unavailable { .. }));
assert!(refuse.to_compute_step().unavailable_reason.is_some());
```

Cascade/`MixtureOfLimits::close` stamps `fabric:route` on COMMIT and on
`fabric_unavailable` REFUSE.

## Prove

```bash
cd /workspace/mixture-of-limits
cargo run -p mol-cli -- prove
# looks for: VERIFIED ecosystem_multi_fabric
```

Offline criterion: CPU Formula COMMIT + simulated Model residual refuse without
Gpu*; mock Metal mapping still carries detect≠joules honesty.
`ecosystem_live_fabric_soft` passes with the feature off (soft-ref) and, when
`fabric-detect` is on, accepts whatever adapters exist without requiring a GPU
and without inventing `measured_j`.

```bash
# Mac (Metal): print adapters and stamp gpu_metal from live inventory
cargo run -p mol-cli --features fabric-detect -- fabric --detect
cargo run -p mol-cli --features fabric-detect -- ecosystem-certify --detect --tier model
cargo run -p mol-cli --features fabric-detect -- ask --detect --close --allow-model "write a short poem about transistors"
```

## Tiny kernel (vector-add)

When schedule chooses Gpu*, `ecosystem-certify` runs a clean-room WGSL vector-add
(`mol-core::kernel`) and stamps `kernel:vector_add` with `execution_proof`
(checksum + sample). Soft stub offline; live Metal under `--features fabric-detect`
on Apple. See [`wgpu-kernel.md`](./wgpu-kernel.md).

## Out of scope

Live Ferric EFA / silicon meters, inventing RAPL from wgpu detect, vendoring
Ferric crates, requiring GPU for CI prove.
