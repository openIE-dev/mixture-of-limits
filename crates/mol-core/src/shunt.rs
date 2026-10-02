//! Tier-2 shunt / package meter HAL.
//!
//! Only a real shunt reading may populate `measured_j`. The default stub never
//! invents; FPGA Stage C stays `board_synth_claimed=false` / measured absent.

use serde::{Deserialize, Serialize};

use crate::energy::{Joules, MeasureSource};
use crate::BOARD_SYNTH_CLAIMED;

/// Tier-2 shunt HAL — package joule reading when a certified meter is present.
pub trait ShuntHal: Send + Sync {
    /// Probe capability (no invent).
    fn probe(&self) -> ShuntCapability;
    /// Read package joules over a window. `None` when unavailable — never invent.
    fn read_package_j(&self, window_ms: u64) -> Option<ShuntReading>;
}

/// Capability report for Tier-2 shunt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShuntCapability {
    /// True only when a real shunt path is present.
    pub available: bool,
    /// Source label.
    pub source: String,
    /// Detail (why unavailable / which path).
    pub detail: String,
    /// Always false on soft-ref / stub.
    pub board_synth_claimed: bool,
}

impl ShuntCapability {
    /// Unavailable stub.
    pub fn unavailable(detail: impl Into<String>) -> Self {
        Self {
            available: false,
            source: "shunt_stub".into(),
            detail: detail.into(),
            board_synth_claimed: BOARD_SYNTH_CLAIMED,
        }
    }
}

/// One Tier-2 reading.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShuntReading {
    /// Package joules from the shunt.
    pub measured_j: Joules,
    /// Must be [`MeasureSource::Shunt`].
    pub source: MeasureSource,
    /// Window milliseconds.
    pub window_ms: u64,
    /// Always false unless a live board path claims synth (MoL soft-ref: false).
    pub board_synth_claimed: bool,
    /// Detail.
    pub detail: String,
}

impl ShuntReading {
    /// Construct a honest shunt reading (caller must have a real meter).
    pub fn package(measured_j: f64, window_ms: u64, detail: impl Into<String>) -> Self {
        Self {
            measured_j: Joules::new(measured_j),
            source: MeasureSource::Shunt,
            window_ms,
            board_synth_claimed: BOARD_SYNTH_CLAIMED,
            detail: detail.into(),
        }
    }
}

/// Default Tier-2 stub — never invents `measured_j`.
#[derive(Debug, Default, Clone, Copy)]
pub struct StubShuntHal;

impl ShuntHal for StubShuntHal {
    fn probe(&self) -> ShuntCapability {
        ShuntCapability::unavailable(
            "Tier-2 shunt HAL stub: no certified package meter attached; measured_j stays None; board_synth_claimed=false (FPGA Stage C optional stays unmetered)",
        )
    }

    fn read_package_j(&self, _window_ms: u64) -> Option<ShuntReading> {
        None
    }
}

/// Fixture shunt for tests only — still labeled Shunt; never used as soft-ref invent.
#[derive(Debug, Clone)]
pub struct FixtureShuntHal {
    /// Package joules to return when `armed`.
    pub joules: f64,
    /// When false, behaves like stub (unavailable).
    pub armed: bool,
}

impl FixtureShuntHal {
    /// Armed fixture.
    pub fn armed(joules: f64) -> Self {
        Self { joules, armed: true }
    }
}

impl ShuntHal for FixtureShuntHal {
    fn probe(&self) -> ShuntCapability {
        if self.armed {
            ShuntCapability {
                available: true,
                source: "shunt_fixture".into(),
                detail: "test fixture shunt (not board synth)".into(),
                board_synth_claimed: BOARD_SYNTH_CLAIMED,
            }
        } else {
            ShuntCapability::unavailable("fixture shunt disarmed")
        }
    }

    fn read_package_j(&self, window_ms: u64) -> Option<ShuntReading> {
        if !self.armed {
            return None;
        }
        Some(ShuntReading::package(
            self.joules,
            window_ms,
            "fixture shunt package reading",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_never_invents() {
        let s = StubShuntHal;
        assert!(!s.probe().available);
        assert!(!s.probe().board_synth_claimed);
        assert!(s.read_package_j(10).is_none());
    }

    #[test]
    fn fixture_reads_when_armed() {
        let s = FixtureShuntHal::armed(0.42);
        let r = s.read_package_j(5).unwrap();
        assert_eq!(r.source, MeasureSource::Shunt);
        assert!((r.measured_j.0 - 0.42).abs() < 1e-12);
        assert!(!r.board_synth_claimed);
    }
}
