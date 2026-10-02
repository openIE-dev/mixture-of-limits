# Discovery inventory — Mixture of Limits / WCA / NI / Laya

## On disk (machine `4e1505fa-e469-4612-9792-38d478ba57e6`)

| Path | What |
|---|---|
| `/Users/dcharlot/Desktop/mol-sync/mixture-of-limits/` | **Primary clean-room Rust reference** — workspace crates `mol-core` … `mol-cli` / `mol-desktop`; `mol prove` (PLAN + A1–A6); git → `openIE-dev/mixture-of-limits` |
| `/Users/dcharlot/Desktop/mol-sync/*.tgz` | Kernel / meter / multi-fabric sync archives |
| `/Users/dcharlot/data-share/vibe-coding/research-openie-web/` | Living papers hub (git → `openIE-dev/research-openie-web`) — MoL / NI / Satiation triad |
| `/Users/dcharlot/data-share/vibe-coding/openie-fpga/` | WCA commit-gate FPGA artifacts (`wca_commit_gate_pt_v2.*`) |
| `/Users/dcharlot/data-share/vibe-coding/research-openie-web/content-src/stage-c/` | WCA Verilog / UART / board-evidence companions |
| This package | `/Users/dcharlot/data-share/vibe-coding/mixture-of-limits-product/` |

No separate `wca-lut-edge` checkout found under `data-share/vibe-coding` in this sweep (cited as sibling pattern in MoL README).

## GitHub (`openIE-dev` / `dcharlot*`)

| Repo | Status |
|---|---|
| `openIE-dev/research-openie-web` | **Exists** — MoL/NI/Satiation papers |
| `openIE-dev/mixture-of-limits` | **Exists** — https://github.com/openIE-dev/mixture-of-limits (clean-room + `product/`) |
| `dcharlot` personal MoL/Laya repos | No MoL/Laya product repo in filtered `openIE-dev` list |

Auth account used for list: `dcharlot65-openie`.

## Laya / System One citations (already in research)

| Location | Citation |
|---|---|
| `research-openie-web/src/content/papers/ni.md` | System One (Laya / Jev class); link `https://laya-ai.com/system-one-models`; arXiv:2503.23303, arXiv:2510.01237 |
| `research-openie-web/src/content/papers/satiation.md` | System One glossary + Laya product page bibliography |
| `research-openie-web/docs/EDUCATIONAL_PROSE.md` | System One = typed decision models (Laya / Jev class) |
| `research-openie-web/src/pages/index.astro` | “System One decides in software. WCA decides whether the machine is allowed to move.” |
| Living NI/Satiation figures | System One → WCA → Plant complement diagrams |

No Laya source code under openIE-dev; citations are product-page / preprint references only.

## Decision

Wire product docs into the **Desktop clean-room** (`product/` mirror) and own the git package under **vibe-coding/mixture-of-limits-product**. Do not orphan docs: CLI stubs target Rust `mol-cli`.

## Jev Arena / System One measurement ecosystem (product competitive)

| Resource | Role |
|---|---|
| https://github.com/theaiautomators/jev-arena | Local decision-model lab (accuracy, speed, memory, workflow replay) |
| https://github.com/fstandhartinger/jevbench | Independent Jev-class typed-decision benchmark |
| https://benchmarkheaven.com/jev-models | Live JevBench board |
| https://huggingface.co/spaces/Hanno-Labs/decision-bench-leaderboard | DecisionBench leaderboard |
| https://typesafe.ai/ · https://docs.typesafe.ai/models | Hosted Jev System One API |
| Models (residual leaf class) | EldanRing/Winnow-12B, Mapika/decider-4b, convaiinnovations/laya, TheoLeeCJ/SemIf-OpenJev, bespokelabs/Bespoke-Nimble-9B, Contrastive-LM/CLM-v0.1-8B, crh225/plumb-4b, Qwen/Qwen3.5-4B, MoritzLaurer/ModernBERT-large-zeroshot-v2.0 |

Product positioning: [COMPETITIVE.md](../COMPETITIVE.md), [ARENA.md](../ARENA.md). Do not invent Arena scores as Mixture of Limits wins.
