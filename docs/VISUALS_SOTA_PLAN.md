# SOTA visuals production plan — research.openie.dev

**Owner:** David Charlot / OpenIE  
**Date:** Thu Oct 1, 2026 (America/New_York, EDT)  
**Status:** Concrete ship plan — MoL first (live), then NI, then Satiation  
**Site root:** `/Users/dcharlot/data-share/vibe-coding/research-openie-web`  
**Living root:** `public/living/`  
**Scene source:** `/workspace/mol-manim/` (box) + `manim/mol/` (site mirror)

---

## 0. Honesty rules (non-negotiable)

| Rule | Enforcement in every visual |
|------|-----------------------------|
| No fake `measured_j` | Receipt panels show `measured_j=None` unless Metered probe returned a number |
| No fake board watts / RAPL / NVML meters | Soft-ref / living stubs never draw wattmeters as measured |
| Landauer = **labeled estimate** | Always caption: “thermodynamic lower-bound **estimate**” |
| `board_synth_claimed=false` | Chip or footer on any energy-adjacent figure |
| Analytical / catalog / OpCounter ≠ board power | `[SURROGATE]` / Estimated / Unmetered chips from living honesty legend |
| Soft-ref `mol prove` ~29 VERIFIED | Software constructive existence only — not FPGA package energy |

Reuse `public/living/js/honesty-legend.js` chips. Prefer Manim end-cards that literally print the honesty line.

---

## 1. Tooling

| Layer | Tool | Role |
|-------|------|------|
| Primary motion | **Manim Community** ≥0.19 (Mac Homebrew `manim`/`manimce` **v0.21.0**) | Narrated explainer MP4/GIF/WebM for lineage, cascade, floors, close, MoL≠MoE |
| Interactive living | SVG + CSS / Observable / D3 (optional) | Hover cite chips, cascade toggle, compose predicate — keep figure IDs stable |
| Existing 3D | Canvas orbit (`ni-3d-01`, `sat-3d-01`) | Keep; do not replace with Manim |
| Encode | ffmpeg (box + Mac) | mp4 → gif/webm; `-movflags +faststart` for web |
| Box install note | **Blocked** Oct 1 2026: no sudo apt; pycairo needs `pkg-config` + cairo **dev** headers. Runtime libcairo2 present but pip manim fails. **Render on Mac**; sync sources to `/workspace/mol-manim/`. |

### Render pipeline

```
scenes/<name>.py  →  manim -qm --format=mp4,gif
                 →  media/videos/.../<Scene>.mp4|.gif
                 →  public/living/<study>/media/<fig-id>-<slug>.{mp4,gif,webm}
                 →  <video|img> inside living section id="<fig-id>"
```

Suggested flags for web shorts:

```bash
manim -qm --fps 30 --format=mp4 scenes/cascade.py CascadeLast
ffmpeg -y -i CascadeLast.mp4 -vf "fps=12,scale=960:-1:flags=lanczos" -loop 0 CascadeLast.gif
ffmpeg -y -i CascadeLast.mp4 -c:v libvpx-vp9 -b:v 0 -crf 35 -an CascadeLast.webm
```

---

## 2. Per-study figure map (living IDs already on site)

### 2.1 MoL — **P0 / ship first** (`public/living/mol/index.html`)

| Living ID | Manim / visual | Scene class (suggested) | Media path | Notes |
|-----------|----------------|-------------------------|------------|-------|
| `mol-inf-01` | Lineage: Brahe→Kepler→Newton→Shannon→Landauer→MoL floors | `LineageCompress` | `mol/media/mol-inf-01-lineage.{mp4,gif}` | Landauer labeled **estimate** |
| `mol-diag-01` | Cascade Lookup→Formula→Solver→**Model LAST** | `CascadeLast` | `mol/media/mol-diag-01-cascade.{mp4,gif}` | Grammar covered ⇒ do not open model |
| `mol-diag-02` | Named floors + **E ≥ θ·μ** with Landauer estimate label | `EnergyFloorEstimate` | `mol/media/mol-diag-02-floors.{mp4,gif}` | VoI / grammar / energy / efa_certificate / settle_refuse / primitive_gap |
| `mol-diag-03` | Close: propose→certify→commit\|refuse→receipt | `CloseReceipt` | `mol/media/mol-diag-03-close.{mp4,gif}` | `measured_j` only if Metered; refuse = lawful success |
| `mol-tab-01` | MoL ≠ MoE contrast | `MolNotMoe` | `mol/media/mol-tab-01-not-moe.{mp4,gif}` | Side-by-side axes from living table |
| `mol-tab-02` | Prove ↔ claim honesty | Optional later / SVG | — | Table is already honest; motion optional |
| `mol-note-01` | Energy honesty legend | Prefer shared `honesty-legend.js` | — | Do not animate fake meters |

**First 5 MoL scenes (priority order):**

1. **Lineage** (`mol-inf-01`) — Kepler→Newton→Shannon→Landauer  
2. **Cascade** (`mol-diag-01`) — Lookup→Formula→Solver→Model LAST  
3. **E≥θ·μ** (`mol-diag-02`) — Landauer estimate label; `board_synth_claimed=false`  
4. **Close** (`mol-diag-03`) — propose→certify→commit|refuse→receipt  
5. **MoL≠MoE** (`mol-tab-01`) — mixture outside generation vs inside model  

### 2.2 NI — queue after MoL motion ships (`public/living/ni/index.html`)

| Living ID | Visual upgrade | Manim / other | Honesty |
|-----------|----------------|---------------|---------|
| `ni-inf-01` | Motion twin of Brahe→Kepler notational lineage | Manim timeline + cite chips | illustrative |
| `ni-diag-01` | System One → WCA complement | Manim layers or keep SVG | illustrative |
| `ni-diag-02` | Commit stack / compose predicates | Manim + interactive toggle | illustrative |
| `ni-fig-01` | Annotated `wca.commit.v1` | Keep interactive JSON; optional Manim flythrough | measured schema / illustrative UI |
| `ni-run-01`…`03` | Keep code-runners | No Manim replacement | seed-1 J stays **[SURROGATE]** |
| `ni-inf-02` | Joule tiers A/B/C | Manim ladder + locked false board flag | `board_synth_claimed=false` |
| `ni-3d-01` | Keep canvas orbit | — | modelled (toy) |
| `shared-*` | Shared honesty / complement | Prefer one Manim + two embeddings | same IDs |

### 2.3 Satiation — queue after NI (`public/living/satiation/index.html`)

| Living ID | Visual upgrade | Manim / other | Honesty |
|-----------|----------------|---------------|---------|
| `sat-inf-01` | Engagement vs economic-done lenses | Manim split-lens | illustrative |
| `sat-fig-01` | Bliss-point geometry | Manim + existing `sat-3d-01` | modelled (textbook) |
| `sat-tab-01` | Race→satiation→commodity | Keep table; optional motion | **[SOFT]** historical |
| `sat-fig-02` | Epoch free-at-margin arc | Cite Epoch; **≠ free joules** badge | no invented series |
| `sat-inf-02` | Scarcity map | Manim / SVG tiles | illustrative |
| `sat-diag-01` | Refuse = done ∪ unsafe | Manim Venn | design claim |
| `sat-tab-02` / `sat-fig-04` | Rules / steelman | Keep tables | design / illustrative |
| `sat-3d-01` | Keep canvas bliss surface | — | modelled (textbook) |

---

## 3. Production schedule (concrete)

| Wave | Deliverable | Exit criteria |
|------|-------------|---------------|
| **W0 (this pass)** | Plan + 1–2 Manim scenes + ≥1 rendered asset wired into `living/mol/` | Cascade or lineage visible on `/living/mol/`; honesty caption present |
| **W1** | Scenes 3–5 (floors, close, MoL≠MoE) + WebM | All five MoL P0 motion IDs have media |
| **W2** | Light SVG/D3 interactivity for cascade + close (optional) | Same IDs; Manim as poster/autoplay fallback |
| **W3** | NI Manim: `ni-inf-01`, `ni-diag-02`, `ni-inf-02` | No fake board watts; shared legend intact |
| **W4** | Satiation Manim: `sat-inf-01`, `sat-diag-01`, `sat-fig-01` motion twin | Epoch cites only; bliss labelled textbook |

---

## 4. Living page wire pattern

Inside each MoL section, prefer:

```html
<figure class="living-media" data-fig="mol-diag-01">
  <video autoplay muted loop playsinline poster="media/mol-diag-01-cascade.gif"
         aria-label="Cascade Lookup to Model LAST">
    <source src="media/mol-diag-01-cascade.webm" type="video/webm"/>
    <source src="media/mol-diag-01-cascade.mp4" type="video/mp4"/>
  </video>
  <figcaption class="caption">
    mol-diag-01 · illustrative · Grammar covered ⇒ do not open Model.
    Energy: Unmetered / Estimated only · board_synth_claimed=false
  </figcaption>
</figure>
```

CSS: reuse `.figure-block` / `.figure-shell`; add `.living-media video { width:100%; max-width:960px; border-radius:8px; }` in `living.css` if missing.

---

## 5. Desktop mol-sync

Docs mirror target (if reachable):  
`/Users/dcharlot/Desktop/mol-sync/mixture-of-limits/docs/VISUALS_SOTA_PLAN.md`  
(kernel tree under Desktop `mol-sync/`; copy plan there for offline sync.)

Site copy: `content-src/VISUALS_SOTA_PLAN.md`

---

## 6. Next queue (NI → Satiation)

1. After MoL five scenes ship: Manim `ni-inf-01` + `ni-diag-02` (commit stack) using same style kit (dark navy, monospace IDs, honesty footer).  
2. Then `sat-diag-01` refuse Venn + `sat-inf-01` lens toggle motion.  
3. Do **not** Manim-replace `ni-run-*` or invent Epoch curve points.

---

## 7. Checklist before commit/push of NEW media

- [ ] Real mp4/gif rendered (not placeholder)  
- [ ] Wired under correct living `id=`  
- [ ] Honesty caption / chips present  
- [ ] `pnpm build` (or site build script) still passes  
- [ ] No invented joules / board watts in frames or captions  
