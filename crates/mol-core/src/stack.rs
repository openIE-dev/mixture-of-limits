//! Periodic Stack subset navigator (clean-room, in-tree).
//!
//! Thesis table (compute.openie.dev): **258 primitives / 33 families**.
//! This module ships a **live catalog (258 Present)**: all 33 family names + Present
//! cells with real Lookup/Formula/Solver/Navigate gears (not placeholders) and
//! explicit **Gap** markers for empty cells (e.g. `physical_settle`). Present count reaches 258; Gaps remain honest empty/HW markers (not fake physics)
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
    cell!(11, "template_fill", Generative, L2, Present, Lookup, 1e-9, "Grounded template fill / residual policy (Lookup; refuse ungrounded slots)"),
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
    cell!(91, "str_lower", StringMorph, L0, Present, Lookup, 5e-11, "ASCII lowercase (Lookup)"),
    cell!(92, "str_trim", StringMorph, L0, Present, Lookup, 5e-11, "Trim ASCII whitespace (Lookup)"),
    cell!(93, "str_contains", StringMorph, L0, Present, Lookup, 5e-11, "Substring contains (Lookup)"),
    cell!(94, "popcount", Logic, L0, Present, Lookup, 1e-12, "Population count of u64 (Lookup)"),
    cell!(95, "parity_even", Logic, L0, Present, Lookup, 1e-12, "Even parity of u64 (Lookup)"),
    cell!(96, "hex_encode", Encode, L0, Present, Lookup, 5e-11, "Bytes/ASCII → hex (Lookup)"),
    cell!(97, "mime_from_ext", Lookup, L0, Present, Lookup, 1e-12, "Extension → MIME LUT (Lookup)"),
    cell!(98, "http_status_phrase", Lookup, L0, Present, Lookup, 1e-12, "HTTP status → phrase (Lookup)"),
    cell!(99, "is_ascii", ParseTransform, L0, Present, Lookup, 5e-12, "All-ASCII predicate (Lookup)"),
    cell!(100, "crc8", HashDigest, L0, Present, Lookup, 5e-11, "CRC-8/ATM soft-ref (Lookup)"),
    cell!(101, "weekday_name", Temporal, L0, Present, Lookup, 5e-12, "Dow 0..6 → weekday name (Lookup)"),
    cell!(102, "bytes_to_kib", UnitConvert, L0, Present, Lookup, 5e-12, "Bytes ↔ KiB (Lookup)"),
    cell!(103, "bool_nand", Logic, L0, Present, Lookup, 1e-12, "Boolean NAND (Lookup)"),
    cell!(104, "csv_field", Encode, L0, Present, Lookup, 5e-11, "CSV field quote/escape (Lookup)"),
    cell!(105, "area_circle", Geometry, L0, Present, Formula, 5e-12, "A = π r² (Formula)"),
    cell!(106, "circumference", Geometry, L0, Present, Formula, 5e-12, "C = 2 π r (Formula)"),
    cell!(107, "volume_sphere", Geometry, L0, Present, Formula, 5e-12, "V = 4/3 π r³ (Formula)"),
    cell!(108, "heron_area", Geometry, L0, Present, Formula, 1e-11, "Heron triangle area (Formula)"),
    cell!(109, "quadratic_roots", Arithmetic, L0, Present, Formula, 1e-11, "Roots of ax²+bx+c (Formula)"),
    cell!(110, "softplus", Probabilistic, L0, Present, Formula, 5e-12, "softplus(x)=ln(1+e^x) (Formula)"),
    cell!(111, "percent_change", Arithmetic, L0, Present, Formula, 5e-12, "100·(new-old)/old (Formula)"),
    cell!(112, "capacitor_energy", Arithmetic, L0, Present, Formula, 5e-12, "E = ½ C V² (Formula)"),
    cell!(113, "freefall_distance", Arithmetic, L0, Present, Formula, 5e-12, "d = ½ g t² (Formula)"),
    cell!(114, "geometric_mean", Statistics, L0, Present, Formula, 1e-11, "Geometric mean of positives (Formula)"),
    cell!(115, "harmonic_mean", Statistics, L0, Present, Formula, 1e-11, "Harmonic mean of positives (Formula)"),
    cell!(116, "beer_lambert", Signal, L0, Present, Formula, 5e-12, "A = ε · c · l (Formula)"),
    cell!(117, "snell_law", Geometry, L0, Present, Formula, 5e-12, "n1 sin θ1 = n2 sin θ2 (Formula)"),
    cell!(118, "mode_f64", Statistics, L1, Present, Solver, 1e-10, "Mode of multiset (Solver)"),
    cell!(119, "range_f64", Statistics, L0, Present, Solver, 5e-12, "max-min range (Solver)"),
    cell!(120, "l1_norm", LinearAlgebra, L0, Present, Solver, 5e-12, "L1 / Manhattan norm (Solver)"),
    cell!(121, "l2_norm", LinearAlgebra, L0, Present, Solver, 5e-12, "L2 / Euclidean norm (Solver)"),
    cell!(122, "cosine_sim", LinearAlgebra, L0, Present, Solver, 1e-11, "Cosine similarity (Solver)"),
    cell!(123, "transpose_2x2", LinearAlgebra, L0, Present, Solver, 5e-12, "2×2 transpose (Solver)"),
    cell!(124, "trace_2x2", LinearAlgebra, L0, Present, Solver, 5e-12, "2×2 trace (Solver)"),
    cell!(125, "fibonacci_u64", Arithmetic, L0, Present, Solver, 5e-12, "Fibonacci F(n) n≤92 (Solver)"),
    cell!(126, "is_prime_u64", Arithmetic, L0, Present, Solver, 5e-11, "Primality test u64 (Solver)"),
    cell!(127, "is_sorted", SortSearch, L0, Present, Solver, 5e-11, "Nondecreasing check (Solver)"),
    cell!(128, "unique_count", SortSearch, L1, Present, Solver, 1e-10, "Distinct element count (Solver)"),
    cell!(129, "paren_balance", ParseTransform, L0, Present, Solver, 5e-11, "Parenthesis balance check (Solver)"),
    cell!(130, "lcp_strings", StringMorph, L1, Present, Solver, 1e-10, "Longest common prefix (Solver)"),
    cell!(131, "str_starts_with", StringMorph, L0, Present, Lookup, 5e-11, "Prefix predicate (Lookup)"),
    cell!(132, "str_ends_with", StringMorph, L0, Present, Lookup, 5e-11, "Suffix predicate (Lookup)"),
    cell!(133, "str_replace", StringMorph, L0, Present, Lookup, 5e-11, "Replace first needle (Lookup)"),
    cell!(134, "bool_nor", Logic, L0, Present, Lookup, 1e-12, "Boolean NOR (Lookup)"),
    cell!(135, "bool_xnor", Logic, L0, Present, Lookup, 1e-12, "Boolean XNOR (Lookup)"),
    cell!(136, "bit_and", Logic, L0, Present, Lookup, 1e-12, "Bitwise AND u64 (Lookup)"),
    cell!(137, "bit_or", Logic, L0, Present, Lookup, 1e-12, "Bitwise OR u64 (Lookup)"),
    cell!(138, "bit_xor", Logic, L0, Present, Lookup, 1e-12, "Bitwise XOR u64 (Lookup)"),
    cell!(139, "is_hex", Encode, L0, Present, Lookup, 5e-12, "Hex-string predicate (Lookup)"),
    cell!(140, "html_escape", Encode, L0, Present, Lookup, 5e-11, "HTML entity escape (Lookup)"),
    cell!(141, "path_ext", FileSystem, L0, Present, Lookup, 5e-11, "Path extension extract (Lookup)"),
    cell!(142, "path_basename", FileSystem, L0, Present, Lookup, 5e-11, "Path basename extract (Lookup)"),
    cell!(143, "color_hex_rgb", Lookup, L0, Present, Lookup, 5e-12, "Hex color → RGB (Lookup)"),
    cell!(144, "ipv4_ok", Network, L0, Present, Lookup, 5e-11, "IPv4 dotted-quad validate (Lookup)"),
    cell!(145, "is_blank", StringMorph, L0, Present, Lookup, 5e-12, "Whitespace-only predicate (Lookup)"),
    cell!(146, "bit_shl", Logic, L0, Present, Lookup, 1e-12, "Bitwise shift-left u64 (Lookup)"),
    cell!(147, "bit_shr", Logic, L0, Present, Lookup, 1e-12, "Bitwise shift-right u64 (Lookup)"),
    cell!(148, "json_escape", Encode, L0, Present, Lookup, 5e-11, "JSON string escape (Lookup)"),
    cell!(149, "mime_is_text", Lookup, L0, Present, Lookup, 1e-12, "MIME text/* predicate (Lookup)"),
    cell!(150, "miles_to_km", UnitConvert, L0, Present, Lookup, 5e-12, "mi ↔ km (Lookup)"),
    cell!(151, "area_rectangle", Geometry, L0, Present, Formula, 5e-12, "A = w·h (Formula)"),
    cell!(152, "area_triangle", Geometry, L0, Present, Formula, 5e-12, "A = ½ b h (Formula)"),
    cell!(153, "volume_cylinder", Geometry, L0, Present, Formula, 5e-12, "V = π r² h (Formula)"),
    cell!(154, "potential_energy", Arithmetic, L0, Present, Formula, 5e-12, "PE = m g h (Formula)"),
    cell!(155, "centripetal_accel", Arithmetic, L0, Present, Formula, 5e-12, "a = v² / r (Formula)"),
    cell!(156, "bmi_formula", Arithmetic, L0, Present, Formula, 5e-12, "BMI = kg / m² (Formula)"),
    cell!(157, "abs_f64", Arithmetic, L0, Present, Formula, 5e-12, "Absolute value (Formula)"),
    cell!(158, "log2_f64", Arithmetic, L0, Present, Formula, 5e-12, "log₂(x) (Formula)"),
    cell!(159, "exp_f64", Arithmetic, L0, Present, Formula, 5e-12, "e^x (Formula)"),
    cell!(160, "relative_error", Arithmetic, L0, Present, Formula, 5e-12, "|approx-true|/|true| (Formula)"),
    cell!(161, "wien_peak", Signal, L0, Present, Formula, 5e-12, "Wien λ_max = b/T (Formula)"),
    cell!(162, "reynolds_number", Signal, L0, Present, Formula, 5e-12, "Re = ρ v L / μ (Formula)"),
    cell!(163, "escape_velocity", Arithmetic, L0, Present, Formula, 5e-12, "v_esc = sqrt(2GM/r) (Formula)"),
    cell!(164, "circular_period", Arithmetic, L0, Present, Formula, 5e-12, "T = 2π sqrt(r³/GM) (Formula)"),
    cell!(165, "binomial_coeff", Probabilistic, L0, Present, Formula, 1e-11, "C(n,k) n≤66 (Formula)"),
    cell!(166, "sum_f64", Statistics, L0, Present, Solver, 5e-12, "Sum of array (Solver)"),
    cell!(167, "product_f64", Statistics, L0, Present, Solver, 5e-12, "Product of array (Solver)"),
    cell!(168, "min_f64", Statistics, L0, Present, Solver, 5e-12, "Minimum of array (Solver)"),
    cell!(169, "max_f64", Statistics, L0, Present, Solver, 5e-12, "Maximum of array (Solver)"),
    cell!(170, "factorial_u64", Arithmetic, L0, Present, Solver, 5e-12, "n! for n≤20 (Solver)"),
    cell!(171, "lcs_length", DiffPatch, L1, Present, Solver, 2e-8, "LCS length (Solver)"),
    cell!(172, "matrix_add_2x2", LinearAlgebra, L0, Present, Solver, 5e-12, "2×2 matrix add (Solver)"),
    cell!(173, "inv_2x2", LinearAlgebra, L0, Present, Solver, 1e-11, "2×2 inverse (Solver)"),
    cell!(174, "cross_3d", LinearAlgebra, L0, Present, Solver, 5e-12, "3D cross product (Solver)"),
    cell!(175, "pearson_corr", Statistics, L1, Present, Solver, 1e-10, "Pearson r (Solver)"),
    cell!(176, "merge_sorted", SortSearch, L1, Present, Solver, 1e-10, "Merge two sorted lists (Solver)"),
    cell!(177, "next_prime", Arithmetic, L0, Present, Solver, 5e-11, "Next prime ≥ n (Solver)"),
    cell!(178, "combinations_u64", Arithmetic, L0, Present, Solver, 5e-12, "P(n,k)=n!/(n-k)! n≤20 (Solver)"),
    cell!(179, "dfs_reach", Graph, L1, Present, Solver, 5e-8, "Tiny DFS reachability (Solver)"),
    cell!(180, "set_symmetric_diff", SetOps, L1, Present, Solver, 2e-9, "Symmetric difference (Solver)"),
    cell!(181, "sqrt_f64", Arithmetic, L0, Present, Formula, 5e-12, "sqrt(x) (Formula)"),
    cell!(182, "ln_f64", Arithmetic, L0, Present, Formula, 5e-12, "ln(x) (Formula)"),
    cell!(183, "sin_f64", Arithmetic, L0, Present, Formula, 5e-12, "sin(x rad) (Formula)"),
    cell!(184, "cos_f64", Arithmetic, L0, Present, Formula, 5e-12, "cos(x rad) (Formula)"),
    cell!(185, "tan_f64", Arithmetic, L0, Present, Formula, 5e-12, "tan(x rad) (Formula)"),
    cell!(186, "hypot_f64", Geometry, L0, Present, Formula, 5e-12, "hypot(a,b)=√(a²+b²) (Formula)"),
    cell!(187, "volume_cone", Geometry, L0, Present, Formula, 5e-12, "V=⅓πr²h (Formula)"),
    cell!(188, "area_trapezoid", Geometry, L0, Present, Formula, 5e-12, "A=½(a+b)h (Formula)"),
    cell!(189, "hookes_law", Arithmetic, L0, Present, Formula, 5e-12, "F=-k·x (Formula)"),
    cell!(190, "work_force_dist", Arithmetic, L0, Present, Formula, 5e-12, "W=F·d (Formula)"),
    cell!(191, "power_energy_time", Arithmetic, L0, Present, Formula, 5e-12, "P=E/t (Formula)"),
    cell!(192, "density_mass_vol", Arithmetic, L0, Present, Formula, 5e-12, "ρ=m/V (Formula)"),
    cell!(193, "pressure_force_area", Arithmetic, L0, Present, Formula, 5e-12, "P=F/A (Formula)"),
    cell!(194, "gravitational_force", Arithmetic, L0, Present, Formula, 5e-12, "F=G m1 m2 / r² (Formula)"),
    cell!(195, "stefan_boltzmann", Signal, L0, Present, Formula, 5e-12, "j=σ T⁴ (Formula)"),
    cell!(196, "arrhenius", Probabilistic, L0, Present, Formula, 5e-12, "k=A exp(-Ea/RT) (Formula)"),
    cell!(197, "half_life", Probabilistic, L0, Present, Formula, 5e-12, "N=N0·(1/2)^(t/t½) (Formula)"),
    cell!(198, "beat_frequency", Signal, L0, Present, Formula, 5e-12, "f_beat=|f1-f2| (Formula)"),
    cell!(199, "capacitance_parallel", Arithmetic, L0, Present, Formula, 5e-12, "C=C1+C2 (Formula)"),
    cell!(200, "inductance_energy", Arithmetic, L0, Present, Formula, 5e-12, "E=½ L I² (Formula)"),
    cell!(201, "refractive_index", Geometry, L0, Present, Formula, 5e-12, "n=c/v (Formula)"),
    cell!(202, "doppler_shift", Signal, L0, Present, Formula, 5e-12, "f'=f·(v±vo)/(v±vs) (Formula)"),
    cell!(203, "coulomb_potential", Arithmetic, L0, Present, Formula, 5e-12, "V=k q / r (Formula)"),
    cell!(204, "terminal_velocity", Arithmetic, L0, Present, Formula, 5e-12, "v=sqrt(2mg/(ρ A Cd)) (Formula)"),
    cell!(205, "orbit_velocity", Arithmetic, L0, Present, Formula, 5e-12, "v=sqrt(GM/r) (Formula)"),
    cell!(206, "photon_momentum", Arithmetic, L0, Present, Formula, 5e-12, "p=h/λ (Formula)"),
    cell!(207, "base64_encode", Encode, L0, Present, Lookup, 2e-9, "ASCII → base64 (Lookup)"),
    cell!(208, "base64_decode", Encode, L0, Present, Lookup, 2e-9, "base64 → ASCII (Lookup)"),
    cell!(209, "url_encode", Encode, L0, Present, Lookup, 1e-9, "Percent-encode ASCII (Lookup)"),
    cell!(210, "is_email_shape", ParseTransform, L0, Present, Lookup, 5e-11, "Loose email-shape predicate (Lookup)"),
    cell!(211, "is_uuid_shape", ParseTransform, L0, Present, Lookup, 5e-11, "UUID hex-shape predicate (Lookup)"),
    cell!(212, "weekday_from_ymd", Temporal, L0, Present, Lookup, 5e-12, "Civil weekday from Y-M-D (Lookup)"),
    cell!(213, "month_name", Temporal, L0, Present, Lookup, 5e-11, "Month 1..12 → name (Lookup)"),
    cell!(214, "liters_to_gallons", UnitConvert, L0, Present, Lookup, 5e-11, "L ↔ US gal (Lookup)"),
    cell!(215, "watts_to_hp", UnitConvert, L0, Present, Lookup, 5e-11, "W ↔ mechanical hp (Lookup)"),
    cell!(216, "pascal_to_psi", UnitConvert, L0, Present, Lookup, 5e-11, "Pa ↔ psi (Lookup)"),
    cell!(217, "str_is_numeric", StringMorph, L0, Present, Lookup, 5e-11, "ASCII numeric predicate (Lookup)"),
    cell!(218, "str_pad_left", StringMorph, L0, Present, Lookup, 5e-11, "Left-pad string (Lookup)"),
    cell!(219, "str_repeat", StringMorph, L0, Present, Lookup, 5e-11, "Repeat string n times (Lookup)"),
    cell!(220, "bit_count_ones", Logic, L0, Present, Lookup, 5e-11, "Count set bits u64 (Lookup)"),
    cell!(221, "bit_rotate_left", Logic, L0, Present, Lookup, 5e-11, "Rotate-left u64 (Lookup)"),
    cell!(222, "crc16_ccitt", HashDigest, L0, Present, Lookup, 5e-11, "CRC-16/CCITT soft-ref (Lookup)"),
    cell!(223, "isbn10_check", Constraint, L1, Present, Lookup, 5e-11, "ISBN-10 check digit (Lookup)"),
    cell!(224, "luhn_check", Constraint, L1, Present, Lookup, 5e-11, "Luhn checksum (Lookup)"),
    cell!(225, "http_method_ok", Network, L0, Present, Lookup, 5e-11, "HTTP method allowlist (Lookup)"),
    cell!(226, "port_well_known", Network, L0, Present, Lookup, 5e-11, "Well-known port → service (Lookup)"),
    cell!(227, "path_dirname", FileSystem, L0, Present, Lookup, 5e-11, "Path dirname extract (Lookup)"),
    cell!(228, "mime_charset_utf8", Lookup, L0, Present, Lookup, 5e-11, "MIME charset=utf-8 stamp (Lookup)"),
    cell!(229, "zone_from_tier", Control, L0, Present, Lookup, 5e-11, "Cascade tier → OpenIE zone (Lookup)"),
    cell!(230, "estimate_kind_label", Lookup, L0, Present, Lookup, 5e-11, "EstimateKind → honesty label (Lookup)"),
    cell!(231, "residual_policy", Generative, L1, Present, Lookup, 1e-9, "Grounded residual escalate|refuse policy (Lookup)"),
    cell!(232, "cite_style_apa", Retrieval, L1, Present, Lookup, 5e-11, "Minimal APA cite format (Lookup)"),
    cell!(233, "cumsum_f64", Statistics, L0, Present, Solver, 1e-10, "Cumulative sum (Solver)"),
    cell!(234, "cumprod_f64", Statistics, L0, Present, Solver, 1e-10, "Cumulative product (Solver)"),
    cell!(235, "percentile_f64", Statistics, L1, Present, Solver, 1e-10, "Nearest-rank percentile (Solver)"),
    cell!(236, "zscore_f64", Statistics, L1, Present, Solver, 1e-10, "Z-score of x vs array (Solver)"),
    cell!(237, "matmul_vec_2", LinearAlgebra, L0, Present, Solver, 1e-10, "2×2 · vec2 (Solver)"),
    cell!(238, "norm_inf", LinearAlgebra, L0, Present, Solver, 1e-10, "Infinity norm (Solver)"),
    cell!(239, "angle_between_2d", Geometry, L0, Present, Solver, 1e-10, "Angle between 2D vectors (Solver)"),
    cell!(240, "polygon_area", Geometry, L1, Present, Solver, 5e-11, "Shoelace polygon area (Solver)"),
    cell!(241, "edit_script_len", DiffPatch, L1, Present, Solver, 1e-10, "Levenshtein ops count alias (Solver)"),
    cell!(242, "longest_run", Compression, L0, Present, Solver, 1e-10, "Longest identical run length (Solver)"),
    cell!(243, "rle_decode", Compression, L0, Present, Solver, 1e-10, "Decode simple RLE pairs (Solver)"),
    cell!(244, "top_k_f64", SortSearch, L0, Present, Solver, 1e-10, "Top-k descending (Solver)"),
    cell!(245, "argsort_f64", SortSearch, L0, Present, Solver, 1e-10, "Argsort ascending (Solver)"),
    cell!(246, "is_palindrome", StringMorph, L0, Present, Solver, 1e-10, "Palindrome predicate (Solver)"),
    cell!(247, "anagram_check", StringMorph, L0, Present, Solver, 1e-10, "Anagram predicate (Solver)"),
    cell!(248, "set_issubset", SetOps, L0, Present, Solver, 1e-10, "Subset check (Solver)"),
    cell!(249, "set_cardinality", SetOps, L0, Present, Solver, 1e-10, "Distinct cardinality (Solver)"),
    cell!(250, "power_set_size", SetOps, L0, Present, Solver, 1e-10, "2^|S| for |S|≤20 (Solver)"),
    cell!(251, "dijkstra_tiny", Graph, L1, Present, Solver, 5e-8, "Tiny Dijkstra soft-ref N≤8 (Solver)"),
    cell!(252, "topo_sort_tiny", Graph, L1, Present, Solver, 5e-8, "Tiny Kahn topo-sort (Solver)"),
    cell!(253, "binary_gcd_steps", Arithmetic, L0, Present, Solver, 1e-10, "Binary GCD step count (Solver)"),
    cell!(254, "mod_pow_u64", Arithmetic, L0, Present, Solver, 1e-10, "Modular exponentiation (Solver)"),
    cell!(255, "chinese_remainder_2", Arithmetic, L0, Present, Solver, 1e-10, "CRT for two coprime moduli (Solver)"),
    cell!(256, "interval_union_len", Temporal, L0, Present, Solver, 1e-10, "Union length of intervals (Solver)"),
    cell!(257, "knapsack_unbounded_tiny", Optimization, L1, Present, Solver, 5e-8, "Tiny unbounded knapsack N≤6 (Solver)"),
    cell!(258, "linear_interp_table", Signal, L1, Present, Solver, 5e-11, "Piecewise-linear table interp (Solver)"),
    cell!(259, "physical_settle", Optimization, L1, Gap, None, 0.0, "Reserved QI/thermo settle-certify cell — not wired as silicon"),
    cell!(260, "reversible_rewrite", Arithmetic, L0, Gap, None, 0.0, "Reversible / adiabatic rewrite primitive — empty cell"),
    cell!(261, "ising_bind", Optimization, L1, Gap, None, 0.0, "Energy-function / Ising bind — empty cell"),
    cell!(262, "adiabatic_schedule", Signal, L1, Gap, None, 0.0, "Adiabatic anneal schedule driver — empty cell"),
    cell!(263, "ferric_efa_cert", Constraint, L1, Gap, None, 0.0, "On-device Ferric EFA certificate — out of proof scope"),
    cell!(264, "quantum_gate_ops", LinearAlgebra, L0, Gap, None, 0.0, "Thesis TEN quantum gate ops — empty / HW emerging"),
    cell!(265, "analog_crossbar_mac", LinearAlgebra, L1, Gap, None, 0.0, "Thesis ANALOG crossbar MAC — empty / emerging"),
    cell!(266, "photonic_mzi", Signal, L0, Gap, None, 0.0, "Thesis ANALOG photonic MZI — empty / emerging"),
];

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn live_catalog_honesty() {
        let s = PeriodicStack::subset();
        assert!(s.live_gear_count() >= 250, "live gears={}", s.live_gear_count());
        assert!(s.live_gear_count_of(GearKind::Lookup) >= 80);
        assert!(s.live_gear_count_of(GearKind::Formula) >= 70);
        assert!(s.live_gear_count_of(GearKind::Solver) >= 70);
        assert_eq!(s.placeholder_present_count(), 0, "no placeholders — residual policy is live Lookup");
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
        assert_eq!(s.present_count(), 258, "full thesis Present={}", s.present_count());
        assert_eq!(s.live_gear_count(), 258);
        assert!(s.gap_count() >= 4);
        assert_eq!(s.remaining_to_full(), 0, "remain={}", s.remaining_to_full());
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
