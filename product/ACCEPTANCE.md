# ACCEPTANCE.md — Mixture of Limits product

Physics-informed acceptance. Soft-ref proves constructive existence; these tests bind the **product** surface (`mol.yaml` + `mol run` / `mol prove`).

**Invariants.** Estimates ≠ `measured_j`. `board_synth_claimed=false` on soft-ref. `ModelGenerated` never commits without an NI certificate. Refuse is success when VoI=0 or `C(z)=1`.

## A1 — O(1) grammar hit → no model

| Field | Value |
|---|---|
| Given | Ticket AST matches Lookup grammar / LUT row (support-desk resolution code path) |
| When | `mol run --chore mol.yaml` (or clean-room close on equivalent typed ask) |
| Then | Cascade closes at **Lookup** (or Formula if LUT miss but schema covered) |
| And | Model leaf **not** invoked; receipt `replay_class` ∈ {Deterministic, RetrievedCited, Composed} |
| And | Receipt stamps `estimated_j` (labeled); `measured_j=None` unless meter present |

**Prove bridge:** clean-room `mol prove` already requires formula/lookup without model (PLAN P3-class).

## A2 — VoI=0 → refuse

| Field | Value |
|---|---|
| Given | Free-form or residual ask with `allow_model=false` OR VoI table says ΔB ≤ 0 past floor |
| When | Close / `mol run` |
| Then | Outcome **REFUSE**; limit id `voi` |
| And | Zero executor / model calls for the irreversible act |
| And | Receipt carries refuse reason; `estimated_j` present |

**Prove bridge:** VoI refuse on free-form when `!allow_model`.

## A3 — C=1 → satiation stop

| Field | Value |
|---|---|
| Given | Completeness clauses in `mol.yaml` all true (`C(z)=1`) |
| When | Further synthesis requested on the same episode |
| Then | Outcome **REFUSE**; limit id `satiation` (or `completeness`) |
| And | No additional model spend advances `C` (definitional stop) |
| And | Economic refuse distinct from physical/certificate refuse on the receipt |

## A4 — ModelGenerated never commits without NI cert

| Field | Value |
|---|---|
| Given | Cascade reaches `model_fallback` with `replay_class=ModelGenerated` |
| When | NI / WCA / EFA certificate missing or fail |
| Then | **REFUSE** (`ni_certificate` / `efa_certificate`); executor not called |
| And | Type law: `ModelGenerated` cannot strengthen to `Deterministic` |
| And | Commit only after certify pass stamps certificate id on receipt |

**Prove bridge:** ReplayClass coercion refuse; EFA diverge refuse; AutomateGate certify-before-commit.

## A5 — Estimate joules on every receipt

| Field | Value |
|---|---|
| Given | Any close (commit or refuse) |
| When | Receipt emitted |
| Then | `estimated_j` set; `estimate_kind` / Tier-0 catalog label present |
| And | Landauer / μ fields are **estimates** (`mu_source=catalog` when catalog) |
| And | No copy of `estimated_j` into `measured_j` |

## A6 — `measured_j` only when meter present

| Field | Value |
|---|---|
| Given | Soft-ref / meter feature off / probe unavailable |
| When | Close |
| Then | `measured_j=None` (or unset); `measure_source=unavailable` |
| Given | Meter feature on **and** probe returns package reading |
| When | Close with meter |
| Then | `measured_j` set only from that Metered reading; never invent; never rail-sum into package |

**Prove bridge:** OS meter honesty (PLAN P20); soft-ref prove keeps `measured_j=None`.

## A7 — Live in-crate NI/WCA/EFA certify

| Field | Value |
|---|---|
| Given | Close path uses `InCrateNiCertify` (not soft-ref-only without ids) |
| When | Proposal certified allow / diverge refuse |
| Then | Receipt stamps `certificate_ids` (`ni:`/`efa:`/`wca:`); commit\|refuse |
| And | FPGA Stage C stays `stage_c_measured=false`; `board_synth_claimed=false` |

## A8 — Residual Model LAST leaf

| Field | Value |
|---|---|
| Given | VoI>0 + `allow_model` + budget + fabric |
| When | Residual adapter proposes |
| Then | ReplayClass=`ModelGenerated`; uncertified never commits |
| And | Certified commit keeps ModelGenerated + certificate ids |

## A9 — Durable episode C(z)

| Field | Value |
|---|---|
| Given | `EpisodeStore` with CompletenessSnapshot |
| When | Closes across process / reload |
| Then | C(z)=1 satiates; further synthesis REFUSE `satiation` |

## A10 — `mol bench` J/query

| Field | Value |
|---|---|
| Given | MoL vs always-model vs MoE-sim |
| When | `mol bench` |
| Then | Energy labeled **Estimated** or **Metered** only; never invent `measured_j` |

## A11 — Phase-1 micro-perception

| Field | Value |
|---|---|
| Given | `phase1.enabled=true` |
| When | Unstructured ticket-ish input |
| Then | Rule AST transducer emits typed query; unrecognized refuses parser-as-model |

## A12 — Primitive Distillation Loop v1 (hardened)

| Field | Value |
|---|---|
| Given | Certified Model LAST commit |
| When | `mol distill` / `distill_certified_model_last` then cascade with `with_distill_store` |
| Then | Lookup/Formula append with Deterministic replay; **second pass** matching the distilled pattern closes at **Lookup** (or Formula) **without** opening Model LAST; uncertified refuses |

## A13 — Tier-1 NVML + Tier-2 shunt HAL

| Field | Value |
|---|---|
| Given | Optional meters |
| When | Probe absent / stub |
| Then | `measured_j=None`; StubShuntHal never invents; NVML without sample API stays unavailable |


## A14 — `mol arena` head-on

| Field | Value |
|---|---|
| Given | Arena chores: typed decision / ticket-close / risk (+ Phase-1 unstructured) |
| When | `mol arena` (or `mol bench --arena`) |
| Then | MoL cascade vs frontier_sim vs system_one_leaf (+ optional real_leaf via `--endpoint`); Lookup + **Formula** risk + **Solver** route/knapsack when LUT misses; **Phase-1** raw-ish → typed AST → cascade (baselines kept); metrics correct_close, refuse_when_C=1, estimated_j (Estimated), latency; never invent `measured_j`; estimates≠measured_j |

## Acceptance matrix

| ID | Must pass |
|---|---|
| A1 | Grammar/LUT path model-cold |
| A2 | VoI=0 refuse |
| A3 | C=1 satiation refuse |
| A4 | NI cert gate on ModelGenerated |
| A5 | estimated_j always |
| A6 | measured_j honesty |
| A7 | Live in-crate NI certify + certificate ids |
| A8 | Residual Model LAST + cert gate |
| A9 | Durable EpisodeStore C(z) |
| A10 | `mol bench` Estimated\|Metered only |
| A11 | Phase-1 rule AST workable |
| A12 | Distill v1 certified→Lookup; second pass Lookup without model |
| A13 | Tier-1 NVML + Tier-2 shunt honesty |
| A14 | `mol arena` head-on Estimated only; no invent |

## How to run (today)

```bash
cd /Users/dcharlot/Desktop/mol-sync/mixture-of-limits
cargo test --workspace
cargo run -p mol-cli -- prove          # PLAN.md + product A1–A14
cargo run -p mol-cli -- run --chore product/mol.yaml
cargo run -p mol-cli -- bench
cargo run -p mol-cli -- arena
# publish table: product/ARENA_RESULTS.md (from actual run)
cargo run -p mol-cli -- phase1 "please close ticket as R-OK"
cargo run -p mol-cli -- distill "ticket summary" --store product/fixtures/distill_store.json
```

`mol prove` embeds product acceptance A1–A14. Competitive positioning: [COMPETITIVE.md](./COMPETITIVE.md).
