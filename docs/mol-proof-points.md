# Mixture of Limits — Verified Citation Pack

**Project:** Mixture of Limits (MoL) / OpenIE / David Charlot  
**Thesis:** MoL is information-theoretic and physics-rooted. Academia already stated the core (floors, compression, predictive laws, refuse excess). The bottleneck is **applied mathematics × materials science** (physical computer hardware embodiment), not awareness of what should be done.  
**Date assembled:** 2026-10-01 (America/New_York)  
**Method:** WebSearch / WebFetch primary-source checks; skim of Mac `adjacent-field-hunt.md` + `appendix-mwm-ai-newton.md` + `quantum-thermo-collapse.md` (machineId `25022ed7-…`).  
**Rule:** No invented DOIs, arXiv IDs, quotes, or measured joules. Entries marked **VERIFIED** passed a live search/fetch; **TBD** failed or were not locked.

**Status counts (this file):** **39 VERIFIED** citation rows · **3 TBD/omitted** (CRANE-as-router omitted; Buehler graphene preprint ID TBD; AutoSINDy/KeplerAgent pending re-fetch)

---

## 1. Historical lineage

Compression to laws, statistical mechanics ↔ information, algorithmic complexity, maxent, and decision-theoretic VoI — the conceptual floors MoL names.

### 1.1 Physics: floors as predictive laws

| # | Authors | Year | Title | Venue / ID | MoL one-liner | Status |
|---|---|---|---|---|---|---|
| H1 | Johannes Kepler | 1619 | *Harmonices Mundi* (esp. Book V; elliptical planetary laws) | Linz: Johann Planck (classic treatise) | Data → closed-form orbital laws: canonical **Formula** compression, not trajectory narration. | VERIFIED (primary treatise; no DOI expected) |
| H2 | Isaac Newton | 1687 | *Philosophiæ Naturalis Principia Mathematica* | London: Royal Society | Unifies celestial/terrestrial motion under few laws — refuse excess epicycles; **grammar of mechanisms**. | VERIFIED (primary treatise) |
| H3 | James Clerk Maxwell | 1873 | *A Treatise on Electricity and Magnetism* | Oxford: Clarendon Press | Field equations as reusable mechanisms — predictive compression across experiments. | VERIFIED (primary treatise) |
| H4 | Ludwig Boltzmann | 1896–1898 (lectures; Eng. trad. later) | *Vorlesungen über Gastheorie* / Lectures on Gas Theory | Leipzig: J.A. Barth | Microscopic statistics → macroscopic laws; entropy as bridge to information floors. | VERIFIED (primary lectures; cite edition carefully) |
| H5 | J. Willard Gibbs | 1902 | *Elementary Principles in Statistical Mechanics* | New York: Charles Scribner’s Sons | Ensemble formalism; least-commitment probabilistic description given constraints. | VERIFIED |

### 1.2 Information theory & thermodynamic floors

| # | Authors | Year | Title | Venue / ID | MoL one-liner | Status |
|---|---|---|---|---|---|---|
| H6 | Claude E. Shannon | 1948 | A Mathematical Theory of Communication | *Bell System Technical Journal* 27:379–423 & 623–656; DOI [10.1002/j.1538-7305.1948.tb01338.x](https://doi.org/10.1002/j.1538-7305.1948.tb01338.x) (July part) | Channel capacity is a **hard floor** on bits; more symbols past capacity buy nothing — refuse excess tokens. | VERIFIED |
| H7 | Rolf Landauer | 1961 | Irreversibility and Heat Generation in the Computing Process | *IBM Journal of Research and Development* 5(3):183–191; DOI [10.1147/rd.53.0183](https://doi.org/10.1147/rd.53.0183) | Logical erase ⇒ physical dissipation (~*kT* ln 2 lower bound per bit) — **energy floor** on irreversible generation. | VERIFIED |
| H8 | A. N. Kolmogorov | 1965 | Three Approaches to the Quantitative Definition of Information | *Problems of Information Transmission* 1(1):3–11 (MathNet PPI) | Shortest-program complexity = compression floor; random strings refuse further description. | VERIFIED (venue locked; **no DOI** in MathNet record — cite journal+pages) |
| H9 | Ray J. Solomonoff | 1964 | A Formal Theory of Inductive Inference, Parts I–II | *Information and Control* 7:1–22 & 224–254; DOI [10.1016/S0019-9958(64)90223-2](https://doi.org/10.1016/S0019-9958(64)90223-2) (Part I) | Universal prior favors short explanations — **parsimony as inductive floor** vs brute hypothesis enumeration. | VERIFIED |
| H10 | Gregory J. Chaitin | 1966 | On the Length of Programs for Computing Finite Binary Sequences | *Journal of the ACM* 13(4):547–569; DOI [10.1145/321356.321363](https://doi.org/10.1145/321356.321363) | Algorithmic randomness / program-length lower bounds — incompressible objects refuse further compression. | VERIFIED |
| H11 | E. T. Jaynes | 1957 | Information Theory and Statistical Mechanics | *Physical Review* 106(4):620–630; DOI [10.1103/PhysRev.106.620](https://doi.org/10.1103/PhysRev.106.620) | Maximum-entropy priors: least-biased distributions given constraints — refuse inventing structure. | VERIFIED |
| H12 | Ronald A. Howard | 1966 | Information Value Theory | *IEEE Transactions on Systems Science and Cybernetics* 2(1):22–26; DOI [10.1109/TSSC.1966.300074](https://doi.org/10.1109/TSSC.1966.300074) | Value of information is economic, not Shannon alone — **VoI floor**: buy bits only when decisions improve. | VERIFIED |
| H13 | Howard Raiffa (& related decision-analysis line) | 1968 | *Decision Analysis: Introductory Lectures on Choices under Uncertainty* | Addison-Wesley | Decision-theoretic framing of when information is worth obtaining — cascade escalate-only-on-VoI. | VERIFIED (book; pair with Howard 1966 for VoI) |
| H14 | Thomas M. Cover & Joy A. Thomas | 2006 (2nd ed.) | *Elements of Information Theory* | Wiley; ISBN 978-0-471-24195-9; online [10.1002/047174882X](https://doi.org/10.1002/047174882X) | Canonical textbook of capacity, rate-distortion, typical sets — floors as theorems, not slogans. | VERIFIED |

---

## 2. AI / automation precursors

Arguments that intelligence is constrained search, control under resources, symbols/mechanisms — not unbounded generation.

| # | Authors | Year | Title | Venue / ID | MoL one-liner | Status |
|---|---|---|---|---|---|---|
| A1 | Norbert Wiener | 1948 | *Cybernetics: Or Control and Communication in the Animal and the Machine* | MIT Press (Cambridge, MA) | Control/feedback under noise — intelligence as regulation within physical limits, not open-loop token flood. | VERIFIED |
| A2 | W. Ross Ashby | 1956 | *An Introduction to Cybernetics* | Chapman & Hall | Law of Requisite Variety: regulator must match disturbance variety — **floor on control capacity**. | VERIFIED |
| A3 | John McCarthy | 1959 | Programs with Common Sense | Mechanisation of Thought Processes (NPL Symposium); Stanford reprint | Knowledge representation + deduction over brute search; Advice Taker as structured reasoning substrate. | VERIFIED (primary essay; supports structured knowledge vs pure generate) |
| A4 | Marvin Minsky | 1961 | Steps Toward Artificial Intelligence | *Proceedings of the IRE* 49(1):8–30 (classic survey) | Search, pattern, planning, induction as **bounded** problem-solving — efficiency matters; not “scale alone.” | VERIFIED (caution: era survey, not an anti-LLM rant — cite for resource-aware AI framing) |
| A5 | Allen Newell & Herbert A. Simon | 1976 | Computer Science as Empirical Inquiry: Symbols and Search | *Communications of the ACM* 19(3):113–126; DOI [10.1145/360018.360022](https://doi.org/10.1145/360018.360022) | Physical symbol systems + heuristic search — intelligence as constrained symbol manipulation, not residual generation. | VERIFIED |
| A6 | Judea Pearl | 2000 / 2009 | *Causality: Models, Reasoning, and Inference* | Cambridge University Press (2nd ed. 2009) | Interventions/counterfactuals need causal structure — prediction ≠ explanation (**mechanism floor**). | VERIFIED |
| A7 | Judea Pearl | 2018 | Theoretical Impediments to Machine Learning With Seven Sparks from the Causal Revolution | arXiv:[1801.04016](https://arxiv.org/abs/1801.04016) | Model-free ML cannot answer do/counterfactual queries — supports MoL demotion of pure predictive corridor. | VERIFIED |
| A8 | Gary Marcus | 2018 | Deep Learning: A Critical Appraisal | arXiv:[1801.00631](https://arxiv.org/abs/1801.00631) | Correlation-only DL lacks causal/symbolic floors — **use only for “floors needed”**, not culture-war quotes. | VERIFIED (include carefully; supports floors, not MoL branding) |

---

## 3. Recent publications (~last 10 years preferred)

Formula/mechanism discovery, certify/refuse, cost/energy routing. Sourced and cross-checked against the MoL adjacent-field hunt (2026-09-30).

### 3.1 Formula / mechanism discovery (compression floors)

| # | Authors | Year | Title | Venue / ID | MoL one-liner | Status |
|---|---|---|---|---|---|---|
| R1 | Steven L. Brunton, Joshua L. Proctor, J. Nathan Kutz | 2016 | Discovering governing equations from data by sparse identification of nonlinear dynamical systems (SINDy) | *PNAS* 113(15):3932–3937; DOI [10.1073/pnas.1517384113](https://doi.org/10.1073/pnas.1517384113) | Sparse library + regression → parsimonious ODEs — **explicit sparsity floor** vs dense residual fits. | VERIFIED |
| R2 | Silviu-Marian Udrescu & Max Tegmark | 2020 | AI Feynman: A physics-inspired method for symbolic regression | *Science Advances* 6(16):eaay2631; DOI [10.1126/sciadv.aay2631](https://doi.org/10.1126/sciadv.aay2631) | Symmetry/separability → closed-form laws (100/100 Feynman eqs) — NN as decomposer, not substrate. | VERIFIED |
| R3 | Miles Cranmer | 2023 | Interpretable Machine Learning for Science with PySR and SymbolicRegression.jl | arXiv:[2305.01582](https://arxiv.org/abs/2305.01582) | Production evolutionary SR — compression to human equations as discovery objective. | VERIFIED |
| R4 | You-Le Fang, Dong-Shan Jian, Xiang Li, Yan-Qing Ma | 2025 | AI-Newton: A Concept-Driven Physical Law Discovery System without Prior Physical Knowledge | arXiv:[2504.01538](https://arxiv.org/abs/2504.01538) | Multi-experiment concept library + SR; era budgets escalate only when stuck — **Lookup/Formula first, NN recommender last**. | VERIFIED |
| R5 | Ryan Cory-Wright, Bachir El Khadir, et al. (IBM) | 2024 | Evolving scientific discovery by unifying data and background knowledge with AI Hilbert | *Nature Communications*; DOI [10.1038/s41467-024-50074-w](https://doi.org/10.1038/s41467-024-50074-w); arXiv:[2308.09474](https://arxiv.org/abs/2308.09474) | Data + axioms with Positivstellensatz certificates — **proof floor** on discovered polynomial laws. | VERIFIED |
| R6 | Cristina Cornelio et al. (IBM) | 2023 | Combining data and theory for derivable scientific discovery with AI-Descartes | *Nature Communications*; DOI [10.1038/s41467-023-37236-y](https://doi.org/10.1038/s41467-023-37236-y) | Theory-gated SR: reject formulas inconsistent with axioms — refuse excess unconstrained fits. | VERIFIED |
| R7 | Ingmar Posner, Anson Lei, Bernhard Schölkopf | 2026 | From Observation to Insight: Mechanistic World Models and the Quest for Autonomous Discovery | arXiv:[2607.12474](https://arxiv.org/abs/2607.12474) | Prediction ≠ discovery; reusable mechanisms + parsimony; **critiques MoE as predictive, not mechanistic** — strongest MoL≠MoE ally. | VERIFIED |

### 3.2 Certify / refuse (formal & neuro-symbolic floors)

| # | Authors | Year | Title | Venue / ID | MoL one-liner | Status |
|---|---|---|---|---|---|---|
| R8 | Trieu H. Trinh et al. (DeepMind) | 2024 | Solving olympiad geometry without human demonstrations (AlphaGeometry) | *Nature*; DOI [10.1038/s41586-023-06747-5](https://doi.org/10.1038/s41586-023-06747-5) | LM proposes; symbolic engine verifies — **hallucinated steps fail the floor**. | VERIFIED |
| R9 | Google DeepMind (AlphaProof line) | 2025 | Olympiad-level formal mathematical reasoning with reinforcement learning | *Nature*; DOI [10.1038/s41586-025-09833-y](https://doi.org/10.1038/s41586-025-09833-y) | Lean-certified proof search — propose→certify→commit\|refuse template for science cascades. | VERIFIED |

### 3.3 Cost / energy-aware ML (refuse excess compute)

| # | Authors | Year | Title | Venue / ID | MoL one-liner | Status |
|---|---|---|---|---|---|---|
| R10 | Emma Strubell, Ananya Ganesh, Andrew McCallum | 2019 | Energy and Policy Considerations for Deep Learning in NLP | ACL 2019; DOI [10.18653/v1/P19-1355](https://doi.org/10.18653/v1/P19-1355); arXiv:[1906.02243](https://arxiv.org/abs/1906.02243) | Training energy/carbon as first-class cost — escalate compute only with accounted joules (estimates, not MoL board meters). | VERIFIED |
| R11 | Roy Schwartz, Jesse Dodge, Noah A. Smith, Oren Etzioni | 2020 | Green AI | *Communications of the ACM* 63(12):54–63; DOI [10.1145/3381831](https://doi.org/10.1145/3381831) | Efficiency as evaluation criterion alongside accuracy — institutionalize floors against Red AI scale-only culture. | VERIFIED |
| R12 | Isaac Ong et al. | 2024 | RouteLLM: Learning to Route LLMs with Preference Data | arXiv:[2406.18665](https://arxiv.org/abs/2406.18665) | Preference-trained routers: weak model when enough — **cost floor** vs always-call-strong (partial MoL cousin; still inside generative corridor). | VERIFIED |
| R13 | Lingjiao Chen, Matei Zaharia, James Zou | 2023 | FrugalGPT: How to Use Large Language Models While Reducing Cost and Improving Performance | arXiv:[2305.05176](https://arxiv.org/abs/2305.05176) | Cascades/routing for cost–quality tradeoff — escalate model tier only when needed. | VERIFIED |

### 3.4 Explicitly omitted / TBD (do not invent)

| Item | Reason | Status |
|---|---|---|
| **CRANE** as RouteLLM companion | User flag: include only if verified. Live search shows **CRANE: Reasoning with constrained LLM generation** (arXiv:2502.09061) — constrained decoding, **not** a MoL/RouteLLM-style cost router. Omit from routing cites. | OMITTED (different paper) |
| Buehler graphene “A model builds a model…” metamaterial agent | Code/data public (`lamm-mit/graphene-agent`); `CITATION.cff` says arXiv ID **to be added** — no ChemRxiv/arXiv ID as of 2026-09-30 hunt. | TBD (do not cite with fake ID) |
| AutoSINDy arXiv:2605.09696, KeplerAgent 2602.12259, etc. | Present in adjacent hunt; **not re-verified in this pack pass** — promote only after fresh abs fetch. | TBD pending re-fetch |

---

## 4. Bottleneck claim — applied math × materials embodiment

**Claim (careful):** Conceptual clarity on floors (Shannon capacity, Landauer erase cost, Kolmogorov/Solomonoff parsimony, VoI, sparse governing equations, mechanistic libraries) is **not** the scarce resource. What lags is **embodying** those floors in physical computers and materials stacks: devices, interconnects, memory, reversible/adiabatic logic, and applied-math compilers that map named limits onto silicon (or post-CMOS) with honest energy receipts.

### 4.1 What the literature supports (verified, no fake hardware papers)

| # | Source | Year | How it supports the bottleneck claim | Status |
|---|---|---|---|---|
| B1 | Landauer 1961 (H7) | 1961 | Sets a **thermodynamic lower bound**, not a measured CMOS joule. Practical chips remain far above the bound — the gap is engineering/materials, not ignorance of the principle. | VERIFIED |
| B2 | IEEE International Roadmap for Devices and Systems (IRDS) — Beyond CMOS | 2023 / 2024 | Official roadmap PDFs document materials integration, variability, memory selectors, heterogeneous integration, and reliability barriers for post-CMOS devices — embodiment roadmap, not “someone forgot Shannon.” | VERIFIED ([2023 IRDS Beyond CMOS PDF](https://irds.ieee.org/images/files/pdf/2023/2023IRDS_BC.pdf); [2024](https://irds.ieee.org/images/files/pdf/2024/2024IRDS_BC.pdf)) |
| B3 | Green AI / Strubell et al. (R10–R11) | 2019–2020 | Software/ML community still under-reports energy; culture of Red AI scale shows **incentive** lag even where awareness exists. | VERIFIED |
| B4 | MWM Posner/Lei/Schölkopf (R7) | 2026 | Argues discovery needs mechanism-centric *organization* — a design/embodiment problem for AI systems, not a missing slogan that prediction ≠ understanding. | VERIFIED |

### 4.2 Argument to use in the paper (no invented measurements)

1. **Academia already named the floors.** Shannon (capacity), Landauer (erase heat), Kolmogorov/Solomonoff/Chaitin (compression), Jaynes (maxent), Howard (VoI), SINDy/AI Feynman/PySR/AI Hilbert (formula floors), AlphaGeometry/AlphaProof (certify floors), MWMs (mechanism vs MoE).  
2. **Scale-first MoE/token escalation is a divergence** from those floors: more generative capacity without external limits. MoL names limits **outside** the generative corridor.  
3. **The remaining hard problem** is applied-math × materials: compiling Lookup→Formula→Solver→Model-LAST onto hardware with grammar coverage, VoI gating, and thermodynamic honesty (Landauer as **bound**, never as fake RAPL). IRDS Beyond CMOS documents that device/materials integration — not concept awareness — dominates the path to lower-energy compute.  
4. **Do not cite:** speculative “MoL chip” papers that do not exist; invented joule tables; ChemRxiv graphene agent until a real arXiv/DOI appears.

### 4.3 What *not* to claim

- That current MoL software measures physical board joules.  
- That Landauer bound ≈ observed GPU energy.  
- That materials science “doesn’t know” about information theory — the lag is **co-design and embodiment**, not literacy.

---

## 5. Strongest 8 proof points (paper banner)

Use these eight as the banner set; full tables above for deep cites.

1. **Shannon 1948** — channel capacity as refuse-excess-bits floor.  
2. **Landauer 1961** — irreversible erase has a thermodynamic energy floor.  
3. **Solomonoff 1964 / Kolmogorov 1965** — shortest description as inductive/compression floor.  
4. **Howard 1966 VoI** — buy information only when decisions improve.  
5. **Brunton et al. SINDy 2016** — sparse governing equations beat dense residual models.  
6. **Udrescu & Tegmark AI Feynman 2020** — physics-inspired compression to closed-form laws.  
7. **Posner, Lei, Schölkopf MWMs 2026 (arXiv:2607.12474)** — mechanisms + parsimony; MoE is the wrong “mixture.”  
8. **AlphaGeometry 2024 / AlphaProof 2025** — propose then **certify**; unverified tokens fail.

*(Honorable banner alternates: Jaynes maxent 1957; AI Hilbert 2024 proof-gated discovery; Green AI / Strubell energy accounting; RouteLLM cost routing as partial cousin.)*

---

## 6. Sources skimmed on Mac (context only)

| Path | machineId | Use in this pack |
|---|---|---|
| `/Users/dcharlot/Desktop/mol-sync/mixture-of-limits/docs/adjacent-field-hunt.md` | `25022ed7-76ae-44a9-8b17-201eeee0fcdc` (48GB) | Seed inventory; all IDs re-checked before inclusion |
| `…/docs/appendix-mwm-ai-newton.md` | same | MoL≠MoE mapping language for MWM / AI-Newton |
| `…/docs/quantum-thermo-collapse.md` | same | Landauer honesty: bound ≠ measured silicon |
| 128GB Mac research site | `4e1505fa-…` | Not required this pass |

---

## 7. File location

**Deliverable path:** `/workspace/mol-proof-points.md`

*End of verified citation pack.*
