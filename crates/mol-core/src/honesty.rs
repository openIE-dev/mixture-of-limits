//! Energy honesty — measured vs estimated vs modeled vs unavailable.
//!
//! Law (Lux `lux-energy` + Nova `navigator.energy` + MoL):
//! - Estimation is the universal floor (runs everywhere, including WASM).
//! - Measurement replaces estimate only when a real counter exists.
//! - Modeled (Nova T4-modeled / CPU-proxy) must never be labeled Measured.
//! - `measured_j` is Some **iff** `MeasureSource::is_measured()` — never invent.

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::energy::{Joules, MeasureSource};
use crate::floor::{Floor, FloorKind};

/// Honesty class for a joule figure on a receipt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EnergyHonestyClass {
    /// No soft measurement; catalog / cascade estimate only.
    #[default]
    Estimated,
    /// Real hardware / OS counter (RAPL, NVML, IOReport, powermetrics…).
    Measured,
    /// CPU-util / proxy model — honest but not silicon-metered (Nova T4-modeled).
    Modeled,
    /// Probe attempted / required but unavailable.
    Unavailable,
}

impl EnergyHonestyClass {
    /// Wire label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Estimated => "estimated",
            Self::Measured => "measured",
            Self::Modeled => "modeled",
            Self::Unavailable => "unavailable",
        }
    }

    /// Derive honesty class from measure source + optional measured joules.
    pub fn from_measure(source: MeasureSource, measured_j: Option<Joules>) -> Self {
        match source {
            MeasureSource::Unavailable => {
                if measured_j.is_some() {
                    // Illegal pairing — treat as unavailable; caller must refuse.
                    Self::Unavailable
                } else {
                    Self::Estimated
                }
            }
            MeasureSource::CatalogSurrogate | MeasureSource::CascadeEstimate => Self::Estimated,
            MeasureSource::CpuProxy => Self::Modeled,
            s if s.is_measured() => {
                if measured_j.is_some() {
                    Self::Measured
                } else {
                    Self::Unavailable
                }
            }
            _ => Self::Estimated,
        }
    }
}

impl fmt::Display for EnergyHonestyClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// True when `measured_j` and `measure_source` agree (never invent).
pub fn energy_pair_honest(measured_j: Option<Joules>, source: MeasureSource) -> bool {
    match (measured_j, source.is_measured()) {
        (Some(j), true) => j.is_valid(),
        (None, false) => true,
        // measured without measured source, or measured source without joules
        (Some(_), false) | (None, true) => false,
    }
}

/// Fail-closed when measured energy is required but missing / dishonest.
pub fn require_measured_or_refuse(
    measured_j: Option<Joules>,
    source: MeasureSource,
) -> Result<(), Floor> {
    // Package `measured_j` is required. A components-only IOReport/powermetrics sample
    // (rails present, no package/combined channel) is an honest MeterSample but must
    // REFUSE here — never invent a package sum from CPU/GPU/ANE/DRAM.
    if measured_j.is_none() {
        return Err(Floor::new(
            "energy_honesty_require_measured",
            FloorKind::EnergyHonesty,
            format!(
                "fail-closed: package measured_j required; got None (source={}) — never invent package from component rails",
                source.label()
            ),
        ));
    }
    if !energy_pair_honest(measured_j, source) {
        return Err(Floor::new(
            "energy_honesty_pair",
            FloorKind::EnergyHonesty,
            format!(
                "measured_j/source pair dishonest (measured_j={}, source={})",
                measured_j.is_some(),
                source.label()
            ),
        ));
    }
    let class = EnergyHonestyClass::from_measure(source, measured_j);
    // Modeled (CpuProxy) is honest as modeled but must never satisfy meter_required.
    if class != EnergyHonestyClass::Measured {
        return Err(Floor::new(
            "energy_honesty_require_measured",
            FloorKind::EnergyHonesty,
            format!(
                "fail-closed: Measured honesty + real measured_j required; got class={} source={} (never invent measured_j)",
                class.label(),
                source.label()
            ),
        ));
    }
    Ok(())
}

/// Fail-closed when provenance (cite / capsule / meter) is required but absent.
pub fn require_provenance_or_refuse(
    has_citations: bool,
    has_capsule: bool,
    has_measured: bool,
    require_cite: bool,
    require_capsule: bool,
    require_meter: bool,
) -> Result<(), Floor> {
    if require_cite && !has_citations {
        return Err(Floor::new(
            "provenance_missing_cite",
            FloorKind::ProvenanceMissing,
            "fail-closed: citation provenance required but citation_ids empty",
        ));
    }
    if require_capsule && !has_capsule {
        return Err(Floor::new(
            "provenance_missing_capsule",
            FloorKind::ProvenanceMissing,
            "fail-closed: encapsulation capsule required but absent",
        ));
    }
    if require_meter && !has_measured {
        return Err(Floor::new(
            "provenance_missing_meter",
            FloorKind::ProvenanceMissing,
            "fail-closed: meter provenance required; measured_j absent (never invent)",
        ));
    }
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::energy::Joules;

    #[test]
    fn pair_honest_none_unavailable() {
        assert!(energy_pair_honest(None, MeasureSource::Unavailable));
        assert!(energy_pair_honest(None, MeasureSource::CatalogSurrogate));
    }

    #[test]
    fn pair_rejects_invented_measured() {
        assert!(!energy_pair_honest(Some(Joules::new(1.0)), MeasureSource::Unavailable));
        assert!(!energy_pair_honest(None, MeasureSource::Rapl));
    }

    #[test]
    fn require_measured_refuses_soft_ref() {
        let err = require_measured_or_refuse(None, MeasureSource::Unavailable).unwrap_err();
        assert_eq!(err.kind, FloorKind::EnergyHonesty);
    }

    #[test]
    fn modeled_is_not_measured() {
        let h = EnergyHonestyClass::from_measure(MeasureSource::CpuProxy, Some(Joules::new(0.1)));
        assert_eq!(h, EnergyHonestyClass::Modeled);
    }

    #[test]
    fn require_measured_refuses_modeled_cpu_proxy() {
        let err = require_measured_or_refuse(Some(Joules::new(0.1)), MeasureSource::CpuProxy)
            .unwrap_err();
        assert_eq!(err.kind, FloorKind::EnergyHonesty);
        assert_eq!(err.id.as_str(), "energy_honesty_require_measured");
    }

    #[test]
    fn require_measured_ok_rapl() {
        assert!(require_measured_or_refuse(Some(Joules::new(0.1)), MeasureSource::Rapl).is_ok());
    }
}
