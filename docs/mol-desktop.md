# MoL Desktop — Energy Harness Shell

**Product:** Rust-native cross-platform **energy harness** for Mixture of Limits — operator surface for the secure energy+compute WASM ecosystem law — not an IDE chat clone or Photon/Nova shell.

**Crate:** `mol-desktop` · **Headless API:** `ShellSession` (default / prove) · **GUI:** optional `gui` feature (egui/eframe)

Author: David Charlot / OpenIE · License: Apache-2.0 OR MIT

---

## Why this exists (dominance, not parity)

MoL Desktop is the operator surface for **computer intelligence as law**: floors, cascade, certify, **commit|refuse**, joule receipts, multi-fabric routing. Chat-first coding agents optimize tokens and autocomplete. MoL optimizes **verified decisions per joule** with full transaction science.

### Dominance axes (must beat VS Code / Cursor / Zed / Grok Bot / Pi / Hermes / Claude Code)

| # | Axis | MoL Desktop | Typical IDE / agent chat |
|---|---|---|---|
| 1 | **Joules-per-verified-decision** | Every act stamps `estimated_j`, Landauer ratio, μ catalog; session **joule ledger** | Tokens / $ / latency — energy is a footnote or absent |
| 2 | **MoL owns close** | `commit\|refuse` with typed `ReplayClass` (Deterministic / RetrievedCited / Composed); model never substrate | Model is the substrate; “accept diff” without certify grammar |
| 3 | **Multi-fabric inventory** | CPU / Metal / Vulkan / WebGPU / ThermoSettle / LUT / neuromorphic — cheapest sufficient | One GPU/cloud default; no fabric routing law |
| 4 | **Full transaction ledger** | Transcript replay, HMAC receipts, bitemporal memory, capability default-deny | Chat history / git — not joule-certified close fingerprints |
| 5 | **Machine protocol first** | `ShellSession` = MCP-class lane; human only on consequential confirm (**interface tax**) | Human-in-the-loop chat is the product; protocol is bolted on |
| 6 | **Energy harness UX** | Floors fired, cascade steps, fabric chosen, measured_j honesty — primary chrome | Explorer / tabs / Copilot panel — IDE features primary |
| 7 | **Rust native, no Electron** | **egui/eframe** (chosen) — Linux + Mac (+ Windows); Tauri rejected for WebView/webkit tax | Electron / webview shells common |

### Competitive matrix (explicit)

| Product | Primary unit | Close / refuse | Energy | Fabric | Protocol | Native |
|---|---|---|---|---|---|---|
| **MoL Desktop** | Verified decision + joules | MoL certify → commit\|refuse | Ledger + Landauer + optional OS meter | Multi-fabric inventory | Shell API first | egui/eframe Rust |
| VS Code | Files / extensions | Human save | None | N/A | LSP/DAP | Electron |
| Cursor | Agent chat + diffs | Accept/reject patch | Tokens/$ | Cloud GPU implicit | Chat + SCP-ish | Electron |
| Zed | Editor speed | Human | None | GPU for UI | LSP | Rust native (editor, not CI law) |
| Grok Bot | Conversational agent | Soft tool confirm | Opaque | Cloud | Tool calling | Service |
| Pi / Hermes | Agent frameworks | Tool policies vary | Rarely joules | Varies | Often MCP | Mixed |
| Claude Code | Repo agent CLI/IDE | Diff accept | Tokens | Cloud | CLI/tools | Mixed |

**Interface tax:** thin human confirm only when capability/mutate/irreversible; machine lane (`ask_close`) is the default fidelity path.

---

## UI surfaces (energy harness)

| Panel | Role |
|---|---|
| **Ask / Close** | Query box → MoL `close` → COMMIT\|REFUSE |
| **EcosystemCertify** | Run e2e certify: detect · tier `formula`\|`model` · `meter_required` · meter-ms → one receipt |
| **Receipt** | zone, fabric, limit, `estimated_j` vs `measured_j`, Landauer ratio, cascade steps, cite/compose ids |
| **Ecosystem stamps** | `energy_honesty` / `measured_j`, `encapsulation`, `agent_lane`, `fabric_id`, `kernel:vector_add` proof, `compute_steps`, commit\|refuse — missing/refuse stamps visible; Measured ≠ Estimated |
| **Fabric inventory** | soft-ref / detect presence (detect ≠ joules) |
| **Joule ledger** | per-act est_j + honesty + enc/agent stamps + cumulative |
| **Transcript replay** | JSONL close fingerprints without model escalate |
| **Confirm** | consequential acts only |

Honesty: `measured_j=None` + `measure_source=unavailable` when VM / no permission / feature off. **Never invent RAPL or powermetrics numbers.** Optional `energy-meter`: Linux powercap (package/core/dram/gpu); macOS IOReport rails (CPU/GPU/ANE/DRAM) + SMC `PSTR` package joules (no sudo); powermetrics `combined_power` still needs sudo. Windows ETW is a later stub.

### `FailClosedPolicy::meter_required`

When the OS meter (feature `energy-meter`) or an honest fixture supplies real package `measured_j` with **Measured** honesty, close may **COMMIT**. Probe fail, missing sample, or **Modeled** (CpuProxy) → **REFUSE**. Attach via `MolRequest::with_meter_sample` / `AskOpts::with_meter_required`. Never invent `measured_j` to pass the gate.

---

## Stack choice: egui/eframe (not Tauri, not Electron)

| Option | Verdict |
|---|---|
| **egui/eframe** | **Chosen.** Pure Rust immediate-mode; builds on Linux box + Mac (wgpu/Metal); optional `gui` feature; headless lib always builds; aligns with Ferric/wgpu culture; no WebView. |
| Tauri | Rejected for default path — Linux needs webkit2gtk; heavier web surface; fights “machine protocol first”. |
| Electron | Forbidden — interface tax + joule hypocrisy. |

---

## Crate layout

```text
crates/mol-desktop/
  src/lib.rs          # ShellSession + views; GUI_ENABLED
  src/shell.rs        # ask_close, ecosystem_certify, ledger, transcript, meter attach
  src/view.rs         # ReceiptView (+ fabric_id / kernel proof), FabricView, JouleLedger*, Confirm*
  src/gui.rs          # #[cfg(feature = "gui")] eframe app (Ask/Close + EcosystemCertify)
  src/bin/mol-desktop.rs  # requires --features gui
  tests/shell_headless.rs
```

---

## How to run

### Headless (prove / CI / Linux box without caring about display)

```bash
cd /workspace/mixture-of-limits   # or clone path on Mac
cargo test -p mol-desktop
cargo test --workspace
cargo run -p mol-cli -- prove     # includes desktop_shell_headless criterion
```

### Optional OS energy meter

```bash
cargo run -p mol-cli --features energy-meter -- meter
cargo run -p mol-cli --features energy-meter -- meter --sample-ms 20
cargo run -p mol-cli --features energy-meter -- ask "landauer joules per bit" --close --meter
```

On Mac: RAPL sysfs absent; powermetrics requires `MOL_METER_ALLOW_POWERMETRICS=1` (often root). If unavailable → `measured_j=None` (honest).

### GUI on Mac (or Linux with display)

```bash
cargo run -p mol-desktop --features gui --bin mol-desktop
# Mac (sync tree):
cd /Users/dcharlot/Desktop/mol-sync/mixture-of-limits
cargo run -p mol-desktop --features gui --bin mol-desktop
# Live meter + fabric detect (Metal / SMC on Apple Silicon):
cargo run -p mol-desktop --features gui,energy-meter,fabric-detect --bin mol-desktop
```

Requires a display. Headless VMs: skip GUI; shell API remains the product core.

In the window: **EcosystemCertify** controls → detect · tier formula\|model · meter_required · meter-ms → **Run EcosystemCertify**. Stamps show `energy_honesty`/`measured_j`, encapsulation, agent_lane, `fabric_id`, `kernel:vector_add` proof, compute_steps, commit\|refuse. Soft-ref Formula commits offline with `measured_j=None`.

### Headless EcosystemCertify (protocol lane)

```bash
# Soft-ref (prove path)
cargo test -p mol-desktop ecosystem_certify
# Or via CLI (same law):
cargo run -p mol-cli -- ecosystem-certify
cargo run -p mol-cli --features fabric-detect,energy-meter -- \
  ecosystem-certify --detect --tier model --meter-required --meter-ms 200
```

---

## Machine protocol lane

`ShellSession::ask_close` and `ShellSession::ecosystem_certify` are the stable backends for future MCP-class tools:

1. Client sends query + budget (+ optional meter/confirm flags) **or** EcosystemCertifyOpts (detect, tier, meter_required, meter-ms)
2. MoL close / EcosystemCertify → receipt view JSON (full ecosystem stamps)
3. Ledger + transcript append
4. Human confirm only when `require_confirm` / mutate capability

IDE features (tree, tabs, fuzzy find) are **secondary** roadmap — never block energy harness fidelity.

---

## Secure ecosystem reference (not HCI chrome)

**Correction:** Lux Studio + Nova Browser are a **complete energy-traceable + compute-traceable WASM / encapsulated secure ecosystem** — not a chrome/HCI skin to clone. MoL is **law** that subordinates those harnesses.

→ Full architecture law: [`lux-nova-hci-reference.md`](lux-nova-hci-reference.md)

### MoL borrow stance (desktop)

| Borrow | Do | Don’t |
|---|---|---|
| Encapsulation / WASM capsules | Stamp `encapsulation` + `compute_steps` on close; refuse host-share | Ship Photon/Nova shell as MoL UI |
| Energy honesty | `estimated_j` always; `measured_j` only on real meter; Modeled ≠ Measured; UI labels both | Invent RAPL/IOReport |
| Agent Lane isolation | Separate agent profile/cookies; refuse opaque shared bots; `agent_lane` on ReceiptView | Share human session with agent loops |
| Fail-closed | `meter_required` commits on real measured_j, refuses otherwise; capsule/cite gates | Keyword-only safety |
| Interface tax | `ShellSession` machine lane first; confirm only on consequential acts | Chat-first Copilot chrome |

Annex inventories (crate lists): [`_source_lux_crate_inventory.md`](_source_lux_crate_inventory.md), [`_source_nova_crate_inventory.md`](_source_nova_crate_inventory.md), [`_source_nova_shell_inventory.md`](_source_nova_shell_inventory.md).

