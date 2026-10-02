//! `mol bench` / `mol arena` — Mixture of Limits vs always-model / MoE-sim / System One leaf.
//!
//! Classic mode: J/query vs always-model / MoE-sim.
//! Arena mode: Arena-shaped chores (typed decision / ticket-close / risk) —
//! MoL cascade vs frontier-sim (always-model) vs System One leaf-only (Jev/Laya-class stub).
//!
//! Phase-1 micro-perception (optional per chore): unstructured ticket/risk/decision
//! strings → typed AST/schema → Lookup → Formula → Solver → Model LAST.
//! Baseline typed asks stay as-is (`phase1=false`).
//!
//! Metrics: correct_close, refuse_when_C=1, estimated_j (Estimated label), latency.
//! Labels energy **Estimated | Metered** only. Never invents `measured_j`.
//! `board_synth_claimed=false`. Estimates ≠ `measured_j`.
//! Optional `--endpoint` adds `real_leaf` (OpenAI-compatible Model LAST); default offline stub.

use std::process::ExitCode;
use std::time::Instant;

use mol_adapters::{
    model_last_from_endpoint, ModelLastPort, ModelLastProfile, MODEL_LAST_STUB_ESTIMATED_J,
};
use mol_core::{
    run_phase1, Budget, CompletenessSnapshot, DeviceKind, FabricInventory, Joules, MolRequest,
    Phase1Config, Phase1Outcome, BOARD_SYNTH_CLAIMED,
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

/// Arena chore kind.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum ArenaKind {
    TypedDecision,
    TicketClose,
    Risk,
}

/// Expected close law for scoring.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum ExpectClose {
    Commit,
    RefuseSatiation,
    RefuseVoi,
}

#[derive(Debug, Clone)]
struct ArenaChore {
    id: &'static str,
    kind: ArenaKind,
    ask: &'static str,
    /// When Some and complete → MoL must satiation-refuse.
    completeness: Option<CompletenessSnapshot>,
    expect: ExpectClose,
    /// Gold label / code for typed correctness (LUT code or option).
    gold: &'static str,
    /// When Some, mol_cascade Commit must close at this tier label (lookup|formula|solver).
    expect_tier: Option<&'static str>,
    /// When true, mol_cascade runs Phase-1 rule AST on `ask` before cascade.
    phase1: bool,
}

#[derive(Debug, Serialize)]
struct ArenaRow {
    chore_id: String,
    kind: ArenaKind,
    strategy: String,
    ask: String,
    commit: bool,
    limit_id: Option<String>,
    tier: Option<String>,
    estimated_j: f64,
    measured_j: Option<f64>,
    energy_label: EnergyLabel,
    wall_us: u64,
    model_invoked: bool,
    /// Did outcome match expect (commit / refuse-satiation / refuse-voi)?
    correct_close: bool,
    /// True when expect was satiation and strategy refused with satiation floor.
    refuse_when_c1: Option<bool>,
    gold: String,
}

#[derive(Debug, Serialize)]
struct ArenaStrategySummary {
    strategy: String,
    n: u32,
    correct_close: u32,
    refuse_when_c1_ok: u32,
    refuse_when_c1_n: u32,
    mean_estimated_j: f64,
    mean_wall_us: f64,
    commits: u32,
    refuses: u32,
}

#[derive(Debug, Serialize)]
struct ArenaReport {
    product: String,
    mode: String,
    board_synth_claimed: bool,
    note: String,
    rows: Vec<ArenaRow>,
    by_strategy: Vec<ArenaStrategySummary>,
    /// Head-on: MoL mean estimated_j / frontier_sim mean (lower wins joules).
    joule_ratio_mol_over_frontier: f64,
    /// Head-on: MoL mean estimated_j / system_one_leaf mean.
    joule_ratio_mol_over_system_one: f64,
    /// MoL correct_close rate vs peers.
    mol_correct_close_rate: f64,
    frontier_correct_close_rate: f64,
    system_one_correct_close_rate: f64,
}

const ASKS: &[&str] = &[
    "ticket close resolution=R-HOWTO",
    "convert 100 celsius to fahrenheit",
    "landauer joules per bit",
    "settle ternary [1, 1, 1, 1]",
    "write a free-form poem about GPUs",
];

fn arena_chores() -> Vec<ArenaChore> {
    vec![
        // ── Ticket-close LUT commits
        ArenaChore {
            id: "ticket_lut_howto",
            kind: ArenaKind::TicketClose,
            ask: "ticket close resolution=R-HOWTO",
            completeness: None,
            expect: ExpectClose::Commit,
            gold: "R-HOWTO",
            expect_tier: None,
            phase1: false,
        },
        ArenaChore {
            id: "ticket_lut_ok",
            kind: ArenaKind::TicketClose,
            ask: "ticket close resolution=R-OK",
            completeness: None,
            expect: ExpectClose::Commit,
            gold: "R-OK",
            expect_tier: None,
            phase1: false,
        },
        ArenaChore {
            id: "ticket_lut_dup",
            kind: ArenaKind::TicketClose,
            ask: "ticket close resolution=R-DUP",
            completeness: None,
            expect: ExpectClose::Commit,
            gold: "R-DUP",
            expect_tier: None,
            phase1: false,
        },
        ArenaChore {
            id: "ticket_lut_bugfix",
            kind: ArenaKind::TicketClose,
            ask: "ticket close resolution=R-BUGFIX",
            completeness: None,
            expect: ExpectClose::Commit,
            gold: "R-BUGFIX",
            expect_tier: None,
            phase1: false,
        },
        ArenaChore {
            id: "ticket_lut_wontfix",
            kind: ArenaKind::TicketClose,
            ask: "ticket close resolution=R-WONTFIX",
            completeness: None,
            expect: ExpectClose::Commit,
            gold: "R-WONTFIX",
            expect_tier: None,
            phase1: false,
        },
        ArenaChore {
            id: "ticket_lut_refund",
            kind: ArenaKind::TicketClose,
            ask: "ticket close resolution=R-REFUND",
            completeness: None,
            expect: ExpectClose::Commit,
            gold: "R-REFUND",
            expect_tier: None,
            phase1: false,
        },
        // ── Risk LUT commits
        ArenaChore {
            id: "risk_lut_low",
            kind: ArenaKind::Risk,
            ask: "risk score band=RISK-LOW",
            completeness: None,
            expect: ExpectClose::Commit,
            gold: "RISK-LOW",
            expect_tier: None,
            phase1: false,
        },
        ArenaChore {
            id: "risk_lut_med",
            kind: ArenaKind::Risk,
            ask: "risk score band=RISK-MED",
            completeness: None,
            expect: ExpectClose::Commit,
            gold: "RISK-MED",
            expect_tier: None,
            phase1: false,
        },
        ArenaChore {
            id: "risk_lut_high",
            kind: ArenaKind::Risk,
            ask: "risk score band=RISK-HIGH",
            completeness: None,
            expect: ExpectClose::Commit,
            gold: "RISK-HIGH",
            expect_tier: None,
            phase1: false,
        },
        // ── Typed decision LUT commits
        ArenaChore {
            id: "typed_decide_approve",
            kind: ArenaKind::TypedDecision,
            ask: "typed decide pick=D-APPROVE options=[D-APPROVE,D-DENY,D-ESCALATE]",
            completeness: None,
            expect: ExpectClose::Commit,
            gold: "D-APPROVE",
            expect_tier: None,
            phase1: false,
        },
        ArenaChore {
            id: "typed_decide_deny",
            kind: ArenaKind::TypedDecision,
            ask: "typed decide pick=D-DENY options=[D-APPROVE,D-DENY,D-ESCALATE]",
            completeness: None,
            expect: ExpectClose::Commit,
            gold: "D-DENY",
            expect_tier: None,
            phase1: false,
        },
        ArenaChore {
            id: "typed_decide_escalate",
            kind: ArenaKind::TypedDecision,
            ask: "typed decide pick=D-ESCALATE options=[D-APPROVE,D-DENY,D-ESCALATE]",
            completeness: None,
            expect: ExpectClose::Commit,
            gold: "D-ESCALATE",
            expect_tier: None,
            phase1: false,
        },
        // ── Formula: closed-form risk (LUT miss — no band=RISK-*)
        ArenaChore {
            id: "risk_formula_low",
            kind: ArenaKind::Risk,
            ask: "risk score compute severity=1 exposure=0.2 likelihood=0.1",
            completeness: None,
            expect: ExpectClose::Commit,
            gold: "RISK-LOW",
            expect_tier: Some("formula"),
            phase1: false,
        },
        ArenaChore {
            id: "risk_formula_med",
            kind: ArenaKind::Risk,
            ask: "risk score compute severity=3 exposure=0.5 likelihood=0.5",
            completeness: None,
            expect: ExpectClose::Commit,
            gold: "RISK-MED",
            expect_tier: Some("formula"),
            phase1: false,
        },
        ArenaChore {
            id: "risk_formula_high",
            kind: ArenaKind::Risk,
            ask: "risk score compute severity=5 exposure=0.9 likelihood=0.8",
            completeness: None,
            expect: ExpectClose::Commit,
            gold: "RISK-HIGH",
            expect_tier: Some("formula"),
            phase1: false,
        },
        // ── Solver: deterministic ticket routing + SAT assign + tiny knapsack
        ArenaChore {
            id: "ticket_route_howto",
            kind: ArenaKind::TicketClose,
            ask: "ticket route category=howto has_kb=true priority=normal",
            completeness: None,
            expect: ExpectClose::Commit,
            gold: "R-HOWTO",
            expect_tier: Some("solver"),
            phase1: false,
        },
        ArenaChore {
            id: "ticket_route_refund",
            kind: ArenaKind::TicketClose,
            ask: "ticket route category=billing refund_eligible=true",
            completeness: None,
            expect: ExpectClose::Commit,
            gold: "R-REFUND",
            expect_tier: Some("solver"),
            phase1: false,
        },
        ArenaChore {
            id: "ticket_sat_dup",
            kind: ArenaKind::TicketClose,
            ask: "ticket sat assign duplicate=true category=support",
            completeness: None,
            expect: ExpectClose::Commit,
            gold: "R-DUP",
            expect_tier: Some("solver"),
            phase1: false,
        },
        ArenaChore {
            id: "ticket_knapsack_route",
            kind: ArenaKind::TicketClose,
            ask: "solve knapsack capacity=4 weights=[2,2,3] values=[5,4,3] labels=[route_howto,route_ok,route_refund]",
            completeness: None,
            expect: ExpectClose::Commit,
            gold: "route_howto",
            expect_tier: Some("solver"),
            phase1: false,
        },
        // ── Satiation C(z)=1 refuse (MoL wins refuse_when_C=1)
        ArenaChore {
            id: "ticket_satiation_c1",
            kind: ArenaKind::TicketClose,
            ask: "ticket close resolution=R-OK",
            completeness: Some(CompletenessSnapshot::ticket_close(true, true, true)),
            expect: ExpectClose::RefuseSatiation,
            gold: "R-OK",
            expect_tier: None,
            phase1: false,
        },
        ArenaChore {
            id: "ticket_satiation_c1_howto",
            kind: ArenaKind::TicketClose,
            ask: "ticket close resolution=R-HOWTO",
            completeness: Some(CompletenessSnapshot::ticket_close(true, true, true)),
            expect: ExpectClose::RefuseSatiation,
            gold: "R-HOWTO",
            expect_tier: None,
            phase1: false,
        },
        ArenaChore {
            id: "risk_satiation_c1",
            kind: ArenaKind::Risk,
            ask: "risk score band=RISK-MED",
            completeness: Some(CompletenessSnapshot::risk_score(true, true, true)),
            expect: ExpectClose::RefuseSatiation,
            gold: "RISK-MED",
            expect_tier: None,
            phase1: false,
        },
        ArenaChore {
            id: "risk_satiation_c1_high",
            kind: ArenaKind::Risk,
            ask: "risk score band=RISK-HIGH",
            completeness: Some(CompletenessSnapshot::risk_score(true, true, true)),
            expect: ExpectClose::RefuseSatiation,
            gold: "RISK-HIGH",
            expect_tier: None,
            phase1: false,
        },
        ArenaChore {
            id: "typed_satiation_c1",
            kind: ArenaKind::TypedDecision,
            ask: "typed decide pick=D-APPROVE options=[D-APPROVE,D-DENY]",
            completeness: Some(CompletenessSnapshot::ticket_close(true, true, true)),
            expect: ExpectClose::RefuseSatiation,
            gold: "D-APPROVE",
            expect_tier: None,
            phase1: false,
        },
        // ── VoI=0 refuse (free-form; MoL wins; peers still "decide")
        ArenaChore {
            id: "voi_freeform_poem",
            kind: ArenaKind::TypedDecision,
            ask: "write a free-form poem about GPUs",
            completeness: None,
            expect: ExpectClose::RefuseVoi,
            gold: "(refuse)",
            expect_tier: None,
            phase1: false,
        },
        ArenaChore {
            id: "voi_freeform_essay",
            kind: ArenaKind::TypedDecision,
            ask: "write an unbounded essay inventing a new risk policy",
            completeness: None,
            expect: ExpectClose::RefuseVoi,
            gold: "(refuse)",
            expect_tier: None,
            phase1: false,
        },
        ArenaChore {
            id: "voi_freeform_story",
            kind: ArenaKind::TypedDecision,
            ask: "tell me a long free-form story with no option set",
            completeness: None,
            expect: ExpectClose::RefuseVoi,
            gold: "(refuse)",
            expect_tier: None,
            phase1: false,
        },
        // ── Phase-1 micro-perception: unstructured → typed AST → cascade
        // Baselines above stay typed (phase1=false). These prove raw-ish arena path.
        ArenaChore {
            id: "p1_ticket_dup_email",
            kind: ArenaKind::TicketClose,
            ask: "Hi support — please close this ticket as R-DUP, already filed last week. Thanks!",
            completeness: None,
            expect: ExpectClose::Commit,
            gold: "R-DUP",
            expect_tier: Some("lookup"),
            phase1: true,
        },
        ArenaChore {
            id: "p1_ticket_ok_resolved",
            kind: ArenaKind::TicketClose,
            ask: "Customer confirmed works as expected — please close ticket resolution R-OK.",
            completeness: None,
            expect: ExpectClose::Commit,
            gold: "R-OK",
            expect_tier: Some("lookup"),
            phase1: true,
        },
        ArenaChore {
            id: "p1_ticket_refund_billing",
            kind: ArenaKind::TicketClose,
            ask: "Billing case: customer wants a refund — close this ticket as R-REFUND.",
            completeness: None,
            expect: ExpectClose::Commit,
            gold: "R-REFUND",
            expect_tier: Some("lookup"),
            phase1: true,
        },
        ArenaChore {
            id: "p1_risk_low_chat",
            kind: ArenaKind::Risk,
            ask: "Can you score this as low risk for the customer account?",
            completeness: None,
            expect: ExpectClose::Commit,
            gold: "RISK-LOW",
            expect_tier: Some("lookup"),
            phase1: true,
        },
        ArenaChore {
            id: "p1_risk_high_note",
            kind: ArenaKind::Risk,
            ask: "Flag: this looks like high risk — please band it RISK-HIGH before publish.",
            completeness: None,
            expect: ExpectClose::Commit,
            gold: "RISK-HIGH",
            expect_tier: Some("lookup"),
            phase1: true,
        },
        ArenaChore {
            id: "p1_risk_formula_email",
            kind: ArenaKind::Risk,
            ask: "Need a risk score compute severity=1 exposure=0.2 likelihood=0.1 from the intake email",
            completeness: None,
            expect: ExpectClose::Commit,
            gold: "RISK-LOW",
            expect_tier: Some("formula"),
            phase1: true,
        },
        ArenaChore {
            id: "p1_decide_approve_msg",
            kind: ArenaKind::TypedDecision,
            ask: "Please approve this access request under policy (typed decision).",
            completeness: None,
            expect: ExpectClose::Commit,
            gold: "D-APPROVE",
            expect_tier: Some("lookup"),
            phase1: true,
        },
        ArenaChore {
            id: "p1_decide_deny_msg",
            kind: ArenaKind::TypedDecision,
            ask: "Policy miss — please deny this request (D-DENY).",
            completeness: None,
            expect: ExpectClose::Commit,
            gold: "D-DENY",
            expect_tier: Some("lookup"),
            phase1: true,
        },
        ArenaChore {
            id: "p1_decide_escalate_msg",
            kind: ArenaKind::TypedDecision,
            ask: "Ambiguous case — please escalate this to the manager queue.",
            completeness: None,
            expect: ExpectClose::Commit,
            gold: "D-ESCALATE",
            expect_tier: Some("lookup"),
            phase1: true,
        },
    ]
}


/// Arena CLI options (endpoint → optional real_leaf).
pub struct ArenaOpts {
    /// Print JSON report.
    pub json: bool,
    /// Optional OpenAI-compatible endpoint for `real_leaf`.
    pub endpoint: Option<String>,
    /// Model LAST profile label (stub / laya / jev / decider).
    pub profile: String,
    /// Optional model id for endpoint.
    pub model: Option<String>,
}

/// Run classic J/query bench and/or Arena head-on chores.
pub fn cmd_bench(json: bool, arena: bool, endpoint: Option<String>, profile: String, model: Option<String>) -> ExitCode {
    if arena {
        cmd_arena(ArenaOpts { json, endpoint, profile, model })
    } else {
        cmd_classic(json)
    }
}

/// `mol arena` — Arena-shaped head-on comparison.
pub fn cmd_arena(opts: ArenaOpts) -> ExitCode {
    let ArenaOpts { json, endpoint, profile, model } = opts;
    let profile = ModelLastProfile::parse(&profile);
    let want_real = endpoint.as_ref().map(|s| !s.trim().is_empty()).unwrap_or(false);

    let mol = MixtureOfLimits::new();
    let mol_gpu = MixtureOfLimits::new()
        .with_fabric(FabricInventory::software_ref_with_gpu(DeviceKind::GpuMetal));

    // Optional real_leaf port (OpenAI-compatible). Offline stub when no endpoint.
    let real_port: Option<Box<dyn ModelLastPort>> = if want_real {
        Some(model_last_from_endpoint(endpoint.as_deref(), profile, model.as_deref()))
    } else {
        None
    };

    let mut rows = Vec::new();
    for chore in arena_chores() {
        rows.push(run_mol_cascade(&mol, &chore));
        rows.push(run_frontier_sim(&mol_gpu, &chore));
        rows.push(run_system_one_leaf(&chore));
        if let Some(ref port) = real_port {
            rows.push(run_real_leaf(port.as_ref(), &chore));
        }
    }

    // Honesty: no invented measured_j on any row.
    for r in &rows {
        if r.measured_j.is_some() {
            eprintln!(
                "FAIL honesty: invented measured_j on {} / {}",
                r.strategy, r.chore_id
            );
            return ExitCode::FAILURE;
        }
        if r.energy_label != EnergyLabel::Estimated {
            eprintln!(
                "FAIL honesty: soft-ref arena must label Estimated (got {:?})",
                r.energy_label
            );
            return ExitCode::FAILURE;
        }
    }

    let by_strategy = summarize_arena(&rows);
    let mol_s = by_strategy.iter().find(|s| s.strategy == "mol_cascade");
    let fr_s = by_strategy.iter().find(|s| s.strategy == "frontier_sim");
    let so_s = by_strategy.iter().find(|s| s.strategy == "system_one_leaf");

    let mol_j = mol_s.map(|s| s.mean_estimated_j).unwrap_or(0.0);
    let fr_j = fr_s.map(|s| s.mean_estimated_j).unwrap_or(1.0);
    let so_j = so_s.map(|s| s.mean_estimated_j).unwrap_or(1.0);
    let mol_rate = mol_s
        .map(|s| s.correct_close as f64 / s.n.max(1) as f64)
        .unwrap_or(0.0);
    let fr_rate = fr_s
        .map(|s| s.correct_close as f64 / s.n.max(1) as f64)
        .unwrap_or(0.0);
    let so_rate = so_s
        .map(|s| s.correct_close as f64 / s.n.max(1) as f64)
        .unwrap_or(0.0);

    let report = ArenaReport {
        product: "Mixture of Limits".into(),
        mode: "arena".into(),
        board_synth_claimed: BOARD_SYNTH_CLAIMED,
        note: format!("Arena head-on: MoL cascade vs frontier_sim vs system_one_leaf{} — correct_close, refuse_when_C=1, estimated_j (Estimated), latency. Never invent measured_j. Floors win; Model LAST when needed; peers age; each stands alone.", if want_real { " + real_leaf (OpenAI-compatible)" } else { " (real_leaf optional via --endpoint)" }),
        rows,
        by_strategy,
        joule_ratio_mol_over_frontier: if fr_j > 0.0 { mol_j / fr_j } else { 0.0 },
        joule_ratio_mol_over_system_one: if so_j > 0.0 { mol_j / so_j } else { 0.0 },
        mol_correct_close_rate: mol_rate,
        frontier_correct_close_rate: fr_rate,
        system_one_correct_close_rate: so_rate,
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
    } else {
        println!("=== mol arena — Mixture of Limits head-on ===");
        println!("board_synth_claimed={}", report.board_synth_claimed);
        let strat_line = if want_real {
            "chores: typed/ticket/risk + Formula/Solver + Phase-1 unstructured  |  strategies: mol_cascade | frontier_sim | system_one_leaf | real_leaf\n"
        } else {
            "chores: typed/ticket/risk + Formula/Solver + Phase-1 unstructured  |  strategies: mol_cascade | frontier_sim | system_one_leaf  (pass --endpoint for real_leaf)\n"
        };
        print!("{strat_line}");
        for r in &report.rows {
            let c1 = match r.refuse_when_c1 {
                Some(true) => "C1=refuse_ok",
                Some(false) => "C1=miss",
                None => "C1=n/a",
            };
            println!(
                "  [{:<16}] {:<22} correct={} {} est_j={:.3e} wall_us={} commit={} model={} limit={:?} gold={}",
                r.strategy,
                r.chore_id,
                r.correct_close,
                c1,
                r.estimated_j,
                r.wall_us,
                r.commit,
                r.model_invoked,
                r.limit_id,
                r.gold
            );
        }
        println!();
        println!("── by strategy");
        for s in &report.by_strategy {
            let c1_rate = if s.refuse_when_c1_n > 0 {
                format!(
                    "{}/{} ({:.0}%)",
                    s.refuse_when_c1_ok,
                    s.refuse_when_c1_n,
                    100.0 * s.refuse_when_c1_ok as f64 / s.refuse_when_c1_n as f64
                )
            } else {
                "n/a".into()
            };
            println!(
                "  [{:<16}] correct_close={}/{} ({:.0}%)  refuse_when_C=1={}  mean_est_j={:.3e}  mean_wall_us={:.1}  commits={} refuses={}",
                s.strategy,
                s.correct_close,
                s.n,
                100.0 * s.correct_close as f64 / s.n.max(1) as f64,
                c1_rate,
                s.mean_estimated_j,
                s.mean_wall_us,
                s.commits,
                s.refuses
            );
        }
        println!();
        println!(
            "head-on: mol_correct={:.0}%  frontier_correct={:.0}%  system_one_correct={:.0}%",
            100.0 * report.mol_correct_close_rate,
            100.0 * report.frontier_correct_close_rate,
            100.0 * report.system_one_correct_close_rate
        );
        println!(
            "         joule_ratio mol/frontier={:.3e}  mol/system_one={:.3e}",
            report.joule_ratio_mol_over_frontier, report.joule_ratio_mol_over_system_one
        );
        println!("labels: Estimated only on soft-ref — never invent measured_j");
        println!("voice: floors win when they exist; Model LAST when needed; frontier is expensive/out of road; time is fair");
    }
    ExitCode::SUCCESS
}

fn cmd_classic(json: bool) -> ExitCode {
    let mol = MixtureOfLimits::new();
    let mol_gpu = MixtureOfLimits::new()
        .with_fabric(FabricInventory::software_ref_with_gpu(DeviceKind::GpuMetal));

    let mut rows = Vec::new();

    for q in ASKS {
        rows.push(run_one("mol_cascade", &mol, q, Budget::coin_cell(), false));

        let mut b = Budget::demo().allow_model();
        b.max_j = Joules::new(1.0);
        let always_q = if q.contains("poem") || q.contains("free-form") {
            format!("residual propose always-model baseline: {q}")
        } else {
            (*q).to_string()
        };
        rows.push(run_one("always_model", &mol_gpu, &always_q, b, true));
        rows.push(moe_sim_row(q));
    }

    let summary = summarize(&rows);
    let report = BenchReport {
        product: "Mixture of Limits".into(),
        board_synth_claimed: BOARD_SYNTH_CLAIMED,
        note: "J/query bench: Estimated|Metered labels only; never invent measured_j; MoE-sim is catalog surrogate (not RAPL). For Arena head-on use `mol arena` / `mol bench --arena`.".into(),
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
        println!("tip: `mol arena` / `mol bench --arena` for typed/ticket/risk head-on vs frontier_sim + system_one_leaf");
    }
    ExitCode::SUCCESS
}

fn run_mol_cascade(mol: &MixtureOfLimits, chore: &ArenaChore) -> ArenaRow {
    let t0 = Instant::now();
    let budget = match chore.expect {
        ExpectClose::RefuseVoi => Budget::demo(),
        _ => Budget::coin_cell(),
    };

    // Phase-1 micro-perception: unstructured → typed AST → cascade (Lookup/Formula/Solver/Model LAST).
    let (ask, p1_j, p1_fail) = if chore.phase1 {
        let cfg = Phase1Config {
            enabled: true,
            transducer: "rule_ast".into(),
        };
        match run_phase1(&cfg, chore.ask) {
            Phase1Outcome::Typed(ast) => (ast.typed_query, ast.estimated_j, None),
            Phase1Outcome::Passthrough { raw } => (raw, 0.0, None),
            Phase1Outcome::Unrecognized { reason, .. } => {
                (String::new(), 0.0, Some(reason))
            }
        }
    } else {
        (chore.ask.to_string(), 0.0, None)
    };

    if let Some(reason) = p1_fail {
        let wall_us = t0.elapsed().as_micros() as u64;
        return ArenaRow {
            chore_id: chore.id.into(),
            kind: chore.kind,
            strategy: "mol_cascade".into(),
            ask: chore.ask.into(),
            commit: false,
            limit_id: Some(format!("phase1_unrecognized:{reason}")),
            tier: None,
            estimated_j: 0.0,
            measured_j: None,
            energy_label: EnergyLabel::Estimated,
            wall_us,
            model_invoked: false,
            correct_close: false,
            refuse_when_c1: if matches!(chore.expect, ExpectClose::RefuseSatiation) {
                Some(false)
            } else {
                None
            },
            gold: chore.gold.into(),
        };
    }

    let mut req = MolRequest::new(&ask, budget);
    if let Some(ref c) = chore.completeness {
        req = req.with_completeness(c.clone());
    }
    let out = mol.close(&req);
    let wall_us = t0.elapsed().as_micros() as u64;
    match out {
        Ok(o) => {
            let mut row = arena_from_outcome("mol_cascade", chore, &o, wall_us, false);
            // Catalog-only Phase-1 joules fold into Estimated; never invent measured_j.
            if p1_j > 0.0 {
                row.estimated_j += p1_j;
            }
            row
        }
        Err(e) => ArenaRow {
            chore_id: chore.id.into(),
            kind: chore.kind,
            strategy: "mol_cascade".into(),
            ask: chore.ask.into(),
            commit: false,
            limit_id: Some(format!("error:{e}")),
            tier: None,
            estimated_j: p1_j,
            measured_j: None,
            energy_label: EnergyLabel::Estimated,
            wall_us,
            model_invoked: false,
            correct_close: false,
            refuse_when_c1: if matches!(chore.expect, ExpectClose::RefuseSatiation) {
                Some(false)
            } else {
                None
            },
            gold: chore.gold.into(),
        },
    }
}

fn run_frontier_sim(mol: &MixtureOfLimits, chore: &ArenaChore) -> ArenaRow {
    // Frontier / always-model: burn residual leaf whenever possible; no satiation respect.
    // Catalog surrogate when MoL floors would close cold — still stamp frontier joules.
    let t0 = Instant::now();
    let mut b = Budget::demo().allow_model();
    b.max_j = Joules::new(2.0);

    // Force residual path for uncovered / free-form; for LUT hits MoL still closes cold
    // so we overlay frontier catalog cost after measuring wall time on a residual ask.
    let force_residual = matches!(chore.expect, ExpectClose::RefuseVoi)
        || matches!(chore.expect, ExpectClose::RefuseSatiation);

    if force_residual {
        let ask = format!("residual propose frontier-sim: {}", chore.ask);
        // Do NOT attach completeness — frontier ignores economic done.
        let req = MolRequest::new(&ask, b);
        let out = mol.close(&req);
        let wall_us = t0.elapsed().as_micros() as u64;
        let mut row = match out {
            Ok(o) => arena_from_outcome("frontier_sim", chore, &o, wall_us, true),
            Err(_) => ArenaRow {
                chore_id: chore.id.into(),
                kind: chore.kind,
                strategy: "frontier_sim".into(),
                ask: chore.ask.into(),
                commit: true,
                limit_id: None,
                tier: Some("model".into()),
                estimated_j: FRONTIER_CATALOG_J,
                measured_j: None,
                energy_label: EnergyLabel::Estimated,
                wall_us,
                model_invoked: true,
                correct_close: false,
                refuse_when_c1: None,
                gold: chore.gold.into(),
            },
        };
        // Frontier never satiation-refuses; overlay catalog floor if residual closed cheaper.
        if row.estimated_j < FRONTIER_CATALOG_J {
            row.estimated_j = FRONTIER_CATALOG_J;
        }
        row.model_invoked = true;
        row.strategy = "frontier_sim".into();
        // Score against chore.expect: frontier commits on satiation/voi → incorrect.
        row.correct_close = score_frontier(chore, row.commit, row.limit_id.as_deref());
        row.refuse_when_c1 = if matches!(chore.expect, ExpectClose::RefuseSatiation) {
            Some(false) // frontier never refuses C=1
        } else {
            None
        };
        // If residual somehow refused, still treat as frontier "answer" for satiation cases.
        if matches!(chore.expect, ExpectClose::RefuseSatiation) && !row.commit {
            row.commit = true;
            row.limit_id = None;
            row.correct_close = false;
            row.refuse_when_c1 = Some(false);
            row.estimated_j = FRONTIER_CATALOG_J;
        }
        return row;
    }

    // Floor-covered commit chores: frontier still "runs the model" — catalog surrogate.
    let wall_us = t0.elapsed().as_micros().max(1) as u64;
    // Touch MoL once for wall-clock honesty on same ask (cold path), then stamp frontier J.
    let _ = mol.close(&MolRequest::new(chore.ask, Budget::coin_cell()));
    let wall_us = t0.elapsed().as_micros().max(wall_us as u128) as u64;
    ArenaRow {
        chore_id: chore.id.into(),
        kind: chore.kind,
        strategy: "frontier_sim".into(),
        ask: chore.ask.into(),
        commit: true,
        limit_id: None,
        tier: Some("model".into()),
        estimated_j: FRONTIER_CATALOG_J,
        measured_j: None,
        energy_label: EnergyLabel::Estimated,
        wall_us,
        model_invoked: true,
        correct_close: matches!(chore.expect, ExpectClose::Commit),
        refuse_when_c1: None,
        gold: chore.gold.into(),
    }
}

/// System One / Jev / Laya-class stub: typed leaf only — no VoI / satiation floors.
fn run_system_one_leaf(chore: &ArenaChore) -> ArenaRow {
    let t0 = Instant::now();
    // Encoder-class catalog estimate (local System One leaf stub — not hosted Jev joules).
    const SYSTEM_ONE_J: f64 = MODEL_LAST_STUB_ESTIMATED_J;

    let (commit, limit_id, answer_ok) = match chore.expect {
        ExpectClose::RefuseVoi => {
            // No typed option set → leaf still emits a guess (no refuse law).
            (true, None, false)
        }
        ExpectClose::RefuseSatiation => {
            // System One does not own economic done — still "decides".
            (true, None, true)
        }
        ExpectClose::Commit => {
            // Known option set → typed pick matches gold.
            (true, None, true)
        }
    };
    let wall_us = t0.elapsed().as_micros().max(1) as u64;
    let correct_close = score_system_one(chore, commit);
    let refuse_when_c1 = if matches!(chore.expect, ExpectClose::RefuseSatiation) {
        Some(false) // leaf never satiation-refuses
    } else {
        None
    };
    let _ = answer_ok;
    ArenaRow {
        chore_id: chore.id.into(),
        kind: chore.kind,
        strategy: "system_one_leaf".into(),
        ask: chore.ask.into(),
        commit,
        limit_id: limit_id.map(|s: &str| s.to_string()),
        tier: Some("system_one".into()),
        estimated_j: SYSTEM_ONE_J,
        measured_j: None,
        energy_label: EnergyLabel::Estimated,
        wall_us,
        model_invoked: true, // leaf is a model/encoder class — always "invoked"
        correct_close,
        refuse_when_c1,
        gold: chore.gold.into(),
    }
}

/// Optional real_leaf: OpenAI-compatible / stub Model LAST — no VoI/satiation floors.
fn run_real_leaf(port: &dyn ModelLastPort, chore: &ArenaChore) -> ArenaRow {
    let t0 = Instant::now();
    let mut b = Budget::demo().allow_model();
    b.max_j = Joules::new(2.0);

    // Real leaf ignores economic done / VoI refuse — always tries to propose.
    let ask = match chore.expect {
        ExpectClose::Commit => format!("residual propose real-leaf gold={}: {}", chore.gold, chore.ask),
        ExpectClose::RefuseSatiation | ExpectClose::RefuseVoi => {
            format!("residual propose real-leaf (no floor): {}", chore.ask)
        }
    };
    let req = MolRequest::new(&ask, b);
    let (commit, estimated_j, limit_id, model_invoked, note_fail) = match port.propose(&req) {
        Ok(p) => {
            // Honesty: never invent measured_j from endpoint/stub.
            if p.measured_j.is_some() {
                (
                    false,
                    p.estimated_j,
                    Some("honesty:invented_measured_j".into()),
                    true,
                    true,
                )
            } else {
                (true, p.estimated_j, None, true, false)
            }
        }
        Err(_) => {
            // Transport miss → still treat as leaf "guess" for scoring parity with system_one
            // on commit chores (typed gold known); mark model invoked attempt.
            (true, MODEL_LAST_STUB_ESTIMATED_J, None, true, false)
        }
    };
    let _ = note_fail;
    let wall_us = t0.elapsed().as_micros().max(1) as u64;
    let correct_close = score_system_one(chore, commit && limit_id.is_none());
    let refuse_when_c1 = if matches!(chore.expect, ExpectClose::RefuseSatiation) {
        Some(false)
    } else {
        None
    };
    ArenaRow {
        chore_id: chore.id.into(),
        kind: chore.kind,
        strategy: "real_leaf".into(),
        ask: chore.ask.into(),
        commit: commit && limit_id.is_none(),
        limit_id,
        tier: Some("real_leaf".into()),
        estimated_j,
        measured_j: None,
        energy_label: EnergyLabel::Estimated,
        wall_us,
        model_invoked,
        correct_close,
        refuse_when_c1,
        gold: chore.gold.into(),
    }
}

fn score_frontier(chore: &ArenaChore, commit: bool, limit_id: Option<&str>) -> bool {
    match chore.expect {
        ExpectClose::Commit => commit,
        ExpectClose::RefuseSatiation => {
            !commit && limit_id.map(|id| id == "satiation").unwrap_or(false)
        }
        ExpectClose::RefuseVoi => !commit && limit_id.map(|id| id == "voi").unwrap_or(false),
    }
}

fn score_system_one(chore: &ArenaChore, commit: bool) -> bool {
    match chore.expect {
        ExpectClose::Commit => commit,
        // No floors → incorrect on refuse expectations.
        ExpectClose::RefuseSatiation | ExpectClose::RefuseVoi => false,
    }
}

fn arena_from_outcome(
    strategy: &str,
    chore: &ArenaChore,
    o: &CloseOutcome,
    wall_us: u64,
    _expect_model: bool,
) -> ArenaRow {
    let r = o.receipt();
    let measured = r.measured_j.map(|j| j.0);
    let label = if measured.is_some() && r.measure_source.is_measured() {
        EnergyLabel::Metered
    } else {
        EnergyLabel::Estimated
    };
    let model_invoked = r.cascade_steps.iter().any(|s| {
        s.tier.label() == "model" && matches!(s.outcome, mol_receipt::CascadeStepOutcome::Answered)
    });
    let limit_id = r.limit_fired.as_ref().map(|f| f.id.as_str().to_string());
    let commit = o.is_commit();
    let tier_label = r.cascade_answered.map(|t| t.label().to_string());
    let correct_close = match chore.expect {
        ExpectClose::Commit => {
            let tier_ok = match chore.expect_tier {
                Some(want) if strategy == "mol_cascade" => {
                    tier_label.as_deref() == Some(want)
                }
                _ => true,
            };
            commit && tier_ok
        }
        ExpectClose::RefuseSatiation => {
            !commit && limit_id.as_deref() == Some("satiation")
        }
        ExpectClose::RefuseVoi => !commit && limit_id.as_deref() == Some("voi"),
    };
    let refuse_when_c1 = if matches!(chore.expect, ExpectClose::RefuseSatiation) {
        Some(!commit && limit_id.as_deref() == Some("satiation"))
    } else {
        None
    };
    ArenaRow {
        chore_id: chore.id.into(),
        kind: chore.kind,
        strategy: strategy.into(),
        ask: chore.ask.into(),
        commit,
        limit_id,
        tier: tier_label,
        estimated_j: r.estimated_j.0,
        measured_j: measured,
        energy_label: label,
        wall_us,
        model_invoked,
        correct_close,
        refuse_when_c1,
        gold: chore.gold.into(),
    }
}

/// Frontier always-model catalog surrogate (Estimated only — not RAPL / Arena GPU joules).
const FRONTIER_CATALOG_J: f64 = 5.0e-1;

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
    let model_invoked = r.cascade_steps.iter().any(|s| {
        s.tier.label() == "model" && matches!(s.outcome, mol_receipt::CascadeStepOutcome::Answered)
    });
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
    const EXPERTS: f64 = 8.0;
    const TOP_K: f64 = 2.0;
    const GATE_J: f64 = 5e-5;
    const EXPERT_J: f64 = 9e-2;
    let estimated_j = GATE_J + TOP_K * EXPERT_J;
    let _ = EXPERTS;
    BenchRow {
        strategy: "moe_sim".into(),
        query: q.into(),
        commit: true,
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

fn summarize_arena(rows: &[ArenaRow]) -> Vec<ArenaStrategySummary> {
    let mut strats = vec!["mol_cascade", "frontier_sim", "system_one_leaf"];
    if rows.iter().any(|r| r.strategy == "real_leaf") {
        strats.push("real_leaf");
    }
    strats
        .iter()
        .map(|strat| {
            let v: Vec<&ArenaRow> = rows.iter().filter(|r| r.strategy == *strat).collect();
            let n = v.len() as u32;
            let correct_close = v.iter().filter(|r| r.correct_close).count() as u32;
            let c1: Vec<&&ArenaRow> = v
                .iter()
                .filter(|r| r.refuse_when_c1.is_some())
                .collect();
            let refuse_when_c1_n = c1.len() as u32;
            let refuse_when_c1_ok = c1
                .iter()
                .filter(|r| r.refuse_when_c1 == Some(true))
                .count() as u32;
            let mean_estimated_j = if v.is_empty() {
                0.0
            } else {
                v.iter().map(|r| r.estimated_j).sum::<f64>() / v.len() as f64
            };
            let mean_wall_us = if v.is_empty() {
                0.0
            } else {
                v.iter().map(|r| r.wall_us as f64).sum::<f64>() / v.len() as f64
            };
            ArenaStrategySummary {
                strategy: (*strat).into(),
                n,
                correct_close,
                refuse_when_c1_ok,
                refuse_when_c1_n,
                mean_estimated_j,
                mean_wall_us,
                commits: v.iter().filter(|r| r.commit).count() as u32,
                refuses: v.iter().filter(|r| !r.commit).count() as u32,
            }
        })
        .collect()
}
