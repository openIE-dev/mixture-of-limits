//! Joules wrapper and honesty labels for estimated vs measured energy.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Energy in joules (f64 wrapper for clarity in receipts).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Joules(pub f64);

impl Joules {
    /// Zero energy.
    pub const ZERO: Self = Self(0.0);

    /// Construct from raw joules.
    pub const fn new(j: f64) -> Self {
        Self(j)
    }

    /// Microjoules convenience.
    pub fn microjoules(self) -> f64 {
        self.0 * 1e6
    }

    /// Add energies.
    pub fn saturating_add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }

    /// True if finite and non-negative.
    pub fn is_valid(self) -> bool {
        self.0.is_finite() && self.0 >= 0.0
    }
}

impl Default for Joules {
    fn default() -> Self {
        Self::ZERO
    }
}

impl fmt::Display for Joules {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.abs() < 1e-12 {
            write!(f, "{:.3e} J", self.0)
        } else if self.0.abs() < 1e-3 {
            write!(f, "{:.3} µJ", self.microjoules())
        } else {
            write!(f, "{:.6} J", self.0)
        }
    }
}

/// How an energy number was obtained — honesty contract.
///
/// **Never invent RAPL/NVML.** Catalog / analytical / surrogate are estimates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MeasureSource {
    /// No soft measurement attempted / available.
    #[default]
    Unavailable,
    /// Analytical catalog / OpCounter × constants (WCA-style surrogate).
    CatalogSurrogate,
    /// Tier-cost table estimate (MoL cascade gear table).
    CascadeEstimate,
    /// Soft RAPL read (only when real sensors present — MoL never fabricates).
    Rapl,
    /// Soft NVML read (only when real sensors present).
    Nvml,
    /// CPU time × proxy watts (labeled, not silicon meter).
    CpuProxy,
    /// macOS powermetrics sample (plist or text). Only when the tool returned numbers.
    Powermetrics,
    /// macOS IOReport / IOKit Energy Model counters. Only when a channel had a known unit.
    IoReport,
    /// macOS SMC system-total power (`PSTR` watts × sample window). Not a rail sum.
    ///
    /// Apple exposes `PSTR` via IOKit `AppleSMC` without sudo; macmon labels it
    /// `sys_power`. There is no Energy Model "Package" channel on current silicon —
    /// this is the honest package-equivalent for `meter_required`, distinct from
    /// IOReport CPU/GPU/ANE/DRAM rails and from inventing `cpu+gpu+ane`.
    Smc,
    /// macOS energy (generic). Prefer [`Self::Powermetrics`], [`Self::IoReport`], or [`Self::Smc`].
    MacOsEnergy,
    /// Windows energy estimation / ETW. Not implemented — probe stays unavailable.
    WindowsEnergy,
    /// Tier-2 certified shunt / package meter. Only when a real HAL reading exists.
    Shunt,
}

impl MeasureSource {
    /// True iff this source claims a soft hardware/process measurement.
    pub const fn is_measured(self) -> bool {
        matches!(
            self,
            Self::Rapl
                | Self::Nvml
                | Self::CpuProxy
                | Self::Powermetrics
                | Self::IoReport
                | Self::Smc
                | Self::MacOsEnergy
                | Self::WindowsEnergy
                | Self::Shunt
        )
    }

    /// Wire / receipt label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Unavailable => "unavailable",
            Self::CatalogSurrogate => "catalog_surrogate",
            Self::CascadeEstimate => "cascade_estimate",
            Self::Rapl => "rapl",
            Self::Nvml => "nvml",
            Self::CpuProxy => "cpu_proxy",
            Self::Powermetrics => "powermetrics",
            Self::IoReport => "ioreport",
            Self::Smc => "smc",
            Self::MacOsEnergy => "macos_energy",
            Self::WindowsEnergy => "windows_energy",
            Self::Shunt => "shunt",
        }
    }
}

impl fmt::Display for MeasureSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// Kind of estimate (always distinct from measured).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EstimateKind {
    /// Tier table / analytical constants.
    #[default]
    Analytical,
    /// μ calib corpus when present.
    Calib,
    /// Manual / test fixture.
    Fixture,
}
