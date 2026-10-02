//! MixtureOfLimits router — owns close end-to-end when paired with certify.

use mol_adapters::{
    EfaCertificatePort, EfaDecision, EfaProposal, InCrateNiCertify,
    WcaCommitPort,
};
use mol_cascade::{CascadeEngine, CascadeResult, DistillStore, ModelStub};
use std::sync::Mutex;

use mol_core::{
    looks_recall_ask, looks_remember_ask, parse_recall_key, parse_remember, route_fabric,
    BitemporalStore, CascadeTier, DeviceKind, EstimateKind, FabricInventory, Floor, FloorKind,
    Joules, MeasureSource, MolRequest, MuSource, OpenIeZone, QueryKind, ReplayClass, Result,
    DEFAULT_THETA_BITS,
};
use mol_receipt::{CascadeStepOutcome, CascadeStepRecord, MolReceipt, ReceiptBuilder};

use crate::registry::{default_registry, LimitEval, NamedLimit};

/// Outcome of the Mixture of Limits router.
#[derive(Debug, Clone)]
pub enum MolOutcome {
    /// Cascade closed with an answer.
    Answered(CascadeResult),
    /// A limit fired before / instead of answering.
    Refused {
        /// Binding floor.
        floor: Floor,
        /// Receipt naming the limit.
        receipt: MolReceipt,
    },
}

impl MolOutcome {
    /// True if answered.
    pub fn is_answered(&self) -> bool {
        matches!(self, Self::Answered(r) if r.closed)
    }

    /// Receipt accessor.
    pub fn receipt(&self) -> &MolReceipt {
        match self {
            Self::Answered(r) => &r.receipt,
            Self::Refused { receipt, .. } => receipt,
        }
    }
}

/// End-to-end close outcome: propose → certify → commit|refuse → receipt.
#[derive(Debug, Clone)]
pub enum CloseOutcome {
    /// Certified commit (side effect not executed by MoL; caller may act).
    Commit {
        /// Cascade / route outcome that was certified.
        outcome: MolOutcome,
        /// Final receipt (may annotate certify).
        receipt: MolReceipt,
    },
    /// Refused at route or certify.
    Refuse {
        /// Binding floor.
        floor: Floor,
        /// Receipt.
        receipt: MolReceipt,
    },
}

impl CloseOutcome {
    /// True on certified commit.
    pub fn is_commit(&self) -> bool {
        matches!(self, Self::Commit { .. })
    }

    /// Receipt accessor.
    pub fn receipt(&self) -> &MolReceipt {
        match self {
            Self::Commit { receipt, .. } | Self::Refuse { receipt, .. } => receipt,
        }
    }
}

/// Mixture of Limits — evaluate named limits in order, then cascade.
///
/// Holds a bitemporal memory store. **Writes land only on [`Self::close`] COMMIT**
/// for well-formed remember/store asks (never on route alone or on refuse).
pub struct MixtureOfLimits {
    /// Ordered registry.
    pub limits: Vec<NamedLimit>,
    /// Cascade engine.
    pub cascade: CascadeEngine,
    /// Bitemporal agent memory (valid-time + transaction-time).
    /// Interior mutex so `close` can commit writes through `&self` (AutomateGate).
    pub memory: Mutex<BitemporalStore>,
    /// Available-device fabric inventory (shared with cascade; soft-ref Cpu always).
    pub fabric: FabricInventory,
}

impl Default for MixtureOfLimits {
    fn default() -> Self {
        Self::new()
    }
}

impl MixtureOfLimits {
    /// Default limits + cascade + empty bitemporal memory.
    pub fn new() -> Self {
        let fabric = FabricInventory::software_ref();
        Self {
            limits: default_registry(),
            cascade: CascadeEngine::new().with_fabric(fabric.clone()),
            memory: Mutex::new(BitemporalStore::new()),
            fabric,
        }
    }

    /// With custom registry.
    pub fn with_limits(limits: Vec<NamedLimit>) -> Self {
        let fabric = FabricInventory::software_ref();
        Self {
            limits,
            cascade: CascadeEngine::new().with_fabric(fabric.clone()),
            memory: Mutex::new(BitemporalStore::new()),
            fabric,
        }
    }

    /// With custom fabric inventory (propagates to cascade; preserves Model LAST leaf).
    pub fn with_fabric(mut self, fabric: FabricInventory) -> Self {
        self.fabric = fabric.clone();
        self.cascade = self.cascade.with_fabric(fabric);
        self
    }

    /// Replace Model LAST residual leaf (stub or OpenAI-compatible adapter).
    pub fn with_model(mut self, model: Box<dyn ModelStub>) -> Self {
        self.cascade = self.cascade.with_model(model);
        self
    }

    /// Attach distill store so second-pass Lookup/Formula closes without model.
    pub fn with_distill_store(mut self, store: DistillStore) -> Self {
        self.cascade = self.cascade.with_distill_store(store);
        self
    }

    /// Borrow memory store (lock).
    pub fn memory_lock(&self) -> std::sync::MutexGuard<'_, BitemporalStore> {
        self.memory.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// List limit ids.
    pub fn limit_ids(&self) -> Vec<&str> {
        self.limits.iter().map(|l| l.id.as_str()).collect()
    }

    /// Evaluate limits then cascade.
    ///
    /// Order: safety → energy → latency → VoI → … ; first bind wins.
    /// VoI on free-form without allow_model binds early (refuse excess tokens).
    /// Primitive gap binds when query names a missing Periodic Stack cell.
    /// Otherwise cascade runs; cascade may still refuse model / settle.


    /// Stamp encapsulation / agent_lane / compute_steps / energy_honesty onto a receipt.
    ///
    /// When `req.meter_sample` is present, apply it (real probe / fixture only —
    /// never invent `measured_j`).
    fn stamp_ecosystem_receipt(receipt: &mut mol_receipt::MolReceipt, req: &MolRequest) {
        use mol_core::{AgentLaneReceipt, ComputeStepReceipt, EncapsulationReceipt};
        if let Some(ref capsule) = req.capsule {
            receipt.encapsulation = Some(EncapsulationReceipt::from(capsule));
            receipt.compute_steps.push(
                ComputeStepReceipt::estimated(
                    "capsule:context",
                    None,
                    receipt.estimated_j,
                )
                .with_capsule(capsule.id.as_str()),
            );
        }
        if let Some(ref policy) = req.agent_lane {
            receipt.agent_lane = Some(AgentLaneReceipt::from(policy));
        }
        if let Some(ref sample) = req.meter_sample {
            receipt.apply_meter_sample(sample);
        } else {
            receipt.refresh_energy_honesty();
        }
    }

    /// Fail-closed ecosystem gate: encapsulation, agent isolation, provenance, meter honesty.
    ///
    /// Soft-ref default (`FailClosedPolicy::open`) skips all gates. Never invents `measured_j`.
    fn ecosystem_gate(&self, req: &MolRequest) -> Result<Option<MolOutcome>> {
        use mol_core::{
            require_measured_or_refuse, require_provenance_or_refuse, AgentLaneReceipt,
            EncapsulationReceipt, EnergyHonestyClass, EstimateKind, Floor, FloorKind, Joules,
            MeasureSource, MuSource,
        };

        if let Some(ref policy) = req.agent_lane {
            if let Err(floor) = policy.check() {
                let receipt = ReceiptBuilder::new()
                    .query(&req.query)
                    .estimated_j(Joules::ZERO)
                    .estimate_kind(EstimateKind::Analytical)
                    .measure_source(MeasureSource::CatalogSurrogate)
                    .mu_source(MuSource::Catalog)
                    .energy_honesty(EnergyHonestyClass::Estimated)
                    .agent_lane(AgentLaneReceipt::from(policy))
                    .limit_fired(floor.clone())
                    .budget(req.budget)
                    .fabric_inventory(self.fabric.clone())
                    .executed(false)
                    .rationale(format!("ecosystem_gate: agent isolation — {}", floor.reason))
                    .build();
                return Ok(Some(MolOutcome::Refused { floor, receipt }));
            }
        } else if req.fail_closed.require_agent_isolation {
            let floor = Floor::new(
                "agent_isolation_missing",
                FloorKind::AgentIsolation,
                "fail-closed: Agent Lane isolation policy required but absent",
            );
            let receipt = ReceiptBuilder::new()
                .query(&req.query)
                .estimated_j(Joules::ZERO)
                .estimate_kind(EstimateKind::Analytical)
                .measure_source(MeasureSource::CatalogSurrogate)
                .limit_fired(floor.clone())
                .budget(req.budget)
                .fabric_inventory(self.fabric.clone())
                .executed(false)
                .rationale("ecosystem_gate: missing agent isolation policy")
                .build();
            return Ok(Some(MolOutcome::Refused { floor, receipt }));
        }

        if let Some(ref capsule) = req.capsule {
            if let Err(floor) = capsule.check(req.fail_closed.require_capsule) {
                let receipt = ReceiptBuilder::new()
                    .query(&req.query)
                    .estimated_j(Joules::ZERO)
                    .estimate_kind(EstimateKind::Analytical)
                    .measure_source(MeasureSource::CatalogSurrogate)
                    .encapsulation(EncapsulationReceipt::from(capsule))
                    .limit_fired(floor.clone())
                    .budget(req.budget)
                    .fabric_inventory(self.fabric.clone())
                    .executed(false)
                    .rationale(format!("ecosystem_gate: encapsulation — {}", floor.reason))
                    .build();
                return Ok(Some(MolOutcome::Refused { floor, receipt }));
            }
        } else if req.fail_closed.require_capsule {
            let floor = Floor::new(
                "encapsulation_missing",
                FloorKind::Encapsulation,
                "fail-closed: sealed capsule required but absent",
            );
            let receipt = ReceiptBuilder::new()
                .query(&req.query)
                .estimated_j(Joules::ZERO)
                .estimate_kind(EstimateKind::Analytical)
                .measure_source(MeasureSource::CatalogSurrogate)
                .limit_fired(floor.clone())
                .budget(req.budget)
                .fabric_inventory(self.fabric.clone())
                .executed(false)
                .rationale("ecosystem_gate: missing capsule")
                .build();
            return Ok(Some(MolOutcome::Refused { floor, receipt }));
        }

        if req.fail_closed.require_measured {
            // Live / fixture meter path: commit only when sample supplies real measured_j
            // with Measured honesty. Probe fail / Modeled / missing → refuse. Never invent.
            let (measured_j, source) = match &req.meter_sample {
                Some(s) if s.honesty_ok() => (s.measured_j, s.source),
                Some(_) => (None, MeasureSource::Unavailable),
                None => (None, MeasureSource::Unavailable),
            };
            if let Err(floor) = require_measured_or_refuse(measured_j, source) {
                let honesty = EnergyHonestyClass::from_measure(source, measured_j);
                let mut receipt = ReceiptBuilder::new()
                    .query(&req.query)
                    .estimated_j(Joules::ZERO)
                    .estimate_kind(EstimateKind::Analytical)
                    .measure_source(MeasureSource::Unavailable)
                    .energy_honesty(EnergyHonestyClass::Unavailable)
                    .limit_fired(floor.clone())
                    .budget(req.budget)
                    .fabric_inventory(self.fabric.clone())
                    .executed(false)
                    .rationale(format!(
                        "ecosystem_gate: measured joules required; refuse (source={}, honesty={}, has_j={}) — never invent",
                        source.label(),
                        honesty.label(),
                        measured_j.is_some()
                    ))
                    .build();
                // Honest Modeled (CpuProxy) sample: stamp pair as Modeled — not Measured.
                // Soft-ref / probe-miss keeps measured_j=None + Unavailable (no invention).
                if let Some(ref sample) = req.meter_sample {
                    if sample.honesty_ok()
                        && EnergyHonestyClass::from_measure(sample.source, sample.measured_j)
                            == EnergyHonestyClass::Modeled
                    {
                        receipt.apply_meter_sample(sample);
                    }
                }
                return Ok(Some(MolOutcome::Refused { floor, receipt }));
            }
        }

        if req.fail_closed.require_citations {
            if let Err(floor) =
                require_provenance_or_refuse(false, req.capsule.is_some(), false, true, false, false)
            {
                let receipt = ReceiptBuilder::new()
                    .query(&req.query)
                    .estimated_j(Joules::ZERO)
                    .estimate_kind(EstimateKind::Analytical)
                    .measure_source(MeasureSource::CatalogSurrogate)
                    .limit_fired(floor.clone())
                    .budget(req.budget)
                    .fabric_inventory(self.fabric.clone())
                    .executed(false)
                    .rationale("ecosystem_gate: citations required")
                    .build();
                return Ok(Some(MolOutcome::Refused { floor, receipt }));
            }
        }

        Ok(None)
    }

    /// Route a request through named limits then the cascade.
    ///
    /// Ecosystem fail-closed / encapsulation / agent isolation runs first.
    pub fn route(&self, req: &MolRequest) -> Result<MolOutcome> {
        // Ecosystem fail-closed / encapsulation / agent isolation (before named floors).
        if let Some(refused) = self.ecosystem_gate(req)? {
            return Ok(refused);
        }

        // Bitemporal memory path (before named floors / cascade).
        if matches!(req.kind, QueryKind::MemoryWrite) || looks_remember_ask(&req.query) {
            return self.route_memory_write_propose(req);
        }
        if matches!(req.kind, QueryKind::MemoryRecall) || looks_recall_ask(&req.query) {
            return self.route_memory_recall(req);
        }

        let path_cost = 0.0;
        for lim in &self.limits {
            // Skip floors owned by cascade or automate certify (except primitive_gap / VoI / energy…).
            match lim.kind {
                FloorKind::GrammarCoverage
                | FloorKind::WcaRefuse
                | FloorKind::EfaCertificate
                | FloorKind::SettleRefuse
                | FloorKind::Information
                | FloorKind::Safety
                | FloorKind::Encapsulation
                | FloorKind::EnergyHonesty
                | FloorKind::ProvenanceMissing
                | FloorKind::AgentIsolation => continue,
                FloorKind::PrimitiveGap
                | FloorKind::Energy
                | FloorKind::Latency
                | FloorKind::ValueOfInformation
                | FloorKind::Satiation => {}
            }
            match lim.evaluate(req, path_cost) {
                LimitEval::Pass => {}
                LimitEval::Bind(floor) => {
                    let receipt = ReceiptBuilder::new()
                        .query(&req.query)
                        .estimated_j(Joules::ZERO)
                        .estimate_kind(EstimateKind::Analytical)
                        .measure_source(MeasureSource::CatalogSurrogate)
                        .mu_source(MuSource::Catalog)
                        .limit_fired(floor.clone())
                        .budget(req.budget)
                        .fabric_inventory(self.fabric.clone())
                        .executed(false)
                        .rationale(format!(
                            "MixtureOfLimits: {} fired before cascade; fabric_inventory=[{}]",
                            floor.id,
                            self.fabric.summary()
                        ))
                        .build();
                    return Ok(MolOutcome::Refused { floor, receipt });
                }
            }
        }

        let result = self.cascade.run(req)?;
        if result.closed {
            Ok(MolOutcome::Answered(result))
        } else if let Some(floor) = result.receipt.limit_fired.clone() {
            Ok(MolOutcome::Refused {
                floor,
                receipt: result.receipt,
            })
        } else {
            Ok(MolOutcome::Answered(result))
        }
    }

    /// Own close end-to-end: route → live in-crate NI/EFA/WCA certify → commit|refuse → receipt.
    ///
    /// Uses [`InCrateNiCertify`] (real certificate ids + commit|refuse receipt).
    /// Never fakes RAPL; `board_synth_claimed=false`; FPGA Stage C stays unmetered.
    pub fn close(&self, req: &MolRequest) -> Result<CloseOutcome> {
        self.close_live(req)
    }

    /// Live in-crate NI certify path (certificate ids stamped on receipt).
    pub fn close_live(&self, req: &MolRequest) -> Result<CloseOutcome> {
        let outcome = self.route(req)?;
        if !outcome.is_answered() {
            let floor = outcome
                .receipt()
                .limit_fired
                .clone()
                .unwrap_or_else(|| Floor::new("route_refuse", FloorKind::WcaRefuse, "route refused"));
            return Ok(CloseOutcome::Refuse {
                floor,
                receipt: outcome.receipt().clone(),
            });
        }

        let mut base = outcome.receipt().clone();
        Self::stamp_ecosystem_receipt(&mut base, req);
        let summary = base
            .answer
            .clone()
            .unwrap_or_else(|| req.query.clone());
        let estimated_j = base.estimated_j.0;
        let efa_tag = extract_efa_tag(&req.query);

        let ni = InCrateNiCertify::new();
        let live = ni.certify_live(&summary, estimated_j, efa_tag, None)?;
        let cert_ids = vec![
            live.ni.certificate_id.clone(),
            live.ni.efa_id.clone(),
            live.ni.wca_id.clone(),
        ];

        if !live.ni.allows_commit() {
            let floor = live.floor.unwrap_or_else(|| {
                Floor::new(
                    "ni_certificate",
                    FloorKind::EfaCertificate,
                    "NI certificate refuse",
                )
            });
            let limit_id = floor.id.as_str().to_string();
            // ModelGenerated uncertified → explicit ni_certificate id when applicable
            let floor = if base.replay_class == Some(ReplayClass::ModelGenerated)
                && limit_id != "wca_refuse"
            {
                Floor::new(
                    if limit_id == "efa_certificate" {
                        "efa_certificate"
                    } else {
                        "ni_certificate"
                    },
                    FloorKind::EfaCertificate,
                    floor.reason.clone(),
                )
            } else {
                floor
            };
            let mut rb = ReceiptBuilder::new()
                .query(&req.query)
                .estimated_j(Joules::new(estimated_j + live.ni.estimated_j))
                .estimate_kind(EstimateKind::Analytical)
                .measure_source(MeasureSource::CatalogSurrogate)
                .mu_source(MuSource::Catalog)
                .limit_fired(floor.clone())
                .steps(base.cascade_steps.clone())
                .budget(req.budget)
                .fabric(
                    base.fabric_chosen,
                    base.fabric_inventory
                        .clone()
                        .unwrap_or_else(|| self.fabric.clone()),
                )
                .certificate_ids(cert_ids)
                .executed(false)
                .rationale(format!(
                    "close REFUSE at live NI certify (ids={:?}; stage_c_measured=false): {}",
                    [
                        live.ni.certificate_id.as_str(),
                        live.ni.efa_id.as_str(),
                        live.ni.wca_id.as_str()
                    ],
                    live.ni.reasons.join("; ")
                ));
            if let Some(rc) = base.replay_class {
                rb = rb.replay_class(rc);
            }
            let receipt = rb.build();
            return Ok(CloseOutcome::Refuse { floor, receipt });
        }

        // ModelGenerated commits only with NI cert (already allowed above).
        let mut receipt = base;
        receipt.executed = Some(false);
        receipt.certificate_ids = cert_ids.clone();
        let cert_note = if receipt.replay_class == Some(ReplayClass::ModelGenerated) {
            format!(
                "; NI certificate stamped ids={:?}; replay stays ModelGenerated (never Deterministic); stage_c_measured=false",
                cert_ids
            )
        } else {
            format!("; NI certificate stamped ids={:?}; stage_c_measured=false", cert_ids)
        };
        receipt.rationale = format!(
            "{}; close COMMIT after live in-crate NI/EFA/WCA certify{cert_note}",
            receipt.rationale
        );

        if let Some(proposal) = parse_remember(&req.query) {
            let fact = {
                let mut store = self.memory_lock();
                store.commit_proposal(&proposal, Some(receipt.id.clone()))
            };
            receipt.answer = Some(format!(
                "MEMORY_COMMIT key={} value={} {} (valid_from={} tx_id={})",
                fact.key,
                fact.value,
                fact.cite(),
                fact.valid_from.to_rfc3339(),
                fact.tx_id
            ));
            receipt.citation_ids = vec![fact.id.as_str().to_string()];
            receipt.replay_class = Some(ReplayClass::Deterministic);
            receipt.zone = Some(OpenIeZone::Z1);
            receipt.cascade_answered = Some(CascadeTier::Lookup);
            receipt.executed = Some(true);
            receipt.rationale = format!(
                "{}; bitemporal memory WRITE committed via MoL close (tx_id={})",
                receipt.rationale, fact.tx_id
            );
        }

        Ok(CloseOutcome::Commit { outcome, receipt })
    }

    /// Close with injected certify ports (tests / future live adapters).
    pub fn close_with(
        &self,
        req: &MolRequest,
        efa: &dyn EfaCertificatePort,
        wca: &dyn WcaCommitPort,
    ) -> Result<CloseOutcome> {
        let outcome = self.route(req)?;
        if !outcome.is_answered() {
            let floor = outcome
                .receipt()
                .limit_fired
                .clone()
                .unwrap_or_else(|| Floor::new("route_refuse", FloorKind::WcaRefuse, "route refused"));
            return Ok(CloseOutcome::Refuse {
                floor,
                receipt: outcome.receipt().clone(),
            });
        }

        let mut base = outcome.receipt().clone();
        Self::stamp_ecosystem_receipt(&mut base, req);
        let summary = base
            .answer
            .clone()
            .unwrap_or_else(|| req.query.clone());
        let estimated_j = base.estimated_j.0;

        // EFA certificate first (BMI spirit).
        let efa_res = efa.certify(&EfaProposal {
            summary: summary.clone(),
            estimated_j,
            energy_residual: None,
            tag: extract_efa_tag(&req.query),
        })?;
        if efa_res.decision == EfaDecision::Refuse {
            let floor = efa_res.floor.unwrap_or_else(|| {
                Floor::new(
                    "efa_certificate",
                    FloorKind::EfaCertificate,
                    "EFA certificate refuse",
                )
            });
            let receipt = ReceiptBuilder::new()
                .query(&req.query)
                .estimated_j(Joules::new(estimated_j + efa_res.estimated_j))
                .estimate_kind(EstimateKind::Analytical)
                .measure_source(MeasureSource::CatalogSurrogate)
                .mu_source(MuSource::Catalog)
                .limit_fired(floor.clone())
                .steps(base.cascade_steps.clone())
                .budget(req.budget)
                .fabric(
                    base.fabric_chosen,
                    base.fabric_inventory.clone().unwrap_or_else(|| self.fabric.clone()),
                )
                .executed(false)
                .rationale(format!(
                    "close REFUSE at EFA certify: {}",
                    efa_res.reasons.join("; ")
                ))
                .build();
            return Ok(CloseOutcome::Refuse { floor, receipt });
        }

        // WCA certify.
        let wca_res = wca.certify(&summary, estimated_j)?;
        if wca_res.decision != "allow" {
            let floor = wca_res.floor.unwrap_or_else(|| {
                Floor::new("wca_refuse", FloorKind::WcaRefuse, "WCA certify refuse")
            });
            let receipt = ReceiptBuilder::new()
                .query(&req.query)
                .estimated_j(Joules::new(estimated_j))
                .estimate_kind(EstimateKind::Analytical)
                .measure_source(MeasureSource::CatalogSurrogate)
                .mu_source(MuSource::Catalog)
                .limit_fired(floor.clone())
                .steps(base.cascade_steps.clone())
                .budget(req.budget)
                .fabric(
                    base.fabric_chosen,
                    base.fabric_inventory.clone().unwrap_or_else(|| self.fabric.clone()),
                )
                .executed(false)
                .rationale(format!(
                    "close REFUSE at WCA certify: {}",
                    wca_res.reasons.join("; ")
                ))
                .build();
            return Ok(CloseOutcome::Refuse { floor, receipt });
        }

        // ModelGenerated never launders to Deterministic; commit only after NI cert.
        if base.replay_class == Some(ReplayClass::ModelGenerated) {
            if efa_res.decision != EfaDecision::Allow || wca_res.decision != "allow" {
                let floor = Floor::new(
                    "ni_certificate",
                    FloorKind::EfaCertificate,
                    "ModelGenerated proposal refused: NI/EFA certificate missing or fail",
                );
                let receipt = ReceiptBuilder::new()
                    .query(&req.query)
                    .estimated_j(Joules::new(estimated_j))
                    .estimate_kind(EstimateKind::Analytical)
                    .measure_source(MeasureSource::CatalogSurrogate)
                    .mu_source(MuSource::Catalog)
                    .replay_class(ReplayClass::ModelGenerated)
                    .limit_fired(floor.clone())
                    .steps(base.cascade_steps.clone())
                    .budget(req.budget)
                    .fabric(
                        base.fabric_chosen,
                        base.fabric_inventory
                            .clone()
                            .unwrap_or_else(|| self.fabric.clone()),
                    )
                    .executed(false)
                    .rationale(
                        "close REFUSE: ModelGenerated never commits without NI certificate",
                    )
                    .build();
                return Ok(CloseOutcome::Refuse { floor, receipt });
            }
        }

        let mut receipt = base;
        receipt.executed = Some(false);
        let inj_ids = vec![
            format!("efa-inj:{}", efa_res.reasons.first().cloned().unwrap_or_else(|| "allow".into())),
            format!("wca-inj:{}", wca_res.reasons.first().cloned().unwrap_or_else(|| "allow".into())),
        ];
        receipt.certificate_ids = inj_ids.clone();
        let cert_note = if receipt.replay_class == Some(ReplayClass::ModelGenerated) {
            format!(
                "; NI certificate stamped (efa+wca allow); replay stays ModelGenerated (never Deterministic); cert_ids={:?}",
                inj_ids
            )
        } else {
            format!("; cert_ids={:?}", inj_ids)
        };
        receipt.rationale = format!(
            "{}; close COMMIT after EFA+WCA certify (injected ports){cert_note}",
            receipt.rationale
        );

        // Memory write lands ONLY on certified COMMIT (never on route/refuse).
        if let Some(proposal) = parse_remember(&req.query) {
            let fact = {
                let mut store = self.memory_lock();
                store.commit_proposal(&proposal, Some(receipt.id.clone()))
            };
            receipt.answer = Some(format!(
                "MEMORY_COMMIT key={} value={} {} (valid_from={} tx_id={})",
                fact.key,
                fact.value,
                fact.cite(),
                fact.valid_from.to_rfc3339(),
                fact.tx_id
            ));
            receipt.citation_ids = vec![fact.id.as_str().to_string()];
            receipt.replay_class = Some(ReplayClass::Deterministic);
            receipt.zone = Some(OpenIeZone::Z1);
            receipt.cascade_answered = Some(CascadeTier::Lookup);
            receipt.executed = Some(true);
            receipt.rationale = format!(
                "{}; bitemporal memory WRITE committed via MoL close (tx_id={})",
                receipt.rationale, fact.tx_id
            );
        }

        Ok(CloseOutcome::Commit { outcome, receipt })
    }

    /// Propose a memory write (route only — does **not** mutate the store).
    fn route_memory_write_propose(&self, req: &MolRequest) -> Result<MolOutcome> {
        let Some(proposal) = parse_remember(&req.query) else {
            let floor = Floor::new(
                "memory_malformed",
                FloorKind::Information,
                "memory write ask missing key=value; refuse invent",
            );
            let receipt = ReceiptBuilder::new()
                .query(&req.query)
                .estimated_j(Joules::ZERO)
                .estimate_kind(EstimateKind::Analytical)
                .measure_source(MeasureSource::CatalogSurrogate)
                .mu_source(MuSource::Catalog)
                .limit_fired(floor.clone())
                .budget(req.budget)
                .fabric_inventory(self.fabric.clone())
                .executed(false)
                .rationale("MixtureOfLimits: malformed memory write before cascade")
                .build();
            return Ok(MolOutcome::Refused { floor, receipt });
        };
        let text = format!(
            "MEMORY_WRITE_PROPOSED key={} value={} (lands only on MoL close COMMIT)",
            proposal.key, proposal.value
        );
        let steps = vec![CascadeStepRecord {
            tier: CascadeTier::Lookup,
            us: 0,
            outcome: CascadeStepOutcome::Answered,
            estimated_j: Joules::new(1e-12),
        }];
        let fabric_dec = route_fabric(CascadeTier::Lookup, &self.fabric, &req.budget);
        let chosen = fabric_dec.chosen().or(Some(DeviceKind::Cpu));
        let receipt = ReceiptBuilder::new()
            .query(&req.query)
            .answered(CascadeTier::Lookup)
            .zone(OpenIeZone::Z1)
            .replay_class(ReplayClass::Deterministic)
            .answer(&text)
            .estimated_j(Joules::new(1e-12))
            .estimate_kind(EstimateKind::Analytical)
            .measure_source(MeasureSource::CatalogSurrogate)
            .mu_source(MuSource::Catalog)
            .steps(steps)
            .budget(req.budget)
            .landauer_bits(DEFAULT_THETA_BITS)
            .fabric(chosen, self.fabric.clone())
            .executed(false)
            .rationale(format!(
                "memory write proposed at Lookup; store unchanged until close COMMIT; model cold; fabric_chosen={}",
                chosen.map(|k| k.label()).unwrap_or("none")
            ))
            .build();
        Ok(MolOutcome::Answered(CascadeResult {
            answer: Some(text),
            receipt,
            closed: true,
        }))
    }

    /// Recall cited memory (bitemporal as-of now).
    fn route_memory_recall(&self, req: &MolRequest) -> Result<MolOutcome> {
        let Some(key) = parse_recall_key(&req.query) else {
            let floor = Floor::new(
                "memory_malformed",
                FloorKind::Information,
                "memory recall ask missing key; refuse invent",
            );
            let receipt = ReceiptBuilder::new()
                .query(&req.query)
                .estimated_j(Joules::ZERO)
                .estimate_kind(EstimateKind::Analytical)
                .measure_source(MeasureSource::CatalogSurrogate)
                .mu_source(MuSource::Catalog)
                .limit_fired(floor.clone())
                .budget(req.budget)
                .fabric_inventory(self.fabric.clone())
                .executed(false)
                .rationale("MixtureOfLimits: malformed memory recall")
                .build();
            return Ok(MolOutcome::Refused { floor, receipt });
        };
        let hit = {
            let store = self.memory_lock();
            store.recall(&key, None, None).cloned()
        };
        let Some(fact) = hit else {
            let floor = Floor::new(
                "memory_unknown",
                FloorKind::Information,
                format!("bitemporal recall: no fact for key '{key}'; refuse invent"),
            );
            let receipt = ReceiptBuilder::new()
                .query(&req.query)
                .estimated_j(Joules::ZERO)
                .estimate_kind(EstimateKind::Analytical)
                .measure_source(MeasureSource::CatalogSurrogate)
                .mu_source(MuSource::Catalog)
                .limit_fired(floor.clone())
                .budget(req.budget)
                .fabric_inventory(self.fabric.clone())
                .executed(false)
                .rationale(format!("MixtureOfLimits: memory_unknown for '{key}'"))
                .build();
            return Ok(MolOutcome::Refused { floor, receipt });
        };
        let text = fact.recalled_answer();
        let cites = vec![fact.id.as_str().to_string()];
        let steps = vec![CascadeStepRecord {
            tier: CascadeTier::Lookup,
            us: 0,
            outcome: CascadeStepOutcome::Answered,
            estimated_j: Joules::new(1e-12),
        }];
        let fabric_dec = route_fabric(CascadeTier::Lookup, &self.fabric, &req.budget);
        let chosen = fabric_dec.chosen().or(Some(DeviceKind::Cpu));
        let receipt = ReceiptBuilder::new()
            .query(&req.query)
            .answered(CascadeTier::Lookup)
            .zone(OpenIeZone::Z2)
            .replay_class(ReplayClass::RetrievedCited)
            .answer(&text)
            .citations(cites)
            .estimated_j(Joules::new(1e-12))
            .estimate_kind(EstimateKind::Analytical)
            .measure_source(MeasureSource::CatalogSurrogate)
            .mu_source(MuSource::Catalog)
            .steps(steps)
            .budget(req.budget)
            .landauer_bits(DEFAULT_THETA_BITS)
            .fabric(chosen, self.fabric.clone())
            .executed(false)
            .rationale(format!(
                "bitemporal memory RECALL cited {}; valid-time+tx-time as-of now; model cold; fabric_chosen={}",
                fact.id,
                chosen.map(|k| k.label()).unwrap_or("none")
            ))
            .build();
        Ok(MolOutcome::Answered(CascadeResult {
            answer: Some(text),
            receipt,
            closed: true,
        }))
    }
}

fn extract_efa_tag(query: &str) -> Option<String> {
    let q = query.to_ascii_lowercase();
    for t in ["diverge", "spoof", "uncertified", "false-safe", "efa refuse"] {
        if q.contains(t) {
            return Some(t.into());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use mol_core::{Budget, OpenIeZone, QueryKind, ReplayClass};

    #[test]
    fn voi_refuses_freeform() {
        let mol = MixtureOfLimits::new();
        let req = MolRequest::new("tell me a story about GPUs", Budget::demo());
        let out = mol.route(&req).unwrap();
        match out {
            MolOutcome::Refused { floor, .. } => {
                assert_eq!(floor.id.as_str(), "voi");
            }
            MolOutcome::Answered(_) => panic!("expected VoI refuse"),
        }
    }

    #[test]
    fn formula_answers() {
        let mol = MixtureOfLimits::new();
        let req = MolRequest::new("landauer joules per bit", Budget::coin_cell());
        let out = mol.route(&req).unwrap();
        assert!(out.is_answered());
    }

    #[test]
    fn primitive_gap_binds() {
        let mol = MixtureOfLimits::new();
        let req = MolRequest::new(
            "primitive gap: physical_settle not on stack",
            Budget::demo(),
        );
        let out = mol.route(&req).unwrap();
        match out {
            MolOutcome::Refused { floor, .. } => {
                assert_eq!(floor.id.as_str(), "primitive_gap");
                assert_eq!(floor.kind, FloorKind::PrimitiveGap);
            }
            MolOutcome::Answered(_) => panic!("expected primitive_gap"),
        }
    }

    #[test]
    fn close_commits_formula() {
        let mol = MixtureOfLimits::new();
        let req = MolRequest::new("landauer joules per bit", Budget::coin_cell());
        let out = mol.close(&req).unwrap();
        assert!(out.is_commit());
        assert!(!out.receipt().board_synth_claimed);
        assert!(out.receipt().measured_j.is_none());
    }

    #[test]
    fn close_refuses_efa_tag() {
        let mol = MixtureOfLimits::new();
        // Cascade will miss / VoI — use a formula query with diverge tag so route answers
        // then certify refuses. Attach tag to a formula-covered query.
        let req = MolRequest::new(
            "landauer joules per bit diverge",
            Budget::coin_cell(),
        );
        // kind still ClosedFormPhysics (landauer matches first... wait, classify order:
        // primitive_gap, settle, convert, landauer — "diverge" alone doesn't change kind)
        let out = mol.close(&req).unwrap();
        assert!(!out.is_commit());
        let floor = match &out {
            CloseOutcome::Refuse { floor, .. } => floor,
            CloseOutcome::Commit { .. } => panic!("expected EFA refuse"),
        };
        assert_eq!(floor.kind, FloorKind::EfaCertificate);
    }

    #[test]
    fn memory_write_only_on_close_commit() {
        let mol = MixtureOfLimits::new();
        let req = MolRequest::new("remember demo_key = hello_mol", Budget::demo());
        assert_eq!(req.kind, QueryKind::MemoryWrite);
        let routed = mol.route(&req).unwrap();
        assert!(routed.is_answered());
        assert!(mol.memory_lock().is_empty(), "route must not write");
        let closed = mol.close(&req).unwrap();
        assert!(closed.is_commit());
        assert_eq!(mol.memory_lock().len(), 1);
        let r = closed.receipt();
        assert!(r.citation_ids.iter().any(|c| c.starts_with("memory:")));
        assert!(r.measured_j.is_none());
        assert!(!r.board_synth_claimed);
    }

    #[test]
    fn memory_recall_cites_after_commit() {
        let mol = MixtureOfLimits::new();
        mol.close(&MolRequest::new(
            "remember landauer_note = E_min = kT ln2",
            Budget::demo(),
        ))
        .unwrap();
        let out = mol
            .close(&MolRequest::new("recall landauer_note", Budget::demo()))
            .unwrap();
        assert!(out.is_commit());
        let r = out.receipt();
        assert_eq!(r.replay_class, Some(ReplayClass::RetrievedCited));
        assert_eq!(r.zone, Some(OpenIeZone::Z2));
        assert!(!r.citation_ids.is_empty());
        assert!(r.answer.as_deref().unwrap_or("").contains("cite:memory:"));
    }

    #[test]
    fn memory_unknown_refuses() {
        let mol = MixtureOfLimits::new();
        let out = mol
            .close(&MolRequest::new("recall missing_key_xyz", Budget::demo()))
            .unwrap();
        assert!(!out.is_commit());
        assert_eq!(
            out.receipt().limit_fired.as_ref().map(|f| f.id.as_str()),
            Some("memory_unknown")
        );
    }

    #[test]
    fn meter_required_refuses_without_sample() {
        use mol_core::{EnergyHonestyClass, FailClosedPolicy, FloorKind};
        let mol = MixtureOfLimits::new();
        let out = mol
            .close(
                &MolRequest::new("landauer joules per bit", Budget::coin_cell())
                    .with_fail_closed(FailClosedPolicy::meter_required()),
            )
            .unwrap();
        assert!(!out.is_commit());
        assert_eq!(
            out.receipt().limit_fired.as_ref().map(|f| f.kind),
            Some(FloorKind::EnergyHonesty)
        );
        assert!(out.receipt().measured_j.is_none());
        // build() refreshes honesty: Unavailable source + None → Estimated (pair-honest).
        assert!(
            matches!(
                out.receipt().energy_honesty,
                EnergyHonestyClass::Unavailable | EnergyHonestyClass::Estimated
            ),
            "got {:?}",
            out.receipt().energy_honesty
        );
        assert_eq!(out.receipt().measure_source, mol_core::MeasureSource::Unavailable);
    }

    #[test]
    fn meter_required_commits_with_rapl_fixture() {
        use mol_core::{
            sample_from_rapl_counters, EnergyHonestyClass, FailClosedPolicy, MeasureSource,
            RaplCounter,
        };
        let before = vec![RaplCounter {
            name: "package-0".into(),
            energy_uj: 1_000_000,
            max_energy_uj: Some(10_000_000),
        }];
        let after = vec![RaplCounter {
            name: "package-0".into(),
            energy_uj: 2_500_000,
            max_energy_uj: Some(10_000_000),
        }];
        let sample = sample_from_rapl_counters(&before, &after, 20);
        assert!(sample.records_measurement(), "{}", sample.detail);
        assert!(sample.measured_j.is_some());

        let mol = MixtureOfLimits::new();
        let out = mol
            .close(
                &MolRequest::new("landauer joules per bit", Budget::coin_cell())
                    .with_fail_closed(FailClosedPolicy::meter_required())
                    .with_meter_sample(sample.clone()),
            )
            .unwrap();
        assert!(out.is_commit(), "rationale={}", out.receipt().rationale);
        let r = out.receipt();
        assert_eq!(r.measured_j, sample.measured_j);
        assert_eq!(r.measure_source, MeasureSource::Rapl);
        assert_eq!(r.energy_honesty, EnergyHonestyClass::Measured);
        assert!(r.energy_honesty_ok());
    }

    #[test]
    fn meter_required_refuses_unavailable_sample() {
        use mol_core::{FailClosedPolicy, FloorKind, MeterSample};
        let mol = MixtureOfLimits::new();
        let out = mol
            .close(
                &MolRequest::new("landauer joules per bit", Budget::coin_cell())
                    .with_fail_closed(FailClosedPolicy::meter_required())
                    .with_meter_sample(MeterSample::unavailable(10, "probe failed")),
            )
            .unwrap();
        assert!(!out.is_commit());
        assert_eq!(
            out.receipt().limit_fired.as_ref().map(|f| f.kind),
            Some(FloorKind::EnergyHonesty)
        );
        assert!(out.receipt().measured_j.is_none());
    }

    #[test]
    fn meter_required_refuses_modeled_cpu_proxy() {
        use mol_core::{
            ComponentJoules, EnergyHonestyClass, FailClosedPolicy, FloorKind, Joules,
            MeasureSource, MeterComponent, MeterSample,
        };
        let sample = MeterSample {
            measured_j: Some(Joules::new(0.05)),
            source: MeasureSource::CpuProxy,
            components: vec![ComponentJoules {
                component: MeterComponent::Package,
                joules: Joules::new(0.05),
                measure_source: MeasureSource::CpuProxy,
            }],
            window_ms: 10,
            detail: "cpu proxy fixture".into(),
        };
        assert!(sample.honesty_ok());
        let mol = MixtureOfLimits::new();
        let out = mol
            .close(
                &MolRequest::new("landauer joules per bit", Budget::coin_cell())
                    .with_fail_closed(FailClosedPolicy::meter_required())
                    .with_meter_sample(sample),
            )
            .unwrap();
        assert!(!out.is_commit());
        assert_eq!(
            out.receipt().limit_fired.as_ref().map(|f| f.kind),
            Some(FloorKind::EnergyHonesty)
        );
        // CpuProxy joules may appear (pair-honest) but class must stay Modeled — not Measured.
        assert_eq!(out.receipt().energy_honesty, EnergyHonestyClass::Modeled);
        assert_eq!(out.receipt().measure_source, MeasureSource::CpuProxy);
        assert!(out.receipt().energy_honesty_ok());
        assert_ne!(out.receipt().energy_honesty, EnergyHonestyClass::Measured);
    }
}
