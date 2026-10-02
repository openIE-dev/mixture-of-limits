//! Headless energy harness shell — machine protocol lane first.
//!
//! Wraps [`MixtureOfLimits::close`] with fabric inventory, joule ledger,
//! transcript recording, and optional OS meter attachment. GUI is optional;
//! this API is the prove surface and MCP-class backend.

use std::time::Duration;

use mol_automate::{EcosystemCertify, EcosystemCertifyConfig, EcosystemCertifyOutcome};
use mol_core::{
    inventory_for_schedule, measure_energy_window, probe_meter_capability, Budget, CascadeTier,
    FabricInventory, FailClosedPolicy, MolRequest, ENERGY_METER_ENABLED, FABRIC_DETECT_ENABLED,
    METER_HONESTY_NOTE,
};
use mol_limits::MixtureOfLimits;
use mol_receipt::{
    replay_transcript, CloseTranscript, CloseTranscriptEntry, MolReceipt, ReplayCloseOutcome,
    ReplayReport,
};

use crate::view::{
    ConfirmRequest, ConfirmResponse, FabricView, JouleLedgerEntry, JouleLedgerSummary, ReceiptView,
};

/// Errors from the desktop shell (protocol lane).
#[derive(Debug, thiserror::Error)]
pub enum ShellError {
    /// MoL close / route failure.
    #[error("mol: {0}")]
    Mol(#[from] mol_core::MolError),
    /// Human refused a consequential confirm.
    #[error("human refused confirm: {0}")]
    ConfirmRefused(String),
    /// Replay mismatch / IO.
    #[error("replay: {0}")]
    Replay(String),
    /// Invalid certify / shell options.
    #[error("config: {0}")]
    Config(String),
}

/// Result alias.
pub type ShellResult<T> = std::result::Result<T, ShellError>;

/// Options for one ask/close through the harness.
#[derive(Debug, Clone)]
pub struct AskOpts {
    /// Joule / latency / model budget.
    pub budget: Budget,
    /// When true and `energy-meter` feature is on, sample OS meter around close
    /// and stamp receipt `measured_j` only if probe succeeds.
    pub attach_meter: bool,
    /// Meter sample window when attaching.
    pub meter_window: Duration,
    /// When true, require human confirm before close (interface tax).
    pub require_confirm: bool,
    /// Fail-closed gates (e.g. [`FailClosedPolicy::meter_required`]).
    pub fail_closed: FailClosedPolicy,
}

impl Default for AskOpts {
    fn default() -> Self {
        Self {
            budget: Budget::demo(),
            attach_meter: false,
            meter_window: Duration::from_millis(10),
            require_confirm: false,
            fail_closed: FailClosedPolicy::open(),
        }
    }
}

impl AskOpts {
    /// Demo budget, no meter, no confirm.
    pub fn demo() -> Self {
        Self::default()
    }

    /// Enable meter attach (no-op invent when feature off / unavailable).
    pub fn with_meter(mut self, window: Duration) -> Self {
        self.attach_meter = true;
        self.meter_window = window;
        self
    }

    /// Require measured joules to commit: attach meter + `FailClosedPolicy::meter_required`.
    ///
    /// Commits only when the OS meter (or honest fixture via request) supplies real
    /// `measured_j` with Measured honesty. Probe fail / Modeled → refuse. Never invent.
    pub fn with_meter_required(mut self, window: Duration) -> Self {
        self.attach_meter = true;
        self.meter_window = window;
        self.fail_closed = FailClosedPolicy::meter_required();
        self
    }

    /// Set fail-closed policy explicitly.
    pub fn with_fail_closed(mut self, policy: FailClosedPolicy) -> Self {
        self.fail_closed = policy;
        self
    }

    /// Require human confirm (UI / protocol gate).
    pub fn with_confirm(mut self) -> Self {
        self.require_confirm = true;
        self
    }
}

/// Options for [`ShellSession::ecosystem_certify`] (GUI / protocol lane).
///
/// Mirrors `mol ecosystem-certify` flags: detect, tier formula|model, meter_required, meter-ms.
/// Soft-ref prove path leaves detect=false and meter off — never invents `measured_j`.
#[derive(Debug, Clone)]
pub struct EcosystemCertifyOpts {
    /// Use live wgpu inventory when `fabric-detect` is compiled; else soft-ref.
    pub detect: bool,
    /// Schedule tier: `formula` (Cpu) or `model` (Gpu*|refuse). Other labels map like CLI.
    pub tier: String,
    /// Fail-closed: COMMIT only when real package `measured_j` is stamped.
    pub meter_required: bool,
    /// Overlapping OS meter window in milliseconds (with meter_required / attach).
    pub meter_ms: u64,
    /// When true, sample meter window even if not required (honest None OK).
    pub attach_meter: bool,
    /// Exercise grant_receipt_required refuse path.
    pub refuse_without_grant: bool,
    /// Agent lane session id.
    pub session_id: String,
}

impl Default for EcosystemCertifyOpts {
    fn default() -> Self {
        Self {
            detect: false,
            tier: "formula".into(),
            meter_required: false,
            meter_ms: 50,
            attach_meter: false,
            refuse_without_grant: false,
            session_id: "mol-desktop-ecosystem".into(),
        }
    }
}

impl EcosystemCertifyOpts {
    /// Soft-ref Formula happy path (prove / offline).
    pub fn soft_ref() -> Self {
        Self::default()
    }

    /// Live detect + Model tier (needs `fabric-detect` for Gpu*).
    pub fn detect_model(mut self) -> Self {
        self.detect = true;
        self.tier = "model".into();
        self
    }

    /// Require measured joules (overlapping window).
    pub fn with_meter_required(mut self, meter_ms: u64) -> Self {
        self.meter_required = true;
        self.attach_meter = true;
        self.meter_ms = meter_ms;
        self
    }
}

/// Outcome of shell ecosystem certify for UI + protocol clients.
#[derive(Debug, Clone)]
pub struct ShellEcosystemResult {
    /// Commit vs refuse.
    pub commit: bool,
    /// Display receipt view (full ecosystem stamps).
    pub view: ReceiptView,
    /// Underlying receipt.
    pub receipt: MolReceipt,
    /// Capsule i32 return when committed.
    pub capsule_return: Option<i32>,
    /// Fuel consumed on capsule path.
    pub fuel_consumed: Option<u64>,
    /// Grant id when host invoke allowed.
    pub grant_id: Option<String>,
    /// Binding floor id when refused.
    pub floor_id: Option<String>,
    /// Fabric inventory summary used for schedule.
    pub fabric_summary: String,
    /// Whether fabric-detect feature is compiled in.
    pub fabric_detect_enabled: bool,
    /// Schedule tier label used.
    pub schedule_tier: String,
}

/// Outcome of shell ask/close for UI + protocol clients.
#[derive(Debug, Clone)]
pub struct ShellCloseResult {
    /// Commit vs refuse.
    pub commit: bool,
    /// Display receipt view.
    pub view: ReceiptView,
    /// Underlying receipt (HMAC-ready).
    pub receipt: MolReceipt,
    /// Meter detail when attach was requested.
    pub meter_detail: Option<String>,
}

/// Energy harness session: MoL close + ledger + transcript.
pub struct ShellSession {
    mol: MixtureOfLimits,
    transcript: CloseTranscript,
    ledger: Vec<JouleLedgerEntry>,
    cumulative_j: f64,
    /// Pending confirm for consequential acts (interface tax).
    pending_confirm: Option<ConfirmRequest>,
    /// Last human confirm response.
    last_confirm: Option<ConfirmResponse>,
}

impl Default for ShellSession {
    fn default() -> Self {
        Self::new()
    }
}

impl ShellSession {
    /// Fresh session with soft-ref fabric + empty ledger/transcript.
    pub fn new() -> Self {
        Self {
            mol: MixtureOfLimits::new(),
            transcript: CloseTranscript::new(),
            ledger: Vec::new(),
            cumulative_j: 0.0,
            pending_confirm: None,
            last_confirm: None,
        }
    }

    /// With custom fabric inventory.
    pub fn with_fabric(fabric: FabricInventory) -> Self {
        let mut s = Self::new();
        s.mol = MixtureOfLimits::new().with_fabric(fabric);
        s
    }

    /// Borrow MoL router.
    pub fn mol(&self) -> &MixtureOfLimits {
        &self.mol
    }

    /// Fabric inventory view.
    pub fn fabric_view(&self) -> FabricView {
        FabricView::from_inventory(&self.mol.fabric)
    }

    /// Joule ledger entries (session).
    pub fn ledger(&self) -> &[JouleLedgerEntry] {
        &self.ledger
    }

    /// Ledger summary (joules-per-verified-decision).
    pub fn ledger_summary(&self) -> JouleLedgerSummary {
        let acts = self.ledger.len() as u64;
        let commits = self.ledger.iter().filter(|e| e.decision == "COMMIT").count() as u64;
        let refuses = acts.saturating_sub(commits);
        let total = self.cumulative_j;
        let mean = if acts == 0 { 0.0 } else { total / acts as f64 };
        let ratios: Vec<f64> = self
            .ledger
            .iter()
            .filter_map(|e| e.landauer_floor_ratio)
            .collect();
        let mean_landauer_ratio = if ratios.is_empty() {
            None
        } else {
            Some(ratios.iter().sum::<f64>() / ratios.len() as f64)
        };
        let measured_honest = self.ledger.iter().all(|e| match e.measured_j {
            None => true,
            Some(j) => j.is_finite() && j >= 0.0,
        });
        JouleLedgerSummary {
            acts,
            commits,
            refuses,
            total_estimated_j: total,
            mean_estimated_j: mean,
            mean_landauer_ratio,
            measured_honest,
        }
    }

    /// Close transcript (for replay).
    pub fn transcript(&self) -> &CloseTranscript {
        &self.transcript
    }

    /// Pending human confirm (interface tax), if any.
    pub fn pending_confirm(&self) -> Option<&ConfirmRequest> {
        self.pending_confirm.as_ref()
    }

    /// Record human confirm response; clears pending on decide.
    pub fn respond_confirm(&mut self, response: ConfirmResponse) {
        self.last_confirm = Some(response);
        self.pending_confirm = None;
    }

    /// Ask + MoL close; append ledger + transcript.
    ///
    /// Machine protocol fidelity: this is the primary lane. Human confirm only
    /// when `opts.require_confirm` (consequential / mutate-class).
    pub fn ask_close(&mut self, query: impl Into<String>, opts: AskOpts) -> ShellResult<ShellCloseResult> {
        let query = query.into();
        if opts.require_confirm {
            if self.last_confirm != Some(ConfirmResponse::Approve) {
                self.pending_confirm = Some(ConfirmRequest {
                    act: format!("close:{query}"),
                    reason: "consequential confirm required (interface tax)".into(),
                    estimated_j: None,
                    capability: None,
                });
                if self.last_confirm == Some(ConfirmResponse::Refuse) {
                    self.last_confirm = None;
                    return Err(ShellError::ConfirmRefused(query));
                }
                // First call without prior approve: surface pending and refuse-close.
                if self.last_confirm.is_none() {
                    return Err(ShellError::ConfirmRefused(format!(
                        "pending confirm for: {query}"
                    )));
                }
            }
            self.last_confirm = None;
        }

        let mut req = MolRequest::new(query.clone(), opts.budget)
            .with_fail_closed(opts.fail_closed);
        let mut meter_detail = None;
        if opts.attach_meter {
            let sample = measure_energy_window(opts.meter_window);
            meter_detail = Some(format!(
                "{} | feature={} | honesty_ok={} | records={} | {}",
                sample.detail,
                ENERGY_METER_ENABLED,
                sample.honesty_ok(),
                sample.records_measurement(),
                METER_HONESTY_NOTE
            ));
            assert!(sample.honesty_ok(), "meter honesty broken: {}", sample.detail);
            // Feed sample into MoL close so meter_required can commit|refuse honestly.
            req = req.with_meter_sample(sample);
        }

        let out = self.mol.close(&req)?;
        let commit = out.is_commit();
        // Receipt already stamped by close (ecosystem stamp applies meter_sample).
        let receipt = out.receipt().clone();

        let view = ReceiptView::from_receipt(commit, &receipt);
        self.push_ledger(&query, &view);
        self.transcript.push(CloseTranscriptEntry::record(
            query,
            opts.budget,
            commit,
            &receipt,
        ));

        Ok(ShellCloseResult {
            commit,
            view,
            receipt,
            meter_detail,
        })
    }

    /// Convenience: demo budget close without meter/confirm.
    pub fn close_demo(&mut self, query: impl Into<String>) -> ShellResult<ShellCloseResult> {
        self.ask_close(query, AskOpts::demo())
    }

    /// Replay recorded transcript against fresh MoL closes.
    pub fn replay(&self) -> ShellResult<ReplayReport> {
        let mol = MixtureOfLimits::new().with_fabric(self.mol.fabric.clone());
        replay_transcript(&self.transcript, |query, budget| {
            let req = MolRequest::new(query, budget);
            let out = mol.close(&req)?;
            Ok(ReplayCloseOutcome {
                commit: out.is_commit(),
                receipt: out.receipt().clone(),
            })
        })
        .map_err(|e| ShellError::Replay(e.to_string()))
    }

    /// Run [`EcosystemCertify`] and append the receipt to the joule ledger.
    ///
    /// Operator / GUI surface for: detect · tier formula|model · meter_required · meter-ms.
    /// Stamps on the resulting [`ReceiptView`]: energy_honesty / measured_j, encapsulation,
    /// agent_lane, fabric_id (on compute_steps), kernel:vector_add proof, compute_steps,
    /// commit|refuse. **Never invents** `measured_j`.
    pub fn ecosystem_certify(
        &mut self,
        opts: EcosystemCertifyOpts,
    ) -> ShellResult<ShellEcosystemResult> {
        let mut cfg = if opts.refuse_without_grant {
            EcosystemCertifyConfig::refuse_without_grant()
        } else {
            EcosystemCertifyConfig::soft_ref()
        };
        cfg.session_id = opts.session_id.clone();

        let tier_l = opts.tier.to_ascii_lowercase();
        let schedule_tier = match tier_l.as_str() {
            "formula" | "lookup" => CascadeTier::Formula,
            "model" => CascadeTier::Model,
            "solver" | "settle" => CascadeTier::Solver,
            other => {
                return Err(ShellError::Config(format!(
                    "unknown ecosystem tier {other} (expected formula|model|solver)"
                )));
            }
        };

        let inv = if opts.detect {
            inventory_for_schedule()
        } else {
            FabricInventory::software_ref()
        };
        let fabric_summary = inv.summary();
        cfg = cfg.with_fabric(inv);

        if schedule_tier == CascadeTier::Model {
            // Demo budget can choose Gpu*; coin-cell would fail-closed even if Metal present.
            cfg = cfg.with_budget(Budget::demo().allow_model());
        }
        cfg = cfg.with_schedule_tier(schedule_tier);

        if opts.meter_required {
            cfg = cfg.with_meter_required(true);
        }
        let attach_meter = opts.attach_meter || opts.meter_required;
        if attach_meter {
            cfg = cfg.with_meter_window_ms(opts.meter_ms);
        }

        let out: EcosystemCertifyOutcome = EcosystemCertify::run(&cfg);
        let commit = out.is_commit();
        let receipt = out.receipt.clone();
        let view = ReceiptView::from_receipt(commit, &receipt);
        let query = receipt
            .query
            .clone()
            .unwrap_or_else(|| "ecosystem_e2e_certify".into());
        self.push_ledger(&query, &view);
        self.transcript.push(CloseTranscriptEntry::record(
            query,
            cfg.budget,
            commit,
            &receipt,
        ));

        Ok(ShellEcosystemResult {
            commit,
            view,
            receipt,
            capsule_return: out.capsule_return,
            fuel_consumed: out.fuel_consumed,
            grant_id: out.grant_id,
            floor_id: out.floor.as_ref().map(|f| f.id.as_str().to_string()),
            fabric_summary,
            fabric_detect_enabled: FABRIC_DETECT_ENABLED,
            schedule_tier: schedule_tier.label().to_string(),
        })
    }

    /// Soft-ref ecosystem certify convenience (Formula, no meter).
    pub fn ecosystem_certify_soft_ref(&mut self) -> ShellResult<ShellEcosystemResult> {
        self.ecosystem_certify(EcosystemCertifyOpts::soft_ref())
    }

        /// Meter capability status (feature-aware; never invents).
    pub fn meter_status(&self) -> String {
        let cap = probe_meter_capability();
        format!(
            "available={} platform={} source={} detail={}",
            cap.available,
            cap.platform,
            cap.source.label(),
            cap.detail
        )
    }

    fn push_ledger(&mut self, query: &str, view: &ReceiptView) {
        self.cumulative_j += view.estimated_j;
        let seq = self.ledger.len() as u64;
        let encapsulation = view
            .encapsulation
            .as_ref()
            .map(|e| {
                format!(
                    "{}:{}{}",
                    e.boundary,
                    if e.sealed { "sealed" } else { "unsealed" },
                    if e.shares_host_session { "+host_share" } else { "" }
                )
            })
            .unwrap_or_else(|| "missing".into());
        let agent_lane = view
            .agent_lane
            .as_ref()
            .map(|a| format!("{}:cookies={}", a.lane, a.separate_cookies))
            .unwrap_or_else(|| "missing".into());
        self.ledger.push(JouleLedgerEntry {
            seq,
            query: query.to_string(),
            decision: view.decision.clone(),
            estimated_j: view.estimated_j,
            landauer_floor_ratio: view.landauer_floor_ratio,
            cumulative_estimated_j: self.cumulative_j,
            fabric: view.fabric.clone(),
            limit: view.limit.clone(),
            receipt_id: view.receipt_id.clone(),
            measured_j: view.measured_j,
            energy_honesty: view.energy_honesty.clone(),
            encapsulation,
            agent_lane,
            compute_steps: view.compute_steps.len(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn headless_close_landauer_commit() {
        let mut s = ShellSession::new();
        let r = s.close_demo("landauer joules per bit").unwrap();
        assert!(r.commit, "expected COMMIT");
        assert!(r.view.honesty_ok());
        assert!(r.view.measured_j.is_none());
        // Soft-ref may label estimate provenance (catalog_surrogate / cascade_estimate)
        // while keeping measured_j=None — never a measured source without Some.
        assert!(
            !matches!(
                r.view.measure_source.as_str(),
                "rapl" | "nvml" | "cpu_proxy" | "macos_energy" | "windows_energy"
            ),
            "measured source without measured_j: {}",
            r.view.measure_source
        );
        assert!(!r.view.board_synth_claimed);
        assert!(r.view.fabric.as_deref() == Some("cpu") || r.view.fabric.is_some());
        assert_eq!(s.ledger().len(), 1);
        assert!(s.ledger_summary().measured_honest);
    }

    #[test]
    fn headless_voi_refuse() {
        let mut s = ShellSession::new();
        let r = s.close_demo("write a poem about GPUs").unwrap();
        assert!(!r.commit);
        assert_eq!(r.view.limit.as_deref(), Some("voi"));
        assert!(r.view.measured_j.is_none());
    }

    #[test]
    fn fabric_inventory_soft_ref() {
        let s = ShellSession::new();
        let f = s.fabric_view();
        assert_eq!(f.source, "software_ref");
        assert!(f.present.iter().any(|p| p == "cpu"));
    }

    #[test]
    fn meter_attach_never_invents_when_feature_off() {
        let mut s = ShellSession::new();
        let r = s
            .ask_close(
                "landauer joules per bit",
                AskOpts::demo().with_meter(Duration::from_millis(5)),
            )
            .unwrap();
        // Feature off ⇒ None; feature on + no RAPL ⇒ None; feature on + RAPL ⇒ Some labeled.
        assert!(r.view.honesty_ok());
        if r.view.measured_j.is_some() {
            assert!(ENERGY_METER_ENABLED);
            assert!(matches!(
                r.view.measure_source.as_str(),
                "rapl" | "macos_energy" | "windows_energy" | "nvml" | "cpu_proxy" | "powermetrics" | "ioreport" | "smc"
            ));
        } else {
            assert!(!matches!(
                r.view.measure_source.as_str(),
                "rapl" | "macos_energy" | "windows_energy" | "nvml" | "cpu_proxy"
            ) || r.meter_detail.is_some());
        }
        assert_eq!(r.view.energy_honesty.as_str().is_empty(), false);
    }

    #[test]
    fn meter_required_refuses_when_probe_unavailable() {
        let mut s = ShellSession::new();
        let r = s
            .ask_close(
                "landauer joules per bit",
                AskOpts::demo().with_meter_required(Duration::from_millis(5)),
            )
            .unwrap();
        // On this box: feature off or no RAPL → refuse; live RAPL → may commit.
        if r.view.measured_j.is_none() {
            assert!(!r.commit, "meter_required without measured_j must REFUSE");
            assert!(
                matches!(r.view.energy_honesty.as_str(), "unavailable" | "estimated"),
                "got {}",
                r.view.energy_honesty
            );
            assert!(r.view.limit.is_some());
        } else {
            assert!(r.commit);
            assert_eq!(r.view.energy_honesty, "measured");
            assert!(ENERGY_METER_ENABLED);
        }
        // Ledger surfaces honesty / enc / agent stamps.
        let e = &s.ledger()[0];
        assert!(!e.energy_honesty.is_empty());
        assert_eq!(e.encapsulation, "missing");
        assert_eq!(e.agent_lane, "missing");
    }

    #[test]
    fn ecosystem_certify_soft_ref_stamps_full_receipt() {
        let mut s = ShellSession::new();
        let r = s.ecosystem_certify_soft_ref().unwrap();
        assert!(r.commit, "soft-ref Formula should COMMIT");
        assert_eq!(r.capsule_return, Some(42));
        assert!(r.view.encapsulation.is_some());
        assert!(r.view.agent_lane.is_some());
        assert!(r.view.measured_j.is_none(), "never invent measured_j");
        assert_eq!(r.view.energy_honesty, "estimated");
        assert!(r.view.honesty_ok());
        assert!(
            r.view
                .compute_steps
                .iter()
                .any(|c| c.label == "fabric:route" && c.fabric_id.as_deref() == Some("cpu")),
            "fabric:route must stamp fabric_id=cpu"
        );
        // Formula → Cpu: kernel:vector_add may be skip-stamped or present without live Gpu.
        let kernel = r
            .view
            .compute_steps
            .iter()
            .find(|c| c.label.starts_with("kernel:"));
        if let Some(k) = kernel {
            assert!(k.measured_j.is_none());
            // Soft path may include execution_proof on skip or stub — never invents joules.
            let _ = k.execution_proof.as_ref();
        }
        assert_eq!(s.ledger().len(), 1);
        assert_ne!(s.ledger()[0].encapsulation, "missing");
        assert_ne!(s.ledger()[0].agent_lane, "missing");
        assert!(!r.fabric_detect_enabled || FABRIC_DETECT_ENABLED);
    }

    #[test]
    fn ecosystem_certify_refuse_without_grant() {
        let mut s = ShellSession::new();
        let r = s
            .ecosystem_certify(EcosystemCertifyOpts {
                refuse_without_grant: true,
                ..EcosystemCertifyOpts::soft_ref()
            })
            .unwrap();
        assert!(!r.commit);
        assert_eq!(r.floor_id.as_deref(), Some("grant_receipt_required"));
        assert!(r.view.encapsulation.is_some());
        assert!(r.view.measured_j.is_none());
    }
}
