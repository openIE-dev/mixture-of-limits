# Appendix — Mechanistic World Models & AI-Newton → MoL

**Date:** 2026-09-30 (America/New_York)  
**Parent:** [BLUEPRINT.md](../BLUEPRINT.md) · hunt inventory: [adjacent-field-hunt.md](./adjacent-field-hunt.md)  
**Scope:** Deep MoL mapping for two Tier-A allies. Implications for `mol-cascade` / `mol-limits` Rust crates. No code changes in this appendix — traits/limits to **borrow** vs **refuse**.

---

## 1. Mechanistic World Models (MWMs)

**Cite:** Ingmar Posner, Anson Lei, Bernhard Schölkopf — *From Observation to Insight: Mechanistic World Models and the Quest for Autonomous Discovery*  
**arXiv:** [2607.12474](https://arxiv.org/abs/2607.12474) · [HTML](https://arxiv.org/html/2607.12474) · [PDF](https://arxiv.org/pdf/2607.12474.pdf)  
**Class:** mechanistic / compression-first (**blueprint paper**, not a shipped product)

### 1.1 Core claim (capsule)

Prediction ≠ scientific discovery. Foundation models and conventional world models organize knowledge around **forecasting observations**. Scientific understanding requires **reusable explanatory mechanisms**. MWMs make mechanisms the centre of representation, computation, and learning.

Anatomy (paper eq. 8):

```text
W ≜ (T, Z, M, S)
  T  — semantic variable types
  Z  — typed latent variables
  M  — library of reusable mechanisms m ≜ (Σ_m, f_m)
  S  — bindings that instantiate mechanisms on a system's variables
```

Inductive pressures: **parsimony** (smallest mechanism library explaining most observations) + **compositionality** (reuse/recombine rather than relearn). Design principles: joint predictive+explanatory modelling; mechanisms as fundamental unit; mechanism reuse; hierarchical organisation.

### 1.2 Mechanism library vs MoE (critical for MoL ≠ MoE)

MWM §3.4 explicitly discusses **Mixture of Experts**:

| MoE (as MWM critiques it) | MWM mechanism library | MoL named floors |
|---|---|---|
| Experts partition *predictive* computation | Modules must be *explanatory mechanisms* | Limits live *outside* generation |
| Routing optimizes likelihood / loss | Binding optimizes reuse + parsimony + prediction | Routing = cheapest-sufficient cascade |
| More experts → capacity for same generative act | More mechanisms only when reuse fails | More named floors → sharper refuse |
| Specialization need not match world structure | Typed signatures + bindings force structure | Grammar coverage / VoI / energy bind |

**MoL citation line:** MWMs are the strongest *philosophical* ally for “MoL ≠ MoE.” MoE is predictive organization inside the generative corridor; MoL (and MWMs) put structured constraints **outside** unconstrained next-token escalation. Do **not** rebrand MoE experts as MoL limits.

### 1.3 Floors / refuse — how MWMs rhyme with MoL

MWM does not use MoL vocabulary, but maps cleanly:

| MWM idea | MoL floor / gear |
|---|---|
| Parsimony pressure Ω_p on (T, M, Z) | `information` / `voi` — more bits of mechanism library only when explanatory scope rises |
| Compositionality Ω_c across bindings | Grammar reuse — prefer Lookup/Formula already in registry |
| Active inquiry / experimental design | Escalate only when informative; else stop (VoI) |
| “Add new mechanism only when necessary” | Refuse escalate to Model when Formula/Solver cover |
| Critique of post-hoc mechanistic interpretability | Explanatory organisation must be **encouraged during learning**, not extracted after — parallel to MoL demoting NN as residual leaf |
| Orbital-mechanics probe (Vafa et al.): predictors without Newtonian internals | Prediction ≠ mechanism — same MoL attack on AlphaFold-as-physics marketing |

### 1.4 Critique of AI Scientists (MWM §5)

Posner/Lei/Schölkopf:

> AI Scientists / Autonomous Discovery / AI-for-Science agent stacks (Sakana-class, Co-Scientist-class, etc.) **automate workflow** (plan, code, write, review) but **inherit predictive foundation-model substrates**. They do not solve how **explanatory knowledge** should be represented.

They footnote **Robot Scientist** (Sparkes/King; Gower et al.) as earlier workflow automation — same distinction: automate the cycle ≠ learn mechanisms.

**MoL agreement:** Sakana / paper mills = Tier C antagonists. MoL wager = **A (formula/mechanism) + B (physics leaves)** beat **C (token escalation)**. MWMs supply the philosophy; MoL supplies the **executable navigation law** (cascade + receipts + refuse).

### 1.5 Cousins MWM Table 1 (for crate borrowing)

| Cousin | MoL borrow hint |
|---|---|
| DreamCoder / NEO / PoE-World | Mechanism-as-program library ↔ `Lookup` registry of named formulas |
| COMET | Compete-and-compose bindings ↔ cascade step selection with reuse |
| VCD / DIDS / SPARTAN | Variable/structure discovery — **phase 3** Periodic Stack / `primitive_gap`, not v0.1 |
| Equation discovery (SINDy, AI Feynman) | Already Tier A Formula gear |

---

## 2. AI-Newton

**Cite:** You-Le Fang, Dong-Shan Jian, Xiang Li, Yan-Qing Ma (Peking Univ.) — *AI-Newton: A Concept-Driven Physical Law Discovery System without Prior Physical Knowledge*  
**arXiv:** [2504.01538](https://arxiv.org/abs/2504.01538) · [HTML](https://arxiv.org/html/2504.01538) · GitHub: [Science-Discovery/AI-Newton](https://github.com/Science-Discovery/AI-Newton)  
**Class:** true physics-executable cascade (+ light hybrid recommendation NN)

### 2.1 Core claim (capsule)

Single-experiment SR/NN fits produce **specific** models. Human physicists extract **concepts** and **general** laws across many experiments. AI-Newton:

1. Proposes interpretable physical **concepts** (DSL symbols) to build laws.  
2. Progressively **generalizes** laws to broader domains via **plausible reasoning**.  

Applied to **46** noisy classical-mechanics experiments → rediscovers Newton II, energy conservation (kinetic + elastic + gravitational potentials), universal gravitation; ~90 concepts / ~50 general laws typical run; also numerically equates gravitational vs inertial mass (weak equivalence signal).

Architecture:

```text
Experiment base  ↔  Theory base (symbols → concepts → specific laws → general laws)
                         ↑
              Autonomous discovery workflow
              (UCB+NN recommender, era-control, SR, differential algebra simplify)
```

### 2.2 Cascade mapping (Lookup → Formula → Solver → Model LAST)

| MoL gear | AI-Newton component | Mapping note |
|---|---|---|
| **Lookup** | Theory base concept/law registry; intrinsic-concept measurement procedures that *retrieve* values via defining experiments | Registry of named concepts = MoL LUT / grammar entries |
| **Formula** | Specific laws via SR (instantiation-verification + PCA differential polynomial regression); general laws as compact symbolic equations (e.g. Newton II with potentials) | Primary gear — closed-form commit |
| **Solver** | Rosenfeld–Gröbner differential-algebra simplification; numerical controlled-variable dependency analysis; ODE data generators for experiments | Sparse/symbolic solve + verification — not MD/DFT, but certifying algebra |
| **Model LAST** | Dynamically adapted NN inside **recommendation engine** only (experiment/concept selection); **not** the law substrate | Correct demotion — NN is residual orchestrator for search, not the physics |

**Era-control strategy** ≈ MoL floors: early eras have short wall-clock budgets (force simple experiments / cheap Formula); escalate era time exponentially only after knowledge stalls — **escalate only on miss**, refuse unbounded search.

### 2.3 What to cite vs what not to overclaim

- **Cite:** multi-experiment concept libraries; general vs specific laws; NN-as-recommender; era budgets as VoI/latency floors.  
- **Do not overclaim:** rediscovery of known mechanics ≠ frontier discovery; future LLM augmentation of DSL/recommender (paper’s own forward look) must remain **leaf**, or MoL invariant breaks.

---

## 3. Comparison table

| Axis | **MWM** | **AI-Newton** | **Buehler SciAgents / AtomAgents / SparksMatter** | **MoL / MathGround** | **FM-hype (Fractal Mechanics / Consciousness Project)** |
|---|---|---|---|---|---|
| Primary artifact | Conceptual blueprint: mechanism library (T,Z,M,S) | Running system: DSL concepts + general laws from 46 expts | Multi-agent LLM + KG / MD / DFT loops | Navigation law + Rust cascade + joule receipts | Narrative φ/Fibonacci ToE cascade |
| Substrate | Mechanisms (explanatory) | Symbolic concepts & laws | LLM swarm + physics tools | Floors outside generation; Model demoted | Aesthetic cascade language |
| MoE stance | Explicit critique — MoE = predictive modules | N/A (not MoE) | Uses agents/tools, not MoE-as-science | **MoL ≠ MoE** hard invariant | Irrelevant / marketing adjacent |
| Compression | Parsimony + compositionality pressures | Concise concepts enable SR; law simplification | Reports/hypotheses; physics leaf compresses reality | Formula/LUT first; VoI stop | Claims compression via φ — **unvalidated** |
| Executable floor | Not shipped; cousins (COMET, DreamCoder) | SR + differential algebra + simulated noisy ODE data | MD / DFT / (Matter 2025) in-situ experiment | Lookup/Formula/Solver + refuse; Landauer-labeled estimates | Missing community-validated Solver receipts |
| Agent role | Active inquiry *after* mechanism substrate | Recommender picks experiments/concepts | Agents *are* the product (plan/critique/run) | AutomateGate: propose→certify→commit\|refuse | Notebook/LLM exploration as proof-substitute |
| Risk if copied naively | Stay conceptual forever | Scope locked to rediscovery mechanics | LLM paper-mill without DFT refuse | Overbuilding adapters before law proofs | Citing as physics |
| Best MoL use | Sister law citation; mechanism-registry design | Formula/Lookup trait shapes; era = budget floors | Tier B hybrid exemplar; agent-builds-instruments | Self | Skeptical inventory only |

---

## 4. Concrete implications for Rust crates

### 4.1 `mol-cascade` — borrow

| Borrow | Source | Suggested shape (design only) |
|---|---|---|
| **Concept / law registry as Lookup** | AI-Newton theory base | Extend `GrammarCoverage` / Lookup gear with named `ConceptId` / `LawId` entries (stringly v0.1 → typed later) |
| **Specific vs general law tiers** | AI-Newton | Optional metadata on Formula hits: `LawScope::{Specific, General}` — prefer General when multiple match |
| **Mechanism signature typing** | MWM `Σ_m` typed roles | Future `Formula` / Solver APIs: typed input/output roles (aligns Periodic Stack primitives phase 3) |
| **NN residual recommender pattern** | AI-Newton recommendation NN | Keep `ModelStub` as **selector** of which Lookup/Formula to try — never as answer emitter by default |
| **Era / budget escalation** | AI-Newton era-control | Map to existing `Budget { max_j, max_latency, allow_model }` — document era as latency ladder in BLUEPRINT examples |

### 4.2 `mol-limits` — borrow

| Borrow | Source | Suggested limit / policy |
|---|---|---|
| Parsimony stop | MWM Ω_p | Strengthen `voi` / `information`: refuse new Model tokens when grammar already covers |
| Mechanism-reuse preference | MWM compositionality | Soft policy: fire `grammar` coverage before opening Solver/Model |
| Proof / consistency refuse | AI Hilbert cousin (hunt row 23) | Future: `wca_refuse`-like certify for symbolic commits — **not** in v0.1 |
| Experiment-stall escalate | AI-Newton eras | Only raise `max_latency` / allow Solver after N Lookup+Formula misses — already spirit of cascade order |

### 4.3 Refuse (do **not** import)

| Refuse | Why |
|---|---|
| **MoE routing as MoL** | MWM §3.4 + BLUEPRINT §1 — experts inside generative corridor ≠ named floors |
| **Unconstrained AI Scientist paper loops** | MWM §5 — workflow without mechanism substrate; violates Model LAST + receipts |
| **Training / shipping weights in-tree** | BLUEPRINT §5 — no candle/tract; Model remains stub leaf |
| **φ / Fractal Mechanics cascades as floors** | No falsification receipts; ToE-hype |
| **Equating MLIP/GNN prediction with Formula** | Prediction ≠ mechanism (MWM + MoL AlphaFold lesson) |
| **Silent estimate = RAPL** | Honesty invariant — MWM/AI-Newton do not license fake joule probes |
| **Auto-merge sibling crates** | Policy: openie-leapfrog / jouledb / wca-lut-edge remain pattern-only |

### 4.4 One-line crate motto

> **From MWM:** organize knowledge as a *mechanism/limit library*, not an expert mixture.  
> **From AI-Newton:** escalate Lookup→Formula→Solver with *era budgets* and keep any NN as *recommender residual*.  
> **From both:** refuse AI-Scientist token mills and FM-hype that skip certify/receipt floors.

---

## 5. Cross-links

- Hunt inventory rows **4** (AI-Newton), **6** (MWM), Sweep-2 rows **23–28** (Hilbert, DreamCoder, COMET, …): [adjacent-field-hunt.md](./adjacent-field-hunt.md)  
- Law statement & cascade: [BLUEPRINT.md](../BLUEPRINT.md) §§1–3  
- User-facing summary: [README.md](../README.md)


---

## Sweep 3 cross-link — Buehler ChemRxiv gap (2026-09-30 ET)

**ChemRxiv / arXiv DOI for “models building models” / architected-graphene AI agent:** **STILL MISSING.** Titled manuscript + reproducible release exist without preprint ID:

- Title: *A model builds a model: an AI agent constructs a validated atomistic instrument and uses it to discover what sets the strength of architected graphene* (M. J. Buehler, 2026; `CITATION.cff` notes arXiv ID to be added).
- Code/data: [lamm-mit/graphene-agent](https://github.com/lamm-mit/graphene-agent) · [HF graphene-agent-data](https://huggingface.co/datasets/lamm-mit/graphene-agent-data); sibling archive [MetaMaterialsDiscovery](https://huggingface.co/lamm-mit/MetaMaterialsDiscovery) (placeholder `arXiv:xxxx.yyyyy`).
- Separate **FOUND** journal DOI (atom-by-atom MAC, not the metamaterial simulator narrative): Matter 2025 [10.1016/j.matt.2025.102000](https://doi.org/10.1016/j.matt.2025.102000).

Full candidate table: [adjacent-field-hunt.md](./adjacent-field-hunt.md) § Sweep 3. Do not add a fake ChemRxiv/arXiv DOI to BLUEPRINT until minted.
