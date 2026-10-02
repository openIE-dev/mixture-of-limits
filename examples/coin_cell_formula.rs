//! Coin-cell path: closed-form wins; model never invoked.
//!
//! ```bash
//! cargo run --example coin_cell_formula -p mol-limits
//! ```
//! This example lives at workspace root; run via:
//! `cargo run --example coin_cell_formula`

use mol_core::{Budget, CascadeTier, MolRequest};
use mol_limits::MixtureOfLimits;

fn main() {
    let mol = MixtureOfLimits::new();
    let req = MolRequest::new("landauer joules per bit at 300 kelvin", Budget::coin_cell());
    let out = mol.route(&req).expect("route");
    assert!(out.is_answered(), "formula must close on coin-cell budget");
    let r = out.receipt();
    assert_eq!(r.cascade_answered, Some(CascadeTier::Formula));
    assert!(r.measured_j.is_none(), "never invent RAPL");
    assert!(!r.board_synth_claimed);
    // Model step should be absent or never Answered.
    assert!(
        r.cascade_steps
            .iter()
            .all(|s| s.tier != CascadeTier::Model
                || !matches!(
                    s.outcome,
                    mol_receipt::CascadeStepOutcome::Answered
                )),
        "model must not answer"
    );
    println!("coin-cell OK: {}", r.answer.as_deref().unwrap_or("?"));
    println!("estimated_j={}", r.estimated_j);
}
