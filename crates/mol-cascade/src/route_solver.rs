//! Arena Solver gears: deterministic ticket routing rules + tiny SAT / knapsack.
//!
//! LUT miss path — when no `resolution=R-*` code is present, route attrs / SAT
//! assignment / tiny 0-1 knapsack close at Solver before Model LAST.

use mol_core::{CascadeTier, MolError, MolRequest, QueryKind, Result};

use crate::grammar::{GrammarCoverage, TierAnswer};

/// Deterministic ticket-routing rules + tiny SAT assign + 0-1 knapsack (Solver).
#[derive(Debug, Default, Clone)]
pub struct TicketRouteSolver;

impl TicketRouteSolver {
    fn looks_route(query: &str) -> bool {
        let q = query.to_ascii_lowercase();
        q.contains("ticket route")
            || q.contains("route rules")
            || q.contains("ticket sat")
            || q.contains("sat route")
            || q.contains("sat assign")
            || (q.contains("solve") && q.contains("knapsack"))
            || (q.contains("route") && q.contains("knapsack"))
    }

    /// Parse `key=value` boolean-ish flags from query.
    fn flag(q: &str, key: &str) -> bool {
        let q = q.to_ascii_lowercase();
        let key = key.to_ascii_lowercase();
        for sep in ['=', ':'] {
            let pat = format!("{key}{sep}");
            if let Some(i) = q.find(&pat) {
                let rest = &q[i + pat.len()..];
                let tok = rest
                    .split(|c: char| c.is_whitespace() || c == ',' || c == ';' || c == ']')
                    .next()
                    .unwrap_or("")
                    .trim();
                return matches!(tok, "true" | "1" | "yes" | "y");
            }
        }
        // Bare token presence (e.g. sat assign duplicate howto_kb)
        let is_key = format!("is_{key}");
        let has_key = format!("has_{key}");
        q.split(|c: char| !c.is_ascii_alphanumeric() && c != '_' && c != '-')
            .any(|t| t == key || t == is_key || t == has_key)
    }

    fn category(q: &str) -> Option<String> {
        let q = q.to_ascii_lowercase();
        for sep in ['=', ':'] {
            let pat = format!("category{sep}");
            if let Some(i) = q.find(&pat) {
                let rest = &q[i + pat.len()..];
                let tok = rest
                    .split(|c: char| c.is_whitespace() || c == ',' || c == ';' || c == ']')
                    .next()
                    .unwrap_or("")
                    .trim();
                if !tok.is_empty() {
                    return Some(tok.to_string());
                }
            }
        }
        None
    }

    /// Rule table (first match wins). Deterministic; no model.
    fn route_rules(query: &str) -> Option<String> {
        let q = query.to_ascii_lowercase();
        if !(q.contains("ticket route") || q.contains("route rules") || q.contains("ticket sat")
            || q.contains("sat route") || q.contains("sat assign"))
        {
            return None;
        }
        // Prefer explicit SAT assign tokens when present.
        let dup = Self::flag(&q, "duplicate") || Self::flag(&q, "dup");
        let has_kb = Self::flag(&q, "has_kb") || Self::flag(&q, "howto_kb") || q.contains("howto_kb");
        let refund = Self::flag(&q, "refund_eligible")
            || Self::flag(&q, "refund")
            || q.contains("refund_ok");
        let bug = Self::flag(&q, "bug_confirmed") || Self::flag(&q, "bug");
        let wont = Self::flag(&q, "out_of_scope") || Self::flag(&q, "wontfix");
        let cat = Self::category(&q).unwrap_or_default();

        let (code, why) = if dup {
            ("R-DUP", "rule: duplicate=true")
        } else if has_kb || cat == "howto" || cat == "kb" {
            ("R-HOWTO", "rule: category=howto|kb / has_kb")
        } else if refund || cat == "billing" || cat == "refund" {
            ("R-REFUND", "rule: refund_eligible / category=billing|refund")
        } else if bug || cat == "bug" {
            ("R-BUGFIX", "rule: bug_confirmed / category=bug")
        } else if wont || cat == "wontfix" || cat == "oos" {
            ("R-WONTFIX", "rule: out_of_scope")
        } else if cat == "ok" || cat == "resolved" || Self::flag(&q, "resolved") {
            ("R-OK", "rule: category=ok|resolved")
        } else {
            return None;
        };
        Some(format!(
            "ticket route → {code} ({why}; Solver/rules; LUT miss; model never invoked; estimates≠measured_j)"
        ))
    }

    /// Tiny 0-1 knapsack (N≤8): maximize value under capacity.
    ///
    /// `solve knapsack capacity=5 weights=[2,3,4] values=[3,4,5] labels=[A,B,C]`
    fn knapsack(query: &str) -> Option<String> {
        let q = query.to_ascii_lowercase();
        if !(q.contains("knapsack") && (q.contains("solve") || q.contains("route"))) {
            return None;
        }
        let cap = extract_f64_param(&q, "capacity").unwrap_or(0.0);
        if cap <= 0.0 {
            return None;
        }
        let weights = extract_f64_list(&q, "weights")?;
        let values = extract_f64_list(&q, "values")?;
        if weights.len() != values.len() || weights.is_empty() || weights.len() > 8 {
            return None;
        }
        let labels = extract_label_list(query, "labels").unwrap_or_else(|| {
            (0..weights.len()).map(|i| format!("item{i}")).collect()
        });
        if labels.len() != weights.len() {
            return None;
        }
        let n = weights.len();
        let mut best_val = f64::NEG_INFINITY;
        let mut best_mask = 0u32;
        for mask in 0..(1u32 << n) {
            let mut w = 0.0;
            let mut v = 0.0;
            for i in 0..n {
                if mask & (1 << i) != 0 {
                    w += weights[i];
                    v += values[i];
                }
            }
            if w <= cap + 1e-12 && v > best_val {
                best_val = v;
                best_mask = mask;
            }
        }
        if !best_val.is_finite() {
            return None;
        }
        let mut picked = Vec::new();
        let mut wsum = 0.0;
        for i in 0..n {
            if best_mask & (1 << i) != 0 {
                picked.push(labels[i].clone());
                wsum += weights[i];
            }
        }
        Some(format!(
            "0-1 knapsack: pick=[{}] value={best_val:.4} weight={wsum:.4}/{cap:.4} (Solver/LP-tiny; LUT miss; model never invoked; estimates≠measured_j)",
            picked.join(",")
        ))
    }

    fn try_solve(query: &str) -> Option<String> {
        Self::route_rules(query).or_else(|| Self::knapsack(query))
    }
}

impl GrammarCoverage for TicketRouteSolver {
    fn tier(&self) -> CascadeTier {
        CascadeTier::Solver
    }

    fn covers(&self, req: &MolRequest) -> bool {
        if !Self::looks_route(&req.query) {
            return false;
        }
        Self::try_solve(&req.query).is_some()
            || matches!(req.kind, QueryKind::TicketClose | QueryKind::LinearSolve)
    }

    fn try_answer(&self, req: &MolRequest) -> Result<TierAnswer> {
        Self::try_solve(&req.query)
            .map(TierAnswer::text)
            .ok_or_else(|| MolError::NotCovered(format!("ticket route / knapsack miss: {}", req.query)))
    }
}

fn extract_f64_param(q: &str, key: &str) -> Option<f64> {
    for sep in ['=', ':'] {
        let pat = format!("{key}{sep}");
        if let Some(i) = q.find(&pat) {
            let rest = &q[i + pat.len()..];
            let tok = rest
                .split(|c: char| c.is_whitespace() || c == ',' || c == ';')
                .next()
                .unwrap_or("");
            if let Ok(v) = tok
                .trim_matches(|c: char| !c.is_ascii_digit() && c != '.' && c != '-' && c != '+')
                .parse()
            {
                return Some(v);
            }
        }
    }
    None
}

fn extract_f64_list(q: &str, key: &str) -> Option<Vec<f64>> {
    for sep in ['=', ':'] {
        let pat = format!("{key}{sep}");
        if let Some(i) = q.find(&pat) {
            let rest = &q[i + pat.len()..];
            let start = rest.find('[')?;
            let end = rest[start..].find(']')? + start;
            let inner = &rest[start + 1..end];
            let nums: Vec<f64> = inner
                .split(|c: char| c == ',' || c.is_whitespace())
                .filter(|s| !s.is_empty())
                .filter_map(|s| {
                    s.trim_matches(|c: char| !c.is_ascii_digit() && c != '.' && c != '-' && c != '+')
                        .parse()
                        .ok()
                })
                .collect();
            if !nums.is_empty() {
                return Some(nums);
            }
        }
    }
    None
}

fn extract_label_list(query: &str, key: &str) -> Option<Vec<String>> {
    let q_lower = query.to_ascii_lowercase();
    let key = key.to_ascii_lowercase();
    for sep in ['=', ':'] {
        let pat = format!("{key}{sep}");
        if let Some(i) = q_lower.find(&pat) {
            // Use original casing slice at same byte index (ascii keys only).
            let rest = &query[i + pat.len()..];
            let start = rest.find('[')?;
            let end = rest[start..].find(']')? + start;
            let inner = &rest[start + 1..end];
            let labels: Vec<String> = inner
                .split(|c: char| c == ',' || c.is_whitespace())
                .map(|s| s.trim().trim_matches(|c: char| c == '"' || c == '\''))
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect();
            if !labels.is_empty() {
                return Some(labels);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use mol_core::Budget;

    #[test]
    fn route_howto_and_refund() {
        let s = TicketRouteSolver;
        let a = s
            .try_answer(&MolRequest::new(
                "ticket route category=howto has_kb=true priority=normal",
                Budget::demo(),
            ))
            .unwrap();
        assert!(a.text.contains("R-HOWTO"), "{}", a.text);
        let b = s
            .try_answer(&MolRequest::new(
                "ticket route category=billing refund_eligible=true",
                Budget::demo(),
            ))
            .unwrap();
        assert!(b.text.contains("R-REFUND"), "{}", b.text);
    }

    #[test]
    fn sat_assign_dup() {
        let s = TicketRouteSolver;
        let a = s
            .try_answer(&MolRequest::new(
                "ticket sat assign duplicate=true category=support",
                Budget::demo(),
            ))
            .unwrap();
        assert!(a.text.contains("R-DUP"), "{}", a.text);
    }

    #[test]
    fn knapsack_picks() {
        let s = TicketRouteSolver;
        let a = s
            .try_answer(&MolRequest::new(
                "solve knapsack capacity=4 weights=[2,2,3] values=[5,4,3] labels=[route_howto,route_ok,route_refund]",
                Budget::demo(),
            ))
            .unwrap();
        assert!(a.text.contains("route_howto") && a.text.contains("route_ok"), "{}", a.text);
        assert!(a.text.contains("knapsack"), "{}", a.text);
    }
}
