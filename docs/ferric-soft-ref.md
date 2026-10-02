# Ferric soft-ref inventory

MoL wires a **soft-ref Ferric inventory** over sibling `ferric` / `ferrix`
checkouts when `MOL_FERRIC_ROOT` is set or a documented Mac path exists.

## Honesty

- No path-dep on Ferric crates (reference semantics only)
- `stage_c_measured=false` always
- `board_synth_claimed=false` always
- Soft-ref EFA certify available in-tree even when checkout absent
- On-device Ferric / MuJoCo / robot meters → `live_ferric_meter_stub` (AdapterStub)

## API

```rust
use mol_adapters::{probe_ferric, certify_ferric_soft, live_ferric_meter_stub};

let inv = probe_ferric();
assert!(!inv.stage_c_measured);
assert!(inv.soft_ref_efa);
let cert = certify_ferric_soft("ferric soft-ref fabric inventory");
assert!(!cert.board_synth_claimed);
```

Prove: `product_ferric_soft_ref`. Sibling Stage C: [`fpga-stage-c.md`](./fpga-stage-c.md).
