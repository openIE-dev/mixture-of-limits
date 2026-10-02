# CLI outline — `mol prove` / `mol run` (Rust)

Matches the existing clean-room language: **Rust** workspace `mol-cli` (clap). TypeScript is not the reference language here.

Binary: `mol` (`cargo run -p mol-cli -- …`)

## Exists today (clean-room)

```bash
mol prove              # PLAN.md criteria → VERIFIED / exit 0
mol ask "…" [--close]  # route or close
mol demo
mol agent | memory | fabric | meter | replay | ecosystem-certify
mol limits
mol explain-cascade "…"
```

## Product stubs (wire into `mol-cli`)

### `mol run`

```text
mol run --chore <path-to-mol.yaml> [--allow-model] [--meter] [--meter-required] [--receipt-json]
```

Behavior:

1. Load declarative chore (`version`, `limits`, `pipeline`, `on_refuse`, `completeness`, `measurement`).
2. Bind floors from yaml into `MixtureOfLimits` registry (grammar, voi, energy, ni_certificate, satiation).
3. Execute cascade Lookup → Formula → Solver → Model LAST per `pipeline` + `cascade.allow_model`.
4. On propose from model_fallback: require NI cert before commit; else refuse (`on_refuse.certificate_fail`).
5. If `C(z)=1`: refuse with satiation reason (`on_refuse.completeness_true`).
6. Stamp receipt: `estimated_j` always; `measured_j` only when meter present; never invent.

Exit codes: `0` commit or intentional refuse-with-receipt; non-zero on schema/load/invariant failure.

### `mol prove` (product extension)

Keep clean-room PLAN criteria. Optionally:

```text
mol prove --acceptance <ACCEPTANCE.md|path>
```

Maps A1–A6 to VERIFIED lines (grammar model-cold, VoI refuse, satiation, NI gate, estimate honesty, meter honesty).

### Sketch patch point (`crates/mol-cli/src/main.rs`)

```rust
/// Run a declarative Mixture of Limits chore (mol.yaml).
Run {
    /// Path to mol.yaml
    #[arg(long)]
    chore: PathBuf,
    #[arg(long, default_value_t = false)]
    allow_model: bool,
    #[arg(long, default_value_t = false)]
    meter: bool,
    #[arg(long, default_value_t = false)]
    meter_required: bool,
    #[arg(long, default_value_t = false)]
    receipt_json: bool,
},
```

Implementation lives beside `prove.rs` as `run.rs`: parse yaml → build `MolRequest` / floors → `MixtureOfLimits::close` → print receipt. MVP may start as a stub that validates schema and delegates known asks to existing close paths.

## Roadmap CLI (paper §11.5 — not MVP claim)

`mol dev` (watch, replay, receipt diff) → `mol bench` (J/query vs MoE baselines with Estimated|Metered labels only).
