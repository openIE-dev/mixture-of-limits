//! Optional egui/eframe energy harness window (`gui` feature).
//!
//! Minimal SOTA surface: ask box, close, EcosystemCertify, receipt fields
//! (zone/fabric/limit/estimated_j/measured_j honesty, encapsulation, agent_lane,
//! fabric_id, kernel:vector_add proof, compute_steps), fabric inventory, joule ledger.
//! IDE chrome is secondary to CI law.

use eframe::egui;
use mol_core::Budget;

use crate::shell::{AskOpts, EcosystemCertifyOpts, ShellSession};
use crate::view::{ConfirmResponse, ReceiptView};

/// Run the native window (blocks).
pub fn run_native() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([980.0, 720.0])
            .with_title("MoL Energy Harness"),
        ..Default::default()
    };
    eframe::run_native(
        "MoL Energy Harness",
        options,
        Box::new(|_cc| Ok(Box::new(MolDesktopApp::new()))),
    )
}

struct MolDesktopApp {
    session: ShellSession,
    ask: String,
    attach_meter: bool,
    meter_required: bool,
    // EcosystemCertify controls
    eco_detect: bool,
    eco_tier_model: bool, // false = formula, true = model
    eco_meter_required: bool,
    eco_meter_ms: String,
    eco_refuse_without_grant: bool,
    status: String,
    last_harness: String,
    receipt_json: String,
    last_view: Option<ReceiptView>,
    eco_extra: String,
}

impl MolDesktopApp {
    fn new() -> Self {
        let session = ShellSession::new();
        let fabric = session.fabric_view().summary.clone();
        Self {
            session,
            ask: "landauer joules per bit".into(),
            attach_meter: false,
            meter_required: false,
            eco_detect: false,
            eco_tier_model: false,
            eco_meter_required: false,
            eco_meter_ms: "50".into(),
            eco_refuse_without_grant: false,
            status: format!("fabric: {fabric}"),
            last_harness: String::new(),
            receipt_json: String::new(),
            last_view: None,
            eco_extra: String::new(),
        }
    }

    fn apply_view(&mut self, view: ReceiptView, commit: bool, meter_detail: Option<String>) {
        self.last_harness = view.harness_line();
        self.status = format!(
            "{} | ledger acts={} total_est_j={:.3e}",
            if commit { "COMMIT" } else { "REFUSE" },
            self.session.ledger_summary().acts,
            self.session.ledger_summary().total_estimated_j
        );
        if let Some(d) = meter_detail {
            self.status = format!("{} | meter: {}", self.status, d);
        }
        self.receipt_json = serde_json::to_string_pretty(&view).unwrap_or_default();
        self.last_view = Some(view);
    }

    fn do_close(&mut self) {
        let mut opts = AskOpts {
            budget: Budget::demo(),
            attach_meter: self.attach_meter || self.meter_required,
            ..AskOpts::default()
        };
        if self.meter_required {
            let window = opts.meter_window;
            opts = opts.with_meter_required(window);
        }
        match self.session.ask_close(self.ask.clone(), opts) {
            Ok(r) => {
                self.eco_extra.clear();
                self.apply_view(r.view, r.commit, r.meter_detail);
            }
            Err(e) => {
                self.status = format!("error: {e}");
            }
        }
    }

    fn do_ecosystem_certify(&mut self) {
        let meter_ms: u64 = self.eco_meter_ms.trim().parse().unwrap_or(50);
        let opts = EcosystemCertifyOpts {
            detect: self.eco_detect,
            tier: if self.eco_tier_model {
                "model".into()
            } else {
                "formula".into()
            },
            meter_required: self.eco_meter_required,
            meter_ms,
            attach_meter: self.eco_meter_required,
            refuse_without_grant: self.eco_refuse_without_grant,
            session_id: "mol-desktop-ecosystem".into(),
        };
        match self.session.ecosystem_certify(opts) {
            Ok(r) => {
                self.eco_extra = format!(
                    "ecosystem: tier={} detect_feature={} fabric=[{}] capsule_return={:?} fuel={:?} grant={:?} floor={:?}",
                    r.schedule_tier,
                    r.fabric_detect_enabled,
                    r.fabric_summary,
                    r.capsule_return,
                    r.fuel_consumed,
                    r.grant_id,
                    r.floor_id
                );
                self.apply_view(r.view, r.commit, None);
            }
            Err(e) => {
                self.status = format!("ecosystem error: {e}");
            }
        }
    }
}

fn show_ecosystem_stamps(ui: &mut egui::Ui, view: &ReceiptView) {
    ui.group(|ui| {
        ui.strong("Ecosystem stamps");
        ui.monospace(format!(
            "decision={}  energy_honesty={}  (measured_j={} · estimate_kind={} · measure_source={})",
            view.decision,
            view.energy_honesty,
            view
                .measured_j
                .map(|j| format!("{j:.3e}"))
                .unwrap_or_else(|| "None".into()),
            view.estimate_kind,
            view.measure_source
        ));
        ui.small("Labels: Measured = real OS meter; Estimated/Modeled ≠ Measured — never invent measured_j");
        ui.monospace(format!(
            "fabric_chosen={}  zone={}  limit={}",
            view.fabric.as_deref().unwrap_or("-"),
            view.zone.as_deref().unwrap_or("-"),
            view.limit.as_deref().unwrap_or("-")
        ));
        match &view.encapsulation {
            Some(e) => ui.monospace(format!(
                "encapsulation: id={} boundary={} sealed={} host_share={}",
                e.capsule_id, e.boundary, e.sealed, e.shares_host_session
            )),
            None => ui.monospace("encapsulation: missing (refuse stamp when required)"),
        };
        match &view.agent_lane {
            Some(a) => ui.monospace(format!(
                "agent_lane: {} profile={} cookies={} storage={} opaque_bot={}",
                a.lane,
                a.separate_profile,
                a.separate_cookies,
                a.separate_storage,
                a.allow_shared_opaque_bot
            )),
            None => ui.monospace("agent_lane: missing (refuse stamp when required)"),
        };
        if view.compute_steps.is_empty() {
            ui.monospace("compute_steps: (none)");
        } else {
            ui.monospace(format!("compute_steps: {}", view.compute_steps.len()));
            for s in &view.compute_steps {
                let fid = s.fabric_id.as_deref().unwrap_or("-");
                let meas = s
                    .measured_j
                    .map(|j| format!("{j:.2e}"))
                    .unwrap_or_else(|| "None".into());
                ui.small(format!(
                    "  · {} fabric_id={} honesty={} est={:.2e} measured={}{}",
                    s.label,
                    fid,
                    s.honesty,
                    s.estimated_j,
                    meas,
                    s.unavailable_reason
                        .as_ref()
                        .map(|r| format!(" unavailable={r}"))
                        .unwrap_or_default()
                ));
                if s.label.starts_with("kernel:") {
                    ui.small(format!(
                        "      kernel proof: {}",
                        s.execution_proof.as_deref().unwrap_or("(none)")
                    ));
                } else if let Some(ref proof) = s.execution_proof {
                    ui.small(format!("      execution_proof: {proof}"));
                }
            }
        }
    });
}

impl eframe::App for MolDesktopApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.heading("MoL Energy Harness");
            ui.label(
                "Joules-per-verified-decision · commit|refuse · multi-fabric · EcosystemCertify · not a chat clone",
            );
        });

        egui::SidePanel::left("fabric").default_width(240.0).show(ctx, |ui| {
            ui.heading("Fabric inventory");
            let f = self.session.fabric_view();
            ui.monospace(&f.summary);
            ui.small(&f.honesty_note);
            ui.separator();
            ui.heading("Meter (ask/close)");
            ui.monospace(self.session.meter_status());
            ui.checkbox(&mut self.attach_meter, "attach OS meter on close");
            ui.checkbox(
                &mut self.meter_required,
                "meter_required (fail-closed; refuse if no measured_j)",
            );
            ui.small("Never invent measured_j · Measured ≠ Modeled/Estimated");
            ui.separator();
            ui.heading("Ledger");
            let s = self.session.ledger_summary();
            ui.label(format!(
                "acts={} commits={} refuses={}",
                s.acts, s.commits, s.refuses
            ));
            ui.label(format!("total_est_j={:.3e}", s.total_estimated_j));
            ui.label(format!("mean_est_j={:.3e}", s.mean_estimated_j));
            if let Some(r) = s.mean_landauer_ratio {
                ui.label(format!("mean_landauer_ratio={r:.3e}"));
            }
            egui::ScrollArea::vertical().max_height(220.0).show(ui, |ui| {
                for e in self.session.ledger().iter().rev().take(12) {
                    ui.small(format!(
                        "#{} {} honesty={} est={:.2e} measured={} lim={} enc={} agent={} steps={}",
                        e.seq,
                        e.decision,
                        e.energy_honesty,
                        e.estimated_j,
                        e.measured_j
                            .map(|j| format!("{j:.2e}"))
                            .unwrap_or_else(|| "None".into()),
                        e.limit.as_deref().unwrap_or("-"),
                        e.encapsulation,
                        e.agent_lane,
                        e.compute_steps
                    ));
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("Ask / Close");
                ui.text_edit_multiline(&mut self.ask);
                ui.horizontal(|ui| {
                    if ui.button("Close (MoL owns commit|refuse)").clicked() {
                        self.do_close();
                    }
                    if ui.button("Clear ask").clicked() {
                        self.ask.clear();
                    }
                });

                ui.separator();
                ui.heading("EcosystemCertify");
                ui.small(
                    "One receipt: Agent Lane + fabric schedule + WASM capsule + GrantReceipt + optional meter/kernel",
                );
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.eco_detect, "detect (wgpu / fabric-detect)");
                    ui.label("tier:");
                    ui.radio_value(&mut self.eco_tier_model, false, "formula");
                    ui.radio_value(&mut self.eco_tier_model, true, "model");
                });
                ui.horizontal(|ui| {
                    ui.checkbox(
                        &mut self.eco_meter_required,
                        "meter_required (fail-closed)",
                    );
                    ui.label("meter-ms:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.eco_meter_ms)
                            .desired_width(64.0)
                            .hint_text("50"),
                    );
                    ui.checkbox(
                        &mut self.eco_refuse_without_grant,
                        "refuse without grant",
                    );
                });
                if ui
                    .button("Run EcosystemCertify")
                    .on_hover_text(
                        "Soft-ref Formula commits offline. Model+detect needs fabric-detect; meter_required needs energy-meter + live probe.",
                    )
                    .clicked()
                {
                    self.do_ecosystem_certify();
                }
                if !self.eco_extra.is_empty() {
                    ui.monospace(&self.eco_extra);
                }

                ui.separator();
                ui.label(&self.status);
                if !self.last_harness.is_empty() {
                    ui.group(|ui| {
                        ui.strong("Receipt harness");
                        ui.monospace(&self.last_harness);
                    });
                }
                if let Some(ref view) = self.last_view {
                    show_ecosystem_stamps(ui, view);
                } else if let Ok(view) =
                    serde_json::from_str::<ReceiptView>(&self.receipt_json)
                {
                    show_ecosystem_stamps(ui, &view);
                }
                ui.separator();
                ui.collapsing("Receipt view JSON", |ui| {
                    egui::ScrollArea::vertical().max_height(240.0).show(ui, |ui| {
                        ui.monospace(&self.receipt_json);
                    });
                });
                ui.separator();
                ui.small(
                    "Interface tax: human confirm only on consequential acts; machine protocol lane is ShellSession.",
                );
                let _ = ConfirmResponse::Approve; // keep confirm types linked for future panel
            });
        });
    }
}
