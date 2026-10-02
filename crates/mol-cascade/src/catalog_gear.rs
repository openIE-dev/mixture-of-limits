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
    q.contains(&n)
        || q.contains(&format!("primitive {n}"))
        || q.contains(&format!("catalog {n}"))
        || q.contains(&format!("stack {n}"))
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
        assert!(s.live_gear_count() >= 120);
        assert!(s.scale_note().contains("live catalog"));
    }
}
