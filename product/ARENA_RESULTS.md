# ARENA_RESULTS.md — soft-ref head-on scoreboard

**Source:** actual `cargo run -p mol-cli -- arena` on this tree.  
**When:** Fri Oct 2, 2026 02:28 AM EDT (America/New_York).  
**Claim class:** soft-ref **Estimated** only — never invent `measured_j`.  
`board_synth_claimed=false`.

## By strategy

| Strategy | correct_close | refuse_when_C=1 | mean estimated_j | mean wall_us | commits | refuses |
|---|---|---|---|---|---|---|
| **mol_cascade** | **20/20 (100%)** | **5/5 (100%)** | **~1.1e-16** | ~48 | 12 | 8 |
| frontier_sim | 12/20 (60%) | 0/5 (0%) | 5.0e-1 | ~68 | 20 | 0 |
| system_one_leaf | 12/20 (60%) | 0/5 (0%) | 2.5e-4 | ~1 | 20 | 0 |

**Head-on (Estimated):** mol/frontier ≈ **2.2e-16**, mol/system_one ≈ **4.4e-13**.

MoL wins close law whenever floors exist (LUT commit + satiation/VoI refuse). Frontier and System One leaf stubs still commit past `C(z)=1` and VoI=0 — correct on pure typed commits, wrong on economic done.

Optional `real_leaf` (OpenAI-compatible Model LAST) is off in this table — pass `--endpoint URL` (and optional `--profile laya|jev|decider`) to score it. Offline Model LAST stub remains the default residual path; never invents `measured_j`.

## Chore set (n=20)

| Kind | Count | Expect |
|---|---|---|
| Ticket-close LUT | 6 | Commit (R-HOWTO/OK/DUP/BUGFIX/WONTFIX/REFUND) |
| Risk LUT | 3 | Commit (RISK-LOW/MED/HIGH) |
| Typed decision LUT | 3 | Commit (D-APPROVE/DENY/ESCALATE) |
| Satiation C(z)=1 | 5 | MoL REFUSE `satiation` |
| VoI=0 free-form | 3 | MoL REFUSE `voi` |

## Re-run

```bash
cd /Users/dcharlot/Desktop/mol-sync/mixture-of-limits
cargo run -p mol-cli -- arena
cargo run -p mol-cli -- arena --json
cargo run -p mol-cli -- arena --endpoint http://127.0.0.1:8080/v1 --profile laya
cargo run -p mol-cli -- prove   # A14 arena honesty
```

Do **not** paste invented board / Arena GPU joules into this table. Update from a fresh `mol arena` run only.
