//! Capsule runtime — sandboxed WASM invoke under MoL encapsulation bounds.
//!
//! Default path: in-process stub that validates a real `.wasm` fixture and
//! executes a tiny fuel-limited interpreter (no FS/net). Optional `wasmtime`
//! feature swaps in Wasmtime with fuel + no WASI.
//!
//! Energy honesty: fuel → **estimated** joules only. `measured_j` stays `None`
//! unless a real meter is attached elsewhere — never invent measured from fuel.

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::compute::ComputeStepReceipt;
use crate::encapsulation::{CapsuleBoundary, CapsuleContext, EncapsulationReceipt};
use crate::energy::{EstimateKind, Joules, MeasureSource};
use crate::fabric::DeviceKind;
use crate::floor::{Floor, FloorKind};
use crate::honesty::EnergyHonestyClass;

/// Embedded fixture: `(module (func (export "add") (param i32 i32) (result i32)
///   local.get 0 local.get 1 i32.add))` — 41 bytes, no imports.
pub const FIXTURE_ADD_WASM: &[u8] = include_bytes!("../fixtures/add.wasm");

/// Analytical joules-per-fuel unit (estimate floor; not a silicon meter).
pub const JOULES_PER_FUEL: f64 = 1e-15;

/// Host capabilities a capsule may request. Default: none granted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CapsuleGrant {
    /// No host escape.
    #[default]
    None,
    /// Filesystem access (denied unless explicitly granted — prove refuses).
    Fs,
    /// Network access (denied unless explicitly granted — prove refuses).
    Net,
}

impl CapsuleGrant {
    /// Wire label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Fs => "fs",
            Self::Net => "net",
        }
    }
}

impl fmt::Display for CapsuleGrant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// Hard bounds for a capsule invoke.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapsuleBounds {
    /// Max WASM fuel units (instructions / approximate cost).
    pub max_fuel: u64,
    /// Max linear memory pages (64 KiB each). Stub enforces 0–1.
    pub max_memory_pages: u32,
    /// Granted host capabilities (empty = sealed / no FS/net).
    pub grants: Vec<CapsuleGrant>,
}

impl CapsuleBounds {
    /// Tight soft-ref defaults: fuel budget, one page, no host grants.
    pub fn sealed_default() -> Self {
        Self {
            max_fuel: 10_000,
            max_memory_pages: 1,
            grants: vec![],
        }
    }

    /// True when FS or Net is in the grant list.
    pub fn requests_host_escape(&self) -> bool {
        self.grants
            .iter()
            .any(|g| matches!(g, CapsuleGrant::Fs | CapsuleGrant::Net))
    }
}

impl Default for CapsuleBounds {
    fn default() -> Self {
        Self::sealed_default()
    }
}

/// Invoke request against a WASM module under capsule context.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapsuleInvoke {
    /// Capsule context (must be sealed + encapsulated when fail-closed).
    pub context: CapsuleContext,
    /// Bounds / grants.
    pub bounds: CapsuleBounds,
    /// Raw WASM module bytes.
    pub module: Vec<u8>,
    /// Exported function name.
    pub export: String,
    /// i32 arguments (fixture `add` takes two).
    pub args: Vec<i32>,
}

impl CapsuleInvoke {
    /// Build sealed wasm-module invoke of the embedded `add` fixture.
    pub fn sealed_add_fixture(a: i32, b: i32) -> Self {
        Self {
            context: CapsuleContext {
                id: crate::encapsulation::CapsuleId::new("mol:fixture:add@0.1.0"),
                boundary: CapsuleBoundary::WasmModule,
                sealed: true,
                shares_host_session: false,
            },
            bounds: CapsuleBounds::sealed_default(),
            module: FIXTURE_ADD_WASM.to_vec(),
            export: "add".into(),
            args: vec![a, b],
        }
    }
}

/// Commit | refuse outcome of a capsule certify path.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapsuleDecision {
    /// Capsule ran within bounds; receipts stamped.
    Commit,
    /// Capsule refused (encapsulation / fuel / grant / trap).
    Refuse,
}

/// Certify result: decision + encapsulation + compute steps + energy honesty.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapsuleCertifyResult {
    /// Commit or refuse.
    pub decision: CapsuleDecision,
    /// Floor when refused.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub floor: Option<Floor>,
    /// Encapsulation stamp.
    pub encapsulation: EncapsulationReceipt,
    /// Compute step receipts (fuel estimate; measured_j always None here).
    pub compute_steps: Vec<ComputeStepReceipt>,
    /// Estimated joules from fuel (analytical).
    pub estimated_j: Joules,
    /// Always None on the capsule fuel path — never invent measured from fuel.
    pub measured_j: Option<Joules>,
    /// Measure source (cascade/catalog estimate for fuel).
    pub measure_source: MeasureSource,
    /// Honesty class (Estimated for fuel path).
    pub energy_honesty: EnergyHonestyClass,
    /// Estimate kind label.
    pub estimate_kind: EstimateKind,
    /// Fuel consumed (when invoke ran).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fuel_consumed: Option<u64>,
    /// i32 return value when committed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub return_i32: Option<i32>,
    /// Runtime backend label (`stub` | `wasmtime`).
    pub runtime: String,
    /// Human rationale.
    pub rationale: String,
}

impl CapsuleCertifyResult {
    /// True when decision is Commit.
    pub fn is_commit(&self) -> bool {
        matches!(self.decision, CapsuleDecision::Commit)
    }

    /// Energy pair honesty (fuel path must keep measured_j=None).
    pub fn honesty_ok(&self) -> bool {
        crate::honesty::energy_pair_honest(self.measured_j, self.measure_source)
            && self.measured_j.is_none()
            && self.compute_steps.iter().all(|s| s.honesty_ok() && s.measured_j.is_none())
    }
}

/// Trait for capsule WASM runtimes (stub or wasmtime).
pub trait CapsuleRuntime {
    /// Backend label.
    fn name(&self) -> &'static str;

    /// Run module under bounds; stamp receipts; commit|refuse fail-closed.
    fn certify(&self, invoke: &CapsuleInvoke) -> CapsuleCertifyResult;
}

fn refuse(
    invoke: &CapsuleInvoke,
    runtime: &str,
    floor: Floor,
    rationale: impl Into<String>,
) -> CapsuleCertifyResult {
    CapsuleCertifyResult {
        decision: CapsuleDecision::Refuse,
        floor: Some(floor),
        encapsulation: EncapsulationReceipt::from(&invoke.context),
        compute_steps: vec![ComputeStepReceipt::estimated(
            "capsule:refuse",
            Some(DeviceKind::WasmBrowser),
            Joules::ZERO,
        )
        .with_capsule(invoke.context.id.as_str())],
        estimated_j: Joules::ZERO,
        measured_j: None,
        measure_source: MeasureSource::CascadeEstimate,
        energy_honesty: EnergyHonestyClass::Estimated,
        estimate_kind: EstimateKind::Analytical,
        fuel_consumed: None,
        return_i32: None,
        runtime: runtime.into(),
        rationale: rationale.into(),
    }
}

fn commit(
    invoke: &CapsuleInvoke,
    runtime: &str,
    fuel: u64,
    ret: i32,
    rationale: impl Into<String>,
) -> CapsuleCertifyResult {
    let estimated_j = Joules::new(fuel as f64 * JOULES_PER_FUEL);
    let step = ComputeStepReceipt {
        label: "capsule:invoke".into(),
        fabric: Some(DeviceKind::WasmBrowser),
        fabric_id: Some(DeviceKind::WasmBrowser.label().into()),
        unavailable_reason: None,
        estimated_j,
        estimate_kind: EstimateKind::Fixture,
        measured_j: None,
        measure_source: MeasureSource::CascadeEstimate,
        honesty: EnergyHonestyClass::Estimated,
        capsule_id: Some(invoke.context.id.as_str().to_string()),
        execution_proof: None,
    };
    let fuel_step = ComputeStepReceipt {
        label: format!("capsule:fuel:{fuel}"),
        fabric: Some(DeviceKind::WasmBrowser),
        fabric_id: Some(DeviceKind::WasmBrowser.label().into()),
        unavailable_reason: None,
        estimated_j,
        estimate_kind: EstimateKind::Analytical,
        measured_j: None,
        measure_source: MeasureSource::CascadeEstimate,
        honesty: EnergyHonestyClass::Estimated,
        capsule_id: Some(invoke.context.id.as_str().to_string()),
        execution_proof: None,
    };
    CapsuleCertifyResult {
        decision: CapsuleDecision::Commit,
        floor: None,
        encapsulation: EncapsulationReceipt::from(&invoke.context),
        compute_steps: vec![step, fuel_step],
        estimated_j,
        measured_j: None,
        measure_source: MeasureSource::CascadeEstimate,
        energy_honesty: EnergyHonestyClass::Estimated,
        estimate_kind: EstimateKind::Fixture,
        fuel_consumed: Some(fuel),
        return_i32: Some(ret),
        runtime: runtime.into(),
        rationale: rationale.into(),
    }
}

/// Preflight fail-closed checks shared by stub and wasmtime.
fn preflight(invoke: &CapsuleInvoke, runtime: &str) -> Option<CapsuleCertifyResult> {
    // Host escape grants are never auto-allowed on the soft-ref certify path.
    if invoke.bounds.requests_host_escape() {
        return Some(refuse(
            invoke,
            runtime,
            Floor::new(
                "capsule_grant_denied",
                FloorKind::Encapsulation,
                format!(
                    "capsule '{}' requested FS/net grant — refused (no FS/net unless granted; soft-ref denies)",
                    invoke.context.id
                ),
            ),
            "fail-closed: FS/net grant not permitted on sealed capsule path",
        ));
    }
    if let Err(floor) = invoke.context.check(true) {
        return Some(refuse(
            invoke,
            runtime,
            floor,
            "fail-closed: capsule encapsulation check failed",
        ));
    }
    if invoke.module.len() < 8 || &invoke.module[0..4] != b"\0asm" {
        return Some(refuse(
            invoke,
            runtime,
            Floor::new(
                "capsule_invalid_wasm",
                FloorKind::Encapsulation,
                "module missing WASM magic",
            ),
            "fail-closed: invalid WASM module",
        ));
    }
    if invoke.module[4..8] != [0x01, 0x00, 0x00, 0x00] {
        return Some(refuse(
            invoke,
            runtime,
            Floor::new(
                "capsule_invalid_wasm",
                FloorKind::Encapsulation,
                "unsupported WASM version",
            ),
            "fail-closed: unsupported WASM version",
        ));
    }
    None
}

/// In-process stub runtime: validates real WASM bytes and interprets a tiny
/// subset (local.get / i32.add / end) under fuel. No FS/net.
#[derive(Debug, Default, Clone, Copy)]
pub struct StubCapsuleRuntime;

impl CapsuleRuntime for StubCapsuleRuntime {
    fn name(&self) -> &'static str {
        "stub"
    }

    fn certify(&self, invoke: &CapsuleInvoke) -> CapsuleCertifyResult {
        if let Some(r) = preflight(invoke, self.name()) {
            return r;
        }
        match interpret_i32_export(invoke) {
            Ok((fuel, ret)) => {
                if fuel > invoke.bounds.max_fuel {
                    return refuse(
                        invoke,
                        self.name(),
                        Floor::new(
                            "capsule_fuel_exceeded",
                            FloorKind::Energy,
                            format!(
                                "fuel {fuel} exceeded max_fuel {}",
                                invoke.bounds.max_fuel
                            ),
                        ),
                        "fail-closed: capsule fuel exceeded",
                    );
                }
                commit(
                    invoke,
                    self.name(),
                    fuel,
                    ret,
                    format!(
                        "stub capsule invoke export='{}' args={:?} → {ret}; fuel={fuel}; estimated_j only",
                        invoke.export, invoke.args
                    ),
                )
            }
            Err(floor) => refuse(
                invoke,
                self.name(),
                floor,
                "fail-closed: stub capsule interpret refused",
            ),
        }
    }
}

/// Minimal WASM decoder + interpreter for pure i32 functions with
/// `local.get` / `i32.add` / `end` only. Returns (fuel_consumed, result).
fn interpret_i32_export(invoke: &CapsuleInvoke) -> Result<(u64, i32), Floor> {
    let bytes = &invoke.module;
    let mut types: Vec<(u32, u32)> = Vec::new(); // (n_params, n_results) for functype
    let mut func_type_idxs: Vec<u32> = Vec::new();
    let mut exports: Vec<(String, u32)> = Vec::new(); // name → func idx
    let mut bodies: Vec<Vec<u8>> = Vec::new();

    let mut i = 8usize; // after magic+version
    while i < bytes.len() {
        let id = bytes[i];
        i += 1;
        let (size, ni) = read_u32_leb(bytes, i)?;
        i = ni;
        let end = i + size as usize;
        if end > bytes.len() {
            return Err(Floor::new(
                "capsule_invalid_wasm",
                FloorKind::Encapsulation,
                "section truncated",
            ));
        }
        let section = &bytes[i..end];
        match id {
            1 => {
                // type
                let (count, mut p) = read_u32_leb(section, 0)?;
                for _ in 0..count {
                    if p >= section.len() || section[p] != 0x60 {
                        return Err(bad_wasm("expected functype"));
                    }
                    p += 1;
                    let (nparams, np) = read_u32_leb(section, p)?;
                    p = np + nparams as usize; // skip param types (must be i32=0x7f)
                    for k in 0..nparams {
                        let t = section
                            .get(np + k as usize)
                            .copied()
                            .ok_or_else(|| bad_wasm("param type truncated"))?;
                        if t != 0x7f {
                            return Err(bad_wasm("stub only supports i32 params"));
                        }
                    }
                    let (nresults, nr) = read_u32_leb(section, p)?;
                    p = nr;
                    for k in 0..nresults {
                        let t = section
                            .get(p + k as usize)
                            .copied()
                            .ok_or_else(|| bad_wasm("result type truncated"))?;
                        if t != 0x7f {
                            return Err(bad_wasm("stub only supports i32 results"));
                        }
                    }
                    p += nresults as usize;
                    types.push((nparams, nresults));
                }
            }
            3 => {
                let (count, mut p) = read_u32_leb(section, 0)?;
                for _ in 0..count {
                    let (idx, np) = read_u32_leb(section, p)?;
                    p = np;
                    func_type_idxs.push(idx);
                }
            }
            7 => {
                let (count, mut p) = read_u32_leb(section, 0)?;
                for _ in 0..count {
                    let (nlen, np) = read_u32_leb(section, p)?;
                    p = np;
                    let name = std::str::from_utf8(
                        section
                            .get(p..p + nlen as usize)
                            .ok_or_else(|| bad_wasm("export name truncated"))?,
                    )
                    .map_err(|_| bad_wasm("export name utf8"))?
                    .to_string();
                    p += nlen as usize;
                    let kind = *section.get(p).ok_or_else(|| bad_wasm("export kind"))?;
                    p += 1;
                    let (idx, np) = read_u32_leb(section, p)?;
                    p = np;
                    if kind == 0 {
                        exports.push((name, idx));
                    }
                }
            }
            10 => {
                let (count, mut p) = read_u32_leb(section, 0)?;
                for _ in 0..count {
                    let (body_size, np) = read_u32_leb(section, p)?;
                    p = np;
                    let body_end = p + body_size as usize;
                    let body = section
                        .get(p..body_end)
                        .ok_or_else(|| bad_wasm("code body truncated"))?
                        .to_vec();
                    bodies.push(body);
                    p = body_end;
                }
            }
            0 | 2 | 4 | 5 | 6 | 8 | 9 | 11 | 12 => {
                // custom / import / table / memory / global / start / element / data / datacount
                // Imports: refuse (no host bindings).
                if id == 2 {
                    let (count, _) = read_u32_leb(section, 0)?;
                    if count > 0 {
                        return Err(Floor::new(
                            "capsule_import_denied",
                            FloorKind::Encapsulation,
                            "WASM imports refused — sealed capsule has no host bindings",
                        ));
                    }
                }
            }
            _ => {
                return Err(bad_wasm(&format!("unsupported section id {id}")));
            }
        }
        i = end;
    }

    let func_idx = exports
        .iter()
        .find(|(n, _)| n == &invoke.export)
        .map(|(_, i)| *i)
        .ok_or_else(|| {
            Floor::new(
                "capsule_export_missing",
                FloorKind::Encapsulation,
                format!("export '{}' not found", invoke.export),
            )
        })?;
    let type_idx = *func_type_idxs
        .get(func_idx as usize)
        .ok_or_else(|| bad_wasm("func type idx OOB"))?;
    let (nparams, nresults) = *types
        .get(type_idx as usize)
        .ok_or_else(|| bad_wasm("type OOB"))?;
    if nresults != 1 {
        return Err(bad_wasm("stub expects single i32 result"));
    }
    if invoke.args.len() as u32 != nparams {
        return Err(Floor::new(
            "capsule_arity",
            FloorKind::Encapsulation,
            format!(
                "arity mismatch: export wants {nparams} args, got {}",
                invoke.args.len()
            ),
        ));
    }
    let body = bodies
        .get(func_idx as usize)
        .ok_or_else(|| bad_wasm("body OOB"))?;

    // Body: local_count (leb) then locals decls, then ops ... 0x0b
    let (nlocal_groups, mut p) = read_u32_leb(body, 0)?;
    let mut nlocals = 0u32;
    for _ in 0..nlocal_groups {
        let (n, np) = read_u32_leb(body, p)?;
        p = np;
        let _ty = *body.get(p).ok_or_else(|| bad_wasm("local type"))?;
        p += 1;
        nlocals = nlocals.saturating_add(n);
    }
    let mut locals: Vec<i32> = invoke.args.clone();
    locals.extend(std::iter::repeat(0).take(nlocals as usize));

    let mut stack: Vec<i32> = Vec::new();
    let mut fuel: u64 = 0;
    let max_fuel = invoke.bounds.max_fuel;
    while p < body.len() {
        let op = body[p];
        p += 1;
        fuel = fuel.saturating_add(1);
        if fuel > max_fuel {
            return Err(Floor::new(
                "capsule_fuel_exceeded",
                FloorKind::Energy,
                format!("fuel exceeded max_fuel {max_fuel}"),
            ));
        }
        match op {
            0x20 => {
                // local.get
                let (idx, np) = read_u32_leb(body, p)?;
                p = np;
                let v = *locals.get(idx as usize).ok_or_else(|| bad_wasm("local.get OOB"))?;
                stack.push(v);
            }
            0x6a => {
                // i32.add
                let b = stack.pop().ok_or_else(|| bad_wasm("i32.add under"))?;
                let a = stack.pop().ok_or_else(|| bad_wasm("i32.add under"))?;
                stack.push(a.wrapping_add(b));
            }
            0x0b => break, // end
            other => {
                return Err(Floor::new(
                    "capsule_opcode_denied",
                    FloorKind::Encapsulation,
                    format!("opcode 0x{other:02x} not allowed in sealed stub capsule"),
                ));
            }
        }
    }
    let ret = stack
        .pop()
        .ok_or_else(|| bad_wasm("empty result stack"))?;
    Ok((fuel, ret))
}

fn bad_wasm(msg: &str) -> Floor {
    Floor::new("capsule_invalid_wasm", FloorKind::Encapsulation, msg)
}

fn read_u32_leb(bytes: &[u8], mut i: usize) -> Result<(u32, usize), Floor> {
    let mut result = 0u32;
    let mut shift = 0u32;
    loop {
        let b = *bytes
            .get(i)
            .ok_or_else(|| bad_wasm("leb128 truncated"))?;
        i += 1;
        result |= u32::from(b & 0x7f) << shift;
        if b & 0x80 == 0 {
            return Ok((result, i));
        }
        shift += 7;
        if shift > 28 {
            return Err(bad_wasm("leb128 overflow"));
        }
    }
}

/// Default runtime for prove: stub (always available).
pub fn default_capsule_runtime() -> StubCapsuleRuntime {
    StubCapsuleRuntime
}

/// Certify a sealed fixture add under stub (happy path helper).
pub fn certify_sealed_add_fixture(a: i32, b: i32) -> CapsuleCertifyResult {
    StubCapsuleRuntime.certify(&CapsuleInvoke::sealed_add_fixture(a, b))
}

#[cfg(feature = "wasmtime")]
mod wasmtime_rt {
    use super::*;
    use wasmtime::{Config, Engine, Linker, Module, Store};

    /// Wasmtime-backed capsule runtime (fuel on; no WASI / no host imports).
    #[derive(Debug, Default, Clone, Copy)]
    pub struct WasmtimeCapsuleRuntime;

    impl CapsuleRuntime for WasmtimeCapsuleRuntime {
        fn name(&self) -> &'static str {
            "wasmtime"
        }

        fn certify(&self, invoke: &CapsuleInvoke) -> CapsuleCertifyResult {
            if let Some(r) = preflight(invoke, self.name()) {
                return r;
            }
            let mut config = Config::new();
            config.consume_fuel(true);
            let engine = match Engine::new(&config) {
                Ok(e) => e,
                Err(e) => {
                    return refuse(
                        invoke,
                        self.name(),
                        Floor::new(
                            "capsule_runtime_error",
                            FloorKind::Encapsulation,
                            format!("wasmtime engine: {e}"),
                        ),
                        "fail-closed: wasmtime engine init",
                    );
                }
            };
            let module = match Module::new(&engine, &invoke.module) {
                Ok(m) => m,
                Err(e) => {
                    return refuse(
                        invoke,
                        self.name(),
                        Floor::new(
                            "capsule_invalid_wasm",
                            FloorKind::Encapsulation,
                            format!("wasmtime module: {e}"),
                        ),
                        "fail-closed: wasmtime reject module",
                    );
                }
            };
            // Refuse modules with imports (no host bindings).
            if module.imports().next().is_some() {
                return refuse(
                    invoke,
                    self.name(),
                    Floor::new(
                        "capsule_import_denied",
                        FloorKind::Encapsulation,
                        "WASM imports refused — sealed capsule has no host bindings",
                    ),
                    "fail-closed: imports present",
                );
            }
            let linker = Linker::new(&engine);
            let mut store = Store::new(&engine, ());
            if let Err(e) = store.set_fuel(invoke.bounds.max_fuel) {
                return refuse(
                    invoke,
                    self.name(),
                    Floor::new(
                        "capsule_runtime_error",
                        FloorKind::Encapsulation,
                        format!("set_fuel: {e}"),
                    ),
                    "fail-closed: set_fuel",
                );
            }
            let instance = match linker.instantiate(&mut store, &module) {
                Ok(i) => i,
                Err(e) => {
                    return refuse(
                        invoke,
                        self.name(),
                        Floor::new(
                            "capsule_runtime_error",
                            FloorKind::Encapsulation,
                            format!("instantiate: {e}"),
                        ),
                        "fail-closed: instantiate",
                    );
                }
            };
            let func = match instance.get_typed_func::<(i32, i32), i32>(&mut store, &invoke.export)
            {
                Ok(f) => f,
                Err(e) => {
                    return refuse(
                        invoke,
                        self.name(),
                        Floor::new(
                            "capsule_export_missing",
                            FloorKind::Encapsulation,
                            format!("typed func '{}': {e}", invoke.export),
                        ),
                        "fail-closed: export missing / wrong type",
                    );
                }
            };
            if invoke.args.len() != 2 {
                return refuse(
                    invoke,
                    self.name(),
                    Floor::new(
                        "capsule_arity",
                        FloorKind::Encapsulation,
                        "wasmtime path currently expects 2 i32 args",
                    ),
                    "fail-closed: arity",
                );
            }
            let a = invoke.args[0];
            let b = invoke.args[1];
            match func.call(&mut store, (a, b)) {
                Ok(ret) => {
                    let remaining = store.get_fuel().unwrap_or(0);
                    let fuel = invoke.bounds.max_fuel.saturating_sub(remaining);
                    commit(
                        invoke,
                        self.name(),
                        fuel.max(1),
                        ret,
                        format!(
                            "wasmtime capsule invoke export='{}' → {ret}; fuel={fuel}; estimated_j only",
                            invoke.export
                        ),
                    )
                }
                Err(e) => {
                    let msg = format!("{e}");
                    let id = if msg.contains("fuel") {
                        "capsule_fuel_exceeded"
                    } else {
                        "capsule_trap"
                    };
                    let kind = if id == "capsule_fuel_exceeded" {
                        FloorKind::Energy
                    } else {
                        FloorKind::Encapsulation
                    };
                    refuse(
                        invoke,
                        self.name(),
                        Floor::new(id, kind, msg),
                        "fail-closed: wasmtime trap / fuel",
                    )
                }
            }
        }
    }

    /// Construct wasmtime runtime.
    pub fn wasmtime_capsule_runtime() -> WasmtimeCapsuleRuntime {
        WasmtimeCapsuleRuntime
    }
}

#[cfg(feature = "wasmtime")]
pub use wasmtime_rt::{wasmtime_capsule_runtime, WasmtimeCapsuleRuntime};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixture_add_commits_estimated_only() {
        let r = certify_sealed_add_fixture(2, 40);
        assert!(r.is_commit(), "{:?}", r);
        assert_eq!(r.return_i32, Some(42));
        assert!(r.measured_j.is_none());
        assert!(r.honesty_ok());
        assert!(r.encapsulation.sealed);
        assert!(!r.compute_steps.is_empty());
        assert_eq!(r.runtime, "stub");
    }

    #[test]
    fn host_share_refused() {
        let mut inv = CapsuleInvoke::sealed_add_fixture(1, 1);
        inv.context.shares_host_session = true;
        let r = StubCapsuleRuntime.certify(&inv);
        assert!(!r.is_commit());
        assert_eq!(
            r.floor.as_ref().map(|f| f.kind),
            Some(FloorKind::Encapsulation)
        );
    }

    #[test]
    fn fs_grant_refused() {
        let mut inv = CapsuleInvoke::sealed_add_fixture(1, 1);
        inv.bounds.grants.push(CapsuleGrant::Fs);
        let r = StubCapsuleRuntime.certify(&inv);
        assert!(!r.is_commit());
        assert_eq!(
            r.floor.as_ref().map(|f| f.id.as_str()),
            Some("capsule_grant_denied")
        );
    }

    #[test]
    fn fuel_exceeded_refused() {
        let mut inv = CapsuleInvoke::sealed_add_fixture(1, 1);
        inv.bounds.max_fuel = 1; // add uses local.get×2 + i32.add + end ≥ 3
        let r = StubCapsuleRuntime.certify(&inv);
        assert!(!r.is_commit());
        assert_eq!(
            r.floor.as_ref().map(|f| f.id.as_str()),
            Some("capsule_fuel_exceeded")
        );
    }

    #[test]
    fn unsealed_refused() {
        let mut inv = CapsuleInvoke::sealed_add_fixture(1, 1);
        inv.context.sealed = false;
        let r = StubCapsuleRuntime.certify(&inv);
        assert!(!r.is_commit());
    }

    #[test]
    fn fixture_bytes_are_real_wasm() {
        assert_eq!(&FIXTURE_ADD_WASM[0..4], b"\0asm");
        assert!(FIXTURE_ADD_WASM.len() >= 8);
    }

    #[cfg(feature = "wasmtime")]
    #[test]
    fn wasmtime_add_fixture_commits() {
        let r = WasmtimeCapsuleRuntime.certify(&CapsuleInvoke::sealed_add_fixture(3, 4));
        assert!(r.is_commit(), "{:?}", r);
        assert_eq!(r.return_i32, Some(7));
        assert!(r.measured_j.is_none());
        assert_eq!(r.runtime, "wasmtime");
        assert!(r.honesty_ok());
    }
}
