# COMPETITIVE.md — Mixture of Limits vs System One / Jev Arena class / default LLM agents

**Voice:** Mixture of Limits in full. Estimates ≠ `measured_j`. Refuse is success when VoI=0 or `C(z)=1`. Win on **J/query**, **refuse correctness**, and **certify-before-commit** — not on chat Elo, and not on typed decision-accuracy leaderboards alone.

## Scoreboard (what we win on)

| Axis | Mixture of Limits product | System One / Jev Arena class | Default LLM agents |
|---|---|---|---|
| Problem class | Any chore with written `C(z)` — navigation + commit + satiation | Typed decisions / classification over **known option sets** (state + rubric → distribution) | Open-ended generation; usually no written done |
| Cascade | Lookup → Formula → Solver → **Model LAST** | Fast typed System One proposals (Model LAST / residual proposer) | Always-call-model (or tool-loop that still burns tokens) |
| O(1) grammar hit | Bloom / HashMap LUT — **model never invoked** (A1) | N/A (option set already typed; model or encoder still runs) | Rarely; model still in the loop |
| VoI=0 | **REFUSE** with limit `voi` (A2) | Outside product scope | Keeps generating |
| Completeness `C(z)=1` | **REFUSE** satiation — economic reason code (A3) | Does not own economic done | No satiation stop |
| Irreversible commit | NI propose → certify → commit\|refuse → receipt (A4) | Does not own plant motion | Often commits / acts without certificate |
| `ModelGenerated` | Never launders to Deterministic; cert required to commit | Proposals stay typed System One | Model text treated as action |
| Joules | `estimated_j` always; `measured_j` only if meter (A5–A6) | Latency / $/decision / memory as deployment metrics — not MoL Landauer receipts | Token/cost proxies; package joules often invented or omitted |
| Win metric | **J/query**, refuse correctness, certify rate | Decision / classification accuracy, calibration, speed, cost on structured tasks | Chat Elo / vibe benchmarks |

## What Jev Arena (and kin) measure

These surfaces evaluate **decision / classification accuracy on structured tasks** — not Mixture of Limits navigation law.

| Surface | Role | What it measures |
|---|---|---|
| [Jev Arena](https://github.com/theaiautomators/jev-arena) | Local evaluation lab (independent; unaffiliated with TypeSafe) | Accuracy, speed, memory, inspectable cases, workflow replays, shareable reports across pinned decision-model profiles |
| [JevBench](https://github.com/fstandhartinger/jevbench) → [Benchmark Heaven](https://benchmarkheaven.com/jev-models) | Independent Jev-class benchmark | Typed answer over a bounded rubric + label set; composite of intelligence / calibration / speed / cost (versions evolve; do not invent MoL scores from their board) |
| [DecisionBench leaderboard](https://huggingface.co/spaces/Hanno-Labs/decision-bench-leaderboard) | Document-grounded decision suite | Task / family / domain / primitive breakdowns for document-grounded decisions |
| [TypeSafe docs — Models](https://docs.typesafe.ai/models) | Hosted System One API | `POST /v1/systemone`; Jev returns typed decisions with calibrated probabilities over supplied options |

**Builder reading (Arena’s own framing):** use a leaderboard to shortlist, then test the deployment you intend to build. Aggregate typed accuracy is not proof of chore close, plant safety, or economic done. Mixture of Limits does not re-score Arena / JevBench / DecisionBench runs and does not claim to win their axes.

## System One / Model LAST proposers (same class)

TypeSafe’s product language: **System One Models** are built for decisions inside software — typed outputs, calibrated confidence, machine-native rather than chat ([typesafe.ai](https://typesafe.ai/), [docs.typesafe.ai/models](https://docs.typesafe.ai/models)). **Jev** is TypeSafe’s flagship hosted System One model.

Open and community rows that sit in the **same proposer class** (typed choice / score / noul over known options — Model LAST leaf for Mixture of Limits, not a rival cascade):

| Model / project | Role in the class |
|---|---|
| **Jev** (TypeSafe) | Hosted System One reference; typed decisions + confidence for automation thresholds |
| **Laya** (`convaiinnovations/laya`) | Encoder System One (ModernBERT-class heads); local typed proposers |
| **Decider** (`Mapika/decider-4b` and siblings) | Fine-tuned Qwen3.5 decision models; often `/v1/systemone`-compatible |
| **SemIf** (`TheoLeeCJ/SemIf-OpenJev`) | Training-free option scoring from open LLMs (formerly OpenJev) |
| **Bespoke-Nimble-9B** (`bespokelabs/Bespoke-Nimble-9B`) | Fine-tuned typed-decision model + training recipe |
| **Winnow-12B** (`EldanRing/Winnow-12B`) | Local decision profile measured in Arena-style labs |
| **Plumb-4B** (`crh225/plumb-4b`) | JevK-class LoRA / decision fine-tune on small Qwen3.5 bases |
| **CLM-v0.1-8B** (`Contrastive-LM/CLM-v0.1-8B`) | Contrastive / decision-oriented open weights in the same measurement ecosystem |
| **Qwen3.5-4B** (`Qwen/Qwen3.5-4B`) | Common open base / control backbone under SemIf, Plumb, Decider-class stacks |
| **ModernBERT-large-zeroshot-v2.0** (`MoritzLaurer/ModernBERT-large-zeroshot-v2.0`) | Compact NLI / zeroshot classifier — System One–adjacent leaf for narrow label sets |

Directory references for the class (not MoL endorsements): [laya-ai.com/system-one-models](https://laya-ai.com/system-one-models), Arena model guides, JevBench adapters.

**Kev** and other TypeSafe-compatible local servers belong in the same proposer bucket when they speak typed distributions over fixed option menus.

## Positioning (one paragraph)

**System One / Jev / Laya / Decider / Nimble / SemIf / Winnow / Plumb** (and kin) collapse generation cost when the option set is known: state + rubric → typed distribution software can act on. Jev Arena, JevBench, and DecisionBench measure how well those proposers (and classifiers) pick labels under deployment constraints. **Mixture of Limits does not replace them for closed menus.** It owns the complementary global problem: **energy-bounded navigation**, **certify-before-commit**, and **satiation stop** after floors — including open chores until grammar/VoI say otherwise. When floors do not close, those models remain the lawful **residual Model LAST** leaf: they propose; NI certify decides whether the machine may move; receipts keep `estimated_j` honest. Default LLM agents lose on J/query and refuse correctness because completeness is never written and ModelGenerated proposals commit without an NI certificate.

## Where Mixture of Limits wins vs where those models still matter

| Mixture of Limits wins | System One / Arena class still matters |
|---|---|
| Lookup → Formula → Solver **first** (model cold by default) | Closed option menus where typed accuracy + calibration are the product |
| VoI=0 / `C(z)=1` **refuse** as success | Shortlisting which residual leaf to plug in when Model LAST opens |
| NI **certify-before-commit** + receipt | Latency / memory / choice-count fit for a local or hosted proposer |
| **`estimated_j` / J·query** honesty (Estimated\|Metered only) | Workflow replays that stress software-usable outputs (Arena) |
| Open chores until grammar covers them | Document-grounded or policy label tasks measured by DecisionBench / Arena subsets |

## What we do not claim

- Not higher chat Elo than frontier LLMs.
- Not higher typed decision accuracy than Jev / Laya / Decider / Arena leaders — we do not invent Arena or JevBench scores.
- Not shipped board / FPGA package energy (`board_synth_claimed=false` on soft-ref).
- Not inventing `measured_j` from Landauer or catalog μ.
- Not replacing System One for typed known option sets.
- Not geographic superiority of any lab’s network path or GPU; measure on the deployment you ship.

## Operator proof

```bash
cd /Users/dcharlot/Desktop/mol-sync/mixture-of-limits
cargo run -p mol-cli -- prove          # PLAN + product A1–A13
cargo run -p mol-cli -- run --chore product/mol.yaml
cargo run -p mol-cli -- run --chore product/chores/financial_risk_scoring.yaml
cargo run -p mol-cli -- bench          # J/query vs always-model / MoE-sim (Estimated|Metered)
```

Beside Mixture of Limits benches: run Jev Arena for residual-leaf shortlists — see [ARENA.md](./ARENA.md). Do not mix Arena accuracy into `mol bench` J/query rows.

Research triad: [Mixture of Limits](https://research.openie.dev/papers/mol/) (navigation) · [Notational Intelligence](https://research.openie.dev/papers/ni/) (commit; System One = typed proposers) · [Satiation](https://research.openie.dev/papers/satiation/) (economic done).
