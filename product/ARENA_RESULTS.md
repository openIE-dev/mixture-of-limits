# ARENA_RESULTS.md — soft-ref head-on scoreboard

**Source:** actual `cargo run -p mol-cli -- arena` on this tree.  
**When:** Fri Oct 02, 2026 12:55 PM EDT (America/New_York).
**Claim class:** soft-ref **Estimated** only — never invent `measured_j`. Estimates ≠ `measured_j`.  
`board_synth_claimed=false`.

## By strategy

| Strategy | correct_close | refuse_when_C=1 | mean estimated_j | mean wall_us | commits | refuses |
|---|---|---|---|---|---|---|
| **mol_cascade** | **36/36 (100%)** | **5/5 (100%)** | **~2.07e-11** | ~148 | 28 | 8 |
| frontier_sim | 28/36 (78%) | 0/5 (0%) | 5.0e-1 | ~160 | 36 | 0 |
| system_one_leaf | 28/36 (78%) | 0/5 (0%) | 2.5e-4 | ~1 | 36 | 0 |

**Head-on (Estimated):** mol/frontier ≈ **4.1e-11**, mol/system_one ≈ **8.3e-8**.

MoL wins close law whenever floors exist — **Lookup** LUT commits, **Formula** closed-form risk (LUT miss), **Solver** ticket route / SAT assign / tiny knapsack (LUT miss), **Phase-1** unstructured ticket/risk/decision → typed AST → cascade (Lookup/Formula; Model LAST last), plus satiation/VoI refuse. Typed baselines kept (`phase1=false`). Frontier and System One leaf stubs still commit past `C(z)=1` and VoI=0 — correct on pure typed commits, wrong on economic done.

Optional `real_leaf` (OpenAI-compatible Model LAST) is off in this table — pass `--endpoint URL` (and optional `--profile laya|jev|decider`) to score it. Offline Model LAST stub remains the default residual path; never invents `measured_j`.

## Chore set (n=36)

| Kind | Count | Expect |
|---|---|---|
| Ticket-close LUT | 6 | Commit Lookup (R-HOWTO/OK/DUP/BUGFIX/WONTFIX/REFUND) |
| Risk LUT | 3 | Commit Lookup (RISK-LOW/MED/HIGH) |
| Typed decision LUT | 3 | Commit Lookup (D-APPROVE/DENY/ESCALATE) |
| **Risk Formula** | 3 | Commit **Formula** (`risk score compute …`; LUT miss) |
| **Ticket Solver** | 4 | Commit **Solver** (route rules / sat assign / knapsack; LUT miss) |
| **Phase-1 unstructured** | 9 | Rule AST → typed → Lookup/Formula (ticket/risk/decision) |
| Satiation C(z)=1 | 5 | MoL REFUSE `satiation` |
| VoI=0 free-form | 3 | MoL REFUSE `voi` |

## Re-run

```bash
cd /Users/dcharlot/Desktop/mol-sync/mixture-of-limits
cargo run -p mol-cli -- arena
cargo run -p mol-cli -- arena --json
cargo run -p mol-cli -- arena --endpoint http://127.0.0.1:8080/v1 --profile laya
cargo run -p mol-cli -- prove   # A14 arena honesty (+ Phase-1→typed)
cargo run -p mol-cli -- phase1 "please close ticket as R-OK"
```

Do **not** paste invented board / Arena GPU joules into this table. Update from a fresh `mol arena` run only.
