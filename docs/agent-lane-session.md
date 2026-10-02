# MoL Agent Lane session path

Clean-room **real session path** for Agent Lane isolation. MoL does **not**
modify Lux/Nova/`web-browser`; this is law + soft-ref runtime in-tree.

## Law

| Rule | Behavior |
|---|---|
| Separate partitions | Each [`AgentLaneSession`] owns distinct logical **cookies / profile / storage** roots (`lane:{human\|agent}/{cookies\|profile\|storage}`) |
| No cross-lane share | Attaching or aliasing another lane’s root → **refuse** (`agent_isolation_cross_lane_share` / `agent_isolation_cross_lane`) |
| Host invoke provenance | [`host_invoke`] requires [`LaneProvenance`] (session_id + lane + partition_root) — missing → **refuse** `provenance_missing_lane` |
| Keyword confirm insufficient | Typing `confirm` / `yes` alone never authorizes — need typed [`GrantReceipt`] |
| Grant receipt | Grant must cover session + [`HostCapability`]; mismatch → refuse |
| Stamp | Allow stamps [`AgentLaneReceipt`] with partition roots + `grant_id` |

Nova MVP still documents storage as an honest gap (`nova_mvp_agent` /
`separate_storage=false`). The **real session path** closes that gap in MoL law
(`AgentLaneSession::open_agent` always partitions storage).

## API sketch

```rust
use mol_core::{
    host_invoke, AgentLaneSession, GrantReceipt, HostCapability, HostInvokeRequest,
    KeywordConfirm, LaneProvenance, PartitionSurface,
};

let agent = AgentLaneSession::open_agent("agent-1");
let human = AgentLaneSession::open_human("human-1");
agent.refuse_cross_lane_share(&human, PartitionSurface::Cookies)?; // Err

let prov = LaneProvenance::from_session(&agent, PartitionSurface::Cookies);
let grant = GrantReceipt::issue(&agent, HostCapability::ModelContextExecute, 1);

// Keyword alone → refuse
let _ = host_invoke(&HostInvokeRequest {
    capability: HostCapability::ModelContextExecute,
    session: agent.clone(),
    provenance: Some(prov.clone()),
    keyword_confirm: Some(KeywordConfirm { keyword: "confirm".into() }),
    grant: None,
});

// Provenance + grant → allow + AgentLaneReceipt stamp
let ok = host_invoke(&HostInvokeRequest {
    capability: HostCapability::ModelContextExecute,
    session: agent,
    provenance: Some(prov),
    keyword_confirm: None,
    grant: Some(grant),
});
assert!(ok.is_allow());
```

## Prove

```bash
cd /workspace/mixture-of-limits
cargo run -p mol-cli -- prove
# looks for: VERIFIED ecosystem_agent_lane_session
```

## Out of scope

Live browser cookie jars, Chromium profile dirs, Lux/Nova path-deps, inventing
measured joules, keyword-only safety as a sufficient gate.
