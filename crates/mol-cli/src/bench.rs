//! `mol bench` — J/query comparison: Mixture of Limits vs always-model / MoE-sim.
//!
//! Labels energy as **Estimated** or **Metered** only. Never invents board joules.
//! `board_synth_claimed=false`.

use std::process::ExitCode;
use std::time::Instant;

use mol_core::{
    Budget, DeviceKind, FabricInventory, Joules, MolRequest, BOARD_SYNTH_CLAIMED,
};
use mol_limits::{CloseOutcome, MixtureOfLimits};
use serde::Serialize;

/// Energy label for bench rows — Estimated | Metered only.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum EnergyLabel {
    Estimated,
    Metered,
}

#[derive(Debug, Serialize)]
struct BenchRow {
    strategy: String,
    query: String,
    commit: bool,
    limit_id: Option<String>,
    tier: Option<String>,
    estimated_j: f64,
    measured_j: Option<f64>,
    energy_label: EnergyLabel,
    wall_us: u64,
    model_invoked: bool,
}

#[derive(Debug, Serialize)]
struct BenchReport {
    product: String,
    board_synth_claimed: bool,
    note: String,
    rows: Vec<BenchRow>,
    summary: BenchSummary,
}

#[derive(Debug, Serialize)]
struct BenchSummary {
    mol_mean_estimated_j: f64,
    always_model_mean_estimated_j: f64,
    moe_sim_mean_estimated_j: f64,
    mol_commits: u32,
    always_model_commits: u32,
    mol_refuses: u32,
    joule_ratio_mol_over_always_model: f64,
}

const ASKS: &[&str] = &[
    "ticket close resolution=R-HOWTO",
    "convert 100 celsius to fahrenheit",
    "landauer joules per bit",
    "settle ternary [1, 1, 1, 1]",
    "write a free-form poem about GPUs",
];

/// Run J/query bench.
pub fn cmd_bench(json: bool) -> ExitCode {
    let mol = MixtureOfLimits::new();
    let mol_gpu = MixtureOfLimits::new()
        .with_fabric(FabricInventory::software_ref_with_gpu(DeviceKind::GpuMetal));

    let mut rows = Vec::new();

    for q in ASKS {
        // --- Mixture of Limits (default: model cold) ---
        rows.push(run_one("mol_cascade", &mol, q, Budget::coin_cell(), false));

        // --- Always-model baseline (forces model leaf when possible) ---
        let mut b = Budget::demo().allow_model();
        b.max_j = Joules::new(1.0);
        // Prefix residual so free-form / uncovered opens model; grammar hits still close cold.
        let always_q = if q.contains("poem") || q.contains("free-form") {
            format!("residual propose always-model baseline: {q}")
        } else {
            (*q).to_string()
        };
        rows.push(run_one(
            "always_model",
            &mol_gpu,
            &always_q,
            b,
            true,
        ));

        // --- MoE-sim: catalog surrogate = N experts × gate cost (estimate only) ---
        rows.push(moe_sim_row(q));
    }

    let summary = summarize(&rows);
    let report = BenchReport {
        product: "Mixture of Limits".into(),
        board_synth_claimed: BOARD_SYNTH_CLAIMED,
        note: "J/query bench: Estimated|Metered labels only; never invent measured_j; MoE-sim is catalog surrogate (not RAPL)".into(),
        rows,
        summary,
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
    } else {
        println!("=== mol bench — Mixture of Limits vs always-model / MoE-sim ===");
        println!("board_synth_claimed={}\n", report.board_synth_claimed);
        for r in &report.rows {
            println!(
                "  [{:<14}] {:>10.3e} J ({:?}) commit={} model={} limit={:?}  {}",
                r.strategy,
                r.estimated_j,
                r.energy_label,
                r.commit,
                r.model_invoked,
                r.limit_id,
                r.query
            );
        }
        println!();
        println!(
            "summary: mol_mean_est_j={:.3e}  always_model_mean_est_j={:.3e}  moe_sim_mean_est_j={:.3e}",
            report.summary.mol_mean_estimated_j,
            report.summary.always_model_mean_estimated_j,
            report.summary.moe_sim_mean_estimated_j
        );
        println!(
            "         mol_commits={} always_model_commits={} mol_refuses={}  ratio_mol/always={:.3}",
            report.summary.mol_commits,
            report.summary.always_model_commits,
            report.summary.mol_refuses,
            report.summary.joule_ratio_mol_over_always_model
        );
        println!("labels: Estimated|Metered only — no invented board joules");
    }
    ExitCode::SUCCESS
}

fn run_one(
    strategy: &str,
    mol: &MixtureOfLimits,
    q: &str,
    budget: Budget,
    expect_model_path: bool,
) -> BenchRow {
    let t0 = Instant::now();
    let out = mol.close(&MolRequest::new(q, budget));
    let wall_us = t0.elapsed().as_micros() as u64;
    match out {
        Ok(o) => row_from_outcome(strategy, q, &o, wall_us, expect_model_path),
        Err(e) => BenchRow {
            strategy: strategy.into(),
            query: q.into(),
            commit: false,
            limit_id: Some(format!("error:{e}")),
            tier: None,
            estimated_j: 0.0,
            measured_j: None,
            energy_label: EnergyLabel::Estimated,
            wall_us,
            model_invoked: false,
        },
    }
}

fn row_from_outcome(
    strategy: &str,
    q: &str,
    o: &CloseOutcome,
    wall_us: u64,
    _expect_model: bool,
) -> BenchRow {
    let r = o.receipt();
    let measured = r.measured_j.map(|j| j.0);
    let label = if measured.is_some() && r.measure_source.is_measured() {
        EnergyLabel::Metered
    } else {
        EnergyLabel::Estimated
    };
    let model_invoked = r
        .cascade_steps
        .iter()
        .any(|s| s.tier.label() == "model" && matches!(s.outcome, mol_receipt::CascadeStepOutcome::Answered));
    BenchRow {
        strategy: strategy.into(),
        query: q.into(),
        commit: o.is_commit(),
        limit_id: r.limit_fired.as_ref().map(|f| f.id.as_str().to_string()),
        tier: r.cascade_answered.map(|t| t.label().to_string()),
        estimated_j: r.estimated_j.0,
        measured_j: measured,
        energy_label: label,
        wall_us,
        model_invoked,
    }
}

fn moe_sim_row(q: &str) -> BenchRow {
    // Catalog surrogate: gate + 2-of-8 experts × model leaf estimate.
    // Labeled Estimated only — never Metered.
    const EXPERTS: f64 = 8.0;
    const TOP_K: f64 = 2.0;
    const GATE_J: f64 = 5e-5;
    const EXPERT_J: f64 = 9e-2; // ~model leaf catalog μ class
    let estimated_j = GATE_J + TOP_K * EXPERT_J;
    let _ = EXPERTS;
    BenchRow {
        strategy: "moe_sim".into(),
        query: q.into(),
        commit: true, // MoE-sim always "answers" (no refuse law)
        limit_id: None,
        tier: Some("moe_sim".into()),
        estimated_j,
        measured_j: None,
        energy_label: EnergyLabel::Estimated,
        wall_us: 0,
        model_invoked: true,
    }
}

fn summarize(rows: &[BenchRow]) -> BenchSummary {
    fn mean(rows: &[BenchRow], strat: &str) -> f64 {
        let v: Vec<f64> = rows
            .iter()
            .filter(|r| r.strategy == strat)
            .map(|r| r.estimated_j)
            .collect();
        if v.is_empty() {
            0.0
        } else {
            v.iter().sum::<f64>() / v.len() as f64
        }
    }
    let mol_mean = mean(rows, "mol_cascade");
    let always_mean = mean(rows, "always_model");
    let moe_mean = mean(rows, "moe_sim");
    let mol_commits = rows
        .iter()
        .filter(|r| r.strategy == "mol_cascade" && r.commit)
        .count() as u32;
    let always_commits = rows
        .iter()
        .filter(|r| r.strategy == "always_model" && r.commit)
        .count() as u32;
    let mol_refuses = rows
        .iter()
        .filter(|r| r.strategy == "mol_cascade" && !r.commit)
        .count() as u32;
    let ratio = if always_mean > 0.0 {
        mol_mean / always_mean
    } else {
        0.0
    };
    BenchSummary {
        mol_mean_estimated_j: mol_mean,
        always_model_mean_estimated_j: always_mean,
        moe_sim_mean_estimated_j: moe_mean,
        mol_commits,
        always_model_commits: always_commits,
        mol_refuses,
        joule_ratio_mol_over_always_model: ratio,
    }
}
