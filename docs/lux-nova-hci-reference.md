# Secure Energy+Compute Traceable WASM Ecosystem

**Audience:** Mixture of Limits (MoL) — law that subordinates harnesses.  
**Corrected thesis (David Charlot):** Lux Studio + Nova Browser are **not** an HCI chrome reference. They are a **complete energy-traceable and compute-traceable ecosystem** — a **scalable WASM / fully encapsulated secure ecosystem** so nothing like OpenAI / frontier-vendor **opaque bots** can happen.

**Rule:** MoL is **law** (floors, formula/settle/cite, commit|refuse, joule receipts). MoL does **not** clone Photon shells, Nova everything-app modules, or Lux `studio.html`. Borrow encapsulation, metering honesty, Agent Lane isolation, and fail-closed grammar; keep certify/close ownership in MoL.

**Sources (read-only — do not modify `lux-design` or `web-browser`):**
- Lux: `/Users/dcharlot/vibe-coding/lux-design`
- Nova: `/Users/dcharlot/vibe-coding/web-browser`
- WASM / energy substrate sibling: `/Users/dcharlot/vibe-coding/joulesperbit-lux-wasm-grind5`
- MoL desktop: [`mol-desktop.md`](./mol-desktop.md)
- Annex inventories (crate lists, not the lead): [`_source_lux_crate_inventory.md`](./_source_lux_crate_inventory.md), [`_source_nova_crate_inventory.md`](./_source_nova_crate_inventory.md), [`_source_nova_shell_inventory.md`](./_source_nova_shell_inventory.md)

**Generated:** 2026-09-30 (America/New_York). Verified against source trees; unknown roles left unmarked rather than guessed.

---

## Table of contents

1. [Architecture law](#1-architecture-law)
2. [Verified ecosystem extraction](#2-verified-ecosystem-extraction)
3. [Anti-patterns (explicit)](#3-anti-patterns-explicit)
4. [MoL fold — floors, receipts, fail-closed](#4-mol-fold--floors-receipts-fail-closed)
5. [Relationship map](#5-relationship-map)
6. [Borrow matrix (law, not chrome)](#6-borrow-matrix-law-not-chrome)
7. [Executive digest](#7-executive-digest)
8. [Annex pointers](#8-annex-pointers)

---

# 1. Architecture law

## 1.1 What this ecosystem is

| Claim | Meaning |
|---|---|
| **Energy-traceable** | Every consequential unit of work can answer “how many joules?” with an honesty label: **estimated** (universal floor, including WASM) vs **measured** (RAPL / IOReport / powermetrics / NVML when real) vs **modeled** (CPU-proxy — never labeled measured). |
| **Compute-traceable** | Cascade / fabric / capsule steps emit receipts: fabric chosen, estimated_j, optional measured_j, capsule id. Detect ≠ joules. |
| **Fully encapsulated** | Compute ships as sealed WASM Components / modules (or Agent Lane / process sandbox). Host session (cookies, profile, storage) is **not** shared with opaque multi-tool bots. |
| **Secure by fail-closed** | Missing meter when required, missing sealed capsule when required, broken Agent Lane isolation, missing citations when required → **refuse**, never invent evidence. |
| **Scalable WASM continuum** | Same artifact / interface across device → edge → cloud hosts (Lux WIT `lux:runtime/runtime@0.1.0`, WASI Preview 2). |

## 1.2 MoL’s place

```
┌─────────────────────────────────────────────────────────────────┐
│  Lux / Nova / joulesperbit  =  encapsulated energy+compute      │
│  harnesses & surfaces (WASM runtime, browser, ledgers, lanes)   │
└───────────────────────────────┬─────────────────────────────────┘
                                │ subordinated by
                                v
┌─────────────────────────────────────────────────────────────────┐
│  MoL = LAW                                                      │
│  floors · formula/settle/cite · commit|refuse · joule receipts  │
│  mathematical compression vs MoE scaling                        │
│  interface tax: machine/protocol fidelity first;                │
│  thin human confirm only on consequential acts                  │
└─────────────────────────────────────────────────────────────────┘
```

**Proof law (unchanged):** `E(x) ≥ θ(D)·μ(S,V)`.  
**Cascade:** Lookup → Closed-form → Sparse solver → Stochastic model **LAST**.  
**Close:** propose → certify → **commit|refuse** → receipt. Model is never the substrate.

## 1.3 Interface tax

Machine / protocol fidelity is primary (`ShellSession::ask_close`, MCP-class lanes, WebMCP-style tool preference over pixel actuation). Human UI is a thin confirm surface on irreversible / mutate / meter-required acts — not a chat-first product.

---

# 2. Verified ecosystem extraction

Facts below are checked in source. Do not invent meters, ports, or crate roles.

## 2.1 Lux — energy-metered WASM publishing + coding OS

| Pillar | Verified fact | Path |
|---|---|---|
| Product thesis | Content → cinematic experiences; each compile targets **energy-metered Lux WASM** + **energy receipt**; host on **Invisible Infrastructure**; pricing **per-joule** | `lux-studio-product-concept.md` |
| WASM engine | `lux-engine` WASM entry (`LuxApp`); `wasm-pack` build path in README | `crates/engine`, `README.md` |
| WASM Component runtime | `lux-wasm-runtime`: native rlib **or** WASI Preview 2 Component exporting `lux:runtime/runtime@0.1.0` | `crates/wasm_runtime`, `docs/AI_NATIVE_RUNTIME.wit` |
| Energy ledger | `lux-energy`: estimation = universal floor (incl. browser/wasm); measurement = RAPL / NVML / macmon(IOReport); `EnergyMethod::{Estimated,Measured}`; process-global `EnergyLedger`; cloud tiers carry own energy (never substitute local idle) | `crates/energy`, `docs/ENERGY_OBSERVABILITY.md` |
| Cost vs joules | `lux-cost` converts joules → money/CO₂; energy stays physical | `crates/energy` docs |
| Local coding brain | Desktop GGUF (`lux-llm-gguf`) vs browser WGSL (`lux-ai`); physics split, not preference | `docs/LOCAL_CODING_BRAIN.md` |
| Permissions / path jail | `Permissions::{plan,build}`; plan omits write/bash; path resolve refuses `..` / absolute escape | `crates/code_tools/src/repo.rs`, `docs/AGGREGATION.md` |
| Route → verify → escalate | `lux-code-route`: cheapest tier first; escalate only on real verifier fail | `crates/code_route` |
| Desktop link to Nova | Lux desktop/host **path-depend** on Nova `photon` + `nova_sdk` | `crates/desktop/Cargo.toml` |

## 2.2 Nova — privacy browser + Agent Lane + navigator.energy

| Pillar | Verified fact | Path |
|---|---|---|
| Energy API | `navigator.energy`: tiers `T0-rapl` / `T1-ioreport` / `T2-emi` / **`T4-modeled`**; honesty that macOS CPU-time backend reports **modeled**, not measured | `docs/examples/02-navigator-energy.md`, `crates/nova_compute/src/energy_sampler.rs` |
| Compute crate | `nova_compute`: energy sampler + scheduler + ledger design | `crates/nova_compute` |
| Agent Lane | `AppLane::{Human,Agent}`; separate profile (`~/.nova/profiles/agent`); **isolated cookie jar**; human-only `nova://` denylist | `docs/agent-lane.md` |
| Agent Lane gaps (honest) | localStorage/IndexedDB **not** yet partitioned; ACP bind / fail-closed “browser unavailable” **not** productized | `docs/agent-lane.md` |
| WebMCP | `document.modelContext` register/list/execute; host invoke **fail-closed** without JS bridge | `docs/webmcp.md` |
| Web Bot Auth | RFC 9421 Ed25519 on Agent Lane egress; fail-closed when enabled + missing keys; Human Lane never signs | `docs/web-bot-auth.md` |
| Ports | DevTools/CDP `:9222`, ACP `:9333`, NWP `:9077` | `README.md` |
| Lux lang port | `nova_lux_lang` = Lux language front-end port; **WASM codegen not in this crate** | crate docs / README |
| STALE | WASM bypass `dist/` dated 2026-04-13 **cannot rebuild** (`nova_web` never existed) | README feature table |

## 2.3 joulesperbit-lux-wasm-grind5 — WASM substrate sibling

| Pillar | Verified fact |
|---|---|
| Role | Independent OpenIE vertical stack (JouleDB / flowg / mathground); **every layer measures joules**; WASM deployability (JouleDB browser WASM, flowg wasm backends, bit-identical native↔wasm32 golden hashes in twin program) |
| Cargo | **No** path-dep on `lux-design` or `web-browser` at census — conceptual / energy sibling, not HCI shell |
| MoL borrow | Energy receipts + WASM encapsulation + conservation/transcript culture — not a Photon clone |

## 2.4 Invisible Infra / joules pricing honesty (Lux)

- Hosting priced **per joule** of CDN + compute per view; AI generation priced in inference joules (`lux-studio-product-concept.md`).
- Estimation runs everywhere; measurement only when counters exist — **never invent RAPL/IOReport**.
- MoL mirrors this: `estimated_j` always; `measured_j: Option` only on real probe.

---

# 3. Anti-patterns (explicit)

These are **forbidden** under MoL ecosystem law (and called out by Lux/Nova honesty docs):

1. **Opaque multi-tool agent loops** sharing human cookies/profile/storage (frontier-vendor bot pattern).
2. **Unmetered inference** presented as measured joules.
3. **Inventing `measured_j`** when RAPL/IOReport/powermetrics/NVML unavailable.
4. **Labeling modeled / CPU-proxy as Measured** (Nova T4-modeled lesson).
5. **Shared storage with agents** without partition (Nova Agent Lane gap — do not claim done).
6. **Keyword-only safety as sufficient** (MoL requires typed floors + certify + commit|refuse).
7. **Chat-first chrome as the product** (interface tax inverted).
8. **Cloning Photon / Nova shell / Lux studio as MoL core** — MoL stays law.

---

# 4. MoL fold — floors, receipts, fail-closed

Clean-room implementation in `/workspace/mixture-of-limits` (does **not** modify Lux/Nova).

## 4.1 New / extended floors (`FloorKind`)

| Floor | When it binds |
|---|---|
| `Encapsulation` | Unsealed capsule when required; non-encapsulated boundary when required; **any** `shares_host_session` |
| `EnergyHonesty` | `require_measured` but no real meter; dishonest measured_j/source pair |
| `ProvenanceMissing` | Required citations / capsule / meter provenance absent |
| `AgentIsolation` | Opaque shared-bot policy; Agent Lane without separate profile/cookies; policy missing when required |

## 4.2 Core APIs (`mol-core`)

- `CapsuleContext` / `CapsuleBoundary` / `EncapsulationReceipt`
- `EnergyHonestyClass::{Estimated,Measured,Modeled,Unavailable}` + `energy_pair_honest` + `require_measured_or_refuse` (Measured only — Modeled/CpuProxy refuses)
- `FailClosedPolicy` on `MolRequest` (`require_measured|capsule|citations|agent_isolation`); helpers `open` / `encapsulated_agent` / `meter_required`
- `MolRequest::with_meter_sample(MeterSample)` — attach real OS meter / honest fixture (never invent)
- `AgentIsolationPolicy` / `AppLane` / `AgentLaneReceipt` (Nova Agent Lane–class)
- `ComputeStepReceipt` (fabric + estimated_j + optional measured_j + honesty + capsule_id)

## 4.3 Receipt fields (`mol-receipt`)

- `energy_honesty`, `encapsulation`, `agent_lane`, `compute_steps`
- Soft-ref never sets `measured_j`; `refresh_energy_honesty()` / `apply_meter_sample` keep pair law
- Desktop `ReceiptView` + joule ledger surface the same stamps (measured vs estimated labels; missing/refuse visible)

## 4.4 Router gate

`MixtureOfLimits::ecosystem_gate` runs **before** named floors / cascade. Successful close stamps encapsulation + agent_lane + estimated compute steps + optional meter sample.

**`meter_required`:** COMMIT only when attached sample supplies real package `measured_j` with **Measured** honesty; probe fail / missing / Modeled → REFUSE. Never invent `measured_j`.

## 4.5 Prove

`mol prove` criteria: `ecosystem_encapsulation`, `ecosystem_agent_isolation`, `ecosystem_fail_closed_meter` (refuse without sample **and** COMMIT on RAPL fixture), `ecosystem_energy_honesty_types` (offline stubs OK; **never invent measured_j**).

---

# 5. Relationship map

```
lux-design ──path──► web-browser/{photon,nova_sdk}     (desktop UI framework)
web-browser ──port──► lux-lang front-end only          (nova_lux_lang; no lux-design path-dep)
joulesperbit-lux-wasm-grind5 ──theme──► energy+WASM    (no Cargo edge to Lux/Nova)
MoL ──law──► subordinates all three                    (no path-deps; clean-room floors)
```

Shared patterns (not shared packages): energy ledger honesty, HyperDB embeds, MCP/ACP (different expansions — don’t conflate Nova `:9333` with lux-code-acp/Zed), Agent Lane vs Permissions plan|build.

---

# 6. Borrow matrix (law, not chrome)

| Axis | Borrow | MoL law |
|---|---|---|
| Encapsulation | Lux WASM Component + path jail | `CapsuleContext` + `FloorKind::Encapsulation` |
| Energy honesty | lux-energy Estimated/Measured; Nova modeled tier | `EnergyHonestyClass`; never invent measured_j |
| Compute receipts | EnergyLedger / navigator.energy / StepReceipts | `ComputeStepReceipt` + cascade steps + fabric |
| Fail-closed | WebMCP host invoke; Web Bot Auth missing keys | `FailClosedPolicy` → ProvenanceMissing / EnergyHonesty |
| Agent isolation | Nova Agent Lane cookies/profile | `AgentIsolationPolicy` → AgentIsolation floor |
| Verify/escalate | lux-code-route | Cascade + certify; model last |
| Close grammar | — | **MoL owns commit\|refuse + ReplayClass** |
| HCI shell | — | **egui energy harness only**; not Photon/Nova clone |

---

# 7. Executive digest

1. **Lux+Nova+joulesperbit are one secure energy+compute WASM ecosystem**, not an HCI skin for MoL to copy.
2. **Encapsulation** (WASM Component / module / Agent Lane / process) is load-bearing so opaque frontier bots cannot share human session state.
3. **Estimation is the universal joule floor**; measurement is optional refinement with honesty labels (incl. modeled ≠ measured).
4. **Invisible Infra / per-joule pricing** and compile-time energy receipts are Lux product law — MoL encodes the certify side.
5. **Agent Lane** isolates cookies/profile; storage partition and ACP fail-closed remain honest gaps in Nova — MoL still refuses opaque share.
6. **WebMCP / Web Bot Auth** teach fail-closed host invoke and signed agent egress — MoL folds as provenance/meter gates.
7. **MoL = law**: floors, formula/settle/cite, commit|refuse, joule+compute receipts; mathematical compression over MoE sprawl.
8. **Interface tax**: machine protocol first; thin human confirm on consequential acts only.
9. **Anti-patterns**: opaque agent loops, unmetered-as-measured, shared agent storage, keyword-only safety, chat-as-product.
10. **Folded into MoL runtime**: Encapsulation / EnergyHonesty / ProvenanceMissing / AgentIsolation floors + receipt fields + `mol prove` green.
11. **Desktop shell** remains energy harness (`mol-desktop`) pointing at this ecosystem framing — not a studio/browser clone.
12. **Agent Lane session path (done in MoL)**: logical cookie/profile/storage partitions + `host_invoke` provenance + `GrantReceipt` (keyword insufficient) — see `docs/agent-lane-session.md`. Next residual: live meter into fail-closed when OS counters exist; keep Lux/Nova unmodified.

---

# 8. Annex pointers

Full crate / shell inventories (lead with architecture law above; lists are annex):

- [`_source_lux_crate_inventory.md`](./_source_lux_crate_inventory.md) — Lux ~197 workspace crates
- [`_source_nova_crate_inventory.md`](./_source_nova_crate_inventory.md) — Nova workspace crates
- [`_source_nova_shell_inventory.md`](./_source_nova_shell_inventory.md) — Nova shell modules

Operator shell: [`mol-desktop.md`](./mol-desktop.md)

---

*End. All claims trace to cited trees/docs as of 2026-09-30 ET; MoL code is clean-room and does not modify lux-design or web-browser.*
