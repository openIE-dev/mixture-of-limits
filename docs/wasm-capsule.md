# MoL WASM capsule certify

Clean-room path: run a tiny sandboxed WASM module under capsule bounds, stamp
`EncapsulationReceipt` + `ComputeStepReceipt`(s) with **energy honesty**, then
**commit|refuse** fail-closed.

## Law

| Rule | Behavior |
|---|---|
| Sealed + encapsulated boundary | Required when fail-closed / prove happy path |
| No FS / net | Grants `Fs` / `Net` → **refuse** (`capsule_grant_denied`) unless future explicit grant path |
| No host imports | Import section non-empty → **refuse** |
| Fuel bound | Exceed `max_fuel` → **refuse** (`capsule_fuel_exceeded`) |
| Host-session share | `shares_host_session` → **refuse** (Encapsulation floor) |
| Energy honesty | Fuel → **estimated_j** only (`MeasureSource::CascadeEstimate`, `EnergyHonestyClass::Estimated`). **`measured_j` stays `None`** — never invent measured from fuel |

## Runtime

- **Default / prove:** `StubCapsuleRuntime` — validates real `.wasm` bytes and
  interprets a tiny opcode subset (`local.get`, `i32.add`, `end`) under fuel.
- **Optional:** `--features wasmtime` on `mol-core` enables `WasmtimeCapsuleRuntime`
  (fuel on, no WASI). Soft-ref prove does **not** require Wasmtime.

Fixture: `crates/mol-core/fixtures/add.wasm` — exports `add(i32,i32)->i32`.

## API sketch

```rust
use mol_core::{CapsuleInvoke, CapsuleRuntime, StubCapsuleRuntime};

let r = StubCapsuleRuntime.certify(&CapsuleInvoke::sealed_add_fixture(2, 40));
assert!(r.is_commit());
assert_eq!(r.return_i32, Some(42));
assert!(r.measured_j.is_none());
assert!(r.encapsulation.sealed);
assert!(!r.compute_steps.is_empty());
```

MoL `close` with `MolRequest::with_capsule(...)` still stamps encapsulation +
`compute_steps` on the receipt (context law). The certify path above is the
executable WASM bound.

## Prove

```bash
cd /workspace/mixture-of-limits
cargo run -p mol-cli -- prove
# looks for: VERIFIED ecosystem_wasm_capsule
```

Optional Wasmtime smoke (not required for prove):

```bash
cargo test -p mol-core --features wasmtime
```

## Out of scope

Live WASI Preview 2 Component Model host, browser Wasm, inventing `measured_j`
from fuel, FS/net-enabled capsules, Lux/Nova/joulesperbit path-deps.
