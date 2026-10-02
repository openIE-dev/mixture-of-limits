# ARENA.md — Jev Arena beside `mol bench`

**Purpose:** Shortlist and inspect **System One / Model LAST** residual proposers with [Jev Arena](https://github.com/theaiautomators/jev-arena). Keep those results separate from Mixture of Limits **J/query** benches. Do not invent or import Arena / JevBench / DecisionBench scores into Mixture of Limits receipts.

## Two different questions

| Harness | Question | Success |
|---|---|---|
| **`mol bench`** | Did Lookup → Formula → Solver close before a model? Was refuse / certify / joule labeling honest? | **J/query**, refuse correctness, certify rate; labels **Estimated \| Metered** only |
| **Jev Arena** | Among typed decision / classification profiles, which one is accurate, fast, and memory-fit for *this* deployment? | Decision / classification accuracy, latency, validity, workflow replay on pinned profiles |
| **JevBench** / **DecisionBench** | Same class as Arena: structured decision accuracy (different suites and scorers) | Leaderboard / sealed aggregates — use for shortlist, not for MoL close law |

Mixture of Limits owns navigation + NI certify + satiation. Arena owns residual-leaf measurement when Model LAST is allowed.

## Run Mixture of Limits bench (canonical)

```bash
cd /Users/dcharlot/Desktop/mol-sync/mixture-of-limits
cargo run -p mol-cli -- bench
cargo run -p mol-cli -- bench --json   # machine-readable; still Estimated|Metered only
cargo run -p mol-cli -- prove          # includes A10 bench-label honesty
```

Soft-ref never invents `measured_j`. MoE-sim rows are catalog surrogates, not RAPL / Arena GPU joules.

## Run Jev Arena (local lab)

Independent project; unaffiliated with TypeSafe or OpenIE. Follow upstream README for Node 22+, uv, Docker, and GPU toolkit.

**Linux:**

```sh
git clone https://github.com/theaiautomators/jev-arena.git
cd jev-arena
bash start-arena.sh
# smaller first roster:
bash start-arena.sh --prepare laya plumb decider
```

Open `http://127.0.0.1:8787`. Setup → prepare selected models → Run a new test (Smoke → Demo → Full). Full runs can take hours and may call hosted Jev if `TYPESAFE_API_KEY` is set.

**Windows:** `.\Start-Arena.ps1` (optional `-Prepare -Models laya,plumb,decider`).

Optional hosted Jev: copy `.env.example` → `.env`, set `TYPESAFE_API_KEY`, restart. Keys stay server-side; never commit `.env`.

Portable reports without local weights: see Arena `results/README.md` and published HTML / evidence packages.

## Recommended operator workflow

1. Write chore `C(z)` and floors in `product/mol.yaml` (or a chore under `product/chores/`).
2. Prove close law: `mol prove` / `mol run --chore …` with `allow_model=false` until floors fail honestly.
3. If Model LAST must open, shortlist residual proposers with Arena (or JevBench / DecisionBench for aggregate context).
4. Plug one proposer behind the residual leaf; keep NI certify between proposal and irreversible commit.
5. Re-run `mol bench` for J/query — do **not** paste Arena accuracy into MoL joule rows.

## Related links

| Link | Use |
|---|---|
| https://github.com/theaiautomators/jev-arena | Local decision-model lab |
| https://github.com/fstandhartinger/jevbench | Jev-class typed-decision benchmark |
| https://benchmarkheaven.com/jev-models | Live JevBench board |
| https://huggingface.co/spaces/Hanno-Labs/decision-bench-leaderboard | DecisionBench leaderboard |
| https://typesafe.ai/ · https://docs.typesafe.ai/models | Hosted Jev / System One API |
| [COMPETITIVE.md](./COMPETITIVE.md) | Positioning: floors first; proposers as residual leaf |

## Honesty fence

- No MoL claim of winning Arena matched-label accuracy, JevBench composite, or DecisionBench family scores.
- No geographic ranking of labs, networks, or GPUs — measure on the hardware and path you will ship.
- Teacher-labeled or generated Arena cases measure agreement under those protocols, not plant safety or satiation.
