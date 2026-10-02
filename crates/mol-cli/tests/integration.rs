//! Workspace integration tests for Mixture of Limits — path to MoL owns close.

use mol_adapters::{
    EfaCertificatePort, EfaDecision, EfaProposal, KlereDecision, KlereSettleJob, KlereSettlePort,
    StubEfaCertificate, StubKlereSettle, StubOpenIeRuntime, OpenIeRuntimePort,
};
use mol_automate::{Act, ActKind, AutomateGate, CommitDecision};
use mol_core::{
    Budget, CascadeTier, Deterministic, FloorKind, ModelGenerated, MolRequest, QueryKind,
    ReplayClass, TypedAnswer, BOARD_SYNTH_CLAIMED,
};
use mol_limits::{CloseOutcome, MixtureOfLimits, MolOutcome};
use mol_receipt::{
    sign_receipt, verify_receipt_integrity, verify_signature, CascadeStepOutcome,
};

#[test]
fn demo_formula_and_lookup_close() {
    let mol = MixtureOfLimits::new();
    for q in [
        "landauer joules per bit",
        "convert 0 celsius to fahrenheit",
        "rest energy for mass 0.001 kg E=mc2",
    ] {
        let out = mol
            .close(&MolRequest::new(q, Budget::coin_cell()))
            .unwrap();
        assert!(out.is_commit(), "expected commit for {q}");
        let r = out.receipt();
        assert!(r.measured_j.is_none());
        assert_eq!(r.board_synth_claimed, BOARD_SYNTH_CLAIMED);
        assert!(matches!(
            r.cascade_answered,
            Some(CascadeTier::Lookup) | Some(CascadeTier::Formula)
        ));
    }
}

#[test]
fn linear_solve_closes_at_solver() {
    let mol = MixtureOfLimits::new();
    let out = mol
        .route(&MolRequest::new(
            "solve 2x2 [[1,0],[0,1]] [3,4]",
            Budget::demo(),
        ))
        .unwrap();
    assert!(out.is_answered());
    assert_eq!(
        out.receipt().cascade_answered,
        Some(CascadeTier::Solver)
    );
}

#[test]
fn voi_refuses_excess_tokens() {
    let mol = MixtureOfLimits::new();
    let out = mol
        .route(&MolRequest::new(
            "invent a mythology for database indexes",
            Budget::demo(),
        ))
        .unwrap();
    match out {
        MolOutcome::Refused { floor, .. } => {
            assert_eq!(floor.id.as_str(), "voi");
        }
        MolOutcome::Answered(_) => panic!("should refuse"),
    }
}

#[test]
fn model_never_answered_without_allow() {
    let mol = MixtureOfLimits::new();
    let mut req = MolRequest::new("solve nothing useful here xyz", Budget::demo());
    req.kind = QueryKind::LinearSolve;
    let out = mol.route(&req).unwrap();
    assert!(!out.is_answered());
    let r = out.receipt();
    assert!(r
        .cascade_steps
        .iter()
        .filter(|s| s.tier == CascadeTier::Model)
        .all(|s| matches!(
            s.outcome,
            CascadeStepOutcome::Skipped | CascadeStepOutcome::Refused | CascadeStepOutcome::Miss
        )));
}

#[test]
fn replay_model_cannot_become_deterministic() {
    let m = TypedAnswer::<&str, ModelGenerated>::new("noise");
    assert!(m.weaken_to::<Deterministic>().is_err());
    let d = TypedAnswer::<i32, Deterministic>::new(1);
    assert_eq!(d.replay_class(), ReplayClass::Deterministic);
}

#[test]
fn receipt_hmac_roundtrip() {
    let mol = MixtureOfLimits::new();
    let out = mol
        .route(&MolRequest::new("landauer joules per bit", Budget::demo()))
        .unwrap();
    let mut r = out.receipt().clone();
    sign_receipt(&mut r, b"test-secret").unwrap();
    verify_receipt_integrity(&r).unwrap();
    verify_signature(&r, b"test-secret").unwrap();
    assert!(verify_signature(&r, b"wrong").is_err());
}

#[test]
fn automate_default_denies_mutate() {
    let g = AutomateGate::default();
    let out = g.gate(&Act::new(ActKind::Mutate, "touch /tmp/x", 1e-12));
    assert!(matches!(out.decision, CommitDecision::Refuse(_)));
    assert!(!out.board_synth_claimed);
}

#[test]
fn settle_and_settle_refuse() {
    let mol = MixtureOfLimits::new();
    let ok = mol
        .route(&MolRequest::new(
            "settle ternary [1, 1, 1, 1]",
            Budget::demo(),
        ))
        .unwrap();
    assert!(
        ok.is_answered()
            || ok
                .receipt()
                .limit_fired
                .as_ref()
                .map(|f| f.kind == FloorKind::SettleRefuse)
                .unwrap_or(false)
    );

    let no = mol
        .route(&MolRequest::new(
            "settle refuse will not settle [1,1,1]",
            Budget::demo(),
        ))
        .unwrap();
    assert!(!no.is_answered());
    assert_eq!(
        no.receipt().limit_fired.as_ref().map(|f| f.kind),
        Some(FloorKind::SettleRefuse)
    );
}

#[test]
fn efa_and_klere_stubs_honesty() {
    let efa = StubEfaCertificate;
    let r = efa
        .certify(&EfaProposal {
            summary: "act".into(),
            estimated_j: 1e-9,
            energy_residual: Some(1.0),
            tag: None,
        })
        .unwrap();
    assert_eq!(r.decision, EfaDecision::Refuse);
    assert!(!r.board_synth_claimed);

    let k = StubKlereSettle;
    let r = k
        .settle(&KlereSettleJob {
            summary: "will not settle".into(),
            state: vec![1],
            max_steps: 1,
            force_refuse: true,
        })
        .unwrap();
    assert_eq!(r.decision, KlereDecision::Refuse);
    assert!(r.measured_j.is_none());
}

#[test]
fn primitive_gap_and_close_efa_refuse() {
    let mol = MixtureOfLimits::new();
    let gap = mol
        .route(&MolRequest::new(
            "missing primitive on periodic stack gap",
            Budget::demo(),
        ))
        .unwrap();
    match gap {
        MolOutcome::Refused { floor, .. } => assert_eq!(floor.kind, FloorKind::PrimitiveGap),
        MolOutcome::Answered(_) => panic!("expected gap"),
    }

    let close = mol
        .close(&MolRequest::new(
            "landauer joules per bit uncertified",
            Budget::coin_cell(),
        ))
        .unwrap();
    match close {
        CloseOutcome::Refuse { floor, receipt } => {
            assert_eq!(floor.kind, FloorKind::EfaCertificate);
            assert!(receipt.measured_j.is_none());
            assert!(!receipt.board_synth_claimed);
        }
        CloseOutcome::Commit { .. } => panic!("expected efa refuse"),
    }
}

#[test]
fn openie_offline_port() {
    let o = StubOpenIeRuntime;
    let r = o
        .ask(&MolRequest::new("convert 1 meters to feet", Budget::demo()))
        .unwrap();
    assert!(r.offline);
    assert_eq!(r.zone, mol_core::OpenIeZone::Z1);
}
