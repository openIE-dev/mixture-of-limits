//! `mol` CLI — Mixture of Limits operator surface.

mod prove;
mod run;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use mol_adapters::{
    EfaCertificatePort, EfaProposal, JouleDbCascadePort, KlereSettleJob, KlereSettlePort,
    OpenIeRuntimePort, StubEfaCertificate, StubJouleDbCascade, StubKlereSettle, StubOpenIeRuntime,
    StubWcaCommit, WcaCommitPort,
};
use mol_automate::{Act, ActKind, AgentLoop, AutomateGate, Capability, CapabilitySet, EcosystemCertify, EcosystemCertifyConfig};
use mol_core::{
    detect_adapter_probes, detect_inventory, inventory_for_schedule, measure_energy_window,
    meter_status_line, probe_meter_capability, schedule_fabric, AdapterBackendHint, Budget,
    CascadeTier, DeviceKind, FabricInventory, FailClosedPolicy, Joules, MolRequest, MuCatalog,
    QueryKind, BOARD_SYNTH_CLAIMED, DETECT_HONESTY_NOTE, ENERGY_METER_ENABLED,
    FABRIC_DETECT_ENABLED, MACOS_METER_HELP, METER_HONESTY_NOTE,
    VERSION as CORE_V,
};
use std::time::Duration;
use mol_limits::{CloseOutcome, MixtureOfLimits};
use mol_receipt::{
    replay_transcript, sign_receipt, verify_receipt_integrity, verify_signature, CloseTranscript,
    MolReceipt, ReplayCloseOutcome, HMAC_SECRET_ENV,
};

#[derive(Parser, Debug)]
#[command(
    name = "mol",
    version = env!("CARGO_PKG_VERSION"),
    about = "Mixture of Limits — floors where more bits stop buying outcomes",
    long_about = "Navigation law over the Periodic Stack. Cascade: Lookup → Formula → Solver/settle → Model LAST.\n\
                  MoL owns close: propose → certify → commit|refuse → receipt."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Ask a query through MixtureOfLimits + cascade.
    Ask {
        /// Query text.
        query: String,
        /// Max joules (estimated).
        #[arg(long, default_value_t = 1e-3)]
        max_j: f64,
        /// Soft latency ms.
        #[arg(long)]
        max_latency_ms: Option<u64>,
        /// Allow demoted model leaf (still a stub — no weights).
        #[arg(long, default_value_t = false)]
        allow_model: bool,
        /// Run close (route + EFA/WCA certify) instead of route alone.
        #[arg(long, default_value_t = false)]
        close: bool,
        /// Print full JSON receipt.
        #[arg(long, default_value_t = false)]
        receipt_json: bool,
        /// Attach OS energy meter sample around close (feature `energy-meter`; never invents).
        #[arg(long, default_value_t = false)]
        meter: bool,
        /// Meter sample window milliseconds (with `--meter` / `--meter-required`).
        #[arg(long, default_value_t = 10)]
        meter_ms: u64,
        /// Fail-closed: sample meter, require real package measured_j to COMMIT (never invent).
        #[arg(long, default_value_t = false)]
        meter_required: bool,
        /// Use live wgpu fabric inventory (`fabric-detect`) instead of soft-ref.
        /// Formula still stamps Cpu; model residual stamps Gpu* or refuses.
        #[arg(long, default_value_t = false)]
        detect: bool,
    },
    /// Explain which cascade tiers would be tried and their surrogate joules.
    ExplainCascade {
        /// Query text.
        query: String,
    },
    /// List named limits in the default registry.
    Limits,
    /// Verify a receipt JSON file (honesty + optional HMAC).
    ReceiptVerify {
        /// Path to receipt JSON.
        path: PathBuf,
        /// HMAC secret (else env MOL_RECEIPT_SECRET).
        #[arg(long)]
        secret: Option<String>,
    },
    /// End-to-end demo: formula/lookup close; VoI refuse; settle/certificate refuse; honesty.
    Demo,
    /// Replay a close transcript JSONL; checks commit/refuse+limit without model.
    Replay {
        /// Path to JSONL transcript (one CloseTranscriptEntry per line).
        path: PathBuf,
        /// Write a fresh transcript from built-in sample asks to PATH then replay.
        #[arg(long, default_value_t = false)]
        record_samples: bool,
    },
    /// Run PLAN.md proof criteria; exit 0 iff all VERIFIED.
    Prove,
    /// Run a declarative Mixture of Limits chore (mol.yaml product surface).
    Run {
        /// Path to mol.yaml
        #[arg(long)]
        chore: PathBuf,
        /// Allow demoted model leaf (still stub — no weights).
        #[arg(long, default_value_t = false)]
        allow_model: bool,
        /// Print full JSON receipt.
        #[arg(long, default_value_t = false)]
        receipt_json: bool,
    },
    /// Thin agent mailbox loop: Goal/Message/Act → MoL close → transcript.
    Agent {
        /// Extra goal text(s) to post (can repeat). Demo goals also seeded unless --no-demo.
        #[arg(long = "goal")]
        goals: Vec<String>,
        /// Optional message body posted as user→mol (formula/cite path).
        #[arg(long)]
        message: Option<String>,
        /// Skip seeding demo goals (only --goal / --message).
        #[arg(long, default_value_t = false)]
        no_demo: bool,
    },
    /// Bitemporal state+memory demo: capability refuse → write via close COMMIT → recall cite.
    Memory,
    /// Print fabric inventory: soft-ref (always) and optional wgpu detect.
    Fabric {
        /// Attempt live wgpu adapter probe (requires `--features fabric-detect`).
        #[arg(long, default_value_t = false)]
        detect: bool,
        /// Also print mock Metal/Vulkan/WebGPU mapping (no live GPU).
        #[arg(long, default_value_t = false)]
        mock: bool,
        /// Print inventory JSON.
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    /// Probe OS energy meter (RAPL/powercap, macOS powermetrics). Never invents joules.
    Meter {
        /// Sample interval milliseconds (0 = capability probe only).
        #[arg(long, default_value_t = 0)]
        sample_ms: u64,
    },
    /// End-to-end ecosystem certify → single receipt (Agent Lane + fabric + WASM + GrantReceipt).
    EcosystemCertify {
        /// Omit GrantReceipt to exercise refuse path (`grant_receipt_required`).
        #[arg(long, default_value_t = false)]
        refuse_without_grant: bool,
        /// Print full JSON receipt.
        #[arg(long, default_value_t = false)]
        receipt_json: bool,
        /// Agent lane session id.
        #[arg(long, default_value = "ecosystem-e2e")]
        session: String,
        /// First add.wasm operand.
        #[arg(long, default_value_t = 2)]
        a: i32,
        /// Second add.wasm operand.
        #[arg(long, default_value_t = 40)]
        b: i32,
        /// Attach live OS energy meter sample (feature `energy-meter`; never invents).
        #[arg(long, default_value_t = false)]
        meter: bool,
        /// Meter sample window milliseconds (with `--meter` / `--meter-required`).
        #[arg(long, default_value_t = 100)]
        meter_ms: u64,
        /// Fail-closed: sample meter, require real package measured_j to COMMIT (never invent).
        #[arg(long, default_value_t = false)]
        meter_required: bool,
        /// Schedule from live wgpu inventory (`fabric-detect`). Soft-ref if the feature is off.
        #[arg(long, default_value_t = false)]
        detect: bool,
        /// Fabric schedule tier: formula (Cpu) or model (cheapest Gpu* or refuse).
        #[arg(long, default_value = "formula")]
        tier: String,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Commands::Ask {
            query,
            max_j,
            max_latency_ms,
            allow_model,
            close,
            receipt_json,
            meter,
            meter_ms,
            meter_required,
            detect,
        } => cmd_ask(
            query,
            max_j,
            max_latency_ms,
            allow_model,
            close,
            receipt_json,
            meter,
            meter_ms,
            meter_required,
            detect,
        ),
        Commands::ExplainCascade { query } => cmd_explain(query),
        Commands::Limits => cmd_limits(),
        Commands::ReceiptVerify { path, secret } => cmd_verify(path, secret),
        Commands::Demo => cmd_demo(),
        Commands::Replay { path, record_samples } => cmd_replay(path, record_samples),
        Commands::Prove => {
            if prove::run_prove() {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Commands::Run {
            chore,
            allow_model,
            receipt_json,
        } => run::cmd_run(chore, allow_model, receipt_json),
        Commands::Agent {
            goals,
            message,
            no_demo,
        } => cmd_agent(!no_demo, goals, message),
        Commands::Memory => cmd_memory(),
        Commands::Fabric { detect, mock, json } => cmd_fabric(detect, mock, json),
        Commands::Meter { sample_ms } => cmd_meter(sample_ms),
        Commands::EcosystemCertify {
            refuse_without_grant,
            receipt_json,
            session,
            a,
            b,
            meter,
            meter_ms,
            meter_required,
            detect,
            tier,
        } => cmd_ecosystem_certify(
            refuse_without_grant,
            receipt_json,
            session,
            a,
            b,
            meter,
            meter_ms,
            meter_required,
            detect,
            tier,
        ),
    }
}

fn budget(max_j: f64, max_latency_ms: Option<u64>, allow_model: bool) -> Budget {
    let mut b = Budget::joules(max_j);
    if let Some(ms) = max_latency_ms {
        b = b.with_latency_ms(ms);
    }
    if allow_model {
        b = b.allow_model();
    }
    b
}

fn print_receipt_honesty(r: &MolReceipt) {
    println!("estimated_j: {} ({:?})", r.estimated_j, r.estimate_kind);
    println!(
        "mu: {} (mu_source={})",
        r.mu.map(|m| format!("{m:.6e}")).unwrap_or_else(|| "None".into()),
        r.mu_source
    );
    if let Some(ratio) = r.landauer_floor_ratio {
        println!("landauer_floor_ratio: {ratio:.6e}");
    }
    println!(
        "measured_j: {} (source={})",
        r.measured_j
            .map(|j| j.to_string())
            .unwrap_or_else(|| "None".into()),
        r.measure_source
    );
    if r.component_measured.is_empty() {
        println!("component_measured: (none)");
    } else {
        for c in &r.component_measured {
            println!(
                "component_measured: {} {:.6e} J ({})",
                c.component, c.joules.0, c.measure_source
            );
        }
    }
    println!("board_synth_claimed: {}", r.board_synth_claimed);
}

fn print_fabric_stamp(r: &MolReceipt) {
    let chosen = r
        .fabric_chosen
        .map(|k| {
            format!(
                "{} backend={}",
                k.label(),
                k.backend_token().unwrap_or("-")
            )
        })
        .unwrap_or_else(|| "none".into());
    let src = r
        .fabric_inventory
        .as_ref()
        .map(|i| i.source.to_string())
        .unwrap_or_else(|| "none".into());
    println!("fabric_chosen: {chosen}  inventory_source={src}");
    for s in r.compute_steps.iter().filter(|s| s.label == "fabric:route") {
        println!(
            "fabric:route fabric_id={} unavailable={} estimated_j={} measured_j={}",
            s.fabric_id.as_deref().unwrap_or("-"),
            s.unavailable_reason.as_deref().unwrap_or("-"),
            s.estimated_j,
            s.measured_j
                .map(|j| j.to_string())
                .unwrap_or_else(|| "None".into())
        );
    }
}

fn cmd_ask(
    query: String,
    max_j: f64,
    max_latency_ms: Option<u64>,
    allow_model: bool,
    close: bool,
    receipt_json: bool,
    meter: bool,
    meter_ms: u64,
    meter_required: bool,
    detect: bool,
) -> ExitCode {
    let mut mol = MixtureOfLimits::new();
    if detect {
        let inv = inventory_for_schedule();
        println!(
            "fabric detect: feature={FABRIC_DETECT_ENABLED} source={} present=[{}] gpu={}",
            inv.source,
            inv.summary(),
            inv.cheapest_gpu()
                .map(|k| format!("{} ({})", k.label(), k.backend_token().unwrap_or("-")))
                .unwrap_or_else(|| "none".into())
        );
        for p in detect_adapter_probes() {
            println!(
                "  adapter name={:?} backend={} device_class={}",
                p.name, p.backend, p.device_class
            );
        }
        mol = mol.with_fabric(inv);
    }
    let attach_meter = meter || meter_required;
    // meter_required implies close so the fail-closed gate can commit|refuse.
    let close = close || meter_required;
    let mut req = MolRequest::new(query, budget(max_j, max_latency_ms, allow_model));
    if meter_required {
        req = req.with_fail_closed(FailClosedPolicy::meter_required());
    }
    let meter_sample = if attach_meter {
        Some(measure_energy_window(Duration::from_millis(meter_ms)))
    } else {
        None
    };
    if let Some(ref sample) = meter_sample {
        if !sample.honesty_ok() {
            eprintln!("meter honesty failure: {}", sample.detail);
            return ExitCode::FAILURE;
        }
        println!(
            "meter: feature={} source={} window_ms={} measured_j={} detail={}",
            ENERGY_METER_ENABLED,
            sample.source.label(),
            sample.window_ms,
            sample
                .measured_j
                .map(|j| format!("{:.6e}", j.0))
                .unwrap_or_else(|| "None".into()),
            sample.detail
        );
        for c in &sample.components {
            println!(
                "  component {} {:.6e} J",
                c.component, c.joules.0
            );
        }
        // Attach before close so meter_required can commit|refuse honestly (never invent).
        req = req.with_meter_sample(sample.clone());
    }
    let (answered, receipt) = if close {
        match mol.close(&req) {
            Ok(CloseOutcome::Commit { receipt, .. }) => (true, receipt),
            Ok(CloseOutcome::Refuse { receipt, .. }) => (false, receipt),
            Err(e) => {
                eprintln!("error: {e}");
                return ExitCode::FAILURE;
            }
        }
    } else {
        match mol.route(&req) {
            Ok(out) => (out.is_answered(), out.receipt().clone()),
            Err(e) => {
                eprintln!("error: {e}");
                return ExitCode::FAILURE;
            }
        }
    };

    if let Some(ans) = receipt.answer.as_ref() {
        println!("answer: {ans}");
    } else {
        println!("answer: (refused)");
    }
    if let Some(tier) = receipt.cascade_answered {
        println!("cascade_answered: {tier}");
    }
    if let Some(z) = receipt.zone {
        println!("zone: {z}");
    }
    if let Some(rc) = receipt.replay_class {
        println!("replay_class: {rc}");
    }
    if !receipt.citation_ids.is_empty() {
        println!("citation_ids: {}", receipt.citation_ids.join(", "));
    }
    if !receipt.composed_from.is_empty() {
        println!("composed_from: {}", receipt.composed_from.join(", "));
    }
    if let Some(syn) = &receipt.synthesis {
        println!(
            "synthesis: kind={} claim_count={} composed_from=[{}]",
            syn.kind,
            syn.claim_count,
            syn.composed_from.join(", ")
        );
    }
    if let Some(floor) = &receipt.limit_fired {
        println!("limit_fired: {} — {}", floor.id, floor.reason);
    }
    print_receipt_honesty(&receipt);
    print_fabric_stamp(&receipt);
    if receipt_json {
        match receipt.to_json() {
            Ok(j) => println!("{j}"),
            Err(e) => {
                eprintln!("receipt json error: {e}");
                return ExitCode::FAILURE;
            }
        }
    }
    if answered {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(2)
    }
}

fn cmd_explain(query: String) -> ExitCode {
    let kind = QueryKind::classify(&query);
    println!("query: {query}");
    println!("classified_kind: {kind:?}");
    println!("board_synth_claimed: {BOARD_SYNTH_CLAIMED}");
    println!();
    println!("cascade gears (cheapest → hottest); E≈θ·μ catalog (mu_source=catalog):");
    for tier in CascadeTier::routing_order() {
        let est = MuCatalog::estimate_tier(tier);
        println!(
            "  {:>8}  zone={}  thermo={}  replay={}  μ={:.3e}  θ·μ≈{:.3e} J",
            tier.label(),
            tier.openie_zone(),
            tier.thermo(),
            tier.default_replay(),
            est.mu,
            est.estimated_j.0
        );
    }
    println!();
    let stub = StubJouleDbCascade;
    let req = MolRequest::new(&query, Budget::demo());
    match stub.explain(&req) {
        Ok(ex) => {
            println!(
                "jouledb offline map: mol_tier={} → jouledb_tier={} ({})",
                ex.mol_tier, ex.jouledb_tier, ex.note
            );
        }
        Err(e) => println!("jouledb map: {e}"),
    }
    let openie = StubOpenIeRuntime;
    if let Ok(r) = openie.ask(&req) {
        println!(
            "openie offline map: zone={} mol_tier={} offline={}",
            r.zone, r.mol_tier, r.offline
        );
    }
    println!();
    println!("model leaf: demoted; returns ModelRefused unless --allow-model");
    ExitCode::SUCCESS
}

fn cmd_limits() -> ExitCode {
    let mol = MixtureOfLimits::new();
    println!("MixtureOfLimits registry (order = evaluation order):");
    for (i, id) in mol.limit_ids().into_iter().enumerate() {
        let lim = &mol.limits[i];
        println!(
            "  {}. {}  kind={}  — {}",
            i + 1,
            id,
            lim.kind,
            lim.description
        );
    }
    ExitCode::SUCCESS
}

fn cmd_verify(path: PathBuf, secret: Option<String>) -> ExitCode {
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("read {}: {e}", path.display());
            return ExitCode::FAILURE;
        }
    };
    let receipt: MolReceipt = match MolReceipt::from_json(&text) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("parse: {e}");
            return ExitCode::FAILURE;
        }
    };
    if let Err(e) = verify_receipt_integrity(&receipt) {
        eprintln!("integrity FAIL: {e}");
        return ExitCode::FAILURE;
    }
    let sec = secret
        .map(|s| s.into_bytes())
        .or_else(|| std::env::var(HMAC_SECRET_ENV).ok().map(|s| s.into_bytes()));
    if let Some(secret) = sec {
        if receipt.signature_hex.is_some() {
            if let Err(e) = verify_signature(&receipt, &secret) {
                eprintln!("HMAC FAIL: {e}");
                return ExitCode::FAILURE;
            }
            println!("HMAC: ok");
        } else {
            println!("HMAC: no signature on receipt (integrity ok)");
        }
    } else {
        println!("HMAC: skipped (no secret)");
    }
    println!("integrity: ok");
    println!("id: {}", receipt.id);
    println!("board_synth_claimed: {}", receipt.board_synth_claimed);
    ExitCode::SUCCESS
}

fn cmd_demo() -> ExitCode {
    println!("=== Mixture of Limits demo (v{CORE_V}) — path to MoL owns close ===");
    println!("Cascade: Lookup → Formula → Z2 retrieve → Z1 compose → Solver/settle → Model LAST (demoted).");
    println!("Close: propose → certify (EFA+WCA) → commit|refuse → receipt.\n");

    let mol = MixtureOfLimits::new();

    // 1) Cheap closes
    println!("── 1. Cheap closes (formula / lookup / solver)");
    for q in [
        "convert 100 celsius to fahrenheit",
        "landauer joules per bit at 300 kelvin",
        "rest energy for mass 1 kg (E=mc2)",
        "solve 2x2 [[2,1],[5,3]] [8,19]",
    ] {
        demo_close(&mol, q, Budget::coin_cell());
    }

    // 1b) Z2 retrieve+cite
    println!("── 1b. Z2 retrieve+cite (RetrievedCited) + unknown refuse");
    demo_close(&mol, "what is the landauer principle", Budget::demo());
    demo_close(&mol, "cite the mol law", Budget::demo());
    demo_close(
        &mol,
        "what is the capital of Atlantis xyzzy-unknown-claim",
        Budget::demo(),
    );

    // 1c) Z1 compose/synthesis from ≥2 cited claims
    println!("── 1c. Z1 compose/synthesis (Composed) + missing refuse");
    demo_close(
        &mol,
        "compose claim:landauer.principle and claim:mol.law",
        Budget::demo(),
    );
    demo_close(&mol, "compose landauer principle with mol law", Budget::demo());
    demo_close(
        &mol,
        "compose claim:landauer.principle and claim:does.not.exist",
        Budget::demo(),
    );

    // 2) VoI refuse
    println!("── 2. VoI refuse (free-form excess tokens)");
    demo_close(&mol, "write a poem about GPUs", Budget::demo());

    // 3) Settle path
    println!("── 3. Solver/settle close + settle refuse");
    demo_close(&mol, "settle ternary [1, 1, 1, 1]", Budget::demo());
    demo_close(
        &mol,
        "settle refuse will not settle [1, 1, 1]",
        Budget::demo(),
    );
    let klere = StubKlereSettle;
    let k = klere
        .settle(&KlereSettleJob {
            summary: "demo settle refuse".into(),
            state: vec![1, 1, 1],
            max_steps: 4,
            force_refuse: true,
        })
        .expect("klere stub");
    println!(
        "   klere stub: decision={:?} measured_j={:?} board_synth={}",
        k.decision, k.measured_j, k.board_synth_claimed
    );
    println!();

    // 4) EFA certificate refuse
    println!("── 4. EFA certificate refuse (BMI propose/certify/refuse)");
    let efa = StubEfaCertificate;
    let cert = efa
        .certify(&EfaProposal {
            summary: "7-DOF arm swing".into(),
            estimated_j: 1e-6,
            energy_residual: None,
            tag: Some("diverge".into()),
        })
        .expect("efa stub");
    println!(
        "   efa stub: decision={:?} board_synth={} reasons={}",
        cert.decision,
        cert.board_synth_claimed,
        cert.reasons.join("; ")
    );
    demo_close(
        &mol,
        "landauer joules per bit diverge",
        Budget::coin_cell(),
    );

    // 5) Automate mutate refuse + certify path
    println!("── 5. Automate: capability deny + EFA certify refuse");
    let g = AutomateGate::default();
    let out = g.gate(&Act::new(ActKind::Mutate, "touch /tmp/x", 1e-12));
    println!(
        "   mutate default: {:?} board_synth={}",
        out.decision, out.board_synth_claimed
    );
    let mut caps = CapabilitySet::sense_propose();
    caps.granted.push(Capability::Mutate);
    let g2 = AutomateGate::new(caps);
    let mut act = Act::new(ActKind::Mutate, "arm swing", 1e-9);
    act.payload = Some(serde_json::json!({"tag": "diverge"}));
    let out2 = g2.gate(&act);
    println!(
        "   mutate+cap diverge: {:?} limit={:?}",
        out2.decision,
        out2.receipt.limit_fired.as_ref().map(|f| f.id.as_str())
    );
    println!();

    // 6) Primitive gap
    println!("── 6. Primitive gap floor");
    demo_close(
        &mol,
        "primitive gap: physical_settle missing on Periodic Stack",
        Budget::demo(),
    );

    // 7) OpenIE / WCA offline ports
    println!("── 7. Ports (offline / software-reference)");
    let openie = StubOpenIeRuntime;
    let req = MolRequest::new("landauer joules per bit", Budget::demo());
    let oi = openie.ask(&req).expect("openie");
    println!(
        "   openie: zone={} tier={} offline={}",
        oi.zone, oi.mol_tier, oi.offline
    );
    let wca = StubWcaCommit;
    let w = wca.certify("safe propose", 1e-12).expect("wca");
    println!(
        "   wca: decision={} board_synth={}",
        w.decision, w.board_synth_claimed
    );
    println!();

    // 8) Thin agent mailbox loop
    println!("── 8. Thin agent mailbox loop (Goal → close → transcript)");
    let mut agent = AgentLoop::new();
    let report = agent.run_demo();
    print!("{}", report.display_block());
    match agent.demo_expectations_met() {
        Ok(()) => println!("   demo expectations: OK (formula/cite/compose + voi/settle refuse; model cold)"),
        Err(e) => println!("   demo expectations FAIL: {e}"),
    }
    println!();

    println!("Honesty: estimated ≠ measured; measured_j=None; no fake RAPL/NVML;");
    println!("board_synth_claimed=false; model never substrate.");
    println!("Gaps still stub: live openie-path / wca-path / efa Ferric / klere-vm FPGA.");
    ExitCode::SUCCESS
}


fn cmd_replay(path: PathBuf, record_samples: bool) -> ExitCode {
    let mol = MixtureOfLimits::new();
    if record_samples {
        let samples: &[(&str, Budget)] = &[
            ("landauer joules per bit", Budget::coin_cell()),
            ("convert 100 celsius to fahrenheit", Budget::coin_cell()),
            ("write a poem about GPUs", Budget::demo()),
            ("settle ternary [1, 1, 1, 1]", Budget::demo()),
            ("settle refuse will not settle [1,1,1]", Budget::demo()),
            ("landauer joules per bit diverge", Budget::coin_cell()),
            ("stack navigate family logic", Budget::coin_cell()),
            ("primitive gap: physical_settle not on stack", Budget::demo()),
        ];
        let mut transcript = CloseTranscript::new();
        for (q, b) in samples {
            match mol.close(&MolRequest::new(*q, *b)) {
                Ok(out) => {
                    transcript.push(mol_receipt::CloseTranscriptEntry::record(
                        *q,
                        *b,
                        out.is_commit(),
                        out.receipt(),
                    ));
                }
                Err(e) => {
                    eprintln!("record error on '{q}': {e}");
                    return ExitCode::FAILURE;
                }
            }
        }
        if let Err(e) = transcript.save_jsonl(&path) {
            eprintln!("write {}: {e}", path.display());
            return ExitCode::FAILURE;
        }
        println!("wrote {} entries → {}", transcript.entries.len(), path.display());
    }

    let transcript = match CloseTranscript::load_jsonl(&path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("load {}: {e}", path.display());
            return ExitCode::FAILURE;
        }
    };
    if transcript.entries.is_empty() {
        eprintln!("empty transcript");
        return ExitCode::FAILURE;
    }
    let report = match replay_transcript(&transcript, |q, b| {
        let out = mol.close(&MolRequest::new(q, b))?;
        Ok(ReplayCloseOutcome {
            commit: out.is_commit(),
            receipt: out.receipt().clone(),
        })
    }) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("replay error: {e}");
            return ExitCode::FAILURE;
        }
    };
    print!("{}", report.display_block());
    if report.ok() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}



fn cmd_fabric(detect: bool, mock: bool, json: bool) -> ExitCode {
    println!("=== MoL fabric inventory ===");
    println!("honesty: {DETECT_HONESTY_NOTE}");
    println!("meter: {}", meter_status_line());
    println!("fabric-detect feature compiled: {FABRIC_DETECT_ENABLED}");
    println!();

    let soft = FabricInventory::software_ref();
    println!("── soft-ref (default / CI prove path)");
    println!("   source: {}", soft.source);
    println!("   present: [{}]", soft.summary());
    println!("   valid_software_ref: {}", soft.is_valid_software_ref());
    for row in &soft.devices {
        let flag = if row.present { "present" } else { "absent " };
        let note = row.note.as_deref().unwrap_or("");
        if row.present || matches!(row.kind, DeviceKind::Cpu) {
            println!("   {flag}  {:<22} {note}", row.kind.label());
        }
    }
    println!();

    if mock {
        let mocked = FabricInventory::from_adapter_hints(&[
            AdapterBackendHint::Metal,
            AdapterBackendHint::Vulkan,
            AdapterBackendHint::BrowserWebGpu,
        ]);
        println!("── mock hints (Metal→GpuMetal, Vulkan→GpuVulkan, WebGPU→GpuWebGpu)");
        println!("   source: {}", mocked.source);
        println!("   present: [{}]", mocked.summary());
        for row in &mocked.devices {
            if row.present {
                let note = row.note.as_deref().unwrap_or("");
                println!("   present  {:<22} {note}", row.kind.label());
            }
        }
        println!();
        if json {
            match serde_json::to_string_pretty(&mocked) {
                Ok(j) => println!("mock_json:\n{j}"),
                Err(e) => eprintln!("mock json error: {e}"),
            }
        }
    }

    if detect {
        println!("── detect (wgpu adapters)");
        if !FABRIC_DETECT_ENABLED {
            println!("   feature fabric-detect OFF — rebuild with:");
            println!("     cargo run -p mol-cli --features fabric-detect -- fabric --detect");
            println!("   falling back to soft-ref (offline-safe; no GPU required for prove)");
        } else {
            let probes = detect_adapter_probes();
            let inv = detect_inventory();
            println!("   source: {}", inv.source);
            println!("   adapters enumerated: {}", probes.len());
            for p in &probes {
                let mapped = p
                    .backend
                    .to_device_kind()
                    .map(|k| k.label().to_string())
                    .unwrap_or_else(|| "(no Gpu* mapping)".into());
                println!(
                    "   adapter name={:?} backend={} → {}",
                    p.name, p.backend, mapped
                );
            }
            println!("   inventory present: [{}]", inv.summary());
            for row in &inv.devices {
                if row.present {
                    let note = row.note.as_deref().unwrap_or("");
                    println!("   present  {:<22} {note}", row.kind.label());
                }
            }
            println!("   note: detection marks presence only; measured_j remains None");
            print_schedule_stamps(&inv);
            if json {
                match serde_json::to_string_pretty(&inv) {
                    Ok(j) => println!("detect_json:\n{j}"),
                    Err(e) => eprintln!("detect json error: {e}"),
                }
            }
        }
        println!();
    }

    if json && !mock && !detect {
        match serde_json::to_string_pretty(&soft) {
            Ok(j) => println!("soft_ref_json:\n{j}"),
            Err(e) => {
                eprintln!("json error: {e}");
                return ExitCode::FAILURE;
            }
        }
    }

    // Soft-ref path always succeeds (prove/offline).
    if !soft.is_valid_software_ref() {
        eprintln!("soft-ref inventory invalid");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn cmd_memory() -> ExitCode {
    println!("=== MoL bitemporal state + memory ===");
    println!("valid-time + transaction-time store; writes only via MoL close COMMIT");
    println!("refuse on VoI / missing Mutate capability; recall → RetrievedCited\n");

    let mut agent = AgentLoop::new();
    match agent.run_memory_demo() {
        Ok((cap_r, write_c, recall_c, cite)) => {
            println!("capability_refuse_without_mutate: {cap_r}");
            println!("write_commit_with_mutate:        {write_c}");
            println!("recall_commit_cited:             {recall_c}");
            println!("citation_id:                     {cite}");
            let store = agent.gate.mol.memory_lock();
            println!("store_facts: {}", store.len());
            for f in store.facts() {
                println!(
                    "  {} key={} value={} tx={} valid_from={}",
                    f.id,
                    f.key,
                    f.value,
                    f.tx_id,
                    f.valid_from.to_rfc3339()
                );
            }
            drop(store);
            // Honesty sample from last transcript entry
            if let Some(e) = agent.mailbox.transcript().last() {
                println!(
                    "last_receipt: measured_j={:?} board_synth={} model_answered={}",
                    e.receipt.measured_j,
                    e.receipt.board_synth_claimed,
                    e.model_answered
                );
            }
            if cap_r && write_c && recall_c && cite.starts_with("memory:") {
                println!("\nMEMORY DEMO: OK");
                ExitCode::SUCCESS
            } else {
                println!("\nMEMORY DEMO: FAIL fingerprint");
                ExitCode::FAILURE
            }
        }
        Err(e) => {
            eprintln!("memory demo error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn cmd_agent(demo: bool, goals: Vec<String>, message: Option<String>) -> ExitCode {
    println!("=== MoL thin agent mailbox loop ===");
    println!("sense → classify → cascade/floors → certify → commit|refuse → record");
    println!("allow_model=false (no silent model escalate)\n");

    let mut agent = AgentLoop::new();
    if demo {
        agent.seed_demo_goals();
        println!("seeded demo goals: formula / cite / compose / voi refuse / settle refuse");
    }
    for g in &goals {
        let id = agent.post_goal(g);
        println!("posted goal #{id}: {g}");
    }
    if let Some(body) = message.as_ref() {
        let id = agent.post_message("user", "mol", body);
        println!("posted message #{id}: {body}");
    }
    if agent.mailbox.pending_count() == 0 {
        eprintln!("nothing pending (use --demo or --goal / --message)");
        return ExitCode::FAILURE;
    }

    let report = agent.run();
    print!("{}", report.display_block());
    for entry in agent.mailbox.transcript() {
        let r = &entry.receipt;
        println!(
            "  receipt#{} estimated_j={} measured_j={:?} board_synth={} mu_source={}",
            entry.mail_id,
            r.estimated_j,
            r.measured_j,
            r.board_synth_claimed,
            r.mu_source
        );
    }
    if demo {
        match agent.demo_expectations_met() {
            Ok(()) => println!("demo expectations: OK"),
            Err(e) => {
                eprintln!("demo expectations FAIL: {e}");
                return ExitCode::FAILURE;
            }
        }
    }
    if report.ok_no_model() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn demo_close(mol: &MixtureOfLimits, q: &str, b: Budget) {
    println!("   ask: {q}");
    let req = MolRequest::new(q, b);
    match mol.close(&req) {
        Ok(out) => {
            let r = out.receipt();
            match &r.answer {
                Some(a) => println!("   answer: {a}"),
                None => println!("   answer: (refused)"),
            }
            if let Some(t) = r.cascade_answered {
                let z = r.zone.map(|z| z.to_string()).unwrap_or_else(|| t.openie_zone().to_string());
                println!("   closed_at: {t} / {z}");
            }
            if let Some(rc) = r.replay_class {
                println!("   replay_class: {rc}");
            }
            if !r.citation_ids.is_empty() {
                println!("   citation_ids: {}", r.citation_ids.join(", "));
            }
            if !r.composed_from.is_empty() {
                println!("   composed_from: {}", r.composed_from.join(", "));
            }
            if let Some(syn) = &r.synthesis {
                println!(
                    "   synthesis: kind={} count={} from=[{}]",
                    syn.kind,
                    syn.claim_count,
                    syn.composed_from.join(", ")
                );
            }
            if let Some(f) = &r.limit_fired {
                println!("   limit_fired: {} — {}", f.id, f.reason);
            }
            println!(
                "   close: {}  estimated_j={}  measured_j={}  board_synth={}",
                if out.is_commit() { "COMMIT" } else { "REFUSE" },
                r.estimated_j,
                r.measured_j
                    .map(|j| j.to_string())
                    .unwrap_or_else(|| "None".into()),
                r.board_synth_claimed
            );
            let mut signed = r.clone();
            let _ = sign_receipt(&mut signed, b"mol-demo-secret");
            println!();
        }
        Err(e) => println!("   error: {e}\n"),
    }
}

fn cmd_meter(sample_ms: u64) -> ExitCode {
    println!("{}", meter_status_line());
    let cap = probe_meter_capability();
    println!(
        "capability: available={} platform={} source={}",
        cap.available,
        cap.platform,
        cap.source.label()
    );
    println!("note: {}", METER_HONESTY_NOTE);
    println!("{MACOS_METER_HELP}");
    if sample_ms == 0 {
        println!("sample: skipped (pass --sample-ms N to read counters; 0 does not invent joules)");
        return ExitCode::SUCCESS;
    }
    let sample = measure_energy_window(Duration::from_millis(sample_ms));
    if !sample.honesty_ok() {
        eprintln!("FAIL meter honesty: {}", sample.detail);
        return ExitCode::FAILURE;
    }
    println!(
        "sample: measured_j={} source={} window_ms={} detail={}",
        sample
            .measured_j
            .map(|j| j.to_string())
            .unwrap_or_else(|| "None".into()),
        sample.source.label(),
        sample.window_ms,
        sample.detail
    );
    if sample.components.is_empty() {
        println!("components: (none — unavailable or not exposed)");
    } else {
        for c in &sample.components {
            println!(
                "component: {:<8} {:.6e} J  source={}",
                c.component, c.joules.0, c.measure_source
            );
        }
    }
    ExitCode::SUCCESS
}

fn print_schedule_stamps(inv: &FabricInventory) {
    let formula = schedule_fabric(
        CascadeTier::Formula,
        inv,
        &Budget::coin_cell(),
        Joules::new(1e-12),
    );
    let model_budget = Budget::demo().allow_model();
    let model = schedule_fabric(CascadeTier::Model, inv, &model_budget, Joules::ZERO);
    for (label, sched) in [("formula", &formula), ("model", &model)] {
        let step = sched.to_compute_step();
        let backend = sched
            .chosen_kind()
            .and_then(|k| k.backend_token())
            .unwrap_or("-");
        println!(
            "   schedule {label}: fabric_id={} backend={backend} unavailable={} measured_j={}",
            step.fabric_id.as_deref().unwrap_or("-"),
            step.unavailable_reason.as_deref().unwrap_or("-"),
            step.measured_j
                .map(|j| j.to_string())
                .unwrap_or_else(|| "None".into())
        );
    }
}

fn cmd_ecosystem_certify(
    refuse_without_grant: bool,
    receipt_json: bool,
    session: String,
    a: i32,
    b: i32,
    meter: bool,
    meter_ms: u64,
    meter_required: bool,
    detect: bool,
    tier: String,
) -> ExitCode {
    println!("=== MoL ecosystem e2e certify ===");
    let attach_meter = meter || meter_required;
    let mut cfg = if refuse_without_grant {
        EcosystemCertifyConfig::refuse_without_grant()
    } else {
        EcosystemCertifyConfig::soft_ref()
    };
    cfg.session_id = session;
    cfg.add_args = (a, b);
    let tier_l = tier.to_ascii_lowercase();
    let schedule_tier = match tier_l.as_str() {
        "formula" | "lookup" => CascadeTier::Formula,
        "model" => CascadeTier::Model,
        "solver" | "settle" => CascadeTier::Solver,
        other => {
            eprintln!("unknown --tier {other} (expected formula|model|solver)");
            return ExitCode::FAILURE;
        }
    };
    if detect {
        let inv = inventory_for_schedule();
        println!(
            "fabric detect: feature={FABRIC_DETECT_ENABLED} source={} present=[{}]",
            inv.source,
            inv.summary()
        );
        for probe in &inv.probes {
            let mapped = probe
                .backend
                .to_device_kind()
                .map(|k| k.label().to_string())
                .unwrap_or_else(|| "(no Gpu* mapping)".into());
            println!(
                "  adapter name={:?} backend={} device_class={} → {mapped}",
                probe.name, probe.backend, probe.device_class
            );
        }
        if !FABRIC_DETECT_ENABLED {
            println!("  feature off — inventory is soft-ref (Cpu). Rebuild with --features fabric-detect");
        }
        cfg = cfg.with_fabric(inv);
    }
    if schedule_tier == CascadeTier::Model {
        // Demo budget can choose Gpu*; coin-cell would fail-closed even if Metal is present.
        cfg = cfg.with_budget(Budget::demo().allow_model());
    }
    cfg = cfg.with_schedule_tier(schedule_tier);
    println!(
        "schedule_tier: {}  budget_max_j={} allow_model={}",
        schedule_tier.label(),
        cfg.budget.max_j,
        cfg.budget.allow_model
    );
    if meter_required {
        cfg = cfg.with_meter_required(true);
    }
    if attach_meter {
        // Overlapping window: SMC/RAPL samples while the Metal/wgpu kernel runs
        // (same wall-clock interval). Never invent measured_j; never sum rails.
        println!(
            "meter: feature={} overlapping_window_ms={} (sample during kernel when Gpu*)",
            ENERGY_METER_ENABLED, meter_ms
        );
        cfg = cfg.with_meter_window_ms(meter_ms);
    }
    let out = EcosystemCertify::run(&cfg);
    let r = &out.receipt;
    println!(
        "decision: {}",
        if out.is_commit() { "COMMIT" } else { "REFUSE" }
    );
    if let Some(ref floor) = out.floor {
        println!("floor: {} ({})", floor.id, floor.kind);
    }
    println!(
        "capsule_return: {:?}  fuel: {:?}  grant_id: {:?}",
        out.capsule_return, out.fuel_consumed, out.grant_id
    );
    println!(
        "stamps: encapsulation={} agent_lane={} fabric_chosen={:?} compute_steps={} energy_honesty={}",
        r.encapsulation.is_some(),
        r.agent_lane.is_some(),
        r.fabric_chosen,
        r.compute_steps.len(),
        r.energy_honesty
    );
    print_receipt_honesty(r);
    print_fabric_stamp(r);
    println!(
        "receipt_meter: measured_j={} source={} energy_honesty={}",
        r.measured_j
            .map(|j| format!("{:.6e}", j.0))
            .unwrap_or_else(|| "None".into()),
        r.measure_source.label(),
        r.energy_honesty.label()
    );
    for s in &r.compute_steps {
        if s.label.starts_with("kernel:") {
            println!(
                "kernel_step: label={} fabric_id={:?} proof={:?} measured_j={} source={} honesty={}",
                s.label,
                s.fabric_id,
                s.execution_proof,
                s.measured_j
                    .map(|j| format!("{:.6e}", j.0))
                    .unwrap_or_else(|| "None".into()),
                s.measure_source.label(),
                s.honesty.label()
            );
        }
    }
    if receipt_json {
        match r.to_json() {
            Ok(j) => println!("{j}"),
            Err(e) => {
                eprintln!("receipt json error: {e}");
                return ExitCode::FAILURE;
            }
        }
    }
    if out.is_commit() {
        ExitCode::SUCCESS
    } else if out.floor.as_ref().map(|f| f.id.as_str()) == Some("fabric_unavailable")
        && !meter_required
    {
        // Honest fail-closed: no sufficient fabric. Do not invent a GPU or joules.
        println!("fabric_unavailable: fail-closed (expected when Gpu* absent or budget too tight)");
        ExitCode::SUCCESS
    } else if refuse_without_grant && !meter_required {
        // Expected refuse is success for the refuse-path demo.
        ExitCode::SUCCESS
    } else if meter_required && out.is_refuse() {
        // meter_required refuse is an honest outcome (no invented joules).
        ExitCode::FAILURE
    } else {
        ExitCode::FAILURE
    }
}

