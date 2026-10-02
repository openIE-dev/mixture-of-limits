# COMPETITIVE.md — Mixture of Limits vs Laya / Jev·Kev class / default LLM agents

**Voice:** Mixture of Limits in full. Estimates ≠ `measured_j`. Refuse is success when VoI=0 or `C(z)=1`. Win on **J/query**, **refuse correctness**, and **certify-before-commit** — not on chat Elo.

## Scoreboard (what we win on)

| Axis | Mixture of Limits product | Laya System One | Jev / Kev class | Default LLM agents |
|---|---|---|---|---|
| Problem class | Any chore with written `C(z)` — navigation + commit + satiation | Typed decisions over **known option sets** | Typed / System One proposers (same class as Laya) | Open-ended generation; usually no written done |
| Cascade | Lookup → Formula → Solver → Model LAST | Fast typed System One proposals | Proposer over option menus | Always-call-model (or tool-loop that still burns tokens) |
| O(1) grammar hit | Bloom / HashMap LUT — **model never invoked** (A1) | N/A (option set already typed) | N/A | Rarely; model still in the loop |
| VoI=0 | **REFUSE** with limit `voi` (A2) | Outside product scope | Outside product scope | Keeps generating |
| Completeness `C(z)=1` | **REFUSE** satiation — economic reason code (A3) | Does not own economic done | Does not own economic done | No satiation stop |
| Irreversible commit | NI propose → certify → commit\|refuse → receipt (A4) | Does not own plant motion | Does not own plant motion | Often commits / acts without certificate |
| `ModelGenerated` | Never launders to Deterministic; cert required to commit | Proposals stay in typed System One | Same class | Model text treated as action |
| Joules | `estimated_j` always; `measured_j` only if meter (A5–A6) | Not the product claim | Not the product claim | Token/cost proxies; package joules often invented or omitted |
| Win metric | **J/query**, refuse correctness, certify rate | Latency / typed accuracy on menus | Latency / typed accuracy | Chat Elo / vibe benchmarks |

## Positioning (one paragraph)

**Laya System One** collapses generation cost when the option set is known ([laya-ai.com/system-one-models](https://laya-ai.com/system-one-models)). **Jev / Kev** sit in that typed-proposer class. Mixture of Limits does not replace them for closed menus. It owns the complementary global problem: **energy-bounded navigation**, **certify-before-commit**, and **satiation stop** after floors — including open chores until grammar/VoI say otherwise. Default LLM agents lose on J/query and refuse correctness because completeness is never written and ModelGenerated proposals commit without an NI certificate.

## What we do not claim

- Not higher chat Elo than frontier LLMs.
- Not shipped board / FPGA package energy (`board_synth_claimed=false` on soft-ref).
- Not inventing `measured_j` from Landauer or catalog μ.
- Not replacing Laya for typed known option sets.

## Operator proof

```bash
cd /Users/dcharlot/Desktop/mol-sync/mixture-of-limits
cargo run -p mol-cli -- prove          # PLAN + product A1–A6
cargo run -p mol-cli -- run --chore product/mol.yaml
cargo run -p mol-cli -- run --chore product/chores/financial_risk_scoring.yaml
```

Research triad: [Mixture of Limits](https://research.openie.dev/papers/mol/) (navigation) · [Notational Intelligence](https://research.openie.dev/papers/ni/) (commit) · [Satiation](https://research.openie.dev/papers/satiation/) (economic done).
