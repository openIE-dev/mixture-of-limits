//! Headless shell API criterion (GUI optional).
use mol_desktop::{AskOpts, EcosystemCertifyOpts, ShellSession};
use std::time::Duration;

#[test]
fn shell_api_headless_close_and_ledger() {
    let mut s = ShellSession::new();
    let a = s.close_demo("landauer joules per bit").expect("close");
    assert!(a.commit);
    assert!(a.view.estimated_j >= 0.0);
    assert!(a.view.measured_j.is_none(), "soft-ref default: measured_j=None");
    assert!(!a.view.board_synth_claimed);
    assert!(a.view.honesty_ok());

    let b = s.close_demo("write a poem about GPUs").expect("voi");
    assert!(!b.commit);
    assert_eq!(b.view.limit.as_deref(), Some("voi"));

    let sum = s.ledger_summary();
    assert_eq!(sum.acts, 2);
    assert_eq!(sum.commits, 1);
    assert_eq!(sum.refuses, 1);
    assert!(sum.measured_honest);

    let fabric = s.fabric_view();
    assert_eq!(fabric.source, "software_ref");
    assert!(fabric.present.iter().any(|p| p == "cpu"));
}

#[test]
fn shell_meter_attach_honest() {
    let mut s = ShellSession::new();
    let r = s
        .ask_close(
            "convert 100 celsius to fahrenheit",
            AskOpts::demo().with_meter(Duration::from_millis(5)),
        )
        .expect("close");
    assert!(r.view.honesty_ok());
    // Never invent: either None or labeled measured source.
    match r.view.measured_j {
        None => {}
        Some(j) => {
            assert!(j.is_finite() && j >= 0.0);
            assert!(matches!(
                r.view.measure_source.as_str(),
                "rapl"
                    | "macos_energy"
                    | "windows_energy"
                    | "nvml"
                    | "cpu_proxy"
                    | "powermetrics"
                    | "ioreport"
                    | "smc"
            ));
        }
    }
}

#[test]
fn shell_ecosystem_certify_soft_ref() {
    let mut s = ShellSession::new();
    let r = s.ecosystem_certify_soft_ref().expect("ecosystem");
    assert!(r.commit);
    assert_eq!(r.capsule_return, Some(42));
    assert!(r.view.encapsulation.as_ref().map(|e| e.sealed) == Some(true));
    assert!(r.view.agent_lane.is_some());
    assert!(r.view.measured_j.is_none(), "never invent measured_j");
    assert_eq!(r.view.energy_honesty, "estimated");
    assert!(
        r.view
            .compute_steps
            .iter()
            .any(|c| c.label == "fabric:route" && c.fabric_id.as_deref() == Some("cpu"))
    );
    // kernel:vector_add may be skip-stamped on Cpu schedule; proof optional, joules never invented
    for c in &r.view.compute_steps {
        if c.label.starts_with("kernel:") {
            assert!(c.measured_j.is_none());
        }
    }
    let refuse = s
        .ecosystem_certify(EcosystemCertifyOpts {
            refuse_without_grant: true,
            ..EcosystemCertifyOpts::soft_ref()
        })
        .expect("refuse path");
    assert!(!refuse.commit);
    assert_eq!(refuse.floor_id.as_deref(), Some("grant_receipt_required"));
}
