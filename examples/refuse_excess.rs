//! VoI / budget refuse — excess tokens do not buy outcomes.
//!
//! `cargo run --example refuse_excess`

use mol_core::{Budget, FloorKind, MolRequest};
use mol_limits::{MixtureOfLimits, MolOutcome};

fn main() {
    let mol = MixtureOfLimits::new();
    let req = MolRequest::new(
        "write a long speculative essay about consciousness in silicon",
        Budget::demo(),
    );
    let out = mol.route(&req).expect("route");
    match out {
        MolOutcome::Refused { floor, receipt } => {
            assert_eq!(floor.kind, FloorKind::ValueOfInformation);
            assert!(receipt.answer.is_none());
            assert!(!receipt.board_synth_claimed);
            println!("refuse OK: limit={} — {}", floor.id, floor.reason);
            println!("estimated_j={} (no model spend)", receipt.estimated_j);
        }
        MolOutcome::Answered(_) => panic!("expected VoI refuse for free-form"),
    }
}
