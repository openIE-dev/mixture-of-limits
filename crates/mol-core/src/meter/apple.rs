//! macOS component energy: IOReport rails + SMC `PSTR` package, then `powermetrics`.
//!
//! - **IOReport** Energy Model (private framework): CPU / GPU / ANE / DRAM rails
//!   without sudo. Exhaustive dumps (11k+ channels / 364 Energy Model) show **no**
//!   Package/Combined channel on current Apple Silicon — never invent by summing.
//! - **SMC `PSTR`**: system total power (watts) via IOKit `AppleSMC`, no sudo.
//!   Same sensor macmon exposes as `sys_power`. Integrated over the sample window
//!   → package `measured_j` with [`MeasureSource::Smc`].
//! - **`powermetrics`**: still needs root; provides `combined_power` when available.
//! - Unknown IOReport units are skipped — never scaled by a guessed factor.

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};
use std::ptr;
use std::thread;
use std::time::{Duration, Instant};

use super::{
    fold_named_joules, merge_rails_with_package, sample_from_smc_pstr_watts, MeterCapability,
    MeterSample, NamedJoule,
};
use crate::energy::MeasureSource;

const CF_STRING_UTF8: u32 = 0x0800_0100;

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    fn CFStringCreateWithCString(
        alloc: *const c_void,
        cstr: *const c_char,
        encoding: u32,
    ) -> *const c_void;
    fn CFStringGetCString(
        s: *const c_void,
        buf: *mut c_char,
        size: isize,
        encoding: u32,
    ) -> u8;
    fn CFRelease(cf: *const c_void);
    fn CFDictionaryGetCount(d: *const c_void) -> isize;
    fn CFDictionaryGetValue(d: *const c_void, key: *const c_void) -> *const c_void;
    fn CFDictionaryCreateMutableCopy(
        alloc: *const c_void,
        capacity: isize,
        d: *const c_void,
    ) -> *const c_void;
    fn CFArrayGetCount(a: *const c_void) -> isize;
    fn CFArrayGetValueAtIndex(a: *const c_void, idx: isize) -> *const c_void;
}

#[link(name = "IOReport")]
unsafe extern "C" {
    fn IOReportCopyChannelsInGroup(
        group: *const c_void,
        subgroup: *const c_void,
        a: u64,
        b: u64,
        c: u64,
    ) -> *const c_void;
    fn IOReportCreateSubscription(
        a: *const c_void,
        desired: *const c_void,
        subbed: *mut *const c_void,
        channel_id: u64,
        b: *const c_void,
    ) -> *const c_void;
    fn IOReportCreateSamples(
        sub: *const c_void,
        chan: *const c_void,
        a: *const c_void,
    ) -> *const c_void;
    fn IOReportCreateSamplesDelta(
        prev: *const c_void,
        next: *const c_void,
        a: *const c_void,
    ) -> *const c_void;
    fn IOReportChannelGetChannelName(item: *const c_void) -> *const c_void;
    fn IOReportChannelGetGroup(item: *const c_void) -> *const c_void;
    fn IOReportChannelGetUnitLabel(item: *const c_void) -> *const c_void;
    fn IOReportSimpleGetIntegerValue(item: *const c_void, column: i32) -> i64;
}

#[link(name = "IOKit", kind = "framework")]
unsafe extern "C" {
    static kIOMasterPortDefault: *mut c_void;
    fn IOServiceMatching(name: *const c_char) -> *mut c_void;
    fn IOServiceGetMatchingService(master: *mut c_void, matching: *mut c_void) -> u32;
    fn IOServiceOpen(device: u32, owning_task: u32, type_: u32, connect: *mut u32) -> i32;
    fn IOServiceClose(connect: u32) -> i32;
    fn IOConnectCallStructMethod(
        connect: u32,
        selector: u32,
        input: *const c_void,
        input_cnt: usize,
        output: *mut c_void,
        output_cnt: *mut usize,
    ) -> i32;
    fn mach_task_self() -> u32;
}

unsafe extern "C" {
    fn geteuid() -> u32;
}

fn is_root() -> bool {
    unsafe { geteuid() == 0 }
}

fn cfstr(s: &str) -> *const c_void {
    let Ok(c) = CString::new(s) else {
        return ptr::null();
    };
    unsafe { CFStringCreateWithCString(ptr::null(), c.as_ptr(), CF_STRING_UTF8) }
}

fn from_cf(s: *const c_void) -> Option<String> {
    if s.is_null() {
        return None;
    }
    let mut buf = [0i8; 256];
    let ok = unsafe { CFStringGetCString(s, buf.as_mut_ptr(), buf.len() as isize, CF_STRING_UTF8) };
    if ok == 0 {
        return None;
    }
    let c = unsafe { CStr::from_ptr(buf.as_ptr()) };
    Some(c.to_string_lossy().into_owned())
}

fn dict_get(dict: *const c_void, key: &str) -> *const c_void {
    if dict.is_null() {
        return ptr::null();
    }
    let k = cfstr(key);
    if k.is_null() {
        return ptr::null();
    }
    let v = unsafe { CFDictionaryGetValue(dict, k) };
    unsafe { CFRelease(k) };
    v
}

/// Joules from an IOReport simple integer. Unknown units return `None` (no guess).
fn joules_from_unit(raw: i64, unit: &str) -> Option<f64> {
    if raw < 0 {
        return None;
    }
    let v = raw as f64;
    let j = match unit.trim() {
        "mJ" => v / 1e3,
        "uJ" | "µJ" | "μJ" => v / 1e6,
        "nJ" => v / 1e9,
        "J" => v,
        _ => return None,
    };
    if j.is_finite() && j >= 0.0 {
        Some(j)
    } else {
        None
    }
}

struct EnergySub {
    chan: *const c_void,
    sub: *const c_void,
}

impl Drop for EnergySub {
    fn drop(&mut self) {
        unsafe {
            if !self.sub.is_null() {
                CFRelease(self.sub);
            }
            if !self.chan.is_null() {
                CFRelease(self.chan);
            }
        }
    }
}

fn open_energy_model() -> Result<EnergySub, String> {
    let group = cfstr("Energy Model");
    if group.is_null() {
        return Err("CFString Energy Model failed".into());
    }
    let copied = unsafe { IOReportCopyChannelsInGroup(group, ptr::null(), 0, 0, 0) };
    unsafe { CFRelease(group) };
    if copied.is_null() {
        return Err("IOReportCopyChannelsInGroup(Energy Model) returned null".into());
    }
    let count = unsafe { CFDictionaryGetCount(copied) };
    let chan = unsafe { CFDictionaryCreateMutableCopy(ptr::null(), count, copied) };
    unsafe { CFRelease(copied) };
    if chan.is_null() {
        return Err("IOReport channel copy failed".into());
    }
    if dict_get(chan, "IOReportChannels").is_null() {
        unsafe { CFRelease(chan) };
        return Err("Energy Model dict has no IOReportChannels".into());
    }
    let mut subbed: *const c_void = ptr::null();
    let sub = unsafe { IOReportCreateSubscription(ptr::null(), chan, &mut subbed, 0, ptr::null()) };
    if !subbed.is_null() {
        unsafe { CFRelease(subbed) };
    }
    if sub.is_null() {
        unsafe { CFRelease(chan) };
        return Err("IOReportCreateSubscription failed".into());
    }
    Ok(EnergySub { chan, sub })
}

fn rows_from_delta(delta: *const c_void) -> Result<Vec<NamedJoule>, String> {
    let channels = dict_get(delta, "IOReportChannels");
    if channels.is_null() {
        return Err("delta sample missing IOReportChannels".into());
    }
    let n = unsafe { CFArrayGetCount(channels) };
    if n < 0 {
        return Err("IOReportChannels count invalid".into());
    }
    let mut rows = Vec::new();
    for i in 0..n {
        let item = unsafe { CFArrayGetValueAtIndex(channels, i) };
        if item.is_null() {
            continue;
        }
        let group = from_cf(unsafe { IOReportChannelGetGroup(item) }).unwrap_or_default();
        if group != "Energy Model" {
            continue;
        }
        let Some(name) = from_cf(unsafe { IOReportChannelGetChannelName(item) }) else {
            continue;
        };
        let Some(unit) = from_cf(unsafe { IOReportChannelGetUnitLabel(item) }) else {
            continue;
        };
        let raw = unsafe { IOReportSimpleGetIntegerValue(item, 0) };
        let Some(joules) = joules_from_unit(raw, unit.trim()) else {
            continue;
        };
        rows.push(NamedJoule { name, joules });
    }
    Ok(rows)
}

fn sample_ioreport(window: Duration, window_ms: u64) -> Result<MeterSample, String> {
    let sub = open_energy_model()?;
    let s1 = unsafe { IOReportCreateSamples(sub.sub, sub.chan, ptr::null()) };
    if s1.is_null() {
        return Err("IOReportCreateSamples (t0) failed".into());
    }
    thread::sleep(window);
    let s2 = unsafe { IOReportCreateSamples(sub.sub, sub.chan, ptr::null()) };
    if s2.is_null() {
        unsafe { CFRelease(s1) };
        return Err("IOReportCreateSamples (t1) failed".into());
    }
    let delta = unsafe { IOReportCreateSamplesDelta(s1, s2, ptr::null()) };
    unsafe {
        CFRelease(s1);
        CFRelease(s2);
    }
    if delta.is_null() {
        return Err("IOReportCreateSamplesDelta failed".into());
    }
    let rows = match rows_from_delta(delta) {
        Ok(r) => {
            unsafe { CFRelease(delta) };
            r
        }
        Err(e) => {
            unsafe { CFRelease(delta) };
            return Err(e);
        }
    };
    if rows.is_empty() {
        return Err(
            "IOReport Energy Model returned no channels with unit mJ/uJ/nJ/J — measured_j=None"
                .into(),
        );
    }
    let sample = fold_named_joules(&rows, MeasureSource::IoReport, window_ms, "ioreport");
    if sample.components.is_empty() && sample.measured_j.is_none() {
        return Err(format!(
            "IOReport channels not mapped to CPU/GPU/ANE/DRAM/package ({})",
            rows.iter()
                .map(|r| r.name.as_str())
                .take(8)
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    Ok(sample)
}

// ── SMC PSTR (system total power) ───────────────────────────────────────────

#[repr(C)]
#[derive(Clone, Copy)]
struct SmcKeyDataVer {
    major: u8,
    minor: u8,
    build: u8,
    reserved: u8,
    release: u16,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct SmcPLimitData {
    version: u16,
    length: u16,
    cpu_p_limit: u32,
    gpu_p_limit: u32,
    mem_p_limit: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct SmcKeyInfo {
    data_size: u32,
    data_type: u32,
    data_attributes: u8,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct SmcKeyData {
    key: u32,
    vers: SmcKeyDataVer,
    p_limit_data: SmcPLimitData,
    key_info: SmcKeyInfo,
    result: u8,
    status: u8,
    data8: u8,
    data32: u32,
    bytes: [u8; 32],
}

impl Default for SmcKeyData {
    fn default() -> Self {
        Self {
            key: 0,
            vers: SmcKeyDataVer {
                major: 0,
                minor: 0,
                build: 0,
                reserved: 0,
                release: 0,
            },
            p_limit_data: SmcPLimitData {
                version: 0,
                length: 0,
                cpu_p_limit: 0,
                gpu_p_limit: 0,
                mem_p_limit: 0,
            },
            key_info: SmcKeyInfo {
                data_size: 0,
                data_type: 0,
                data_attributes: 0,
            },
            result: 0,
            status: 0,
            data8: 0,
            data32: 0,
            bytes: [0; 32],
        }
    }
}

const SMC_FLOAT_TYPE: u32 = 0x666c7420; // 'flt '
const SMC_CMD_READ_KEYINFO: u8 = 9;
const SMC_CMD_READ_BYTES: u8 = 5;
const KERNEL_INDEX_SMC: u32 = 2;

fn smc_key_id(key: &str) -> Option<u32> {
    let b = key.as_bytes();
    if b.len() != 4 {
        return None;
    }
    Some(u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
}

struct SmcConn {
    conn: u32,
}

impl Drop for SmcConn {
    fn drop(&mut self) {
        if self.conn != 0 {
            unsafe {
                IOServiceClose(self.conn);
            }
        }
    }
}

fn smc_open() -> Result<SmcConn, String> {
    let name = CString::new("AppleSMC").map_err(|_| "AppleSMC CString")?;
    let matching = unsafe { IOServiceMatching(name.as_ptr()) };
    if matching.is_null() {
        return Err("IOServiceMatching(AppleSMC) null".into());
    }
    let service = unsafe { IOServiceGetMatchingService(kIOMasterPortDefault, matching) };
    if service == 0 {
        return Err("IOServiceGetMatchingService(AppleSMC) returned 0".into());
    }
    let mut conn = 0u32;
    let kr = unsafe { IOServiceOpen(service, mach_task_self(), 0, &mut conn) };
    if kr != 0 || conn == 0 {
        return Err(format!("IOServiceOpen(AppleSMC) failed kr={kr}"));
    }
    Ok(SmcConn { conn })
}

fn smc_call(conn: u32, input: &SmcKeyData) -> Result<SmcKeyData, String> {
    let mut output = SmcKeyData::default();
    let mut out_size = std::mem::size_of::<SmcKeyData>();
    let kr = unsafe {
        IOConnectCallStructMethod(
            conn,
            KERNEL_INDEX_SMC,
            input as *const _ as *const c_void,
            std::mem::size_of::<SmcKeyData>(),
            &mut output as *mut _ as *mut c_void,
            &mut out_size,
        )
    };
    if kr != 0 {
        return Err(format!("IOConnectCallStructMethod kr={kr}"));
    }
    if output.result == 132 {
        return Err("SMC key not found".into());
    }
    if output.result != 0 {
        return Err(format!("SMC error result={}", output.result));
    }
    Ok(output)
}

fn smc_read_float(conn: u32, key: &str) -> Result<f32, String> {
    let key_id = smc_key_id(key).ok_or_else(|| "SMC key must be 4 bytes".to_string())?;
    let mut info_req = SmcKeyData::default();
    info_req.key = key_id;
    info_req.data8 = SMC_CMD_READ_KEYINFO;
    let info = smc_call(conn, &info_req)?;
    if info.key_info.data_size != 4 || info.key_info.data_type != SMC_FLOAT_TYPE {
        return Err(format!(
            "SMC key {key} is not flt (size={}, type=0x{:08x})",
            info.key_info.data_size, info.key_info.data_type
        ));
    }
    let mut read_req = SmcKeyData::default();
    read_req.key = key_id;
    read_req.key_info = info.key_info;
    read_req.data8 = SMC_CMD_READ_BYTES;
    let val = smc_call(conn, &read_req)?;
    let bytes: [u8; 4] = val.bytes[0..4]
        .try_into()
        .map_err(|_| "SMC float bytes truncated".to_string())?;
    Ok(f32::from_le_bytes(bytes))
}

fn probe_smc_pstr() -> Result<f32, String> {
    let smc = smc_open()?;
    let w = smc_read_float(smc.conn, "PSTR")?;
    if !w.is_finite() || w < 0.0 {
        return Err(format!("SMC PSTR non-finite/negative: {w}"));
    }
    Ok(w)
}

/// Average SMC `PSTR` watts over `window`, convert to package joules.
fn sample_smc_pstr(window: Duration, window_ms: u64) -> Result<MeterSample, String> {
    let smc = smc_open()?;
    let mut watts = Vec::new();
    let start = Instant::now();
    // Prime
    watts.push(smc_read_float(smc.conn, "PSTR")?);
    let step = Duration::from_millis(100).min(window / 4).max(Duration::from_millis(20));
    while start.elapsed() < window {
        let remaining = window.saturating_sub(start.elapsed());
        thread::sleep(remaining.min(step));
        match smc_read_float(smc.conn, "PSTR") {
            Ok(w) if w.is_finite() && w >= 0.0 => watts.push(w),
            Ok(w) => return Err(format!("SMC PSTR invalid mid-window: {w}")),
            Err(e) => return Err(e),
        }
    }
    if watts.is_empty() {
        return Err("SMC PSTR produced no samples".into());
    }
    let avg = watts.iter().map(|w| *w as f64).sum::<f64>() / watts.len() as f64;
    let sample = sample_from_smc_pstr_watts(avg, window_ms);
    if sample.measured_j.is_none() {
        return Err(sample.detail);
    }
    Ok(sample)
}

fn powermetrics_bin() -> Option<&'static str> {
    let p = "/usr/bin/powermetrics";
    if std::path::Path::new(p).is_file() {
        Some(p)
    } else {
        None
    }
}

/// Capability only — does not invent joules.
pub fn probe() -> MeterCapability {
    let smc = probe_smc_pstr();
    let ior = open_energy_model();
    match (smc, ior) {
        (Ok(w), Ok(_)) => MeterCapability {
            available: true,
            source: MeasureSource::Smc,
            detail: format!(
                "SMC PSTR readable ({w:.3} W) for package measured_j; IOReport Energy Model rails present (CPU/GPU/ANE/DRAM). No Energy Model Package/Combined channel — package is PSTR×window, never a rail sum. powermetrics still needs sudo for combined_power."
            ),
            platform: "macos_smc_ioreport",
        },
        (Ok(w), Err(e)) => MeterCapability {
            available: true,
            source: MeasureSource::Smc,
            detail: format!(
                "SMC PSTR readable ({w:.3} W) for package measured_j; IOReport unavailable ({e})"
            ),
            platform: "macos_smc",
        },
        (Err(smc_err), Ok(_)) => MeterCapability {
            available: true,
            source: MeasureSource::IoReport,
            detail: format!(
                "IOReport Energy Model rails present; SMC PSTR unavailable ({smc_err}) — package measured_j=None unless root powermetrics combined_power. Never sum rails."
            ),
            platform: "macos_ioreport",
        },
        (Err(smc_err), Err(ioreport_err)) => match powermetrics_bin() {
            Some(bin) if is_root() => MeterCapability {
                available: true,
                source: MeasureSource::Powermetrics,
                detail: format!(
                    "SMC unavailable ({smc_err}); IOReport unavailable ({ioreport_err}). {bin} present and euid=0."
                ),
                platform: "macos_powermetrics",
            },
            Some(bin) => MeterCapability::unavailable(
                "macos",
                format!(
                    "SMC PSTR unavailable ({smc_err}); IOReport unavailable ({ioreport_err}). {bin} exists but needs root — measured_j=None."
                ),
            ),
            None => MeterCapability::unavailable(
                "macos",
                format!(
                    "SMC PSTR unavailable ({smc_err}); IOReport unavailable ({ioreport_err}); powermetrics missing — measured_j=None"
                ),
            ),
        },
    }
}

/// Interval sample. Prefer SMC PSTR package + IOReport rails; powermetrics if needed.
///
/// IOReport delta and SMC `PSTR` averaging share one wall-clock window (no 2× sleep).
pub fn measure(window: Duration, window_ms: u64) -> MeterSample {
    finish_measure(sample_ioreport_and_smc(window, window_ms), window_ms)
}

/// Same as [`measure`], but `work` runs **inside** the SMC/IOReport sample window
/// (overlapping). Used to stamp Metal kernel receipts with real package `measured_j`.
pub fn measure_during<F, T>(min_window: Duration, window_ms: u64, work: F) -> (T, MeterSample)
where
    F: FnOnce() -> T,
{
    let w = min_window;
    let ms = window_ms;
    let handle = thread::spawn(move || sample_ioreport_and_smc(w, ms));
    let out = work();
    let pair = handle
        .join()
        .unwrap_or_else(|_| (Err("meter thread panicked".into()), Err("meter thread panicked".into())));
    (out, finish_measure(pair, window_ms))
}

fn finish_measure(
    (rails, smc): (Result<MeterSample, String>, Result<MeterSample, String>),
    window_ms: u64,
) -> MeterSample {
    match (rails, smc) {
        (Ok(rails), Ok(pkg)) => {
            let merged = merge_rails_with_package(&rails, &pkg);
            if merged.records_measurement() {
                return merged;
            }
            pkg
        }
        (Err(_), Ok(pkg)) => pkg,
        (Ok(rails), Err(smc_err)) => {
            if rails.measured_j.is_some() {
                return rails;
            }
            let pm = measure_powermetrics(
                window_ms,
                &format!("ioreport rails ok; smc PSTR: {smc_err}"),
            );
            if pm.measured_j.is_some() {
                return merge_rails_with_package(&rails, &pm);
            }
            rails
        }
        (Err(e), Err(smc_err)) => measure_powermetrics(
            window_ms,
            &format!("ioreport: {e}; smc PSTR: {smc_err}"),
        ),
    }
}

/// One window: IOReport Energy Model delta + SMC PSTR average in parallel (threaded).
fn sample_ioreport_and_smc(
    window: Duration,
    window_ms: u64,
) -> (Result<MeterSample, String>, Result<MeterSample, String>) {
    let w = window;
    let ms = window_ms;
    let handle = thread::spawn(move || sample_smc_pstr(w, ms));
    let rails = sample_ioreport(window, window_ms);
    let smc = handle.join().unwrap_or_else(|_| Err("SMC PSTR thread panicked".into()));
    (rails, smc)
}

fn measure_powermetrics(window_ms: u64, prior_note: &str) -> MeterSample {
    let Some(bin) = powermetrics_bin() else {
        return MeterSample::unavailable(
            window_ms,
            format!("{prior_note}; powermetrics not installed — measured_j=None"),
        );
    };
    let interval = window_ms.max(50).to_string();
    let output = std::process::Command::new(bin)
        .args([
            "-f",
            "plist",
            "-n",
            "1",
            "-i",
            &interval,
            "--samplers",
            "cpu_power,gpu_power",
            "-a",
            "0",
        ])
        .output();
    let Ok(out) = output else {
        return MeterSample::unavailable(
            window_ms,
            format!("{prior_note}; powermetrics spawn failed — measured_j=None"),
        );
    };
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    if !out.status.success() {
        let err = stderr.trim();
        let hint = if err.to_ascii_lowercase().contains("superuser")
            || err.to_ascii_lowercase().contains("root")
            || err.to_ascii_lowercase().contains("not permitted")
        {
            " (needs sudo)"
        } else {
            ""
        };
        return MeterSample::unavailable(
            window_ms,
            format!(
                "{prior_note}; powermetrics exited {:?}{hint}: {} — measured_j=None",
                out.status.code(),
                err.chars().take(240).collect::<String>()
            ),
        );
    }
    let parsed = super::parse_powermetrics_output(&stdout, window_ms);
    if parsed.records_measurement() {
        return parsed;
    }
    if !stdout.contains("<plist") {
        let text = super::parse_powermetrics_output(&stdout, window_ms);
        if text.records_measurement() {
            return text;
        }
    }
    MeterSample::unavailable(
        window_ms,
        format!(
            "{prior_note}; powermetrics produced no CPU/GPU/ANE/DRAM/package fields — measured_j=None"
        ),
    )
}
