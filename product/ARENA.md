# ARENA.md — `mol arena` head-on vs System One / frontier

**Purpose:** Run Arena-shaped chores — **typed decision / ticket-close / risk** — and score Mixture of Limits cascade against **frontier_sim** (always-model) and **system_one_leaf** (Jev/Laya-class stub). Compete directly. Floors win when they exist; Model LAST still used when needed; peers age; each stands alone. Never invent `measured_j`. Estimates ≠ `measured_j`.

## One harness, head-on scoreboard

| Strategy | What it is | How it closes |
|---|---|---|
| **mol_cascade** | Mixture of Limits Lookup → Formula → Solver → Model LAST | Floors first; VoI / `C(z)=1` refuse; NI certify |
| **frontier_sim** | Always-model / frontier catalog surrogate | Burns model joules; ignores satiation / VoI refuse law |
| **system_one_leaf** | Jev / Laya-class typed leaf stub | Known options → decide; **no** economic satiation; no VoI refuse |
| **real_leaf** (optional) | OpenAI-compatible Model LAST (`--endpoint`) | Same close-law miss as System One on C=1 / VoI=0; offline stub if no endpoint |

| Metric | Meaning |
|---|---|
| **correct_close** | Commit when expect commit; refuse satiation when `C(z)=1`; refuse VoI when VoI=0 |
| **refuse_when_C=1** | Satiation floor fired (MoL) vs peers that still “decide” |
| **estimated_j** | Catalog / receipt estimate — label **Estimated** (soft-ref) or **Metered** only |
| **latency** | Wall microseconds per chore |

External [Jev Arena](https://github.com/theaiautomators/jev-arena) remains useful to shortlist residual proposers. `mol arena` scores the **close law** those proposers sit under.

## Run

```bash
cd /Users/dcharlot/Desktop/mol-sync/mixture-of-limits
cargo run -p mol-cli -- arena
cargo run -p mol-cli -- arena --json
cargo run -p mol-cli -- arena --endpoint http://127.0.0.1:8080/v1 --profile laya
cargo run -p mol-cli -- bench --arena   # same path
cargo run -p mol-cli -- bench           # classic J/query vs always-model / MoE-sim
cargo run -p mol-cli -- prove           # includes A10 labels + A14 arena honesty
```

Published soft-ref table: [ARENA_RESULTS.md](./ARENA_RESULTS.md). Model LAST adapter: offline stub by default; `--endpoint` for OpenAI-compatible inference; HF paths for Laya / Jev / Decider are documented — never invent `measured_j`.

Soft-ref never invents `measured_j`. Frontier / System One / MoE-sim joules are **Estimated** catalog surrogates — not RAPL, not Arena GPU package joules.

## Chore shapes (fixtures under `product/fixtures/`)

| Kind | Example ask | Expect |
|---|---|---|
| Ticket-close LUT | `R-HOWTO` / `R-OK` / `R-DUP` / `R-BUGFIX` / `R-WONTFIX` / `R-REFUND` | Commit Lookup (model cold) |
| Risk LUT | `RISK-LOW` / `RISK-MED` / `RISK-HIGH` | Commit Lookup |
| Typed decision | `D-APPROVE` / `D-DENY` / `D-ESCALATE` | Commit Lookup |
| **Risk Formula** | `risk score compute severity=… exposure=… likelihood=…` (LUT miss) | Commit **Formula** closed-form band |
| **Ticket Solver** | `ticket route …` / `ticket sat assign …` / `solve knapsack …` (LUT miss) | Commit **Solver** rules / SAT / tiny LP |
| Satiation | ticket / risk / typed + `C(z)=1` | MoL **REFUSE** `satiation`; peers still commit |
| VoI=0 | free-form poem / essay / story | MoL **REFUSE** `voi`; peers still commit |

## Recommended operator workflow

1. Write chore `C(z)` and floors in `product/mol.yaml` (or `product/chores/`).
2. Prove close law: `mol prove` / `mol run --chore …` with `allow_model=false` until floors fail honestly.
3. Score head-on: `mol arena` — report correct_close, refuse-when-C=1, estimated_j, latency.
4. If Model LAST must open, shortlist residual proposers with Jev Arena / JevBench / DecisionBench.
5. Plug one proposer behind the residual leaf; keep NI certify between proposal and irreversible commit.
6. Re-run `mol bench` / `mol arena` — do **not** invent measured board joules.

## Related links

| Link | Use |
|---|---|
| https://github.com/theaiautomators/jev-arena | Local decision-model lab (residual shortlist) |
| https://github.com/fstandhartinger/jevbench | Jev-class typed-decision benchmark |
| https://benchmarkheaven.com/jev-models | Live JevBench board |
| https://huggingface.co/spaces/Hanno-Labs/decision-bench-leaderboard | DecisionBench leaderboard |
| https://typesafe.ai/ · https://docs.typesafe.ai/models | Hosted Jev / System One API |
| [COMPETITIVE.md](./COMPETITIVE.md) | Head-on positioning + soft-ref numbers |

## Honesty

- No invented `measured_j`. Soft-ref rows are **Estimated**.
- No invented Arena / JevBench composite scores pasted into MoL receipts.
- Frontier catalog `estimated_j` is a surrogate — expensive by design, not a meter claim.
- System One leaf stub costs are catalog encoder-class estimates — not hosted Jev package joules.
