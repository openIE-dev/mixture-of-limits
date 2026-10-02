//! Live Periodic Stack catalog gears — real Lookup / Formula / Solver closes.
//!
//! Present cells with [`mol_core::GearKind`] bindings execute here. Placeholders
//! (`GearKind::None`) and Gaps do **not** close. Counts stay honest vs 258/33.

use std::collections::{BTreeSet, VecDeque};

use mol_core::{
    CascadeTier, GearKind, MolError, MolRequest, PeriodicStack, Result, BOLTZMANN_J_PER_K,
};

use crate::grammar::{GrammarCoverage, TierAnswer};

fn param(q: &str, key: &str) -> Option<f64> {
    let lower = q.to_ascii_lowercase();
    let key = key.to_ascii_lowercase();
    for sep in ['=', ':'] {
        let pat = format!("{key}{sep}");
        if let Some(i) = lower.find(&pat) {
            let rest = &q[i + pat.len()..];
            let tok = rest
                .split(|c: char| c.is_whitespace() || c == ',' || c == ';' || c == ']')
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

fn param_str<'a>(q: &'a str, key: &str) -> Option<&'a str> {
    let lower = q.to_ascii_lowercase();
    let key = key.to_ascii_lowercase();
    for sep in ['=', ':'] {
        let pat = format!("{key}{sep}");
        if let Some(i) = lower.find(&pat) {
            let rest = &q[i + pat.len()..];
            let tok = rest
                .split(|c: char| c.is_whitespace() || c == ',' || c == ';')
                .next()
                .unwrap_or("")
                .trim()
                .trim_matches('"')
                .trim_matches('\'');
            if !tok.is_empty() {
                // Preserve original slice length from lower offset — use lower indices carefully.
                // Re-find on original with same pattern length.
                let orig_pat = format!("{key}{sep}");
                if let Some(j) = q.to_ascii_lowercase().find(&orig_pat) {
                    let r = &q[j + orig_pat.len()..];
                    let t = r
                        .split(|c: char| c.is_whitespace() || c == ',' || c == ';')
                        .next()
                        .unwrap_or("")
                        .trim()
                        .trim_matches('"')
                        .trim_matches('\'');
                    if !t.is_empty() {
                        return Some(t);
                    }
                }
                return Some(tok);
            }
        }
    }
    None
}

fn list_f64(q: &str, key: &str) -> Option<Vec<f64>> {
    let lower = q.to_ascii_lowercase();
    for sep in ['=', ':'] {
        let pat = format!("{}{sep}", key.to_ascii_lowercase());
        if let Some(i) = lower.find(&pat) {
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

fn list_str(q: &str, key: &str) -> Option<Vec<String>> {
    let lower = q.to_ascii_lowercase();
    for sep in ['=', ':'] {
        let pat = format!("{}{sep}", key.to_ascii_lowercase());
        if let Some(i) = lower.find(&pat) {
            let rest = &q[i + pat.len()..];
            let start = rest.find('[')?;
            let end = rest[start..].find(']')? + start;
            let inner = &rest[start + 1..end];
            let items: Vec<String> = inner
                .split(',')
                .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
                .filter(|s| !s.is_empty())
                .collect();
            if !items.is_empty() {
                return Some(items);
            }
        }
    }
    None
}

fn mentions(q: &str, name: &str) -> bool {
    let q = q.to_ascii_lowercase();
    let n = name.to_ascii_lowercase();
    // Whole-token match so sum_f64 does not steal cumsum_f64 (etc.).
    let token_hit = |hay: &str, needle: &str| -> bool {
        let bytes = hay.as_bytes();
        let nb = needle.as_bytes();
        let mut start = 0usize;
        while let Some(rel) = hay[start..].find(needle) {
            let i = start + rel;
            let before_ok = i == 0 || {
                let b = bytes[i - 1];
                !b.is_ascii_alphanumeric() && b != b'_'
            };
            let after = i + nb.len();
            let after_ok = after >= bytes.len() || {
                let b = bytes[after];
                !b.is_ascii_alphanumeric() && b != b'_'
            };
            if before_ok && after_ok {
                return true;
            }
            start = i + 1;
            if start >= hay.len() {
                break;
            }
        }
        false
    };
    token_hit(&q, &n)
        || token_hit(&q, &format!("primitive {n}"))
        || token_hit(&q, &format!("catalog {n}"))
        || token_hit(&q, &format!("stack {n}"))
}

fn tag(name: &str, gear: &str, body: String) -> String {
    format!("{body} [live_catalog primitive={name} gear={gear}; soft-ref; estimates≠measured_j]")
}

fn parse_bool_token(s: &str) -> Option<bool> {
    match s.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "t" | "yes" | "y" => Some(true),
        "0" | "false" | "f" | "no" | "n" => Some(false),
        _ => None,
    }
}

fn fnv1a64(s: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.as_bytes() {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

fn sha256_hex(s: &str) -> String {
    // Minimal SHA-256 (public domain style) — enough for soft-ref catalog.
    sha256::digest(s.as_bytes())
}


fn b64_encode(s: &str) -> String {
    const T: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let bytes = s.as_bytes();
    let mut out = String::new();
    let mut i = 0;
    while i < bytes.len() {
        let b0 = bytes[i] as u32;
        let b1 = if i + 1 < bytes.len() { bytes[i + 1] as u32 } else { 0 };
        let b2 = if i + 2 < bytes.len() { bytes[i + 2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        out.push(T[((triple >> 18) & 63) as usize] as char);
        out.push(T[((triple >> 12) & 63) as usize] as char);
        if i + 1 < bytes.len() {
            out.push(T[((triple >> 6) & 63) as usize] as char);
        } else {
            out.push('=');
        }
        if i + 2 < bytes.len() {
            out.push(T[(triple & 63) as usize] as char);
        } else {
            out.push('=');
        }
        i += 3;
    }
    out
}

fn b64_decode(s: &str) -> Option<String> {
    fn val(c: u8) -> Option<u8> {
        Some(match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => return None,
        })
    }
    let clean: Vec<u8> = s.bytes().filter(|b| !b.is_ascii_whitespace()).collect();
    if clean.len() % 4 != 0 {
        return None;
    }
    let mut out = Vec::new();
    for chunk in clean.chunks(4) {
        let (c0, c1, c2, c3) = (chunk[0], chunk[1], chunk[2], chunk[3]);
        let v0 = val(c0)?;
        let v1 = val(c1)?;
        let v2 = if c2 == b'=' { 0 } else { val(c2)? };
        let v3 = if c3 == b'=' { 0 } else { val(c3)? };
        let triple = ((v0 as u32) << 18) | ((v1 as u32) << 12) | ((v2 as u32) << 6) | (v3 as u32);
        out.push(((triple >> 16) & 255) as u8);
        if c2 != b'=' {
            out.push(((triple >> 8) & 255) as u8);
        }
        if c3 != b'=' {
            out.push((triple & 255) as u8);
        }
    }
    String::from_utf8(out).ok()
}


mod sha256 {
    // Compact SHA-256 for catalog Lookup (no extra crate dep).
    pub fn digest(data: &[u8]) -> String {
        let mut h: [u32; 8] = [
            0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
            0x5be0cd19,
        ];
        let mut msg = data.to_vec();
        let bit_len = (data.len() as u64) * 8;
        msg.push(0x80);
        while (msg.len() % 64) != 56 {
            msg.push(0);
        }
        msg.extend_from_slice(&bit_len.to_be_bytes());
        const K: [u32; 64] = [
            0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
            0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
            0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
            0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
            0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
            0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
            0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
            0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
            0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
            0xc67178f2,
        ];
        for chunk in msg.chunks(64) {
            let mut w = [0u32; 64];
            for i in 0..16 {
                w[i] = u32::from_be_bytes([
                    chunk[i * 4],
                    chunk[i * 4 + 1],
                    chunk[i * 4 + 2],
                    chunk[i * 4 + 3],
                ]);
            }
            for i in 16..64 {
                let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
                let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
                w[i] = w[i - 16]
                    .wrapping_add(s0)
                    .wrapping_add(w[i - 7])
                    .wrapping_add(s1);
            }
            let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh) =
                (h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]);
            for i in 0..64 {
                let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
                let ch = (e & f) ^ ((!e) & g);
                let t1 = hh
                    .wrapping_add(s1)
                    .wrapping_add(ch)
                    .wrapping_add(K[i])
                    .wrapping_add(w[i]);
                let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
                let maj = (a & b) ^ (a & c) ^ (b & c);
                let t2 = s0.wrapping_add(maj);
                hh = g;
                g = f;
                f = e;
                e = d.wrapping_add(t1);
                d = c;
                c = b;
                b = a;
                a = t1.wrapping_add(t2);
            }
            h[0] = h[0].wrapping_add(a);
            h[1] = h[1].wrapping_add(b);
            h[2] = h[2].wrapping_add(c);
            h[3] = h[3].wrapping_add(d);
            h[4] = h[4].wrapping_add(e);
            h[5] = h[5].wrapping_add(f);
            h[6] = h[6].wrapping_add(g);
            h[7] = h[7].wrapping_add(hh);
        }
        h.iter().map(|x| format!("{x:08x}")).collect()
    }
}

fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut cur = vec![0; b.len() + 1];
    for i in 1..=a.len() {
        cur[0] = i;
        for j in 1..=b.len() {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            cur[j] = (prev[j] + 1).min(cur[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[b.len()]
}

/// Live Formula catalog gear.
#[derive(Debug, Default, Clone)]
pub struct CatalogFormula;

impl CatalogFormula {
    fn eval(query: &str) -> Option<String> {
        let q = query.to_ascii_lowercase();
        let stack = PeriodicStack::subset();

        // Guard: only live Formula cells (or well-known aliases already Formula-tagged).
        let try_name = |name: &str| -> bool {
            mentions(&q, name)
                && stack
                    .get_by_name(name)
                    .map(|c| c.gear == GearKind::Formula)
                    .unwrap_or(false)
        };

        if try_name("add_f64") {
            let a = param(&q, "a")?;
            let b = param(&q, "b")?;
            return Some(tag("add_f64", "formula", format!("add_f64: {a} + {b} = {}", a + b)));
        }
        if try_name("mul_f64") {
            let a = param(&q, "a")?;
            let b = param(&q, "b")?;
            return Some(tag("mul_f64", "formula", format!("mul_f64: {a} * {b} = {}", a * b)));
        }
        if try_name("div_f64") {
            let a = param(&q, "a")?;
            let b = param(&q, "b")?;
            if b.abs() < 1e-15 {
                return Some(tag("div_f64", "formula", "div_f64: refuse divide-by-zero".into()));
            }
            return Some(tag("div_f64", "formula", format!("div_f64: {a} / {b} = {}", a / b)));
        }
        if try_name("pow_f64") {
            let a = param(&q, "a")?;
            let b = param(&q, "b")?;
            return Some(tag("pow_f64", "formula", format!("pow_f64: {a}^{b} = {}", a.powf(b))));
        }
        if try_name("pythagoras") {
            let a = param(&q, "a")?;
            let b = param(&q, "b")?;
            let c = (a * a + b * b).sqrt();
            return Some(tag("pythagoras", "formula", format!("pythagoras: c=sqrt({a}²+{b}²)={c}")));
        }
        if try_name("ohms_law") {
            if let (Some(i), Some(r)) = (param(&q, "i").or_else(|| param(&q, "current")), param(&q, "r").or_else(|| param(&q, "resistance"))) {
                return Some(tag("ohms_law", "formula", format!("ohms_law: V=I·R={i}·{r}={}", i * r)));
            }
            if let (Some(v), Some(r)) = (param(&q, "v").or_else(|| param(&q, "voltage")), param(&q, "r")) {
                return Some(tag("ohms_law", "formula", format!("ohms_law: I=V/R={v}/{r}={}", v / r)));
            }
        }
        if try_name("kinetic_energy") {
            let m = param(&q, "m").or_else(|| param(&q, "mass"))?;
            let v = param(&q, "v").or_else(|| param(&q, "velocity"))?;
            let ke = 0.5 * m * v * v;
            return Some(tag("kinetic_energy", "formula", format!("kinetic_energy: ½·{m}·{v}²={ke}")));
        }
        if try_name("ideal_gas") {
            // Solve for missing among P,V,n,T with R=8.314462618
            const R: f64 = 8.314_462_618;
            let p = param(&q, "p").or_else(|| param(&q, "pressure"));
            let v = param(&q, "v").or_else(|| param(&q, "volume"));
            let n = param(&q, "n").or_else(|| param(&q, "moles"));
            let t = param(&q, "t").or_else(|| param(&q, "temp")).or_else(|| param(&q, "kelvin"));
            let ans = match (p, v, n, t) {
                (None, Some(v), Some(n), Some(t)) => format!("ideal_gas: P=nRT/V={n}·{R}·{t}/{v}={}", n * R * t / v),
                (Some(p), None, Some(n), Some(t)) => format!("ideal_gas: V=nRT/P={n}·{R}·{t}/{p}={}", n * R * t / p),
                (Some(p), Some(v), None, Some(t)) => format!("ideal_gas: n=PV/RT={p}·{v}/({R}·{t})={}", p * v / (R * t)),
                (Some(p), Some(v), Some(n), None) => format!("ideal_gas: T=PV/nR={p}·{v}/({n}·{R})={}", p * v / (n * R)),
                (Some(p), Some(v), Some(n), Some(t)) => format!("ideal_gas: check PV={} nRT={} (R={R})", p * v, n * R * t),
                _ => return None,
            };
            return Some(tag("ideal_gas", "formula", ans));
        }
        if try_name("coulomb") {
            const K: f64 = 8.987_551_792_3e9;
            let q1 = param(&q, "q1")?;
            let q2 = param(&q, "q2")?;
            let r = param(&q, "r")?;
            if r.abs() < 1e-15 {
                return Some(tag("coulomb", "formula", "coulomb: refuse r=0".into()));
            }
            let f = K * q1 * q2 / (r * r);
            return Some(tag("coulomb", "formula", format!("coulomb: F=k·q1·q2/r²={f}")));
        }
        if try_name("planck_e") {
            const H: f64 = 6.626_070_15e-34;
            let f = param(&q, "f").or_else(|| param(&q, "freq")).or_else(|| param(&q, "hz"))?;
            return Some(tag("planck_e", "formula", format!("planck_e: E=h·f={H}·{f}={}", H * f)));
        }
        if try_name("boltzmann_factor") {
            let e = param(&q, "e").or_else(|| param(&q, "energy"))?;
            let t = param(&q, "t").or_else(|| param(&q, "kelvin")).unwrap_or(300.0);
            let bf = (-e / (BOLTZMANN_J_PER_K * t)).exp();
            return Some(tag(
                "boltzmann_factor",
                "formula",
                format!("boltzmann_factor: exp(-E/kT)=exp(-{e}/({BOLTZMANN_J_PER_K}·{t}))={bf}"),
            ));
        }
        if try_name("snr_db") {
            let s = param(&q, "s").or_else(|| param(&q, "signal"))?;
            let n = param(&q, "n").or_else(|| param(&q, "noise"))?;
            if n <= 0.0 || s <= 0.0 {
                return None;
            }
            let db = 10.0 * (s / n).log10();
            return Some(tag("snr_db", "formula", format!("snr_db: 10·log10({s}/{n})={db}")));
        }
        if try_name("compound_interest") {
            let p = param(&q, "p").or_else(|| param(&q, "principal"))?;
            let r = param(&q, "r").or_else(|| param(&q, "rate"))?;
            let n = param(&q, "n").or_else(|| param(&q, "periods"))?;
            let a = p * (1.0 + r).powf(n);
            return Some(tag("compound_interest", "formula", format!("compound_interest: A=P(1+r)^n={a}")));
        }
        if try_name("gaussian_pdf") {
            let x = param(&q, "x")?;
            let mu = param(&q, "mu").unwrap_or(0.0);
            let sigma = param(&q, "sigma").unwrap_or(1.0);
            if sigma <= 0.0 {
                return None;
            }
            let z = (x - mu) / sigma;
            let pdf = (-0.5 * z * z).exp() / (sigma * (2.0 * std::f64::consts::PI).sqrt());
            return Some(tag("gaussian_pdf", "formula", format!("gaussian_pdf: φ({x};{mu},{sigma})={pdf}")));
        }
        if try_name("wave_lambda") {
            const C: f64 = 299_792_458.0;
            let f = param(&q, "f").or_else(|| param(&q, "freq")).or_else(|| param(&q, "hz"))?;
            if f == 0.0 {
                return None;
            }
            return Some(tag("wave_lambda", "formula", format!("wave_lambda: λ=c/f={C}/{f}={}", C / f)));
        }
        if try_name("logit") {
            let p = param(&q, "p")?;
            if p <= 0.0 || p >= 1.0 {
                return Some(tag("logit", "formula", "logit: refuse p∉(0,1)".into()));
            }
            return Some(tag("logit", "formula", format!("logit: ln({p}/(1-{p}))={}", (p / (1.0 - p)).ln())));
        }
        if try_name("sigmoid") {
            let x = param(&q, "x")?;
            let s = 1.0 / (1.0 + (-x).exp());
            return Some(tag("sigmoid", "formula", format!("sigmoid: σ({x})={s}")));
        }
        if try_name("entropy_bits") {
            let ps = list_f64(&q, "p").or_else(|| list_f64(&q, "probs"))?;
            let sum: f64 = ps.iter().sum();
            if (sum - 1.0).abs() > 1e-6 || ps.iter().any(|p| *p < 0.0) {
                return Some(tag("entropy_bits", "formula", "entropy_bits: refuse — probs must be ≥0 and sum≈1".into()));
            }
            let h: f64 = ps
                .iter()
                .filter(|p| **p > 0.0)
                .map(|p| -p * p.log2())
                .sum();
            return Some(tag("entropy_bits", "formula", format!("entropy_bits: H={h} bits")));
        }
        if try_name("det_2x2") {
            let a = param(&q, "a")?;
            let b = param(&q, "b")?;
            let c = param(&q, "c")?;
            let d = param(&q, "d")?;
            return Some(tag("det_2x2", "formula", format!("det_2x2: ad-bc={}-{}={}", a * d, b * c, a * d - b * c)));
        }
        if try_name("lerp") {
            let a = param(&q, "a")?;
            let b = param(&q, "b")?;
            let t = param(&q, "t")?;
            let y = (1.0 - t) * a + t * b;
            return Some(tag("lerp", "formula", format!("lerp: (1-{t})·{a}+{t}·{b}={y}")));
        }
        if try_name("rest_energy") {
            const C: f64 = 299_792_458.0;
            let m = param(&q, "m").or_else(|| param(&q, "mass"))?;
            let e = m * C * C;
            return Some(tag("rest_energy", "formula", format!("rest_energy: E=mc²={m}·c²={e}")));
        }
        if try_name("bit_bound") {
            let n = param(&q, "n").or_else(|| param(&q, "symbols"))?;
            let m = param(&q, "m").or_else(|| param(&q, "alphabet")).unwrap_or(2.0);
            if n < 0.0 || m < 2.0 {
                return Some(tag("bit_bound", "formula", "bit_bound: refuse n<0 or alphabet<2".into()));
            }
            let bits = n * m.log2();
            return Some(tag("bit_bound", "formula", format!("bit_bound: N·log2(M)={n}·log2({m})={bits}")));
        }
        if try_name("area_circle") {
            let r = param(&q, "r").or_else(|| param(&q, "radius"))?;
            let a = std::f64::consts::PI * r * r;
            return Some(tag("area_circle", "formula", format!("area_circle: π·{r}²={a}")));
        }
        if try_name("circumference") {
            let r = param(&q, "r").or_else(|| param(&q, "radius"))?;
            let c = 2.0 * std::f64::consts::PI * r;
            return Some(tag("circumference", "formula", format!("circumference: 2π·{r}={c}")));
        }
        if try_name("volume_sphere") {
            let r = param(&q, "r").or_else(|| param(&q, "radius"))?;
            let v = 4.0 / 3.0 * std::f64::consts::PI * r * r * r;
            return Some(tag("volume_sphere", "formula", format!("volume_sphere: (4/3)π·{r}³={v}")));
        }
        if try_name("heron_area") {
            let a = param(&q, "a")?;
            let b = param(&q, "b")?;
            let c = param(&q, "c")?;
            let s = 0.5 * (a + b + c);
            let area2 = s * (s - a) * (s - b) * (s - c);
            if area2 < 0.0 {
                return Some(tag("heron_area", "formula", "heron_area: refuse non-triangle".into()));
            }
            return Some(tag("heron_area", "formula", format!("heron_area: sqrt(s(s-a)(s-b)(s-c))={}", area2.sqrt())));
        }
        if try_name("quadratic_roots") {
            let a = param(&q, "a")?;
            let b = param(&q, "b")?;
            let c = param(&q, "c")?;
            if a.abs() < 1e-15 {
                return Some(tag("quadratic_roots", "formula", "quadratic_roots: refuse a=0".into()));
            }
            let disc = b * b - 4.0 * a * c;
            if disc < 0.0 {
                return Some(tag("quadratic_roots", "formula", format!("quadratic_roots: complex disc={disc}")));
            }
            let s = disc.sqrt();
            let r1 = (-b + s) / (2.0 * a);
            let r2 = (-b - s) / (2.0 * a);
            return Some(tag("quadratic_roots", "formula", format!("quadratic_roots: r1={r1} r2={r2}")));
        }
        if try_name("softplus") {
            let x = param(&q, "x")?;
            let y = if x > 20.0 { x } else { (1.0 + x.exp()).ln() };
            return Some(tag("softplus", "formula", format!("softplus: ln(1+e^{x})={y}")));
        }
        if try_name("percent_change") {
            let old = param(&q, "old").or_else(|| param(&q, "from"))?;
            let newv = param(&q, "new").or_else(|| param(&q, "to"))?;
            if old.abs() < 1e-15 {
                return Some(tag("percent_change", "formula", "percent_change: refuse old=0".into()));
            }
            let pct = 100.0 * (newv - old) / old;
            return Some(tag("percent_change", "formula", format!("percent_change: {pct}%")));
        }
        if try_name("capacitor_energy") {
            let c = param(&q, "c").or_else(|| param(&q, "capacitance"))?;
            let v = param(&q, "v").or_else(|| param(&q, "voltage"))?;
            let e = 0.5 * c * v * v;
            return Some(tag("capacitor_energy", "formula", format!("capacitor_energy: ½·{c}·{v}²={e}")));
        }
        if try_name("freefall_distance") {
            let t = param(&q, "t").or_else(|| param(&q, "time"))?;
            let g = param(&q, "g").unwrap_or(9.80665);
            let d = 0.5 * g * t * t;
            return Some(tag("freefall_distance", "formula", format!("freefall_distance: ½·{g}·{t}²={d}")));
        }
        if try_name("geometric_mean") {
            let xs = list_f64(&q, "xs").or_else(|| list_f64(&q, "values"))?;
            if xs.iter().any(|x| *x <= 0.0) {
                return Some(tag("geometric_mean", "formula", "geometric_mean: refuse non-positive".into()));
            }
            let log_sum: f64 = xs.iter().map(|x| x.ln()).sum();
            let g = (log_sum / xs.len() as f64).exp();
            return Some(tag("geometric_mean", "formula", format!("geometric_mean: {g}")));
        }
        if try_name("harmonic_mean") {
            let xs = list_f64(&q, "xs").or_else(|| list_f64(&q, "values"))?;
            if xs.iter().any(|x| *x <= 0.0) {
                return Some(tag("harmonic_mean", "formula", "harmonic_mean: refuse non-positive".into()));
            }
            let inv: f64 = xs.iter().map(|x| 1.0 / x).sum();
            let h = xs.len() as f64 / inv;
            return Some(tag("harmonic_mean", "formula", format!("harmonic_mean: {h}")));
        }
        if try_name("beer_lambert") {
            let eps = param(&q, "eps").or_else(|| param(&q, "epsilon")).or_else(|| param(&q, "absorptivity"))?;
            let c = param(&q, "c").or_else(|| param(&q, "concentration"))?;
            let l = param(&q, "l").or_else(|| param(&q, "path")).or_else(|| param(&q, "length"))?;
            let a = eps * c * l;
            return Some(tag("beer_lambert", "formula", format!("beer_lambert: A=ε·c·l={eps}·{c}·{l}={a}")));
        }
        if try_name("snell_law") {
            let n1 = param(&q, "n1")?;
            let n2 = param(&q, "n2")?;
            if let Some(t1) = param(&q, "theta1").or_else(|| param(&q, "t1")) {
                let s = n1 * t1.to_radians().sin() / n2;
                if s.abs() > 1.0 {
                    return Some(tag("snell_law", "formula", "snell_law: total internal reflection".into()));
                }
                let t2 = s.asin().to_degrees();
                return Some(tag("snell_law", "formula", format!("snell_law: θ2={t2} deg")));
            }
            if let Some(t2) = param(&q, "theta2").or_else(|| param(&q, "t2")) {
                let s = n2 * t2.to_radians().sin() / n1;
                if s.abs() > 1.0 {
                    return Some(tag("snell_law", "formula", "snell_law: total internal reflection".into()));
                }
                let t1 = s.asin().to_degrees();
                return Some(tag("snell_law", "formula", format!("snell_law: θ1={t1} deg")));
            }
            return None;
        }

        if try_name("area_rectangle") {
            let w = param(&q, "w").or_else(|| param(&q, "width"))?;
            let h = param(&q, "h").or_else(|| param(&q, "height"))?;
            return Some(tag("area_rectangle", "formula", format!("area_rectangle: A=w·h={w}·{h}={}", w * h)));
        }
        if try_name("area_triangle") {
            let b = param(&q, "b").or_else(|| param(&q, "base"))?;
            let h = param(&q, "h").or_else(|| param(&q, "height"))?;
            return Some(tag("area_triangle", "formula", format!("area_triangle: A=½bh=0.5·{b}·{h}={}", 0.5 * b * h)));
        }
        if try_name("volume_cylinder") {
            let r = param(&q, "r").or_else(|| param(&q, "radius"))?;
            let h = param(&q, "h").or_else(|| param(&q, "height"))?;
            let v = std::f64::consts::PI * r * r * h;
            return Some(tag("volume_cylinder", "formula", format!("volume_cylinder: V=πr²h={v}")));
        }
        if try_name("potential_energy") {
            let m = param(&q, "m").or_else(|| param(&q, "mass"))?;
            let g = param(&q, "g").unwrap_or(9.80665);
            let h = param(&q, "h").or_else(|| param(&q, "height"))?;
            return Some(tag("potential_energy", "formula", format!("potential_energy: PE=mgh={m}·{g}·{h}={}", m * g * h)));
        }
        if try_name("centripetal_accel") {
            let v = param(&q, "v").or_else(|| param(&q, "speed"))?;
            let r = param(&q, "r").or_else(|| param(&q, "radius"))?;
            if r == 0.0 { return None; }
            return Some(tag("centripetal_accel", "formula", format!("centripetal_accel: a=v²/r={}/{r}={}", v * v, v * v / r)));
        }
        if try_name("bmi_formula") {
            let kg = param(&q, "kg").or_else(|| param(&q, "mass")).or_else(|| param(&q, "m"))?;
            let m = param(&q, "m_height").or_else(|| param(&q, "height")).or_else(|| param(&q, "h"))?;
            if m == 0.0 { return None; }
            return Some(tag("bmi_formula", "formula", format!("bmi_formula: BMI={kg}/{m}²={}", kg / (m * m))));
        }
        if try_name("abs_f64") {
            let x = param(&q, "x").or_else(|| param(&q, "a"))?;
            return Some(tag("abs_f64", "formula", format!("abs_f64: |{x}|={}", x.abs())));
        }
        if try_name("log2_f64") {
            let x = param(&q, "x").or_else(|| param(&q, "a"))?;
            if x <= 0.0 { return None; }
            return Some(tag("log2_f64", "formula", format!("log2_f64: log2({x})={}", x.log2())));
        }
        if try_name("exp_f64") {
            let x = param(&q, "x").or_else(|| param(&q, "a"))?;
            return Some(tag("exp_f64", "formula", format!("exp_f64: e^{x}={}", x.exp())));
        }
        if try_name("relative_error") {
            let approx = param(&q, "approx").or_else(|| param(&q, "a"))?;
            let truth = param(&q, "true").or_else(|| param(&q, "t")).or_else(|| param(&q, "exact"))?;
            if truth == 0.0 { return None; }
            let e = (approx - truth).abs() / truth.abs();
            return Some(tag("relative_error", "formula", format!("relative_error: |{approx}-{truth}|/|{truth}|={e}")));
        }
        if try_name("wien_peak") {
            let t = param(&q, "t").or_else(|| param(&q, "temp")).or_else(|| param(&q, "kelvin"))?;
            if t <= 0.0 { return None; }
            const B: f64 = 2.897771955e-3; // m·K
            let lam = B / t;
            return Some(tag("wien_peak", "formula", format!("wien_peak: λ_max=b/T={B}/{t}={lam} m")));
        }
        if try_name("reynolds_number") {
            let rho = param(&q, "rho").or_else(|| param(&q, "density"))?;
            let v = param(&q, "v").or_else(|| param(&q, "speed"))?;
            let l = param(&q, "l").or_else(|| param(&q, "length")).or_else(|| param(&q, "L"))?;
            let mu = param(&q, "mu").or_else(|| param(&q, "viscosity"))?;
            if mu == 0.0 { return None; }
            let re = rho * v * l / mu;
            return Some(tag("reynolds_number", "formula", format!("reynolds_number: Re=ρvL/μ={re}")));
        }
        if try_name("escape_velocity") {
            let g = param(&q, "gm").or_else(|| param(&q, "mu")).or_else(|| param(&q, "GM"))?;
            let r = param(&q, "r").or_else(|| param(&q, "radius"))?;
            if r <= 0.0 { return None; }
            let v = (2.0 * g / r).sqrt();
            return Some(tag("escape_velocity", "formula", format!("escape_velocity: v_esc=sqrt(2GM/r)={v}")));
        }
        if try_name("circular_period") {
            let g = param(&q, "gm").or_else(|| param(&q, "mu")).or_else(|| param(&q, "GM"))?;
            let r = param(&q, "r").or_else(|| param(&q, "radius"))?;
            if g <= 0.0 || r <= 0.0 { return None; }
            let t = 2.0 * std::f64::consts::PI * (r * r * r / g).sqrt();
            return Some(tag("circular_period", "formula", format!("circular_period: T=2π√(r³/GM)={t}")));
        }
        if try_name("binomial_coeff") {
            let n = param(&q, "n")? as u64;
            let k = param(&q, "k")? as u64;
            if n > 66 || k > n { return None; }
            let mut c: u128 = 1;
            let kk = k.min(n - k);
            for i in 0..kk {
                c = c * (n - i) as u128 / (i + 1) as u128;
            }
            return Some(tag("binomial_coeff", "formula", format!("binomial_coeff: C({n},{k})={c}")));
        }

        if try_name("sqrt_f64") {
            let x = param(&q, "x").or_else(|| param(&q, "a"))?;
            if x < 0.0 { return Some(tag("sqrt_f64", "formula", "sqrt_f64: refuse x<0".into())); }
            return Some(tag("sqrt_f64", "formula", format!("sqrt_f64: √{x}={}", x.sqrt())));
        }
        if try_name("ln_f64") {
            let x = param(&q, "x").or_else(|| param(&q, "a"))?;
            if x <= 0.0 { return Some(tag("ln_f64", "formula", "ln_f64: refuse x≤0".into())); }
            return Some(tag("ln_f64", "formula", format!("ln_f64: ln({x})={}", x.ln())));
        }
        if try_name("sin_f64") {
            let x = param(&q, "x").or_else(|| param(&q, "a"))?;
            return Some(tag("sin_f64", "formula", format!("sin_f64: sin({x})={}", x.sin())));
        }
        if try_name("cos_f64") {
            let x = param(&q, "x").or_else(|| param(&q, "a"))?;
            return Some(tag("cos_f64", "formula", format!("cos_f64: cos({x})={}", x.cos())));
        }
        if try_name("tan_f64") {
            let x = param(&q, "x").or_else(|| param(&q, "a"))?;
            return Some(tag("tan_f64", "formula", format!("tan_f64: tan({x})={}", x.tan())));
        }
        if try_name("hypot_f64") {
            let a = param(&q, "a").or_else(|| param(&q, "x"))?;
            let b = param(&q, "b").or_else(|| param(&q, "y"))?;
            return Some(tag("hypot_f64", "formula", format!("hypot_f64: √({a}²+{b}²)={}", a.hypot(b))));
        }
        if try_name("volume_cone") {
            let r = param(&q, "r").or_else(|| param(&q, "radius"))?;
            let h = param(&q, "h").or_else(|| param(&q, "height"))?;
            let v = std::f64::consts::PI * r * r * h / 3.0;
            return Some(tag("volume_cone", "formula", format!("volume_cone: V=⅓πr²h={v}")));
        }
        if try_name("area_trapezoid") {
            let a = param(&q, "a").or_else(|| param(&q, "base1"))?;
            let b = param(&q, "b").or_else(|| param(&q, "base2"))?;
            let h = param(&q, "h").or_else(|| param(&q, "height"))?;
            return Some(tag("area_trapezoid", "formula", format!("area_trapezoid: A=½({a}+{b})·{h}={}", 0.5 * (a + b) * h)));
        }
        if try_name("hookes_law") {
            let k = param(&q, "k").or_else(|| param(&q, "spring"))?;
            let x = param(&q, "x").or_else(|| param(&q, "disp"))?;
            return Some(tag("hookes_law", "formula", format!("hookes_law: F=-k·x=-{k}·{x}={}", -k * x)));
        }
        if try_name("work_force_dist") {
            let f = param(&q, "f").or_else(|| param(&q, "force"))?;
            let d = param(&q, "d").or_else(|| param(&q, "dist")).or_else(|| param(&q, "distance"))?;
            return Some(tag("work_force_dist", "formula", format!("work_force_dist: W=F·d={f}·{d}={}", f * d)));
        }
        if try_name("power_energy_time") {
            let e = param(&q, "e").or_else(|| param(&q, "energy"))?;
            let t = param(&q, "t").or_else(|| param(&q, "time"))?;
            if t == 0.0 { return Some(tag("power_energy_time", "formula", "power_energy_time: refuse t=0".into())); }
            return Some(tag("power_energy_time", "formula", format!("power_energy_time: P=E/t={e}/{t}={}", e / t)));
        }
        if try_name("density_mass_vol") {
            let m = param(&q, "m").or_else(|| param(&q, "mass"))?;
            let v = param(&q, "v").or_else(|| param(&q, "volume"))?;
            if v == 0.0 { return None; }
            return Some(tag("density_mass_vol", "formula", format!("density_mass_vol: ρ=m/V={m}/{v}={}", m / v)));
        }
        if try_name("pressure_force_area") {
            let f = param(&q, "f").or_else(|| param(&q, "force"))?;
            let a = param(&q, "a").or_else(|| param(&q, "area"))?;
            if a == 0.0 { return None; }
            return Some(tag("pressure_force_area", "formula", format!("pressure_force_area: P=F/A={f}/{a}={}", f / a)));
        }
        if try_name("gravitational_force") {
            const G: f64 = 6.67430e-11;
            let m1 = param(&q, "m1")?;
            let m2 = param(&q, "m2")?;
            let r = param(&q, "r")?;
            if r == 0.0 { return None; }
            let f = G * m1 * m2 / (r * r);
            return Some(tag("gravitational_force", "formula", format!("gravitational_force: F=G·m1·m2/r²={f}")));
        }
        if try_name("stefan_boltzmann") {
            const SIGMA: f64 = 5.670374419e-8;
            let t = param(&q, "t").or_else(|| param(&q, "temp")).or_else(|| param(&q, "kelvin"))?;
            let j = SIGMA * t.powi(4);
            return Some(tag("stefan_boltzmann", "formula", format!("stefan_boltzmann: j=σT⁴={j}")));
        }
        if try_name("arrhenius") {
            const R: f64 = 8.314462618;
            let a = param(&q, "a").or_else(|| param(&q, "A"))?;
            let ea = param(&q, "ea").or_else(|| param(&q, "Ea"))?;
            let t = param(&q, "t").or_else(|| param(&q, "temp"))?;
            if t <= 0.0 { return None; }
            let k = a * (-ea / (R * t)).exp();
            return Some(tag("arrhenius", "formula", format!("arrhenius: k=A·exp(-Ea/RT)={k}")));
        }
        if try_name("half_life") {
            let n0 = param(&q, "n0").or_else(|| param(&q, "n_zero")).unwrap_or(1.0);
            let t = param(&q, "t").or_else(|| param(&q, "time"))?;
            let th = param(&q, "half").or_else(|| param(&q, "t_half")).or_else(|| param(&q, "thalf"))?;
            if th <= 0.0 { return None; }
            let n = n0 * 0.5_f64.powf(t / th);
            return Some(tag("half_life", "formula", format!("half_life: N={n0}·(1/2)^({t}/{th})={n}")));
        }
        if try_name("beat_frequency") {
            let f1 = param(&q, "f1")?;
            let f2 = param(&q, "f2")?;
            return Some(tag("beat_frequency", "formula", format!("beat_frequency: |{f1}-{f2}|={}", (f1 - f2).abs())));
        }
        if try_name("capacitance_parallel") {
            let c1 = param(&q, "c1")?;
            let c2 = param(&q, "c2")?;
            return Some(tag("capacitance_parallel", "formula", format!("capacitance_parallel: C={c1}+{c2}={}", c1 + c2)));
        }
        if try_name("inductance_energy") {
            let l = param(&q, "l").or_else(|| param(&q, "L"))?;
            let i = param(&q, "i").or_else(|| param(&q, "current"))?;
            return Some(tag("inductance_energy", "formula", format!("inductance_energy: E=½LI²=0.5·{l}·{i}²={}", 0.5 * l * i * i)));
        }
        if try_name("refractive_index") {
            const C: f64 = 299_792_458.0;
            let v = param(&q, "v").or_else(|| param(&q, "speed"))?;
            if v <= 0.0 { return None; }
            return Some(tag("refractive_index", "formula", format!("refractive_index: n=c/v={C}/{v}={}", C / v)));
        }
        if try_name("doppler_shift") {
            let f = param(&q, "f").or_else(|| param(&q, "freq"))?;
            let v = param(&q, "v").or_else(|| param(&q, "medium")).unwrap_or(343.0);
            let vo = param(&q, "vo").unwrap_or(0.0);
            let vs = param(&q, "vs").unwrap_or(0.0);
            let den = v + vs;
            if den == 0.0 { return None; }
            let fp = f * (v + vo) / den;
            return Some(tag("doppler_shift", "formula", format!("doppler_shift: f'={fp}")));
        }
        if try_name("coulomb_potential") {
            const K: f64 = 8.987_551_792_3e9;
            let qq = param(&q, "q")?;
            let r = param(&q, "r")?;
            if r == 0.0 { return None; }
            return Some(tag("coulomb_potential", "formula", format!("coulomb_potential: V=kq/r={K}·{qq}/{r}={}", K * qq / r)));
        }
        if try_name("terminal_velocity") {
            let m = param(&q, "m").or_else(|| param(&q, "mass"))?;
            let g = param(&q, "g").unwrap_or(9.80665);
            let rho = param(&q, "rho").or_else(|| param(&q, "density"))?;
            let a = param(&q, "a").or_else(|| param(&q, "area"))?;
            let cd = param(&q, "cd").or_else(|| param(&q, "drag")).unwrap_or(1.0);
            let den = rho * a * cd;
            if den <= 0.0 { return None; }
            let v = (2.0 * m * g / den).sqrt();
            return Some(tag("terminal_velocity", "formula", format!("terminal_velocity: v={v}")));
        }
        if try_name("orbit_velocity") {
            let gm = param(&q, "gm").or_else(|| param(&q, "GM")).or_else(|| param(&q, "mu"))?;
            let r = param(&q, "r").or_else(|| param(&q, "radius"))?;
            if r <= 0.0 { return None; }
            return Some(tag("orbit_velocity", "formula", format!("orbit_velocity: v=√(GM/r)={}", (gm / r).sqrt())));
        }
        if try_name("photon_momentum") {
            const H: f64 = 6.626_070_15e-34;
            let lam = param(&q, "lambda").or_else(|| param(&q, "l")).or_else(|| param(&q, "wavelength"))?;
            if lam <= 0.0 { return None; }
            return Some(tag("photon_momentum", "formula", format!("photon_momentum: p=h/λ={H}/{lam}={}", H / lam)));
        }

        None
    }
}

impl GrammarCoverage for CatalogFormula {
    fn tier(&self) -> CascadeTier {
        CascadeTier::Formula
    }

    fn covers(&self, req: &MolRequest) -> bool {
        Self::eval(&req.query).is_some()
    }

    fn try_answer(&self, req: &MolRequest) -> Result<TierAnswer> {
        Self::eval(&req.query)
            .map(TierAnswer::text)
            .ok_or_else(|| MolError::NotCovered(format!("catalog formula miss: {}", req.query)))
    }
}

/// Live Lookup catalog gear.
#[derive(Debug, Default, Clone)]
pub struct CatalogLookup;

impl CatalogLookup {
    fn eval(query: &str) -> Option<String> {
        let q = query.to_ascii_lowercase();
        let stack = PeriodicStack::subset();
        let try_name = |name: &str| -> bool {
            mentions(&q, name)
                && stack
                    .get_by_name(name)
                    .map(|c| c.gear == GearKind::Lookup || c.gear == GearKind::Navigate)
                    .unwrap_or(false)
        };

        if try_name("bool_and") || try_name("bool_or") || try_name("bool_xor") {
            let a = parse_bool_token(param_str(query, "a")?)?;
            let b = parse_bool_token(param_str(query, "b")?)?;
            let (name, v) = if try_name("bool_and") {
                ("bool_and", a && b)
            } else if try_name("bool_or") {
                ("bool_or", a || b)
            } else {
                ("bool_xor", a ^ b)
            };
            return Some(tag(name, "lookup", format!("{name}: {a} ⊕ {b} → {v}").replace('⊕', if name=="bool_and"{"∧"} else if name=="bool_or"{"∨"} else {"⊻"})));
        }
        if try_name("bool_not") {
            let a = parse_bool_token(param_str(query, "a")?)?;
            return Some(tag("bool_not", "lookup", format!("bool_not: ¬{a} → {}", !a)));
        }
        if try_name("tokenize") {
            let text = param_str(query, "text").or_else(|| param_str(query, "s"))?;
            let n = text.split_whitespace().count();
            return Some(tag("tokenize", "lookup", format!("tokenize: {n} tokens")));
        }
        if try_name("hash_fnv1a") {
            let s = param_str(query, "s").or_else(|| param_str(query, "payload"))?;
            return Some(tag("hash_fnv1a", "lookup", format!("hash_fnv1a: 0x{:016x}", fnv1a64(s))));
        }
        if try_name("sha256") {
            let s = param_str(query, "payload").or_else(|| param_str(query, "s"))?;
            return Some(tag("sha256", "lookup", format!("sha256: {}", sha256_hex(s))));
        }
        if try_name("str_reverse") {
            let s = param_str(query, "s").or_else(|| param_str(query, "text"))?;
            let rev: String = s.chars().rev().collect();
            return Some(tag("str_reverse", "lookup", format!("str_reverse: {rev}")));
        }
        if try_name("str_len") {
            let s = param_str(query, "s").or_else(|| param_str(query, "text"))?;
            return Some(tag("str_len", "lookup", format!("str_len: {}", s.chars().count())));
        }
        if try_name("str_upper") {
            let s = param_str(query, "s").or_else(|| param_str(query, "text"))?;
            return Some(tag("str_upper", "lookup", format!("str_upper: {}", s.to_ascii_uppercase())));
        }
        if try_name("compare_pred") {
            let a = param(&q, "a")?;
            let b = param(&q, "b")?;
            let op = param_str(query, "op").unwrap_or("eq");
            let v = match op {
                "eq" | "==" => a == b,
                "ne" | "!=" => a != b,
                "lt" | "<" => a < b,
                "le" | "<=" => a <= b,
                "gt" | ">" => a > b,
                "ge" | ">=" => a >= b,
                _ => return None,
            };
            return Some(tag("compare_pred", "lookup", format!("compare_pred: {a} {op} {b} → {v}")));
        }
        if try_name("clamp_f64") {
            let x = param(&q, "x")?;
            let lo = param(&q, "lo").unwrap_or(0.0);
            let hi = param(&q, "hi").unwrap_or(1.0);
            return Some(tag("clamp_f64", "lookup", format!("clamp_f64: clamp({x},{lo},{hi})={}", x.clamp(lo, hi))));
        }
        if try_name("rle_encode") {
            let s = param_str(query, "s").or_else(|| param_str(query, "text"))?;
            let mut out = String::new();
            let mut chars = s.chars().peekable();
            while let Some(c) = chars.next() {
                let mut n = 1usize;
                while chars.peek() == Some(&c) {
                    chars.next();
                    n += 1;
                }
                out.push(c);
                out.push_str(&n.to_string());
            }
            return Some(tag("rle_encode", "lookup", format!("rle_encode: {out}")));
        }
        if try_name("cite_format") {
            let id = param_str(query, "id").unwrap_or("claim:unknown");
            let title = param_str(query, "title").unwrap_or("untitled");
            return Some(tag("cite_format", "lookup", format!("cite_format: [{id}] {title}")));
        }
        if try_name("schema_check") {
            let required = list_str(query, "required")?;
            let present = list_str(query, "present").unwrap_or_default();
            let missing: Vec<_> = required
                .iter()
                .filter(|k| !present.iter().any(|p| p == *k))
                .cloned()
                .collect();
            if missing.is_empty() {
                return Some(tag("schema_check", "lookup", "schema_check: OK".into()));
            }
            return Some(tag(
                "schema_check",
                "lookup",
                format!("schema_check: MISSING [{}]", missing.join(",")),
            ));
        }
        if try_name("json_encode") {
            // Encode pairs from keys=[..] values=[..]
            let keys = list_str(query, "keys")?;
            let vals = list_str(query, "values")?;
            if keys.len() != vals.len() {
                return None;
            }
            let body: Vec<String> = keys
                .iter()
                .zip(vals.iter())
                .map(|(k, v)| format!("\"{k}\":\"{v}\""))
                .collect();
            return Some(tag("json_encode", "lookup", format!("json_encode: {{{}}}", body.join(","))));
        }
        if try_name("route_zone") {
            let zone = param_str(query, "zone").or_else(|| param_str(query, "z"))?;
            let zone_l = zone.to_ascii_lowercase();
            let z = match zone_l.as_str() {
                "1" | "z1" => "Z1",
                "2" | "z2" => "Z2",
                "3" | "z3" => "Z3",
                _ => zone,
            };
            return Some(tag("route_zone", "lookup", format!("route_zone: {z}")));
        }
        if try_name("url_join") {
            let base = param_str(query, "base")?;
            let path = param_str(query, "path").unwrap_or("");
            let joined = format!(
                "{}/{}",
                base.trim_end_matches('/'),
                path.trim_start_matches('/')
            );
            return Some(tag("url_join", "lookup", format!("url_join: {joined}")));
        }
        if try_name("hdc_bind") {
            let a = param_str(query, "a")?;
            let b = param_str(query, "b")?;
            // XOR of FNV hashes as soft-ref bind.
            let h = fnv1a64(a) ^ fnv1a64(b);
            return Some(tag("hdc_bind", "lookup", format!("hdc_bind: 0x{h:016x}")));
        }
        if try_name("stdout_write") {
            let payload = param_str(query, "payload").or_else(|| param_str(query, "s"))?;
            return Some(tag("stdout_write", "lookup", format!("stdout_write: wrote {} bytes (soft-ref)", payload.len())));
        }
        if try_name("bst_lookup") || try_name("set_member") {
            let needle = param_str(query, "needle").or_else(|| param_str(query, "x"))?;
            let items = list_str(query, "set").or_else(|| list_str(query, "items"))?;
            let hit = items.iter().any(|i| i == needle);
            let name = if try_name("bst_lookup") { "bst_lookup" } else { "set_member" };
            return Some(tag(name, "lookup", format!("{name}: {needle} ∈ set → {hit}")));
        }
        if try_name("dfa_step") {
            let state = param_str(query, "state").unwrap_or("s0");
            let input = param_str(query, "input").unwrap_or("0");
            // Tiny demo DFA: s0 -0→ s0, s0 -1→ s1, s1 -0→ s0, s1 -1→ s1
            let next = match (state, input) {
                ("s0", "0") => "s0",
                ("s0", "1") => "s1",
                ("s1", "0") => "s0",
                ("s1", "1") => "s1",
                _ => "s_reject",
            };
            return Some(tag("dfa_step", "lookup", format!("dfa_step: {state} --{input}→ {next}")));
        }
        if try_name("type_check_expr") {
            let expr = param_str(query, "expr").unwrap_or("");
            let ok = expr.chars().all(|c| c.is_ascii_alphanumeric() || "+-*/()_.".contains(c));
            return Some(tag(
                "type_check_expr",
                "lookup",
                format!("type_check_expr: {} ({})", if ok { "OK:num_expr" } else { "REJECT" }, expr),
            ));
        }
        if try_name("kelvin_to_celsius") {
            if let Some(k) = param(&q, "kelvin").or_else(|| param(&q, "k")) {
                return Some(tag("kelvin_to_celsius", "lookup", format!("kelvin_to_celsius: {k} K = {} °C", k - 273.15)));
            }
            if let Some(c) = param(&q, "celsius").or_else(|| param(&q, "c")) {
                return Some(tag("kelvin_to_celsius", "lookup", format!("celsius_to_kelvin: {c} °C = {} K", c + 273.15)));
            }
        }
        if try_name("radians_to_degrees") {
            if let Some(r) = param(&q, "rad").or_else(|| param(&q, "radians")) {
                return Some(tag("radians_to_degrees", "lookup", format!("radians_to_degrees: {r} rad = {} deg", r.to_degrees())));
            }
            if let Some(d) = param(&q, "deg").or_else(|| param(&q, "degrees")) {
                return Some(tag("radians_to_degrees", "lookup", format!("degrees_to_radians: {d} deg = {} rad", d.to_radians())));
            }
        }
        if try_name("fs_exists") {
            let path = param_str(query, "path")?;
            // Sandboxed: only allow relative demo paths; never touch real FS beyond string check.
            let allowed = path.starts_with("demo/") || path.starts_with("fixture/");
            let exists = allowed && (path.ends_with(".yaml") || path.ends_with(".json") || path.ends_with(".txt"));
            return Some(tag(
                "fs_exists",
                "lookup",
                format!("fs_exists: path={path} sandboxed_exists={exists} (no host FS probe)"),
            ));
        }
        if try_name("auto_emit_receipt") {
            let act = param_str(query, "act").unwrap_or("noop");
            return Some(tag(
                "auto_emit_receipt",
                "lookup",
                format!("auto_emit_receipt: act={act}; receipt soft-ref emitted"),
            ));
        }
        if try_name("dict_lookup") {
            let key = param_str(query, "key")?;
            let keys = list_str(query, "keys")?;
            let vals = list_str(query, "values")?;
            if keys.len() != vals.len() {
                return None;
            }
            for (k, v) in keys.iter().zip(vals.iter()) {
                if k == key {
                    return Some(tag("dict_lookup", "lookup", format!("dict_lookup: {key} → {v}")));
                }
            }
            return Some(tag("dict_lookup", "lookup", format!("dict_lookup: {key} MISS")));
        }
        if try_name("str_lower") {
            let s = param_str(query, "s").or_else(|| param_str(query, "text"))?;
            return Some(tag("str_lower", "lookup", format!("str_lower: {}", s.to_ascii_lowercase())));
        }
        if try_name("str_trim") {
            let s = param_str(query, "s").or_else(|| param_str(query, "text"))?;
            return Some(tag("str_trim", "lookup", format!("str_trim: {}", s.trim())));
        }
        if try_name("str_contains") {
            let s = param_str(query, "s").or_else(|| param_str(query, "text"))?;
            let sub = param_str(query, "sub").or_else(|| param_str(query, "needle"))?;
            return Some(tag("str_contains", "lookup", format!("str_contains: {} → {}", sub, s.contains(sub))));
        }
        if try_name("popcount") {
            let x = param(&q, "x").or_else(|| param(&q, "n"))? as u64;
            return Some(tag("popcount", "lookup", format!("popcount: {x} → {}", x.count_ones())));
        }
        if try_name("parity_even") {
            let x = param(&q, "x").or_else(|| param(&q, "n"))? as u64;
            let even = x.count_ones() % 2 == 0;
            return Some(tag("parity_even", "lookup", format!("parity_even: {x} → {even}")));
        }
        if try_name("hex_encode") {
            let s = param_str(query, "s").or_else(|| param_str(query, "payload"))?;
            let hex: String = s.bytes().map(|b| format!("{b:02x}")).collect();
            return Some(tag("hex_encode", "lookup", format!("hex_encode: {hex}")));
        }
        if try_name("mime_from_ext") {
            let ext = param_str(query, "ext").or_else(|| param_str(query, "extension"))?.trim_start_matches('.').to_ascii_lowercase();
            let mime = match ext.as_str() {
                "json" => "application/json",
                "yaml" | "yml" => "application/yaml",
                "txt" => "text/plain",
                "html" | "htm" => "text/html",
                "csv" => "text/csv",
                "png" => "image/png",
                "jpg" | "jpeg" => "image/jpeg",
                "pdf" => "application/pdf",
                "wasm" => "application/wasm",
                _ => "application/octet-stream",
            };
            return Some(tag("mime_from_ext", "lookup", format!("mime_from_ext: .{ext} → {mime}")));
        }
        if try_name("http_status_phrase") {
            let code = param(&q, "code").or_else(|| param(&q, "status"))? as i64;
            let phrase = match code {
                200 => "OK",
                201 => "Created",
                204 => "No Content",
                301 => "Moved Permanently",
                302 => "Found",
                400 => "Bad Request",
                401 => "Unauthorized",
                403 => "Forbidden",
                404 => "Not Found",
                409 => "Conflict",
                429 => "Too Many Requests",
                500 => "Internal Server Error",
                502 => "Bad Gateway",
                503 => "Service Unavailable",
                _ => "Unknown",
            };
            return Some(tag("http_status_phrase", "lookup", format!("http_status_phrase: {code} {phrase}")));
        }
        if try_name("is_ascii") {
            let s = param_str(query, "s").or_else(|| param_str(query, "text"))?;
            return Some(tag("is_ascii", "lookup", format!("is_ascii: {}", s.is_ascii())));
        }
        if try_name("crc8") {
            let s = param_str(query, "s").or_else(|| param_str(query, "payload"))?;
            let mut crc: u8 = 0;
            for &b in s.as_bytes() {
                crc ^= b;
                for _ in 0..8 {
                    if crc & 0x80 != 0 {
                        crc = (crc << 1) ^ 0x07;
                    } else {
                        crc <<= 1;
                    }
                }
            }
            return Some(tag("crc8", "lookup", format!("crc8: 0x{crc:02x}")));
        }
        if try_name("weekday_name") {
            let d = param(&q, "dow").or_else(|| param(&q, "day")).or_else(|| param(&q, "d"))? as i64;
            let name = match d.rem_euclid(7) {
                0 => "Sunday",
                1 => "Monday",
                2 => "Tuesday",
                3 => "Wednesday",
                4 => "Thursday",
                5 => "Friday",
                _ => "Saturday",
            };
            return Some(tag("weekday_name", "lookup", format!("weekday_name: dow={d} → {name}")));
        }
        if try_name("bytes_to_kib") {
            if let Some(b) = param(&q, "bytes").or_else(|| param(&q, "b")) {
                return Some(tag("bytes_to_kib", "lookup", format!("bytes_to_kib: {b} B = {} KiB", b / 1024.0)));
            }
            if let Some(k) = param(&q, "kib").or_else(|| param(&q, "ki")) {
                return Some(tag("bytes_to_kib", "lookup", format!("kib_to_bytes: {k} KiB = {} B", k * 1024.0)));
            }
        }
        if try_name("bool_nand") {
            let a = parse_bool_token(param_str(query, "a")?)?;
            let b = parse_bool_token(param_str(query, "b")?)?;
            return Some(tag("bool_nand", "lookup", format!("bool_nand: ¬({a} ∧ {b}) → {}", !(a && b))));
        }
        if try_name("csv_field") {
            let s = param_str(query, "s").or_else(|| param_str(query, "text"))?;
            let needs = s.contains(',') || s.contains('"') || s.contains('\n') || s.contains(' ');
            let out = if needs {
                format!("\"{}\"", s.replace('"', "\"\""))
            } else {
                s.to_string()
            };
            return Some(tag("csv_field", "lookup", format!("csv_field: {out}")));
        }
        
        if try_name("str_starts_with") {
            let s = param_str(query, "s").or_else(|| param_str(query, "text"))?;
            let p = param_str(query, "prefix").or_else(|| param_str(query, "p"))?;
            return Some(tag("str_starts_with", "lookup", format!("str_starts_with: {} → {}", p, s.starts_with(p))));
        }
        if try_name("str_ends_with") {
            let s = param_str(query, "s").or_else(|| param_str(query, "text"))?;
            let p = param_str(query, "suffix").or_else(|| param_str(query, "p"))?;
            return Some(tag("str_ends_with", "lookup", format!("str_ends_with: {} → {}", p, s.ends_with(p))));
        }
        if try_name("str_replace") {
            let s = param_str(query, "s").or_else(|| param_str(query, "text"))?;
            let old = param_str(query, "old").or_else(|| param_str(query, "from"))?;
            let new = param_str(query, "new").or_else(|| param_str(query, "to")).unwrap_or("");
            let out = s.replacen(old, new, 1);
            return Some(tag("str_replace", "lookup", format!("str_replace: \"{out}\"")));
        }
        if try_name("bool_nor") {
            let a = parse_bool_token(param_str(query, "a")?)?;
            let b = parse_bool_token(param_str(query, "b")?)?;
            return Some(tag("bool_nor", "lookup", format!("bool_nor: ¬({a} ∨ {b}) → {}", !(a || b))));
        }
        if try_name("bool_xnor") {
            let a = parse_bool_token(param_str(query, "a")?)?;
            let b = parse_bool_token(param_str(query, "b")?)?;
            return Some(tag("bool_xnor", "lookup", format!("bool_xnor: {a} ⊙ {b} → {}", a == b)));
        }
        if try_name("bit_and") || try_name("bit_or") || try_name("bit_xor") {
            let a = param(&q, "a").or_else(|| param(&q, "x"))? as u64;
            let b = param(&q, "b").or_else(|| param(&q, "y"))? as u64;
            let (name, v) = if try_name("bit_and") {
                ("bit_and", a & b)
            } else if try_name("bit_or") {
                ("bit_or", a | b)
            } else {
                ("bit_xor", a ^ b)
            };
            return Some(tag(name, "lookup", format!("{name}: {a} ∘ {b} → {v}")));
        }
        if try_name("bit_shl") {
            let a = param(&q, "a").or_else(|| param(&q, "x"))? as u64;
            let n = param(&q, "n").or_else(|| param(&q, "shift"))? as u32;
            if n >= 64 { return None; }
            return Some(tag("bit_shl", "lookup", format!("bit_shl: {a} << {n} → {}", a << n)));
        }
        if try_name("bit_shr") {
            let a = param(&q, "a").or_else(|| param(&q, "x"))? as u64;
            let n = param(&q, "n").or_else(|| param(&q, "shift"))? as u32;
            if n >= 64 { return None; }
            return Some(tag("bit_shr", "lookup", format!("bit_shr: {a} >> {n} → {}", a >> n)));
        }
        if try_name("is_hex") {
            let s = param_str(query, "s").or_else(|| param_str(query, "hex")).or_else(|| param_str(query, "x"))?;
            let ok = !s.is_empty() && s.bytes().all(|b| b.is_ascii_hexdigit());
            return Some(tag("is_hex", "lookup", format!("is_hex: \"{s}\" → {ok}")));
        }
        if try_name("html_escape") {
            let s = param_str(query, "s").or_else(|| param_str(query, "text"))?;
            let out = s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;");
            return Some(tag("html_escape", "lookup", format!("html_escape: {out}")));
        }
        if try_name("path_ext") {
            let p = param_str(query, "path").or_else(|| param_str(query, "p")).or_else(|| param_str(query, "s"))?;
            let ext = std::path::Path::new(p).extension().and_then(|e| e.to_str()).unwrap_or("");
            return Some(tag("path_ext", "lookup", format!("path_ext: \"{p}\" → \"{ext}\"")));
        }
        if try_name("path_basename") {
            let p = param_str(query, "path").or_else(|| param_str(query, "p")).or_else(|| param_str(query, "s"))?;
            let base = std::path::Path::new(p).file_name().and_then(|e| e.to_str()).unwrap_or(p);
            return Some(tag("path_basename", "lookup", format!("path_basename: \"{p}\" → \"{base}\"")));
        }
        if try_name("color_hex_rgb") {
            let s = param_str(query, "hex").or_else(|| param_str(query, "color")).or_else(|| param_str(query, "s"))?;
            let h = s.trim().trim_start_matches('#');
            if h.len() != 6 || !h.bytes().all(|b| b.is_ascii_hexdigit()) { return None; }
            let r = u8::from_str_radix(&h[0..2], 16).ok()?;
            let g = u8::from_str_radix(&h[2..4], 16).ok()?;
            let b = u8::from_str_radix(&h[4..6], 16).ok()?;
            return Some(tag("color_hex_rgb", "lookup", format!("color_hex_rgb: #{h} → rgb({r},{g},{b})")));
        }
        if try_name("ipv4_ok") {
            let s = param_str(query, "ip").or_else(|| param_str(query, "s")).or_else(|| param_str(query, "addr"))?;
            let parts: Vec<_> = s.split('.').collect();
            let ok = parts.len() == 4 && parts.iter().all(|p| p.parse::<u8>().is_ok());
            return Some(tag("ipv4_ok", "lookup", format!("ipv4_ok: \"{s}\" → {ok}")));
        }
        if try_name("is_blank") {
            let s = param_str(query, "s").or_else(|| param_str(query, "text")).unwrap_or("");
            let ok = s.chars().all(|c| c.is_whitespace());
            return Some(tag("is_blank", "lookup", format!("is_blank: → {ok}")));
        }
        if try_name("json_escape") {
            let s = param_str(query, "s").or_else(|| param_str(query, "text"))?;
            let mut out = String::from('"');
            for c in s.chars() {
                match c {
                    '"' => out.push_str("\\\""),
                    '\\' => out.push_str("\\\\"),
                    '\n' => out.push_str("\\n"),
                    '\r' => out.push_str("\\r"),
                    '\t' => out.push_str("\\t"),
                    c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
                    c => out.push(c),
                }
            }
            out.push('"');
            return Some(tag("json_escape", "lookup", format!("json_escape: {out}")));
        }
        if try_name("mime_is_text") {
            let m = param_str(query, "mime").or_else(|| param_str(query, "type")).or_else(|| param_str(query, "s"))?;
            let ok = m.to_ascii_lowercase().starts_with("text/");
            return Some(tag("mime_is_text", "lookup", format!("mime_is_text: \"{m}\" → {ok}")));
        }
        if try_name("miles_to_km") {
            if let Some(mi) = param(&q, "mi").or_else(|| param(&q, "miles")) {
                return Some(tag("miles_to_km", "lookup", format!("miles_to_km: {mi} mi = {} km", mi * 1.609344)));
            }
            if let Some(km) = param(&q, "km").or_else(|| param(&q, "kilometers")) {
                return Some(tag("miles_to_km", "lookup", format!("km_to_miles: {km} km = {} mi", km / 1.609344)));
            }
        }

        if try_name("template_fill") {
            let tmpl = param_str(query, "template").or_else(|| param_str(query, "tmpl"))?;
            let mut out = tmpl.to_string();
            if let (Some(keys), Some(vals)) = (list_str(query, "keys"), list_str(query, "values")) {
                if keys.len() != vals.len() {
                    return Some(tag("template_fill", "lookup", "template_fill: refuse keys/values length mismatch".into()));
                }
                for (k, v) in keys.iter().zip(vals.iter()) {
                    out = out.replace(&format!("{{{k}}}"), v);
                }
            }
            for key in ["name", "id", "value", "user", "item"] {
                if let Some(v) = param_str(query, key) {
                    out = out.replace(&format!("{{{key}}}"), v);
                }
            }
            if out.contains('{') && out.contains('}') {
                return Some(tag(
                    "template_fill",
                    "lookup",
                    format!("template_fill: REFUSE ungrounded slots remain in '{out}' — residual escalate only under allow_model+VoI"),
                ));
            }
            return Some(tag("template_fill", "lookup", format!("template_fill: grounded '{out}'")));
        }
        if try_name("residual_policy") {
            let voi = param(&q, "voi").unwrap_or(0.0);
            let c_z = param(&q, "c_z").or_else(|| param(&q, "cz")).unwrap_or(0.0);
            let allow = parse_bool_token(param_str(query, "allow_model").unwrap_or("false")).unwrap_or(false);
            let decision = if c_z >= 1.0 {
                "refuse_satiation"
            } else if !allow || voi <= 0.0 {
                "refuse_voi"
            } else {
                "escalate_model_last"
            };
            return Some(tag(
                "residual_policy",
                "lookup",
                format!("residual_policy: decision={decision} (voi={voi}, c_z={c_z}, allow_model={allow}); Model LAST only on escalate; never launders Deterministic"),
            ));
        }
        if try_name("base64_encode") {
            let s = param_str(query, "s").or_else(|| param_str(query, "text")).or_else(|| param_str(query, "payload"))?;
            return Some(tag("base64_encode", "lookup", format!("base64_encode: {}", b64_encode(s))));
        }
        if try_name("base64_decode") {
            let s = param_str(query, "s").or_else(|| param_str(query, "text")).or_else(|| param_str(query, "payload"))?;
            return match b64_decode(s) {
                Some(t) => Some(tag("base64_decode", "lookup", format!("base64_decode: {t}"))),
                None => Some(tag("base64_decode", "lookup", "base64_decode: refuse invalid".into())),
            };
        }
        if try_name("url_encode") {
            let s = param_str(query, "s").or_else(|| param_str(query, "text"))?;
            let enc: String = s.bytes().map(|b| match b {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (b as char).to_string(),
                _ => format!("%{b:02X}"),
            }).collect();
            return Some(tag("url_encode", "lookup", format!("url_encode: {enc}")));
        }
        if try_name("is_email_shape") {
            let s = param_str(query, "s").or_else(|| param_str(query, "email"))?;
            let ok = s.contains('@') && s.contains('.') && !s.contains(' ') && s.len() >= 5;
            return Some(tag("is_email_shape", "lookup", format!("is_email_shape: {ok}")));
        }
        if try_name("is_uuid_shape") {
            let s = param_str(query, "s").or_else(|| param_str(query, "uuid"))?;
            let parts: Vec<_> = s.split('-').collect();
            let ok = parts.len() == 5
                && parts[0].len() == 8
                && parts[1].len() == 4
                && parts[2].len() == 4
                && parts[3].len() == 4
                && parts[4].len() == 12
                && s.chars().all(|c| c.is_ascii_hexdigit() || c == '-');
            return Some(tag("is_uuid_shape", "lookup", format!("is_uuid_shape: {ok}")));
        }
        if try_name("weekday_from_ymd") {
            let y = param(&q, "y").or_else(|| param(&q, "year"))? as i32;
            let m = param(&q, "m").or_else(|| param(&q, "month"))? as i32;
            let d = param(&q, "d").or_else(|| param(&q, "day"))? as i32;
            let t = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
            if !(1..=12).contains(&m) || !(1..=31).contains(&d) { return None; }
            let mut yy = y;
            if m < 3 { yy -= 1; }
            let dow = (yy + yy / 4 - yy / 100 + yy / 400 + t[(m as usize) - 1] + d).rem_euclid(7);
            let names = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
            return Some(tag("weekday_from_ymd", "lookup", format!("weekday_from_ymd: {}-{m:02}-{d:02} → {}", y, names[dow as usize])));
        }
        if try_name("month_name") {
            let m = param(&q, "m").or_else(|| param(&q, "month"))? as usize;
            let names = ["", "January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];
            if m == 0 || m > 12 { return None; }
            return Some(tag("month_name", "lookup", format!("month_name: {m} → {}", names[m])));
        }
        if try_name("liters_to_gallons") {
            let l = param(&q, "l").or_else(|| param(&q, "liters"));
            let g = param(&q, "gal").or_else(|| param(&q, "gallons"));
            return match (l, g) {
                (Some(l), None) => Some(tag("liters_to_gallons", "lookup", format!("liters_to_gallons: {l} L = {} gal", l / 3.785411784))),
                (None, Some(g)) => Some(tag("liters_to_gallons", "lookup", format!("liters_to_gallons: {g} gal = {} L", g * 3.785411784))),
                _ => None,
            };
        }
        if try_name("watts_to_hp") {
            let w = param(&q, "w").or_else(|| param(&q, "watts"));
            let hp = param(&q, "hp");
            return match (w, hp) {
                (Some(w), None) => Some(tag("watts_to_hp", "lookup", format!("watts_to_hp: {w} W = {} hp", w / 745.6998715822702))),
                (None, Some(hp)) => Some(tag("watts_to_hp", "lookup", format!("watts_to_hp: {hp} hp = {} W", hp * 745.6998715822702))),
                _ => None,
            };
        }
        if try_name("pascal_to_psi") {
            let pa = param(&q, "pa").or_else(|| param(&q, "pascal"));
            let psi = param(&q, "psi");
            return match (pa, psi) {
                (Some(pa), None) => Some(tag("pascal_to_psi", "lookup", format!("pascal_to_psi: {pa} Pa = {} psi", pa / 6894.757293168))),
                (None, Some(psi)) => Some(tag("pascal_to_psi", "lookup", format!("pascal_to_psi: {psi} psi = {} Pa", psi * 6894.757293168))),
                _ => None,
            };
        }
        if try_name("str_is_numeric") {
            let s = param_str(query, "s").or_else(|| param_str(query, "text"))?;
            let ok = !s.is_empty() && s.chars().all(|c| c.is_ascii_digit() || c == '.' || c == '-' || c == '+');
            return Some(tag("str_is_numeric", "lookup", format!("str_is_numeric: {ok}")));
        }
        if try_name("str_pad_left") {
            let s = param_str(query, "s").or_else(|| param_str(query, "text"))?;
            let n = param(&q, "n").or_else(|| param(&q, "width"))? as usize;
            let pad = param_str(query, "pad").unwrap_or(" ");
            let ch = pad.chars().next().unwrap_or(' ');
            let mut out = s.to_string();
            while out.chars().count() < n { out.insert(0, ch); }
            return Some(tag("str_pad_left", "lookup", format!("str_pad_left: {out}")));
        }
        if try_name("str_repeat") {
            let s = param_str(query, "s").or_else(|| param_str(query, "text"))?;
            let n = param(&q, "n")? as usize;
            if n > 10_000 { return None; }
            return Some(tag("str_repeat", "lookup", format!("str_repeat: {}", s.repeat(n))));
        }
        if try_name("bit_count_ones") {
            let x = param(&q, "x").or_else(|| param(&q, "a"))? as u64;
            return Some(tag("bit_count_ones", "lookup", format!("bit_count_ones: {}", x.count_ones())));
        }
        if try_name("bit_rotate_left") {
            let x = param(&q, "x").or_else(|| param(&q, "a"))? as u64;
            let n = param(&q, "n").unwrap_or(1.0) as u32;
            return Some(tag("bit_rotate_left", "lookup", format!("bit_rotate_left: 0x{:016x}", x.rotate_left(n))));
        }
        if try_name("crc16_ccitt") {
            let s = param_str(query, "s").or_else(|| param_str(query, "payload"))?;
            let mut crc: u16 = 0xFFFF;
            for &b in s.as_bytes() {
                crc ^= (b as u16) << 8;
                for _ in 0..8 {
                    if crc & 0x8000 != 0 { crc = (crc << 1) ^ 0x1021; } else { crc <<= 1; }
                }
            }
            return Some(tag("crc16_ccitt", "lookup", format!("crc16_ccitt: 0x{crc:04x}")));
        }
        if try_name("isbn10_check") {
            let s = param_str(query, "s").or_else(|| param_str(query, "isbn"))?;
            let digits: Vec<char> = s.chars().filter(|c| c.is_ascii_digit() || *c == 'X' || *c == 'x').collect();
            if digits.len() != 10 { return Some(tag("isbn10_check", "lookup", "isbn10_check: refuse len≠10".into())); }
            let mut sum = 0i32;
            for (i, c) in digits.iter().enumerate() {
                let v = if *c == 'X' || *c == 'x' { 10 } else { c.to_digit(10).unwrap_or(0) as i32 };
                sum += v * (10 - i as i32);
            }
            let ok = sum % 11 == 0;
            return Some(tag("isbn10_check", "lookup", format!("isbn10_check: {ok}")));
        }
        if try_name("luhn_check") {
            let s = param_str(query, "s").or_else(|| param_str(query, "pan")).or_else(|| param_str(query, "number"))?;
            let digits: Vec<u32> = s.chars().filter(|c| c.is_ascii_digit()).filter_map(|c| c.to_digit(10)).collect();
            if digits.len() < 2 { return None; }
            let mut sum = 0u32;
            let mut alt = false;
            for &d in digits.iter().rev() {
                let mut v = d;
                if alt { v *= 2; if v > 9 { v -= 9; } }
                sum += v;
                alt = !alt;
            }
            let ok = sum % 10 == 0;
            return Some(tag("luhn_check", "lookup", format!("luhn_check: {ok}")));
        }
        if try_name("http_method_ok") {
            let m = param_str(query, "method").or_else(|| param_str(query, "m"))?.to_ascii_uppercase();
            let ok = matches!(m.as_str(), "GET" | "POST" | "PUT" | "PATCH" | "DELETE" | "HEAD" | "OPTIONS");
            return Some(tag("http_method_ok", "lookup", format!("http_method_ok: {m} → {ok}")));
        }
        if try_name("port_well_known") {
            let p = param(&q, "port").or_else(|| param(&q, "p"))? as u16;
            let svc = match p {
                20 | 21 => "ftp", 22 => "ssh", 25 => "smtp", 53 => "dns", 80 => "http",
                110 => "pop3", 143 => "imap", 443 => "https", 5432 => "postgres", 6379 => "redis",
                _ => "unknown",
            };
            return Some(tag("port_well_known", "lookup", format!("port_well_known: {p} → {svc}")));
        }
        if try_name("path_dirname") {
            let p = param_str(query, "path").or_else(|| param_str(query, "p"))?;
            let dir = if let Some(i) = p.rfind('/') { if i == 0 { "/" } else { &p[..i] } } else { "." };
            return Some(tag("path_dirname", "lookup", format!("path_dirname: {dir}")));
        }
        if try_name("mime_charset_utf8") {
            let mime = param_str(query, "mime").or_else(|| param_str(query, "type")).unwrap_or("text/plain");
            return Some(tag("mime_charset_utf8", "lookup", format!("mime_charset_utf8: {mime}; charset=utf-8")));
        }
        if try_name("zone_from_tier") {
            let t = param_str(query, "tier").or_else(|| param_str(query, "t"))?.to_ascii_lowercase();
            let z = match t.as_str() {
                "lookup" | "formula" | "solver" | "settle" => "Z1",
                "compose" | "retrieve" | "cite" => "Z2",
                "model" | "generative" => "Z3",
                _ => "Z?",
            };
            return Some(tag("zone_from_tier", "lookup", format!("zone_from_tier: {t} → {z}")));
        }
        if try_name("estimate_kind_label") {
            let k = param_str(query, "kind").or_else(|| param_str(query, "k"))?.to_ascii_lowercase();
            let label = match k.as_str() {
                "catalog" | "mu_catalog" | "tier0" => "Estimated",
                "metered" | "rapl" | "nvml" | "smc" => "Metered",
                "unavailable" | "none" => "Unmetered",
                _ => "Estimated",
            };
            return Some(tag("estimate_kind_label", "lookup", format!("estimate_kind_label: {k} → {label}")));
        }
        if try_name("cite_style_apa") {
            let author = param_str(query, "author").unwrap_or("Unknown");
            let year = param_str(query, "year").unwrap_or("n.d.");
            let title = param_str(query, "title").unwrap_or("Untitled");
            return Some(tag("cite_style_apa", "lookup", format!("cite_style_apa: {author} ({year}). {title}.")));
        }

        None
    }
}

impl GrammarCoverage for CatalogLookup {
    fn tier(&self) -> CascadeTier {
        CascadeTier::Lookup
    }

    fn covers(&self, req: &MolRequest) -> bool {
        Self::eval(&req.query).is_some()
    }

    fn try_answer(&self, req: &MolRequest) -> Result<TierAnswer> {
        Self::eval(&req.query)
            .map(TierAnswer::text)
            .ok_or_else(|| MolError::NotCovered(format!("catalog lookup miss: {}", req.query)))
    }
}

/// Live Solver catalog gear.
#[derive(Debug, Default, Clone)]
pub struct CatalogSolver;

impl CatalogSolver {
    fn eval(query: &str) -> Option<String> {
        let q = query.to_ascii_lowercase();
        let stack = PeriodicStack::subset();
        let try_name = |name: &str| -> bool {
            mentions(&q, name)
                && stack
                    .get_by_name(name)
                    .map(|c| c.gear == GearKind::Solver)
                    .unwrap_or(false)
        };

        if try_name("mean_f64") {
            let xs = list_f64(&q, "xs").or_else(|| list_f64(&q, "values"))?;
            let m = xs.iter().sum::<f64>() / xs.len() as f64;
            return Some(tag("mean_f64", "solver", format!("mean_f64: {m}")));
        }
        if try_name("variance_f64") || try_name("std_f64") {
            let xs = list_f64(&q, "xs").or_else(|| list_f64(&q, "values"))?;
            let m = xs.iter().sum::<f64>() / xs.len() as f64;
            let var = xs.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / xs.len() as f64;
            if try_name("std_f64") {
                return Some(tag("std_f64", "solver", format!("std_f64: {}", var.sqrt())));
            }
            return Some(tag("variance_f64", "solver", format!("variance_f64: {var}")));
        }
        if try_name("median_f64") {
            let mut xs = list_f64(&q, "xs").or_else(|| list_f64(&q, "values"))?;
            xs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let mid = xs.len() / 2;
            let med = if xs.len() % 2 == 1 {
                xs[mid]
            } else {
                0.5 * (xs[mid - 1] + xs[mid])
            };
            return Some(tag("median_f64", "solver", format!("median_f64: {med}")));
        }
        if try_name("argmin") || try_name("argmax") {
            let xs = list_f64(&q, "xs").or_else(|| list_f64(&q, "values"))?;
            let (idx, v) = if try_name("argmin") {
                xs.iter()
                    .enumerate()
                    .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                    .map(|(i, v)| (i, *v))?
            } else {
                xs.iter()
                    .enumerate()
                    .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                    .map(|(i, v)| (i, *v))?
            };
            let name = if try_name("argmin") { "argmin" } else { "argmax" };
            return Some(tag(name, "solver", format!("{name}: index={idx} value={v}")));
        }
        if try_name("edit_distance") {
            let a = param_str(query, "a")?;
            let b = param_str(query, "b")?;
            return Some(tag("edit_distance", "solver", format!("edit_distance: {}", levenshtein(a, b))));
        }
        if try_name("hamming_distance") {
            let a = param_str(query, "a")?;
            let b = param_str(query, "b")?;
            if a.len() != b.len() {
                return Some(tag("hamming_distance", "solver", "hamming_distance: refuse unequal lengths".into()));
            }
            let d = a.chars().zip(b.chars()).filter(|(x, y)| x != y).count();
            return Some(tag("hamming_distance", "solver", format!("hamming_distance: {d}")));
        }
        if try_name("interval_intersect") {
            let a0 = param(&q, "a0")?;
            let a1 = param(&q, "a1")?;
            let b0 = param(&q, "b0")?;
            let b1 = param(&q, "b1")?;
            let lo = a0.max(b0);
            let hi = a1.min(b1);
            if lo <= hi {
                return Some(tag("interval_intersect", "solver", format!("interval_intersect: [{lo},{hi}]")));
            }
            return Some(tag("interval_intersect", "solver", "interval_intersect: empty".into()));
        }
        if try_name("set_union") || try_name("set_intersect") || try_name("set_difference") {
            let a: BTreeSet<_> = list_str(query, "a")?.into_iter().collect();
            let b: BTreeSet<_> = list_str(query, "b")?.into_iter().collect();
            let (name, out): (&str, BTreeSet<_>) = if try_name("set_union") {
                ("set_union", a.union(&b).cloned().collect())
            } else if try_name("set_intersect") {
                ("set_intersect", a.intersection(&b).cloned().collect())
            } else {
                ("set_difference", a.difference(&b).cloned().collect())
            };
            let items: Vec<_> = out.into_iter().collect();
            return Some(tag(name, "solver", format!("{name}: [{}]", items.join(","))));
        }
        if try_name("gcd_u64") || try_name("lcm_u64") {
            let a = param(&q, "a")? as u64;
            let b = param(&q, "b")? as u64;
            let g = {
                let (mut x, mut y) = (a, b);
                while y != 0 {
                    let t = y;
                    y = x % y;
                    x = t;
                }
                x
            };
            if try_name("lcm_u64") {
                let l = if g == 0 { 0 } else { a / g * b };
                return Some(tag("lcm_u64", "solver", format!("lcm_u64: {l}")));
            }
            return Some(tag("gcd_u64", "solver", format!("gcd_u64: {g}")));
        }
        if try_name("dot_f64") {
            let a = list_f64(&q, "a")?;
            let b = list_f64(&q, "b")?;
            if a.len() != b.len() {
                return None;
            }
            let d: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
            return Some(tag("dot_f64", "solver", format!("dot_f64: {d}")));
        }
        if try_name("sort_f64") {
            let mut xs = list_f64(&q, "xs").or_else(|| list_f64(&q, "values"))?;
            xs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            return Some(tag("sort_f64", "solver", format!("sort_f64: {:?}", xs)));
        }
        if try_name("prefix_sum") {
            let xs = list_f64(&q, "xs").or_else(|| list_f64(&q, "values"))?;
            let mut acc = 0.0;
            let mut out = Vec::new();
            for x in xs {
                acc += x;
                out.push(acc);
            }
            return Some(tag("prefix_sum", "solver", format!("prefix_sum: {:?}", out)));
        }
        if try_name("dist_euclid_2d") {
            let x1 = param(&q, "x1")?;
            let y1 = param(&q, "y1")?;
            let x2 = param(&q, "x2")?;
            let y2 = param(&q, "y2")?;
            let d = ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();
            return Some(tag("dist_euclid_2d", "solver", format!("dist_euclid_2d: {d}")));
        }
        if try_name("dist_euclid_3d") {
            let x1 = param(&q, "x1")?;
            let y1 = param(&q, "y1")?;
            let z1 = param(&q, "z1")?;
            let x2 = param(&q, "x2")?;
            let y2 = param(&q, "y2")?;
            let z2 = param(&q, "z2")?;
            let d = ((x2 - x1).powi(2) + (y2 - y1).powi(2) + (z2 - z1).powi(2)).sqrt();
            return Some(tag("dist_euclid_3d", "solver", format!("dist_euclid_3d: {d}")));
        }
        if try_name("matmul_2x2") {
            // A=[[a,b],[c,d]] B=[[e,f],[g,h]]
            let a = param(&q, "a")?;
            let b = param(&q, "b")?;
            let c = param(&q, "c")?;
            let d = param(&q, "d")?;
            let e = param(&q, "e")?;
            let f = param(&q, "f")?;
            let g = param(&q, "g")?;
            let h = param(&q, "h")?;
            let r00 = a * e + b * g;
            let r01 = a * f + b * h;
            let r10 = c * e + d * g;
            let r11 = c * f + d * h;
            return Some(tag(
                "matmul_2x2",
                "solver",
                format!("matmul_2x2: [[{r00},{r01}],[{r10},{r11}]]"),
            ));
        }
        if try_name("binary_search") {
            let xs = list_f64(&q, "xs")?;
            let needle = param(&q, "needle").or_else(|| param(&q, "x"))?;
            // Require sorted non-decreasing
            let mut lo = 0isize;
            let mut hi = xs.len() as isize - 1;
            let mut found = None;
            while lo <= hi {
                let mid = (lo + hi) / 2;
                let v = xs[mid as usize];
                if (v - needle).abs() < 1e-12 {
                    found = Some(mid as usize);
                    break;
                } else if v < needle {
                    lo = mid + 1;
                } else {
                    hi = mid - 1;
                }
            }
            return Some(tag(
                "binary_search",
                "solver",
                format!("binary_search: {:?}", found),
            ));
        }
        if try_name("moving_avg") {
            let xs = list_f64(&q, "xs")?;
            let w = param(&q, "window").unwrap_or(3.0).round().max(1.0) as usize;
            if w > xs.len() {
                return None;
            }
            let mut out = Vec::new();
            for i in 0..=(xs.len() - w) {
                let m = xs[i..i + w].iter().sum::<f64>() / w as f64;
                out.push(m);
            }
            return Some(tag("moving_avg", "solver", format!("moving_avg: {:?}", out)));
        }
        if try_name("bfs") {
            // edges=[[0,1],[1,2]] start=0 goal=2
            let edges = list_f64(&q, "edges");
            // Also accept edges as pairs via edge list of flat nums
            let flat = edges.or_else(|| list_f64(&q, "e"))?;
            if flat.len() % 2 != 0 {
                return None;
            }
            let start = param(&q, "start").unwrap_or(0.0) as i64;
            let goal = param(&q, "goal")?;
            let goal = goal as i64;
            let mut adj: std::collections::BTreeMap<i64, Vec<i64>> = std::collections::BTreeMap::new();
            for chunk in flat.chunks(2) {
                let (u, v) = (chunk[0] as i64, chunk[1] as i64);
                adj.entry(u).or_default().push(v);
                adj.entry(v).or_default().push(u);
            }
            let mut qe = VecDeque::from([start]);
            let mut seen = BTreeSet::from([start]);
            let mut parent = std::collections::BTreeMap::from([(start, None)]);
            while let Some(u) = qe.pop_front() {
                if u == goal {
                    let mut path = vec![u];
                    let mut cur = u;
                    while let Some(Some(p)) = parent.get(&cur).copied() {
                        path.push(p);
                        cur = p;
                    }
                    path.reverse();
                    return Some(tag("bfs", "solver", format!("bfs: path={path:?}")));
                }
                for &v in adj.get(&u).into_iter().flatten() {
                    if seen.insert(v) {
                        parent.insert(v, Some(u));
                        qe.push_back(v);
                    }
                }
            }
            return Some(tag("bfs", "solver", "bfs: unreachable".into()));
        }
        if try_name("mode_f64") {
            let xs = list_f64(&q, "xs").or_else(|| list_f64(&q, "values"))?;
            let mut best = xs[0];
            let mut best_n = 0usize;
            for &v in &xs {
                let n = xs.iter().filter(|x| (**x - v).abs() < 1e-12).count();
                if n > best_n {
                    best_n = n;
                    best = v;
                }
            }
            return Some(tag("mode_f64", "solver", format!("mode_f64: {best} (count={best_n})")));
        }
        if try_name("range_f64") {
            let xs = list_f64(&q, "xs").or_else(|| list_f64(&q, "values"))?;
            let min = xs.iter().cloned().fold(f64::INFINITY, f64::min);
            let max = xs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            return Some(tag("range_f64", "solver", format!("range_f64: max-min={}-{}={}", max, min, max - min)));
        }
        if try_name("l1_norm") {
            let xs = list_f64(&q, "xs").or_else(|| list_f64(&q, "values")).or_else(|| list_f64(&q, "v"))?;
            let n: f64 = xs.iter().map(|x| x.abs()).sum();
            return Some(tag("l1_norm", "solver", format!("l1_norm: {n}")));
        }
        if try_name("l2_norm") {
            let xs = list_f64(&q, "xs").or_else(|| list_f64(&q, "values")).or_else(|| list_f64(&q, "v"))?;
            let n = xs.iter().map(|x| x * x).sum::<f64>().sqrt();
            return Some(tag("l2_norm", "solver", format!("l2_norm: {n}")));
        }
        if try_name("cosine_sim") {
            let a = list_f64(&q, "a")?;
            let b = list_f64(&q, "b")?;
            if a.len() != b.len() || a.is_empty() {
                return None;
            }
            let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
            let na = a.iter().map(|x| x * x).sum::<f64>().sqrt();
            let nb = b.iter().map(|x| x * x).sum::<f64>().sqrt();
            if na < 1e-15 || nb < 1e-15 {
                return Some(tag("cosine_sim", "solver", "cosine_sim: refuse zero vector".into()));
            }
            return Some(tag("cosine_sim", "solver", format!("cosine_sim: {}", dot / (na * nb))));
        }
        if try_name("transpose_2x2") {
            let a = param(&q, "a")?;
            let b = param(&q, "b")?;
            let c = param(&q, "c")?;
            let d = param(&q, "d")?;
            return Some(tag("transpose_2x2", "solver", format!("transpose_2x2: [[{a},{c}],[{b},{d}]]")));
        }
        if try_name("trace_2x2") {
            let a = param(&q, "a")?;
            let d = param(&q, "d")?;
            return Some(tag("trace_2x2", "solver", format!("trace_2x2: a+d={}", a + d)));
        }
        if try_name("fibonacci_u64") {
            let n = param(&q, "n")? as u64;
            if n > 92 {
                return Some(tag(
                    "fibonacci_u64",
                    "solver",
                    "fibonacci_u64: refuse n>92 (u64 overflow)".into(),
                ));
            }
            if n == 0 {
                return Some(tag("fibonacci_u64", "solver", "fibonacci_u64: F(0)=0".into()));
            }
            let mut f0: u64 = 0;
            let mut f1: u64 = 1;
            for _ in 1..n {
                let t = f0.wrapping_add(f1);
                f0 = f1;
                f1 = t;
            }
            return Some(tag(
                "fibonacci_u64",
                "solver",
                format!("fibonacci_u64: F({n})={f1}"),
            ));
        }
        if try_name("is_prime_u64") {
            let n = param(&q, "n").or_else(|| param(&q, "x"))? as u64;
            let prime = {
                if n < 2 {
                    false
                } else if n % 2 == 0 {
                    n == 2
                } else {
                    let mut d = 3u64;
                    let mut ok = true;
                    while d * d <= n {
                        if n % d == 0 {
                            ok = false;
                            break;
                        }
                        d += 2;
                    }
                    ok
                }
            };
            return Some(tag("is_prime_u64", "solver", format!("is_prime_u64: {n} → {prime}")));
        }
        if try_name("is_sorted") {
            let xs = list_f64(&q, "xs").or_else(|| list_f64(&q, "values"))?;
            let sorted = xs.windows(2).all(|w| w[0] <= w[1]);
            return Some(tag("is_sorted", "solver", format!("is_sorted: {sorted}")));
        }
        if try_name("unique_count") {
            let xs = list_f64(&q, "xs").or_else(|| list_f64(&q, "values"))?;
            let mut seen = BTreeSet::new();
            for x in xs {
                // bucket by 1e-12 grid via bits string for f64 uniqueness soft-ref
                seen.insert(format!("{x:.12}"));
            }
            return Some(tag("unique_count", "solver", format!("unique_count: {}", seen.len())));
        }
        if try_name("paren_balance") {
            let s = param_str(query, "s").or_else(|| param_str(query, "text"))?;
            let mut depth = 0i64;
            let mut ok = true;
            for c in s.chars() {
                match c {
                    '(' | '[' | '{' => depth += 1,
                    ')' | ']' | '}' => {
                        depth -= 1;
                        if depth < 0 {
                            ok = false;
                            break;
                        }
                    }
                    _ => {}
                }
            }
            ok = ok && depth == 0;
            return Some(tag("paren_balance", "solver", format!("paren_balance: {ok}")));
        }
        if try_name("lcp_strings") {
            let items = list_str(query, "strings").or_else(|| list_str(query, "xs")).or_else(|| list_str(query, "a"))?;
            if items.is_empty() {
                return Some(tag("lcp_strings", "solver", "lcp_strings: \"\"".into()));
            }
            let mut prefix = items[0].clone();
            for s in items.iter().skip(1) {
                let mut i = 0;
                let pb = prefix.as_bytes();
                let sb = s.as_bytes();
                while i < pb.len() && i < sb.len() && pb[i] == sb[i] {
                    i += 1;
                }
                prefix.truncate(i);
                if prefix.is_empty() {
                    break;
                }
            }
            return Some(tag("lcp_strings", "solver", format!("lcp_strings: \"{prefix}\"")));
        }

        if try_name("sum_f64") {
            let xs = list_f64(&q, "xs").or_else(|| list_f64(&q, "values"))?;
            let s = xs.iter().sum::<f64>();
            return Some(tag("sum_f64", "solver", format!("sum_f64: {s}")));
        }
        if try_name("product_f64") {
            let xs = list_f64(&q, "xs").or_else(|| list_f64(&q, "values"))?;
            let p = xs.iter().fold(1.0, |a, b| a * b);
            return Some(tag("product_f64", "solver", format!("product_f64: {p}")));
        }
        if try_name("min_f64") {
            let xs = list_f64(&q, "xs").or_else(|| list_f64(&q, "values"))?;
            let m = xs.iter().cloned().fold(f64::INFINITY, f64::min);
            return Some(tag("min_f64", "solver", format!("min_f64: {m}")));
        }
        if try_name("max_f64") {
            let xs = list_f64(&q, "xs").or_else(|| list_f64(&q, "values"))?;
            let m = xs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            return Some(tag("max_f64", "solver", format!("max_f64: {m}")));
        }
        if try_name("factorial_u64") {
            let n = param(&q, "n")? as u64;
            if n > 20 { return None; }
            let mut f: u64 = 1;
            for i in 2..=n { f = f.saturating_mul(i); }
            return Some(tag("factorial_u64", "solver", format!("factorial_u64: {n}!={f}")));
        }
        if try_name("lcs_length") {
            let a = param_str(query, "a").or_else(|| param_str(query, "s1"))?;
            let b = param_str(query, "b").or_else(|| param_str(query, "s2"))?;
            let aa: Vec<char> = a.chars().collect();
            let bb: Vec<char> = b.chars().collect();
            let mut prev = vec![0usize; bb.len() + 1];
            let mut cur = vec![0usize; bb.len() + 1];
            for i in 1..=aa.len() {
                for j in 1..=bb.len() {
                    cur[j] = if aa[i - 1] == bb[j - 1] { prev[j - 1] + 1 } else { prev[j].max(cur[j - 1]) };
                }
                std::mem::swap(&mut prev, &mut cur);
                cur.fill(0);
            }
            return Some(tag("lcs_length", "solver", format!("lcs_length: {}", prev[bb.len()])));
        }
        if try_name("matrix_add_2x2") {
            let a = list_f64(&q, "a")?;
            let b = list_f64(&q, "b")?;
            if a.len() != 4 || b.len() != 4 { return None; }
            let c: Vec<f64> = a.iter().zip(b.iter()).map(|(x, y)| x + y).collect();
            return Some(tag("matrix_add_2x2", "solver", format!("matrix_add_2x2: [{},{},{},{}]", c[0], c[1], c[2], c[3])));
        }
        if try_name("inv_2x2") {
            let a = list_f64(&q, "a").or_else(|| list_f64(&q, "m"))?;
            if a.len() != 4 { return None; }
            let det = a[0] * a[3] - a[1] * a[2];
            if det.abs() < 1e-15 {
                return Some(tag("inv_2x2", "solver", "inv_2x2: singular".into()));
            }
            let inv = [a[3] / det, -a[1] / det, -a[2] / det, a[0] / det];
            return Some(tag("inv_2x2", "solver", format!("inv_2x2: [{},{},{},{}]", inv[0], inv[1], inv[2], inv[3])));
        }
        if try_name("cross_3d") {
            let a = list_f64(&q, "a")?;
            let b = list_f64(&q, "b")?;
            if a.len() != 3 || b.len() != 3 { return None; }
            let c = [
                a[1] * b[2] - a[2] * b[1],
                a[2] * b[0] - a[0] * b[2],
                a[0] * b[1] - a[1] * b[0],
            ];
            return Some(tag("cross_3d", "solver", format!("cross_3d: [{},{},{}]", c[0], c[1], c[2])));
        }
        if try_name("pearson_corr") {
            let xs = list_f64(&q, "x").or_else(|| list_f64(&q, "xs"))?;
            let ys = list_f64(&q, "y").or_else(|| list_f64(&q, "ys"))?;
            if xs.len() != ys.len() || xs.len() < 2 { return None; }
            let n = xs.len() as f64;
            let mx = xs.iter().sum::<f64>() / n;
            let my = ys.iter().sum::<f64>() / n;
            let mut num = 0.0;
            let mut dx = 0.0;
            let mut dy = 0.0;
            for (x, y) in xs.iter().zip(ys.iter()) {
                let a = x - mx;
                let b = y - my;
                num += a * b;
                dx += a * a;
                dy += b * b;
            }
            if dx == 0.0 || dy == 0.0 { return None; }
            let r = num / (dx.sqrt() * dy.sqrt());
            return Some(tag("pearson_corr", "solver", format!("pearson_corr: r={r}")));
        }
        if try_name("merge_sorted") {
            let a = list_f64(&q, "a").or_else(|| list_f64(&q, "xs"))?;
            let b = list_f64(&q, "b").or_else(|| list_f64(&q, "ys"))?;
            let mut i = 0usize;
            let mut j = 0usize;
            let mut out = Vec::with_capacity(a.len() + b.len());
            while i < a.len() && j < b.len() {
                if a[i] <= b[j] { out.push(a[i]); i += 1; } else { out.push(b[j]); j += 1; }
            }
            out.extend_from_slice(&a[i..]);
            out.extend_from_slice(&b[j..]);
            return Some(tag("merge_sorted", "solver", format!("merge_sorted: {out:?}")));
        }
        if try_name("next_prime") {
            let mut n = param(&q, "n").or_else(|| param(&q, "x"))? as u64;
            if n <= 2 { return Some(tag("next_prime", "solver", "next_prime: 2".into())); }
            if n % 2 == 0 { n += 1; }
            'outer: loop {
                let mut d = 3u64;
                while d * d <= n {
                    if n % d == 0 { n += 2; continue 'outer; }
                    d += 2;
                }
                return Some(tag("next_prime", "solver", format!("next_prime: {n}")));
            }
        }
        if try_name("combinations_u64") {
            let n = param(&q, "n")? as u64;
            let k = param(&q, "k")? as u64;
            if n > 20 || k > n { return None; }
            let mut p: u64 = 1;
            for i in 0..k { p = p.saturating_mul(n - i); }
            return Some(tag("combinations_u64", "solver", format!("combinations_u64: P({n},{k})={p}")));
        }
        if try_name("dfs_reach") {
            // edges as pairs in flat list edges=[u,v,u,v,...]; start=; goal=
            let edges = list_f64(&q, "edges")?;
            let start = param(&q, "start").or_else(|| param(&q, "s"))? as i64;
            let goal = param(&q, "goal").or_else(|| param(&q, "g")).or_else(|| param(&q, "t"))? as i64;
            let mut adj: std::collections::BTreeMap<i64, Vec<i64>> = std::collections::BTreeMap::new();
            for chunk in edges.chunks(2) {
                if chunk.len() < 2 { break; }
                let u = chunk[0] as i64;
                let v = chunk[1] as i64;
                adj.entry(u).or_default().push(v);
                adj.entry(v).or_default().push(u);
            }
            let mut seen = BTreeSet::new();
            let mut stack = vec![start];
            while let Some(u) = stack.pop() {
                if !seen.insert(u) { continue; }
                if u == goal {
                    return Some(tag("dfs_reach", "solver", format!("dfs_reach: {start}→{goal} reachable=true")));
                }
                if let Some(nei) = adj.get(&u) {
                    for &v in nei { stack.push(v); }
                }
            }
            return Some(tag("dfs_reach", "solver", format!("dfs_reach: {start}→{goal} reachable=false")));
        }
        if try_name("set_symmetric_diff") {
            let a = list_str(query, "a").or_else(|| list_str(query, "xs"))?;
            let b = list_str(query, "b").or_else(|| list_str(query, "ys"))?;
            let sa: BTreeSet<_> = a.into_iter().collect();
            let sb: BTreeSet<_> = b.into_iter().collect();
            let mut out: Vec<_> = sa.symmetric_difference(&sb).cloned().collect();
            out.sort();
            return Some(tag("set_symmetric_diff", "solver", format!("set_symmetric_diff: {out:?}")));
        }


        if try_name("cumsum_f64") {
            let xs = list_f64(&q, "xs").or_else(|| list_f64(&q, "a"))?;
            let mut acc = 0.0;
            let out: Vec<f64> = xs.into_iter().map(|x| { acc += x; acc }).collect();
            return Some(tag("cumsum_f64", "solver", format!("cumsum_f64: {out:?}")));
        }
        if try_name("cumprod_f64") {
            let xs = list_f64(&q, "xs").or_else(|| list_f64(&q, "a"))?;
            let mut acc = 1.0;
            let out: Vec<f64> = xs.into_iter().map(|x| { acc *= x; acc }).collect();
            return Some(tag("cumprod_f64", "solver", format!("cumprod_f64: {out:?}")));
        }
        if try_name("percentile_f64") {
            let mut xs = list_f64(&q, "xs").or_else(|| list_f64(&q, "a"))?;
            let p = param(&q, "p").or_else(|| param(&q, "pct")).unwrap_or(50.0);
            if xs.is_empty() || !(0.0..=100.0).contains(&p) { return None; }
            xs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let k = ((p / 100.0) * (xs.len() as f64 - 1.0)).round() as usize;
            let v = xs[k.min(xs.len() - 1)];
            return Some(tag("percentile_f64", "solver", format!("percentile_f64: p{p}={v}")));
        }
        if try_name("zscore_f64") {
            let xs = list_f64(&q, "xs").or_else(|| list_f64(&q, "a"))?;
            let x = param(&q, "x")?;
            if xs.len() < 2 { return None; }
            let mean = xs.iter().sum::<f64>() / xs.len() as f64;
            let var = xs.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / xs.len() as f64;
            let sd = var.sqrt();
            if sd == 0.0 { return None; }
            return Some(tag("zscore_f64", "solver", format!("zscore_f64: z=({x}-{mean})/{sd}={}", (x - mean) / sd)));
        }
        if try_name("matmul_vec_2") {
            let a = param(&q, "a")?; let b = param(&q, "b")?;
            let c = param(&q, "c")?; let d = param(&q, "d")?;
            let x = param(&q, "x")?; let y = param(&q, "y")?;
            return Some(tag("matmul_vec_2", "solver", format!("matmul_vec_2: [{}, {}]", a * x + b * y, c * x + d * y)));
        }
        if try_name("norm_inf") {
            let xs = list_f64(&q, "xs").or_else(|| list_f64(&q, "a"))?;
            let m = xs.iter().map(|v| v.abs()).fold(0.0_f64, f64::max);
            return Some(tag("norm_inf", "solver", format!("norm_inf: {m}")));
        }
        if try_name("angle_between_2d") {
            let x1 = param(&q, "x1")?; let y1 = param(&q, "y1")?;
            let x2 = param(&q, "x2")?; let y2 = param(&q, "y2")?;
            let n1 = (x1 * x1 + y1 * y1).sqrt();
            let n2 = (x2 * x2 + y2 * y2).sqrt();
            if n1 == 0.0 || n2 == 0.0 { return None; }
            let cos = ((x1 * x2 + y1 * y2) / (n1 * n2)).clamp(-1.0, 1.0);
            let ang = cos.acos().to_degrees();
            return Some(tag("angle_between_2d", "solver", format!("angle_between_2d: {ang} deg")));
        }
        if try_name("polygon_area") {
            let pts = list_f64(&q, "pts").or_else(|| list_f64(&q, "xy"))?;
            if pts.len() < 6 || pts.len() % 2 != 0 { return None; }
            let n = pts.len() / 2;
            let mut area = 0.0;
            for i in 0..n {
                let j = (i + 1) % n;
                area += pts[2 * i] * pts[2 * j + 1];
                area -= pts[2 * j] * pts[2 * i + 1];
            }
            area = area.abs() / 2.0;
            return Some(tag("polygon_area", "solver", format!("polygon_area: {area}")));
        }
        if try_name("edit_script_len") {
            let a = param_str(query, "a").or_else(|| param_str(query, "s1"))?;
            let b = param_str(query, "b").or_else(|| param_str(query, "s2"))?;
            let aa: Vec<char> = a.chars().collect();
            let bb: Vec<char> = b.chars().collect();
            let mut dp = vec![vec![0usize; bb.len() + 1]; aa.len() + 1];
            for i in 0..=aa.len() { dp[i][0] = i; }
            for j in 0..=bb.len() { dp[0][j] = j; }
            for i in 1..=aa.len() {
                for j in 1..=bb.len() {
                    let cost = if aa[i - 1] == bb[j - 1] { 0 } else { 1 };
                    dp[i][j] = (dp[i - 1][j] + 1).min(dp[i][j - 1] + 1).min(dp[i - 1][j - 1] + cost);
                }
            }
            return Some(tag("edit_script_len", "solver", format!("edit_script_len: {}", dp[aa.len()][bb.len()])));
        }
        if try_name("longest_run") {
            let s = param_str(query, "s").or_else(|| param_str(query, "text"))?;
            let mut best = 0usize;
            let mut cur = 0usize;
            let mut prev: Option<char> = None;
            for c in s.chars() {
                if Some(c) == prev { cur += 1; } else { cur = 1; prev = Some(c); }
                best = best.max(cur);
            }
            return Some(tag("longest_run", "solver", format!("longest_run: {best}")));
        }
        if try_name("rle_decode") {
            let counts = list_f64(&q, "counts").or_else(|| list_f64(&q, "n"))?;
            let chars = list_str(query, "chars").or_else(|| list_str(query, "c"))?;
            if counts.len() != chars.len() { return None; }
            let mut out = String::new();
            for (n, ch) in counts.iter().zip(chars.iter()) {
                let k = (*n as usize).min(10_000);
                let c = ch.chars().next().unwrap_or('?');
                out.extend(std::iter::repeat(c).take(k));
            }
            return Some(tag("rle_decode", "solver", format!("rle_decode: {out}")));
        }
        if try_name("top_k_f64") {
            let mut xs = list_f64(&q, "xs").or_else(|| list_f64(&q, "a"))?;
            let k = param(&q, "k").unwrap_or(3.0) as usize;
            xs.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
            xs.truncate(k.min(xs.len()));
            return Some(tag("top_k_f64", "solver", format!("top_k_f64: {xs:?}")));
        }
        if try_name("argsort_f64") {
            let xs = list_f64(&q, "xs").or_else(|| list_f64(&q, "a"))?;
            let mut idx: Vec<usize> = (0..xs.len()).collect();
            idx.sort_by(|&i, &j| xs[i].partial_cmp(&xs[j]).unwrap_or(std::cmp::Ordering::Equal));
            return Some(tag("argsort_f64", "solver", format!("argsort_f64: {idx:?}")));
        }
        if try_name("is_palindrome") {
            let s = param_str(query, "s").or_else(|| param_str(query, "text"))?;
            let chars: Vec<char> = s.chars().filter(|c| !c.is_whitespace()).collect();
            let ok = chars.iter().eq(chars.iter().rev());
            return Some(tag("is_palindrome", "solver", format!("is_palindrome: {ok}")));
        }
        if try_name("anagram_check") {
            let a = param_str(query, "a")?;
            let b = param_str(query, "b")?;
            let mut aa: Vec<char> = a.chars().filter(|c| !c.is_whitespace()).map(|c| c.to_ascii_lowercase()).collect();
            let mut bb: Vec<char> = b.chars().filter(|c| !c.is_whitespace()).map(|c| c.to_ascii_lowercase()).collect();
            aa.sort_unstable(); bb.sort_unstable();
            return Some(tag("anagram_check", "solver", format!("anagram_check: {}", aa == bb)));
        }
        if try_name("set_issubset") {
            let a: BTreeSet<_> = list_str(query, "a")?.into_iter().collect();
            let b: BTreeSet<_> = list_str(query, "b")?.into_iter().collect();
            return Some(tag("set_issubset", "solver", format!("set_issubset: {}", a.is_subset(&b))));
        }
        if try_name("set_cardinality") {
            let a: BTreeSet<_> = list_str(query, "a").or_else(|| list_str(query, "set"))?.into_iter().collect();
            return Some(tag("set_cardinality", "solver", format!("set_cardinality: {}", a.len())));
        }
        if try_name("power_set_size") {
            let a: BTreeSet<_> = list_str(query, "a").or_else(|| list_str(query, "set"))?.into_iter().collect();
            if a.len() > 20 { return None; }
            return Some(tag("power_set_size", "solver", format!("power_set_size: {}", 1u64 << a.len())));
        }
        if try_name("dijkstra_tiny") {
            let edges = list_f64(&q, "edges")?;
            let start = param(&q, "start").or_else(|| param(&q, "s"))? as i64;
            let goal = param(&q, "goal").or_else(|| param(&q, "g")).or_else(|| param(&q, "t"))? as i64;
            let mut adj: std::collections::BTreeMap<i64, Vec<(i64, f64)>> = std::collections::BTreeMap::new();
            for chunk in edges.chunks(3) {
                if chunk.len() < 3 { break; }
                adj.entry(chunk[0] as i64).or_default().push((chunk[1] as i64, chunk[2]));
            }
            let mut dist: std::collections::BTreeMap<i64, f64> = std::collections::BTreeMap::new();
            dist.insert(start, 0.0);
            let mut visited = BTreeSet::new();
            for _ in 0..64 {
                let mut best: Option<(i64, f64)> = None;
                for (&n, &d) in &dist {
                    if visited.contains(&n) { continue; }
                    if best.map(|(_, bd)| d < bd).unwrap_or(true) { best = Some((n, d)); }
                }
                let Some((u, du)) = best else { break; };
                if u == goal {
                    return Some(tag("dijkstra_tiny", "solver", format!("dijkstra_tiny: dist={du}")));
                }
                visited.insert(u);
                if let Some(nei) = adj.get(&u) {
                    for &(v, w) in nei {
                        let nd = du + w;
                        let cur = dist.get(&v).copied().unwrap_or(f64::INFINITY);
                        if nd < cur { dist.insert(v, nd); }
                    }
                }
            }
            return Some(tag("dijkstra_tiny", "solver", format!("dijkstra_tiny: unreachable {start}→{goal}")));
        }
        if try_name("topo_sort_tiny") {
            let edges = list_f64(&q, "edges")?;
            let mut indeg: std::collections::BTreeMap<i64, i32> = std::collections::BTreeMap::new();
            let mut adj: std::collections::BTreeMap<i64, Vec<i64>> = std::collections::BTreeMap::new();
            for chunk in edges.chunks(2) {
                if chunk.len() < 2 { break; }
                let u = chunk[0] as i64; let v = chunk[1] as i64;
                adj.entry(u).or_default().push(v);
                indeg.entry(v).or_default();
                indeg.entry(u).or_default();
                *indeg.entry(v).or_default() += 1;
            }
            let mut q: VecDeque<i64> = indeg.iter().filter(|(_, d)| **d == 0).map(|(&n, _)| n).collect();
            let mut out = Vec::new();
            while let Some(u) = q.pop_front() {
                out.push(u);
                if let Some(nei) = adj.get(&u) {
                    for &v in nei {
                        if let Some(d) = indeg.get_mut(&v) {
                            *d -= 1;
                            if *d == 0 { q.push_back(v); }
                        }
                    }
                }
            }
            if out.len() != indeg.len() {
                return Some(tag("topo_sort_tiny", "solver", "topo_sort_tiny: refuse cycle".into()));
            }
            return Some(tag("topo_sort_tiny", "solver", format!("topo_sort_tiny: {out:?}")));
        }
        if try_name("binary_gcd_steps") {
            let mut a = param(&q, "a")? as u64;
            let mut b = param(&q, "b")? as u64;
            let mut steps = 0u64;
            while a != 0 && b != 0 {
                steps += 1;
                if a > b { a %= b; } else { b %= a; }
                if steps > 10_000 { break; }
            }
            return Some(tag("binary_gcd_steps", "solver", format!("binary_gcd_steps: steps={steps} gcd={}", a | b)));
        }
        if try_name("mod_pow_u64") {
            let mut base = param(&q, "base").or_else(|| param(&q, "a"))? as u128;
            let mut exp = param(&q, "exp").or_else(|| param(&q, "e"))? as u64;
            let m = param(&q, "mod").or_else(|| param(&q, "m"))? as u128;
            if m == 0 { return None; }
            let mut r: u128 = 1;
            base %= m;
            while exp > 0 {
                if exp & 1 == 1 { r = (r * base) % m; }
                base = (base * base) % m;
                exp >>= 1;
            }
            return Some(tag("mod_pow_u64", "solver", format!("mod_pow_u64: {r}")));
        }
        if try_name("chinese_remainder_2") {
            let a1 = param(&q, "a1")? as i64;
            let n1 = param(&q, "n1")? as i64;
            let a2 = param(&q, "a2")? as i64;
            let n2 = param(&q, "n2")? as i64;
            if n1 <= 0 || n2 <= 0 { return None; }
            let g = { let mut x=n1.abs(); let mut y=n2.abs(); while y!=0 { let t=x%y; x=y; y=t;} x };
            if g != 1 { return Some(tag("chinese_remainder_2", "solver", "chinese_remainder_2: refuse non-coprime".into())); }
            let mut inv = 0i64; let mut b = 1i64;
            let (mut aa, mut mm) = (n1.rem_euclid(n2), n2);
            while aa > 1 {
                let q = aa / mm;
                let t = mm; mm = aa % mm; aa = t;
                let t = inv; inv = b - q * inv; b = t;
            }
            if inv < 0 { inv += n2; }
            let x = a1 + n1 * ((a2 - a1).rem_euclid(n2) * inv).rem_euclid(n2);
            return Some(tag("chinese_remainder_2", "solver", format!("chinese_remainder_2: x≡{x} (mod {})", n1 * n2)));
        }
        if try_name("interval_union_len") {
            let pts = list_f64(&q, "intervals").or_else(|| list_f64(&q, "iv"))?;
            if pts.len() % 2 != 0 || pts.is_empty() { return None; }
            let mut iv: Vec<(f64, f64)> = pts.chunks(2).map(|c| (c[0].min(c[1]), c[0].max(c[1]))).collect();
            iv.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
            let mut cur_s = iv[0].0; let mut cur_e = iv[0].1; let mut total = 0.0;
            for &(s, e) in iv.iter().skip(1) {
                if s <= cur_e { cur_e = cur_e.max(e); }
                else { total += cur_e - cur_s; cur_s = s; cur_e = e; }
            }
            total += cur_e - cur_s;
            return Some(tag("interval_union_len", "solver", format!("interval_union_len: {total}")));
        }
        if try_name("knapsack_unbounded_tiny") {
            let wts = list_f64(&q, "weights").or_else(|| list_f64(&q, "w"))?;
            let vals = list_f64(&q, "values").or_else(|| list_f64(&q, "v"))?;
            let cap = param(&q, "cap").or_else(|| param(&q, "capacity"))? as usize;
            if wts.len() != vals.len() || wts.len() > 6 || cap > 200 { return None; }
            let mut dp = vec![0.0; cap + 1];
            for c in 0..=cap {
                for i in 0..wts.len() {
                    let w = wts[i] as usize;
                    if w <= c { let cand = dp[c - w] + vals[i]; if cand > dp[c] { dp[c] = cand; } }
                }
            }
            return Some(tag("knapsack_unbounded_tiny", "solver", format!("knapsack_unbounded_tiny: max={}", dp[cap])));
        }
        if try_name("linear_interp_table") {
            let xs = list_f64(&q, "xs")?;
            let ys = list_f64(&q, "ys")?;
            let x = param(&q, "x")?;
            if xs.len() != ys.len() || xs.len() < 2 { return None; }
            if x <= xs[0] { return Some(tag("linear_interp_table", "solver", format!("linear_interp_table: {}", ys[0]))); }
            if x >= xs[xs.len()-1] { return Some(tag("linear_interp_table", "solver", format!("linear_interp_table: {}", ys[ys.len()-1]))); }
            for i in 0..xs.len()-1 {
                if x >= xs[i] && x <= xs[i+1] {
                    let t = (x - xs[i]) / (xs[i+1] - xs[i]);
                    let y = ys[i] * (1.0 - t) + ys[i+1] * t;
                    return Some(tag("linear_interp_table", "solver", format!("linear_interp_table: {y}")));
                }
            }
            return None;
        }

        None
    }
}

impl GrammarCoverage for CatalogSolver {
    fn tier(&self) -> CascadeTier {
        CascadeTier::Solver
    }

    fn covers(&self, req: &MolRequest) -> bool {
        Self::eval(&req.query).is_some()
    }

    fn try_answer(&self, req: &MolRequest) -> Result<TierAnswer> {
        Self::eval(&req.query)
            .map(TierAnswer::text)
            .ok_or_else(|| MolError::NotCovered(format!("catalog solver miss: {}", req.query)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mol_core::Budget;

    #[test]
    fn formula_pythagoras_live() {
        let g = CatalogFormula;
        let a = g
            .try_answer(&MolRequest::new("pythagoras a=3 b=4", Budget::demo()))
            .unwrap();
        assert!(a.text.contains('5') || a.text.contains("5.0"), "{}", a.text);
    }

    #[test]
    fn lookup_bool_and_live() {
        let g = CatalogLookup;
        let a = g
            .try_answer(&MolRequest::new("bool_and a=true b=false", Budget::demo()))
            .unwrap();
        assert!(a.text.contains("false"), "{}", a.text);
    }

    #[test]
    fn solver_mean_live() {
        let g = CatalogSolver;
        let a = g
            .try_answer(&MolRequest::new("mean_f64 xs=[1,2,3,4]", Budget::demo()))
            .unwrap();
        assert!(a.text.contains('2') || a.text.contains("2.5"), "{}", a.text);
    }

    #[test]
    fn live_counts_match_stack() {
        let s = PeriodicStack::subset();
        assert!(s.live_gear_count() >= 250);
        assert!(s.scale_note().contains("live catalog"));
    }
}
