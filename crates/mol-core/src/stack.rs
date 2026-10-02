//! Periodic Stack subset navigator (clean-room, in-tree).
//!
//! Thesis table (compute.openie.dev): **258 primitives / 33 families**.
//! This module ships a **compact subset**: all 33 family names + representative
//! primitives (Present cells MoL can name) and explicit **Gap** markers for
//! empty cells (e.g. `physical_settle`). It does **not** claim full 258 coverage
//! and does not invent silicon/RAPL measurements — OOM joule fields are catalog
//! surrogates only (`estimated`, never `measured_j`).
//!
//! Inspired by OpenIE compute-stack themes; re-encoded clean-room (no path-dep).

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::tier::ThermoClass;

/// Full thesis primitive count (compute.openie.dev).
pub const FULL_TARGET_PRIMITIVES: usize = 258;
/// Full thesis family count.
pub const FULL_TARGET_FAMILIES: usize = 33;

/// Algebraic / operational family (33 thesis families).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u16)]
pub enum StackFamily {
    /// Arithmetic / numeric closed-form.
    Arithmetic = 0,
    /// Parsing / lexing / deterministic transforms.
    ParseTransform = 1,
    /// Boolean / bit ops.
    Logic = 2,
    /// In-memory lookup / dict / table.
    Lookup = 3,
    /// Knowledge retrieval / citation.
    Retrieval = 4,
    /// Constraint check / schema validate.
    Constraint = 5,
    /// Hashing / checksum.
    HashDigest = 6,
    /// Encoding / serialization.
    Encode = 7,
    /// Scheduling / control flow (deterministic).
    Control = 8,
    /// Statistical sampling / generative (Z3 residual).
    Generative = 9,
    /// Hyperdimensional / binding (HDC).
    Hdc = 10,
    /// Cryptographic primitives.
    Crypto = 11,
    /// I/O boundary.
    Io = 12,
    /// Graph algorithms.
    Graph = 13,
    /// Dense / sparse linear algebra.
    LinearAlgebra = 14,
    /// Geometry / computational geometry.
    Geometry = 15,
    /// Descriptive / inferential statistics.
    Statistics = 16,
    /// Signal processing.
    Signal = 17,
    /// Compression / decompression.
    Compression = 18,
    /// Sorting / searching.
    SortSearch = 19,
    /// Tree / trie / hierarchy.
    Tree = 20,
    /// Finite-state machines / automata.
    FiniteState = 21,
    /// Type / schema checking (static).
    TypeCheck = 22,
    /// Diff / patch / edit scripts.
    DiffPatch = 23,
    /// Temporal / interval arithmetic.
    Temporal = 24,
    /// Unit conversion.
    UnitConvert = 25,
    /// String morphology.
    StringMorph = 26,
    /// Set algebra.
    SetOps = 27,
    /// Probabilistic inference (bounded).
    Probabilistic = 28,
    /// Optimization / search over objectives.
    Optimization = 29,
    /// Network / HTTP boundary.
    Network = 30,
    /// Filesystem boundary (sandboxed).
    FileSystem = 31,
    /// Computer automation acts (gated).
    Automation = 32,
}

impl StackFamily {
    /// Thesis family count.
    pub const COUNT: usize = FULL_TARGET_FAMILIES;

    /// All families in id order.
    pub const ALL: [StackFamily; 33] = [
        Self::Arithmetic,
        Self::ParseTransform,
        Self::Logic,
        Self::Lookup,
        Self::Retrieval,
        Self::Constraint,
        Self::HashDigest,
        Self::Encode,
        Self::Control,
        Self::Generative,
        Self::Hdc,
        Self::Crypto,
        Self::Io,
        Self::Graph,
        Self::LinearAlgebra,
        Self::Geometry,
        Self::Statistics,
        Self::Signal,
        Self::Compression,
        Self::SortSearch,
        Self::Tree,
        Self::FiniteState,
        Self::TypeCheck,
        Self::DiffPatch,
        Self::Temporal,
        Self::UnitConvert,
        Self::StringMorph,
        Self::SetOps,
        Self::Probabilistic,
        Self::Optimization,
        Self::Network,
        Self::FileSystem,
        Self::Automation,
    ];

    /// Snake name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Arithmetic => "arithmetic",
            Self::ParseTransform => "parse_transform",
            Self::Logic => "logic",
            Self::Lookup => "lookup",
            Self::Retrieval => "retrieval",
            Self::Constraint => "constraint",
            Self::HashDigest => "hash_digest",
            Self::Encode => "encode",
            Self::Control => "control",
            Self::Generative => "generative",
            Self::Hdc => "hdc",
            Self::Crypto => "crypto",
            Self::Io => "io",
            Self::Graph => "graph",
            Self::LinearAlgebra => "linear_algebra",
            Self::Geometry => "geometry",
            Self::Statistics => "statistics",
            Self::Signal => "signal",
            Self::Compression => "compression",
            Self::SortSearch => "sort_search",
            Self::Tree => "tree",
            Self::FiniteState => "finite_state",
            Self::TypeCheck => "type_check",
            Self::DiffPatch => "diff_patch",
            Self::Temporal => "temporal",
            Self::UnitConvert => "unit_convert",
            Self::StringMorph => "string_morph",
            Self::SetOps => "set_ops",
            Self::Probabilistic => "probabilistic",
            Self::Optimization => "optimization",
            Self::Network => "network",
            Self::FileSystem => "filesystem",
            Self::Automation => "automation",
        }
    }

    /// Stable numeric id.
    pub const fn id(self) -> u16 {
        self as u16
    }

    /// Parse snake / display name.
    pub fn parse(s: &str) -> Option<Self> {
        let n = s.trim().to_ascii_lowercase().replace('-', "_");
        Self::ALL.into_iter().find(|f| f.name() == n)
    }
}

impl fmt::Display for StackFamily {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// Whether a named cell is filled in this subset or an honest empty marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CellStatus {
    /// Named and present in the subset (navigable; may map to MoL gears).
    Present,
    /// Named empty cell — fire `primitive_gap`; do not hallucinate coverage.
    Gap,
}

impl CellStatus {
    /// Label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Present => "present",
            Self::Gap => "gap",
        }
    }
}

/// One cell in the in-tree Periodic Stack subset.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct StackPrimitive {
    /// Stable id (subset namespace; not a claim of full 258 indexing).
    pub id: u16,
    /// Snake name.
    pub name: &'static str,
    /// Family.
    pub family: StackFamily,
    /// Thermodynamic class (catalog label, not RAPL).
    pub thermo: ThermoClass,
    /// Present vs Gap marker.
    pub status: CellStatus,
    /// Order-of-magnitude estimated joules (catalog surrogate; ≠ measured).
    pub estimated_j_oom: f64,
    /// One-line description.
    pub description: &'static str,
}

/// Result of probing the subset for a named cell / query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProbeResult {
    /// Named Present cell.
    Present {
        /// Primitive name.
        name: String,
        /// Family snake name.
        family: String,
    },
    /// Named Gap cell (or unknown name under gap intent).
    Gap {
        /// Requested name.
        name: String,
        /// Optional family if the gap is registered.
        family: Option<String>,
        /// Why this is a gap.
        reason: String,
    },
    /// Query is not a stack probe / navigate.
    NotAProbe,
}

impl ProbeResult {
    /// True when this should bind `primitive_gap`.
    pub fn is_gap(&self) -> bool {
        matches!(self, Self::Gap { .. })
    }
}

/// Compact Periodic Stack subset navigator.
#[derive(Debug, Clone)]
pub struct PeriodicStack {
    cells: &'static [StackPrimitive],
}

impl Default for PeriodicStack {
    fn default() -> Self {
        Self::subset()
    }
}

impl PeriodicStack {
    /// Built-in compact subset (33 families + representative Present/Gap cells).
    pub fn subset() -> Self {
        Self { cells: SUBSET_CELLS }
    }

    /// Scale note vs full thesis table.
    pub fn scale_note(&self) -> String {
        format!(
            "Periodic Stack subset navigator: {} families (all {}), \
             {} seeded cells ({} present, {} gap markers). \
             Full thesis table: {} primitives / {} families (compute.openie.dev). \
             Subset does not claim full 258 coverage; Gap cells refuse via primitive_gap.",
            FULL_TARGET_FAMILIES,
            FULL_TARGET_FAMILIES,
            self.seeded_count(),
            self.present_count(),
            self.gap_count(),
            FULL_TARGET_PRIMITIVES,
            FULL_TARGET_FAMILIES,
        )
    }

    /// Seeded cell count (Present + Gap markers).
    pub fn seeded_count(&self) -> usize {
        self.cells.len()
    }

    /// Present cells only.
    pub fn present_count(&self) -> usize {
        self.cells
            .iter()
            .filter(|c| c.status == CellStatus::Present)
            .count()
    }

    /// Gap markers only.
    pub fn gap_count(&self) -> usize {
        self.cells
            .iter()
            .filter(|c| c.status == CellStatus::Gap)
            .count()
    }

    /// How many Present slots remain toward the 258 thesis target.
    pub fn remaining_to_full(&self) -> usize {
        FULL_TARGET_PRIMITIVES.saturating_sub(self.present_count())
    }

    /// Iterate all seeded cells.
    pub fn iter(&self) -> impl Iterator<Item = &StackPrimitive> {
        self.cells.iter()
    }

    /// Lookup by exact snake name.
    pub fn get_by_name(&self, name: &str) -> Option<&StackPrimitive> {
        let n = name.trim().to_ascii_lowercase();
        self.cells.iter().find(|c| c.name == n)
    }

    /// Lookup by id.
    pub fn get_by_id(&self, id: u16) -> Option<&StackPrimitive> {
        self.cells.iter().find(|c| c.id == id)
    }

    /// Cells in a family.
    pub fn by_family(&self, family: StackFamily) -> impl Iterator<Item = &StackPrimitive> {
        self.cells.iter().filter(move |c| c.family == family)
    }

    /// Probe a primitive name against the subset registry (real cell status).
    pub fn probe_name(&self, name: &str) -> ProbeResult {
        let n = name.trim().to_ascii_lowercase().replace('-', "_");
        if n.is_empty() {
            return ProbeResult::NotAProbe;
        }
        match self.get_by_name(&n) {
            Some(c) if c.status == CellStatus::Present => ProbeResult::Present {
                name: c.name.to_string(),
                family: c.family.name().to_string(),
            },
            Some(c) => ProbeResult::Gap {
                name: c.name.to_string(),
                family: Some(c.family.name().to_string()),
                reason: format!(
                    "Periodic Stack gap marker '{}' (family {}); empty cell — do not hallucinate coverage",
                    c.name,
                    c.family.name()
                ),
            },
            None => ProbeResult::Gap {
                name: n,
                family: None,
                reason: format!(
                    "primitive '{name}' absent from in-tree subset ({} present / {} target); missing cell",
                    self.present_count(),
                    FULL_TARGET_PRIMITIVES
                ),
            },
        }
    }

    /// Extract a primitive token from query text, if any.
    pub fn extract_primitive_token(query: &str) -> Option<String> {
        let lower = query.to_ascii_lowercase();

        // Prefer longest prefixes first.
        let prefixes = [
            "periodic stack primitive:",
            "missing primitive:",
            "probe primitive:",
            "navigate primitive:",
            "stack primitive:",
            "primitive_gap:",
            "primitive gap:",
            "primitive:",
        ];
        for prefix in prefixes {
            if let Some(i) = lower.find(prefix) {
                let rest = lower[i + prefix.len()..].trim();
                if let Some(tok) = rest
                    .split(|c: char| c.is_whitespace() || c == ',' || c == ';' || c == '/')
                    .find(|t| {
                        !t.is_empty()
                            && *t != "on"
                            && *t != "not"
                            && *t != "the"
                            && *t != "missing"
                    })
                {
                    let cleaned = tok
                        .trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                        .to_string();
                    if !cleaned.is_empty() {
                        return Some(cleaned);
                    }
                }
            }
        }

        for key in ["probe primitive ", "navigate primitive ", "stack primitive "] {
            if let Some(i) = lower.find(key) {
                let rest = &lower[i + key.len()..];
                if let Some(tok) = rest.split_whitespace().next() {
                    let cleaned = tok
                        .trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                        .to_string();
                    if !cleaned.is_empty() {
                        return Some(cleaned);
                    }
                }
            }
        }

        // Bare known cell name when stack/gap/probe vocabulary present.
        if gap_or_stack_vocab(&lower) {
            for c in SUBSET_CELLS {
                if lower.contains(c.name) {
                    return Some(c.name.to_string());
                }
            }
        }
        None
    }

    /// True when query is a stack navigate (not a gap refuse).
    pub fn is_navigate_query(query: &str) -> bool {
        let q = query.to_ascii_lowercase();
        (q.contains("stack navigate")
            || q.contains("navigate family")
            || q.contains("navigate primitive")
            || q.contains("periodic stack family")
            || q.contains("periodic stack primitive")
            || q.contains("stack family")
            || q.contains("stack scale")
            || q.contains("periodic stack scale"))
            && !q.contains("gap")
            && !q.contains("missing primitive")
    }

    /// Probe a full query: extract token + registry status, or explicit gap intent.
    pub fn probe_query(&self, query: &str) -> ProbeResult {
        let lower = query.to_ascii_lowercase();

        if Self::is_navigate_query(query) {
            // Navigate path — not a gap bind (Lookup answers).
            if let Some(tok) = Self::extract_primitive_token(query) {
                return match self.probe_name(&tok) {
                    ProbeResult::Gap { name, family, reason } => ProbeResult::Gap {
                        name,
                        family,
                        reason,
                    },
                    other => other,
                };
            }
            return ProbeResult::NotAProbe;
        }

        if let Some(tok) = Self::extract_primitive_token(query) {
            let r = self.probe_name(&tok);
            // Present cells are not gap refuses unless caller insisted "gap".
            if matches!(r, ProbeResult::Present { .. }) && !lower.contains("gap") {
                return ProbeResult::NotAProbe;
            }
            if matches!(r, ProbeResult::Present { .. }) && lower.contains("gap") {
                // Asking "gap" about a Present cell — not missing.
                return ProbeResult::NotAProbe;
            }
            return r;
        }

        // Explicit gap vocabulary without a resolvable Present name → gap probe.
        if lower.contains("primitive gap")
            || lower.contains("primitive_gap")
            || lower.contains("periodic stack gap")
            || lower.contains("missing primitive")
        {
            return ProbeResult::Gap {
                name: "(unnamed)".into(),
                family: None,
                reason: "explicit primitive_gap probe; no Present cell named — refuse hallucinated coverage"
                    .into(),
            };
        }

        ProbeResult::NotAProbe
    }

    /// Format a navigate answer for Lookup gear (family / primitive / scale).
    pub fn navigate_answer(&self, query: &str) -> Option<String> {
        if !Self::is_navigate_query(query) {
            return None;
        }
        let lower = query.to_ascii_lowercase();

        if lower.contains("stack scale") || lower.contains("periodic stack scale") {
            return Some(self.scale_note());
        }

        // Family navigate
        if lower.contains("family") {
            for fam in StackFamily::ALL {
                if lower.contains(fam.name()) {
                    let mut present = Vec::new();
                    let mut gaps = Vec::new();
                    for c in self.by_family(fam) {
                        match c.status {
                            CellStatus::Present => present.push(c.name),
                            CellStatus::Gap => gaps.push(c.name),
                        }
                    }
                    return Some(format!(
                        "stack navigate family={} id={} present={:?} gaps={:?} \
                         (subset {}/{} primitives toward full {}; OOM estimates only, measured_j=None)",
                        fam.name(),
                        fam.id(),
                        present,
                        gaps,
                        self.present_count(),
                        FULL_TARGET_PRIMITIVES,
                        FULL_TARGET_PRIMITIVES
                    ));
                }
            }
            return Some(format!(
                "stack navigate: unknown family in query; known families={}; {}",
                StackFamily::COUNT,
                self.scale_note()
            ));
        }

        // Primitive navigate
        if let Some(tok) = Self::extract_primitive_token(query) {
            match self.get_by_name(&tok) {
                Some(c) if c.status == CellStatus::Present => {
                    return Some(format!(
                        "stack navigate primitive={} id={} family={} thermo={} status=present \
                         estimated_j_oom={:.3e}J (catalog surrogate ≠ RAPL) — {}",
                        c.name,
                        c.id,
                        c.family.name(),
                        c.thermo,
                        c.estimated_j_oom,
                        c.description
                    ));
                }
                Some(c) => {
                    // Gap during navigate — still answer with gap honesty (Lookup),
                    // but probe_query for close will refuse when gap intent.
                    return Some(format!(
                        "stack navigate primitive={} status=gap family={} — empty cell ({}); \
                         fire primitive_gap rather than invent coverage",
                        c.name,
                        c.family.name(),
                        c.description
                    ));
                }
                None => {
                    return Some(format!(
                        "stack navigate primitive='{tok}' status=absent from subset \
                         ({}/{} present); remaining_to_full={}",
                        self.present_count(),
                        FULL_TARGET_PRIMITIVES,
                        self.remaining_to_full()
                    ));
                }
            }
        }

        Some(self.scale_note())
    }
}

fn gap_or_stack_vocab(lower: &str) -> bool {
    lower.contains("gap")
        || lower.contains("primitive")
        || lower.contains("periodic stack")
        || lower.contains("probe")
        || lower.contains("missing")
}

macro_rules! cell {
    ($id:expr, $name:expr, $fam:expr, $th:expr, $st:expr, $j:expr, $desc:expr) => {
        StackPrimitive {
            id: $id,
            name: $name,
            family: $fam,
            thermo: $th,
            status: $st,
            estimated_j_oom: $j,
            description: $desc,
        }
    };
}

/// Compact subset: ≥1 representative Present per family where MoL has a hook,
/// plus explicit Gap markers (QI/thermo empty cells).
const SUBSET_CELLS: &[StackPrimitive] = &{
    use CellStatus::*;
    use StackFamily::*;
    use ThermoClass::*;
    [
        // --- Present (representative; MoL gears / honesty hooks) ---
        cell!(1, "add_f64", Arithmetic, L0, Present, 5e-12, "IEEE-754 add (representative)"),
        cell!(2, "landauer_bit", Arithmetic, L0, Present, 1e-12, "Landauer E_min = k_B T ln2 (Formula)"),
        cell!(3, "tokenize", ParseTransform, L1, Present, 1e-9, "Lexical scan (representative)"),
        cell!(4, "bool_and", Logic, L0, Present, 1e-12, "Boolean AND"),
        cell!(5, "dict_lookup", Lookup, L1, Present, 1e-9, "Hash-map get / LUT sense"),
        cell!(6, "cite_format", Retrieval, L1, Present, 2e-10, "Format citation string"),
        cell!(7, "schema_check", Constraint, L1, Present, 1e-9, "Schema / type check"),
        cell!(8, "hash_fnv1a", HashDigest, L0, Present, 5e-11, "FNV-1a 64-bit"),
        cell!(9, "json_encode", Encode, L1, Present, 2e-9, "JSON encode"),
        cell!(10, "route_zone", Control, L1, Present, 1e-10, "Zone router decision"),
        cell!(11, "template_fill", Generative, L2, Present, 1e-7, "Ungrounded template fill (residual)"),
        cell!(12, "hdc_bind", Hdc, L1, Present, 5e-9, "HDC bind"),
        cell!(13, "sha256", Crypto, L1, Present, 4e-9, "SHA-256 digest"),
        cell!(14, "stdout_write", Io, L1, Present, 1e-8, "Write receipt / answer"),
        cell!(15, "bfs", Graph, L1, Present, 5e-8, "Breadth-first search"),
        cell!(16, "solve_2x2", LinearAlgebra, L1, Present, 1e-9, "Dense 2×2 solve (Solver)"),
        cell!(17, "dist_euclid_2d", Geometry, L0, Present, 5e-12, "2D Euclidean distance"),
        cell!(18, "mean_f64", Statistics, L1, Present, 1e-10, "Arithmetic mean"),
        cell!(19, "moving_avg", Signal, L1, Present, 5e-10, "Moving average"),
        cell!(20, "rle_encode", Compression, L1, Present, 2e-9, "Run-length encode"),
        cell!(21, "binary_search", SortSearch, L0, Present, 5e-11, "Binary search"),
        cell!(22, "bst_lookup", Tree, L1, Present, 1e-9, "BST lookup"),
        cell!(23, "dfa_step", FiniteState, L0, Present, 2e-12, "DFA transition"),
        cell!(24, "type_check_expr", TypeCheck, L1, Present, 1e-9, "Expression type check"),
        cell!(25, "edit_distance", DiffPatch, L1, Present, 2e-8, "Levenshtein distance"),
        cell!(26, "interval_intersect", Temporal, L0, Present, 5e-12, "Interval intersection"),
        cell!(27, "celsius_to_fahrenheit", UnitConvert, L0, Present, 5e-12, "C → F (Lookup)"),
        cell!(28, "str_reverse", StringMorph, L0, Present, 5e-11, "Reverse string"),
        cell!(29, "set_union", SetOps, L1, Present, 2e-9, "Set union"),
        cell!(30, "entropy_bits", Probabilistic, L1, Present, 3e-10, "Shannon entropy (bits)"),
        cell!(31, "argmin", Optimization, L1, Present, 2e-10, "Argmin over array"),
        cell!(32, "url_join", Network, L0, Present, 5e-11, "Join URL parts"),
        cell!(33, "fs_exists", FileSystem, L1, Present, 1e-7, "Path exists check (sandboxed)"),
        cell!(34, "auto_emit_receipt", Automation, L1, Present, 1e-9, "Emit automation receipt"),
        cell!(35, "ternary_settle", Control, L1, Present, 1e-9, "Klere-style ternary settle (Solver)"),
        cell!(36, "shannon_capacity", Probabilistic, L0, Present, 5e-12, "C = B log2(1+SNR) (Formula)"),
        // --- Gap markers (empty cells; primitive_gap) ---
        cell!(
            200,
            "physical_settle",
            Optimization,
            L1,
            Gap,
            0.0,
            "Reserved QI/thermo settle-certify cell — not wired as silicon"
        ),
        cell!(
            201,
            "reversible_rewrite",
            Arithmetic,
            L0,
            Gap,
            0.0,
            "Reversible / adiabatic rewrite primitive — empty cell"
        ),
        cell!(
            202,
            "ising_bind",
            Optimization,
            L1,
            Gap,
            0.0,
            "Energy-function / Ising bind — empty cell"
        ),
        cell!(
            203,
            "adiabatic_schedule",
            Signal,
            L1,
            Gap,
            0.0,
            "Adiabatic anneal schedule driver — empty cell"
        ),
        cell!(
            204,
            "ferric_efa_cert",
            Constraint,
            L1,
            Gap,
            0.0,
            "On-device Ferric EFA certificate — out of proof scope"
        ),
    ]
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thirty_three_families() {
        assert_eq!(StackFamily::COUNT, 33);
        assert_eq!(StackFamily::ALL.len(), 33);
        let names: std::collections::HashSet<_> =
            StackFamily::ALL.iter().map(|f| f.name()).collect();
        assert_eq!(names.len(), 33);
    }

    #[test]
    fn subset_scale_vs_full() {
        let s = PeriodicStack::subset();
        assert_eq!(FULL_TARGET_PRIMITIVES, 258);
        assert_eq!(FULL_TARGET_FAMILIES, 33);
        assert!(s.present_count() >= 33, "at least one present theme per family band");
        assert!(s.gap_count() >= 4);
        assert!(s.remaining_to_full() > 200);
        assert!(s.scale_note().contains("258"));
    }

    #[test]
    fn probe_gap_physical_settle() {
        let s = PeriodicStack::subset();
        assert!(s.probe_name("physical_settle").is_gap());
        let q = s.probe_query("primitive gap: physical_settle not on stack");
        assert!(q.is_gap());
    }

    #[test]
    fn probe_present_not_gap() {
        let s = PeriodicStack::subset();
        match s.probe_name("celsius_to_fahrenheit") {
            ProbeResult::Present { name, .. } => assert_eq!(name, "celsius_to_fahrenheit"),
            other => panic!("expected present, got {other:?}"),
        }
    }

    #[test]
    fn navigate_family_and_scale() {
        let s = PeriodicStack::subset();
        let a = s
            .navigate_answer("stack navigate family arithmetic")
            .expect("nav");
        assert!(a.contains("family=arithmetic"));
        let sc = s.navigate_answer("periodic stack scale").unwrap();
        assert!(sc.contains("258"));
    }
}
