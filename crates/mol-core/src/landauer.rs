//! Landauer floor: `E_min = k_B T ln 2` joules per bit erased.
//!
//! This is a **thermodynamic lower-bound estimate** — never equal to soft
//! RAPL / NVML / CPU-proxy measurements.
//!
//! Thesis hook: reversible / thermodynamic computing shows intelligence can
//! *collapse into physics* via erase-cost floors; MoL annotates the bound on
//! receipts and navigates Lookup→Formula→Solver gears first (see
//! `docs/quantum-thermo-collapse.md`). No quantum or annealer hardware claim.

use serde::{Deserialize, Serialize};

/// Boltzmann constant (J/K).
pub const BOLTZMANN_J_PER_K: f64 = 1.380_649e-23;

/// Natural log of 2.
pub const LN2: f64 = std::f64::consts::LN_2;

/// Default room temperature used for the floor (K).
pub const ROOM_TEMPERATURE_KELVIN: f64 = 300.0;

/// Landauer reference constants.
pub struct Landauer;

impl Landauer {
    /// `E_min = k_B T ln 2` joules per bit erased at temperature `t_kelvin`.
    pub fn joules_per_bit(t_kelvin: f64) -> f64 {
        BOLTZMANN_J_PER_K * t_kelvin * LN2
    }

    /// Floor at 300 K ≈ 2.87e-21 J/bit.
    pub fn room_temp_joules_per_bit() -> f64 {
        Self::joules_per_bit(ROOM_TEMPERATURE_KELVIN)
    }
}

/// Landauer-style bit-erasure thermodynamic floor estimate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LandauerFloor {
    /// Bits erased (known or conservatively inferred).
    pub bits: f64,
    /// Temperature used (K).
    pub t_kelvin: f64,
    /// Floor joules: `k_B T ln2 * bits`.
    pub floor_j: f64,
    /// Short rationale (states estimate ≠ measured RAPL/NVML).
    pub rationale: String,
}

/// Compute Landauer floor from bit-erasure count at room temperature (or `MOL_LANDAUER_T`).
pub fn landauer_floor_from_bits(bits: f64) -> LandauerFloor {
    let bits = if bits.is_finite() && bits > 0.0 {
        bits
    } else {
        0.0
    };
    let t = std::env::var("MOL_LANDAUER_T")
        .ok()
        .and_then(|s| s.parse().ok())
        .filter(|t: &f64| t.is_finite() && *t > 0.0)
        .unwrap_or(ROOM_TEMPERATURE_KELVIN);
    let jpb = Landauer::joules_per_bit(t);
    let floor_j = bits * jpb;
    let rationale = format!(
        "Landauer floor estimate k_B*T*ln2*{bits:.4e} bits at T={t} K \
         (bit-erasure thermodynamic lower bound; not measured RAPL/NVML)"
    );
    LandauerFloor {
        bits,
        t_kelvin: t,
        floor_j,
        rationale,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn room_temp_order_of_magnitude() {
        let j = Landauer::room_temp_joules_per_bit();
        assert!(j > 2e-21 && j < 4e-21, "got {j}");
    }

    #[test]
    fn floor_scales_with_bits() {
        let one = landauer_floor_from_bits(1.0);
        let ten = landauer_floor_from_bits(10.0);
        assert!((ten.floor_j / one.floor_j - 10.0).abs() < 1e-9);
    }
}
