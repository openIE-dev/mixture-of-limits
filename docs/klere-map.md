# Klere (klere.ai) → Mixture of Limits / Physical AI BMI map

**Date:** 2026-09-30 (America/New_York)  
**Parent:** [BLUEPRINT.md](../BLUEPRINT.md) §13 · sibling maps: [physicalai-bmi-map.md](./physicalai-bmi-map.md) · [quantum-thermo-collapse.md](./quantum-thermo-collapse.md) · [adjacent-field-hunt.md](./adjacent-field-hunt.md)  
**Sources fetched:** [klere.ai](https://klere.ai/) · [/overview](https://klere.ai/overview) · [/epu](https://klere.ai/epu) · [/compute](https://klere.ai/compute) · [/energy](https://klere.ai/energy) · [/contact](https://klere.ai/contact) · sitemap (those six URLs only) · `klere-vm` 0.1.2 tarball README (`https://klere.ai/klere-vm-0.1.2.tgz`)  
**Scope:** Read-only adjacent-field map. No crate API change. **Not** the UK carbon consultancy [klere.uk](https://www.klere.uk/) (different org). No claim that MoL v0.1 ships an EPU, FPGA fabric, or klere-vm dependency.

---

## One-paragraph MoL read

Klere, Inc. (Sarasota, FL; schema.org foundingDate 2026) sells a **collapse-intelligence hardware thesis**: an **Energy Processing Unit (EPU)** whose computation *is* settling an energy landscape until silence, with **refuse-when-it-will-not-settle**, on-device ternary {-1,0,+1} memory-colocated state, and every job returning a **receipt** (METER × named rate = joules, plus a REPLAY hash). That is MoL’s reserved `physical_settle` / Solver gear with Model LAST demoted, plus honesty §8 in prose: measured rates on one xck26 FPGA lowering are never averaged with modeled ASIC targets or with laptop-CPU comparisons, and the public `klere-vm` WASM twin **prices** joules at anchored rates rather than metering the host. Overlap with Physical AI BMI’s Energy Lab row (“FPGA settling + accounting / ternary settling fabric / settle ×2”) and thermo/neuromorphic pages is **conceptual and substrate-class**, not a cited cross-link on either site. Klere does **not** name MathGround, OpenIE, Mixture of Limits, JouleDB, or EFA; formula-first closed-form / Lookup→Formula is weak here — the shipped demo is Hopfield-style associative recall and Gibbs annealing, not symbolic law discovery.

---

## Marketing vs technical (label hard)

| Label | Claim / artifact | Source |
|---|---|---|
| **Marketing** | “Accelerating Access”; AI value must not stay “in a few hands”; “Intelligent Era of AI Deployment”; USB-stick future; µW→10s kW power *envelopes* | Home, contact, energy (envelopes explicitly “targets we are building toward, not measurements”) |
| **Marketing / roadmap** | Custom chip and “stick” still ahead; fabric today = programmable chip in lab | Overview honesty block |
| **Technical (site)** | Settling *is* the thinking; stillness = answer; non-settle → report, don’t invent | /epu, /compute |
| **Technical (site)** | Ternary cell; restoring (re-digitized every step); anneals; event-driven; learn by settle × 2, no backprop; “substance of neuromorphic, without the dogma” | /compute |
| **Technical (measured, their label)** | Four lowerings, same xck26 fabric: **0.1596 ± 0.0006**, **0.212 ± 0.039**, **0.2326 ± 0.0075**, **0.377 ± 0.021** pJ per *accumulate* (program unit, not silicon atom); ~**0.19 pJ/LUT** law on three hand-emitted designs (pre-registered vs routing; routing law failed); receipt names the lowering | /energy |
| **Technical (comparison honesty)** | Same job on laptop CPU “a few thousand times more” per step — **one significant figure** on purpose (different chips/meters) | /energy |
| **Technical (package)** | `klere-vm` 0.1.2 CC0 WASM: `settle_recall` (T=0 EPU), `settle_recall_gibbs` (e-QPU annealed Gibbs), `settle_raw`; METER + `joules_nj_measured` (priced) + `joules_nj_modeled` + REPLAY hash; 0.1596 pJ stand-in from xck26/Kria KV260; +0.247 pJ sampler tax for T>0 generic lowering; 0.030 pJ modeled ASIC (Horowitz); not on npm registry | tarball README |
| **Not found on site** | Peer-reviewed paper links, full RTL, datasheet, OpenIE/MathGround citations, neural foundation-model benchmarks | Sitemap + pages |

---

## Thesis / product / technical claims

### Thesis

Access to AI is gated by energy and cloud reach. Intelligence should **live in the device** — resident, private, rewritable like a USB file — and scale by tiling one cell from sensor to rack. The processor is named for what it **spends** (energy), not what it processes (CPU/GPU/TPU contrast).

### Product surface

1. **EPU narrative** — settling fabric; model-as-file; on-board knowledge; on-device learning “in billionths of a joule”; answer + receipt.  
2. **Measured FPGA path** — programmable fabric (xck26 / Kria KV260 class); no custom ASIC yet.  
3. **Public twin** — `npm install https://klere.ai/klere-vm-0.1.2.tgz` / `npx klere recall`; Node, browser WASM, optional native Rust binary (not crates.io).  
4. **Contact** — Klere, Inc.; hello@klere.ai; Sarasota, Florida; builders / partners / investors / engineers.

### Compute primitives (their words)

```text
Ternary (−1 · 0 · +1) · Restoring · Thermodynamic annealing · Event-driven · Learning = settle × 2 (no backprop)
```

As the state falls into the attractor, activity decays; **power draw becomes a convergence meter**; answer is silence. Memory and compute colocated; only the in-progress answer moves, “three symbols wide.”

---

## Relation to MoL / OpenIE / formula-first / thermo / neuromorphic / Physical AI BMI

| Axis | Relation | Strength |
|---|---|---|
| **MoL collapse / Solver** | Settle-to-answer = energy-landscape collapse; non-settle refuse ≈ `wca_refuse` spirit; Model LAST (no generative substrate claimed). Reserved inspiration id `physical_settle`. | **Strong** (conceptual) |
| **MoL honesty / receipts** | Named lowering, measured vs modeled vs priced; REPLAY hash; “joules it prints are priced at the measured rate, not measured on your own machine.” Twin of §8 / `estimated_j` vs `measured_j` / `board_synth_claimed=false`. | **Strong** (provenance discipline) |
| **MoL energy floor** | pJ/accumulate and METER are their accounting unit; do **not** paste into MoL receipts as `measured_j`. Coin-cell rhyme: don’t fetch — knowledge stays on board. | Medium (unit of account cousin) |
| **Formula-first / MathGround** | Demo is associative recall / Gibbs, not Lookup→Formula law discovery. Ternary alphabet overlaps BitNet-style compressions elsewhere, not closed-form identities. | **Weak** |
| **OpenIE / Periodic Stack / JouleDB** | No citations found. Parallel vocabulary only (receipts, joules, refuse). | None (no shared types) |
| **Thermo** | Annealing + e-QPU Gibbs sampler; T>0 sampler tax measured separately; stillness as answer. Cousin of BMI `/thermo` sampling-as-substrate, different stack (ternary fabric vs ferrotherm/p-bit). | Medium–strong |
| **Neuromorphic** | Explicit: event-driven, memory-colocated, self-teaching, “without the dogma.” Activity decay ↔ spikes-per-synapse *intuition*, not the same bound. | Medium (self-positioned) |
| **Physical AI BMI** | BMI Energy Lab substrate map already lists **“FPGA settling + accounting / ternary settling fabric / settle ×2”** with EFA-on-Ferric as fabric target. Klere is a **commercial/product face** of that substrate class (same ternary settle language, settle×2 learning, receipt invariant). Sites do not cross-link each other in this fetch. Charlot Lab / JBI appears on BMI; Klere legal entity is Klere, Inc. Sarasota — **institutional identity not asserted here beyond public pages**. | Strong (substrate-class adjacency) |

---

## Borrow vs refuse

| Borrow (docs only) | Refuse |
|---|---|
| Provenance: rate belongs to a named lowering; priced ≠ measured on host | Importing 0.1596 pJ/acc into MoL as `measured_j` |
| Settle refuse when it will not converge | Treating Hopfield recall as MathGround Formula coverage |
| Receipt = meter × rate + replay hash as field pattern | Shipping klere-vm / FPGA SDK as a MoL dependency in v0.1 |
| Label marketing envelopes (µW–kW) separately from silicon rates | Averaging FPGA, ASIC model, and laptop “few thousand×” into one headline |
| EqProp-like settle×2 local write as learns-at-inference signal | Claiming peer-reviewed verification grade the site does not claim |

---

## Pages inventory

| URL | Role |
|---|---|
| https://klere.ai/ | Access / marketing hero |
| https://klere.ai/overview | Product thesis + honesty (“measured or building toward”) |
| https://klere.ai/epu | What an EPU is |
| https://klere.ai/compute | Ternary / restoring / anneal / settle×2 |
| https://klere.ai/energy | Measured pJ/acc + envelopes + klere-vm install |
| https://klere.ai/contact | Org (Klere, Inc.; Sarasota, FL) |
| https://klere.ai/klere-vm-0.1.2.tgz | Public WASM twin (not sitemap HTML) |

No `/about` or `/docs` (404). Sitemap lists only the six HTML URLs above.

---

## One-line citation

> Klere’s EPU is a ternary settle-to-silence fabric with measured-on-FPGA accumulate receipts and a priced WASM twin; MoL reads it as adjacent collapse/Solver + honesty provenance, not as OpenIE citation or Formula-first coverage, and does not treat their pJ figures as MoL measurements.
