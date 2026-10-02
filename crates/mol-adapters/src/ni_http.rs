//! Live WCA/NI certify over HTTP or MCP-shaped JSON-RPC.
//!
//! Env-gated, default off. When unset or transport fails (and fallback is on),
//! callers use [`crate::InCrateNiCertify`]. Never invents `measured_j`; forces
//! `board_synth_claimed=false` and `stage_c_measured=false` even if a remote
//! claims otherwise (Ferric / FPGA Stage C meters stay OUT OF PROOF SCOPE).

use mol_core::{Floor, FloorKind, MolError, Result, BOARD_SYNTH_CLAIMED};
use serde::{Deserialize, Serialize};

use crate::efa::{EfaCertResult, EfaDecision};
use crate::ni_live::{InCrateNiCertify, LiveCertOutcome, NiCertificate, CertifySource};
use crate::wca::WcaCertResult;

/// Env: NI certify HTTP or MCP base URL (`MOL_NI_CERTIFY_URL`, else `MOL_WCA_CERTIFY_URL`).
pub const ENV_NI_CERTIFY_URL: &str = "MOL_NI_CERTIFY_URL";
/// Alias for WCA-oriented deployments.
pub const ENV_WCA_CERTIFY_URL: &str = "MOL_WCA_CERTIFY_URL";
/// `http` (default) or `mcp`.
pub const ENV_CERTIFY_MODE: &str = "MOL_CERTIFY_MODE";
/// Optional bearer token.
pub const ENV_CERTIFY_TOKEN: &str = "MOL_CERTIFY_TOKEN";
/// `1`/`true` (default) → fall back to in-crate on miss/error; `0`/`false` → surface error.
pub const ENV_CERTIFY_FALLBACK: &str = "MOL_CERTIFY_FALLBACK";
/// Timeout milliseconds (default 5000).
pub const ENV_CERTIFY_TIMEOUT_MS: &str = "MOL_CERTIFY_TIMEOUT_MS";

/// Wire request body for live certify (HTTP POST or MCP tool args).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveCertifyRequest {
    /// Proposal summary (answer text or act).
    pub summary: String,
    /// Catalog / cascade estimated joules (never treated as measured).
    pub estimated_j: f64,
    /// Optional EFA tag (`diverge` → refuse).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub efa_tag: Option<String>,
    /// Optional energy residual for EFA.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub energy_residual: Option<f64>,
}

/// Wire response from a live certify endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveCertifyResponse {
    /// `commit` | `refuse` (also accepts `allow` as commit).
    pub decision: String,
    /// NI certificate id (`ni:…`). Optional — minted locally if absent.
    #[serde(default)]
    pub certificate_id: Option<String>,
    /// EFA id.
    #[serde(default)]
    pub efa_id: Option<String>,
    /// WCA id.
    #[serde(default)]
    pub wca_id: Option<String>,
    /// Reasons.
    #[serde(default)]
    pub reasons: Vec<String>,
    /// Optional floor on refuse (id/kind/reason).
    #[serde(default)]
    pub floor: Option<LiveFloorWire>,
    /// Estimated certify joules from remote (catalog only).
    #[serde(default)]
    pub estimated_j: Option<f64>,
    /// Remote may send measured_j — **ignored** (honesty fence).
    #[serde(default)]
    pub measured_j: Option<f64>,
    /// Remote board claim — **forced false**.
    #[serde(default)]
    pub board_synth_claimed: Option<bool>,
    /// Remote Stage C claim — **forced false**.
    #[serde(default)]
    pub stage_c_measured: Option<bool>,
}

/// Minimal floor wire (avoid requiring full FloorKind parse failures).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveFloorWire {
    /// Floor id.
    pub id: String,
    /// Optional kind label (`efa_certificate`, `wca_refuse`, …).
    #[serde(default)]
    pub kind: Option<String>,
    /// Reason.
    pub reason: String,
}

/// HTTP / MCP live certify client.
#[derive(Debug, Clone)]
pub struct HttpNiCertify {
    /// Base URL (POST target for http; MCP JSON-RPC endpoint for mcp).
    pub endpoint: String,
    /// `http` or `mcp`.
    pub mode: CertifyTransport,
    /// Optional bearer.
    pub token: Option<String>,
    /// Timeout ms.
    pub timeout_ms: u64,
}

/// Transport kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CertifyTransport {
    /// Plain HTTP POST JSON to `{endpoint}` or `{endpoint}/v1/certify`.
    Http,
    /// MCP-shaped JSON-RPC `tools/call` with name `ni_certify` (fallback `wca_certify`).
    Mcp,
}

impl CertifyTransport {
    fn from_env_str(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "mcp" | "jsonrpc" | "json-rpc" => Self::Mcp,
            _ => Self::Http,
        }
    }

    fn source(self) -> CertifySource {
        match self {
            Self::Http => CertifySource::Http,
            Self::Mcp => CertifySource::Mcp,
        }
    }
}

impl HttpNiCertify {
    /// Construct.
    pub fn new(endpoint: impl Into<String>, mode: CertifyTransport) -> Self {
        Self {
            endpoint: endpoint.into(),
            mode,
            token: std::env::var(ENV_CERTIFY_TOKEN).ok().filter(|s| !s.is_empty()),
            timeout_ms: std::env::var(ENV_CERTIFY_TIMEOUT_MS)
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(5_000),
        }
    }

    /// From process env. `None` when no URL configured.
    pub fn from_env() -> Option<Self> {
        let url = std::env::var(ENV_NI_CERTIFY_URL)
            .ok()
            .filter(|s| !s.trim().is_empty())
            .or_else(|| {
                std::env::var(ENV_WCA_CERTIFY_URL)
                    .ok()
                    .filter(|s| !s.trim().is_empty())
            })?;
        let mode = std::env::var(ENV_CERTIFY_MODE)
            .ok()
            .map(|s| CertifyTransport::from_env_str(&s))
            .unwrap_or_else(|| {
                // Heuristic: URL path containing `/mcp` → MCP mode.
                if url.to_ascii_lowercase().contains("/mcp") {
                    CertifyTransport::Mcp
                } else {
                    CertifyTransport::Http
                }
            });
        Some(Self::new(url.trim(), mode))
    }

    fn post_url(&self) -> String {
        let e = self.endpoint.trim_end_matches('/');
        match self.mode {
            CertifyTransport::Mcp => e.to_string(),
            CertifyTransport::Http => {
                if e.ends_with("/v1/certify") || e.ends_with("/certify") {
                    e.to_string()
                } else if e.ends_with("/v1") {
                    format!("{e}/certify")
                } else {
                    format!("{e}/v1/certify")
                }
            }
        }
    }

    /// Call the live endpoint once (no fallback).
    pub fn certify_remote(&self, req: &LiveCertifyRequest) -> Result<LiveCertOutcome> {
        match self.mode {
            CertifyTransport::Http => self.certify_http(req),
            CertifyTransport::Mcp => self.certify_mcp(req),
        }
    }

    fn agent(&self) -> ureq::Agent {
        ureq::AgentBuilder::new()
            .timeout(std::time::Duration::from_millis(self.timeout_ms))
            .build()
    }

    fn with_auth(&self, mut call: ureq::Request) -> ureq::Request {
        call = call.set("Content-Type", "application/json");
        if let Some(ref t) = self.token {
            call = call.set("Authorization", &format!("Bearer {t}"));
        }
        call
    }

    fn certify_http(&self, req: &LiveCertifyRequest) -> Result<LiveCertOutcome> {
        let url = self.post_url();
        let call = self.with_auth(self.agent().post(&url));
        let resp = match call.send_json(serde_json::to_value(req).map_err(|e| {
            MolError::Msg(format!("live NI certify serialize: {e}"))
        })?) {
            Ok(r) => r,
            Err(ureq::Error::Status(code, resp)) => {
                let detail = resp.into_string().unwrap_or_default();
                return Err(MolError::Msg(format!(
                    "live NI/WCA HTTP certify {code}: {detail}; measured_j stays None"
                )));
            }
            Err(e) => {
                return Err(MolError::Msg(format!(
                    "live NI/WCA HTTP certify transport failed ({e}); measured_j stays None"
                )));
            }
        };
        let wire: LiveCertifyResponse = resp.into_json().map_err(|e| {
            MolError::Msg(format!("live NI/WCA HTTP certify JSON: {e}"))
        })?;
        Ok(outcome_from_wire(wire, req, self.mode.source()))
    }

    fn certify_mcp(&self, req: &LiveCertifyRequest) -> Result<LiveCertOutcome> {
        let url = self.post_url();
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "ni_certify",
                "arguments": req
            }
        });
        let call = self.with_auth(self.agent().post(&url));
        let resp = match call.send_json(body) {
            Ok(r) => r,
            Err(ureq::Error::Status(code, resp)) => {
                // Retry once with wca_certify tool name (WCA-only MCP servers).
                let detail = resp.into_string().unwrap_or_default();
                if code == 404 || detail.to_ascii_lowercase().contains("unknown") {
                    return self.certify_mcp_named(req, "wca_certify");
                }
                return Err(MolError::Msg(format!(
                    "live NI/WCA MCP certify {code}: {detail}; measured_j stays None"
                )));
            }
            Err(e) => {
                return Err(MolError::Msg(format!(
                    "live NI/WCA MCP certify transport failed ({e}); measured_j stays None"
                )));
            }
        };
        self.parse_mcp_response(resp, req)
            .or_else(|_| self.certify_mcp_named(req, "wca_certify"))
    }

    fn certify_mcp_named(&self, req: &LiveCertifyRequest, tool: &str) -> Result<LiveCertOutcome> {
        let url = self.post_url();
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": tool,
                "arguments": req
            }
        });
        let call = self.with_auth(self.agent().post(&url));
        let resp = match call.send_json(body) {
            Ok(r) => r,
            Err(ureq::Error::Status(code, resp)) => {
                let detail = resp.into_string().unwrap_or_default();
                return Err(MolError::Msg(format!(
                    "live NI/WCA MCP certify ({tool}) {code}: {detail}; measured_j stays None"
                )));
            }
            Err(e) => {
                return Err(MolError::Msg(format!(
                    "live NI/WCA MCP certify ({tool}) transport failed ({e}); measured_j stays None"
                )));
            }
        };
        self.parse_mcp_response(resp, req)
    }

    fn parse_mcp_response(
        &self,
        resp: ureq::Response,
        req: &LiveCertifyRequest,
    ) -> Result<LiveCertOutcome> {
        let v: serde_json::Value = resp.into_json().map_err(|e| {
            MolError::Msg(format!("live NI/WCA MCP JSON: {e}"))
        })?;
        if let Some(err) = v.get("error") {
            return Err(MolError::Msg(format!(
                "live NI/WCA MCP error: {err}; measured_j stays None"
            )));
        }
        // MCP tools/call often wraps content as { result: { content: [ { text: "{...}" } ] } }
        // or returns structured result directly.
        let wire = if let Some(text) = v
            .pointer("/result/content/0/text")
            .and_then(|t| t.as_str())
        {
            match serde_json::from_str::<LiveCertifyResponse>(text) {
                Ok(w) => w,
                Err(e1) => serde_json::from_str::<serde_json::Value>(text)
                    .and_then(serde_json::from_value::<LiveCertifyResponse>)
                    .map_err(|e2| {
                        MolError::Msg(format!("MCP content text parse: {e1}; nested: {e2}"))
                    })?,
            }
        } else if let Some(result) = v.get("result") {
            // Prefer nested `structuredContent` / direct object.
            if let Some(sc) = result.get("structuredContent") {
                serde_json::from_value(sc.clone()).map_err(|e| {
                    MolError::Msg(format!("MCP structuredContent parse: {e}"))
                })?
            } else {
                serde_json::from_value(result.clone()).map_err(|e| {
                    MolError::Msg(format!("MCP result parse: {e}"))
                })?
            }
        } else {
            return Err(MolError::Msg(
                "live NI/WCA MCP response missing result; measured_j stays None".into(),
            ));
        };
        Ok(outcome_from_wire(wire, req, self.mode.source()))
    }
}

fn mint_id(prefix: &str) -> String {
    let u = uuid::Uuid::new_v4().simple().to_string();
    format!("{prefix}:{}", &u[..12])
}

fn parse_kind(label: Option<&str>, decision_refuse: bool) -> FloorKind {
    match label.map(|s| s.trim().to_ascii_lowercase()).as_deref() {
        Some("wca_refuse") | Some("wca") => FloorKind::WcaRefuse,
        Some("efa_certificate") | Some("efa") => FloorKind::EfaCertificate,
        Some("ni_certificate") | Some("ni") => FloorKind::EfaCertificate,
        Some("safety") => FloorKind::Safety,
        _ if decision_refuse => FloorKind::EfaCertificate,
        _ => FloorKind::EfaCertificate,
    }
}

fn outcome_from_wire(
    wire: LiveCertifyResponse,
    req: &LiveCertifyRequest,
    source: CertifySource,
) -> LiveCertOutcome {
    let decision_raw = wire.decision.trim().to_ascii_lowercase();
    let allows = matches!(decision_raw.as_str(), "commit" | "allow" | "ok" | "pass");
    let decision = if allows { "commit" } else { "refuse" };

    // Honesty fence: never trust remote silicon / board claims; never invent measured_j.
    let _ignored_measured = wire.measured_j;
    let _ignored_board = wire.board_synth_claimed;
    let _ignored_stage = wire.stage_c_measured;

    let ni = NiCertificate {
        certificate_id: wire
            .certificate_id
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| mint_id("ni")),
        efa_id: wire
            .efa_id
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| mint_id("efa")),
        wca_id: wire
            .wca_id
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| mint_id("wca")),
        decision: decision.into(),
        reasons: if wire.reasons.is_empty() {
            vec![format!(
                "live {source:?} certify {decision} for: {}",
                req.summary
            )]
        } else {
            wire.reasons
        },
        board_synth_claimed: BOARD_SYNTH_CLAIMED,
        stage_c_measured: false,
        estimated_j: wire.estimated_j.unwrap_or(req.estimated_j),
        source,
    };

    let floor = if allows {
        None
    } else {
        Some(match wire.floor {
            Some(f) => Floor::new(
                f.id,
                parse_kind(f.kind.as_deref(), true),
                f.reason,
            ),
            None => Floor::new(
                "ni_certificate",
                FloorKind::EfaCertificate,
                format!("live certify refuse for: {}", req.summary),
            ),
        })
    };

    // Reconstruct EFA/WCA slices for LiveCertOutcome completeness (software-shaped).
    let efa = EfaCertResult {
        decision: if allows {
            EfaDecision::Allow
        } else {
            EfaDecision::Refuse
        },
        reasons: ni.reasons.clone(),
        estimated_j: ni.estimated_j,
        board_synth_claimed: BOARD_SYNTH_CLAIMED,
        floor: floor.clone(),
        note: format!(
            "live {:?} NI/WCA certify; Ferric/FPGA Stage C still stub; measured_j ignored/None",
            source
        ),
    };
    let wca = WcaCertResult {
        decision: if allows {
            "allow".into()
        } else {
            "refuse".into()
        },
        reasons: ni.reasons.clone(),
        estimated_j: ni.estimated_j,
        board_synth_claimed: BOARD_SYNTH_CLAIMED,
        floor: floor.clone(),
        note: format!(
            "live {:?} path; not FPGA Stage C meter; measured_j=None",
            source
        ),
    };

    LiveCertOutcome {
        ni,
        efa,
        wca,
        floor,
    }
}

/// Whether env requests fallback to in-crate (default true).
pub fn fallback_enabled() -> bool {
    match std::env::var(ENV_CERTIFY_FALLBACK) {
        Ok(s) => {
            let t = s.trim().to_ascii_lowercase();
            !(t == "0" || t == "false" || t == "no" || t == "off")
        }
        Err(_) => true,
    }
}

/// Prefer live HTTP/MCP certify when configured; else in-crate.
///
/// On live transport/parse failure: fall back to [`InCrateNiCertify`] when
/// [`fallback_enabled`], else return the error.
pub fn certify_live_prefer_env(
    summary: &str,
    estimated_j: f64,
    efa_tag: Option<String>,
    energy_residual: Option<f64>,
) -> Result<LiveCertOutcome> {
    let req = LiveCertifyRequest {
        summary: summary.into(),
        estimated_j,
        efa_tag: efa_tag.clone(),
        energy_residual,
    };
    if let Some(client) = HttpNiCertify::from_env() {
        match client.certify_remote(&req) {
            Ok(out) => return Ok(out),
            Err(e) if fallback_enabled() => {
                let mut out = InCrateNiCertify::new().certify_live(
                    summary,
                    estimated_j,
                    efa_tag,
                    energy_residual,
                )?;
                out.ni.source = CertifySource::InCrateFallback;
                out.ni.reasons.push(format!(
                    "live certify failed → in-crate fallback ({e}); Ferric/FPGA still stub; measured_j=None"
                ));
                return Ok(out);
            }
            Err(e) => return Err(e),
        }
    }
    InCrateNiCertify::new().certify_live(summary, estimated_j, efa_tag, energy_residual)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::{Mutex, OnceLock};
    use std::thread;

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
    }

    fn spawn_http_cert_server(decision: &'static str) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buf = [0u8; 4096];
                let _ = stream.read(&mut buf);
                let body = format!(
                    r#"{{"decision":"{decision}","certificate_id":"ni:livehttp01","efa_id":"efa:livehttp01","wca_id":"wca:livehttp01","reasons":["mock live http"],"estimated_j":1e-9,"measured_j":1.23,"board_synth_claimed":true,"stage_c_measured":true}}"#
                );
                let resp = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = stream.write_all(resp.as_bytes());
            }
        });
        format!("http://{addr}")
    }

    #[test]
    fn http_live_commit_forces_honesty_fence() {
        let _g = env_lock();
        let base = spawn_http_cert_server("commit");
        let client = HttpNiCertify::new(base, CertifyTransport::Http);
        let out = client
            .certify_remote(&LiveCertifyRequest {
                summary: "convert 1 celsius".into(),
                estimated_j: 1e-9,
                efa_tag: None,
                energy_residual: None,
            })
            .unwrap();
        assert!(out.ni.allows_commit());
        assert_eq!(out.ni.source, CertifySource::Http);
        assert_eq!(out.ni.certificate_id, "ni:livehttp01");
        assert!(!out.ni.board_synth_claimed);
        assert!(!out.ni.stage_c_measured);
        // Remote tried to invent measured_j / board — MoL fence dropped them.
    }

    #[test]
    fn prefer_env_fallback_on_bad_url() {
        let _g = env_lock();
        // SAFETY: single-threaded under env_lock; test-only process env.
        unsafe {
            std::env::set_var(ENV_NI_CERTIFY_URL, "http://127.0.0.1:1"); // nothing listening
            std::env::set_var(ENV_CERTIFY_FALLBACK, "1");
            std::env::set_var(ENV_CERTIFY_TIMEOUT_MS, "200");
        }
        let out = certify_live_prefer_env("convert 1 celsius", 1e-9, None, None).unwrap();
        assert!(out.ni.allows_commit());
        assert_eq!(out.ni.source, CertifySource::InCrateFallback);
        assert!(!out.ni.stage_c_measured);
        unsafe {
            std::env::remove_var(ENV_NI_CERTIFY_URL);
            std::env::remove_var(ENV_CERTIFY_FALLBACK);
            std::env::remove_var(ENV_CERTIFY_TIMEOUT_MS);
        }
    }

    #[test]
    fn prefer_env_none_is_in_crate() {
        let _g = env_lock();
        unsafe {
            std::env::remove_var(ENV_NI_CERTIFY_URL);
            std::env::remove_var(ENV_WCA_CERTIFY_URL);
        }
        let out = certify_live_prefer_env("convert 1 celsius", 1e-9, None, None).unwrap();
        assert!(out.ni.allows_commit());
        assert_eq!(out.ni.source, CertifySource::InCrate);
    }
}
