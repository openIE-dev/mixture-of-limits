## 6. Prove ↔ claim map

MoL’s constructive prove is not a leaderboard stunt. It is an **information-theoretic** close law rooted in **physics**: the same historical arc that took astronomy from excess epicycles to Kepler/Newton predictive floors, and that takes computation from Shannon → Landauer → complexity / VoI toward named energy floors. Global academia already holds those proof points; the bottleneck is **applied math × materials (hardware) embodiment**, not awareness of the slogans. This section only maps what `mol prove` actually stamps `VERIFIED` onto paper `claim.*` ids — embodiment of known floors, not invention of new physics.

A claim is **proven** only when the in-tree harness prints `VERIFIED` for the named criterion and exits 0. A claim is **soft-ref only** when PLAN §6 marks it OUT OF PROOF SCOPE, or when the criterion itself stamps soft-ref / fixture / feature-gated behavior rather than live silicon. No row invents `measured_j`, board joules, or `board_synth_claimed=true`.


Measurement labels used below (OpenIE honesty grammar):

| Label | Meaning |
|---|---|
| **Unmetered** | Software-ref / soft-ref path; `measured_j=None`; estimate may be labeled |
| **Estimated** | Catalog / Landauer / fuel / μ surrogate with explicit `EstimateKind` or `mu_source=catalog` |
| **Metered** | Optional live probe (`energy-meter`) actually returned numbers — **not** required for prove; never invented |

Default close path for all proven claims: **Unmetered** (+ **Estimated** where μ / Landauer / fuel appear). `board_synth_claimed=false` everywhere in the proven path.

### 6.1 Harness meta

| Prove # | Criterion (PLAN) | Paper claim id | Status | Measurement |
|---|---|---|---|---|
| P0 | `cargo test --workspace` green | `claim.mol.ci_workspace` | Proven (CI / operator; not printed by `mol prove`) | — |
| P1 | `cargo run -p mol-cli -- prove` exits 0 | `claim.mol.prove_harness` | Proven (prints `VERIFIED` per row below) | Unmetered |

### 6.2 Proven claims (P2–P24)

| Prove # | Criterion (PLAN) | Paper claim id | Claim (one sentence) | Soft-ref / scope note | Measurement |
|---|---|---|---|---|---|
| P2 | Deterministic close (double-run) | `claim.mol.deterministic_close` | Same asks yield the same commit/refuse + limit id. | — | Unmetered |
| P3 | Formula/lookup close without model | `claim.mol.formula_lookup_no_model` | Landauer / unit-convert closes COMMIT at Lookup or Formula; model never answered. | — | Unmetered / Estimated (Landauer floor on receipt) |
| P4 | VoI refuse when `!allow_model` | `claim.mol.voi_refuse` | Free-form ask → REFUSE `voi`. | — | Unmetered |
| P5 | Settle commit + settle refuse | `claim.mol.settle_ternary` | Ternary settle → COMMIT; `will not settle` → REFUSE `settle_refuse`. | Software-ref settle stub; no klere-vm / FPGA | Unmetered |
| P6 | Certificate refuse on diverge | `claim.mol.efa_certificate_refuse` | Formula + `diverge` → REFUSE `efa_certificate`. | In-tree EFA-style cert; Ferric / live BMI loop soft-ref | Unmetered |
| P7 | Capability default-deny mutate | `claim.mol.capability_deny_mutate` | `AutomateGate::default().gate(Mutate)` → Refuse. | — | Unmetered |
| P8 | Receipt honesty | `claim.mol.receipt_honesty` | Software-ref: `measured_j=None`, `board_synth_claimed=false`, estimate labeled (`EstimateKind`). | Live RAPL/NVML default close OUT OF SCOPE | Unmetered / Estimated |
| P9 | `ModelGenerated` ↛ `Deterministic` | `claim.mol.replay_no_strengthen` | `TypedAnswer::weaken_to` returns `ReplayCoercion`. | — | — |
| P10 | Periodic Stack live catalog navigation | `claim.mol.stack_live_catalog` | Family + present + scale → COMMIT at Lookup; 258 live Lookup/Formula/Solver gears; note cites 258/33 honesty. | Remaining μ calib + HW Gaps thesis primitives OUT OF SCOPE | Unmetered |
| P11 | `primitive_gap` via registry probe | `claim.mol.primitive_gap` | Gap marker `physical_settle` + absent name → REFUSE `primitive_gap` (not string-only). | — | Unmetered |
| P12 | μ / impedance catalog | `claim.mol.mu_catalog` | Receipts stamp `mu_source=catalog`, `mu`, `landauer_floor_ratio`; `E≈θ·μ` (not fake RAPL). | μ **calib corpus** OUT OF SCOPE | Estimated |
| P13 | Receipt transcript replay | `claim.mol.receipt_replay` | JSONL/in-memory replay reproduces commit/refuse + limit id; model never answered. | — | Unmetered |
| P14 | Z2 retrieve+cite | `claim.mol.z2_retrieve_cite` | Seeded factual hit → COMMIT `RetrievedCited` + citation ids; unknown → REFUSE `claim_unknown` (no invent). | Seed corpus in-tree only | Unmetered |
| P15 | Z1 compose/synthesis | `claim.mol.z1_compose` | ≥2 cited claims → COMMIT `Composed` + `synthesis.composed_from`; missing → REFUSE `compose_missing`; never laundered as Deterministic/RetrievedCited alone. | — | Unmetered |
| P16 | Agent mailbox loop | `claim.mol.agent_mailbox` | Goal/Message/Act → close → commit\|refuse + receipt in transcript; demo hits formula/cite/compose COMMIT and voi/settle_refuse REFUSE; model cold; honesty flags hold. | — | Unmetered |
| P17 | Bitemporal state + memory | `claim.mol.bitemporal_memory` | Writes only on MoL close COMMIT; Mutate deny / free-form remember → REFUSE; recall → `RetrievedCited` + `memory:` cite; unknown → `memory_unknown`. | — | Unmetered |
| P18 | Fabric routing + soft-ref inventory | `claim.mol.fabric_soft_ref` | Offline inventory valid (`source=software_ref`, Cpu always); Lookup/Formula→Cpu; Settle→ThermoSettle\|Cpu; Model→Gpu* or REFUSE `fabric_unavailable`; detect ≠ measured joules. | Live wgpu presence optional / not required for prove | Unmetered |
| P19 | Desktop shell headless | `claim.mol.desktop_headless` | `ShellSession` ask/close → receipt view (zone/fabric/limit/estimated_j/`measured_j=None`); joule ledger; VoI refuse; GUI not required. | egui GUI optional | Unmetered / Estimated |
| P20 | OS meter honesty | `claim.mol.os_meter_honesty` | Feature off / VM / no permission → `measured_j=None`, empty components, `measure_source=unavailable`; fixtures stamp labeled sources without inventing; default close stays unmetered. | Live probe optional (`energy-meter`); never invent | Unmetered (default) / Metered only when probe returns |
| P21 | WASM capsule certify | `claim.mol.wasm_capsule` | `add.wasm` under stub runtime; fuel → estimated_j only; FS/net grant / fuel exceed / host-share → REFUSE; close stamps capsule. | Optional `wasmtime` feature | Unmetered / Estimated (fuel) |
| P22 | Agent Lane session path | `claim.mol.agent_lane` | Lane partitions; `host_invoke` without provenance refuses; keyword confirm insufficient — need `GrantReceipt`; allow stamps `AgentLaneReceipt`. | — | Unmetered |
| P23 | Multi-fabric compute receipt | `claim.mol.fabric_compute_receipt` | Soft-ref inventory; `ComputeFabric` / `ScheduleDecision` / `ComputeStepReceipt` with `fabric_id`; cascade stamps `fabric:route`; CPU Formula COMMIT; Model residual may `fabric_unavailable`; never invent `measured_j`. | Ferric not path-dep'd | Unmetered |
| P23b | Live fabric soft | `claim.mol.fabric_live_soft` | Feature off stays software_ref; mock/live Metal can stamp `gpu_metal`; Formula stays Cpu; missing Gpu* fail-closed; prove does not require a GPU; `measured_j` never invented. | Soft / optional detect | Unmetered |
| P23c | wgpu / Metal tiny kernel | `claim.mol.kernel_vector_add` | Clean-room WGSL vector-add; soft stub offline (same checksum); Gpu* stamps `kernel:vector_add` + `execution_proof`; optional overlapping meter may stamp package `measured_j` with `energy_honesty=measured` — never invent / never rail-sum. | Live Metal optional; fixture OK for prove | Unmetered (default) / Metered only if meter returns |
| P24 | Ecosystem e2e certify (one receipt) | `claim.mol.ecosystem_e2e` | Lane → fabric (Cpu soft-ref) → WASM → energy honesty (estimated fuel; optional meter_sample) → host invoke only with `GrantReceipt` → commit\|refuse on **one** receipt; refuse-without-grant → `grant_receipt_required`. | — | Unmetered / Estimated |

### 6.3 Soft-ref only (not proven; do not escalate to paper results)

These are PLAN §6 residuals. Citing them is allowed as roadmap / reference semantics. Treating them as measured results is not.

| Soft-ref item | Why soft-ref | Related prove row (if any) |
|---|---|---|
| Live silicon RAPL / NVML as **default** close | Optional `energy-meter`; soft-ref prove stays `measured_j=None` | P8, P20 |
| Live wgpu adapter presence | Feature `fabric-detect`; inventory soft-ref remains prove default | P18, P23b, P23c |
| Ferric / on-device EFA (BMI hardware loop) | Reference semantics only; not path-dep'd | P6, P18, P23* |
| klere-vm WASM / FPGA meter (real pJ) | Software-ref `StubKlereSettle` | P5 |
| Live `openie-path` / leapfrog ask bridge | Adapter port | — |
| Live WCA/`wca-lut-edge` **in-proc** + FPGA Stage C meter | Adapter residual (HTTP\|MCP NI certify **shipped**, env-gated) | Ferric/FPGA stub |
| μ calib corpus + HW Gaps | Live catalog (258 Present / 258 live gears) + Gap + tier μ catalog + Stage C soft-ref inventory are in proof | P10, P12 |
| Trained weights / candle / tract Model leaf | Model stays demoted stub | P3, P4, P16 |

### 6.4 Seeded knowledge claims (Z2 corpus; not the prove map)

In-tree `ClaimStore` seeds used by P14/P15 (retrieve/compose), distinct from the paper claim ids above:

- `claim:landauer.principle`
- `claim:mol.law`
- `claim:mol.proof_law`
- `claim:mol.cascade`
- `claim:stack.scale`
- `claim:honesty.no_fake_rapl`
- `claim:replay.retrieved_cited`
- `claim:openie.z2`

### 6.5 What this map refuses

- No invented `measured_j` values.
- No `board_synth_claimed=true`.
- No board joules / FPGA watt leadership from MoL v0.1 prove.
- No promotion of soft-ref roadmap items into proven claims.
