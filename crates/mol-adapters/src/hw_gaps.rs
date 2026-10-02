//! Soft-ref sims for the eight Periodic Stack HW Gaps.
//!
//! Cells `physical_settle` … `photonic_mzi` stay **Gap** markers in
//! [`mol_core::PeriodicStack`] (silicon still fires `primitive_gap`). This
//! module advances them with **classical software-reference sims / adapters**:
//! estimated joules only, `measured_j=None`, `stage_c_measured=false`,
//! `board_synth_claimed=false`, `silicon_claimed=false`.
//!
//! Never invent RAPL/NVML/board package joules. Soft-ref ≠ Present silicon.

use mol_core::{BOARD_SYNTH_CLAIMED, Floor, FloorKind};
use serde::{Deserialize, Serialize};

/// The eight HW Gap ids (Periodic Stack cells 259–266).
pub const HW_GAP_IDS: [&str; 8] = [
    "physical_settle",
    "reversible_rewrite",
    "ising_bind",
    "adiabatic_schedule",
    "ferric_efa_cert",
    "quantum_gate_ops",
    "analog_crossbar_mac",
    "photonic_mzi",
];

/// Named HW Gap soft-ref sim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HwGapId {
    /// Energy-landscape settle + certify (classical soft-ref).
    PhysicalSettle,
    /// Reversible / info-preserving rewrite (classical soft-ref).
    ReversibleRewrite,
    /// Tiny Ising / QUBO energy bind (classical soft-ref).
    IsingBind,
    /// Linear adiabatic schedule parameter s(t) (classical soft-ref).
    AdiabaticSchedule,
    /// Soft-ref Ferric-shaped EFA certificate (not on-device Ferric).
    FerricEfaCert,
    /// Tiny 1–2 qubit gate ops via classical statevector soft-ref.
    QuantumGateOps,
    /// Soft-ref analog crossbar MAC y=Wx.
    AnalogCrossbarMac,
    /// Soft-ref photonic Mach–Zehnder interferometer 2×2.
    PhotonicMzi,
}

impl HwGapId {
    /// Snake name matching Periodic Stack Gap cells.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PhysicalSettle => "physical_settle",
            Self::ReversibleRewrite => "reversible_rewrite",
            Self::IsingBind => "ising_bind",
            Self::AdiabaticSchedule => "adiabatic_schedule",
            Self::FerricEfaCert => "ferric_efa_cert",
            Self::QuantumGateOps => "quantum_gate_ops",
            Self::AnalogCrossbarMac => "analog_crossbar_mac",
            Self::PhotonicMzi => "photonic_mzi",
        }
    }

    /// Parse snake / hyphen name.
    pub fn parse(s: &str) -> Option<Self> {
        let n = s.trim().to_ascii_lowercase().replace('-', "_");
        match n.as_str() {
            "physical_settle" => Some(Self::PhysicalSettle),
            "reversible_rewrite" => Some(Self::ReversibleRewrite),
            "ising_bind" | "ising" => Some(Self::IsingBind),
            "adiabatic_schedule" | "adiabatic" => Some(Self::AdiabaticSchedule),
            "ferric_efa_cert" | "ferric" => Some(Self::FerricEfaCert),
            "quantum_gate_ops" | "quantum" => Some(Self::QuantumGateOps),
            "analog_crossbar_mac" | "analog" => Some(Self::AnalogCrossbarMac),
            "photonic_mzi" | "photonic" => Some(Self::PhotonicMzi),
            _ => None,
        }
    }

    /// All eight, in stack order.
    pub const fn all() -> [Self; 8] {
        [
            Self::PhysicalSettle,
            Self::ReversibleRewrite,
            Self::IsingBind,
            Self::AdiabaticSchedule,
            Self::FerricEfaCert,
            Self::QuantumGateOps,
            Self::AnalogCrossbarMac,
            Self::PhotonicMzi,
        ]
    }
}

/// Soft-ref sim outcome — never claims silicon meters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HwGapSoftResult {
    /// Gap id snake name.
    pub gap_id: String,
    /// allow | refuse | settled
    pub decision: String,
    /// Human / machine summary of the soft-ref answer.
    pub summary: String,
    /// Catalog / sim estimated joules (≠ measured).
    pub estimated_j: f64,
    /// Always None on soft-ref — no fake meters.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub measured_j: Option<f64>,
    /// Always false until a real Stage C / board meter exists.
    pub stage_c_measured: bool,
    /// Always false.
    pub board_synth_claimed: bool,
    /// Always false — soft-ref ≠ silicon Present.
    pub silicon_claimed: bool,
    /// Optional MoL floor on refuse.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub floor: Option<Floor>,
    /// Honesty note.
    pub note: String,
}

impl HwGapSoftResult {
    fn ok(id: HwGapId, decision: &str, summary: impl Into<String>, estimated_j: f64) -> Self {
        Self {
            gap_id: id.as_str().into(),
            decision: decision.into(),
            summary: summary.into(),
            estimated_j,
            measured_j: None,
            stage_c_measured: false,
            board_synth_claimed: BOARD_SYNTH_CLAIMED,
            silicon_claimed: false,
            floor: None,
            note: format!(
                "soft-ref sim for {}; Gap cell retained (not Present silicon); estimates≠measured_j; stage_c_measured=false",
                id.as_str()
            ),
        }
    }

    fn refuse(id: HwGapId, reason: impl Into<String>, estimated_j: f64) -> Self {
        let reason = reason.into();
        let floor = Floor::new(
            "primitive_gap",
            FloorKind::PrimitiveGap,
            format!("HW Gap soft-ref refuse ({}) — {}", id.as_str(), reason),
        );
        Self {
            gap_id: id.as_str().into(),
            decision: "refuse".into(),
            summary: reason,
            estimated_j,
            measured_j: None,
            stage_c_measured: false,
            board_synth_claimed: BOARD_SYNTH_CLAIMED,
            silicon_claimed: false,
            floor: Some(floor),
            note: format!(
                "soft-ref sim refuse for {}; silicon still Gap; stage_c_measured=false",
                id.as_str()
            ),
        }
    }
}

/// Inventory row for one HW Gap soft-ref sim.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HwGapInventoryRow {
    /// Gap id.
    pub gap_id: String,
    /// Soft-ref sim wired.
    pub soft_ref_sim: bool,
    /// Periodic Stack cell remains Gap (not Present).
    pub stack_status: String,
    /// Always false.
    pub silicon_claimed: bool,
    /// Always false.
    pub stage_c_measured: bool,
    /// Always None path for meters.
    pub measured_j: Option<f64>,
}

/// Soft-ref inventory over all eight HW Gaps.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HwGapInventory {
    /// Rows.
    pub gaps: Vec<HwGapInventoryRow>,
    /// Count of wired soft-ref sims (should be 8).
    pub soft_ref_count: usize,
    /// Always false.
    pub stage_c_measured: bool,
    /// Always false.
    pub board_synth_claimed: bool,
    /// Honesty.
    pub note: String,
}

/// Probe inventory: all eight soft-ref sims present; silicon still Gap.
pub fn probe_hw_gaps() -> HwGapInventory {
    let gaps: Vec<_> = HwGapId::all()
        .into_iter()
        .map(|id| HwGapInventoryRow {
            gap_id: id.as_str().into(),
            soft_ref_sim: true,
            stack_status: "gap".into(),
            silicon_claimed: false,
            stage_c_measured: false,
            measured_j: None,
        })
        .collect();
    let soft_ref_count = gaps.len();
    HwGapInventory {
        gaps,
        soft_ref_count,
        stage_c_measured: false,
        board_synth_claimed: BOARD_SYNTH_CLAIMED,
        note: "8 HW Gap soft-ref sims wired; Periodic Stack cells stay Gap; estimates≠measured_j; stage_c_measured=false until real meters"
            .into(),
    }
}

/// Run a soft-ref sim for one HW Gap. Optional JSON-ish params via `hint`.
pub fn run_hw_gap_soft(id: HwGapId, hint: &str) -> HwGapSoftResult {
    let h = hint.to_ascii_lowercase();
    // Refuse fake meter / silicon claims up front.
    if h.contains("stage_c_measured=true")
        || h.contains("measured_j=")
        || h.contains("silicon_claimed=true")
        || h.contains("board meter claim")
        || h.contains("claim silicon")
    {
        return HwGapSoftResult::refuse(
            id,
            "refuse inventing measured_j / stage_c_measured / silicon Present",
            1e-12,
        );
    }

    match id {
        HwGapId::PhysicalSettle => sim_physical_settle(&h),
        HwGapId::ReversibleRewrite => sim_reversible_rewrite(&h),
        HwGapId::IsingBind => sim_ising_bind(&h),
        HwGapId::AdiabaticSchedule => sim_adiabatic_schedule(&h),
        HwGapId::FerricEfaCert => sim_ferric_efa_cert(&h),
        HwGapId::QuantumGateOps => sim_quantum_gate_ops(&h),
        HwGapId::AnalogCrossbarMac => sim_analog_crossbar_mac(&h),
        HwGapId::PhotonicMzi => sim_photonic_mzi(&h),
    }
}

/// Run by snake name.
pub fn run_hw_gap_soft_named(name: &str, hint: &str) -> Option<HwGapSoftResult> {
    HwGapId::parse(name).map(|id| run_hw_gap_soft(id, hint))
}

/// Advance all eight: run each soft-ref sim once; assert honesty flags.
pub fn advance_all_hw_gaps_soft() -> Vec<HwGapSoftResult> {
    HwGapId::all()
        .into_iter()
        .map(|id| run_hw_gap_soft(id, "soft-ref advance demo"))
        .collect()
}

fn sim_physical_settle(h: &str) -> HwGapSoftResult {
    let id = HwGapId::PhysicalSettle;
    if h.contains("will not settle") || h.contains("diverge") || h.contains("refuse") {
        return HwGapSoftResult::refuse(id, "landscape will not settle (soft-ref)", 5e-11);
    }
    // Classical soft-ref: descend E = Σ s_i^2 toward 0 (dummy fixed point).
    let mut s = [1.0_f64, -0.5, 0.25, -0.125];
    let mut steps = 0u32;
    for _ in 0..16 {
        let mut changed = false;
        for v in &mut s {
            let nxt = *v * 0.5;
            if (nxt - *v).abs() > 1e-9 {
                changed = true;
            }
            *v = nxt;
        }
        steps += 1;
        if !changed {
            break;
        }
    }
    let e: f64 = s.iter().map(|x| x * x).sum();
    HwGapSoftResult::ok(
        id,
        "settled",
        format!("soft-ref physical_settle steps={steps} E≈{e:.3e} (classical; not silicon)"),
        5e-11 * steps as f64,
    )
}

fn sim_reversible_rewrite(h: &str) -> HwGapSoftResult {
    let id = HwGapId::ReversibleRewrite;
    // Bennett-style: (a,b) → (a, a⊕b) is reversible; soft-ref only.
    let a: u64 = if h.contains("a=0") { 0 } else { 0b1011 };
    let b: u64 = if h.contains("b=0") { 0 } else { 0b0101 };
    let b2 = a ^ b;
    let restored_b = a ^ b2;
    if restored_b != b {
        return HwGapSoftResult::refuse(id, "reversible rewrite failed identity check", 1e-12);
    }
    HwGapSoftResult::ok(
        id,
        "allow",
        format!("soft-ref reversible XOR rewrite a={a:#b} b={b:#b} → b'={b2:#b}; restore ok"),
        1e-12,
    )
}

fn sim_ising_bind(h: &str) -> HwGapSoftResult {
    let id = HwGapId::IsingBind;
    // Tiny 1D Ising N=4, J=1, h=0 — enumerate ground energy.
    let n = 4usize;
    let mut best_e = f64::INFINITY;
    let mut best_cfg = 0u32;
    for cfg in 0..(1u32 << n) {
        let mut e = 0.0;
        for i in 0..n {
            let si = if (cfg >> i) & 1 == 1 { 1.0 } else { -1.0 };
            let sj = if (cfg >> ((i + 1) % n)) & 1 == 1 {
                1.0
            } else {
                -1.0
            };
            e += -si * sj; // J=1 ferromagnetic
        }
        if e < best_e {
            best_e = e;
            best_cfg = cfg;
        }
    }
    if h.contains("force_refuse") {
        return HwGapSoftResult::refuse(id, "ising_bind force_refuse", 1e-10);
    }
    HwGapSoftResult::ok(
        id,
        "allow",
        format!("soft-ref Ising N={n} ground E={best_e} cfg={best_cfg:04b} (classical enum)"),
        1e-10,
    )
}

fn sim_adiabatic_schedule(h: &str) -> HwGapSoftResult {
    let id = HwGapId::AdiabaticSchedule;
    let steps = if h.contains("steps=1") { 1 } else { 8 };
    let mut schedule = Vec::with_capacity(steps);
    for t in 0..=steps {
        let s = t as f64 / steps as f64;
        schedule.push(s);
    }
    // Toy gap estimate: min(s,1-s)+ε — soft-ref only.
    let min_gap = schedule
        .iter()
        .map(|s| s.min(1.0 - s) + 1e-3)
        .fold(f64::INFINITY, f64::min);
    HwGapSoftResult::ok(
        id,
        "allow",
        format!(
            "soft-ref adiabatic s(t) steps={steps} min_gap_est={min_gap:.4} (not annealer HW)"
        ),
        2e-11 * steps as f64,
    )
}

fn sim_ferric_efa_cert(h: &str) -> HwGapSoftResult {
    let id = HwGapId::FerricEfaCert;
    if h.contains("diverge") || h.contains("spoof") || h.contains("uncertified") {
        return HwGapSoftResult::refuse(
            id,
            "Ferric soft-ref EFA refuse (diverge/spoof); on-device Ferric OUT OF PROOF SCOPE",
            1e-9,
        );
    }
    HwGapSoftResult::ok(
        id,
        "allow",
        "soft-ref Ferric-shaped EFA certificate allow; not path-dep'd Ferric; no robot meters",
        1e-9,
    )
}

fn sim_quantum_gate_ops(h: &str) -> HwGapSoftResult {
    let id = HwGapId::QuantumGateOps;
    // Classical statevector soft-ref: |0⟩ --H--> (|0⟩+|1⟩)/√2
    let s2 = std::f64::consts::FRAC_1_SQRT_2;
    let (a0, a1) = if h.contains("x_gate") {
        (0.0, 1.0) // X|0⟩ = |1⟩
    } else {
        (s2, s2) // H|0⟩
    };
    let norm = (a0 * a0 + a1 * a1).sqrt();
    if (norm - 1.0).abs() > 1e-9 {
        return HwGapSoftResult::refuse(id, "statevector norm failed", 1e-12);
    }
    HwGapSoftResult::ok(
        id,
        "allow",
        format!("soft-ref 1-qubit gate amps=[{a0:.4},{a1:.4}] norm={norm:.4} (classical SV; not quantum HW)"),
        5e-12,
    )
}

fn sim_analog_crossbar_mac(h: &str) -> HwGapSoftResult {
    let id = HwGapId::AnalogCrossbarMac;
    // y = W x with 2×2 soft-ref.
    let w = [[1.0, 0.5], [0.0, 1.0]];
    let x = if h.contains("x=0") {
        [0.0, 0.0]
    } else {
        [1.0, 2.0]
    };
    let y0 = w[0][0] * x[0] + w[0][1] * x[1];
    let y1 = w[1][0] * x[0] + w[1][1] * x[1];
    HwGapSoftResult::ok(
        id,
        "allow",
        format!("soft-ref crossbar MAC y=[{y0},{y1}] (digital stand-in; not analog silicon)"),
        2e-11,
    )
}

fn sim_photonic_mzi(h: &str) -> HwGapSoftResult {
    let id = HwGapId::PhotonicMzi;
    // 50/50 MZI beamsplitter soft-ref: [[t, i r],[i r, t]] with t=r=1/√2
    let t = std::f64::consts::FRAC_1_SQRT_2;
    let r = t;
    let (in0, in1) = if h.contains("port1") {
        (0.0, 1.0)
    } else {
        (1.0, 0.0)
    };
    // Real-part soft-ref (ignore phase for honesty of classical stand-in):
    let out0 = t * in0 - r * in1;
    let out1 = r * in0 + t * in1;
    let power = out0 * out0 + out1 * out1;
    HwGapSoftResult::ok(
        id,
        "allow",
        format!(
            "soft-ref photonic MZI out≈[{out0:.4},{out1:.4}] power={power:.4} (classical TM; not photonic silicon)"
        ),
        3e-12,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eight_ids_match_stack_names() {
        assert_eq!(HwGapId::all().len(), 8);
        assert_eq!(HW_GAP_IDS.len(), 8);
        for (a, b) in HwGapId::all().iter().zip(HW_GAP_IDS.iter()) {
            assert_eq!(&a.as_str(), b);
        }
    }

    #[test]
    fn inventory_honest() {
        let inv = probe_hw_gaps();
        assert_eq!(inv.soft_ref_count, 8);
        assert!(!inv.stage_c_measured);
        assert!(!inv.board_synth_claimed);
        for g in &inv.gaps {
            assert!(g.soft_ref_sim);
            assert_eq!(g.stack_status, "gap");
            assert!(!g.silicon_claimed);
            assert!(g.measured_j.is_none());
        }
    }

    #[test]
    fn advance_all_never_claims_meters() {
        for r in advance_all_hw_gaps_soft() {
            assert!(r.measured_j.is_none(), "{}", r.gap_id);
            assert!(!r.stage_c_measured, "{}", r.gap_id);
            assert!(!r.board_synth_claimed, "{}", r.gap_id);
            assert!(!r.silicon_claimed, "{}", r.gap_id);
            assert_ne!(r.decision, "refuse", "default demo should allow/settle: {}", r.gap_id);
        }
    }

    #[test]
    fn refuse_fake_meter_claim() {
        let r = run_hw_gap_soft(HwGapId::PhotonicMzi, "stage_c_measured=true board meter claim");
        assert_eq!(r.decision, "refuse");
        assert!(r.measured_j.is_none());
        assert!(!r.stage_c_measured);
    }

    #[test]
    fn physical_settle_can_refuse() {
        let r = run_hw_gap_soft(HwGapId::PhysicalSettle, "will not settle");
        assert_eq!(r.decision, "refuse");
        assert!(r.floor.is_some());
    }
}
