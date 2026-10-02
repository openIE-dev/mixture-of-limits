//! Optional OS energy meter (`energy-meter`, alias `os-meter`).
//!
//! **Feature off (prove default):** capability and samples stay unavailable —
//! `measured_j=None`, no component joules. Soft-ref close does not require a meter.
//!
//! **Feature on:**
//! - **Linux:** RAPL powercap `/sys/class/powercap` (`energy_uj` delta). Package /
//!   core / dram / gpu domains only — unknown zones are not relabeled.
//! - **macOS:** IOReport Energy Model rails (CPU, GPU, ANE, DRAM; no Package channel
//!   on current silicon) + SMC `PSTR` system-total watts × window for package
//!   `measured_j` (no sudo). Then `powermetrics` if root (`combined_power`). Never
//!   invent package by summing rails (`all_power` / macmon-style).
//! - **Windows:** later stub — always unavailable (no ETW joules).
//!
//! Honesty: a component is `Some` only when that counter was actually read.
//! Missing DRAM/ANE is omitted, not zero. Package joules are **not** the sum of
//! rails. VM / permission failure → `measured_j=None`, `measure_source=unavailable`.

use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::energy::{Joules, MeasureSource};

#[cfg(all(feature = "energy-meter", target_os = "macos"))]
mod apple;

/// Compile-time flag — true when the `energy-meter` Cargo feature is enabled.
pub const ENERGY_METER_ENABLED: bool = cfg!(feature = "energy-meter");

/// Honesty note for operators / receipts.
pub const METER_HONESTY_NOTE: &str =
    "OS meter: measured_j=Some only on real probe success; VM/no-permission → None (never invent)";

/// Operator note for the Mac path (sudo / powermetrics / IOReport).
pub const MACOS_METER_HELP: &str = "\
macOS component meter (CPU, GPU, ANE, DRAM, package) — feature energy-meter / os-meter:
  1) IOReport Energy Model rails (CPU/GPU/ANE/DRAM) — no sudo; no Package/Combined channel
     on M-series Energy Model (confirmed across all IOReport groups).
  2) SMC key PSTR (system total power, watts) × sample window → package measured_j — no sudo.
     Same sensor macmon exposes as sys_power. Not a sum of Energy Model rails.
  3) powermetrics (root) still provides cpu/gpu/ane/dram + combined_power when available:
       sudo powermetrics -f plist -i 1000 -n 1 --samplers cpu_power,gpu_power -a 0
  Absent rail or failed probe → that joule stays None. Package is never cpu+gpu+ane+dram.
Windows: ETW / Energy Estimation Engine is a later stub; this build returns unavailable.";

/// SoC rail / domain a meter may report.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MeterComponent {
    /// CPU cluster / core energy.
    Cpu,
    /// GPU energy.
    Gpu,
    /// Apple Neural Engine.
    Ane,
    /// DRAM / memory energy.
    Dram,
    /// Package or powermetrics combined power (explicit counter only).
    Package,
}

impl MeterComponent {
    /// Wire label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Cpu => "cpu",
            Self::Gpu => "gpu",
            Self::Ane => "ane",
            Self::Dram => "dram",
            Self::Package => "package",
        }
    }
}

impl std::fmt::Display for MeterComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

/// One component's measured joules. Present only when that counter was read.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComponentJoules {
    /// Which rail.
    pub component: MeterComponent,
    /// Joules over the sample window (0 is a real sensor zero, not a filler).
    pub joules: Joules,
    /// Probe that produced this number (`rapl` / `powermetrics` / `ioreport`).
    pub measure_source: MeasureSource,
}

/// Result of a capability probe (does not invent joules).
#[derive(Debug, Clone, PartialEq)]
pub struct MeterCapability {
    /// True iff a platform counter/API appears usable for interval measurement.
    pub available: bool,
    /// Preferred measure source if a sample succeeds.
    pub source: MeasureSource,
    /// Human-readable detail (path, permission, why unavailable).
    pub detail: String,
    /// Platform tag (`linux_rapl`, `macos_ioreport`, `windows`, `disabled`, …).
    pub platform: &'static str,
}

impl MeterCapability {
    /// Unavailable stub.
    pub fn unavailable(platform: &'static str, detail: impl Into<String>) -> Self {
        Self {
            available: false,
            source: MeasureSource::Unavailable,
            detail: detail.into(),
            platform,
        }
    }
}

/// One interval sample — joules only when a real counter was read.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MeterSample {
    /// Package/combined joules when that counter exists. Not a sum of rails.
    pub measured_j: Option<Joules>,
    /// Labeled source (`rapl` / `powermetrics` / `ioreport`) or `unavailable`.
    pub source: MeasureSource,
    /// Per-component joules. Omitted components were not exposed.
    pub components: Vec<ComponentJoules>,
    /// Sample window milliseconds.
    pub window_ms: u64,
    /// Detail / counter path.
    pub detail: String,
}

impl MeterSample {
    /// Honest empty sample.
    pub fn unavailable(window_ms: u64, detail: impl Into<String>) -> Self {
        Self {
            measured_j: None,
            source: MeasureSource::Unavailable,
            components: Vec::new(),
            window_ms,
            detail: detail.into(),
        }
    }

    /// Joules for one component, if that rail was read.
    pub fn component(&self, component: MeterComponent) -> Option<Joules> {
        self.components
            .iter()
            .find(|c| c.component == component)
            .map(|c| c.joules)
    }

    /// True when this sample may be stamped on a receipt.
    pub fn records_measurement(&self) -> bool {
        self.honesty_ok()
            && self.source.is_measured()
            && (self.measured_j.is_some() || !self.components.is_empty())
    }

    /// Honesty: Some ⇒ measured source + finite ≥0; None ⇒ no fake source.
    ///
    /// Each component must carry its own measured source (IOReport rails may
    /// coexist with an SMC package). Package `measured_j` must match the package
    /// component, and `self.source` must be that package source when package is
    /// present. Unavailable samples carry no numbers.
    pub fn honesty_ok(&self) -> bool {
        let numbers = self.measured_j.is_some() || !self.components.is_empty();
        if !numbers {
            return !self.source.is_measured() && self.components.is_empty() && self.measured_j.is_none();
        }
        if !self.source.is_measured() {
            return false;
        }
        if let Some(j) = self.measured_j {
            if !j.is_valid() {
                return false;
            }
        }
        for c in &self.components {
            if !c.measure_source.is_measured() || !c.joules.is_valid() {
                return false;
            }
        }
        if let Some(pkg_c) = self
            .components
            .iter()
            .find(|c| c.component == MeterComponent::Package)
        {
            if self.measured_j != Some(pkg_c.joules) {
                return false;
            }
            // Primary source is the package probe (smc / powermetrics / rapl / …).
            if self.source != pkg_c.measure_source {
                return false;
            }
        } else if self.measured_j.is_some() {
            // measured_j is the package/combined counter only.
            return false;
        }
        true
    }
}

/// Probe whether an OS energy counter is exposed (no joule invention).
pub fn probe_meter_capability() -> MeterCapability {
    if !ENERGY_METER_ENABLED {
        return MeterCapability::unavailable(
            "disabled",
            "energy-meter feature off — measured_j stays None (soft-ref / prove default)",
        );
    }
    os_probe()
}

/// Measure energy over `window` by differencing OS counters.
pub fn measure_energy_window(window: Duration) -> MeterSample {
    let window_ms = window.as_millis() as u64;
    if !ENERGY_METER_ENABLED {
        return MeterSample::unavailable(
            window_ms,
            "energy-meter feature off — measured_j=None",
        );
    }
    os_measure(window, window_ms)
}

/// Measure energy over a window that **overlaps** `work` (same wall-clock interval).
///
/// Used so Metal/wgpu kernel dispatch and SMC/RAPL sampling share one window —
/// `measured_j` on the kernel [`crate::compute::ComputeStepReceipt`] is real package
/// joules, never invented / never a rail sum. Feature off → work still runs;
/// sample stays `unavailable` (`measured_j=None`).
pub fn measure_energy_during<F, T>(min_window: Duration, work: F) -> (T, MeterSample)
where
    F: FnOnce() -> T,
{
    let window_ms = min_window.as_millis() as u64;
    if !ENERGY_METER_ENABLED {
        let out = work();
        return (
            out,
            MeterSample::unavailable(
                window_ms,
                "energy-meter feature off — measured_j=None (work ran unmetered)",
            ),
        );
    }
    os_measure_during(min_window, window_ms, work)
}

/// Convenience status line for CLI.
pub fn meter_status_line() -> String {
    let cap = probe_meter_capability();
    format!(
        "energy-meter feature={} platform={} available={} source={} — {} [{}]",
        ENERGY_METER_ENABLED,
        cap.platform,
        cap.available,
        cap.source.label(),
        cap.detail,
        METER_HONESTY_NOTE
    )
}

/// One named energy reading before classification (IOReport channel or similar).
///
/// Used by the macOS IOReport probe (`energy-meter`) and by unit tests.
#[cfg_attr(not(all(feature = "energy-meter", target_os = "macos")), allow(dead_code))]
pub(crate) struct NamedJoule {
    pub name: String,
    pub joules: f64,
}

/// Map Energy Model channel names onto components. Does not invent a package sum.
///
/// Used by the macOS IOReport probe and by unit tests (so Linux lib builds may not call it).
#[cfg_attr(not(all(feature = "energy-meter", target_os = "macos")), allow(dead_code))]
pub(crate) fn fold_named_joules(
    rows: &[NamedJoule],
    source: MeasureSource,
    window_ms: u64,
    how: &str,
) -> MeterSample {
    let mut cpu = Vec::new();
    let mut gpu_energy = Vec::new();
    let mut gpu_other = Vec::new();
    let mut ane = Vec::new();
    let mut dram = Vec::new();
    let mut package = Vec::new();
    for row in rows {
        let name = row.name.trim();
        if !row.joules.is_finite() || row.joules < 0.0 {
            continue;
        }
        if name == "CPU Energy" || name.ends_with("CPU Energy") {
            cpu.push(row.joules);
        } else if name == "GPU Energy" {
            gpu_energy.push(row.joules);
        } else if name.starts_with("GPU") && !name.contains("SRAM") {
            gpu_other.push(row.joules);
        } else if name.starts_with("ANE") {
            ane.push(row.joules);
        } else if name.starts_with("DRAM") {
            dram.push(row.joules);
        } else if name.eq_ignore_ascii_case("package")
            || name.contains("Package Energy")
            || name == "Combined Power"
        {
            package.push(row.joules);
        }
    }
    let gpu = if gpu_energy.is_empty() {
        gpu_other
    } else {
        gpu_energy
    };
    let mut components = Vec::new();
    push_sum(&mut components, MeterComponent::Cpu, &cpu, source);
    push_sum(&mut components, MeterComponent::Gpu, &gpu, source);
    push_sum(&mut components, MeterComponent::Ane, &ane, source);
    push_sum(&mut components, MeterComponent::Dram, &dram, source);
    push_sum(&mut components, MeterComponent::Package, &package, source);
    finish_sample(components, source, window_ms, how)
}

fn push_sum(
    out: &mut Vec<ComponentJoules>,
    component: MeterComponent,
    parts: &[f64],
    source: MeasureSource,
) {
    if parts.is_empty() {
        return;
    }
    let sum: f64 = parts.iter().sum();
    if !sum.is_finite() || sum < 0.0 {
        return;
    }
    out.push(ComponentJoules {
        component,
        joules: Joules::new(sum),
        measure_source: source,
    });
}

fn finish_sample(
    components: Vec<ComponentJoules>,
    source: MeasureSource,
    window_ms: u64,
    how: &str,
) -> MeterSample {
    let measured_j = components
        .iter()
        .find(|c| c.component == MeterComponent::Package)
        .map(|c| c.joules);
    if components.is_empty() {
        return MeterSample::unavailable(
            window_ms,
            format!("{how}: no mapped component counters — measured_j=None"),
        );
    }
    let labels: Vec<&str> = components.iter().map(|c| c.component.label()).collect();
    MeterSample {
        measured_j,
        source,
        components,
        window_ms,
        detail: format!(
            "{how} components=[{}] package_j={}",
            labels.join(","),
            measured_j
                .map(|j| format!("{:.6e}", j.0))
                .unwrap_or_else(|| "None".into())
        ),
    }
}


#[cfg_attr(not(any(test, all(feature = "energy-meter", target_os = "macos"))), allow(dead_code))]
/// Build a package sample from SMC `PSTR` average watts × window (fixture-safe).
///
/// `PSTR` is system total power (not an Energy Model Package channel and not a
/// rail sum). Used by the macOS live probe and by unit tests.
pub fn sample_from_smc_pstr_watts(avg_watts: f64, window_ms: u64) -> MeterSample {
    if !avg_watts.is_finite() || avg_watts < 0.0 || window_ms == 0 {
        return MeterSample::unavailable(
            window_ms,
            "smc PSTR: non-finite/negative watts or zero window — measured_j=None",
        );
    }
    let sec = window_ms as f64 / 1000.0;
    let j = avg_watts * sec;
    if !j.is_finite() || j < 0.0 {
        return MeterSample::unavailable(window_ms, "smc PSTR: joule product invalid — measured_j=None");
    }
    let components = vec![ComponentJoules {
        component: MeterComponent::Package,
        joules: Joules::new(j),
        measure_source: MeasureSource::Smc,
    }];
    finish_sample(
        components,
        MeasureSource::Smc,
        window_ms,
        &format!("smc PSTR avg_W={avg_watts:.6}"),
    )
}

#[cfg_attr(not(any(test, all(feature = "energy-meter", target_os = "macos"))), allow(dead_code))]
/// Merge IOReport component rails with an SMC (or other) package sample.
///
/// Package / `measured_j` / primary `source` come from `package`. Non-package
/// rails from `rails` keep their own `measure_source`. Never invents package
/// from rails when `package.measured_j` is None.
pub fn merge_rails_with_package(rails: &MeterSample, package: &MeterSample) -> MeterSample {
    if !package.honesty_ok() || package.measured_j.is_none() {
        return rails.clone();
    }
    let Some(pkg_c) = package
        .components
        .iter()
        .find(|c| c.component == MeterComponent::Package)
        .cloned()
    else {
        return rails.clone();
    };
    let mut components = vec![pkg_c];
    for c in &rails.components {
        if c.component != MeterComponent::Package {
            components.push(c.clone());
        }
    }
    let window_ms = package.window_ms.max(rails.window_ms);
    let detail = format!(
        "{} | rails: {}",
        package.detail,
        if rails.components.is_empty() {
            "none".into()
        } else {
            rails.detail.clone()
        }
    );
    let sample = MeterSample {
        measured_j: package.measured_j,
        source: package.source,
        components,
        window_ms,
        detail,
    };
    if sample.honesty_ok() {
        sample
    } else {
        // Fall back to package-only rather than a dishonest mix.
        package.clone()
    }
}

/// Parse `powermetrics` plist or text. Fixture-safe (no process spawn).
///
/// Energy keys are millijoules. `combined_power` / `* Power:` lines are
/// milliwatts (or watts) times the **reported** elapsed interval, else `window_ms`.
/// A missing component stays absent. Package is only the combined/package field.
pub fn parse_powermetrics_output(text: &str, window_ms: u64) -> MeterSample {
    let chunk = last_plist_chunk(text);
    if chunk.contains("<key>cpu_energy</key>")
        || chunk.contains("<key>gpu_energy</key>")
        || chunk.contains("<key>ane_energy</key>")
        || chunk.contains("<key>combined_power</key>")
        || chunk.contains("<plist")
    {
        return parse_powermetrics_plist(chunk, window_ms);
    }
    parse_powermetrics_text(text, window_ms)
}

fn last_plist_chunk(text: &str) -> &str {
    let mut best: &str = text;
    for part in text.split('\u{0}') {
        if part.contains("<key>cpu_energy</key>")
            || part.contains("<key>combined_power</key>")
            || part.contains("<key>ane_energy</key>")
        {
            best = part;
        }
    }
    best
}

fn parse_powermetrics_plist(xml: &str, window_ms: u64) -> MeterSample {
    let elapsed_s = plist_num(xml, "elapsed_ns")
        .filter(|ns| *ns > 0.0)
        .map(|ns| ns / 1e9)
        .or_else(|| {
            if window_ms > 0 {
                Some(window_ms as f64 / 1000.0)
            } else {
                None
            }
        });
    let mut components = Vec::new();
    push_mj(&mut components, MeterComponent::Cpu, plist_num(xml, "cpu_energy"));
    push_mj(&mut components, MeterComponent::Gpu, plist_num(xml, "gpu_energy"));
    push_mj(&mut components, MeterComponent::Ane, plist_num(xml, "ane_energy"));
    push_mj(&mut components, MeterComponent::Dram, plist_num(xml, "dram_energy"));
    if let Some(mw) = plist_num(xml, "combined_power") {
        if let Some(sec) = elapsed_s {
            let j = (mw / 1000.0) * sec;
            if j.is_finite() && j >= 0.0 {
                components.push(ComponentJoules {
                    component: MeterComponent::Package,
                    joules: Joules::new(j),
                    measure_source: MeasureSource::Powermetrics,
                });
            }
        }
    }
    if components.is_empty() {
        return MeterSample::unavailable(
            window_ms,
            "powermetrics plist had no cpu/gpu/ane/dram/combined fields — measured_j=None",
        );
    }
    finish_sample(
        components,
        MeasureSource::Powermetrics,
        window_ms,
        "powermetrics plist",
    )
}

fn push_mj(out: &mut Vec<ComponentJoules>, component: MeterComponent, mj: Option<f64>) {
    let Some(mj) = mj else {
        return;
    };
    let j = mj / 1000.0;
    if j.is_finite() && j >= 0.0 {
        out.push(ComponentJoules {
            component,
            joules: Joules::new(j),
            measure_source: MeasureSource::Powermetrics,
        });
    }
}

fn plist_num(xml: &str, key: &str) -> Option<f64> {
    let pat = format!("<key>{key}</key>");
    let mut rest = xml;
    while let Some(i) = rest.find(&pat) {
        let after = rest[i + pat.len()..].trim_start();
        if let Some(v) = read_xml_num(after) {
            return Some(v);
        }
        rest = &rest[i + pat.len()..];
    }
    None
}

fn read_xml_num(s: &str) -> Option<f64> {
    let s = s.trim_start();
    let (tag, end) = if s.starts_with("<integer>") {
        ("<integer>", "</integer>")
    } else if s.starts_with("<real>") {
        ("<real>", "</real>")
    } else {
        return None;
    };
    let body = s[tag.len()..].split(end).next()?.trim();
    body.parse().ok()
}

fn parse_powermetrics_text(text: &str, window_ms: u64) -> MeterSample {
    let elapsed_s = text_elapsed_s(text).or_else(|| {
        if window_ms > 0 {
            Some(window_ms as f64 / 1000.0)
        } else {
            None
        }
    });
    let mut components = Vec::new();
    for line in text.lines() {
        let Some(hit) = classify_power_line(line) else {
            continue;
        };
        let component = hit.component;
        let Some(j) = line_to_joules(&hit, elapsed_s) else {
            continue;
        };
        if components.iter().any(|c: &ComponentJoules| c.component == component) {
            continue;
        }
        components.push(ComponentJoules {
            component,
            joules: Joules::new(j),
            measure_source: MeasureSource::Powermetrics,
        });
    }
    if components.is_empty() {
        return MeterSample::unavailable(
            window_ms,
            "powermetrics text had no component power/energy lines — measured_j=None",
        );
    }
    finish_sample(
        components,
        MeasureSource::Powermetrics,
        window_ms,
        "powermetrics text",
    )
}

struct LineHit {
    component: MeterComponent,
    watts: Option<f64>,
    joules: Option<f64>,
}

fn classify_power_line(line: &str) -> Option<LineHit> {
    let lower = line.to_ascii_lowercase();
    let component = if lower.contains("combined power") || lower.contains("package power") || lower.contains("package energy")
    {
        MeterComponent::Package
    } else if lower.contains("dram power") || lower.contains("dram energy") {
        MeterComponent::Dram
    } else if lower.contains("ane power") || lower.contains("ane energy") {
        MeterComponent::Ane
    } else if lower.contains("gpu power") || lower.contains("gpu energy") {
        MeterComponent::Gpu
    } else if lower.contains("cpu power") || lower.contains("cpu energy") {
        MeterComponent::Cpu
    } else {
        return None;
    };
    let (value, unit) = number_and_unit(line)?;
    if !value.is_finite() || value < 0.0 {
        return None;
    }
    let (watts, joules) = match unit {
        Unit::Mw => (Some(value / 1000.0), None),
        Unit::W => (Some(value), None),
        Unit::Mj => (None, Some(value / 1000.0)),
        Unit::Uj => (None, Some(value / 1e6)),
        Unit::Nj => (None, Some(value / 1e9)),
        Unit::J => (None, Some(value)),
    };
    Some(LineHit {
        component,
        watts,
        joules,
    })
}

enum Unit {
    Mw,
    W,
    Mj,
    Uj,
    Nj,
    J,
}

fn number_and_unit(line: &str) -> Option<(f64, Unit)> {
    let lower = line.to_ascii_lowercase();
    let unit = if lower.contains("mw") || lower.contains("milliwatt") {
        Unit::Mw
    } else if lower.contains("mj") || lower.contains("millijoule") {
        Unit::Mj
    } else if lower.contains("uj") || lower.contains("µj") || lower.contains("μj") {
        Unit::Uj
    } else if lower.contains("nj") || lower.contains("nanojoule") {
        Unit::Nj
    } else if lower.contains(" j") || lower.ends_with('j') {
        Unit::J
    } else if lower.contains(" w") || lower.contains("watt") {
        Unit::W
    } else {
        return None;
    };
    let mut num = String::new();
    let mut seen = false;
    for ch in line.chars() {
        if ch.is_ascii_digit() || (ch == '.' && seen) {
            num.push(ch);
            seen = true;
        } else if seen {
            break;
        }
    }
    let value: f64 = num.parse().ok()?;
    Some((value, unit))
}

fn line_to_joules(hit: &LineHit, elapsed_s: Option<f64>) -> Option<f64> {
    if let Some(j) = hit.joules {
        return (j.is_finite() && j >= 0.0).then_some(j);
    }
    let watts = hit.watts?;
    let sec = elapsed_s?;
    let j = watts * sec;
    (j.is_finite() && j >= 0.0).then_some(j)
}

fn text_elapsed_s(text: &str) -> Option<f64> {
    let lower = text.to_ascii_lowercase();
    let idx = lower.find("ms elapsed")?;
    let before = &lower[..idx];
    let start = before
        .rfind(|c: char| !(c.is_ascii_digit() || c == '.'))
        .map(|i| i + 1)
        .unwrap_or(0);
    let ms: f64 = before[start..].trim().parse().ok()?;
    if ms > 0.0 {
        Some(ms / 1000.0)
    } else {
        None
    }
}

/// One RAPL domain counter (microjoules). Used by the live probe and fixtures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RaplCounter {
    /// Zone `name` file (`package-0`, `core`, `dram`, …).
    pub name: String,
    /// `energy_uj` reading.
    pub energy_uj: u128,
    /// `max_energy_range_uj` when the sysfs file exists (wrap handling).
    pub max_energy_uj: Option<u128>,
}

/// Delta two RAPL snapshots into component joules. No live sysfs.
///
/// Unknown names (`psys`, `uncore`, …) are ignored — not relabeled as package.
/// A wrapping counter without `max_energy_uj` is dropped, not invented.
pub fn sample_from_rapl_counters(
    before: &[RaplCounter],
    after: &[RaplCounter],
    window_ms: u64,
) -> MeterSample {
    let mut cpu = Vec::new();
    let mut gpu = Vec::new();
    let mut dram = Vec::new();
    let mut package = Vec::new();
    for b in before {
        let Some(a) = after.iter().find(|x| x.name == b.name) else {
            continue;
        };
        let Some(component) = rapl_component(&b.name) else {
            continue;
        };
        let Some(delta_uj) = rapl_delta(b.energy_uj, a.energy_uj, b.max_energy_uj.or(a.max_energy_uj))
        else {
            continue;
        };
        let j = delta_uj as f64 * 1.0e-6;
        if !j.is_finite() || j < 0.0 {
            continue;
        }
        match component {
            MeterComponent::Cpu => cpu.push(j),
            MeterComponent::Gpu => gpu.push(j),
            MeterComponent::Dram => dram.push(j),
            MeterComponent::Package => package.push(j),
            MeterComponent::Ane => {}
        }
    }
    let mut components = Vec::new();
    let src = MeasureSource::Rapl;
    push_sum(&mut components, MeterComponent::Cpu, &cpu, src);
    push_sum(&mut components, MeterComponent::Gpu, &gpu, src);
    push_sum(&mut components, MeterComponent::Dram, &dram, src);
    push_sum(&mut components, MeterComponent::Package, &package, src);
    if components.is_empty() {
        return MeterSample::unavailable(
            window_ms,
            "RAPL snapshot had no package/core/dram/gpu delta — measured_j=None",
        );
    }
    finish_sample(components, src, window_ms, "rapl powercap")
}

fn rapl_component(name: &str) -> Option<MeterComponent> {
    let n = name.trim().to_ascii_lowercase();
    if n.starts_with("package") {
        Some(MeterComponent::Package)
    } else if n == "core" || n.starts_with("core") {
        Some(MeterComponent::Cpu)
    } else if n.contains("dram") {
        Some(MeterComponent::Dram)
    } else if n.contains("gpu") {
        Some(MeterComponent::Gpu)
    } else {
        None
    }
}

fn rapl_delta(before: u128, after: u128, max_uj: Option<u128>) -> Option<u128> {
    if after >= before {
        Some(after - before)
    } else {
        let max = max_uj?;
        if max >= before {
            Some(max - before + after)
        } else {
            None
        }
    }
}

#[cfg(feature = "energy-meter")]
fn os_probe() -> MeterCapability {
    #[cfg(target_os = "linux")]
    {
        return linux::probe();
    }
    #[cfg(target_os = "macos")]
    {
        return apple::probe();
    }
    #[cfg(target_os = "windows")]
    {
        return MeterCapability::unavailable(
            "windows",
            "Windows energy ETW/estimation is a later stub; measured_j stays None (no invention)",
        );
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        MeterCapability::unavailable("unsupported", "no OS energy meter backend for this target")
    }
}

#[cfg(feature = "energy-meter")]
fn os_measure(window: Duration, window_ms: u64) -> MeterSample {
    #[cfg(target_os = "linux")]
    {
        return linux::measure(window, window_ms);
    }
    #[cfg(target_os = "macos")]
    {
        return apple::measure(window, window_ms);
    }
    #[cfg(target_os = "windows")]
    {
        let _ = window;
        return MeterSample::unavailable(
            window_ms,
            "Windows energy meter not wired; measured_j=None",
        );
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        let _ = window;
        MeterSample::unavailable(window_ms, "unsupported platform for OS energy meter")
    }
}

#[cfg(feature = "energy-meter")]
fn os_measure_during<F, T>(window: Duration, window_ms: u64, work: F) -> (T, MeterSample)
where
    F: FnOnce() -> T,
{
    #[cfg(target_os = "linux")]
    {
        return linux::measure_during(window, window_ms, work);
    }
    #[cfg(target_os = "macos")]
    {
        return apple::measure_during(window, window_ms, work);
    }
    #[cfg(target_os = "windows")]
    {
        let _ = window;
        let out = work();
        return (
            out,
            MeterSample::unavailable(
                window_ms,
                "Windows energy meter not wired; measured_j=None (work ran unmetered)",
            ),
        );
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        let _ = window;
        let out = work();
        (
            out,
            MeterSample::unavailable(
                window_ms,
                "unsupported platform for OS energy meter (work ran unmetered)",
            ),
        )
    }
}

#[cfg(not(feature = "energy-meter"))]
fn os_probe() -> MeterCapability {
    MeterCapability::unavailable("disabled", "energy-meter feature off")
}

#[cfg(not(feature = "energy-meter"))]
fn os_measure(_window: Duration, window_ms: u64) -> MeterSample {
    MeterSample::unavailable(window_ms, "energy-meter feature off")
}

#[cfg(not(feature = "energy-meter"))]
fn os_measure_during<F, T>(_window: Duration, window_ms: u64, work: F) -> (T, MeterSample)
where
    F: FnOnce() -> T,
{
    let out = work();
    (
        out,
        MeterSample::unavailable(window_ms, "energy-meter feature off — work ran unmetered"),
    )
}

#[cfg(all(feature = "energy-meter", target_os = "linux"))]
mod linux {
    use super::*;
    use std::fs;
    use std::path::Path;
    use std::thread;

    fn read_u128(path: &Path) -> Option<u128> {
        let raw = fs::read_to_string(path).ok()?;
        raw.trim().parse().ok()
    }

    fn collect(root: &Path, out: &mut Vec<RaplCounter>, depth: u32) {
        if depth > 4 {
            return;
        }
        let energy = root.join("energy_uj");
        let name_path = root.join("name");
        if energy.is_file() && name_path.is_file() {
            if let Some(name) = fs::read_to_string(&name_path).ok().map(|s| s.trim().to_string()) {
                if let Some(energy_uj) = read_u128(&energy) {
                    if rapl_component(&name).is_some() {
                        out.push(RaplCounter {
                            name,
                            energy_uj,
                            max_energy_uj: read_u128(&root.join("max_energy_range_uj")),
                        });
                    }
                }
            }
        }
        let Ok(rd) = fs::read_dir(root) else {
            return;
        };
        for ent in rd.flatten() {
            let p = ent.path();
            if !p.is_dir() {
                continue;
            }
            let n = ent.file_name().to_string_lossy().to_string();
            // Skip device/subsystem symlinks that walk the rest of sysfs.
            if n == "device" || n == "subsystem" || n == "power" {
                continue;
            }
            if depth == 0 || n.contains("rapl") || n.contains(':') {
                collect(&p, out, depth + 1);
            }
        }
    }

    fn snapshot() -> Result<Vec<RaplCounter>, String> {
        let root = Path::new("/sys/class/powercap");
        if !root.is_dir() {
            return Err(
                "no /sys/class/powercap (VM or no RAPL exposure) — measured_j=None".into(),
            );
        }
        let mut out = Vec::new();
        collect(root, &mut out, 0);
        if out.is_empty() {
            return Err(
                "powercap present but no package/core/dram/gpu energy_uj — measured_j=None".into(),
            );
        }
        Ok(out)
    }

    pub(super) fn probe() -> MeterCapability {
        match snapshot() {
            Ok(zones) => {
                let names: Vec<&str> = zones.iter().map(|z| z.name.as_str()).collect();
                MeterCapability {
                    available: true,
                    source: MeasureSource::Rapl,
                    detail: format!(
                        "linux powercap RAPL domains [{}] (joules require a delta sample)",
                        names.join(", ")
                    ),
                    platform: "linux_rapl",
                }
            }
            Err(e) => MeterCapability::unavailable("linux_rapl", e),
        }
    }

    pub(super) fn measure(window: Duration, window_ms: u64) -> MeterSample {
        let before = match snapshot() {
            Ok(v) => v,
            Err(e) => return MeterSample::unavailable(window_ms, e),
        };
        thread::sleep(window);
        let after = match snapshot() {
            Ok(v) => v,
            Err(e) => return MeterSample::unavailable(window_ms, e),
        };
        sample_from_rapl_counters(&before, &after, window_ms)
    }

    /// RAPL delta over a window that overlaps `work` (sleep remaining after work).
    pub(super) fn measure_during<F, T>(
        min_window: Duration,
        window_ms: u64,
        work: F,
    ) -> (T, MeterSample)
    where
        F: FnOnce() -> T,
    {
        use std::time::Instant;
        let before = match snapshot() {
            Ok(v) => v,
            Err(e) => {
                let out = work();
                return (out, MeterSample::unavailable(window_ms, e));
            }
        };
        let start = Instant::now();
        let out = work();
        let elapsed = start.elapsed();
        if elapsed < min_window {
            thread::sleep(min_window - elapsed);
        }
        let after = match snapshot() {
            Ok(v) => v,
            Err(e) => return (out, MeterSample::unavailable(window_ms, e)),
        };
        // Report actual elapsed wall time when work outlasted min_window.
        let actual_ms = start.elapsed().as_millis() as u64;
        let report_ms = actual_ms.max(window_ms);
        (out, sample_from_rapl_counters(&before, &after, report_ms))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feature_off_or_probe_never_invents() {
        let cap = probe_meter_capability();
        if !cap.available {
            assert_eq!(cap.source, MeasureSource::Unavailable);
        }
        if !ENERGY_METER_ENABLED {
            assert!(!cap.available);
            assert_eq!(cap.platform, "disabled");
        }
    }

    #[test]
    fn sample_honesty_invariant() {
        let s = measure_energy_window(Duration::from_millis(5));
        assert!(s.honesty_ok(), "detail={}", s.detail);
        if s.measured_j.is_none() {
            if s.components.is_empty() {
                assert!(!s.source.is_measured());
            }
        } else {
            assert!(s.source.is_measured());
            assert!(ENERGY_METER_ENABLED, "Some joules require energy-meter feature");
        }
    }

    #[test]
    fn unavailable_sample_ok() {
        let s = MeterSample::unavailable(1, "test");
        assert!(s.honesty_ok());
        assert!(s.measured_j.is_none());
        assert!(s.components.is_empty());
    }

    #[test]
    fn measure_during_runs_work_and_stays_honest() {
        let (v, s) = measure_energy_during(Duration::from_millis(5), || 42u32);
        assert_eq!(v, 42);
        assert!(s.honesty_ok(), "detail={}", s.detail);
        if s.measured_j.is_none() {
            if s.components.is_empty() {
                assert!(!s.source.is_measured());
            }
        } else {
            assert!(s.source.is_measured());
            assert!(ENERGY_METER_ENABLED);
        }
    }

    #[test]
    fn powermetrics_plist_components_not_summed() {
        let xml = r#"
        <plist><dict>
          <key>elapsed_ns</key><integer>1000000000</integer>
          <key>processor</key><dict>
            <key>ane_energy</key><integer>0</integer>
            <key>cpu_energy</key><integer>89</integer>
            <key>gpu_energy</key><integer>31</integer>
            <key>dram_energy</key><integer>12</integer>
            <key>combined_power</key><real>59.4301</real>
          </dict>
        </dict></plist>"#;
        let s = parse_powermetrics_output(xml, 1000);
        assert!(s.honesty_ok(), "{}", s.detail);
        assert_eq!(s.source, MeasureSource::Powermetrics);
        assert!((s.component(MeterComponent::Cpu).unwrap().0 - 0.089).abs() < 1e-12);
        assert!((s.component(MeterComponent::Gpu).unwrap().0 - 0.031).abs() < 1e-12);
        assert_eq!(s.component(MeterComponent::Ane).unwrap().0, 0.0);
        assert!((s.component(MeterComponent::Dram).unwrap().0 - 0.012).abs() < 1e-12);
        let pkg = s.component(MeterComponent::Package).unwrap().0;
        assert!((pkg - 0.0594301).abs() < 1e-9);
        assert_eq!(s.measured_j.unwrap().0, pkg);
        // Package is combined_power × time, not cpu+gpu+ane+dram.
        let rail_sum = 0.089 + 0.031 + 0.0 + 0.012;
        assert!((pkg - rail_sum).abs() > 1e-4);
    }

    #[test]
    fn powermetrics_text_uses_elapsed_and_keeps_zero_ane() {
        let text = "\
*** Sampled system activity (500.00ms elapsed) ***
CPU Power: 450 mW
GPU Power: 20 mW
ANE Power: 0 mW
DRAM Power: 100 mW
Combined Power (CPU + GPU + ANE): 470 mW
";
        let s = parse_powermetrics_output(text, 1);
        assert!(s.honesty_ok(), "{}", s.detail);
        assert!((s.component(MeterComponent::Cpu).unwrap().0 - 0.225).abs() < 1e-12);
        assert!((s.component(MeterComponent::Gpu).unwrap().0 - 0.010).abs() < 1e-12);
        assert_eq!(s.component(MeterComponent::Ane).unwrap().0, 0.0);
        assert!((s.component(MeterComponent::Dram).unwrap().0 - 0.050).abs() < 1e-12);
        assert!((s.measured_j.unwrap().0 - 0.235).abs() < 1e-12);
    }

    #[test]
    fn powermetrics_permission_invents_nothing() {
        let s = parse_powermetrics_output(
            "powermetrics must be invoked as the superuser\n",
            1000,
        );
        assert!(s.honesty_ok());
        assert!(s.measured_j.is_none());
        assert!(s.components.is_empty());
        assert_eq!(s.source, MeasureSource::Unavailable);
    }

    #[test]
    fn powermetrics_without_package_leaves_measured_j_none() {
        let text = "(200.00ms elapsed)\nCPU Energy: 10 mJ\n";
        let s = parse_powermetrics_output(text, 200);
        assert!(s.honesty_ok(), "{}", s.detail);
        assert!(s.measured_j.is_none());
        assert!((s.component(MeterComponent::Cpu).unwrap().0 - 0.010).abs() < 1e-12);
        assert!(s.component(MeterComponent::Gpu).is_none());
        assert!(s.component(MeterComponent::Ane).is_none());
        assert!(s.component(MeterComponent::Dram).is_none());
    }

    #[test]
    fn rapl_fixture_maps_domains_and_omits_psys() {
        let before = vec![
            RaplCounter { name: "package-0".into(), energy_uj: 1_000_000, max_energy_uj: Some(10_000_000) },
            RaplCounter { name: "core".into(), energy_uj: 0, max_energy_uj: None },
            RaplCounter { name: "dram".into(), energy_uj: 10, max_energy_uj: None },
            RaplCounter { name: "psys".into(), energy_uj: 5, max_energy_uj: None },
        ];
        let after = vec![
            RaplCounter { name: "package-0".into(), energy_uj: 2_500_000, max_energy_uj: Some(10_000_000) },
            RaplCounter { name: "core".into(), energy_uj: 250_000, max_energy_uj: None },
            RaplCounter { name: "dram".into(), energy_uj: 10, max_energy_uj: None },
            RaplCounter { name: "psys".into(), energy_uj: 9_000_000, max_energy_uj: None },
        ];
        let s = sample_from_rapl_counters(&before, &after, 20);
        assert!(s.honesty_ok(), "{}", s.detail);
        assert_eq!(s.source, MeasureSource::Rapl);
        assert!((s.measured_j.unwrap().0 - 1.5).abs() < 1e-9);
        assert!((s.component(MeterComponent::Cpu).unwrap().0 - 0.25).abs() < 1e-9);
        assert_eq!(s.component(MeterComponent::Dram).unwrap().0, 0.0);
        assert!(s.component(MeterComponent::Gpu).is_none());
        assert!(s.component(MeterComponent::Ane).is_none());
        // psys must not inflate package (1.5 J, not ~9 J).
        assert!(s.measured_j.unwrap().0 < 2.0);
    }

    #[test]
    fn rapl_wrap_without_max_does_not_invent() {
        let before = vec![RaplCounter { name: "package-0".into(), energy_uj: 50, max_energy_uj: None }];
        let after = vec![RaplCounter { name: "package-0".into(), energy_uj: 10, max_energy_uj: None }];
        let s = sample_from_rapl_counters(&before, &after, 10);
        assert!(s.measured_j.is_none());
        assert!(s.components.is_empty());
        assert_eq!(s.source, MeasureSource::Unavailable);
    }

    #[test]
    fn ioreport_name_fold_skips_unknown_unit_rows_and_does_not_sum_package() {
        let rows = vec![
            NamedJoule { name: "CPU Energy".into(), joules: 0.02 },
            NamedJoule { name: "EACC_CPU".into(), joules: 99.0 },
            NamedJoule { name: "GPU Energy".into(), joules: 0.01 },
            NamedJoule { name: "GPU SRAM0".into(), joules: 5.0 },
            NamedJoule { name: "ANE0".into(), joules: 0.0 },
            NamedJoule { name: "DRAM0".into(), joules: 0.004 },
        ];
        let s = fold_named_joules(&rows, MeasureSource::IoReport, 200, "ioreport");
        assert!(s.honesty_ok(), "{}", s.detail);
        assert!(s.measured_j.is_none(), "no package channel → measured_j None");
        assert!((s.component(MeterComponent::Cpu).unwrap().0 - 0.02).abs() < 1e-12);
        assert!((s.component(MeterComponent::Gpu).unwrap().0 - 0.01).abs() < 1e-12);
        assert_eq!(s.component(MeterComponent::Ane).unwrap().0, 0.0);
        assert!((s.component(MeterComponent::Dram).unwrap().0 - 0.004).abs() < 1e-12);
    }

    #[test]
    fn smc_pstr_fixture_sets_package_not_rail_sum() {
        let s = sample_from_smc_pstr_watts(10.0, 1000);
        assert!(s.honesty_ok(), "{}", s.detail);
        assert_eq!(s.source, MeasureSource::Smc);
        assert!((s.measured_j.unwrap().0 - 10.0).abs() < 1e-12);
        assert!(s.component(MeterComponent::Cpu).is_none());
    }

    #[test]
    fn merge_smc_package_with_ioreport_rails_keeps_sources() {
        let rails = fold_named_joules(
            &[
                NamedJoule { name: "CPU Energy".into(), joules: 0.02 },
                NamedJoule { name: "GPU Energy".into(), joules: 0.01 },
            ],
            MeasureSource::IoReport,
            200,
            "ioreport",
        );
        let pkg = sample_from_smc_pstr_watts(12.0, 200);
        let s = merge_rails_with_package(&rails, &pkg);
        assert!(s.honesty_ok(), "{}", s.detail);
        assert_eq!(s.source, MeasureSource::Smc);
        assert!((s.measured_j.unwrap().0 - 2.4).abs() < 1e-12);
        assert_eq!(
            s.components
                .iter()
                .find(|c| c.component == MeterComponent::Cpu)
                .unwrap()
                .measure_source,
            MeasureSource::IoReport
        );
        assert_eq!(
            s.components
                .iter()
                .find(|c| c.component == MeterComponent::Package)
                .unwrap()
                .measure_source,
            MeasureSource::Smc
        );
    }
}
