//! `mol distill` — Primitive Distillation Loop v1 CLI.
//!
//! Certified Model LAST → compile Lookup/Formula append. Uncertified never distills.

use std::path::PathBuf;
use std::process::ExitCode;

use mol_cascade::{distill_certified_model_last, DistillStore};
use mol_core::{Budget, DeviceKind, FabricInventory, Joules, MolRequest, ReplayClass};
use mol_limits::MixtureOfLimits;

/// Distill from a live residual propose close (must COMMIT with certificate ids).
pub fn cmd_distill(
    proposal: String,
    gear: String,
    pattern: Option<String>,
    body: Option<String>,
    store_path: PathBuf,
    json: bool,
) -> ExitCode {
    let mol = MixtureOfLimits::new()
        .with_fabric(FabricInventory::software_ref_with_gpu(DeviceKind::GpuMetal));
    let mut budget = Budget::demo().allow_model();
    budget.max_j = Joules::new(1.0);
    let q = if proposal.contains("residual propose") {
        proposal.clone()
    } else {
        format!("residual propose {proposal}")
    };
    let out = match mol.close(&MolRequest::new(&q, budget)) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("distill close error: {e}");
            return ExitCode::FAILURE;
        }
    };
    if !out.is_commit() {
        eprintln!(
            "distill REFUSE: uncertified or VoI/cert refuse (limit={:?}); never append Lookup",
            out.receipt().limit_fired.as_ref().map(|f| f.id.as_str())
        );
        return ExitCode::FAILURE;
    }
    let r = out.receipt();
    if r.replay_class != Some(ReplayClass::ModelGenerated) {
        eprintln!(
            "distill expects ModelGenerated source, got {:?}",
            r.replay_class
        );
        return ExitCode::FAILURE;
    }
    if r.certificate_ids.is_empty() {
        eprintln!("distill REFUSE: empty certificate_ids — uncertified never becomes Lookup");
        return ExitCode::FAILURE;
    }
    let pattern = pattern.unwrap_or_else(|| q.clone());
    let body = body.unwrap_or_else(|| {
        r.answer
            .clone()
            .unwrap_or_else(|| format!("DISTILLED from {}", r.id))
    });
    let entry = match distill_certified_model_last(
        &r.id,
        r.answer.as_deref().unwrap_or(""),
        &r.certificate_ids,
        "ni_in_crate",
        &gear,
        &pattern,
        &body,
    ) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("distill error: {e}");
            return ExitCode::FAILURE;
        }
    };
    let mut store = match DistillStore::load(&store_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("distill store load: {e}");
            return ExitCode::FAILURE;
        }
    };
    if let Err(e) = store.append(entry.clone()) {
        eprintln!("distill append: {e}");
        return ExitCode::FAILURE;
    }
    // Second-pass prove: reload store into MoL — Lookup/Formula without model.
    let mol2 = MixtureOfLimits::new()
        .with_fabric(FabricInventory::software_ref_with_gpu(DeviceKind::GpuMetal))
        .with_distill_store(store.clone());
    let mut b2 = Budget::demo(); // model cold
    b2.max_j = Joules::new(1.0);
    let second = mol2.close(&MolRequest::new(&entry.pattern, b2));
    let second_ok = match &second {
        Ok(o) if o.is_commit() => matches!(
            o.receipt().cascade_answered,
            Some(mol_core::CascadeTier::Lookup) | Some(mol_core::CascadeTier::Formula)
        ),
        _ => false,
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&entry).unwrap());
    } else {
        println!("=== mol distill — Primitive Distillation Loop v1 (hardened) ===");
        println!("appended {} gear={} pattern={}", entry.id, entry.gear, entry.pattern);
        println!("source_receipt={} certs={:?}", entry.source_receipt_id, entry.certificate_ids);
        println!("replay_class=Deterministic (after NI cert); store={}", store_path.display());
        if second_ok {
            println!(
                "second_pass=OK tier={:?} (Lookup/Formula without model)",
                second.as_ref().unwrap().receipt().cascade_answered
            );
        } else {
            println!("second_pass=WARN could not close distilled pattern without model");
        }
    }
    ExitCode::SUCCESS
}

/// Negative path: attempt distill without cert → must fail.
#[allow(dead_code)]
pub fn cmd_distill_refuse_uncertified() -> ExitCode {
    match distill_certified_model_last(
        "none",
        "proposal",
        &[],
        "ni_in_crate",
        "lookup",
        "k",
        "v",
    ) {
        Err(_) => {
            println!("VERIFIED distill_uncertified_refuse — uncertified never becomes Lookup");
            ExitCode::SUCCESS
        }
        Ok(_) => {
            eprintln!("FAIL: uncertified distill must refuse");
            ExitCode::FAILURE
        }
    }
}
