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


/// Which cascade gear can **live-execute** this Present cell (soft-ref).
///
/// `None` = name-only placeholder / residual (navigate ok; no Lookup/Formula/Solver close).
/// Live catalog honesty: only cells with a non-`None` gear count as executable entries
/// toward the 258 thesis table — never claim full coverage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GearKind {
    /// O(1) table / unit / boolean / hash Lookup.
    Lookup,
    /// Closed-form Formula.
    Formula,
    /// Deterministic Solver (linear / search / settle / combinatorial).
    Solver,
    /// Stack navigate / scale meta (Lookup-priced).
    Navigate,
    /// Named Present without a live gear close (placeholder).
    None,
}

impl GearKind {
    /// Snake label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Lookup => "lookup",
            Self::Formula => "formula",
            Self::Solver => "solver",
            Self::Navigate => "navigate",
            Self::None => "none",
        }
    }

    /// True when this cell can close at Lookup/Formula/Solver (not placeholder).
    pub const fn is_live(self) -> bool {
        matches!(self, Self::Lookup | Self::Formula | Self::Solver | Self::Navigate)
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
    /// Live cascade gear binding (None = placeholder name only).
    pub gear: GearKind,
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
            "Periodic Stack live catalog: {} families (all {});              {} seeded ({} present, {} gap); {} live Lookup/Formula/Solver/Navigate gears              ({} placeholder Present; {} toward 258 remain).              Full thesis: {} primitives / {} families (compute.openie.dev).              Live ≠ full 258; Gap cells refuse via primitive_gap; estimates≠measured_j.",
            FULL_TARGET_FAMILIES,
            FULL_TARGET_FAMILIES,
            self.seeded_count(),
            self.present_count(),
            self.gap_count(),
            self.live_gear_count(),
            self.placeholder_present_count(),
            self.remaining_to_full(),
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

    /// Present cells with a live Lookup/Formula/Solver/Navigate gear (not placeholders).
    pub fn live_gear_count(&self) -> usize {
        self.cells
            .iter()
            .filter(|c| c.status == CellStatus::Present && c.gear.is_live())
            .count()
    }

    /// Present cells that are name-only placeholders (gear=None).
    pub fn placeholder_present_count(&self) -> usize {
        self.cells
            .iter()
            .filter(|c| c.status == CellStatus::Present && c.gear == GearKind::None)
            .count()
    }

    /// Live Present cells bound to a specific gear.
    pub fn live_gear_count_of(&self, gear: GearKind) -> usize {
        self.cells
            .iter()
            .filter(|c| c.status == CellStatus::Present && c.gear == gear)
            .count()
    }

    /// Iterate live Present cells (executable gears only).
    pub fn live_cells(&self) -> impl Iterator<Item = &StackPrimitive> {
        self.cells
            .iter()
            .filter(|c| c.status == CellStatus::Present && c.gear.is_live())
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
    ($id:expr, $name:expr, $fam:ident, $th:ident, $st:ident, $gear:ident, $j:expr, $desc:expr) => {
        StackPrimitive {
            id: $id,
            name: $name,
            family: StackFamily::$fam,
            thermo: ThermoClass::$th,
            status: CellStatus::$st,
            gear: GearKind::$gear,
            estimated_j_oom: $j,
            description: $desc,
        }
    };
}

/// Live catalog + honest Gaps: Present cells with GearKind::Lookup|Formula|Solver|Navigate
/// are executable soft-ref entries (not placeholders). GearKind::None = name-only.
/// Thesis target remains 258/33 (compute.openie.dev); this ships a growing live subset.
const SUBSET_CELLS: &[StackPrimitive] = &[
    cell!(1, "add_f64", Arithmetic, L0, Present, Formula, 5e-12, "IEEE-754 add a+b (Formula)"),
    cell!(2, "landauer_bit", Arithmetic, L0, Present, Formula, 1e-12, "Landauer E_min = k_B T ln2 (Formula)"),
    cell!(3, "tokenize", ParseTransform, L1, Present, Lookup, 1e-9, "Lexical token count (Lookup)"),
    cell!(4, "bool_and", Logic, L0, Present, Lookup, 1e-12, "Boolean AND (Lookup)"),
    cell!(5, "dict_lookup", Lookup, L1, Present, Lookup, 1e-9, "Hash-map get / LUT sense (Lookup)"),
    cell!(6, "cite_format", Retrieval, L1, Present, Lookup, 2e-10, "Format citation string (Lookup)"),
    cell!(7, "schema_check", Constraint, L1, Present, Lookup, 1e-9, "Required-keys schema check (Lookup)"),
    cell!(8, "hash_fnv1a", HashDigest, L0, Present, Lookup, 5e-11, "FNV-1a 64-bit (Lookup)"),
    cell!(9, "json_encode", Encode, L1, Present, Lookup, 2e-9, "Encode key=value pairs as JSON object (Lookup)"),
    cell!(10, "route_zone", Control, L1, Present, Lookup, 1e-10, "Zone router Z1/Z2/Z3 (Lookup)"),
    cell!(11, "template_fill", Generative, L2, Present, None, 1e-7, "Ungrounded template fill (placeholder residual)"),
    cell!(12, "hdc_bind", Hdc, L1, Present, Lookup, 5e-9, "HDC XOR-bind of two hex tokens (Lookup)"),
    cell!(13, "sha256", Crypto, L1, Present, Lookup, 4e-9, "SHA-256 hex digest of payload= (Lookup)"),
    cell!(14, "stdout_write", Io, L1, Present, Lookup, 1e-8, "Echo write payload (Lookup soft-ref)"),
    cell!(15, "bfs", Graph, L1, Present, Solver, 5e-8, "Tiny BFS reachability (Solver)"),
    cell!(16, "solve_2x2", LinearAlgebra, L1, Present, Solver, 1e-9, "Dense 2×2 solve (Solver)"),
    cell!(17, "dist_euclid_2d", Geometry, L0, Present, Solver, 5e-12, "2D Euclidean distance (Solver)"),
    cell!(18, "mean_f64", Statistics, L1, Present, Solver, 1e-10, "Arithmetic mean (Solver)"),
    cell!(19, "moving_avg", Signal, L1, Present, Solver, 5e-10, "Moving average window (Solver)"),
    cell!(20, "rle_encode", Compression, L1, Present, Lookup, 2e-9, "Run-length encode ASCII (Lookup)"),
    cell!(21, "binary_search", SortSearch, L0, Present, Solver, 5e-11, "Binary search sorted list (Solver)"),
    cell!(22, "bst_lookup", Tree, L1, Present, Lookup, 1e-9, "Sorted-list BST-style membership (Lookup)"),
    cell!(23, "dfa_step", FiniteState, L0, Present, Lookup, 2e-12, "DFA transition step (Lookup)"),
    cell!(24, "type_check_expr", TypeCheck, L1, Present, Lookup, 1e-9, "Simple expr type check (Lookup)"),
    cell!(25, "edit_distance", DiffPatch, L1, Present, Solver, 2e-8, "Levenshtein distance (Solver)"),
    cell!(26, "interval_intersect", Temporal, L0, Present, Solver, 5e-12, "Interval intersection (Solver)"),
    cell!(27, "celsius_to_fahrenheit", UnitConvert, L0, Present, Lookup, 5e-12, "C → F (Lookup)"),
    cell!(28, "str_reverse", StringMorph, L0, Present, Lookup, 5e-11, "Reverse string (Lookup)"),
    cell!(29, "set_union", SetOps, L1, Present, Solver, 2e-9, "Set union (Solver)"),
    cell!(30, "entropy_bits", Probabilistic, L1, Present, Formula, 3e-10, "Shannon entropy bits (Formula)"),
    cell!(31, "argmin", Optimization, L1, Present, Solver, 2e-10, "Argmin over array (Solver)"),
    cell!(32, "url_join", Network, L0, Present, Lookup, 5e-11, "Join URL parts (Lookup)"),
    cell!(33, "fs_exists", FileSystem, L1, Present, Lookup, 1e-7, "Sandboxed path-exists check (Lookup)"),
    cell!(34, "auto_emit_receipt", Automation, L1, Present, Lookup, 1e-9, "Emit automation receipt stub (Lookup)"),
    cell!(35, "ternary_settle", Control, L1, Present, Solver, 1e-9, "Klere-style ternary settle (Solver)"),
    cell!(36, "shannon_capacity", Probabilistic, L0, Present, Formula, 5e-12, "C = B log2(1+SNR) (Formula)"),
    cell!(37, "mul_f64", Arithmetic, L0, Present, Formula, 5e-12, "IEEE-754 multiply a*b (Formula)"),
    cell!(38, "div_f64", Arithmetic, L0, Present, Formula, 5e-12, "IEEE-754 divide a/b (Formula)"),
    cell!(39, "pow_f64", Arithmetic, L0, Present, Formula, 1e-11, "a^b power (Formula)"),
    cell!(40, "pythagoras", Geometry, L0, Present, Formula, 5e-12, "c = sqrt(a²+b²) (Formula)"),
    cell!(41, "ohms_law", Arithmetic, L0, Present, Formula, 5e-12, "V = I·R (Formula)"),
    cell!(42, "kinetic_energy", Arithmetic, L0, Present, Formula, 5e-12, "KE = ½ m v² (Formula)"),
    cell!(43, "ideal_gas", Arithmetic, L0, Present, Formula, 5e-12, "PV = nRT (Formula)"),
    cell!(44, "coulomb", Arithmetic, L0, Present, Formula, 5e-12, "F = k q1 q2 / r² (Formula)"),
    cell!(45, "planck_e", Arithmetic, L0, Present, Formula, 5e-12, "E = h f (Formula)"),
    cell!(46, "boltzmann_factor", Probabilistic, L0, Present, Formula, 5e-12, "exp(-E/kT) (Formula)"),
    cell!(47, "snr_db", Signal, L0, Present, Formula, 5e-12, "SNR_dB = 10 log10(S/N) (Formula)"),
    cell!(48, "compound_interest", Arithmetic, L0, Present, Formula, 5e-12, "A = P(1+r)^n (Formula)"),
    cell!(49, "gaussian_pdf", Statistics, L0, Present, Formula, 1e-11, "Normal PDF φ(x;μ,σ) (Formula)"),
    cell!(50, "nyquist_rate", Signal, L0, Present, Formula, 5e-12, "Nyquist rate = 2B (Formula)"),
    cell!(51, "rest_energy", Arithmetic, L0, Present, Formula, 5e-12, "E = mc² (Formula)"),
    cell!(52, "wave_lambda", Signal, L0, Present, Formula, 5e-12, "λ = c / f (Formula)"),
    cell!(53, "logit", Probabilistic, L0, Present, Formula, 5e-12, "logit(p) = ln(p/(1-p)) (Formula)"),
    cell!(54, "sigmoid", Probabilistic, L0, Present, Formula, 5e-12, "σ(x) = 1/(1+e^{-x}) (Formula)"),
    cell!(55, "bit_bound", Probabilistic, L0, Present, Formula, 5e-12, "N log2 M alphabet bits (Formula)"),
    cell!(56, "bool_or", Logic, L0, Present, Lookup, 1e-12, "Boolean OR (Lookup)"),
    cell!(57, "bool_xor", Logic, L0, Present, Lookup, 1e-12, "Boolean XOR (Lookup)"),
    cell!(58, "bool_not", Logic, L0, Present, Lookup, 1e-12, "Boolean NOT (Lookup)"),
    cell!(59, "meters_to_feet", UnitConvert, L0, Present, Lookup, 5e-12, "m ↔ ft (Lookup)"),
    cell!(60, "kg_to_lb", UnitConvert, L0, Present, Lookup, 5e-12, "kg ↔ lb (Lookup)"),
    cell!(61, "joule_to_ev", UnitConvert, L0, Present, Lookup, 5e-12, "J ↔ eV (Lookup)"),
    cell!(62, "kelvin_to_celsius", UnitConvert, L0, Present, Lookup, 5e-12, "K ↔ °C (Lookup)"),
    cell!(63, "radians_to_degrees", UnitConvert, L0, Present, Lookup, 5e-12, "rad ↔ deg (Lookup)"),
    cell!(64, "compare_pred", Arithmetic, L0, Present, Lookup, 5e-12, "Compare a ? b → bool (Lookup)"),
    cell!(65, "clamp_f64", Arithmetic, L0, Present, Lookup, 5e-12, "Clamp x into [lo,hi] (Lookup)"),
    cell!(66, "str_len", StringMorph, L0, Present, Lookup, 5e-11, "String length (Lookup)"),
    cell!(67, "str_upper", StringMorph, L0, Present, Lookup, 5e-11, "ASCII uppercase (Lookup)"),
    cell!(68, "set_member", SetOps, L0, Present, Lookup, 1e-9, "Set membership (Lookup)"),
    cell!(69, "ticket_lut", Lookup, L1, Present, Lookup, 1e-9, "Support-desk resolution LUT (Lookup)"),
    cell!(70, "risk_lut", Lookup, L1, Present, Lookup, 1e-9, "Risk-band LUT (Lookup)"),
    cell!(71, "stack_navigate", Lookup, L0, Present, Navigate, 1e-12, "Periodic Stack family/primitive/scale navigate"),
    cell!(72, "argmax", Optimization, L1, Present, Solver, 2e-10, "Argmax over array (Solver)"),
    cell!(73, "variance_f64", Statistics, L1, Present, Solver, 1e-10, "Population variance (Solver)"),
    cell!(74, "std_f64", Statistics, L1, Present, Solver, 1e-10, "Population stdev (Solver)"),
    cell!(75, "set_intersect", SetOps, L1, Present, Solver, 2e-9, "Set intersection (Solver)"),
    cell!(76, "set_difference", SetOps, L1, Present, Solver, 2e-9, "Set difference (Solver)"),
    cell!(77, "gcd_u64", Arithmetic, L0, Present, Solver, 5e-12, "GCD of two integers (Solver)"),
    cell!(78, "lcm_u64", Arithmetic, L0, Present, Solver, 5e-12, "LCM of two integers (Solver)"),
    cell!(79, "dot_f64", LinearAlgebra, L0, Present, Solver, 5e-12, "Dot product of equal-length vectors (Solver)"),
    cell!(80, "ticket_route", Control, L1, Present, Solver, 1e-9, "Ticket route rules / SAT assign (Solver)"),
    cell!(81, "knapsack_01", Optimization, L1, Present, Solver, 5e-8, "Tiny 0-1 knapsack N≤8 (Solver)"),
    cell!(82, "sort_f64", SortSearch, L1, Present, Solver, 1e-9, "Stable sort ascending (Solver)"),
    cell!(83, "median_f64", Statistics, L1, Present, Solver, 1e-10, "Median of array (Solver)"),
    cell!(84, "dist_euclid_3d", Geometry, L0, Present, Solver, 5e-12, "3D Euclidean distance (Solver)"),
    cell!(85, "hamming_distance", DiffPatch, L0, Present, Solver, 5e-11, "Hamming distance equal-length strings (Solver)"),
    cell!(86, "risk_formula", Probabilistic, L0, Present, Formula, 5e-12, "Closed-form risk S=(sev/5)·exp·lik (Formula)"),
    cell!(87, "prefix_sum", SortSearch, L0, Present, Solver, 1e-10, "Inclusive prefix sum (Solver)"),
    cell!(88, "matmul_2x2", LinearAlgebra, L1, Present, Solver, 1e-9, "2×2 matrix multiply (Solver)"),
    cell!(89, "det_2x2", LinearAlgebra, L0, Present, Formula, 5e-12, "2×2 determinant (Formula)"),
    cell!(90, "lerp", Arithmetic, L0, Present, Formula, 5e-12, "Linear interpolate (1-t)a + t b (Formula)"),
    cell!(200, "physical_settle", Optimization, L1, Gap, None, 0.0, "Reserved QI/thermo settle-certify cell — not wired as silicon"),
    cell!(201, "reversible_rewrite", Arithmetic, L0, Gap, None, 0.0, "Reversible / adiabatic rewrite primitive — empty cell"),
    cell!(202, "ising_bind", Optimization, L1, Gap, None, 0.0, "Energy-function / Ising bind — empty cell"),
    cell!(203, "adiabatic_schedule", Signal, L1, Gap, None, 0.0, "Adiabatic anneal schedule driver — empty cell"),
    cell!(204, "ferric_efa_cert", Constraint, L1, Gap, None, 0.0, "On-device Ferric EFA certificate — out of proof scope"),
    cell!(205, "quantum_gate_ops", LinearAlgebra, L0, Gap, None, 0.0, "Thesis TEN quantum gate ops — empty / HW emerging"),
    cell!(206, "analog_crossbar_mac", LinearAlgebra, L1, Gap, None, 0.0, "Thesis ANALOG crossbar MAC — empty / emerging"),
    cell!(207, "photonic_mzi", Signal, L0, Gap, None, 0.0, "Thesis ANALOG photonic MZI — empty / emerging"),
];

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn live_catalog_honesty() {
        let s = PeriodicStack::subset();
        assert!(s.live_gear_count() >= 80, "live gears={}", s.live_gear_count());
        assert!(s.live_gear_count_of(GearKind::Lookup) >= 25);
        assert!(s.live_gear_count_of(GearKind::Formula) >= 20);
        assert!(s.live_gear_count_of(GearKind::Solver) >= 20);
        assert!(s.placeholder_present_count() <= 5, "few placeholders only");
        assert!(s.gap_count() >= 5);
        assert!(s.scale_note().contains("live catalog"));
        assert!(s.scale_note().contains("258"));
        for c in s.live_cells() {
            assert_eq!(c.status, CellStatus::Present);
            assert!(c.gear.is_live());
        }
        for c in s.iter().filter(|c| c.status == CellStatus::Gap) {
            assert_eq!(c.gear, GearKind::None);
        }
    }

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
        assert!(s.present_count() >= 80, "expanded live present={}", s.present_count());
        assert!(s.live_gear_count() >= 80);
        assert!(s.gap_count() >= 4);
        assert!(s.remaining_to_full() > 150, "remain={}", s.remaining_to_full());
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
