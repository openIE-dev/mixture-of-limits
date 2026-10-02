# macOS package meter hunt (2026-09-30, M5 Max, macOS 27.0.1)

## Goal
No-sudo **package / combined** joules so `FailClosedPolicy::meter_required` can COMMIT with real `measured_j` — never invent by summing cpu+gpu+ane+dram.

## Surfaces probed

| Surface | sudo? | Result |
|---|---|---|
| `powermetrics` | **still required** (`must be invoked as the superuser`) | Has `combined_power` when root; not usable no-sudo |
| IOReport `Energy Model` | no | **364** channels; CPU Energy, GPU Energy, ANE*, DRAM*, … — **no Package / Combined** |
| IOReport `CopyAllChannels` | no | **11373** channels; no package/combined energy channel |
| IOReport groups CPU/GPU/SMC/Battery | no | No joule package counter |
| SMC `PSTR` via IOKit `AppleSMC` | **no** | **flt** watts (~9–14 W idle observed) — system total power |
| SMC other keys | no | `PDTR`/`PPBR`/`ID0R` present; not used as package |
| `macmon pipe` | no | `all_power` = **client sum** cpu+gpu+ane (forbidden); `sys_power` = SMC PSTR |
| lux-energy / Nova navigator.energy | read-only | lux uses macmon `all_power` (rail sum); Nova T1-ioreport still process-time model |

## Honesty decision
- **Package `measured_j`:** SMC `PSTR` × sample window → `MeasureSource::Smc` (same sensor macmon calls `sys_power`).
- **Rails:** IOReport Energy Model → `MeasureSource::IoReport` (cpu/gpu/ane/dram).
- **Never** use macmon `all_power` / invent `cpu+gpu+ane` as package.
- `powermetrics` remains optional root fallback for Apple `combined_power`.

## Live expectation
`mol ask --meter-required --features energy-meter` on Mac → COMMIT with `measure_source=smc` and real `measured_j` when PSTR readable.

## Metal kernel overlap (2026-09-30)

`ecosystem-certify --detect --tier model --meter-required --meter-ms 200` with
`--features fabric-detect,energy-meter` samples SMC `PSTR` **during** the
vector-add kernel window (`measure_energy_during`). Kernel
`ComputeStepReceipt` + receipt stamp `measured_j` with `energy_honesty=measured`.
Never invent; never sum IOReport rails into package. Live capture:
`docs/_live_metal_kernel_meter_stamp.txt`.
