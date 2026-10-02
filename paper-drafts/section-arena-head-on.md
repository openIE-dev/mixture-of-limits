# § note — Arena head-on (Mixture of Limits)

**Status:** draft note for research.openie.dev MoL study (not a live site edit).  
**When:** Fri Oct 2, 2026 07:51 AM EDT (America/New_York).  
**Claim class:** soft-ref Estimated only — never invent `measured_j`. Estimates ≠ `measured_j`.

## Thesis

Mixture of Limits competes head-on on Arena-shaped chores (typed decision / ticket-close / risk), including **Phase-1** unstructured → typed AST → Lookup/Formula/Solver/Model LAST. Floors close first; Model LAST still opens when VoI>0; peers age; each stands alone; frontier is expensive and out of road; time is fair.

## Soft-ref scoreboard (`mol arena`)

On the clean-room soft-ref harness (n=36; typed baselines kept + Phase-1 unstructured fixtures):

| Strategy | correct_close | refuse_when_C=1 | mean estimated_j |
|---|---|---|---|
| mol_cascade | 36/36 | 5/5 | ~2.1e-11 |
| frontier_sim | 28/36 | 0/5 | 5e-1 |
| system_one_leaf | 28/36 | 0/5 | 2.5e-4 |

MoL wins close law whenever floors exist (LUT / Formula / Solver / Phase-1→typed + satiation/VoI refuse). Frontier and System One leaf stubs still “decide” past `C(z)=1` and VoI=0 — correct on pure typed commits, wrong on economic done.

## Relation to Jev Arena / JevBench / DecisionBench

Those labs measure typed decision / classification accuracy under deployment constraints. They shortlist residual Model LAST proposers. They do not own navigation floors, NI certify, or satiation stop. Mixture of Limits does not invent their composite scores; it scores the complementary close law on the same chore shapes.

## Operator

```bash
cargo run -p mol-cli -- arena
cargo run -p mol-cli -- prove   # A14 arena honesty
cargo run -p mol-cli -- phase1 "please close ticket as R-OK"
```

Fold into paper §§ when MoL living pages exist. Do not deploy invented board watts. §11.1 research hub path was absent on this machine — no live site deploy this turn.
