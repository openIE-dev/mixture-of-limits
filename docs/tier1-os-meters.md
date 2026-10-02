# Tier-1 OS meters — RAPL / NVML / macOS equivalent

**Law:** `measured_j` populates **only** on real Metered readings. Estimates (catalog μ, Landauer, OpCounter, cascade table) never copy into `measured_j`. Utilization % never becomes joules. Package is never a sum of rails.

| Platform | Probe | Package `measured_j` | Feature |
|---|---|---|---|
| Linux | RAPL powercap `/sys/class/powercap` `energy_uj` delta | package domain when present | `energy-meter` |
| Linux / Windows NVIDIA | `nvidia-smi --query-gpu=power.draw[,energy.consumed]` | `power.draw`×window or energy mJ delta | `energy-meter` |
| macOS | IOReport Energy Model rails + SMC `PSTR` watts×window | SMC `PSTR` (not Energy Model Package — none exists) | `energy-meter` |
| macOS root | `powermetrics` `combined_power` | when root sample returns numbers | `energy-meter` + sudo |
| Soft-ref prove | fixtures only | stamped in A13; default close stays `None` | feature off |

## CLI

```bash
cargo run -p mol-cli --features energy-meter -- meter --sample-ms 200
cargo run -p mol-cli -- prove   # A13: Tier-1 honesty + Tier-2 StubShuntHal
```

## Honesty checklist

- Feature off / VM / no driver / parse fail → `measured_j=None`, `measure_source=unavailable`
- NVML util% CSV → refuse invent
- RAPL unknown zone names ignored (not relabeled)
- IOReport rails never summed into package
- `board_synth_claimed=false` on soft-ref

See also: [macos-package-meter.md](./macos-package-meter.md), [product/ACCEPTANCE.md](../product/ACCEPTANCE.md) A13, [product/ARCHITECTURE.md](../product/ARCHITECTURE.md) measurement tiers.
