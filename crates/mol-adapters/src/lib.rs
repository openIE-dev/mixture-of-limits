//! Trait adapters documenting hooks to sibling OpenIE / BMI / Klere projects.
//!
//! **Default: software-reference stubs** that exercise propose/certify/refuse
//! and settle-or-refuse locally. Optional features `openie-path` / `wca-path` /
//! `jouledb-path` / `efa-path` / `klere-path` are reserved for future light
//! path-deps; v0.1 keeps stubs so this workspace builds without compiling
//! `openie-leapfrog`, `wca-lut-edge`, `jouledb`, Ferric, or klere-vm.
//! Real integration map: `BLUEPRINT.md` §path to MoL.

#![deny(missing_docs)]

mod efa;
mod jouledb;
mod klere;
mod openie;
mod wca;
mod ni_live;
mod model_last;

pub use efa::{
    live_efa_stub, EfaCertResult, EfaCertificatePort, EfaDecision, EfaProposal, StubEfaCertificate,
};
pub use jouledb::{live_query_stub, JouleDbCascadePort, JouleDbExplain, StubJouleDbCascade};
pub use klere::{
    live_klere_stub, parse_ternary_state, KlereDecision, KlereSettleJob, KlereSettlePort,
    KlereSettleResult, StubKlereSettle, Ternary,
};
pub use openie::{live_ask_stub, OpenIeAskResult, OpenIeRuntimePort, StubOpenIeRuntime};
pub use wca::{live_wca_stub, software_reference_cert, StubWcaCommit, WcaCertResult, WcaCommitPort};
pub use ni_live::{InCrateNiCertify, LiveCertOutcome, NiCertificate};
pub use model_last::{
    live_model_last_stub, model_last_from_endpoint, ModelLastPort, ModelLastProfile,
    ModelLastProposal, OpenAiCompatibleModelLast, StubModelLast,
    MODEL_LAST_ENDPOINT_ESTIMATED_J, MODEL_LAST_STUB_ESTIMATED_J,
};

/// Crate version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;
    use mol_core::{Budget, MolRequest, QueryKind};

    #[test]
    fn efa_refuses_diverge() {
        let efa = StubEfaCertificate;
        let r = efa
            .certify(&EfaProposal {
                summary: "arm swing".into(),
                estimated_j: 1e-6,
                energy_residual: None,
                tag: Some("diverge".into()),
            })
            .unwrap();
        assert_eq!(r.decision, EfaDecision::Refuse);
        assert!(!r.board_synth_claimed);
        assert!(r.floor.is_some());
    }

    #[test]
    fn klere_settles_or_refuses() {
        let k = StubKlereSettle;
        let ok = k
            .settle(&KlereSettleJob {
                summary: "demo settle".into(),
                state: vec![1, 1, 1, 1],
                max_steps: 16,
                force_refuse: false,
            })
            .unwrap();
        assert!(matches!(
            ok.decision,
            KlereDecision::Settled | KlereDecision::Refuse
        ));
        assert!(ok.measured_j.is_none());

        let no = k
            .settle(&KlereSettleJob {
                summary: "will not settle".into(),
                state: vec![1, 1, 1],
                max_steps: 8,
                force_refuse: true,
            })
            .unwrap();
        assert_eq!(no.decision, KlereDecision::Refuse);
        assert!(no.floor.is_some());
    }

    #[test]
    fn openie_offline_map() {
        let o = StubOpenIeRuntime;
        let req = MolRequest::new("landauer joules per bit", Budget::demo());
        assert_eq!(req.kind, QueryKind::ClosedFormPhysics);
        let r = o.ask(&req).unwrap();
        assert!(r.offline);
        assert_eq!(r.zone, mol_core::OpenIeZone::Z1);
    }

    #[test]
    fn wca_software_ref_refuses_destructive() {
        let w = StubWcaCommit;
        let r = w.certify("rm -rf /", 1e-9).unwrap();
        assert_eq!(r.decision, "refuse");
        assert!(!r.board_synth_claimed);
    }
}
