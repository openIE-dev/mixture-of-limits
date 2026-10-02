//! `mol run` — declarative Mixture of Limits chore (mol.yaml product surface).
//!
//! Loads chore metadata / limits / cascade / completeness. Soft-ref close:
//! `estimated_j` always; `measured_j` never invented; `board_synth_claimed=false`.
//! Lookup Bloom/LUT hits never call model. VoI=0 / C(z)=1 refuse with reason codes.

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::ExitCode;

use mol_core::{
    Budget, CompletenessClause, CompletenessSnapshot, MolRequest, QueryKind, ReplayClass,
    BOARD_SYNTH_CLAIMED,
};
use mol_limits::{CloseOutcome, MixtureOfLimits};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ChoreFile {
    version: String,
    #[serde(default)]
    product: Option<String>,
    chore: ChoreMeta,
    #[serde(default)]
    cascade: CascadeCfg,
    #[serde(default)]
    measurement: MeasurementCfg,
    #[serde(default)]
    limits: Vec<LimitCfg>,
    #[serde(default)]
    pipeline: Vec<PipelineStep>,
}

#[derive(Debug, Deserialize)]
struct ChoreMeta {
    id: String,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    completeness: Option<CompletenessCfg>,
    /// Optional episode ask override (golden fixtures).
    #[serde(default)]
    ask: Option<String>,
    /// Scenario selector for product acceptance: a1|a2|a3|a4|a5|a6|default
    #[serde(default)]
    scenario: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CompletenessCfg {
    id: String,
    #[serde(default)]
    clauses: Vec<HashMap<String, bool>>,
}

#[derive(Debug, Default, Deserialize)]
struct CascadeCfg {
    #[serde(default)]
    allow_model: bool,
}

#[derive(Debug, Default, Deserialize)]
struct MeasurementCfg {
    #[serde(default)]
    board_synth_claimed: Option<bool>,
    #[serde(default)]
    #[allow(dead_code)]
    estimated_j: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    measured_j: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LimitCfg {
    id: String,
    #[serde(default)]
    #[allow(dead_code)]
    kind: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PipelineStep {
    #[serde(default)]
    tier: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    model: Option<String>,
}

fn snapshot_from_cfg(cfg: &CompletenessCfg) -> CompletenessSnapshot {
    let mut clauses = Vec::new();
    for row in &cfg.clauses {
        for (k, v) in row {
            clauses.push(CompletenessClause::new(k.clone(), *v));
        }
    }
    CompletenessSnapshot::new(&cfg.id, clauses)
}

fn scenario_request(
    chore: &ChoreFile,
    allow: bool,
    scenario: &str,
) -> (MolRequest, &'static str) {
    let mut budget = Budget::joules(1e-3);
    if allow {
        budget = budget.allow_model();
    }

    let (query, kind_hint, completeness, note) = match scenario {
        "a1" | "grammar" | "lookup" => {
            let q = if chore.chore.id.contains("financial") || chore.chore.id.contains("risk") {
                "risk score band=RISK-LOW"
            } else {
                "ticket close resolution=R-OK"
            };
            (q.to_string(), Some(QueryKind::TicketClose), None, "A1 O(1) LUT hit")
        }
        "a2" | "voi" => (
            "write a free-form poem about unbounded tokens".into(),
            Some(QueryKind::FreeForm),
            None,
            "A2 VoI=0 refuse",
        ),
        "a3" | "satiation" => {
            let c = chore
                .chore
                .completeness
                .as_ref()
                .map(snapshot_from_cfg)
                .unwrap_or_else(|| CompletenessSnapshot::ticket_close(true, true, true));
            // Force all true for satiation scenario.
            let c = CompletenessSnapshot::new(
                c.id.clone(),
                c.clauses
                    .into_iter()
                    .map(|cl| CompletenessClause::new(cl.id, true))
                    .collect(),
            );
            (
                "further synthesis after ticket complete".into(),
                Some(QueryKind::FreeForm),
                Some(c),
                "A3 C(z)=1 satiation refuse",
            )
        }
        "a4" | "ni" | "model_cert" => {
            // Force model leaf so ModelGenerated proposal hits NI certify gate.
            budget = budget.allow_model();
            (
                "residual propose uncertified ticket summary".into(),
                Some(QueryKind::FreeForm),
                None,
                "A4 ModelGenerated → NI cert refuse",
            )
        }
        "a5" | "estimate" => (
            "convert 100 celsius to fahrenheit".into(),
            None,
            None,
            "A5 estimated_j always",
        ),
        "a6" | "meter" => (
            "landauer joules per bit".into(),
            None,
            None,
            "A6 measured_j=None without meter",
        ),
        _ => {
            // default: chore ask, else typed LUT-friendly ask.
            // YAML completeness is the predicate *schema*; live C(z) attaches only
            // for satiation scenarios (a3) — otherwise Lookup would never commit.
            let q = chore.chore.ask.clone().unwrap_or_else(|| {
                if chore.chore.id.contains("financial") || chore.chore.id.contains("risk") {
                    "risk score band=RISK-MED".into()
                } else {
                    "ticket close resolution=R-HOWTO".into()
                }
            });
            (q, None, None, "chore default close")
        }
    };

    let mut req = MolRequest::new(query, budget);
    if let Some(k) = kind_hint {
        req = req.with_kind(k);
    }
    if let Some(c) = completeness {
        req = req.with_completeness(c);
    }
    (req, note)
}

/// Product surface: validate mol.yaml, bind floors, close with NI-shaped path.
pub fn cmd_run(chore: PathBuf, allow_model: bool, receipt_json: bool) -> ExitCode {
    cmd_run_scenario(chore, allow_model, receipt_json, None)
}

/// Run with explicit scenario (used by `mol prove` product suite / fixtures).
pub fn cmd_run_scenario(
    chore: PathBuf,
    allow_model: bool,
    receipt_json: bool,
    scenario_override: Option<&str>,
) -> ExitCode {
    let raw = match std::fs::read_to_string(&chore) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("mol run: cannot read {}: {e}", chore.display());
            return ExitCode::FAILURE;
        }
    };
    let cfg: ChoreFile = match serde_yaml::from_str(&raw) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("mol run: invalid mol.yaml: {e}");
            return ExitCode::FAILURE;
        }
    };

    if cfg.measurement.board_synth_claimed.unwrap_or(false) {
        eprintln!(
            "mol run: board_synth_claimed=true refused on soft-ref product path (set false)"
        );
        return ExitCode::FAILURE;
    }
    if BOARD_SYNTH_CLAIMED {
        eprintln!("mol run: invariant BOARD_SYNTH_CLAIMED must be false");
        return ExitCode::FAILURE;
    }

    let allow = allow_model || cfg.cascade.allow_model;
    let scenario = scenario_override
        .map(|s| s.to_string())
        .or_else(|| cfg.chore.scenario.clone())
        .unwrap_or_else(|| "default".into());

    println!("Mixture of Limits · mol run");
    println!(
        "  product={} version={} chore={}",
        cfg.product.as_deref().unwrap_or("mixture-of-limits"),
        cfg.version,
        cfg.chore.id
    );
    if let Some(t) = &cfg.chore.title {
        println!("  title={t}");
    }
    let limit_ids: Vec<&str> = cfg.limits.iter().map(|l| l.id.as_str()).collect();
    println!("  limits={}", limit_ids.join(","));
    println!("  pipeline_tiers={}", cfg.pipeline.iter().filter_map(|p| p.tier.as_deref()).collect::<Vec<_>>().join("→"));
    println!("  allow_model={allow}");
    println!("  scenario={scenario}");
    println!("  measurement: estimated_j=always; measured_j=only_when_meter_present");
    println!("  board_synth_claimed=false");
    println!("  estimates ≠ measured_j");

    let (req, note) = scenario_request(&cfg, allow, scenario.to_ascii_lowercase().as_str());
    println!("  note={note}");
    println!("  query={:?}", req.query);

    let mol = MixtureOfLimits::new();
    match mol.close(&req) {
        Ok(CloseOutcome::Commit { receipt, .. }) => {
            if receipt.estimated_j.0.is_nan() || !receipt.estimated_j.is_valid() {
                eprintln!("mol run: invariant fail — estimated_j missing/invalid");
                return ExitCode::FAILURE;
            }
            if receipt.measured_j.is_some() {
                eprintln!(
                    "mol run: invariant fail — measured_j set without meter (soft-ref must be None)"
                );
                return ExitCode::FAILURE;
            }
            if receipt.board_synth_claimed {
                eprintln!("mol run: invariant fail — board_synth_claimed");
                return ExitCode::FAILURE;
            }
            // ModelGenerated must not have laundered to Deterministic.
            if matches!(receipt.cascade_answered, Some(t) if t.label() == "model")
                && receipt.replay_class == Some(ReplayClass::Deterministic)
            {
                eprintln!("mol run: invariant fail — ModelGenerated laundered to Deterministic");
                return ExitCode::FAILURE;
            }
            println!("  outcome=COMMIT");
            println!(
                "  tier={:?} replay={:?} estimated_j={} measured_j=None board_synth_claimed=false",
                receipt.cascade_answered.map(|t| t.label()),
                receipt.replay_class,
                receipt.estimated_j,
            );
            if receipt_json {
                match serde_json::to_string_pretty(&receipt) {
                    Ok(s) => println!("{s}"),
                    Err(e) => {
                        eprintln!("mol run: receipt json: {e}");
                        return ExitCode::FAILURE;
                    }
                }
            }
            ExitCode::SUCCESS
        }
        Ok(CloseOutcome::Refuse { floor, receipt }) => {
            if !receipt.estimated_j.is_valid() {
                eprintln!("mol run: invariant fail — estimated_j on refuse");
                return ExitCode::FAILURE;
            }
            if receipt.measured_j.is_some() {
                eprintln!("mol run: invariant fail — invent measured_j on refuse");
                return ExitCode::FAILURE;
            }
            println!(
                "  outcome=REFUSE limit={} kind={}",
                floor.id,
                floor.kind.label()
            );
            println!(
                "  reason={}",
                floor.reason
            );
            println!(
                "  estimated_j={} measured_j=None board_synth_claimed=false",
                receipt.estimated_j
            );
            if receipt_json {
                match serde_json::to_string_pretty(&receipt) {
                    Ok(s) => println!("{s}"),
                    Err(e) => {
                        eprintln!("mol run: receipt json: {e}");
                        return ExitCode::FAILURE;
                    }
                }
            }
            // Intentional refuse with receipt is success for the product surface.
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("mol run: close error: {e}");
            ExitCode::FAILURE
        }
    }
}
