//! Structural integrity checks (beyond HMAC).

use mol_core::{MeasureSource, MeterComponent, MolError, BOARD_SYNTH_CLAIMED};

use crate::receipt::MolReceipt;
use crate::sign::{secret_from_env, verify_signature};

/// Verify honesty + optional HMAC.
pub fn verify_receipt_integrity(r: &MolReceipt) -> Result<(), MolError> {
    if r.board_synth_claimed != BOARD_SYNTH_CLAIMED {
        return Err(MolError::ReceiptVerify(
            "board_synth_claimed must be false (software reference)".into(),
        ));
    }
    if r.schema != MolReceipt::SCHEMA {
        return Err(MolError::ReceiptVerify(format!(
            "schema mismatch: {}",
            r.schema
        )));
    }
    // Honesty: measured_j requires a measured source.
    if let Some(j) = r.measured_j {
        if !j.is_valid() {
            return Err(MolError::ReceiptVerify("measured_j not finite/nonneg".into()));
        }
        if !r.measure_source.is_measured() && r.measure_source != MeasureSource::Unavailable {
            // Allow unavailable with None only; if Some measured_j, source must be measured.
            if !r.measure_source.is_measured() {
                return Err(MolError::ReceiptVerify(format!(
                    "measured_j present but measure_source={} is not a measurement",
                    r.measure_source
                )));
            }
        }
        if r.measure_source == MeasureSource::Unavailable {
            return Err(MolError::ReceiptVerify(
                "measured_j set but measure_source=unavailable".into(),
            ));
        }
        if !r.measure_source.is_measured() {
            return Err(MolError::ReceiptVerify(format!(
                "measured_j set but measure_source={} (refuse fake RAPL/NVML labels on estimates)",
                r.measure_source
            )));
        }
    }
    if !r.component_measured.is_empty() && !r.measure_source.is_measured() {
        return Err(MolError::ReceiptVerify(format!(
            "component_measured present but measure_source={} is not a measurement",
            r.measure_source
        )));
    }
    for c in &r.component_measured {
        if !c.joules.is_valid() {
            return Err(MolError::ReceiptVerify(format!(
                "component {} joules not finite/nonneg",
                c.component
            )));
        }
        if !c.measure_source.is_measured() {
            return Err(MolError::ReceiptVerify(format!(
                "component {} source {} is not a measurement",
                c.component, c.measure_source
            )));
        }
        // Package rail must match receipt primary source (smc / rapl / …).
        // Other rails may keep their own probe (e.g. IOReport cpu + SMC package).
        if c.component == MeterComponent::Package && c.measure_source != r.measure_source {
            return Err(MolError::ReceiptVerify(format!(
                "package component source {} != receipt {}",
                c.measure_source, r.measure_source
            )));
        }
    }
    if r.measure_source.is_measured() && r.measured_j.is_none() && r.component_measured.is_empty() {
        return Err(MolError::ReceiptVerify(
            "measure_source claims a meter but no joules were recorded".into(),
        ));
    }
    if !r.estimated_j.is_valid() {
        return Err(MolError::ReceiptVerify("estimated_j not finite/nonneg".into()));
    }
    if let Some(secret) = secret_from_env() {
        if r.signature_hex.is_some() {
            verify_signature(r, &secret)?;
        }
    }
    Ok(())
}
