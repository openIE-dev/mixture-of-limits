# MoL tiny wgpu / Metal compute kernel

Clean-room **vector-add** WGSL under MoL certify — prove that a scheduled Gpu*
fabric actually *runs* compute and stamps a receipt proof. **Never invent
`measured_j`.** Ferric is reference semantics only (not vendored).

## What

| Piece | Detail |
|---|---|
| Kernel | `c[i] = a[i] + b[i]` for `n=64` (`a[i]=i`, `b[i]=2i` → `c[i]=3i`) |
| WGSL | In-tree `mol-core::kernel` (`VECTOR_ADD_WGSL`); workgroup size 64 |
| Feature | `fabric-detect` (alias `wgpu-kernel`) — optional; soft-ref prove stays offline |
| Soft stub | Same CPU math + checksum when feature off / no matching adapter |
| Skip | Formula/Lookup → Cpu schedule stamps `kernel:vector_add` skip (honest) |

## Receipt stamp

`ComputeStepReceipt` label `kernel:vector_add`:

- `fabric_id` = scheduled fabric (`gpu_metal` on Apple Metal)
- `estimated_j` analytical only
- `measured_j=None` unless a real meter is attached elsewhere
- `execution_proof` e.g. `checksum=0x…;n=64;sample=[0,3,6,9,…];mode=live:metal;live=true`

## Wire-in

`EcosystemCertify` after `fabric:route`:

1. Schedule fabric (Model → cheapest Gpu*)
2. `run_tiny_vector_add_if_gpu` → stamp `kernel:vector_add`
3. WASM capsule + GrantReceipt → COMMIT

```bash
# Soft-ref prove (no GPU required)
cargo run -p mol-cli -- prove
# looks for: VERIFIED ecosystem_wgpu_kernel

# Mac live Metal
cargo run -p mol-cli --features fabric-detect -- ecosystem-certify --detect --tier model
# expect: fabric_chosen=Some(GpuMetal), kernel_step … mode=live:metal;live=true
```

## Honesty

Detection and kernel dispatch mark **compute proof**, not joules. Package
`measured_j` requires `energy-meter` / a real meter sample — **never invented**,
never a rail sum.

## Metal + SMC overlapping meter (`meter_required`)

One run stamps the kernel `ComputeStepReceipt` with real package joules:

1. `--detect --tier model` → schedule `gpu_metal`
2. `measure_energy_during` overlaps SMC `PSTR` (+ IOReport rails) with vector-add
3. `--meter-required` → COMMIT only when package `measured_j` is real
4. Kernel step + receipt: `energy_honesty=measured`, `measure_source=smc`

```bash
# Soft-ref prove (fixture path OK; no live silicon)
cargo run -p mol-cli -- prove

# Mac live: Metal kernel + SMC package in one overlapping window
cargo run -p mol-cli --features fabric-detect,energy-meter -- \
  ecosystem-certify --detect --tier model --meter-required --meter-ms 200
```
