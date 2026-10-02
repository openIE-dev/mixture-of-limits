//! Phase-1 bounded micro-perception — unstructured → typed AST.
//!
//! Minimal rule-based transducer (not a second Model LAST). When
//! `phase1.enabled`, free text / email-ish input emits a typed task
//! coordinate for Phase-2 Mixture of Limits cascade:
//! Lookup → Formula → Solver → Model LAST.
//!
//! Covers unstructured ticket / risk / decision strings used by `mol arena`
//! and `mol phase1`. Does **not** invent `measured_j` (catalog `estimated_j` only).

use serde::{Deserialize, Serialize};

use crate::request::QueryKind;

/// Phase-1 gate from chore YAML / CLI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Phase1Config {
    /// When false, tickets arrive typed (MVP default).
    #[serde(default)]
    pub enabled: bool,
    /// Transducer label (`rule_ast` for the in-tree minimal emitter).
    #[serde(default = "default_transducer")]
    pub transducer: String,
}

fn default_transducer() -> String {
    "rule_ast".into()
}

/// Typed AST emitted by Phase-1 (feeds Phase-2 as a typed ask).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypedAst {
    /// Task kind / schema id.
    pub kind: String,
    /// Normalized query for Mixture of Limits cascade.
    pub typed_query: String,
    /// Optional query-kind hint.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub query_kind: Option<QueryKind>,
    /// Provenance note (rule id).
    pub rule: String,
    /// Estimated joules for the transducer step (catalog; never measured_j).
    pub estimated_j: f64,
}

/// Result of Phase-1 attempt.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum Phase1Outcome {
    /// Emitted typed AST.
    Typed(TypedAst),
    /// Phase-1 disabled — pass-through.
    Passthrough {
        /// Original raw input.
        raw: String,
    },
    /// No rule matched; refuse inventing a parser-as-model.
    Unrecognized {
        /// Raw input.
        raw: String,
        /// Reason.
        reason: String,
    },
}

/// Minimal rule-based transducer (real path when `phase1.enabled`).
///
/// Covers support-desk ticket phrases, risk band / closed-form risk cues,
/// typed decision phrases, unit convert, landauer/physics asks, settle, and
/// residual-propose markers. Does **not** open Model LAST.
pub fn transduce(raw: &str) -> Phase1Outcome {
    let s = raw.trim();
    if s.is_empty() {
        return Phase1Outcome::Unrecognized {
            raw: raw.into(),
            reason: "empty input".into(),
        };
    }
    let lower = s.to_ascii_lowercase();

    // ── Ticket-close (unstructured email / chat → typed resolution)
    if looks_ticket(&lower) {
        let code = extract_resolution(&lower).unwrap_or("R-HOWTO");
        return Phase1Outcome::Typed(TypedAst {
            kind: "support_desk.ticket_close".into(),
            typed_query: format!("ticket close resolution={code}"),
            query_kind: Some(QueryKind::TicketClose),
            rule: "ticket_close_resolution".into(),
            estimated_j: 1e-12,
        });
    }

    // ── Risk band LUT (unstructured → risk score band=RISK-*)
    if let Some(band) = extract_risk_band(&lower) {
        if looks_risk(&lower) {
            return Phase1Outcome::Typed(TypedAst {
                kind: "risk.band".into(),
                typed_query: format!("risk score band={band}"),
                query_kind: Some(QueryKind::TicketClose),
                rule: "risk_band_heuristic".into(),
                estimated_j: 1e-12,
            });
        }
    }

    // ── Closed-form risk Formula (features in text → risk score compute …)
    if looks_risk(&lower) {
        if let Some((sev, exp, lik)) = extract_risk_features(&lower) {
            return Phase1Outcome::Typed(TypedAst {
                kind: "risk.formula".into(),
                typed_query: format!(
                    "risk score compute severity={sev} exposure={exp} likelihood={lik}"
                ),
                query_kind: Some(QueryKind::ClosedFormPhysics),
                rule: "risk_formula_features".into(),
                estimated_j: 1.5e-12,
            });
        }
    }

    // ── Typed decision (approve / deny / escalate phrases)
    if let Some(pick) = extract_decision(&lower) {
        return Phase1Outcome::Typed(TypedAst {
            kind: "typed.decision".into(),
            typed_query: format!(
                "typed decide pick={pick} options=[D-APPROVE,D-DENY,D-ESCALATE]"
            ),
            query_kind: Some(QueryKind::TicketClose),
            rule: "typed_decision_heuristic".into(),
            estimated_j: 1e-12,
        });
    }

    // Unit convert
    if lower.contains("convert")
        && (lower.contains("celsius")
            || lower.contains("fahrenheit")
            || lower.contains("joule")
            || lower.contains("electronvolt"))
    {
        return Phase1Outcome::Typed(TypedAst {
            kind: "unit_convert".into(),
            typed_query: normalize_whitespace(s),
            query_kind: Some(QueryKind::UnitConvert),
            rule: "unit_convert".into(),
            estimated_j: 1e-12,
        });
    }

    // Landauer / closed-form physics
    if lower.contains("landauer")
        || lower.contains("shannon capacity")
        || lower.contains("nyquist")
    {
        return Phase1Outcome::Typed(TypedAst {
            kind: "closed_form_physics".into(),
            typed_query: normalize_whitespace(s),
            query_kind: Some(QueryKind::ClosedFormPhysics),
            rule: "physics_formula".into(),
            estimated_j: 1e-12,
        });
    }

    // Ternary settle
    if lower.contains("settle") {
        return Phase1Outcome::Typed(TypedAst {
            kind: "ternary_settle".into(),
            typed_query: normalize_whitespace(s),
            query_kind: Some(QueryKind::Settle),
            rule: "settle".into(),
            estimated_j: 1e-12,
        });
    }

    // Residual propose marker (still ModelGenerated leaf later)
    if lower.contains("residual propose") || lower.contains("model propose") {
        return Phase1Outcome::Typed(TypedAst {
            kind: "residual_propose".into(),
            typed_query: normalize_whitespace(s),
            query_kind: Some(QueryKind::FreeForm),
            rule: "residual_marker".into(),
            estimated_j: 1e-12,
        });
    }

    // Free-form email-ish → try ticket heuristics (default R-OK)
    if lower.contains("please close") || lower.contains("mark resolved") {
        return Phase1Outcome::Typed(TypedAst {
            kind: "support_desk.ticket_close".into(),
            typed_query: "ticket close resolution=R-OK".into(),
            query_kind: Some(QueryKind::TicketClose),
            rule: "email_close_heuristic".into(),
            estimated_j: 2e-12,
        });
    }

    Phase1Outcome::Unrecognized {
        raw: raw.into(),
        reason: "no Phase-1 rule matched; refuse parser-as-model (enable Model LAST only under VoI+budget+cert)".into(),
    }
}

/// Run Phase-1 according to config.
pub fn run_phase1(cfg: &Phase1Config, raw: &str) -> Phase1Outcome {
    if !cfg.enabled {
        return Phase1Outcome::Passthrough { raw: raw.into() };
    }
    transduce(raw)
}

fn looks_ticket(lower: &str) -> bool {
    (lower.contains("ticket")
        && (lower.contains("close")
            || lower.contains("resolution")
            || lower.contains("resolve")
            || lower.contains("closed")))
        || (lower.contains("support")
            && (lower.contains("close") || lower.contains("resolution")))
        || lower.contains("please close this ticket")
        || lower.contains("close this ticket")
        || (lower.contains("case") && lower.contains("close") && extract_resolution(lower).is_some())
}

fn looks_risk(lower: &str) -> bool {
    lower.contains("risk")
        || lower.contains("severity")
        || lower.contains("exposure")
        || lower.contains("likelihood")
        || lower.contains("risk-low")
        || lower.contains("risk-med")
        || lower.contains("risk-high")
}

fn extract_resolution(lower: &str) -> Option<&'static str> {
    // Explicit codes first (longest / most specific).
    let codes: &[(&str, &str)] = &[
        ("r-bugfix", "R-BUGFIX"),
        ("r-wontfix", "R-WONTFIX"),
        ("r-howto", "R-HOWTO"),
        ("r-refund", "R-REFUND"),
        ("r-dup", "R-DUP"),
        ("r-ok", "R-OK"),
        ("r-bug", "R-BUGFIX"), // alias → LUT row
    ];
    for (needle, code) in codes {
        if lower.contains(needle) {
            return Some(code);
        }
    }
    // Natural-language cues
    if lower.contains("duplicate") || lower.contains("already filed") || lower.contains("already open")
    {
        return Some("R-DUP");
    }
    if lower.contains("refund") || lower.contains("billing adjustment") || lower.contains("chargeback")
    {
        return Some("R-REFUND");
    }
    if lower.contains("won't fix")
        || lower.contains("wontfix")
        || lower.contains("out of scope")
        || lower.contains("not planned")
    {
        return Some("R-WONTFIX");
    }
    if lower.contains("bugfix")
        || lower.contains("fix shipped")
        || lower.contains("patched")
        || (lower.contains("bug") && lower.contains("fixed"))
    {
        return Some("R-BUGFIX");
    }
    if lower.contains("how to")
        || lower.contains("howto")
        || lower.contains("knowledge base")
        || lower.contains("kb article")
        || lower.contains("docs answer")
    {
        return Some("R-HOWTO");
    }
    if lower.contains("as designed")
        || lower.contains("works as expected")
        || lower.contains("resolved ok")
        || lower.contains("all good")
    {
        return Some("R-OK");
    }
    None
}

fn extract_risk_band(lower: &str) -> Option<&'static str> {
    if lower.contains("risk-high")
        || lower.contains("band=risk-high")
        || lower.contains("high risk")
        || lower.contains("risk is high")
        || lower.contains("risk: high")
        || lower.contains("severity high")
        || (lower.contains("severe") && lower.contains("risk"))
    {
        return Some("RISK-HIGH");
    }
    if lower.contains("risk-med")
        || lower.contains("risk-medium")
        || lower.contains("band=risk-med")
        || lower.contains("medium risk")
        || lower.contains("moderate risk")
        || lower.contains("risk is medium")
        || lower.contains("risk is med")
        || lower.contains("risk: med")
    {
        return Some("RISK-MED");
    }
    if lower.contains("risk-low")
        || lower.contains("band=risk-low")
        || lower.contains("low risk")
        || lower.contains("risk is low")
        || lower.contains("risk: low")
        || lower.contains("negligible risk")
    {
        return Some("RISK-LOW");
    }
    None
}

fn extract_risk_features(lower: &str) -> Option<(u32, f64, f64)> {
    // Prefer explicit key=value if present.
    let sev = extract_f64_key(lower, "severity").map(|v| v.round() as u32);
    let exp = extract_f64_key(lower, "exposure");
    let lik = extract_f64_key(lower, "likelihood").or_else(|| extract_f64_key(lower, "prob"));
    if let (Some(s), Some(e), Some(l)) = (sev, exp, lik) {
        return Some((s.clamp(1, 5), e.clamp(0.0, 1.0), l.clamp(0.0, 1.0)));
    }
    None
}

fn extract_f64_key(lower: &str, key: &str) -> Option<f64> {
    let pat = format!("{key}=");
    if let Some(i) = lower.find(&pat) {
        let rest = &lower[i + pat.len()..];
        let tok = rest
            .split(|c: char| c.is_whitespace() || c == ',' || c == ';')
            .next()
            .unwrap_or("");
        return tok.parse::<f64>().ok();
    }
    let pat2 = format!("{key}:");
    if let Some(i) = lower.find(&pat2) {
        let rest = &lower[i + pat2.len()..].trim_start();
        let tok = rest
            .split(|c: char| c.is_whitespace() || c == ',' || c == ';')
            .next()
            .unwrap_or("");
        return tok.parse::<f64>().ok();
    }
    None
}

fn extract_decision(lower: &str) -> Option<&'static str> {
    if lower.contains("d-escalate")
        || lower.contains("pick=d-escalate")
        || lower.contains("please escalate")
        || lower.contains("escalate this")
        || lower.contains("needs escalation")
        || lower.contains("send to manager")
    {
        return Some("D-ESCALATE");
    }
    if lower.contains("d-deny")
        || lower.contains("pick=d-deny")
        || lower.contains("please deny")
        || lower.contains("deny this")
        || lower.contains("reject this")
        || lower.contains("do not approve")
        || lower.contains("decision: deny")
    {
        return Some("D-DENY");
    }
    if lower.contains("d-approve")
        || lower.contains("pick=d-approve")
        || lower.contains("please approve")
        || lower.contains("approve this")
        || lower.contains("go ahead and approve")
        || lower.contains("decision: approve")
        || (lower.contains("typed") && lower.contains("approve"))
    {
        return Some("D-APPROVE");
    }
    None
}

fn normalize_whitespace(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_passthrough() {
        let cfg = Phase1Config::default();
        assert!(matches!(
            run_phase1(&cfg, "please close ticket r-ok"),
            Phase1Outcome::Passthrough { .. }
        ));
    }

    #[test]
    fn enabled_ticket_ast() {
        let cfg = Phase1Config {
            enabled: true,
            transducer: "rule_ast".into(),
        };
        match run_phase1(&cfg, "Hi, please close this ticket as R-DUP thanks") {
            Phase1Outcome::Typed(ast) => {
                assert!(ast.typed_query.contains("R-DUP"));
                assert_eq!(ast.kind, "support_desk.ticket_close");
            }
            o => panic!("expected Typed, got {o:?}"),
        }
    }

    #[test]
    fn unstructured_risk_and_decision() {
        let cfg = Phase1Config {
            enabled: true,
            transducer: "rule_ast".into(),
        };
        match run_phase1(&cfg, "Can you score this as low risk for the customer?") {
            Phase1Outcome::Typed(ast) => {
                assert!(ast.typed_query.contains("RISK-LOW"), "{}", ast.typed_query);
            }
            o => panic!("expected Typed risk, got {o:?}"),
        }
        match run_phase1(&cfg, "Please approve this access request under policy.") {
            Phase1Outcome::Typed(ast) => {
                assert!(ast.typed_query.contains("D-APPROVE"), "{}", ast.typed_query);
            }
            o => panic!("expected Typed decide, got {o:?}"),
        }
        match run_phase1(
            &cfg,
            "Need a risk score compute severity=5 exposure=0.9 likelihood=0.8 from the email",
        ) {
            Phase1Outcome::Typed(ast) => {
                assert!(ast.typed_query.contains("severity=5"), "{}", ast.typed_query);
                assert_eq!(ast.kind, "risk.formula");
            }
            o => panic!("expected Typed formula, got {o:?}"),
        }
    }

    #[test]
    fn bugfix_alias_maps_to_lut_row() {
        match transduce("please close ticket r-bug — fix shipped") {
            Phase1Outcome::Typed(ast) => assert!(ast.typed_query.contains("R-BUGFIX")),
            o => panic!("{o:?}"),
        }
    }

    #[test]
    fn unrecognized_refuses_model_as_parser() {
        let o = transduce("asdf qwerty unrelated blob");
        assert!(matches!(o, Phase1Outcome::Unrecognized { .. }));
    }
}
