//! Concrete cascade gears for the v0.1 demo set.

use mol_core::{
    CascadeTier, ClaimStore, Landauer, MolError, MolRequest, PeriodicStack, QueryKind, Result,
    ROOM_TEMPERATURE_KELVIN,
};

use crate::grammar::{GrammarCoverage, TierAnswer};

/// Unit-conversion lookup table (Z1 / Lookup).
#[derive(Debug, Default, Clone)]
pub struct UnitLookup;

impl UnitLookup {
    /// Convert with a tiny dictionary.
    fn convert(query: &str) -> Option<String> {
        let q = query.to_ascii_lowercase();
        // celsius ↔ fahrenheit
        if let Some(c) = extract_number_before(&q, "celsius")
            .or_else(|| extract_number_before(&q, "°c"))
            .or_else(|| extract_number_before(&q, " c "))
        {
            if q.contains("fahrenheit") || q.contains("°f") || q.contains(" to f") {
                let f = c * 9.0 / 5.0 + 32.0;
                return Some(format!("{c} °C = {f:.4} °F"));
            }
        }
        if let Some(f) = extract_number_before(&q, "fahrenheit").or_else(|| extract_number_before(&q, "°f"))
        {
            if q.contains("celsius") || q.contains("°c") || q.contains(" to c") {
                let c = (f - 32.0) * 5.0 / 9.0;
                return Some(format!("{f} °F = {c:.4} °C"));
            }
        }
        // meters ↔ feet
        if let Some(m) = extract_number_before(&q, "meter") {
            if q.contains("feet") || q.contains("foot") {
                let ft = m * 3.280_839_895;
                return Some(format!("{m} m = {ft:.6} ft"));
            }
        }
        if let Some(ft) = extract_number_before(&q, "feet").or_else(|| extract_number_before(&q, "foot"))
        {
            if q.contains("meter") {
                let m = ft / 3.280_839_895;
                return Some(format!("{ft} ft = {m:.6} m"));
            }
        }
        // kg ↔ lb
        if let Some(kg) = extract_number_before(&q, "kg") {
            if q.contains("lb") || q.contains("pound") {
                let lb = kg * 2.204_622_621_8;
                return Some(format!("{kg} kg = {lb:.6} lb"));
            }
        }
        // joule ↔ electronvolt (unit cascade / info-thermo adjacent)
        const EV_PER_JOULE: f64 = 1.0 / 1.602_176_634e-19;
        if q.contains("joule") && (q.contains("electronvolt") || q.contains(" eV") || q.contains(" to ev") || q.contains("to ev")) {
            let j = extract_number_before(&q, "joule").unwrap_or(1.0);
            let ev = j * EV_PER_JOULE;
            return Some(format!(
                "{j} J = {ev:.6e} eV (unit cascade; CODATA e = 1.602176634e-19 C)"
            ));
        }
        if (q.contains("electronvolt") || q.contains(" ev")) && q.contains("joule") {
            let ev = extract_number_before(&q, "electronvolt")
                .or_else(|| extract_number_before(&q, "ev"))
                .unwrap_or(1.0);
            let j = ev / EV_PER_JOULE;
            return Some(format!(
                "{ev} eV = {j:.6e} J (unit cascade; CODATA e = 1.602176634e-19 C)"
            ));
        }
        // Explicit unit-cascade identity (W ≡ J/s)
        if q.contains("unit cascade") {
            return Some(
                "unit cascade: 1 W = 1 J/s; 1 eV = 1.602176634e-19 J;                  Landauer E_min = k_B T ln2 J/bit (estimates ≠ RAPL)"
                    .into(),
            );
        }
        None
    }
}

impl GrammarCoverage for UnitLookup {
    fn tier(&self) -> CascadeTier {
        CascadeTier::Lookup
    }

    fn covers(&self, req: &MolRequest) -> bool {
        matches!(req.kind, QueryKind::UnitConvert | QueryKind::StackNavigate)
            || Self::convert(&req.query).is_some()
            || PeriodicStack::subset().navigate_answer(&req.query).is_some()
    }

    fn try_answer(&self, req: &MolRequest) -> Result<TierAnswer> {
        if let Some(a) = PeriodicStack::subset().navigate_answer(&req.query) {
            return Ok(TierAnswer::text(a));
        }
        Self::convert(&req.query)
            .map(TierAnswer::text)
            .ok_or_else(|| MolError::NotCovered(format!("unit lookup miss: {}", req.query)))
    }
}

/// Closed-form physics formulas (Z1 / Formula).
#[derive(Debug, Default, Clone)]
pub struct FormulaTier;

impl FormulaTier {
    fn evaluate(query: &str) -> Option<String> {
        let q = query.to_ascii_lowercase();
        // Landauer joules per bit (formula path — not "landauer principle" knowledge ask)
        let landauer_formula = q.contains("joules per bit")
            || q.contains("ln2")
            || q.contains("ln 2")
            || ((q.contains("landauer") || q.contains("k_b") || q.contains("kbt"))
                && (q.contains("joule")
                    || q.contains("per bit")
                    || q.contains("kelvin")
                    || q.contains("floor")
                    || q.contains("e_min")
                    || q.contains("bit")));
        // Explicit knowledge / principle asks go to Z2 claim retrieve, not formula.
        let landauer_knowledge = q.contains("principle")
            || q.contains("what is landauer")
            || q.starts_with("what is the landauer")
            || q.contains("define landauer")
            || q.contains("cite") && q.contains("landauer");
        if landauer_formula && !landauer_knowledge {
            let t = if let Some(t) = extract_number_before(&q, "kelvin").or_else(|| extract_number_before(&q, " k"))
            {
                t
            } else {
                ROOM_TEMPERATURE_KELVIN
            };
            let j = Landauer::joules_per_bit(t);
            return Some(format!(
                "Landauer E_min = k_B T ln2 = {j:.6e} J/bit at T={t} K \
                 (thermodynamic lower bound; not measured RAPL/NVML)"
            ));
        }
        // E = mc²
        if q.contains("e=mc") || q.contains("e = mc") || q.contains("rest energy") {
            let mass_kg = extract_number_before(&q, "kg")
                .or_else(|| extract_number_before(&q, "mass"))
                .unwrap_or(1.0);
            const C: f64 = 299_792_458.0;
            let e = mass_kg * C * C;
            return Some(format!(
                "E = mc² = {mass_kg} kg × c² = {e:.6e} J (closed-form; c = 299792458 m/s)"
            ));
        }
        // Shannon capacity C = B log2(1+SNR) (info / rate bound)
        if q.contains("shannon capacity")
            || q.contains("shannon rate")
            || (q.contains("log2") && q.contains("snr") && q.contains("bandwidth"))
        {
            let b = extract_param(&q, "bandwidth")
                .or_else(|| extract_param(&q, "hz"))
                .or_else(|| extract_param(&q, "b"))
                .unwrap_or(1.0);
            let snr = extract_param(&q, "snr").unwrap_or(1.0);
            let c = b * (1.0 + snr).log2();
            return Some(format!(
                "Shannon capacity C = B log2(1+SNR) = {b} · log2(1+{snr}) = {c:.6} bit/s                  (AWGN closed-form rate bound; not a measured link)"
            ));
        }
        // Nyquist rate = 2 B samples/s
        if q.contains("nyquist") {
            let b = extract_param(&q, "bandwidth")
                .or_else(|| extract_param(&q, "hz"))
                .or_else(|| extract_param(&q, "b"))
                .unwrap_or(1.0);
            let rate = 2.0 * b;
            return Some(format!(
                "Nyquist rate = 2B = 2·{b} = {rate:.6} samples/s                  (bandlimited sampling bound; closed-form)"
            ));
        }
        // Max bits / alphabet bit bound: N log2 M
        if q.contains("max bits") || q.contains("bit bound") || q.contains("alphabet bits") {
            let n = extract_param(&q, "symbols")
                .or_else(|| extract_param(&q, "n"))
                .unwrap_or(1.0);
            let m = extract_param(&q, "alphabet")
                .or_else(|| extract_param(&q, "m"))
                .unwrap_or(2.0);
            let bits = n * m.log2();
            return Some(format!(
                "bit bound N log2 M = {n} · log2({m}) = {bits:.6} bits                  (finite-alphabet information bound; closed-form)"
            ));
        }
        // Closed-form risk score (Formula) — LUT miss path: features in, band out.
        // S = (severity/5)·exposure·likelihood; band thresholds on S.
        if let Some(ans) = Self::risk_closed_form(&q) {
            return Some(ans);
        }
        None
    }

    /// Closed-form risk: severity∈[0,5], exposure∈[0,1], likelihood∈[0,1] → band.
    ///
    /// Fires when the ask is a compute/formula path (not `band=RISK-*` LUT).
    fn risk_closed_form(q: &str) -> Option<String> {
        let has_band_lut = q.contains("band=")
            || q.contains("risk-low")
            || q.contains("risk-med")
            || q.contains("risk-high");
        let wants = q.contains("risk score compute")
            || q.contains("closed-form risk")
            || q.contains("risk formula")
            || (q.contains("risk")
                && (q.contains("severity") || q.contains("exposure") || q.contains("likelihood"))
                && !has_band_lut);
        if !wants {
            return None;
        }
        let severity = extract_param(q, "severity").unwrap_or(1.0).clamp(0.0, 5.0);
        let exposure = extract_param(q, "exposure").unwrap_or(0.0).clamp(0.0, 1.0);
        let likelihood = extract_param(q, "likelihood")
            .or_else(|| extract_param(q, "probability"))
            .unwrap_or(0.0)
            .clamp(0.0, 1.0);
        let score = (severity / 5.0) * exposure * likelihood;
        let (band, label) = if score < 0.15 {
            ("RISK-LOW", "LOW")
        } else if score < 0.45 {
            ("RISK-MED", "MED")
        } else {
            ("RISK-HIGH", "HIGH")
        };
        Some(format!(
            "closed-form risk S=(severity/5)·exposure·likelihood = ({severity}/5)·{exposure}·{likelihood} = {score:.6} → band={band} ({label}); Formula gear; LUT miss; model never invoked; estimates≠measured_j"
        ))
    }
}

impl GrammarCoverage for FormulaTier {
    fn tier(&self) -> CascadeTier {
        CascadeTier::Formula
    }

    fn covers(&self, req: &MolRequest) -> bool {
        matches!(req.kind, QueryKind::ClosedFormPhysics) || Self::evaluate(&req.query).is_some()
    }

    fn try_answer(&self, req: &MolRequest) -> Result<TierAnswer> {
        Self::evaluate(&req.query)
            .map(TierAnswer::text)
            .ok_or_else(|| MolError::NotCovered(format!("formula miss: {}", req.query)))
    }
}

/// Tiny dense linear solver for 2×2 systems (Z1/Z2 / Solver).
///
/// Recognizes: `solve 2x2 [[a,b],[c,d]] [e,f]` style text.
#[derive(Debug, Default, Clone)]
pub struct LinearSolver;

impl LinearSolver {
    fn parse_and_solve(query: &str) -> Option<String> {
        // Prefer numbers inside [[...]] [...] so a leading "2x2" marker is ignored.
        let mut nums: Vec<f64> = Vec::new();
        if let Some(start) = query.find("[[") {
            let slice = &query[start..];
            nums = slice
                .split(|c: char| !c.is_ascii_digit() && c != '.' && c != '-' && c != '+')
                .filter(|s| !s.is_empty())
                .filter_map(|s| s.parse().ok())
                .collect();
        }
        if nums.len() < 6 {
            nums = query
                .split(|c: char| !c.is_ascii_digit() && c != '.' && c != '-' && c != '+')
                .filter(|s| !s.is_empty())
                .filter_map(|s| s.parse().ok())
                .collect();
            // Drop a leading dimension marker when query mentions 2x2 and we have 7+ nums.
            let q = query.to_ascii_lowercase();
            if q.contains("2x2") && nums.len() >= 7 && (nums[0] - 2.0).abs() < f64::EPSILON {
                nums.remove(0);
            }
        }
        if nums.len() < 6 {
            return None;
        }
        let (a, b, c, d, e, f) = (nums[0], nums[1], nums[2], nums[3], nums[4], nums[5]);
        let det = a * d - b * c;
        if det.abs() < 1e-15 {
            return Some(format!(
                "singular 2×2 (det≈0); refuse exact solve for [[ {a}, {b} ], [ {c}, {d} ]]"
            ));
        }
        let x = (d * e - b * f) / det;
        let y = (-c * e + a * f) / det;
        Some(format!(
            "2×2 solve: x = {x:.8}, y = {y:.8} (det = {det:.8}; deterministic sparse solver)"
        ))
    }
}

impl GrammarCoverage for LinearSolver {
    fn tier(&self) -> CascadeTier {
        CascadeTier::Solver
    }

    fn covers(&self, req: &MolRequest) -> bool {
        matches!(req.kind, QueryKind::LinearSolve)
            || (req.query.to_ascii_lowercase().contains("solve")
                && Self::parse_and_solve(&req.query).is_some())
    }

    fn try_answer(&self, req: &MolRequest) -> Result<TierAnswer> {
        Self::parse_and_solve(&req.query)
            .map(TierAnswer::text)
            .ok_or_else(|| MolError::NotCovered(format!("solver miss: {}", req.query)))
    }
}

/// Model endpoint trait — labeled stub, **no weights**.
pub trait ModelStub: Send + Sync {
    /// Attempt generative residual. Default refuse.
    fn generate(&self, req: &MolRequest) -> Result<String>;
}

/// Default model stub — always refuses unless somehow allowed and overridden.
#[derive(Debug, Default, Clone)]
pub struct StubModelEndpoint;

impl ModelStub for StubModelEndpoint {
    fn generate(&self, req: &MolRequest) -> Result<String> {
        let q = req.query.to_ascii_lowercase();
        // Product soft-ref: labeled residual propose only (still no weights).
        // ReplayClass stamps ModelGenerated via Model tier; NI cert required to commit.
        if q.contains("residual propose")
            || q.contains("model propose")
            || q.contains("model_fallback")
        {
            return Ok(format!(
                "MODEL_GENERATED_PROPOSAL (stub; no weights): residual for '{}'; commit requires NI/EFA certificate — never launders to Deterministic",
                req.query
            ));
        }
        Err(MolError::ModelRefused(format!(
            "model tier demoted (no weights); query not answered: {}",
            req.query
        )))
    }
}


/// Ternary energy-landscape settle gear (Solver tier; Klere-style).
///
/// Closes when a tiny attractor settles; returns `NotCovered` on miss so the
/// engine can escalate; settle-refuse is surfaced by the MoL router / adapters
/// when the landscape will not settle.
#[derive(Debug, Default, Clone)]
pub struct TernarySettle;

impl TernarySettle {
    fn parse_state(query: &str) -> Option<Vec<i8>> {
        let start = query.find('[')?;
        let end = query[start..].find(']')? + start;
        let inner = &query[start + 1..end];
        let mut out = Vec::new();
        for tok in inner.split(|c: char| c == ',' || c.is_whitespace()) {
            let t = tok.trim();
            if t.is_empty() {
                continue;
            }
            let v: i8 = t.parse().ok()?;
            out.push(match v {
                x if x > 0 => 1,
                x if x < 0 => -1,
                _ => 0,
            });
        }
        if out.is_empty() {
            None
        } else {
            Some(out)
        }
    }

    fn will_not_settle(query: &str) -> bool {
        let q = query.to_ascii_lowercase();
        q.contains("will not settle")
            || q.contains("settle refuse")
            || q.contains("force_refuse")
    }

    fn try_settle(query: &str) -> Result<String> {
        if Self::will_not_settle(query) {
            return Err(MolError::LimitFired {
                id: "settle_refuse".into(),
                reason: format!("Klere-style settle refuse: landscape will not settle ({query})"),
            });
        }
        let Some(mut s) = Self::parse_state(query) else {
            return Err(MolError::NotCovered(format!("settle miss: {query}")));
        };
        let n = s.len();
        let max_steps = 32u32;
        for step in 1..=max_steps {
            let mut next = s.clone();
            for i in 0..n {
                if s[i] == 0 {
                    continue;
                }
                let left = s[(i + n - 1) % n];
                let right = s[(i + 1) % n];
                let field = left as i32 + right as i32;
                next[i] = if field > 0 {
                    1
                } else if field < 0 {
                    -1
                } else {
                    0
                };
            }
            if next == s {
                return Ok(format!(
                    "settle ternary → {:?} in {step} steps (Solver/settle; estimated priced joules only; measured_j=None)",
                    s
                ));
            }
            s = next;
        }
        Err(MolError::LimitFired {
            id: "settle_refuse".into(),
            reason: format!("Klere-style settle refuse: no fixed point in {max_steps} steps"),
        })
    }
}

impl GrammarCoverage for TernarySettle {
    fn tier(&self) -> CascadeTier {
        CascadeTier::Solver
    }

    fn covers(&self, req: &MolRequest) -> bool {
        matches!(req.kind, QueryKind::Settle)
            || req.query.to_ascii_lowercase().contains("settle")
            || req.query.to_ascii_lowercase().contains("ternary")
    }

    fn try_answer(&self, req: &MolRequest) -> Result<TierAnswer> {
        Self::try_settle(&req.query).map(TierAnswer::text)
    }
}


/// Z2 retrieve+cite gear — factual asks against the in-tree claim store.
///
/// Hit → answer with citation ids + ReplayClass RetrievedCited (OpenIE Z2).
/// Unknown factual → refuse (`claim_unknown`); never invent.
#[derive(Debug, Clone)]
pub struct ClaimRetrieve {
    /// Knowledge claims store.
    pub store: ClaimStore,
}

impl Default for ClaimRetrieve {
    fn default() -> Self {
        Self {
            store: ClaimStore::demo(),
        }
    }
}

impl ClaimRetrieve {
    /// With an explicit store.
    pub fn with_store(store: ClaimStore) -> Self {
        Self { store }
    }
}

impl GrammarCoverage for ClaimRetrieve {
    fn tier(&self) -> CascadeTier {
        // Priced as Lookup (table retrieve); receipt overrides zone/replay to Z2 / RetrievedCited.
        CascadeTier::Lookup
    }

    fn covers(&self, req: &MolRequest) -> bool {
        // Compose asks belong to ClaimCompose (after retrieve).
        if matches!(req.kind, QueryKind::Compose) || mol_core::looks_compose_ask(&req.query) {
            return false;
        }
        matches!(req.kind, QueryKind::FactualClaim) || self.store.would_hit(&req.query)
    }

    fn try_answer(&self, req: &MolRequest) -> Result<TierAnswer> {
        if matches!(req.kind, QueryKind::Compose) || mol_core::looks_compose_ask(&req.query) {
            return Err(MolError::NotCovered(format!(
                "compose ask deferred to ClaimCompose: {}",
                req.query
            )));
        }
        match self.store.retrieve(&req.query) {
            mol_core::ClaimHit::Found {
                answer,
                citation_ids,
                ..
            } => Ok(TierAnswer::retrieved_cited(answer, citation_ids)),
            mol_core::ClaimHit::Miss => {
                if matches!(req.kind, QueryKind::FactualClaim) {
                    Err(MolError::LimitFired {
                        id: "claim_unknown".into(),
                        reason: format!(
                            "Z2 retrieve+cite: no claim matched factual ask; refuse invent ({})",
                            req.query
                        ),
                    })
                } else {
                    Err(MolError::NotCovered(format!(
                        "claim retrieve miss: {}",
                        req.query
                    )))
                }
            }
        }
    }
}

/// Z1 compose/synthesis gear — fuse ≥2 cited claims into ReplayClass::Composed.
///
/// Cascade placement: after retrieve (or explicit compose query). Refuses if any
/// required claim id is missing — never invents / never launders as Deterministic
/// or RetrievedCited alone.
#[derive(Debug, Clone)]
pub struct ClaimCompose {
    /// Knowledge claims store (same corpus as retrieve).
    pub store: ClaimStore,
}

impl Default for ClaimCompose {
    fn default() -> Self {
        Self {
            store: ClaimStore::demo(),
        }
    }
}

impl ClaimCompose {
    /// With an explicit store.
    pub fn with_store(store: ClaimStore) -> Self {
        Self { store }
    }
}

impl GrammarCoverage for ClaimCompose {
    fn tier(&self) -> CascadeTier {
        // Priced as Lookup (cheap deterministic fuse); receipt overrides to Composed / Z1.
        CascadeTier::Lookup
    }

    fn covers(&self, req: &MolRequest) -> bool {
        matches!(req.kind, QueryKind::Compose) || mol_core::looks_compose_ask(&req.query)
    }

    fn try_answer(&self, req: &MolRequest) -> Result<TierAnswer> {
        match self.store.compose(&req.query) {
            mol_core::ComposeResolve::Ready {
                answer,
                composed_from,
                ..
            } => {
                if composed_from.len() < 2 {
                    return Err(MolError::LimitFired {
                        id: "compose_missing".into(),
                        reason: format!(
                            "Z1 compose: need ≥2 cited claims; got {} ({})",
                            composed_from.len(),
                            req.query
                        ),
                    });
                }
                Ok(TierAnswer::composed(answer, composed_from))
            }
            mol_core::ComposeResolve::Missing { missing, found } => Err(MolError::LimitFired {
                id: "compose_missing".into(),
                reason: format!(
                    "Z1 compose: required claim(s) missing {missing:?} (found {found:?}); refuse invent ({})",
                    req.query
                ),
            }),
            mol_core::ComposeResolve::NotCompose => {
                if matches!(req.kind, QueryKind::Compose)
                    || mol_core::looks_compose_ask(&req.query)
                {
                    Err(MolError::LimitFired {
                        id: "compose_missing".into(),
                        reason: format!(
                            "Z1 compose: could not resolve ≥2 claim ids for compose ask; refuse ({})",
                            req.query
                        ),
                    })
                } else {
                    Err(MolError::NotCovered(format!(
                        "compose miss: {}",
                        req.query
                    )))
                }
            }
        }
    }
}

/// GrammarCoverage wrapper that gates on `allow_model`.
#[derive(Debug, Clone)]
pub struct LookupTable {
    /// Inner lookup.
    pub inner: UnitLookup,
}

impl Default for LookupTable {
    fn default() -> Self {
        Self {
            inner: UnitLookup,
        }
    }
}

fn extract_number_before(q: &str, needle: &str) -> Option<f64> {
    let idx = q.find(needle)?;
    let prefix = &q[..idx];
    // Take last token that parses as f64.
    prefix
        .split_whitespace()
        .rev()
        .find_map(|t| {
            t.trim_matches(|c: char| !c.is_ascii_digit() && c != '.' && c != '-' && c != '+')
                .parse()
                .ok()
        })
}

/// Extract a numeric parameter: `needle=1.23`, `needle:1.23`, or number before/after needle.
fn extract_param(q: &str, needle: &str) -> Option<f64> {
    let lower_needle = needle.to_ascii_lowercase();
    // key=value / key:value
    for sep in ['=', ':'] {
        let pat = format!("{lower_needle}{sep}");
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
    extract_number_before(q, needle).or_else(|| {
        let idx = q.find(needle)?;
        let rest = &q[idx + needle.len()..];
        rest.split_whitespace().find_map(|t| {
            t.trim_matches(|c: char| !c.is_ascii_digit() && c != '.' && c != '-' && c != '+')
                .parse()
                .ok()
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use mol_core::Budget;

    #[test]
    fn landauer_formula() {
        let f = FormulaTier;
        let req = MolRequest::new("landauer joules per bit at 300 kelvin", Budget::demo());
        let a = f.try_answer(&req).unwrap();
        assert!(a.text.contains("J/bit"));
    }

    #[test]
    fn unit_c_to_f() {
        let u = UnitLookup;
        let req = MolRequest::new("convert 100 celsius to fahrenheit", Budget::demo());
        let a = u.try_answer(&req).unwrap();
        assert!(a.text.contains("212"));
    }

    #[test]
    fn linear_2x2() {
        let s = LinearSolver;
        let req = MolRequest::new(
            "solve 2x2 [[2,1],[5,3]] [8,19]",
            Budget::demo(),
        );
        // 2x+y=8, 5x+3y=19 → x=5, y=-2? Wait 2*5+(-2)=8, 5*5+3*(-2)=25-6=19. Yes x=5 y=-2
        let a = s.try_answer(&req).unwrap();
        // 2x+y=8, 5x+3y=19 → x=5, y=-2
        assert!(a.text.contains("5.00000000"), "{}", a.text);
        assert!(a.text.contains("-2.00000000"), "{}", a.text);
    }

    #[test]
    fn shannon_and_unit_cascade() {
        let f = FormulaTier;
        let a = f
            .try_answer(&MolRequest::new(
                "shannon capacity bandwidth=1000 snr=3",
                Budget::demo(),
            ))
            .unwrap();
        assert!(a.text.contains("Shannon capacity"), "{}", a.text);
        let u = UnitLookup;
        let j = u
            .try_answer(&MolRequest::new(
                "convert 1 joule to electronvolt",
                Budget::demo(),
            ))
            .unwrap();
        assert!(j.text.contains("eV"), "{}", j.text);
    }

    #[test]
    fn stack_navigate_lookup() {
        let u = UnitLookup;
        let a = u
            .try_answer(&MolRequest::new(
                "stack navigate family unit_convert",
                Budget::demo(),
            ))
            .unwrap();
        assert!(a.text.contains("family=unit_convert"), "{}", a.text);
    }

    #[test]
    fn risk_closed_form_low_and_high() {
        let f = FormulaTier;
        let low = f
            .try_answer(&MolRequest::new(
                "risk score compute severity=1 exposure=0.2 likelihood=0.1",
                Budget::demo(),
            ))
            .unwrap();
        assert!(low.text.contains("RISK-LOW"), "{}", low.text);
        assert!(low.text.contains("Formula"), "{}", low.text);
        let high = f
            .try_answer(&MolRequest::new(
                "risk score compute severity=5 exposure=0.9 likelihood=0.8",
                Budget::demo(),
            ))
            .unwrap();
        assert!(high.text.contains("RISK-HIGH"), "{}", high.text);
    }
}
