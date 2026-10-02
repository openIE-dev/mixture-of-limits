//! O(1) Bloom filter + exact HashMap LUT for grammar / resolution hits.
//!
//! Meta-routing must stay strictly cheaper than the smallest allowed inference
//! leaf. A Bloom pre-check + HashMap exact hit never invokes the model.

use std::collections::HashMap;

/// Tiny FNV-1a based Bloom filter (no external deps).
#[derive(Debug, Clone)]
pub struct BloomFilter {
    bits: Vec<u64>,
    /// Number of hash probes.
    _k: u32,
    nbits: usize,
}

impl BloomFilter {
    /// Capacity hint → bit width (rounded up to 64).
    pub fn with_capacity(expected: usize) -> Self {
        // ~10 bits/item, k=3 — false-positive low for MVP LUT sizes.
        let nbits = ((expected.max(8) * 10).div_ceil(64) * 64).max(64);
        Self {
            bits: vec![0u64; nbits / 64],
            _k: 3,
            nbits,
        }
    }

    fn hashes(&self, key: &str) -> [u64; 3] {
        let mut h = 0xcbf29ce484222325u64;
        for b in key.as_bytes() {
            h ^= u64::from(*b);
            h = h.wrapping_mul(0x100000001b3);
        }
        let h2 = h.wrapping_mul(0x9e3779b97f4a7c15).rotate_left(13);
        let h3 = h2.wrapping_mul(0xbf58476d1ce4e5b9).rotate_left(17);
        [h, h2, h3]
    }

    /// Insert key.
    pub fn insert(&mut self, key: &str) {
        for h in self.hashes(key) {
            let i = (h as usize) % self.nbits;
            self.bits[i / 64] |= 1u64 << (i % 64);
        }
    }

    /// May-contain (false positives possible; no false negatives).
    pub fn may_contain(&self, key: &str) -> bool {
        for h in self.hashes(key) {
            let i = (h as usize) % self.nbits;
            if self.bits[i / 64] & (1u64 << (i % 64)) == 0 {
                return false;
            }
        }
        true
    }
}

/// Exact O(1) resolution-code LUT with Bloom prefilter.
#[derive(Debug, Clone)]
pub struct ResolutionLut {
    bloom: BloomFilter,
    /// Normalized key → resolution answer text.
    map: HashMap<String, String>,
}

impl Default for ResolutionLut {
    fn default() -> Self {
        Self::support_desk_demo()
    }
}

impl ResolutionLut {
    /// Empty LUT.
    pub fn new() -> Self {
        Self {
            bloom: BloomFilter::with_capacity(32),
            map: HashMap::new(),
        }
    }

    /// Support-desk MVP resolution codes (A1 grammar hit → no model).
    pub fn support_desk_demo() -> Self {
        let mut lut = Self::new();
        let rows = [
            ("R-OK", "Ticket closed: resolution R-OK (resolved as designed)"),
            ("R-DUP", "Ticket closed: resolution R-DUP (duplicate of existing)"),
            ("R-HOWTO", "Ticket closed: resolution R-HOWTO (answered from knowledge base)"),
            ("R-BUGFIX", "Ticket closed: resolution R-BUGFIX (fix shipped)"),
            ("R-WONTFIX", "Ticket closed: resolution R-WONTFIX (out of scope)"),
            ("R-REFUND", "Ticket closed: resolution R-REFUND (billing adjustment)"),
            // Second chore (financial_risk_scoring) golden rows — same O(1) path.
            ("RISK-LOW", "Risk score band LOW (lookup table; model cold)"),
            ("RISK-MED", "Risk score band MED (lookup table; model cold)"),
            ("RISK-HIGH", "Risk score band HIGH (lookup table; model cold)"),
            // Arena-shaped typed decision (known option set → O(1); model cold).
            ("D-APPROVE", "Typed decision: APPROVE (option-set LUT; model cold)"),
            ("D-DENY", "Typed decision: DENY (option-set LUT; model cold)"),
            ("D-ESCALATE", "Typed decision: ESCALATE (option-set LUT; model cold)"),
        ];
        for (code, ans) in rows {
            lut.insert(code, ans);
        }
        lut
    }

    /// Financial risk sketch: score-band LUT rows.
    pub fn risk_score_demo() -> Self {
        let mut lut = Self::new();
        let rows = [
            ("RISK-LOW", "Risk score band LOW (lookup table; model cold)"),
            ("RISK-MED", "Risk score band MED (lookup table; model cold)"),
            ("RISK-HIGH", "Risk score band HIGH (lookup table; model cold)"),
        ];
        for (code, ans) in rows {
            lut.insert(code, ans);
        }
        lut
    }

    /// Insert code → answer (case-normalized).
    pub fn insert(&mut self, code: &str, answer: impl Into<String>) {
        let key = normalize_code(code);
        self.bloom.insert(&key);
        self.map.insert(key, answer.into());
    }

    /// Bloom may-contain (cheap reject).
    pub fn bloom_may_hit(&self, raw: &str) -> bool {
        let key = normalize_code(raw);
        if key.is_empty() {
            return false;
        }
        self.bloom.may_contain(&key)
    }

    /// Exact HashMap get after Bloom.
    pub fn lookup(&self, raw: &str) -> Option<&str> {
        let key = normalize_code(raw);
        if key.is_empty() || !self.bloom.may_contain(&key) {
            return None;
        }
        self.map.get(&key).map(|s| s.as_str())
    }

    /// Number of exact rows.
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Empty?
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Extract a resolution / risk code from free query text, then lookup.
    pub fn lookup_in_query(&self, query: &str) -> Option<(String, String)> {
        // Prefer explicit resolution=CODE / code=CODE
        let q = query.to_ascii_uppercase();
        for pref in [
            "RESOLUTION=",
            "RESOLUTION_CODE=",
            "CODE=",
            "BAND=",
            "DECISION=",
            "PICK=",
            "OPTION=",
        ] {
            if let Some(i) = q.find(pref) {
                let rest = &q[i + pref.len()..];
                let tok = rest
                    .split(|c: char| c.is_whitespace() || c == ',' || c == ';')
                    .next()
                    .unwrap_or("");
                if let Some(ans) = self.lookup(tok) {
                    return Some((normalize_code(tok), ans.to_string()));
                }
            }
        }
        // Bare tokens that look like R-* / RISK-* / D-*
        for tok in q.split(|c: char| !c.is_ascii_alphanumeric() && c != '-') {
            if tok.starts_with("R-") || tok.starts_with("RISK-") || tok.starts_with("D-") {
                if let Some(ans) = self.lookup(tok) {
                    return Some((normalize_code(tok), ans.to_string()));
                }
            }
        }
        None
    }
}

fn normalize_code(raw: &str) -> String {
    raw.trim()
        .trim_matches(|c: char| c == '"' || c == '\'')
        .to_ascii_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bloom_no_false_negative() {
        let mut b = BloomFilter::with_capacity(16);
        b.insert("R-OK");
        assert!(b.may_contain("R-OK"));
        assert!(!b.may_contain("R-MISSING-XYZ"));
    }

    #[test]
    fn lut_hit_o1() {
        let lut = ResolutionLut::support_desk_demo();
        assert!(lut.lookup("r-ok").is_some());
        assert!(lut.lookup_in_query("ticket close resolution=R-HOWTO").is_some());
        assert!(lut.lookup("R-NOPE").is_none());
    }
}
