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

## Acceptance matrix (MVP)

| ID | Must pass before product “MVP done” |
|---|---|
| A1 | Grammar/LUT path model-cold |
| A2 | VoI=0 refuse |
| A3 | C=1 satiation refuse |
| A4 | NI cert gate on ModelGenerated |
| A5 | estimated_j always |
| A6 | measured_j honesty |

## How to run (today)

```bash
cd /Users/dcharlot/Desktop/mol-sync/mixture-of-limits
cargo test --workspace
cargo run -p mol-cli -- prove          # PLAN.md + product A1–A6 VERIFIED
cargo run -p mol-cli -- run --chore product/mol.yaml
cargo run -p mol-cli -- run --chore product/chores/financial_risk_scoring.yaml
# Golden fixtures: product/fixtures/a1.yaml … a6.yaml (scenario field)
```

`mol prove` embeds product acceptance A1–A6. Competitive positioning: [COMPETITIVE.md](./COMPETITIVE.md).
