## 9. MoL ≠ MoE, and the model↔model SOTA table

### 9.0 Spine (history + mathematics, not a horse race)

MoL is proven as an **information-theoretic** approach rooted in **physics**: formula/law discovery in the Newton–Kepler lineage, not excess generation. Academia already has Shannon → Landauer → complexity/VoI → symbolic and energy floors; MoL **embodies** those floors in a commit|refuse cascade. The remaining bottleneck is applied math × materials (hardware), not an awareness gap. The table below situates field systems so readers see what they optimize — **which neural generator runs** — without turning MoL into another SOTA horse race. MoL asks whether a generator should run at all, converging to VoI/floors the way physics converged to predictive laws.

### 9.1 Law contrast (not a rebrand)

Mixture of Experts (MoE) places experts **inside** a generative corridor: a gate selects parameters; success is next-token likelihood; capacity grows by adding experts to the same act. Mixture of Limits (MoL) places named **floors outside** generation: Lookup → Formula → Solver/settle → **Model LAST**; success is closed grammar + certified commit|refuse + joule receipt; the model is a demoted residual leaf, cold by default (`allow_model=false`).

| Axis | MoE / model↔model field | MoL |
|---|---|---|
| Where the mixture lives | Inside the model (experts, draft/target pairs, strong/weak LLMs) | Outside generation (named floors + cascade gears) |
| Routing object | Which **model** (or draft head) answers | Which **tier** closes: Lookup / Formula / Solver / Model |
| Refuse | Rare; usually escalate to a larger model | First-class: VoI, grammar, settle, certificate, primitive_gap, fabric_unavailable, … |
| Energy honesty | Often latency/cost proxies; some papers meter GPU energy for **LLM** serving | Receipts: `estimated_j` labeled; soft-ref `measured_j=None`; `board_synth_claimed=false`; never invent RAPL |
| Success metric | Tokens/s, cost@quality, speculative speedup | Certified commit + typed ReplayClass + receipt |

MoL does not claim to beat RouteLLM on MT-Bench cost, GreenServ on Wh, or HCSpec on decode speedup. Those systems optimize **which neural generator runs**. MoL asks whether a generator should run at all.

### 9.2 Field SOTA (all model↔model routing / speculation / constrained generation)

Every row below is **model↔model**: routers choose among LLMs; speculative methods draft-then-verify with draft/target models; constrained decoding still samples from an LLM under a grammar. None implements Lookup→Formula→Solver→Model LAST with refuse-to-commit as the close law.

Measurement labels: **Metered** | **Estimated** | **Unmetered** — only when that paper’s own method section supports the label. MoL does **not** re-report their joules as MoL `measured_j`. MoL prove path: **Unmetered** (software-ref); `board_synth_claimed=false`; no invented board joules.

| System | Title (verified) | ID | Class | What it routes / accelerates | Energy / cost as reported by authors | Vs MoL |
|---|---|---|---|---|---|---|
| **RouteLLM** | *RouteLLM: Learning to Route LLMs with Preference Data* (arXiv); ICLR 2025 venue title uses *from Preference Data* | arXiv:[2406.18665](https://arxiv.org/abs/2406.18665) · ICLR 2025 | Preference router (strong↔weak LLM) | Query → strong or weak LLM | Cost / quality (API $); not MoL Landauer receipts | Model↔model; no Lookup/Formula floor; no typed refuse-to-commit |
| **PEARL** (routing) | *PEARL: Performance and energy aware routing for LLMs* | DOI:[10.1016/j.future.2025.108218](https://doi.org/10.1016/j.future.2025.108218) · FGCS 176:108218 | Energy-aware multi-LLM router | Query → LLM under energy cap (EMM predicts energy) | Authors report energy-aware routing (GPU/infra); **not** imported as MoL `measured_j` | Still picks a model; MoL may refuse before any model |
| **GreenServ** | *GreenServ: Energy-Efficient Context-Aware Dynamic Routing for Multi-Model LLM Inference* | arXiv:[2601.17551](https://arxiv.org/abs/2601.17551) · ICPE 2026 | Contextual bandit multi-LLM router | Query features → LLM; accuracy vs **measured GPU energy** (Zeus) | **Metered** (authors’ GPU Wh via Zeus) on their LLM pool — field result, not MoL board synth | Model pool routing; MoL cascade is non-neural first |
| **VoltanaLLM** | *VoltanaLLM: Energy-Efficient and SLO-Aware Disaggregated LLM Serving via Adaptive Frequency Control and State-Space Routing* | arXiv:[2509.04827](https://arxiv.org/abs/2509.04827) | P/D-disagg serving + frequency + state-space route | Prefill/decode instances + GPU frequency under TTFT/ITL SLOs | **Metered** (authors; pyNVML on A100/GH200) up to ~36.3% E2E GPU energy vs max-freq baseline — field serving result | Routes **instances of the same generative stack**; MoL may never open decode |
| **HCSpec** | *HCSpec: Two-Tier Horizontal Cascade Speculative Decoding for High-Efficiency Large Language Model Inference* | DOI:[10.18653/v1/2026.acl-long.353](https://doi.org/10.18653/v1/2026.acl-long.353) · ACL 2026 | Speculative decoding (draft↔target) | Position-specialized draft cascade → target verify | Latency speedup (vs EAGLE-3 / AR); energy not MoL’s claim | Speeds generation; does not refuse generation on floors |
| **CAS-Spec** | *CAS-Spec: Cascade Adaptive Self-Speculative Decoding for On-the-Fly Lossless Inference Acceleration of LLMs* | arXiv:[2510.26843](https://arxiv.org/abs/2510.26843) · NeurIPS 2025 | Self-speculative cascade (DSIA drafts) | Draft stages from target (sparsity/quant) + DyTC | Latency speedup ~1.1×–2.3× AR (authors); lossless tokens | Model-internal draft hierarchy ≠ MoL floors |
| **GCD** | *Grammar-Constrained Decoding for Structured NLP Tasks without Finetuning* | arXiv:[2305.13971](https://arxiv.org/abs/2305.13971) · DOI:[10.18653/v1/2023.emnlp-main.674](https://doi.org/10.18653/v1/2023.emnlp-main.674) · EMNLP 2023 | Grammar-constrained LLM decoding | Mask logits to CFG | Unmetered / quality metrics | Constrains **tokens**; MoL constrains **commit** |
| **CRANE** | *CRANE: Reasoning with constrained LLM generation* | arXiv:[2502.09061](https://arxiv.org/abs/2502.09061) · ICML 2025 (PMLR v267) | Reasoning-augmented constrained decoding | Alternate unconstrained reason ↔ constrained answer | Accuracy on GSM-symbolic / FOLIO; not MoL joules | Still LLM generation under grammar; MoL can close at Formula without an LLM |

**Name collision (do not conflate):** ICLR 2025 also has *PEARL: Parallel Speculative Decoding with Adaptive Draft Length* (arXiv:[2408.11850](https://arxiv.org/abs/2408.11850)) — speculative, not the FGCS energy router. §9’s PEARL row is the **routing** paper (DOI 10.1016/j.future.2025.108218). Speculative PEARL sits in the same model↔model bucket as HCSpec/CAS-Spec if cited later.

### 9.3 What MoL claims instead

From PLAN prove (see §6):

1. **Cascade order is law**, not a heuristic over LLMs: Lookup → Formula → Solver/settle → Model LAST.
2. **Refuse-to-commit** is typed and receipted (`voi`, `settle_refuse`, `efa_certificate`, `primitive_gap`, `fabric_unavailable`, `claim_unknown`, …).
3. **Receipt honesty** on the proven path: `measured_j=None` (Unmetered), estimates labeled (**Estimated**), `board_synth_claimed=false`. No invented measured joules; no board joules.
4. **ReplayClass** cannot strengthen: `ModelGenerated` ↛ `Deterministic`.
5. Soft-ref multi-fabric routing chooses `DeviceKind` after tier close — still not “pick GPT-4 vs Mixtral.”

### 9.4 Honesty fence

| Forbidden in MoL paper results | Allowed |
|---|---|
| Invented `measured_j` | Soft-ref prove with `measured_j=None` |
| `board_synth_claimed=true` without meter + synth log | `board_synth_claimed=false` |
| Importing GreenServ / VoltanaLLM Wh as MoL board energy | Citing them as **field** model↔model SOTA with their own Metered labels |
| Claiming MoL wins tokens/J vs HCSpec | Claiming MoL can refuse before tokens |

Field SOTA above remains **model↔model**. MoL’s contribution is not a better router among generators: it is the navigation law that embodies known information/energy floors, with a prove harness that keeps Model last and refuse first-class.
