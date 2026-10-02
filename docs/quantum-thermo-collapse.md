# Quantum / thermodynamic computing → MoL “collapse intelligence” gears

**Date:** 2026-09-30 (America/New_York)  
**Parent:** [BLUEPRINT.md](../BLUEPRINT.md) §4 · hunt inventory: [adjacent-field-hunt.md](./adjacent-field-hunt.md)  
**Scope:** Inspiration map only. No RAPL/NVML claims; no assertion that MoL v0.1 runs quantum or adiabatic hardware. Landauer annotations remain **thermodynamic lower-bound estimates ≠ measured silicon joules**.

---

## Thesis (keep)

**Formula always wins** when the grammar is covered. Quantum information science and thermodynamic / reversible computing do not replace that rule — they **show new ways to compute that collapse intelligence into physics**. MoL navigates those ways as **cascade gears** and **named floors**; the neural residual stays **LAST**.

Collapse here means: the answer is obtained by **settling, certifying, or unitarily transforming** a structured state — not by spending residual tokens in a generative corridor.

---

## What QI + thermo contribute (capsule)

| Idea | Physics / QI claim (textbook) | MoL reading |
|---|---|---|
| **Unitary / reversible structure** | Closed evolution preserves information; logical reversibility avoids forced erase | Prefer gears that **transform** known structure (Lookup identity, Formula rewrite) over irreversible discard-and-regenerate |
| **Landauer erase cost** | `E_min = k_B T ln 2` per bit erased (lower bound) | Already first-class: `energy` floor + `landauer_floor_J` on receipts — **estimate, not RAPL** |
| **Energy-landscape settle = answer** | Ground / metastable state of an energy function encodes the solution | Solver-tier affinity: Ising / QUBO / adiabatic anneal **inspire** “settle then certify,” not a shipped annealer |
| **Adiabatic / Ising** | Slow Hamiltonian schedule → ground state; spin glass ↔ combinatorial opt | Maps to **Solver** (constrained settle) when a problem is cast as an energy function — still Deterministic/Composed when certified |
| **LUT certify** | Precomputed allow / sense tables; O(1) check before irreversible act | **Lookup** + WCA propose→certify→commit\|refuse; coin-cell power path |

None of these authorize inventing measured board joules, claiming quantum speedup on MoL paths, or treating stochastic Model output as a physical ground state.

---

## Cascade map (Lookup → Formula → Solver → Model LAST)

```text
Lookup   — LUT / unit dictionary / allow-table certify (reversible read; L0 thermo class)
Formula  — closed-form / identity rewrite (unitary-like structure on paper; L0)
Solver   — sparse / exact / constrained settle (Ising–QUBO / adiabatic *inspiration*; L1)
Model    — stochastic residual LAST (L2max); never the substrate; never bypasses commit
```

Cross-walk to existing MoL types (`mol-core`):

| Collapse gear | `CascadeTier` | `ReplayClass` (typical) | `ThermoClass` | Floor / receipt hook |
|---|---|---|---|---|
| LUT certify / reversible read | Lookup | Deterministic | L0 | `grammar`, WCA sense |
| Identity / closed-form rewrite | Formula | Deterministic | L0 | `grammar`; Landauer bits ≈ 0 if no erase |
| Energy-landscape settle + certify | Solver | Composed | L1 | `energy` / VoI; certify before commit |
| Generative residual | Model | ModelGenerated | L2Max | `voi`, `allow_model=false` default |

**Invariant preserved:** `ModelGenerated` cannot coerce to `Deterministic`. An anneal-inspired settle that is **certified** may land as Composed/Deterministic; a sampled proposal that is not certified stays ModelGenerated.

---

## Periodic Stack empty cells

MoL navigates the Periodic Stack (compute.openie.dev: **258 primitives / 33 families**). QI / thermo **inspire** primitives that may still be **empty cells**:

| Empty-cell theme (roadmap) | Why it matters | Honest v0.1 status |
|---|---|---|
| Reversible / adiabatic rewrite primitive | Makes “no erase ⇒ near-Landauer path” explicit in the stack | Not probed; do not hallucinate coverage |
| Energy-function / Ising bind | Casts Solver settle as a named stack leaf | No live probe; Solver demos stay classical sparse algebra |
| Physical-settle certify | “Answer = settled state + certificate,” not token likelihood | Reserved as **inspiration id** `physical_settle` in BLUEPRINT — **not** wired in `default_registry` (would Pass-only like `primitive_gap` if added later) |
| Quantum channel / unitary leaf | QI as structure, not mysticism | Out of scope for v0.1 silicon path |

When the stack lacks a primitive for the task coordinate, fire **`primitive_gap`** (phase 3) — do not invent coverage from QI rhetoric.

---

## How MoL navigates these gears

1. **Classify grammar** — if Lookup/Formula cover, close without Solver or Model.  
2. **Cheapest-sufficient escalate** — Solver only when settle/certify is needed; Model only if `allow_model`.  
3. **Landauer honesty** — annotate erase lower bound; never equate to RAPL/NVML/`measured_j`.  
4. **Refuse over escalate** — VoI / safety / WCA refuse when landscape settle or generation would be unsafe or VoI-negative.  
5. **Coin-cell** — O(1) Formula + LUT refuse; Model leaf cold (`Budget::coin_cell()`).

NN residual remains **last**, not a rename of adiabatic or quantum compute.

---

## Borrow vs refuse (crate implications)

| Borrow (docs / future trait shape) | Refuse |
|---|---|
| Landauer floor math already in `mol-core::landauer` | Fake RAPL/NVML or `board_synth_claimed=true` |
| ThermoClass L0–L2max as **order-of-magnitude mismatch labels** | Claiming MoL path energy = annealer wall joules |
| Solver-tier “settle then certify” vocabulary in BLUEPRINT / receipts narrative | Shipping quantum SDK deps or annealer drivers in v0.1 |
| Reserved limit name `physical_settle` (Pass stub if ever added) | Binding `physical_settle` without a real settle probe |
| WCA LUT certify as reversible-read / allow-table cousin | Treating Model samples as ground-state certificates |

No sibling-tree edits (`openie-leapfrog`, `jouledb`, `wca-lut-edge`). Adapters stay stubs.

---

## One-line MoL citation

> Quantum and thermodynamic computing show that **intelligence can collapse into physics** (unitary structure, Landauer erase cost, energy-landscape settle, LUT certify). MoL’s job is to **navigate those gears** under named floors — formula first, model last — without confusing lower bounds for measured silicon joules.
