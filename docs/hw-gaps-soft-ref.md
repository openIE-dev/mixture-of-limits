# HW Gaps soft-ref sims (physical_settle … photonic_mzi)

The Periodic Stack keeps eight **Gap** markers beyond the 258 Present live
catalog (cells 259–266). Silicon / annealer / photonic / on-device Ferric
coverage is **not** claimed. MoL advances them with **classical soft-ref sims
and adapters** so prove can exercise settle / Ising / adiabatic / MZI / … law
without inventing meters.

## Honesty

| Flag | Soft-ref value |
|---|---|
| Stack `CellStatus` | **Gap** (unchanged) |
| `silicon_claimed` | `false` |
| `measured_j` | `None` |
| `stage_c_measured` | `false` |
| `board_synth_claimed` | `false` |
| `primitive_gap` on Gap probe | still fires |

Soft-ref sim ≠ Present silicon. Fake `stage_c_measured=true` / board-meter
claims **refuse**.

## The eight

| Id | Soft-ref sim |
|---|---|
| `physical_settle` | Classical energy descent to fixed point (or refuse) |
| `reversible_rewrite` | Info-preserving XOR rewrite + restore check |
| `ising_bind` | Tiny N=4 1D Ising ground-state enum |
| `adiabatic_schedule` | Linear s(t) schedule + toy gap estimate |
| `ferric_efa_cert` | Ferric-shaped EFA allow/refuse (not path-dep'd Ferric) |
| `quantum_gate_ops` | Classical 1-qubit statevector (H/X) |
| `analog_crossbar_mac` | Digital stand-in y=Wx |
| `photonic_mzi` | Classical 2×2 MZI transfer (not photonic silicon) |

## API

```rust
use mol_adapters::{
    advance_all_hw_gaps_soft, probe_hw_gaps, run_hw_gap_soft, HwGapId,
};

let inv = probe_hw_gaps();
assert_eq!(inv.soft_ref_count, 8);
assert!(!inv.stage_c_measured);

let r = run_hw_gap_soft(HwGapId::PhysicalSettle, "soft-ref advance demo");
assert!(r.measured_j.is_none());
assert!(!r.silicon_claimed);

let all = advance_all_hw_gaps_soft();
assert_eq!(all.len(), 8);
```

CLI: `mol prove` criterion `product_hw_gaps_soft_ref`; `mol dev` prints soft-ref counts.

See also: [`fpga-stage-c.md`](./fpga-stage-c.md), [`multi-fabric-compute.md`](./multi-fabric-compute.md), BLUEPRINT §QI/thermo.
