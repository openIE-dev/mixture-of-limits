# Mixture of Limits (MoL) — Architecture Blueprint

**Version:** 0.1.0  
**Author thesis:** David Charlot / OpenIE  
**Companion law sites:** [compute.openie.dev](https://compute.openie.dev/) · [proof.openie.dev](https://proof.openie.dev/) · MathGround TR-2026-06  

This document is the authoritative architecture for the `mixture-of-limits` workspace. The Rust crates in `crates/` implement the v0.1 standalone law. Sibling trees (`openie-leapfrog`, `jouledb`, `wca-lut-edge`) are **read for patterns**; adapters are stubs by default.

---

## 1. Law statement (MoL vs MoE)

### Mixture of Limits

**Mixture of Limits** is the universal law of computer intelligence: there exist **floors** where more bits stop buying outcomes. Past a floor, additional tokens, parameters, or joules do not purchase verifiable progress on the task coordinate. Intelligence is therefore a **navigation law** over a structured stack of primitives — not an invitation to dump residual capacity into a generative corridor.

Consequences:

1. **Mathematical compression / formulas beat excess tokens** when the grammar is covered.
2. **Neural nets are a flawed default construct for computer intelligence (CI)** when used as the substrate rather than a residual leaf.
3. Routing is **cheapest-sufficient**: escalate only on miss; refuse when escalation is unsafe or VoI-negative.
4. Proof / energy law: **`E(x) ≥ θ(D)·μ(S,V)`** — joules receipts are first-class; Landauer is the thermodynamic lower bound, never confused with RAPL/NVML.
5. **Available devices / multi-fabric**: optimize compute across **all known devices**, not one accelerator. After the cascade closes a gear, pick the cheapest sufficient `DeviceKind` (`Cpu`, `GpuMetal`/`GpuVulkan`/`GpuWebGpu`, `WasmBrowser`, `ThermoSettle`, `NeuromorphicSpike`, `LutGate`, …). Soft-ref: Cpu always present; others optional presence flags. Ferric (Physical AI BMI) is **reference semantics** for same-kernels-everywhere (wgpu fabrics) — **not** path-dep'd into MoL prove ([PLAN.md](./PLAN.md) §1b).

### Not Mixture of Experts

| MoE (generative corridor) | MoL (navigation law) |
|---|---|
| Experts live *inside* a model; gate selects parameters | Limits live *outside* generation; floors stop escalation |
| More experts → more capacity for the same generative act | More named floors → sharper refuse / cheaper close |
| Success = next-token likelihood | Success = closed grammar + certified commit + joule receipt |
| Model is the substrate | Model is a **demoted residual leaf** |

MoL navigates the **Periodic Stack** (compute.openie.dev: **258 primitives / 33 families**). It does not rebrand MoE.

---

## 2. Limit surface

Named floors (registry order in `mol-limits`; first binding wins where applicable):

| Limit id | Kind | Meaning |
|---|---|---|
| `safety` | Safety | Capability / policy deny (automate path) |
| `energy` | Energy | Joule budget floor (`max_j`); Landauer-aware estimates |
| `latency` | Latency | Soft wall-clock ceiling (not RAPL) |
| `voi` | ValueOfInformation | Marginal bits worthless under budget — **stop** |
| `grammar` | GrammarCoverage | Covered by Lookup/Formula/Solver — or refuse unknown |
| `primitive_gap` | PrimitiveGap | Missing Periodic Stack primitive — **registry probe** over in-tree subset (Gap markers / absent names) |
| `wca_refuse` | WcaRefuse | WCA certify refuse (automate / live MCP later) |
| `information` | Information | Information-theoretic sufficiency (more bits ≠ outcomes) |

### Floors in plain language

- **Information floor** — Shannon / task-coordinate sufficiency; extra tokens are noise.
- **Joule floor (Landauer)** — `E_min = k_B T ln 2` per bit erased; receipts annotate `landauer_floor_J` as **estimate ≠ measured**.
- **VoI stop** — escalate only when marginal utility exceeds threshold; else refuse excess.
- **Grammar-coverage stop** — if Lookup/Formula/Solver cover the query, do not open the model.
- **WCA refuse** — propose → certify → **commit|refuse**; refuse over escalate when unsafe.
- **Primitive gaps** — if the Periodic Stack lacks a primitive for the coordinate, surface the gap (do not hallucinate coverage).

---

## 3. Cascade gears (mapped)

MathGround TR-2026-06 cascade (type-enforced replay classes):

```text
Lookup → Closed-form (Formula) → Sparse solver → Stochastic model LAST
```

Cross-map:

| MoL `CascadeTier` | MathGround | OpenIE zone | JouleDB tier | WCA / System One | Thermo |
|---|---|---|---|---|---|
| Lookup | Lookup | Z1 | Lookup | LUT sense / allow table | L0 |
| Formula | Closed-form | Z1 | Formula | O(1) certificate inputs | L0 |
| Solver | Sparse solver | Z1/Z2 | Extract / Aggregate | Constrained certify | L1 |
| Model | Stochastic LAST | Z3 | Reason | Proposal residual only; **never bypasses commit** | L2max |

OpenIE **Z1 → Z2 → Z3** is the zone cascade; MoL tiers refine the gear teeth inside that philosophy. WCA **propose / certify / commit|refuse** owns irreversible side effects. System One is a **low-joule pre-gate** (choice/score/noul) — it may enrich a proposal; it never bypasses WCA. JouleDB’s **Lookup → Formula → Extract → Aggregate → Reason** is the DB-axis twin; MoL adapters map kinds offline in v0.1.

**Replayability classes (type-enforced):**

- `Deterministic` — closed-form / LUT / proof  
- `RetrievedCited` — retrieve + cite  
- `Composed` — deterministic composition  
- `ModelGenerated` — stochastic  

**Invariant:** `ModelGenerated` cannot coerce to `Deterministic` (`TypedAnswer` + sealed markers in `mol-core`).

QI / thermo **collapse intelligence** mapping of these gears (and Periodic Stack empty cells): §4 and [docs/quantum-thermo-collapse.md](./docs/quantum-thermo-collapse.md).

---

## 4. Collapse intelligence gears (QI + thermo)

**Thesis:** formula always wins when the grammar is covered. Quantum information science and thermodynamic / reversible computing inspire **compute that collapses intelligence into physics** — unitary / reversible structure, Landauer erase cost, energy-landscape settle = answer, adiabatic / Ising schedules, LUT certify. MoL **navigates** those gears; the NN residual stays **LAST**.

Companion note (detail, honesty bounds): **[docs/quantum-thermo-collapse.md](./docs/quantum-thermo-collapse.md)**.

| QI / thermo idea | Cascade gear | Periodic Stack | MoL floor / receipt |
|---|---|---|---|
| LUT / allow-table certify (reversible read) | **Lookup** | Sense / allow primitives | `grammar`; WCA certify |
| Closed-form / identity rewrite (unitary-like on paper) | **Formula** | Closed identities | `grammar`; Landauer bits ≈ 0 if no erase |
| Energy-landscape settle + certify (Ising / adiabatic *inspiration*) | **Solver** | Empty cells until named | `energy` / VoI; certify before commit |
| Stochastic generative residual | **Model LAST** | Not substrate | `voi`; `allow_model=false` default |

```text
Lookup → Formula → Solver → Model LAST
  L0        L0       L1         L2max     (ThermoClass)
```

**Honesty:** Landauer (`E_min = k_B T ln 2` per bit erased) is a **thermodynamic lower-bound estimate** already annotated on receipts — never equal to RAPL/NVML/`measured_j`. MoL v0.1 does **not** claim quantum hardware, adiabatic wall joules, or annealer drivers. Settle-then-certify is wired as Solver/`settle_refuse` + EFA/WCA certify on `close`; reserved Periodic Stack id **`physical_settle`** still surfaces via `primitive_gap` when named as missing. Empty Periodic Stack cells → `primitive_gap` (phase 3), do not hallucinate QI coverage.

Coin-cell rhyme (§10): O(1) Formula + LUT refuse are the edge power move; Model leaf stays cold.

---

## 5. Automation loop

```text
sense → classify grammar → escalate cheapest sufficient → certify → commit|refuse → receipt
```

Implemented as:

1. **Sense / classify** — `QueryKind::classify` (heuristic, not an NN).  
2. **MixtureOfLimits** — evaluate named limits in order; VoI may refuse free-form before cascade.  
3. **CascadeEngine** — Lookup → Formula → Solver; Model skipped unless `allow_model`.  
4. **AutomateGate** — capability-gated `Act`; Mutate/External default deny; destructive patterns refused.  
5. **Receipt** — estimated_j, optional measured_j, limit_fired, cascade_steps, `board_synth_claimed=false`, optional HMAC.

**Rule:** refuse over escalate when unsafe.

---

## 6. NN / model demotion

- No trained weights in this workspace. No `candle` / `tract` requirement.  
- `ModelStub` / `StubModelEndpoint` is a **labeled stub trait**.  
- Default budget: `allow_model = false` → model tier `Skipped` / `ModelRefused`.  
- Even with `allow_model`, the stub refuses unless a future adapter implements a real endpoint — and that endpoint remains a **leaf**, never the routing substrate.  
- Coin-cell / edge power is **O(1) formula + LUT refuse**, not a tiny NN.

---

## 7. Integration map

| Sibling | Role | MoL hook (v0.1) | Future |
|---|---|---|---|
| `openie-leapfrog` | Z1→Z2→Z3, Receipts, μ, System One, automate WCA gate | Patterns only; `OpenIeRuntimePort` stub | `openie-path` feature + thin ask bridge |
| `jouledb` | Energy-metered DB cascade + HDC | `StubJouleDbCascade::explain` offline map | `jouledb-path` live EXPLAIN/query |
| `wca-lut-edge` | LUT gate, propose/certify/commit, surrogate joules | `WcaCommitPort` in-crate + **HTTP/MCP live** (`MOL_WCA_CERTIFY_URL`) with fallback | Optional `wca-path` in-proc still deferred; FPGA Stage C meter stub |
| MathGround / mgai | Cascade thesis TR-2026-06 | Tier enum + replay classes | Shared schema if published |
| proof.openie.dev | Proof law / kernel | Landauer + receipt honesty | Kernel proof_hash bridge via leapfrog |
| EFA / Physical AI BMI | Energy-as-certificate before commit | `StubEfaCertificate` software-reference | Live Ferric/on-device cert (deferred) |
| **Ferric** (BMI) | Pure-Rust multi-fabric AI compute (wgpu/CPU; cloud/local/browser) | **Reference semantics only** — `mol-core::fabric` mirrors multi-device law; no path-dep | Optional future `ferric` feature; ferrotherm/ferromotion sibling fabrics |
| Klere EPU | Ternary settle-or-refuse + priced receipts | `StubKlereSettle` software-reference | Live klere-vm/FPGA (deferred) |

**Policy:** do **not** fork sibling trees. Path-deps are optional and deferred so MoL builds standalone (sibling workspaces are heavy).

---

## 8. Security / honesty

1. **No fake RAPL/NVML/powermetrics** — `measured_j: Option` plus `component_measured` (CPU/GPU/ANE/DRAM/package). `MeasureSource::{Rapl,Nvml,Powermetrics,IoReport,Smc,CpuProxy}` only when that probe returned numbers. Tier-1 live path (`energy-meter`): RAPL powercap, NVML via nvidia-smi (never util%), macOS SMC PSTR + IOReport. Missing rails stay absent (not zero). Package is not a sum of rails. Windows ETW is a later stub. Soft-ref prove A13.  
2. **Estimated vs measured labels** — `estimated_j` + `EstimateKind`; cascade uses `CascadeEstimate` / `CatalogSurrogate`.  
3. **`board_synth_claimed = false`** always (software reference).  
4. **Refuse over escalate** when unsafe (automate destructive heuristics; VoI; budget).  
5. **HMAC signing stub** (`hmac-sha256`) for receipt integrity demos — not a PKI claim; leapfrog-style honesty.  
6. Default-deny capabilities for irreversible acts.

---

## 9. Roadmap (phases 0–3)

| Phase | Goal | Status in v0.1 |
|---|---|---|
| **0** | Standalone law: types, cascade demo, limits router, CLI, receipts | **Shipped** |
| **1** | Adapters: trait ports + feature flags; document field mapping to leapfrog/WCA/JouleDB/EFA/Klere | **Software-reference stubs** (openie offline map, WCA/EFA certify, Klere settle); live path-deps deferred |
| **2** | Live WCA/NI HTTP\|MCP certify (env-gated, default off) + in-crate fallback | **Wired** (`HttpNiCertify` / `certify_live_prefer_env`); System One pre-gate still residual |
| **3** | Periodic-stack navigator (258/33): primitive_gap probe, μ calib optional | **Live catalog shipped** (33 families; 258 Present with 258 real Lookup/Formula/Solver/Navigate gears + honest Gaps; scale vs 258). HW Gaps soft-ref sims ×8 in proof (Gap cells retained). μ calib + silicon HW still residual |

---

## 10. Coin-cell / edge path

The ecosystem’s power move is not a smaller transformer:

- **O(1) formula** closes physics / Landauer / unit identities in microseconds and ~pJ-scale *surrogate* estimates.  
- **LUT refuse** stops unsafe or VoI-negative acts without generative spend.  
- Model leaf stays cold.  

`Budget::coin_cell()` encodes a tiny `max_j` + latency ceiling with `allow_model=false`. See `examples/coin_cell_formula.rs`. Collapse-intelligence rhyme (LUT certify + Landauer honesty, no fake RAPL): §4 / [docs/quantum-thermo-collapse.md](./docs/quantum-thermo-collapse.md).

---

## 11. Crate map (implemented)

| Crate | Responsibility |
|---|---|
| `mol-core` | Limit/Floor, Landauer, Joules, ReplayClass, Tier/Zone, VoI, Budget, MolError, PeriodicStack subset, **BitemporalStore**, **`fabric` (DeviceKind / inventory / route)** |
| `mol-receipt` | MolReceipt, cascade steps, HMAC sign/verify, honesty checks |
| `mol-cascade` | GrammarCoverage, Lookup/Formula/Solver+settle/Model gears, CascadeEngine |
| `mol-limits` | Named registry, MixtureOfLimits router + `close` |
| `mol-automate` | Act / CapabilitySet / AutomateGate; AgentMailbox + AgentLoop |
| `mol-adapters` | Software-reference ports: openie / wca / jouledb / efa / klere |
| `mol-cli` | `mol` binary: ask, explain-cascade, limits, receipt-verify, demo, **agent**, **memory**, **replay**, **prove** |

---

## 12. Proof law reminder

```text
E(x) ≥ θ(D) · μ(S, V)
```

Receipts carry estimated path energy and optional Landauer floor annotation. Soft measurements are labeled and optional. Never claim estimate = RAPL.

---

## 13. Adjacent-field appendix (QI/thermo + MWM / AI-Newton)

Deep mapping of sister programs / physics-compute into MoL cascade + crate borrow/refuse lists:

- **[docs/quantum-thermo-collapse.md](./docs/quantum-thermo-collapse.md)** — QI + thermodynamic / reversible computing as **collapse intelligence** gears (unitary, Landauer, Ising/adiabatic settle, LUT certify) → Lookup→Formula→Solver→Model LAST + Periodic Stack empty cells.
- **[docs/appendix-mwm-ai-newton.md](./docs/appendix-mwm-ai-newton.md)** — Mechanistic World Models (arXiv:2607.12474) and AI-Newton (arXiv:2504.01538); comparison vs Buehler SciAgents, MoL/MathGround, FM-hype; implications for `mol-cascade` / `mol-limits`.
- Ranked public inventory (append-only sweeps): **[docs/adjacent-field-hunt.md](./docs/adjacent-field-hunt.md)**.
- **[docs/physicalai-bmi-map.md](./docs/physicalai-bmi-map.md)** — Charlot Lab / JBI Physical AI (EFA, Energy Lab, thermodynamic sampling, neuromorphic) mapped onto Landauer honesty, collapse intelligence, and cascade gears. Short form: §14.
- **[docs/klere-map.md](./docs/klere-map.md)** — Adjacent field: Klere, Inc. (klere.ai) Energy Processing Unit — ternary settle-to-silence fabric, FPGA-measured pJ/accumulate receipts, priced `klere-vm` twin; MoL collapse/Solver + honesty cousin of BMI “ternary settling fabric / settle ×2,” not an OpenIE/MathGround citation.

These docs are **read-only field work** for thesis adjacency. They do not change v0.1 crate APIs (except optional comment-level honesty in `mol-core`). Sibling trees (`openie-leapfrog`, `jouledb`, `wca-lut-edge`) stay pattern-only.

---

## 14. Physical AI BMI (Charlot Lab / JBI)

Fetched 2026-09-30: [efa.physicalai-bmi.org](https://efa.physicalai-bmi.org/), [energy.physicalai-bmi.org](https://energy.physicalai-bmi.org/) (+ `/thermo`, `/neuromorphic`), hub [physicalai-bmi.org](https://physicalai-bmi.org/). Full map: **[docs/physicalai-bmi-map.md](./docs/physicalai-bmi-map.md)**.

EFA’s claim — one scalar energy is both the policy and a pre-commit Lyapunov certificate, wrapping a frozen policy it did not train — is collapse intelligence in the embodied loop: **Formula/structural identity and Solver settle, Model LAST**, refuse over commit. The Energy Lab and thermo pages price Landauer as operand motion and chip-edge crossings (fJ updates vs pJ reads), and they refuse to average modelled with measured joules (same honesty as §8). Neuromorphic adds an activity floor (spikes per synapse ≲ 1.72), not a second Landauer derivation. The sites do **not** cite MathGround, OpenIE, or JouleDB; joules-per-task and OER/1 are parallel vocabularies. MoL does not adopt Ferric, ferrotherm, or p-bit hardware, and does not import their µJ/fJ figures as `measured_j`. Multi-fabric **law** lives in `mol-core::fabric` (inventory + cheapest-sufficient route); Ferric remains reference semantics for same-kernels-everywhere ([PLAN.md](./PLAN.md) §1b: GitHub `dcharlot-physicalai-bmi/ferric`, Mac `/Users/dcharlot/vibe-coding/ferric`).

---

## 15. Path to MoL — implementation status (focused)

**Goal:** MoL is the **router that owns close** end-to-end.

| Piece | Status |
|---|---|
| Named floors bind: `grammar`, `voi`, `energy`/Landauer, `efa_certificate`, `settle_refuse`, `primitive_gap`, `wca_refuse`, `safety`, `latency`, `information` | **Wired** in `mol-limits` registry + cascade/automate |
| Periodic Stack live catalog (33 families; Present+GearKind live gears + Gaps; scale vs 258) | **Wired** in `mol-core::PeriodicStack` + `mol-cascade::catalog_gear`; Lookup navigate; live Formula/Lookup/Solver closes; `primitive_gap` probe |
| Cascade Lookup → Formula → Solver/settle → Model LAST | **Wired** (`TernarySettle` under Solver; model demoted) |
| Automate propose → certify → commit\|refuse → receipt | **Wired** (`AutomateGate` + `MixtureOfLimits::close`) |
| Thin agent mailbox loop (Goal/Message/Act → close → transcript) | **Wired** (`AgentLoop` / `AgentMailbox` in `mol-automate`; `mol agent`) |
| Bitemporal state + memory (valid-time + tx-time; write on close COMMIT) | **Wired** (`BitemporalStore` in `mol-core`; `MixtureOfLimits` commit; Mutate gate; `mol memory`) |
| Multi-fabric device inventory + routing (cheapest sufficient after tier) | **Wired** (`mol-core::fabric` + `ComputeFabric`/`ScheduleDecision`; soft-ref default; optional `fabric-detect`/`wgpu-kernel`; receipts `fabric:route` + live/soft `kernel:vector_add` with `execution_proof`; Lookup/Formula→Cpu; Model→Gpu*\|refuse; detect/kernel ≠ joules) |
| OpenIE port | **Offline zone map** stub; live `openie-path` deferred (sibling heavy) |
| NI/WCA certify | **In-crate** + **env-gated HTTP/MCP** (`certify_live_prefer_env`); Stage C soft-ref inventory wired; Ferric/FPGA meters still stub |
| EFA certificate stub (BMI propose/certify/refuse) | **Software-reference** `StubEfaCertificate`; no Ferric/MuJoCo hardware |
| Klere settle stub (ternary settle-or-refuse) | **Software-reference** `StubKlereSettle`; `measured_j=None`; no klere-vm/FPGA |
| Honesty | `estimated_j` labeled; `mu`/`mu_source=catalog`; `landauer_floor_ratio`; `measured_j=None`; `board_synth_claimed=false`; never fake RAPL |
| CLI `mol demo` | Full path: cheap close, VoI refuse, settle/cert refuse, primitive gap, ports |
| CLI `mol prove` | **PLAN.md proof harness** — exit 0 + `VERIFIED` for each criterion |
| [PLAN.md](./PLAN.md) | Goal / non-goals / architecture / checklist / proof criteria / out-of-scope |

### Proof how-to

```bash
cd /workspace/mixture-of-limits
cargo test --workspace
cargo run -p mol-cli -- prove
```

Criteria (all must print `VERIFIED`): deterministic close; formula/lookup without model (incl. Shannon/Nyquist/eV cascade); VoI refuse when `!allow_model`; settle commit + settle refuse; certificate refuse on diverge tag; capability default-deny mutate; receipt honesty (`measured_j=None`, `board_synth_claimed=false`, estimate labeled, `mu_source=catalog`); `ModelGenerated` cannot coerce to `Deterministic`; **Periodic Stack subset navigation**; **primitive_gap registry probe refuse**; **μ / impedance catalog** (`E≈θ·μ`); **receipt transcript replay** without model; **bitemporal memory**; **agent mailbox**; **fabric_routing** (Lookup/Formula→Cpu; Settle→ThermoSettle|Cpu; Model→Gpu*|refuse). Proven path is self-contained (no sibling path-deps on openie-leapfrog / jouledb / wca-lut-edge / Ferric).

**Still stub / not live (OUT OF PROOF SCOPE):** leapfrog runtime ask, **Ferric / on-device EFA hardware meters**, **FPGA Stage C / wca-lut-edge in-proc + board meter** (`stage_c_measured=false`), klere-vm WASM/FPGA meter, System One pre-gate, μ calib corpus + **silicon** HW (Gap cells retained). **In proof soft-ref:** Live NI/WCA HTTP|MCP certify; Stage C inventory; Ferric inventory; HW Gaps sims ×8 (`physical_settle`…`photonic_mzi`) — all with `stage_c_measured=false` (see [`docs/hw-gaps-soft-ref.md`](./docs/hw-gaps-soft-ref.md), [`docs/ferric-soft-ref.md`](./docs/ferric-soft-ref.md), [`docs/live-ni-wca-certify.md`](./docs/live-ni-wca-certify.md)).

