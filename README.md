# Mixture of Limits (MoL)

**The universal law of computer intelligence:** floors where more bits stop buying outcomes.

Mathematical compression / formulas beat excess tokens. Neural nets are a **demoted residual leaf**, not the substrate. Navigation law over the [Periodic Stack](https://compute.openie.dev/) (258 primitives / 33 families) — **not** MoE inside the generative corridor.

Proof law: `E(x) ≥ θ(D)·μ(S,V)`. Cascade: **Lookup → Formula → Solver/settle → Model LAST**. MoL **owns close**: propose → certify → commit|refuse → receipt.

> Author: David Charlot / OpenIE · License: Apache-2.0 OR MIT · Rust 1.98 · edition 2024 · v0.1.0

Full architecture: **[BLUEPRINT.md](./BLUEPRINT.md)** (§15 path-to-MoL status).  
Proof plan + criteria: **[PLAN.md](./PLAN.md)**.

---

## Thesis one-pager

| Claim | MoL response |
|---|---|
| More tokens ⇒ more intelligence | **False past a floor** — VoI / grammar / joule limits bind |
| Default CI construct = neural net | **Flawed default** — NN is residual; formulas/LUT first |
| MoE scales experts in-model | MoL scales **named floors** outside generation |
| Energy is a footnote | **Joule receipts** + Landauer floor on every path |
| Unsafe escalate | **Refuse over escalate** (WCA / EFA certificate / Klere settle) |

Automation loop: `sense → classify → cheapest sufficient → certify → commit|refuse → receipt`.

---

## Build / test / run

Requires Rust **1.98** (`rust-toolchain.toml`).

```bash
cd /workspace/mixture-of-limits
cargo test --workspace
cargo run -p mol-cli -- prove    # PLAN.md + product A1–A14 — must print VERIFIED and exit 0
cargo run -p mol-cli -- run --chore product/mol.yaml
cargo run -p mol-cli -- bench    # J/query MoL vs always-model / MoE-sim (Estimated|Metered)
cargo run -p mol-cli -- arena    # head-on typed/ticket/risk vs frontier_sim + system_one_leaf (+ optional real_leaf)
cargo run -p mol-cli -- phase1 "please close ticket as R-OK"
cargo run -p mol-cli -- distill "ticket summary" --store /tmp/mol-distill.json
cargo run -p mol-cli -- agent    # thin mailbox loop: Goal→close→transcript
cargo run -p mol-cli -- memory   # bitemporal state+memory demo
cargo run -p mol-cli -- demo
cargo run -p mol-cli -- ecosystem-certify   # Agent Lane + fabric + WASM → one receipt
cargo run -p mol-cli -- ask "landauer joules per bit"
cargo run -p mol-cli -- ask "landauer joules per bit" --close
cargo run -p mol-cli -- ask "convert 100 celsius to fahrenheit"
cargo run -p mol-cli -- ask "settle ternary [1, 1, 1, 1]"
cargo run -p mol-cli -- ask "settle refuse will not settle [1,1,1]"
cargo run -p mol-cli -- ask "write a poem about GPUs"   # VoI refuse
cargo run -p mol-cli -- ask "primitive gap: physical_settle missing"
cargo run -p mol-cli -- explain-cascade "solve 2x2 [[1,0],[0,1]] [3,4]"
cargo run -p mol-cli -- limits
cargo run -p mol-cli -- meter
cargo run -p mol-cli --features energy-meter -- meter --sample-ms 20
cargo test -p mol-desktop
# Mac / Linux with display — energy harness GUI (egui; not Electron):
cargo run -p mol-desktop --features gui --bin mol-desktop
```

Honesty: receipts print `estimated_j` and `measured_j=None` unless a real meter ran (`energy-meter`). **No fake RAPL/NVML.** `board_synth_claimed=false`. Ferric / FPGA Stage C certify meters remain stubs (`stage_c_measured=false`).

Agent Lane session path (partitions + host invoke grant receipts): [`docs/agent-lane-session.md`](./docs/agent-lane-session.md). WASM capsule: [`docs/wasm-capsule.md`](./docs/wasm-capsule.md). Ecosystem e2e certify: [`docs/ecosystem-e2e-certify.md`](./docs/ecosystem-e2e-certify.md). Live NI/WCA HTTP|MCP certify: [`docs/live-ni-wca-certify.md`](./docs/live-ni-wca-certify.md). Multi-fabric compute: [`docs/multi-fabric-compute.md`](./docs/multi-fabric-compute.md).

---

## Workspace layout

```text
mixture-of-limits/
  BLUEPRINT.md          # law + §15 path-to-MoL status
  crates/
    mol-core/           # floors (incl. efa_certificate, settle_refuse), Landauer, tiers, VoI
    mol-receipt/        # MolReceipt + HMAC stub
    mol-cascade/        # Lookup / Formula / Solver+settle / Model stub
    mol-limits/         # MixtureOfLimits::route + ::close
    mol-automate/       # propose → certify → Commit|Refuse; AgentMailbox loop
    mol-adapters/       # openie / wca / jouledb / efa / klere + InCrateNiCertify
    mol-cli/            # `mol` binary (ask, run, bench, phase1, distill, prove, …)
    product/            # Mixture of Limits product docs + mol.yaml (A1–A14)
    mol-desktop/        # energy harness shell (headless API + optional egui GUI)
```

Sibling trees (patterns only — not forked): `openie-leapfrog`, `jouledb`, `wca-lut-edge`. EFA/Klere are software-reference ports (no Ferric / klere-vm dependency).

---

## Key types

- `CascadeTier::{Lookup,Formula,Solver,Model}` — settle under Solver
- `FloorKind` — `voi`, `energy`, `grammar`, `efa_certificate`, `settle_refuse`, `primitive_gap`, …
- `MixtureOfLimits::route` / `::close` — close = route + EFA/WCA certify + receipt
- `AutomateGate::gate` — capability + certify before commit
- `InCrateNiCertify` + `HttpNiCertify` — live NI/EFA/WCA certify (HTTP|MCP env-gated, in-crate fallback); Ferric/FPGA Stage C still stub
- `ResidualModelAdapter` — Model LAST under VoI>0 + budget + cert
- `EpisodeStore` / `Phase1Config` / `StubShuntHal` / `DistillStore`
- `StubEfaCertificate` / `StubKlereSettle` / `StubOpenIeRuntime` / `StubWcaCommit`

---

## Proof

```bash
cargo test --workspace
cargo run -p mol-cli -- prove
```

`mol prove` checks: deterministic close, formula/lookup without model, VoI refuse, settle commit+refuse, EFA diverge refuse, capability default-deny mutate, receipt honesty (`measured_j=None`, `board_synth=false`, estimate labeled, `mu_source=catalog`), `ModelGenerated` ↛ `Deterministic`, Periodic Stack live-catalog navigation, live Lookup/Formula/Solver gear samples, primitive_gap probe, **μ/impedance catalog** (`E≈θ·μ`), **receipt transcript replay** (no model), Z2 cite + Z1 compose, **agent mailbox loop** (Goal/Message/Act → close → transcript; model cold), **bitemporal state+memory**, **desktop shell headless** (joule ledger / receipt view), and **OS meter honesty** (never invent `measured_j`). See [PLAN.md](./PLAN.md).

Optional `fabric-detect` (wgpu) and `energy-meter`/`os-meter` (Linux RAPL/powercap; Tier-1 NVML via `nvidia-smi` power.draw×window or energy.consumed delta; macOS IOReport rails + SMC PSTR package, or root powermetrics) are feature-gated. Soft-ref prove keeps `measured_j=None`. A failed or VM probe stays `unavailable` and never invents numbers (utilization % never becomes joules). See `mol meter` and product acceptance A13. Details: [docs/tier1-os-meters.md](./docs/tier1-os-meters.md).

Live silicon / RAPL / Ferric EFA hardware / klere-vm FPGA / openie-path / WCA MCP / remaining ~168/258 thesis primitives are **OUT OF PROOF SCOPE**. In-tree Periodic Stack **live catalog** (~89 Present / ≥80 live Lookup·Formula·Solver gears + Gap markers) + registry `primitive_gap` + soft-ref **multi-fabric routing** (`mol-core::fabric`) are **in proof**. Ferric (github.com/dcharlot-physicalai-bmi/ferric) is reference semantics only — not path-dep'd.

## Gaps still stub

Live `openie-path` ask, WCA MCP, Ferric/on-device EFA certificate, klere-vm/FPGA meter, remaining thesis primitives toward 258 + μ calib corpus. Default build ships growing live catalog (honest counts in `scale_note` / `mol prove`).

---

## Product embodiment

Operator product surface (valued product docs + `mol.yaml` + acceptance): [`product/`](./product/).

```bash
cargo run -p mol-cli -- run --chore product/mol.yaml
```

See `product/PRODUCT.md` (vs Laya System One), `product/ACCEPTANCE.md`, `product/ARCHITECTURE.md`.

## Docs

| Doc | Role |
|---|---|
| [PLAN.md](./PLAN.md) | Goal, non-goals, architecture, checklist, proof criteria |
| [BLUEPRINT.md](./BLUEPRINT.md) | Architecture law + path-to-MoL status |
| [docs/quantum-thermo-collapse.md](./docs/quantum-thermo-collapse.md) | QI + thermo collapse gears |
| [docs/physicalai-bmi-map.md](./docs/physicalai-bmi-map.md) | EFA / Energy Lab / neuromorphic map |
| [docs/klere-map.md](./docs/klere-map.md) | Klere EPU / ternary settle map |
| [docs/mol-desktop.md](./docs/mol-desktop.md) | Energy harness desktop — dominance vs IDE/agent chat; egui; Mac runbook |
| [docs/tier1-os-meters.md](./docs/tier1-os-meters.md) | Tier-1 RAPL / NVML / macOS SMC·IOReport — measured_j only on real readings |
| [docs/wasm-capsule.md](./docs/wasm-capsule.md) | WASM capsule certify — stub/wasmtime, fuel estimate honesty, fail-closed |
| [docs/ecosystem-e2e-certify.md](./docs/ecosystem-e2e-certify.md) | End-to-end ecosystem certify — Agent Lane + fabric + WASM + GrantReceipt → one receipt |
| [docs/live-ni-wca-certify.md](./docs/live-ni-wca-certify.md) | Live NI/WCA HTTP\|MCP certify + in-crate fallback; Ferric/FPGA honesty |
