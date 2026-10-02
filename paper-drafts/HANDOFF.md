# MoL drafts handoff — §§6 & 9

**When:** Thu Oct 1, 2026 ~10:42 AM EDT (America/New_York)  
**Author lane:** paper-section drafts for research.openie.dev MoL study  
**Sources preferred:** `/Users/dcharlot/Desktop/mol-sync/mixture-of-limits/` (PLAN, BLUEPRINT, README, docs/adjacent-field-hunt.md)  
**Prototype path:** `mixture-of-limits-fabric-detect` — **absent** on Mac  
**Tone match:** OpenIE NI / Satiation papers (`claim.*` tables, honesty fence, no board watts)

## Deliverables (box)

| File | Path |
|---|---|
| §6 prove↔claim | `/workspace/mol-drafts/section-6-prove-claim.md` |
| §9 MoL≠MoE + SOTA | `/workspace/mol-drafts/section-9-mol-vs-moe-sota.md` |
| This handoff | `/workspace/mol-drafts/HANDOFF.md` |

Do **not** treat these as live site edits. MoL should merge into paper §§1–5,7–8,10.

## Living scaffold status

| Path | Status |
|---|---|
| `research-openie-web/public/living/mol/` | **Exists as empty directory** (mtime Oct 1, 2026 ~10:40 AM EDT); no `index.html`, no figure stubs |
| `dist/living/mol` | Absent |
| Paper content `src/content/papers/mol.md` | Absent (only `ni.md`, `satiation.md`) |

**Decision:** scaffold not ready — living honesty stubs **deferred**. No site files written under `research-openie-web`.

When MoL adds empty figure stubs, honesty discipline to match NI/Satiation:

- Labels: **Metered | Estimated | Unmetered** only when honest  
- Chips: no fake board W; `board_synth_claimed=false` until meter + synth log  
- Captions must say software-ref / soft-ref vs live probe  
- PDF twin IDs if/when living figures ship  

## DOI / arXiv verification (§9 cite rows)

| Cite | Title verified? | ID | Status |
|---|---|---|---|
| RouteLLM | Yes — *Learning to Route LLMs with Preference Data* (arXiv); ICLR 2025 “from Preference Data” | arXiv **2406.18665** | **Verified** (WebFetch abs + ICLR) |
| PEARL (routing) | Yes — *Performance and energy aware routing for LLMs* (Chadli / Botterweck / Saber) | DOI **10.1016/j.future.2025.108218** | **Verified** (ScienceDirect / researchr / Lero); DOI page fetch timed out — DOI string cross-checked via secondary indexes |
| PEARL (speculative) | Name collision noted — *Parallel Speculative Decoding…* | arXiv **2408.11850** | **Verified** as distinct paper; **not** the §9 PEARL row |
| GreenServ | Yes — *Energy-Efficient Context-Aware Dynamic Routing for Multi-Model LLM Inference* | arXiv **2601.17551** | **Verified** (WebFetch abs) |
| VoltanaLLM | Yes — *Energy-Efficient and SLO-Aware Disaggregated LLM Serving via Adaptive Frequency Control and State-Space Routing* | arXiv **2509.04827** | **Verified** (WebFetch abs) |
| HCSpec | Yes — *Two-Tier Horizontal Cascade Speculative Decoding…* | DOI **10.18653/v1/2026.acl-long.353** · ACL 2026 | **Verified** (ACL Anthology fetch) |
| CAS-Spec | Yes — *Cascade Adaptive Self-Speculative Decoding…* | arXiv **2510.26843** | **Verified** (WebFetch abs); NeurIPS 2025 proceedings also listed in search |
| GCD | Yes — *Grammar-Constrained Decoding for Structured NLP Tasks without Finetuning* (Geng et al.) | arXiv **2305.13971** · DOI **10.18653/v1/2023.emnlp-main.674** | **Verified** (ACL Anthology / arXiv search) |
| CRANE | Yes — *Reasoning with constrained LLM generation* (Banerjee et al.) | arXiv **2502.09061** · ICML 2025 PMLR v267 | **Verified** (WebFetch abs + PMLR page); ACM DOI 10.5555/3780338.3780441 also listed |

**Human follow-ups (optional):**

1. Confirm PEARL FGCS page title string on publisher HTML (fetch timeout); DOI is consistent across ScienceDirect / researchr / Lero.  
2. Prefer arXiv vs ICLR title wording for RouteLLM in bibliography (minor “with” vs “from”).  
3. If MoL wants speculative PEARL in the table too, add as separate row — do not merge with FGCS PEARL.

## Constraints honored

- No invented `measured_j` / board joules  
- `board_synth_claimed=false` stated  
- Claims mapped only from PLAN P1–P24 (+ soft-ref §6 residuals)  
- No git push; no `research-openie-web` content edits  

## David spine (post-draft fold-in)

- MoL = information-theoretic / physics lineage (Newton–Kepler formula/law discovery), not excess generation
- Frame as converging to VoI/floors like physics → predictive laws; **not** SOTA horse race
- Bottleneck = applied math × materials (hardware); academia already has Shannon→Landauer→VoI; MoL embodies floors, not a new slogan
- Folded into §6 opening + §9.0 before handoff
