# § note — Arena head-on (Mixture of Limits)

**Status:** draft note for research.openie.dev MoL study (not a live site edit).  
**When:** Fri Oct 2, 2026 (America/New_York).  
**Claim class:** soft-ref Estimated only — never invent `measured_j`.

## Thesis

Mixture of Limits competes head-on on Arena-shaped chores (typed decision / ticket-close / risk). Floors close first; Model LAST still opens when VoI>0; peers age; each stands alone; frontier is expensive and out of road; time is fair.

## Soft-ref scoreboard (`mol arena`)

On the clean-room soft-ref harness:

| Strategy | correct_close | refuse_when_C=1 | mean estimated_j |
|---|---|---|---|
| mol_cascade | 10/10 | 3/3 | ~1e-16 |
| frontier_sim | 6/10 | 0/3 | 5e-1 |
| system_one_leaf | 6/10 | 0/3 | 2.5e-4 |

MoL wins close law whenever floors exist (LUT commit + satiation/VoI refuse). Frontier and System One leaf stubs still “decide” past `C(z)=1` and VoI=0 — correct on pure typed commits, wrong on economic done.

## Relation to Jev Arena / JevBench / DecisionBench

Those labs measure typed decision / classification accuracy under deployment constraints. They shortlist residual Model LAST proposers. They do not own navigation floors, NI certify, or satiation stop. Mixture of Limits does not invent their composite scores; it scores the complementary close law on the same chore shapes.

## Operator

```bash
cargo run -p mol-cli -- arena
cargo run -p mol-cli -- prove   # A14 arena honesty
```

Fold into paper §§ when MoL living pages exist. Do not deploy invented board watts.
