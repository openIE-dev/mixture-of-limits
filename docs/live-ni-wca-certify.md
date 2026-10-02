# Live NI / WCA certify (HTTP | MCP) — in-crate fallback

**Status:** Wired (env-gated, default off). Soft-ref prove stays offline.

## What is live

| Path | When | Source stamp |
|---|---|---|
| **In-crate** `InCrateNiCertify` | No `MOL_NI_CERTIFY_URL` / `MOL_WCA_CERTIFY_URL` | `in_crate` |
| **HTTP** POST `{url}/v1/certify` | URL set, `MOL_CERTIFY_MODE=http` (default) | `http` |
| **MCP-shaped** JSON-RPC `tools/call` (`ni_certify` / `wca_certify`) | URL set + mode `mcp` (or URL contains `/mcp`) | `mcp` |
| **Fallback** | Live configured but transport/parse fails and `MOL_CERTIFY_FALLBACK`≠`0` | `in_crate_fallback` |

Close (`MixtureOfLimits::close`) always goes through `certify_live_prefer_env`.

## Env

| Var | Meaning |
|---|---|
| `MOL_NI_CERTIFY_URL` | Preferred live base URL |
| `MOL_WCA_CERTIFY_URL` | Alias if NI unset |
| `MOL_CERTIFY_MODE` | `http` (default) or `mcp` |
| `MOL_CERTIFY_TOKEN` | Optional `Authorization: Bearer …` |
| `MOL_CERTIFY_FALLBACK` | Default `1` — fall back to in-crate on live error; `0` fail-closed |
| `MOL_CERTIFY_TIMEOUT_MS` | Default `5000` |

## HTTP contract

`POST /v1/certify` JSON body:

```json
{ "summary": "…", "estimated_j": 1e-9, "efa_tag": null, "energy_residual": null }
```

Response (minimal):

```json
{
  "decision": "commit",
  "certificate_id": "ni:…",
  "efa_id": "efa:…",
  "wca_id": "wca:…",
  "reasons": ["…"],
  "estimated_j": 1e-9
}
```

`decision` also accepts `allow`/`refuse`. Missing ids are minted locally (`ni:`/`efa:`/`wca:`).

## Honesty fence (always)

- **`measured_j` only when metered** — remote `measured_j` is **ignored**; soft-ref receipts keep `measured_j=None`.
- **`board_synth_claimed=false`** always on this path (remote `true` forced off).
- **`stage_c_measured=false`** always — **FPGA Stage C / WCA LUT gate on silicon is still a stub** for MoL close.
- **Ferric / on-device EFA hardware certificate is still a stub** — live HTTP/MCP here is a software certify service, not Ferric path-dep or BMI robot loop.
- Catalog `estimated_j` only; never copy estimates into `measured_j`.

## Still stub (OUT OF PROOF SCOPE)

| Item | Status |
|---|---|
| Ferric path-dep / on-device EFA certificate | Stub (reference semantics only) |
| FPGA Stage C / `wca-lut-edge` in-proc / UART board meter | Stub (`stage_c_measured=false`) |
| klere-vm WASM / FPGA pJ meter | Stub |
| openie-leapfrog live ask | Offline zone map only |
| Sibling path-deps (`wca-path` feature) | Reserved; HTTP/MCP does not compile siblings |

## Prove

```bash
cargo run -p mol-cli -- prove   # includes product_a7_* and product_a7b_*
cargo test -p mol-adapters
```

A7 = in-crate ids + commit|refuse. A7b = prefer_env default + transport fallback + `measured_j=None`.
