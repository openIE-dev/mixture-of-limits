//! End-to-end ecosystem certify — one path, one receipt.
//!
//! Closes Agent Lane + fabric schedule + WASM capsule + energy honesty +
//! GrantReceipt host invoke into a single [`MolReceipt`] with full stamps:
//! encapsulation, agent_lane, compute_steps/fabric, energy_honesty.
//!
//! Soft-ref default never invents `measured_j`. Optional [`MeterSample`] may be
//! attached when a real meter provided it.

use std::time::Duration;

use mol_core::{
    energy_pair_honest, host_invoke, measure_energy_during, measure_energy_window,
    require_measured_or_refuse, run_tiny_vector_add_if_gpu, schedule_fabric, AgentLaneSession,
    Budget, CapsuleInvoke, CapsuleRuntime, CascadeTier, DeviceKind, EnergyHonestyClass,
    EstimateKind, FabricInventory, Floor, FloorKind, GrantReceipt, HostCapability,
    HostInvokeRequest, Joules, LaneProvenance, MeasureSource, MeterSample, MuSource,
    PartitionSurface, ReplayClass, StubCapsuleRuntime, BOARD_SYNTH_CLAIMED,
};
use mol_receipt::{MolReceipt, ReceiptBuilder};

/// Configuration for [`EcosystemCertify::run`].
#[derive(Debug, Clone)]
pub struct EcosystemCertifyConfig {
    /// Agent lane session id.
    pub session_id: String,
    /// WASM add fixture args `(a, b)` → expected `a+b` on commit.
    pub add_args: (i32, i32),
    /// Soft-ref fabric inventory (Cpu always offline).
    pub fabric: FabricInventory,
    /// Cascade tier used for fabric schedule (default Formula → Cpu).
    pub schedule_tier: CascadeTier,
    /// Budget for schedule.
    pub budget: Budget,
    /// Optional real meter sample — never invent; honesty checked.
    /// Prefer attaching a fixture for prove; live CLI uses [`Self::meter_window_ms`].
    pub meter_sample: Option<MeterSample>,
    /// When set, sample the OS meter over a window overlapping the Metal/wgpu kernel
    /// (or an idle window if the kernel is skipped). Live path — never invents.
    pub meter_window_ms: Option<u64>,
    /// Fail-closed: COMMIT only when meter_sample supplies real package measured_j.
    /// Never invent measured_j to satisfy.
    pub meter_required: bool,
    /// When true, omit GrantReceipt to exercise the refuse path.
    pub refuse_without_grant: bool,
    /// Host capability granted / requested.
    pub host_capability: HostCapability,
    /// When true (default), run the tiny wgpu/CPU vector-add after a Gpu* schedule
    /// and stamp `kernel:vector_add` on the receipt. Soft-ref Cpu skips live dispatch.
    pub run_kernel: bool,
}

impl Default for EcosystemCertifyConfig {
    fn default() -> Self {
        Self {
            session_id: "ecosystem-e2e".into(),
            add_args: (2, 40),
            fabric: FabricInventory::software_ref(),
            schedule_tier: CascadeTier::Formula,
            budget: Budget::coin_cell(),
            meter_sample: None,
            meter_window_ms: None,
            meter_required: false,
            refuse_without_grant: false,
            host_capability: HostCapability::ModelContextExecute,
            run_kernel: true,
        }
    }
}

impl EcosystemCertifyConfig {
    /// Soft-ref happy-path defaults (CPU + sealed add.wasm + GrantReceipt).
    pub fn soft_ref() -> Self {
        Self::default()
    }

    /// Refuse path: host invoke without grant (keyword alone insufficient).
    pub fn refuse_without_grant() -> Self {
        Self {
            refuse_without_grant: true,
            ..Self::default()
        }
    }

    /// Attach an optional meter sample (must already be honest — never invent).
    pub fn with_meter_sample(mut self, sample: MeterSample) -> Self {
        self.meter_sample = Some(sample);
        self
    }

    /// Fail-closed: require real package `measured_j` on the stamped receipt.
    pub fn with_meter_required(mut self, required: bool) -> Self {
        self.meter_required = required;
        self
    }

    /// Live overlapping meter window (ms). Sampled around the kernel when Gpu* runs.
    /// Soft-ref prove leaves this `None` (fixture via [`Self::with_meter_sample`] OK).
    pub fn with_meter_window_ms(mut self, ms: u64) -> Self {
        self.meter_window_ms = Some(ms);
        self
    }

    /// Replace fabric inventory (live wgpu detect or a mock). Does not invent joules.
    pub fn with_fabric(mut self, fabric: FabricInventory) -> Self {
        self.fabric = fabric;
        self
    }

    /// Cascade tier used for [`mol_core::schedule_fabric`].
    ///
    /// Formula/Lookup stay Cpu. Model stamps the cheapest present Gpu* or refuses
    /// `fabric_unavailable` (fail-closed).
    pub fn with_schedule_tier(mut self, tier: CascadeTier) -> Self {
        self.schedule_tier = tier;
        self
    }

    /// Budget for the fabric schedule (coin-cell refuses hot Gpu* even if present).
    pub fn with_budget(mut self, budget: Budget) -> Self {
        self.budget = budget;
        self
    }

    /// Enable/disable tiny kernel stamp after Gpu* schedule (default true).
    pub fn with_run_kernel(mut self, run: bool) -> Self {
        self.run_kernel = run;
        self
    }
}

/// Outcome of the ecosystem e2e certify path.
#[derive(Debug, Clone)]
pub struct EcosystemCertifyOutcome {
    /// Commit or refuse.
    pub committed: bool,
    /// Single receipt with full ecosystem stamps.
    pub receipt: MolReceipt,
    /// Binding floor when refused.
    pub floor: Option<Floor>,
    /// Capsule i32 return when capsule committed (add fixture).
    pub capsule_return: Option<i32>,
    /// Fuel consumed on capsule path.
    pub fuel_consumed: Option<u64>,
    /// Grant id stamped when host invoke allowed.
    pub grant_id: Option<String>,
}

impl EcosystemCertifyOutcome {
    /// True on COMMIT.
    pub fn is_commit(&self) -> bool {
        self.committed
    }

    /// True on REFUSE.
    pub fn is_refuse(&self) -> bool {
        !self.committed
    }
}

/// End-to-end ecosystem certify orchestrator.
pub struct EcosystemCertify;

impl EcosystemCertify {
    /// Run the single-receipt ecosystem certify path.
    ///
    /// Steps (fail-closed):
    /// 1. Open Agent Lane (agent partition)
    /// 2. Schedule fabric (CPU offline-honest on soft-ref)
    /// 3. Invoke WASM capsule (`add.wasm`) under bounds
    /// 4. Stamp energy honesty (fuel → estimated; optional meter_sample)
    /// 5. Host invoke only with [`GrantReceipt`]
    /// 6. `commit|refuse` with encapsulation, agent_lane, compute_steps/fabric, energy_honesty
    pub fn run(cfg: &EcosystemCertifyConfig) -> EcosystemCertifyOutcome {
        Self::run_with(cfg, &StubCapsuleRuntime)
    }

    /// Same as [`Self::run`] with an injected capsule runtime.
    pub fn run_with(
        cfg: &EcosystemCertifyConfig,
        runtime: &dyn CapsuleRuntime,
    ) -> EcosystemCertifyOutcome {
        // --- 1. Open Agent Lane ---
        let session = AgentLaneSession::open_agent(&cfg.session_id);
        if let Err(floor) = session.check() {
            return Self::refuse_early(
                cfg,
                floor,
                None,
                vec![],
                "ecosystem e2e REFUSE: agent lane session check failed",
            );
        }
        let mut agent_lane = session.to_receipt();

        // --- 2. Schedule fabric (CPU offline-honest) ---
        let sched = schedule_fabric(
            cfg.schedule_tier,
            &cfg.fabric,
            &cfg.budget,
            Joules::new(1e-12),
        );
        let mut compute_steps = vec![sched.to_compute_step()];
        let fabric_chosen = sched.chosen_kind();
        if sched.is_unavailable() {
            let floor = Floor::new(
                "fabric_unavailable",
                FloorKind::WcaRefuse,
                sched
                    .unavailable_reason()
                    .unwrap_or("no sufficient fabric")
                    .to_string(),
            );
            return Self::refuse_early(
                cfg,
                floor,
                Some(agent_lane),
                compute_steps,
                "ecosystem e2e REFUSE: fabric schedule unavailable",
            );
        }
        if fabric_chosen != Some(DeviceKind::Cpu) && cfg.fabric.is_valid_software_ref() {
            // Soft-ref Formula/Lookup must pick Cpu — anything else is a law bug.
            let floor = Floor::new(
                "fabric_schedule_unexpected",
                FloorKind::WcaRefuse,
                format!("soft-ref expected Cpu, got {:?}", fabric_chosen),
            );
            return Self::refuse_early(
                cfg,
                floor,
                Some(agent_lane),
                compute_steps,
                "ecosystem e2e REFUSE: unexpected fabric for soft-ref schedule",
            );
        }

        // --- 2b. Tiny wgpu/CPU vector-add when Gpu* chosen (soft stub offline) ---
        // Optional overlapping OS meter (SMC on Mac) stamps real package measured_j
        // onto the kernel ComputeStepReceipt — never invent / never sum rails.
        let mut effective_meter: Option<MeterSample> = cfg.meter_sample.clone();
        if cfg.run_kernel {
            if let Some(fk) = fabric_chosen {
                let (kern, overlapped) = if let Some(ms) = cfg.meter_window_ms {
                    let (k, s) = measure_energy_during(Duration::from_millis(ms), || {
                        run_tiny_vector_add_if_gpu(fk)
                    });
                    (k, Some(s))
                } else {
                    (run_tiny_vector_add_if_gpu(fk), None)
                };
                let mut step = kern.to_compute_step();
                // Kernel dispatch itself never invents joules.
                if !step.honesty_ok() || step.measured_j.is_some() {
                    let floor = Floor::new(
                        "kernel_energy_honesty",
                        FloorKind::EnergyHonesty,
                        "kernel stamp invented measured_j before meter attach — refuse",
                    );
                    return Self::refuse_early(
                        cfg,
                        floor,
                        Some(agent_lane),
                        {
                            let mut s = compute_steps.clone();
                            s.push(step);
                            s
                        },
                        "ecosystem e2e REFUSE: kernel energy honesty violated",
                    );
                }
                if let Some(s) = overlapped {
                    if s.honesty_ok() {
                        // Prefer overlapping live sample over any pre-attached fixture.
                        effective_meter = Some(s);
                    }
                }
                if let Some(ref sample) = effective_meter {
                    if sample.honesty_ok() && sample.measured_j.is_some() {
                        step.apply_meter_sample(sample);
                        if !step.honesty_ok() {
                            let floor = Floor::new(
                                "kernel_meter_honesty",
                                FloorKind::EnergyHonesty,
                                "kernel meter stamp dishonest — refuse",
                            );
                            return Self::refuse_early(
                                cfg,
                                floor,
                                Some(agent_lane),
                                {
                                    let mut s = compute_steps.clone();
                                    s.push(step);
                                    s
                                },
                                "ecosystem e2e REFUSE: kernel meter honesty violated",
                            );
                        }
                    }
                }
                // Skip stamps still recorded (honest) when schedule is Cpu.
                compute_steps.push(step);
            }
        }
        // If meter_window requested but kernel skipped / absent, still take an idle sample
        // so meter_required can COMMIT|REFUSE honestly (never invent).
        if effective_meter.is_none() {
            if let Some(ms) = cfg.meter_window_ms {
                let s = measure_energy_window(Duration::from_millis(ms));
                if s.honesty_ok() {
                    effective_meter = Some(s);
                }
            }
        }

        // --- 3. Invoke WASM capsule under bounds ---
        let invoke = CapsuleInvoke::sealed_add_fixture(cfg.add_args.0, cfg.add_args.1);
        let capsule = runtime.certify(&invoke);
        compute_steps.extend(capsule.compute_steps.iter().cloned());
        if !capsule.is_commit() {
            let floor = capsule.floor.clone().unwrap_or_else(|| {
                Floor::new(
                    "capsule_refuse",
                    FloorKind::Encapsulation,
                    "capsule certify refused",
                )
            });
            let builder = Self::base_builder(cfg)
                .encapsulation(capsule.encapsulation.clone())
                .agent_lane(agent_lane)
                .compute_steps(compute_steps)
                .estimated_j(capsule.estimated_j)
                .estimate_kind(capsule.estimate_kind)
                .measure_source(capsule.measure_source)
                .energy_honesty(capsule.energy_honesty)
                .limit_fired(floor.clone())
                .fabric(fabric_chosen, cfg.fabric.clone())
                .executed(false)
                .rationale(format!(
                    "ecosystem e2e REFUSE at capsule: {}",
                    capsule.rationale
                ));
            let mut receipt = builder.build();
            Self::stamp_meter(&mut receipt, effective_meter.as_ref());
            return EcosystemCertifyOutcome {
                committed: false,
                receipt,
                floor: Some(floor),
                capsule_return: None,
                fuel_consumed: capsule.fuel_consumed,
                grant_id: None,
            };
        }

        // --- 4. Energy honesty (fuel → estimated; optional meter) ---
        // Capsule fuel path must keep measured_j=None unless a real meter is attached.
        if !capsule.honesty_ok() || capsule.measured_j.is_some() {
            let floor = Floor::new(
                "energy_honesty_capsule_invent",
                FloorKind::EnergyHonesty,
                "capsule fuel path invented measured_j — refuse",
            );
            let builder = Self::base_builder(cfg)
                .encapsulation(capsule.encapsulation.clone())
                .agent_lane(agent_lane)
                .compute_steps(compute_steps)
                .estimated_j(capsule.estimated_j)
                .estimate_kind(capsule.estimate_kind)
                .measure_source(MeasureSource::CascadeEstimate)
                .energy_honesty(EnergyHonestyClass::Estimated)
                .limit_fired(floor.clone())
                .fabric(fabric_chosen, cfg.fabric.clone())
                .executed(false)
                .rationale("ecosystem e2e REFUSE: capsule energy honesty violated");
            let mut receipt = builder.build();
            Self::stamp_meter(&mut receipt, effective_meter.as_ref());
            return EcosystemCertifyOutcome {
                committed: false,
                receipt,
                floor: Some(floor),
                capsule_return: capsule.return_i32,
                fuel_consumed: capsule.fuel_consumed,
                grant_id: None,
            };
        }

        // --- 5. Host invoke only with GrantReceipt ---
        let provenance = LaneProvenance::from_session(&session, PartitionSurface::Cookies);
        let grant = if cfg.refuse_without_grant {
            None
        } else {
            Some(GrantReceipt::issue(&session, cfg.host_capability, 1))
        };
        let host = host_invoke(&HostInvokeRequest {
            capability: cfg.host_capability,
            session: session.clone(),
            provenance: Some(provenance),
            keyword_confirm: None,
            grant: grant.clone(),
        });
        if !host.is_allow() {
            let floor = host.floor().cloned().unwrap_or_else(|| {
                Floor::new(
                    "grant_receipt_required",
                    FloorKind::AgentIsolation,
                    "host invoke refused without GrantReceipt",
                )
            });
            // Merge host refuse receipt partition stamp when present.
            let host_r = host.receipt().clone();
            agent_lane = host_r;
            let builder = Self::base_builder(cfg)
                .encapsulation(capsule.encapsulation.clone())
                .agent_lane(agent_lane)
                .compute_steps(compute_steps)
                .estimated_j(capsule.estimated_j)
                .estimate_kind(capsule.estimate_kind)
                .measure_source(capsule.measure_source)
                .energy_honesty(EnergyHonestyClass::Estimated)
                .limit_fired(floor.clone())
                .fabric(fabric_chosen, cfg.fabric.clone())
                .executed(false)
                .rationale(format!(
                    "ecosystem e2e REFUSE at host invoke: {}",
                    floor.reason
                ));
            let mut receipt = builder.build();
            Self::stamp_meter(&mut receipt, effective_meter.as_ref());
            return EcosystemCertifyOutcome {
                committed: false,
                receipt,
                floor: Some(floor),
                capsule_return: capsule.return_i32,
                fuel_consumed: capsule.fuel_consumed,
                grant_id: None,
            };
        }

        // Stamp grant onto agent_lane from host allow.
        agent_lane = host.receipt().clone();
        let grant_id = match &host {
            mol_core::HostInvokeDecision::Allow { grant_id, .. } => Some(grant_id.clone()),
            _ => agent_lane.grant_id.clone(),
        };

        // --- 6. COMMIT with full stamps ---
        let expected = cfg.add_args.0.wrapping_add(cfg.add_args.1);
        let answer = format!(
            "ECOSYSTEM_E2E_COMMIT session={} fabric={} capsule_add={} fuel={:?} grant={:?}",
            cfg.session_id,
            fabric_chosen.map(|k| k.label()).unwrap_or("none"),
            capsule.return_i32.unwrap_or(expected),
            capsule.fuel_consumed,
            grant_id
        );
        let builder = Self::base_builder(cfg)
            .answered(CascadeTier::Formula)
            .replay_class(ReplayClass::Deterministic)
            .answer(&answer)
            .encapsulation(capsule.encapsulation.clone())
            .agent_lane(agent_lane)
            .compute_steps(compute_steps)
            .estimated_j(capsule.estimated_j)
            .estimate_kind(capsule.estimate_kind)
            .measure_source(capsule.measure_source)
            .energy_honesty(EnergyHonestyClass::Estimated)
            .fabric(fabric_chosen, cfg.fabric.clone())
            .executed(true)
            .rationale(format!(
                "ecosystem e2e COMMIT: agent_lane + fabric:route({}) + kernel:vector_add + WASM capsule ({}) + GrantReceipt; energy_honesty=estimated (fuel); board_synth_claimed={}",
                fabric_chosen.map(|k| k.label()).unwrap_or("none"),
                runtime.name(),
                BOARD_SYNTH_CLAIMED
            ));
        let mut receipt = builder.build();
        Self::stamp_meter(&mut receipt, effective_meter.as_ref());
        // Rationale was built pre-stamp; refresh honesty label after real meter attach.
        if receipt.measured_j.is_some() {
            receipt.rationale = receipt.rationale.replace(
                "energy_honesty=estimated (fuel)",
                &format!(
                    "energy_honesty={} (measured_j from {})",
                    receipt.energy_honesty.label(),
                    receipt.measure_source.label()
                ),
            );
        }

        // Optional fail-closed meter gate (never invent measured_j to satisfy).
        if cfg.meter_required {
            if let Err(floor) =
                require_measured_or_refuse(receipt.measured_j, receipt.measure_source)
            {
                receipt.limit_fired = Some(floor.clone());
                receipt.executed = Some(false);
                receipt.rationale = format!(
                    "{}; meter_required REFUSE: {}",
                    receipt.rationale, floor.reason
                );
                return EcosystemCertifyOutcome {
                    committed: false,
                    receipt,
                    floor: Some(floor),
                    capsule_return: capsule.return_i32,
                    fuel_consumed: capsule.fuel_consumed,
                    grant_id,
                };
            }
        }

        // Final honesty: never invent measured_j; board_synth always false.
        if !energy_pair_honest(receipt.measured_j, receipt.measure_source)
            || receipt.board_synth_claimed
        {
            let floor = Floor::new(
                "energy_honesty_receipt",
                FloorKind::EnergyHonesty,
                "receipt energy pair dishonest or board_synth claimed",
            );
            receipt.limit_fired = Some(floor.clone());
            receipt.executed = Some(false);
            receipt.rationale = format!(
                "{}; post-commit honesty check REFUSE",
                receipt.rationale
            );
            return EcosystemCertifyOutcome {
                committed: false,
                receipt,
                floor: Some(floor),
                capsule_return: capsule.return_i32,
                fuel_consumed: capsule.fuel_consumed,
                grant_id,
            };
        }

        EcosystemCertifyOutcome {
            committed: true,
            receipt,
            floor: None,
            capsule_return: capsule.return_i32,
            fuel_consumed: capsule.fuel_consumed,
            grant_id,
        }
    }

    fn base_builder(cfg: &EcosystemCertifyConfig) -> ReceiptBuilder {
        ReceiptBuilder::new()
            .query(format!(
                "ecosystem_e2e_certify session={} add={}+{}",
                cfg.session_id, cfg.add_args.0, cfg.add_args.1
            ))
            .budget(cfg.budget)
            .mu_source(MuSource::Catalog)
            .measure_source(MeasureSource::CascadeEstimate)
            .energy_honesty(EnergyHonestyClass::Estimated)
            .estimate_kind(EstimateKind::Fixture)
    }

    /// Apply optional meter sample onto the receipt.
    ///
    /// Only stamps `measured_j` when the sample records a real measurement.
    /// Dishonest / empty samples leave the estimated fuel path (`measured_j=None`).
    /// **Never invent** package joules from missing data.
    fn stamp_meter(receipt: &mut MolReceipt, sample: Option<&MeterSample>) {
        if let Some(sample) = sample {
            if sample.honesty_ok() {
                receipt.apply_meter_sample(sample);
            } else {
                // Keep estimated path; do not invent.
                receipt.measured_j = None;
                if !receipt.measure_source.is_measured() {
                    receipt.refresh_energy_honesty();
                }
            }
        } else {
            // Soft-ref: fuel estimate only.
            receipt.measured_j = None;
            receipt.refresh_energy_honesty();
        }
    }

    fn refuse_early(
        cfg: &EcosystemCertifyConfig,
        floor: Floor,
        agent_lane: Option<mol_core::AgentLaneReceipt>,
        compute_steps: Vec<mol_core::ComputeStepReceipt>,
        rationale: &str,
    ) -> EcosystemCertifyOutcome {
        let mut b = Self::base_builder(cfg)
            .estimated_j(Joules::ZERO)
            .limit_fired(floor.clone())
            .compute_steps(compute_steps)
            .fabric_inventory(cfg.fabric.clone())
            .executed(false)
            .rationale(rationale);
        if let Some(a) = agent_lane {
            b = b.agent_lane(a);
        }
        let mut receipt = b.build();
        if let Some(ref sample) = cfg.meter_sample {
            if sample.honesty_ok() {
                receipt.apply_meter_sample(sample);
            }
        } else {
            receipt.refresh_energy_honesty();
        }
        EcosystemCertifyOutcome {
            committed: false,
            receipt,
            floor: Some(floor),
            capsule_return: None,
            fuel_consumed: None,
            grant_id: None,
        }
    }
}

/// Convenience: soft-ref happy path commit.
pub fn run_ecosystem_e2e_certify() -> EcosystemCertifyOutcome {
    EcosystemCertify::run(&EcosystemCertifyConfig::soft_ref())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn soft_ref_commits_with_full_stamps() {
        let out = EcosystemCertify::run(&EcosystemCertifyConfig::soft_ref());
        assert!(out.is_commit(), "rationale={}", out.receipt.rationale);
        assert_eq!(out.capsule_return, Some(42));
        assert!(out.receipt.encapsulation.as_ref().map(|e| e.sealed) == Some(true));
        assert!(out.receipt.agent_lane.is_some());
        assert!(out
            .receipt
            .agent_lane
            .as_ref()
            .unwrap()
            .grant_id
            .is_some());
        assert!(out
            .receipt
            .compute_steps
            .iter()
            .any(|s| s.label == "fabric:route"));
        assert!(out
            .receipt
            .compute_steps
            .iter()
            .any(|s| s.label == "capsule:invoke"));
        assert_eq!(out.receipt.fabric_chosen, Some(DeviceKind::Cpu));
        assert!(out.receipt.measured_j.is_none());
        assert!(!out.receipt.board_synth_claimed);
        assert!(out.receipt.energy_honesty_ok());
        assert_eq!(out.receipt.energy_honesty, EnergyHonestyClass::Estimated);
    }

    #[test]
    fn refuse_without_grant() {
        let out = EcosystemCertify::run(&EcosystemCertifyConfig::refuse_without_grant());
        assert!(out.is_refuse());
        assert_eq!(
            out.floor.as_ref().map(|f| f.id.as_str()),
            Some("grant_receipt_required")
        );
        // Capsule still ran and stamped encapsulation before host refuse.
        assert!(out.receipt.encapsulation.is_some());
        assert!(out.receipt.measured_j.is_none());
    }
}
