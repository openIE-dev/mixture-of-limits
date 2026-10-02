//! Impedance mismatch μ and catalog energy estimates.
//!
//! Proof law: `E(x) ≥ θ(D)·μ(S,V)` with `θ = bits × k_B T ln 2` (Landauer)
//! and catalog `μ` per cascade tier / primitive class.
//!
//! Honesty: `mu_source=catalog` is an analytical surrogate table — **never** RAPL/NVML.

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::energy::Joules;
use crate::landauer::Landauer;
use crate::tier::CascadeTier;

/// Default bit-erasure count for a single tier attempt (θ bookkeeping).
pub const DEFAULT_THETA_BITS: f64 = 64.0;

/// Where receipt `estimated_j` / μ came from.
///
/// Soft-ref MoL always stamps [`MuSource::Catalog`]. Calib is reserved for a
/// future corpus; it is still an estimate label, not a silicon meter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MuSource {
    /// Tier / primitive catalog impedance table (clean-room surrogate).
    #[default]
    Catalog,
    /// Optional μ calib corpus means (not shipped in v0.1 proof path).
    Calib,
}

impl MuSource {
    /// Wire / receipt label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Catalog => "catalog",
            Self::Calib => "calib",
        }
    }

    /// True iff this is the clean-room catalog table (not a fake RAPL claim).
    pub const fn is_catalog(self) -> bool {
        matches!(self, Self::Catalog)
    }
}

impl fmt::Display for MuSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// `E ≈ θ·μ` estimate from catalog impedance.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ImpedanceEstimate {
    /// Bits assumed erased for θ.
    pub bits_erased: f64,
    /// Landauer θ joules (`bits × k_B T ln2`).
    pub theta_j: f64,
    /// Catalog impedance mismatch μ (dimensionless).
    pub mu: f64,
    /// Resulting estimated joules: `θ · μ`.
    pub estimated_j: Joules,
    /// Provenance — always [`MuSource::Catalog`] on the soft-ref path.
    pub mu_source: MuSource,
}

impl ImpedanceEstimate {
    /// Ratio `estimated_j / θ` (equals catalog μ when θ > 0).
    pub fn landauer_floor_ratio(self) -> Option<f64> {
        if self.theta_j > 0.0 && self.theta_j.is_finite() {
            Some(self.estimated_j.0 / self.theta_j)
        } else {
            None
        }
    }
}

/// Clean-room catalog of impedance μ per cascade tier.
///
/// Values follow [`crate::tier::ThermoClass::mismatch_order`] — order-of-magnitude
/// multipliers above Landauer, not measured RAPL/NVML.
pub struct MuCatalog;

impl MuCatalog {
    /// Catalog μ for a cascade tier (Lookup / Formula / Solver / Model).
    pub const fn mu_for_tier(tier: CascadeTier) -> f64 {
        tier.thermo().mismatch_order()
    }

    /// Catalog μ label for receipts / docs.
    pub const fn tier_label(tier: CascadeTier) -> &'static str {
        tier.label()
    }

    /// `E ≈ θ·μ` with default bit count at room-temp Landauer.
    pub fn estimate_tier(tier: CascadeTier) -> ImpedanceEstimate {
        Self::estimate_tier_bits(tier, DEFAULT_THETA_BITS)
    }

    /// `E ≈ θ·μ` with explicit bit-erasure count.
    pub fn estimate_tier_bits(tier: CascadeTier, bits: f64) -> ImpedanceEstimate {
        let bits = if bits.is_finite() && bits > 0.0 {
            bits
        } else {
            0.0
        };
        let mu = Self::mu_for_tier(tier);
        impedance_mismatch_energy(bits, mu)
    }

    /// All tier catalog rows (Lookup → Model) for tests / CLI explain.
    pub fn tier_table() -> [(CascadeTier, f64); 4] {
        let tiers = CascadeTier::routing_order();
        [
            (tiers[0], Self::mu_for_tier(tiers[0])),
            (tiers[1], Self::mu_for_tier(tiers[1])),
            (tiers[2], Self::mu_for_tier(tiers[2])),
            (tiers[3], Self::mu_for_tier(tiers[3])),
        ]
    }
}

/// `θ(p)·μ` with `θ = bits × Landauer` at room temperature.
pub fn impedance_mismatch_energy(bits_erased: f64, mismatch_mu: f64) -> ImpedanceEstimate {
    let bits = if bits_erased.is_finite() && bits_erased > 0.0 {
        bits_erased
    } else {
        0.0
    };
    let mu = if mismatch_mu.is_finite() && mismatch_mu >= 0.0 {
        mismatch_mu
    } else {
        0.0
    };
    let theta_j = bits * Landauer::room_temp_joules_per_bit();
    let estimated_j = Joules::new(theta_j * mu);
    ImpedanceEstimate {
        bits_erased: bits,
        theta_j,
        mu,
        estimated_j,
        mu_source: MuSource::Catalog,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_mu_monotone_hotter() {
        let lookup = MuCatalog::mu_for_tier(CascadeTier::Lookup);
        let formula = MuCatalog::mu_for_tier(CascadeTier::Formula);
        let solver = MuCatalog::mu_for_tier(CascadeTier::Solver);
        let model = MuCatalog::mu_for_tier(CascadeTier::Model);
        assert_eq!(lookup, formula); // both L0
        assert!(solver > formula);
        assert!(model > solver);
    }

    #[test]
    fn estimate_is_theta_times_mu() {
        let e = MuCatalog::estimate_tier(CascadeTier::Formula);
        assert_eq!(e.mu_source, MuSource::Catalog);
        assert!(e.theta_j > 0.0);
        assert!((e.estimated_j.0 - e.theta_j * e.mu).abs() / e.estimated_j.0 < 1e-9);
        let ratio = e.landauer_floor_ratio().unwrap();
        assert!((ratio - e.mu).abs() / e.mu < 1e-9);
    }

    #[test]
    fn mu_source_catalog_label() {
        assert_eq!(MuSource::Catalog.label(), "catalog");
        assert!(MuSource::Catalog.is_catalog());
    }
}
