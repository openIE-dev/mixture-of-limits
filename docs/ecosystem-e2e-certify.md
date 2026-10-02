# MoL ecosystem e2e certify

One **end-to-end** path that closes Agent Lane + multi-fabric schedule + WASM
capsule + energy honesty + GrantReceipt host invoke into a **single receipt**.

Prior criteria (`ecosystem_wasm_capsule`, `ecosystem_agent_lane_session`,
`ecosystem_multi_fabric`, energy honesty) stay green in isolation. This path
**composes** them.

## Law

| Step | Behavior |
|---|---|
| 1. Open Agent Lane | `AgentLaneSession::open_agent` — cookies/profile/storage partitioned |
| 2. Schedule fabric | Soft-ref inventory; Formula → **Cpu** offline-honest; stamps `fabric:route` |
| 3. Invoke WASM capsule | Fixture `add.wasm` under sealed bounds (`StubCapsuleRuntime`) |
| 4. Energy honesty | Fuel → **estimated_j** only; optional `meter_sample` if provided — **never invent `measured_j`** |
| 5. Host invoke | Allow **only** with `GrantReceipt` + lane provenance (keyword alone insufficient) |
| 6. commit\|refuse | Single `MolReceipt` stamps: `encapsulation`, `agent_lane`, `compute_steps`/fabric, `energy_honesty` |

Refuse path (flag / config): omit GrantReceipt → `grant_receipt_required` (capsule
+ encapsulation still stamped for audit).

## API

```rust
use mol_automate::{EcosystemCertify, EcosystemCertifyConfig};

// Happy path (soft-ref)
let ok = EcosystemCertify::run(&EcosystemCertifyConfig::soft_ref());
assert!(ok.is_commit());
assert_eq!(ok.capsule_return, Some(42));
assert!(ok.receipt.encapsulation.as_ref().unwrap().sealed);
assert!(ok.receipt.agent_lane.as_ref().unwrap().grant_id.is_some());
assert_eq!(ok.receipt.fabric_chosen, Some(mol_core::DeviceKind::Cpu));
assert!(ok.receipt.measured_j.is_none());

// Refuse without grant
let refuse = EcosystemCertify::run(&EcosystemCertifyConfig::refuse_without_grant());
assert!(refuse.is_refuse());
```

## CLI / prove

```bash
cd /workspace/mixture-of-limits

# Operator surface
cargo run -p mol-cli -- ecosystem-certify
cargo run -p mol-cli -- ecosystem-certify --receipt-json
cargo run -p mol-cli -- ecosystem-certify --refuse-without-grant

# Live macOS SMC package meter (feature energy-meter; never invents measured_j)
cargo run -p mol-cli --features energy-meter -- ecosystem-certify --meter --meter-ms 200
cargo run -p mol-cli --features energy-meter -- ecosystem-certify --meter-required --meter-ms 200 --receipt-json

# Metal kernel + overlapping SMC (fabric-detect + energy-meter): stamps kernel measured_j
cargo run -p mol-cli --features fabric-detect,energy-meter -- \
  ecosystem-certify --detect --tier model --meter-required --meter-ms 200

# Proof harness (looks for VERIFIED ecosystem_e2e_certify)
cargo run -p mol-cli -- prove
```

## Out of scope

Live Ferric / silicon meters as default, inventing `measured_j` from fuel,
Lux/Nova/`web-browser` path-deps, keyword-only host authorize, GPU required for CI.

SMC / macOS package meter remains a **separate** Mac path (`docs/macos-package-meter.md`);
optional `meter_sample` may be attached when a real probe provided it.
