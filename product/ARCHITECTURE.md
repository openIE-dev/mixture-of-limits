# ARCHITECTURE.md — Mixture of Limits product

Embodiment stack for the valued product. Companion laws: **Navigation** (Mixture of Limits) · **Commit** (Notational Intelligence) · **Economic Reality of Satiation**.

## Dual-phase → Mixture of Limits → NI commit → satiation

```text
  unstructured in (v1.1+ / optional)
       │
       ▼
  ┌─────────────────────────────────────────┐
  │ Phase 1 · bounded micro-perception      │
  │ TinyML / quantized transducer           │
  │ → typed AST / schema / task coordinate  │
  └──────────────────┬──────────────────────┘
                     │ typed request (MVP may start here)
                     ▼
  ┌─────────────────────────────────────────┐
  │ Phase 2 · Mixture of Limits cascade     │
  │ Lookup → Formula → Solver → Model LAST  │
  │ Floors: VoI · grammar · energy · cert   │
  └──────────────────┬──────────────────────┘
                     │
                     ▼
  ┌─────────────────────────────────────────┐
  │ NI / WCA commit gate                    │
  │ propose → certify → commit | refuse     │
  └──────────────────┬──────────────────────┘
                     │
                     ▼
  ┌─────────────────────────────────────────┐
  │ Satiation stop                          │
  │ if C(z)=1 → refuse further synthesis    │
  │ receipt: estimated_j always;            │
  │          measured_j only if meter       │
  └─────────────────────────────────────────┘
```

### Phase 1 (bounded front gear)

- Emits typed AST only. Does not replace Formula. Does not become the substrate.
- MVP: `phase1.enabled=false` — tickets arrive typed.
- Cost class: ultra-light; never a second Model LAST parser by default.

### Phase 2 (Mixture of Limits cascade)

- **Lookup** — O(1) grammar / LUT hit → commit path, model cold (A1).
- **Formula** — schema-covered identity / formula.
- **Solver / settle** — ternary or policy settle under certificate.
- **Model LAST** — residual only when VoI>0 and `allow_model`; proposals are `ModelGenerated`.

Meta-routing stays **strictly cheaper than the smallest allowed inference leaf** (Bloom/trie/EBNF/VoI tables). A neural forward pass is not a floor check — it is another leaf.

### NI commit

- Irreversible acts require a certificate before the executor runs.
- System One / Laya-class proposers may supply options; they do not own plant motion or economic done.
- `ModelGenerated` never commits without NI cert (A4).

### Satiation

- Written `C(z)` in `mol.yaml`. When `C=1`, further synthesis is refused (A3).
- Distinct reason code from VoI / energy / certificate refuses.

### Measurement tiers

| Tier | Source | `measured_j` |
|---|---|---|
| 0 Analytical | Catalog μ, Landauer estimate, OpCounter | Never from estimates alone |
| 1 OS telemetry | RAPL / IOReport / powermetrics | Only if Metered probe succeeds |
| 2 Shunt / package | Certified meter | Metered package reading |

Soft-ref default: `board_synth_claimed=false`, `measured_j=None`.

## Primitive Distillation Loop — **v2** (not MVP)

When grammar is undefined and a residual is worth keeping:

```text
  Model LAST proposal
         │
         ▼
  certify (typed check / proof / settle)
         │ pass
         ▼
  AST compile → new Lookup row or Formula identity
         │
         ▼
  Periodic Stack Present cell (replay class Deterministic)
```

Uncertified proposals never become Lookup. Distilled entries carry provenance (source receipt, certify method, replay class). MVP refuses with `primitive_gap` / grammar-miss instead of distilling.

## Clean-room mapping

| Product layer | Rust clean-room crate |
|---|---|
| Floors / VoI / Landauer / fabric | `mol-core` |
| Cascade tiers | `mol-cascade` |
| `route` / `close` | `mol-limits` |
| Certify gate / agent loop | `mol-automate` |
| WCA / EFA / Klere stubs | `mol-adapters` |
| Receipts | `mol-receipt` |
| `mol prove` / ask / meter | `mol-cli` |

Reference path: `/Users/dcharlot/Desktop/mol-sync/mixture-of-limits/`  
Declared GH (not published yet): `https://github.com/openIE-dev/mixture-of-limits`

## Non-architecture (explicit)

- MoE inside the generative corridor is not the navigation law.
- Always-call-model is not MVP.
- Inventing package joules is forbidden.
