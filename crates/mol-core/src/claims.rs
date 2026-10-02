//! Knowledge claims store — typed claims with ids for OpenIE Z2 retrieve+cite.
//!
//! Clean-room: in-tree corpus only. Factual asks that match a claim answer with
//! citation ids and [`ReplayClass::RetrievedCited`]. Unknown factual asks refuse
//! (do not invent). Optional light 7-axis fields; versioned map (not full bitemporal DB).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

/// Stable claim identifier (e.g. `claim:landauer.principle`).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ClaimId(pub String);

impl ClaimId {
    /// Construct from string.
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// As str.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ClaimId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Light 7-axis annotation (optional; OpenIE-flavored, not a full axis engine).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AxisLite {
    /// Who / agent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub who: Option<String>,
    /// What / subject matter.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub what: Option<String>,
    /// When / temporal.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub when: Option<String>,
    /// Where / locus.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub where_: Option<String>,
    /// Why / rationale.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub why: Option<String>,
    /// How / mechanism.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub how: Option<String>,
    /// Joules / energy / cost axis (labeled estimate context — not RAPL).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub joules: Option<String>,
}

/// One version of a claim statement (simple versioned map).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimVersion {
    /// Monotonic version number (1..).
    pub version: u32,
    /// Statement body at this version.
    pub statement: String,
    /// When this version was recorded (system time).
    pub recorded_at: DateTime<Utc>,
    /// Optional valid-from (application time); None = unbound.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_from: Option<DateTime<Utc>>,
    /// Optional valid-to; None = still current.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_to: Option<DateTime<Utc>>,
}

impl ClaimVersion {
    /// Construct a current version with `now` as recorded_at.
    pub fn current(version: u32, statement: impl Into<String>) -> Self {
        Self {
            version,
            statement: statement.into(),
            recorded_at: Utc::now(),
            valid_from: None,
            valid_to: None,
        }
    }
}

/// Typed knowledge claim.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KnowledgeClaim {
    /// Stable id.
    pub id: ClaimId,
    /// Short subject label.
    pub subject: String,
    /// Retrieval keywords (lowercased at insert).
    pub keywords: Vec<String>,
    /// Version history (latest = last element).
    pub versions: Vec<ClaimVersion>,
    /// Optional light 7-axis fields.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub axis: Option<AxisLite>,
}

impl KnowledgeClaim {
    /// Latest statement text.
    pub fn statement(&self) -> &str {
        self.versions
            .last()
            .map(|v| v.statement.as_str())
            .unwrap_or("")
    }

    /// Latest version number.
    pub fn version(&self) -> u32 {
        self.versions.last().map(|v| v.version).unwrap_or(0)
    }

    /// Format answer line with citation id.
    pub fn cited_answer(&self) -> String {
        format!(
            "{} [cite:{} v{}]",
            self.statement(),
            self.id.as_str(),
            self.version()
        )
    }
}

/// Result of retrieving against the claim store.
#[derive(Debug, Clone, PartialEq)]
pub enum ClaimHit {
    /// One or more claims matched.
    Found {
        /// Ranked matching claims (best first).
        claims: Vec<KnowledgeClaim>,
        /// Citation ids in rank order.
        citation_ids: Vec<String>,
        /// Combined answer text.
        answer: String,
    },
    /// No claim matched.
    Miss,
}

impl ClaimHit {
    /// True on Found.
    pub fn is_found(&self) -> bool {
        matches!(self, Self::Found { .. })
    }
}

/// In-memory versioned claim map (clean-room store).
#[derive(Debug, Clone, Default)]
pub struct ClaimStore {
    by_id: BTreeMap<ClaimId, KnowledgeClaim>,
}

impl ClaimStore {
    /// Empty store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Seeded demo corpus (Landauer, MoL law, stack facts, honesty).
    pub fn demo() -> Self {
        let mut s = Self::new();
        for c in seed_demo_claims() {
            s.upsert(c);
        }
        s
    }

    /// Number of claims.
    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    /// Empty?
    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }

    /// Insert or replace by id.
    pub fn upsert(&mut self, claim: KnowledgeClaim) {
        self.by_id.insert(claim.id.clone(), claim);
    }

    /// Get by id.
    pub fn get(&self, id: &str) -> Option<&KnowledgeClaim> {
        self.by_id.get(&ClaimId::new(id))
    }

    /// All claims (sorted by id).
    pub fn iter(&self) -> impl Iterator<Item = &KnowledgeClaim> {
        self.by_id.values()
    }

    /// Retrieve: score claims by keyword / subject / id overlap with query.
    ///
    /// Returns [`ClaimHit::Found`] when best score ≥ threshold; else Miss.
    pub fn retrieve(&self, query: &str) -> ClaimHit {
        let q = query.to_ascii_lowercase();
        let tokens: Vec<&str> = q
            .split(|c: char| !c.is_ascii_alphanumeric() && c != '_' && c != '.')
            .filter(|t| t.len() >= 2)
            .collect();

        let mut scored: Vec<(i32, &KnowledgeClaim)> = Vec::new();
        for claim in self.by_id.values() {
            let mut score = 0i32;
            let id_l = claim.id.as_str().to_ascii_lowercase();
            let sub_l = claim.subject.to_ascii_lowercase();
            if q.contains(&id_l) || q.contains(claim.id.as_str()) {
                score += 10;
            }
            // bare id tail after claim:
            if let Some(tail) = id_l.strip_prefix("claim:") {
                if q.contains(tail) {
                    score += 6;
                }
            }
            if !sub_l.is_empty() && q.contains(&sub_l) {
                score += 5;
            }
            for kw in &claim.keywords {
                let k = kw.to_ascii_lowercase();
                if k.is_empty() {
                    continue;
                }
                if q.contains(&k) {
                    score += 3;
                }
                for t in &tokens {
                    if k == *t || k.contains(t) || t.contains(&k) {
                        score += 1;
                    }
                }
            }
            // Multi-word phrase bonuses for demo corpus
            if q.contains("landauer")
                && (q.contains("principle") || q.contains("bound") || q.contains("limit"))
                && claim.keywords.iter().any(|k| k.contains("landauer"))
            {
                score += 4;
            }
            if (q.contains("mol law")
                || q.contains("mixture of limits")
                || q.contains("universal law"))
                && claim.keywords.iter().any(|k| k.contains("mol") || k.contains("law"))
            {
                score += 4;
            }
            if score > 0 {
                scored.push((score, claim));
            }
        }
        scored.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.id.cmp(&b.1.id)));

        const THRESHOLD: i32 = 4;
        let hits: Vec<&KnowledgeClaim> = scored
            .into_iter()
            .filter(|(s, _)| *s >= THRESHOLD)
            .map(|(_, c)| c)
            .collect();

        if hits.is_empty() {
            return ClaimHit::Miss;
        }

        // Cap at 1 for RetrievedCited — multi-claim fusion is Compose (ReplayClass::Composed).
        let top: Vec<KnowledgeClaim> = hits.into_iter().take(1).cloned().collect();
        let citation_ids: Vec<String> = top.iter().map(|c| c.id.as_str().to_string()).collect();
        let answer = if top.len() == 1 {
            top[0].cited_answer()
        } else {
            top.iter()
                .map(|c| c.cited_answer())
                .collect::<Vec<_>>()
                .join(" | ")
        };
        ClaimHit::Found {
            claims: top,
            citation_ids,
            answer,
        }
    }

    /// True if retrieve would hit (used by cascade covers).
    pub fn would_hit(&self, query: &str) -> bool {
        self.retrieve(query).is_found()
    }
}

/// Demo seed corpus — Landauer, MoL law one-liners, stack / honesty facts.
pub fn seed_demo_claims() -> Vec<KnowledgeClaim> {
    let now_versions = |statement: &str| vec![ClaimVersion::current(1, statement)];

    vec![
        KnowledgeClaim {
            id: ClaimId::new("claim:landauer.principle"),
            subject: "Landauer principle".into(),
            keywords: vec![
                "landauer".into(),
                "principle".into(),
                "bound".into(),
                "k_b".into(),
                "ln2".into(),
                "bit erased".into(),
                "thermodynamic".into(),
            ],
            versions: now_versions(
                "Landauer principle: erasing one bit costs at least k_B T ln 2 joules \
                 (thermodynamic lower bound; estimate ≠ RAPL/NVML).",
            ),
            axis: Some(AxisLite {
                who: Some("Landauer 1961".into()),
                what: Some("bit erasure energy floor".into()),
                when: Some("1961+".into()),
                where_: Some("thermodynamics / info".into()),
                why: Some("irreversible erase".into()),
                how: Some("E_min = k_B T ln2".into()),
                joules: Some("k_B T ln2 per bit (estimate)".into()),
            }),
        },
        KnowledgeClaim {
            id: ClaimId::new("claim:mol.law"),
            subject: "Mixture of Limits law".into(),
            keywords: vec![
                "mol".into(),
                "mixture of limits".into(),
                "mol law".into(),
                "universal law".into(),
                "floors".into(),
                "more bits".into(),
            ],
            versions: now_versions(
                "MoL law: Mixture of Limits is the universal law of computer intelligence — \
                 named floors where more bits stop buying outcomes; navigate, don't drown in tokens.",
            ),
            axis: Some(AxisLite {
                who: Some("OpenIE / David Charlot".into()),
                what: Some("CI navigation law".into()),
                why: Some("floors bind outcomes".into()),
                how: Some("cascade cheapest-sufficient".into()),
                ..Default::default()
            }),
        },
        KnowledgeClaim {
            id: ClaimId::new("claim:mol.proof_law"),
            subject: "MoL proof law E≥θμ".into(),
            keywords: vec![
                "proof law".into(),
                "theta".into(),
                "mu".into(),
                "impedance".into(),
                "e(x)".into(),
                "θ".into(),
                "μ".into(),
            ],
            versions: now_versions(
                "MoL proof law: E(x) ≥ θ(D)·μ(S,V) with θ = bits×k_B T ln2 and catalog μ per tier \
                 (mu_source=catalog; not fake RAPL).",
            ),
            axis: Some(AxisLite {
                what: Some("energy-impedance inequality".into()),
                joules: Some("E ≈ θ·μ catalog surrogate".into()),
                how: Some("MuCatalog tier table".into()),
                ..Default::default()
            }),
        },
        KnowledgeClaim {
            id: ClaimId::new("claim:mol.cascade"),
            subject: "MoL cascade order".into(),
            keywords: vec![
                "cascade".into(),
                "lookup".into(),
                "formula".into(),
                "solver".into(),
                "model last".into(),
                "mathground".into(),
            ],
            versions: now_versions(
                "MoL / MathGround cascade: Lookup → Formula → Solver/settle → Model LAST \
                 (model demoted; refuse over invent).",
            ),
            axis: None,
        },
        KnowledgeClaim {
            id: ClaimId::new("claim:stack.scale"),
            subject: "Periodic Stack scale".into(),
            keywords: vec![
                "periodic stack".into(),
                "258".into(),
                "33 families".into(),
                "primitives".into(),
                "stack scale".into(),
                "stack size".into(),
            ],
            versions: now_versions(
                "Periodic Stack target scale: 258 primitives across 33 families; \
                 clean-room ships a Present/Gap subset navigator (not the full live catalog).",
            ),
            axis: Some(AxisLite {
                what: Some("stack cardinality".into()),
                how: Some("subset navigator + Gap probes".into()),
                ..Default::default()
            }),
        },
        KnowledgeClaim {
            id: ClaimId::new("claim:honesty.no_fake_rapl"),
            subject: "Receipt honesty".into(),
            keywords: vec![
                "rapl".into(),
                "measured_j".into(),
                "honesty".into(),
                "board_synth".into(),
                "nvml".into(),
            ],
            versions: now_versions(
                "Honesty invariant: software-ref receipts keep measured_j=None and \
                 board_synth_claimed=false; Landauer/catalog estimates are labeled, never sold as RAPL.",
            ),
            axis: Some(AxisLite {
                joules: Some("estimated only; measured_j=None".into()),
                why: Some("no fake silicon meters".into()),
                ..Default::default()
            }),
        },
        KnowledgeClaim {
            id: ClaimId::new("claim:replay.retrieved_cited"),
            subject: "RetrievedCited replay class".into(),
            keywords: vec![
                "retrieved cited".into(),
                "retrieved_cited".into(),
                "replay class".into(),
                "citation".into(),
                "cite".into(),
                "z2".into(),
            ],
            versions: now_versions(
                "ReplayClass RetrievedCited: answer from authoritative retrieve+cite \
                 (OpenIE Z2); cannot strengthen to Deterministic from ModelGenerated.",
            ),
            axis: None,
        },
        KnowledgeClaim {
            id: ClaimId::new("claim:openie.z2"),
            subject: "OpenIE Z2 zone".into(),
            keywords: vec![
                "openie z2".into(),
                "z2 zone".into(),
                "bounded inference".into(),
                "citation zone".into(),
            ],
            versions: now_versions(
                "OpenIE Z2: math ≈ words — bounded inference / citation; MoL maps retrieve+cite \
                 here with ReplayClass RetrievedCited (Z1=Lookup/Formula, Z3=Model).",
            ),
            axis: None,
        },
    ]
}

/// Demo compose recipe: ≥2 claim ids fused into one Composed answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComposeRecipe {
    /// Short demo label.
    pub label: &'static str,
    /// Phrases that select this recipe (lowercase substring match).
    pub cues: &'static [&'static str],
    /// Required claim ids (≥2).
    pub claim_ids: &'static [&'static str],
}

/// Seeded compose demos (Z1 compose from cited claims).
pub fn seed_compose_recipes() -> Vec<ComposeRecipe> {
    vec![
        ComposeRecipe {
            label: "landauer+mol.law",
            cues: &[
                "compose landauer with mol",
                "compose landauer principle with mol",
                "compose landauer and mol",
                "landauer principle with mol law",
                "landauer and mol law",
                "compose: landauer + mol",
            ],
            claim_ids: &["claim:landauer.principle", "claim:mol.law"],
        },
        ComposeRecipe {
            label: "honesty+proof_law",
            cues: &[
                "compose honesty with proof",
                "synthesize honesty and proof",
                "compose receipt honesty with proof law",
                "honesty and proof law",
                "compose: honesty + proof",
            ],
            claim_ids: &["claim:honesty.no_fake_rapl", "claim:mol.proof_law"],
        },
        ComposeRecipe {
            label: "cascade+z2",
            cues: &[
                "compose cascade with z2",
                "compose cascade and openie",
                "synthesize cascade and z2",
                "compose: cascade + z2",
            ],
            claim_ids: &["claim:mol.cascade", "claim:openie.z2"],
        },
    ]
}

/// Heuristic: query is an explicit compose / synthesis ask (Z1 from ≥2 cited claims).
pub fn looks_compose_ask(query: &str) -> bool {
    let q = query.to_ascii_lowercase();
    let trimmed = q.trim();
    if trimmed.starts_with("compose ")
        || trimmed.starts_with("compose:")
        || trimmed.starts_with("compose+")
        || trimmed.starts_with("synthesize ")
        || trimmed.starts_with("synthesize:")
        || trimmed.starts_with("synthesis ")
        || trimmed.starts_with("compose from")
        || trimmed.starts_with("combine claims")
    {
        return true;
    }
    if q.contains(" compose ") && (q.contains("claim:") || q.contains(" with ") || q.contains(" and "))
    {
        return true;
    }
    // Explicit multi-claim id listing
    let claim_mentions = q.matches("claim:").count();
    if claim_mentions >= 2 && (q.contains("compose") || q.contains("synthes") || q.contains(" + ")) {
        return true;
    }
    seed_compose_recipes()
        .iter()
        .any(|r| r.cues.iter().any(|c| q.contains(c)))
}

/// Parse required claim ids for a compose ask.
///
/// Prefers explicit `claim:…` tokens; else matches a seeded recipe; else Miss-style empty.
pub fn parse_compose_requirements(query: &str) -> Vec<String> {
    let q = query.to_ascii_lowercase();
    // Explicit claim:id tokens (preserve canonical form from query when possible).
    let mut ids: Vec<String> = Vec::new();
    let raw = query;
    let lower = raw.to_ascii_lowercase();
    let mut search_from = 0usize;
    while let Some(rel) = lower[search_from..].find("claim:") {
        let start = search_from + rel;
        let rest = &raw[start..];
        let end = rest
            .find(|c: char| c.is_whitespace() || c == ',' || c == ';' || c == ')' || c == ']')
            .unwrap_or(rest.len());
        let id = rest[..end]
            .trim_end_matches(|c: char| !c.is_ascii_alphanumeric() && c != ':' && c != '.' && c != '_' && c != '-')
            .to_string();
        if id.starts_with("claim:") && !ids.iter().any(|x| x.eq_ignore_ascii_case(&id)) {
            ids.push(id);
        }
        search_from = start + end.max(1);
    }
    if ids.len() >= 2 {
        return ids;
    }
    // Seeded recipes
    for recipe in seed_compose_recipes() {
        if recipe.cues.iter().any(|c| q.contains(c)) {
            return recipe
                .claim_ids
                .iter()
                .map(|s| (*s).to_string())
                .collect();
        }
    }
    ids
}

/// Result of resolving compose requirements against the store.
#[derive(Debug, Clone, PartialEq)]
pub enum ComposeResolve {
    /// ≥2 claims ready to fuse.
    Ready {
        /// Resolved claims in request order.
        claims: Vec<KnowledgeClaim>,
        /// Source claim ids.
        composed_from: Vec<String>,
        /// Composed answer text (cites all ids).
        answer: String,
    },
    /// At least one required claim id is absent — refuse (do not invent).
    Missing {
        /// Ids that were required but not in the store.
        missing: Vec<String>,
        /// Ids that resolved.
        found: Vec<String>,
    },
    /// Query did not name ≥2 requirements.
    NotCompose,
}

impl ClaimStore {
    /// Resolve a compose/synthesis ask against the store.
    ///
    /// Requires ≥2 claim ids; refuses if any required id is missing.
    pub fn compose(&self, query: &str) -> ComposeResolve {
        let req_ids = parse_compose_requirements(query);
        if req_ids.len() < 2 {
            return ComposeResolve::NotCompose;
        }
        let mut claims = Vec::new();
        let mut found = Vec::new();
        let mut missing = Vec::new();
        for id in &req_ids {
            match self.get(id) {
                Some(c) => {
                    found.push(id.clone());
                    claims.push(c.clone());
                }
                None => missing.push(id.clone()),
            }
        }
        if !missing.is_empty() {
            return ComposeResolve::Missing { missing, found };
        }
        if claims.len() < 2 {
            return ComposeResolve::NotCompose;
        }
        let composed_from: Vec<String> = claims.iter().map(|c| c.id.as_str().to_string()).collect();
        let parts: Vec<String> = claims
            .iter()
            .map(|c| {
                format!(
                    "({}) {}",
                    c.id.as_str(),
                    c.statement()
                )
            })
            .collect();
        let cites = composed_from
            .iter()
            .map(|id| format!("cite:{id}"))
            .collect::<Vec<_>>()
            .join(", ");
        let answer = format!(
            "Composed (Z1 synthesis from {} cited claims): {} ⇒ fused under ReplayClass::Composed (not RetrievedCited/Deterministic alone). Sources: [{}]",
            claims.len(),
            parts.join(" ⊕ "),
            cites
        );
        ComposeResolve::Ready {
            claims,
            composed_from,
            answer,
        }
    }
}

/// Heuristic: query looks like a factual knowledge ask (Z2 retrieve), not free-form generation.
pub fn looks_factual_ask(query: &str) -> bool {
    let q = query.to_ascii_lowercase();
    let trimmed = q.trim();
    if trimmed.starts_with("cite ")
        || trimmed.starts_with("cite:")
        || trimmed.starts_with("claim:")
        || trimmed.starts_with("claim ")
        || trimmed.starts_with("knowledge:")
        || trimmed.starts_with("what is ")
        || trimmed.starts_with("what's ")
        || trimmed.starts_with("whats ")
        || trimmed.starts_with("define ")
        || trimmed.starts_with("explain the mol")
        || trimmed.starts_with("explain mol")
    {
        return true;
    }
    // Phrase cues that are knowledge asks (formula path still wins when classified earlier).
    let cues = [
        "mol law",
        "mixture of limits law",
        "landauer principle",
        "what is landauer",
        "periodic stack size",
        "periodic stack scale",
        "how many primitives",
        "how many families",
        "retrieved cited",
        "replay class",
        "openie z2",
        "proof law",
        "no fake rapl",
        "measured_j",
        "cascade order",
    ];
    cues.iter().any(|c| q.contains(c))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_store_hits_landauer_principle() {
        let s = ClaimStore::demo();
        assert!(s.len() >= 6);
        let hit = s.retrieve("what is the landauer principle");
        assert!(hit.is_found(), "{hit:?}");
        if let ClaimHit::Found {
            citation_ids,
            answer,
            ..
        } = hit
        {
            assert!(
                citation_ids.iter().any(|id| id.contains("landauer")),
                "{citation_ids:?}"
            );
            assert!(answer.contains("cite:"), "{answer}");
        }
    }

    #[test]
    fn unknown_factual_misses() {
        let s = ClaimStore::demo();
        assert!(!s.retrieve("what is the capital of Atlantis xyzzy").is_found());
    }

    #[test]
    fn mol_law_hit() {
        let s = ClaimStore::demo();
        let hit = s.retrieve("cite the mol law");
        assert!(hit.is_found());
    }

    #[test]
    fn compose_two_claims() {
        let s = ClaimStore::demo();
        let r = s.compose("compose claim:landauer.principle and claim:mol.law");
        match r {
            ComposeResolve::Ready {
                composed_from,
                answer,
                ..
            } => {
                assert_eq!(composed_from.len(), 2);
                assert!(answer.contains("Composed"));
                assert!(answer.contains("cite:claim:landauer.principle"));
                assert!(answer.contains("cite:claim:mol.law"));
            }
            other => panic!("expected Ready, got {other:?}"),
        }
    }

    #[test]
    fn compose_missing_refuses() {
        let s = ClaimStore::demo();
        let r = s.compose("compose claim:landauer.principle and claim:does.not.exist");
        match r {
            ComposeResolve::Missing { missing, .. } => {
                assert!(missing.iter().any(|m| m.contains("does.not.exist")));
            }
            other => panic!("expected Missing, got {other:?}"),
        }
    }

    #[test]
    fn looks_compose_seed_recipe() {
        assert!(looks_compose_ask("compose landauer principle with mol law"));
        assert!(!looks_compose_ask("what is the landauer principle"));
    }

    #[test]
    fn retrieve_caps_at_one() {
        let s = ClaimStore::demo();
        if let ClaimHit::Found { citation_ids, .. } = s.retrieve("mol") {
            assert!(citation_ids.len() <= 1, "{citation_ids:?}");
        }
    }
}
