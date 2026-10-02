//! Phase-1 bounded micro-perception — unstructured → typed AST.
//!
//! Minimal rule-based transducer (not a second Model LAST). When
//! `phase1.enabled`, free text / email-ish input emits a typed task
//! coordinate for Phase-2 Mixture of Limits cascade.

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
/// Covers support-desk ticket phrases, unit convert, landauer/physics asks,
/// settle, and residual-propose markers. Does **not** open Model LAST.
pub fn transduce(raw: &str) -> Phase1Outcome {
    let s = raw.trim();
    if s.is_empty() {
        return Phase1Outcome::Unrecognized {
            raw: raw.into(),
            reason: "empty input".into(),
        };
    }
    let lower = s.to_ascii_lowercase();

    // Support-desk ticket close
    if lower.contains("ticket") && (lower.contains("close") || lower.contains("resolution")) {
        let code = extract_resolution(&lower).unwrap_or("R-HOWTO");
        return Phase1Outcome::Typed(TypedAst {
            kind: "support_desk.ticket_close".into(),
            typed_query: format!("ticket close resolution={code}"),
            query_kind: Some(QueryKind::TicketClose),
            rule: "ticket_close_resolution".into(),
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

    // Free-form email-ish → try ticket heuristics
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

fn extract_resolution(lower: &str) -> Option<&'static str> {
    for code in ["r-ok", "r-dup", "r-howto", "r-bug", "r-wontfix"] {
        if lower.contains(code) {
            return Some(match code {
                "r-ok" => "R-OK",
                "r-dup" => "R-DUP",
                "r-howto" => "R-HOWTO",
                "r-bug" => "R-BUG",
                "r-wontfix" => "R-WONTFIX",
                _ => "R-HOWTO",
            });
        }
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
    fn unrecognized_refuses_model_as_parser() {
        let o = transduce("asdf qwerty unrelated blob");
        assert!(matches!(o, Phase1Outcome::Unrecognized { .. }));
    }
}
