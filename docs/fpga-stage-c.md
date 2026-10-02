# FPGA Stage C / WCA soft-ref inventory

MoL wires a **soft-ref Stage C inventory** over sibling `openie-fpga` artifacts
(WCA commit-gate / seed1 bitstreams, RTL, `.fpga` packs) when
`MOL_OPENIE_FPGA_ROOT` is set or a documented Mac path exists.

## Honesty

- `stage_c_measured=false` always
- `board_synth_claimed=false` always
- Artifact presence ≠ package joules; never invent `measured_j`
- In-proc `wca-lut-edge` / UART board meter remain AdapterStub (`live_stage_c_meter_stub`)

## API

```rust
use mol_adapters::{probe_stage_c, certify_stage_c_soft, live_stage_c_meter_stub};

let inv = probe_stage_c();
assert!(!inv.stage_c_measured);
let cert = certify_stage_c_soft("wca commit gate soft-ref");
assert!(!cert.board_synth_claimed);
```

CLI: `mol dev` prints Stage C artifact presence beside live catalog counts.

See also: [`live-ni-wca-certify.md`](./live-ni-wca-certify.md), BLUEPRINT §15.
