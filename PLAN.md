# Mixture of Limits (MoL) — Implementation Plan

**Workspace:** `/workspace/mixture-of-limits`  
**Author:** David Charlot / OpenIE  
**Rust:** 1.98 · edition 2024 · v0.1.0  
**Policy:** Clean-room law + runtime. Sibling OpenIE / BMI / Klere trees are **reference semantics only** — adapters may document ports, but the proven path is self-contained (no path-deps on `openie-leapfrog`, `jouledb`, or `wca-lut-edge`).

---

## 1. Goal

MoL is the **universal CI law** and a **runnable close path**:

- Named **floors** bind where more bits stop buying outcomes.
- Cascade: **Lookup → Formula → Solver/settle → Model LAST**.
- MoL **owns close**: sense → classify → cheapest sufficient → **certify → commit|refuse → receipt**.
- Proof law: `E(x) ≥ θ(D)·μ(S,V)` with honest joule receipts (Landauer as estimate ≠ RAPL).
- Type-enforced **ReplayClass**: `ModelGenerated` cannot coerce to `Deterministic`.

Stop condition for this plan: `PLAN.md` exists, `cargo test --workspace` green, and `cargo run -p mol-cli -- prove` exits 0 printing **VERIFIED** for every criterion below.

---


---

## 1b. Ferric reference (Physical AI BMI) — not path-dep'd

David Charlot built **Ferric** (Physical AI BMI): one pure-Rust AI compute fabric across cloud/local/browser; wgpu backends (WebGPU/Vulkan/Metal/DX12); CPU reference; eventual heterogeneous scheduler; crates `ferric-core` / `tensor` / `onnx` / `load` / `llama` / `web`. Sibling fabrics: **ferrotherm** (thermodynamic), **ferromotion**.

| Fact | Value |
|---|---|
| GitHub | https://github.com/dcharlot-physicalai-bmi/ferric |
| Mac path | `/Users/dcharlot/vibe-coding/ferric` (+ worktrees `ferric-wt-*`) |
| MoL policy | Clean-room in `/workspace/mixture-of-limits` — **do not** path-dep Ferric for prove (optional feature later OK) |
| MoL use | Ferric = **reference semantics** for same-kernels-everywhere; MoL implements multi-fabric **as law** (`mol-core::fabric`) choosing cheapest sufficient device for a closed gear |

Live Ferric EFA / silicon meters remain **OUT OF PROOF SCOPE**. Soft-ref fabric inventory + routing are **in proof**.



## 1c. MoL Desktop — energy harness (dominate IDE/agent chat)

Product doc: [`docs/mol-desktop.md`](./docs/mol-desktop.md).

**Not a chat clone** of VS Code / Cursor / Zed / Grok Bot / Pi / Hermes / Claude Code. Dominance axes:

1. **Joules-per-verified-decision** + Landauer ratio on every act (others show tokens/$)
2. **MoL close owns commit|refuse** — model never substrate; cite/compose typed replay
3. **Multi-fabric inventory** (CPU/Metal/Vulkan/WebGPU/thermo/LUT) — not one GPU default
4. **Full transaction ledger** — transcript replay, HMAC receipts, bitemporal memory, capability deny
5. **Machine protocol lane first** (MCP-class `ShellSession`); human only on consequential confirm (interface tax)
6. **Energy harness UX** — floors fired, cascade steps, fabric chosen — IDE features secondary
7. **Cross-platform Rust native** — **egui/eframe** chosen (Linux box + Mac; no Electron). Tauri deferred (WebView/webkit tax).

Scaffold: crate `mol-desktop` — headless `ShellSession` always; optional `--features gui` window. Soft-ref prove stays green offline.

**OS energy meter (optional `energy-meter` / `os-meter`) — Tier-1:** Linux RAPL/powercap domains (package/core/dram/gpu). **NVML** via `nvidia-smi` (`power.draw`×window or `energy.consumed` mJ delta → package `measured_j`; never util%). macOS rails **CPU, GPU, ANE, DRAM** via IOReport Energy Model + package via SMC `PSTR` (no sudo); `powermetrics` combined_power still needs root. Windows ETW is a later stub. `measured_j` is the package/combined counter only; `component_measured` holds rails that were actually read. Fail/VM/feature-off → `measured_j=None`, `measure_source=unavailable` — **never invent**, never sum rails into package. Soft-ref `mol prove` A13 covers fixtures + honesty.

## 2. Non-goals

| Non-goal | Why |
|---|---|
| Fluff / survey papers as deliverable | Docs may map adjacent fields; they are not the proof |
| Fake RAPL / NVML / inventing `measured_j` | Honesty invariant; software-ref always `measured_j=None` |
| MoE chase (more in-model experts) | MoL is navigation law over floors, not generative corridor capacity |
| ARC-AGI (or similar) as yardstick | Success = closed grammar + certified commit + receipt honesty |
| Live silicon / Ferric / klere-vm FPGA / Stage C board meter | Roadmap residual — **OUT OF PROOF SCOPE** (HTTP/MCP NI certify is in-tree, env-gated) |
| Path-depending on sibling workspaces for the proof | Adapters stay documented ports; prove path is in-tree |

---

## 3. Architecture

```text
sense → classify grammar → MixtureOfLimits floors
      → CascadeEngine (Lookup → Formula → Z2 retrieve → Z1 compose → Solver/settle → Model LAST)
      → fabric route (cheapest sufficient DeviceKind for closed tier)
      → certify (EFA-style + WCA-style software-ref)
      → commit | refuse
      → MolReceipt (estimated labeled; measured_j=None; board_synth=false; fabric_chosen + fabric_inventory)
```

| Piece | Role |
|---|---|
| **Floors** | Named limits (`voi`, `energy`, `grammar`, `efa_certificate`, `settle_refuse`, `primitive_gap`, `wca_refuse`, `safety`, `latency`, `information`) — first bind wins where applicable |
| **Cascade** | Cheapest-sufficient gears; Model demoted (`allow_model=false` default) |
| **Certify** | EFA-style energy-as-certificate + WCA-style commit gate (in-tree software-reference) |
| **Commit \| Refuse** | AutomateGate + `MixtureOfLimits::close`; refuse over escalate when unsafe |
| **Agent mailbox loop** | Goal / Message / Act proposals → sense → classify → cascade/floors → certify → commit|refuse → receipt into mailbox transcript; `allow_model=false` (no silent model escalate) |
| **Receipts** | `estimated_j` + `EstimateKind`; **`mu` / `mu_source=catalog`**; `landauer_floor_ratio`; `measured_j: Option` (None in soft-ref); `board_synth_claimed=false` |
| **ReplayClass** | `Deterministic` / `RetrievedCited` / `Composed` / `ModelGenerated` — type markers; no strengthen |
| **Z2 claims** | In-tree `ClaimStore` (seed: Landauer, MoL law, stack, honesty); retrieve+cite on factual hit; refuse unknown |
| **Z1 compose** | Fuse ≥2 cited claims → `ReplayClass::Composed` + `SynthesisReceipt` (`composed_from`); after retrieve / explicit compose; refuse if required claim missing; never launder as Deterministic/RetrievedCited alone |
| **Bitemporal memory** | Valid-time + transaction-time `BitemporalStore`; **writes only on MoL close COMMIT** (refuse VoI / Mutate capability); agent recall → `RetrievedCited` + `memory:` cite ids |
| **Multi-fabric** | Available devices across Cpu / Gpu* / Wasm / ThermoSettle / Neuromorphic / LutGate — not one accelerator; after cascade tier, pick cheapest sufficient fabric; soft-ref: Cpu always; optional `fabric-detect` (wgpu) maps Metal/Vulkan/WebGPU; Model→Gpu*|refuse; detect ≠ joules |

---

## 4. Implementation checklist (crates / modules)

| Crate | Responsibility | Proof role |
|---|---|---|
| `mol-core` | Floors, Landauer, Joules, **μ catalog / impedance** (`E≈θ·μ`), ReplayClass/`TypedAnswer`, Tier/Zone, VoI, Budget, MolError, **PeriodicStack subset navigator**, **KnowledgeClaim store**, **BitemporalStore**, **`fabric` (DeviceKind / FabricInventory / route_fabric; optional `fabric-detect` wgpu probe)** | Replay coercion; Landauer honesty; μ catalog; stack probe; Z2 cite; bitemporal; multi-fabric routing; soft-ref inventory |
| `mol-receipt` | `MolReceipt` (+ `mu`/`mu_source`/`landauer_floor_ratio` + **`SynthesisReceipt`/`composed_from`** + **`fabric_chosen`/`fabric_inventory`**), cascade steps, HMAC, **close transcript JSONL replay** | Receipt honesty + μ + fabric provenance + compose; replay without model |
| `mol-cascade` | GrammarCoverage; Lookup / Formula / **Z2 ClaimRetrieve** / **Z1 ClaimCompose** (Composed+`composed_from`; refuse `compose_missing`) / Solver+`TernarySettle` / Model stub; `CascadeEngine` | Formula/lookup/settle/cite/compose without model |
| `mol-limits` | Named registry; `MixtureOfLimits::route` + `::close`; `primitive_gap` ← stack probe; **memory write-on-COMMIT** | Deterministic close; VoI; certificate refuse; gap refuse; bitemporal commit |
| `mol-automate` | Act / CapabilitySet / `AutomateGate`; **`AgentMailbox` + `AgentLoop`**; **`EcosystemCertify`** (Agent Lane + fabric + WASM + GrantReceipt → one receipt); **memory write requires Mutate**; `run_memory_demo` | Capability default-deny mutate; agent-loop; bitemporal memory gate; ecosystem e2e certify |
| `mol-adapters` | Ports: openie / wca / jouledb / efa / klere + **HttpNiCertify** (HTTP\|MCP) + `InCrateNiCertify` fallback | Live NI certify env-gated; Ferric/FPGA stub; EFA+Klere in-tree |
| `mol-cli` | `mol` binary: ask, explain-cascade, limits, receipt-verify, demo, **`agent`**, **`memory`**, **`fabric`**, **`meter`**, **`ecosystem-certify`**, **`replay`**, **`prove`** | `mol prove` exits 0; `mol agent`; `mol memory`; `mol fabric`; `mol meter`; `mol ecosystem-certify`; `mol replay` |
| `mol-desktop` | Energy harness shell: `ShellSession` ask/close, fabric, joule ledger, transcript; optional egui `gui` | Headless shell API in prove (P19); GUI optional |

Deepen clean-room implementations over stubs that fail proof. Ternary settle, EFA-style certificate, floors, and cascade must be **real logic in-tree**.

---

## 5. Proof criteria (all must pass)

| # | Criterion | How proven |
|---|---|---|
| P0 | `cargo test --workspace` green | CI / operator |
| P1 | `cargo run -p mol-cli -- prove` exits 0 | Prints `VERIFIED` per criterion below |
| P2 | **Deterministic** — same asks → same close outcomes (commit/refuse + limit id) | Double-run compare in `prove` |
| P3 | **Formula/lookup close without model** | Close Landauer / unit convert → COMMIT; tier Lookup or Formula |
| P4 | **VoI refuse** free-form when `!allow_model` | Free-form ask → REFUSE; limit id `voi` |
| P5 | **Settle commit + settle refuse** | Ternary settle → COMMIT; `will not settle` → REFUSE `settle_refuse` |
| P6 | **Certificate refuse on diverge tag** | Formula query + `diverge` → REFUSE `efa_certificate` |
| P7 | **Capability default-deny mutate** | `AutomateGate::default().gate(Mutate)` → Refuse |
| P8 | **Receipt honesty** | `measured_j` always `None` in software-ref; `board_synth_claimed=false`; estimate labeled (`EstimateKind`) |
| P9 | **ModelGenerated ↛ Deterministic** | `TypedAnswer::weaken_to` returns `ReplayCoercion` error (type/test) |
| P10 | **Periodic Stack live catalog navigation** | Navigate family + present primitive + scale → COMMIT at Lookup; live gear counts in scale_note; cites 258/33 honesty |
| P11 | **primitive_gap via registry probe** | Gap marker `physical_settle` + absent name → REFUSE `primitive_gap` (not string-only) |
| P12 | **μ / impedance catalog** | Catalog μ per Lookup/Formula/Solver/Model; receipts stamp `mu_source=catalog` + `mu` + `landauer_floor_ratio`; `E≈θ·μ` (not fake RAPL) |
| P13 | **Receipt replay** | JSONL/in-memory close transcript; replaying reproduces commit/refuse + limit id; model never answered |
| P14 | **Z2 retrieve+cite** | Factual ask matching seeded claims → COMMIT with citation ids + `ReplayClass::RetrievedCited` + zone Z2; unknown factual → REFUSE `claim_unknown` (no invent) |
| P15 | **Z1 compose/synthesis** | Compose ≥2 cited claims → COMMIT with `ReplayClass::Composed` + zone Z1 + `synthesis.composed_from` / citation ids; never Deterministic or RetrievedCited alone; missing required claim → REFUSE `compose_missing` |
| P16 | **Agent mailbox loop** | Seed Goal/Message/Act → sense → classify → AutomateGate/`close` → commit|refuse + receipt in mailbox transcript; demo hits formula + cite + compose COMMIT and voi + settle_refuse REFUSE; model never answered; `measured_j=None`; `board_synth_claimed=false` |
| P17 | **Bitemporal state + memory** | Valid-time + transaction-time store; writes **only** via MoL close COMMIT (route alone does not mutate); Mutate capability default-deny refuses remember; free-form remember → REFUSE `voi`; after COMMIT, agent `recall` → COMMIT `RetrievedCited` + `memory:` citation; unknown key → REFUSE `memory_unknown`; `measured_j=None`; `board_synth_claimed=false` |
| P18 | **Fabric routing + soft-ref inventory** | Soft-ref inventory **valid offline** (`source=software_ref`, Cpu always, no spurious GPU); Lookup/Formula → `fabric_chosen=Cpu`; Settle → ThermoSettle\|Cpu; Model residual → Gpu* or REFUSE `fabric_unavailable`; mock Metal→GpuMetal / Vulkan→GpuVulkan / WebGPU→GpuWebGpu; receipts stamp `fabric_chosen` + `fabric_inventory`; **detect ≠ measured joules** (no fake RAPL); optional `fabric-detect` wgpu probe is **not** required for prove (live test `#[ignore]` / feature-gated); Ferric not path-dep'd |
| P19 | **Desktop shell headless** | `mol-desktop::ShellSession` ask/close → receipt view (zone/fabric/limit/estimated_j/`measured_j=None`); fabric soft-ref; joule ledger acts; VoI refuse; GUI feature not required |
| P20 | **OS meter honesty** | Feature off / VM / no permission → `measured_j=None`, empty components, `measure_source=unavailable`. Fixture (not live silicon): powermetrics plist stamps CPU/GPU/ANE/DRAM/package with `measure_source=powermetrics` (package = combined_power × elapsed, not a rail sum); RAPL fixture maps package/dram and **ignores psys**; permission text invents nothing. Default close stays unmetered. Live probe is optional (`energy-meter`). |
| P21 | **WASM capsule certify** | Real `add.wasm` fixture under `StubCapsuleRuntime` (optional `wasmtime` feature); stamp `EncapsulationReceipt` + `ComputeStepReceipt`; fuel → estimated_j only (`measured_j=None`); FS/net grant + fuel exceed + host-share → REFUSE; MoL close stamps capsule |
| P22 | **Agent Lane session path** | Logical cookie/profile/storage partitions per lane; `host_invoke` refuses without lane provenance; no cross-lane share; keyword confirm insufficient — need `GrantReceipt`; allow stamps `AgentLaneReceipt` (roots + grant_id); MoL close with strict session policy commits |
| P23 | **Multi-fabric compute receipt** | Soft-ref inventory (Cpu always; Gpu* soft-unavailable OK); `ComputeFabric` / `ScheduleDecision` / extended `ComputeStepReceipt` (`fabric_id`, `unavailable_reason`); cascade stamps `fabric:route`; CPU Formula COMMIT + Model residual `fabric_unavailable` refuse; never invent `measured_j`; Ferric not path-dep'd |
| P23b | **Live fabric soft** | `ecosystem_live_fabric_soft`: feature off stays software_ref. Mock/live Metal stamps `fabric_id=gpu_metal` (backend `metal`) via `schedule_fabric` / `ecosystem-certify --detect --tier model`. Dx12/Gl map when present. wgpu `DeviceType::Cpu` is not promoted. Formula stays Cpu. Coin-cell and missing Gpu* stay fail-closed. `measured_j` never invented. Prove does not require a GPU. |
| P23c | **wgpu / Metal tiny kernel** | Clean-room WGSL vector-add (`mol-core::kernel`); feature `fabric-detect`/`wgpu-kernel`. Soft stub offline (same checksum). Gpu* schedule stamps `kernel:vector_add` + `execution_proof` (checksum/sample); Cpu schedule skips. Mac live Metal sets `mode=live:metal`. Optional overlapping `energy-meter` (`measure_energy_during` + `--meter-required`) stamps kernel + receipt package `measured_j` with `energy_honesty=measured` (SMC `PSTR` on Mac; never invent / never rail-sum). Fixture path OK for prove. Ferric not vendored. |
| P24 | **Ecosystem e2e certify (single receipt)** | `EcosystemCertify::run`: open Agent Lane → schedule fabric (Cpu soft-ref) → invoke WASM `add.wasm` → stamp energy honesty (estimated fuel; optional meter_sample; never invent `measured_j`) → host invoke only with `GrantReceipt` → `commit|refuse` with encapsulation + agent_lane + compute_steps/fabric + energy_honesty on **one** receipt; refuse-without-grant flag → `grant_receipt_required` |

`prove` prints one line per criterion: `VERIFIED <name>` or `FAIL <name>: …`, then exits 0 only if all verified. Prior criteria stay green; P19–P20 extend the harness.

---

## 6. Roadmap residual — **OUT OF PROOF SCOPE**

Clearly marked; must **not** block `mol prove`:

- Live silicon / RAPL / NVML as **default** close path (optional feature `energy-meter` is in-tree; soft-ref prove stays offline with `measured_j=None`)
- Live wgpu adapter presence is **optional** (`mol-core`/`mol-cli` feature `fabric-detect`); soft-ref inventory remains the prove default — detection never invents `measured_j`
- Ferric / on-device EFA certificate (BMI hardware loop) — Ferric tree is reference only; see §1b
- klere-vm WASM / FPGA meter (real pJ/accumulate)
- Live `openie-path` ask bridge / leapfrog runtime
- Live WCA **in-proc** path-dep against `wca-lut-edge` (HTTP/MCP env certify **shipped**; FPGA Stage C meter still stub)
- μ **calib corpus** + HW Gaps (in-tree **live catalog** 258 Present / 258 Lookup/Formula/Solver/Navigate gears + Gap probes + **tier μ catalog** `mu_source=catalog` are **in proof**; Stage C soft-ref inventory in proof with `stage_c_measured=false`)
- Trained weights / candle / tract model leaf (Model stays demoted stub)

Adapters remain documented ports for these; the proven path never requires them.

---

## 7. Operator how-to (proof)

```bash
cd /workspace/mixture-of-limits
cargo test --workspace
cargo run -p mol-cli -- prove
cargo run -p mol-cli -- fabric                 # soft-ref inventory
cargo run -p mol-cli -- fabric --mock          # Metal/Vulkan/WebGPU mapping without GPU
cargo run -p mol-cli --features fabric-detect -- fabric --detect   # live wgpu when available
cargo test -p mol-core --features fabric-detect -- --ignored       # optional live detect test
cargo run -p mol-cli -- agent          # thin mailbox loop demo
cargo run -p mol-cli -- agent --goal "landauer joules per bit" --no-demo
cargo run -p mol-cli -- meter
cargo run -p mol-cli --features energy-meter -- meter --sample-ms 20
# Mac (component rails). IOReport needs no sudo when exposed; powermetrics needs root:
#   sudo powermetrics -f plist -i 1000 -n 1 --samplers cpu_power,gpu_power -a 0
#   sudo cargo run -p mol-cli --features energy-meter -- meter --sample-ms 1000
cargo test -p mol-desktop
cargo run -p mol-desktop --features gui --bin mol-desktop   # Mac/Linux with display
```

See also `README.md`, `docs/mol-desktop.md`, and `BLUEPRINT.md` §15.
