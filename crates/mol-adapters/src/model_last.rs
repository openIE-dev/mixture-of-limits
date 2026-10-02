//! Model LAST residual proposers — offline stub + optional OpenAI-compatible endpoint.
//!
//! **Default:** runnable offline stub (no weights, no network). Optional
//! `--endpoint` talks to a local or OpenAI-compatible `/v1/chat/completions`.
//! Documented HF paths (Laya / Jev-class / Decider) stay **document+stub** until
//! wired; never invent `measured_j`.

use mol_core::{MolError, MolRequest, Result};
use serde::{Deserialize, Serialize};

/// Catalog surrogate joules for Model LAST leaf (Estimated only — not RAPL).
pub const MODEL_LAST_STUB_ESTIMATED_J: f64 = 2.5e-4;
/// Catalog surrogate when a real OpenAI-compatible call succeeds (Estimated).
pub const MODEL_LAST_ENDPOINT_ESTIMATED_J: f64 = 5.0e-3;

/// Named residual profile (shortlist / documentation).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ModelLastProfile {
    /// Generic residual stub (no weights).
    #[default]
    Stub,
    /// Documented HF: `convaiinnovations/laya` (encoder System One) — stub inference.
    LayaHf,
    /// Documented Jev / System One class (hosted or local) — stub inference.
    JevClass,
    /// Documented HF: `Mapika/decider-4b` (and siblings) — stub inference.
    DeciderHf,
}

impl ModelLastProfile {
    /// Human label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Stub => "stub",
            Self::LayaHf => "laya_hf",
            Self::JevClass => "jev_class",
            Self::DeciderHf => "decider_hf",
        }
    }

    /// Hugging Face / docs pointer (documentation only — this crate does not download weights).
    pub fn hf_or_docs(self) -> &'static str {
        match self {
            Self::Stub => "(no weights; offline Model LAST stub)",
            Self::LayaHf => "https://huggingface.co/convaiinnovations/laya",
            Self::JevClass => "https://docs.typesafe.ai/models · https://github.com/theaiautomators/jev-arena",
            Self::DeciderHf => "https://huggingface.co/Mapika/decider-4b",
        }
    }

    /// Parse CLI / env token.
    pub fn parse(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "laya" | "laya_hf" | "convaiinnovations/laya" => Self::LayaHf,
            "jev" | "jev_class" | "system_one" | "systemone" => Self::JevClass,
            "decider" | "decider_hf" | "mapika/decider-4b" => Self::DeciderHf,
            _ => Self::Stub,
        }
    }
}

/// Proposal from Model LAST (always `ModelGenerated` until NI cert).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelLastProposal {
    /// Proposal text (never a Deterministic launder).
    pub text: String,
    /// Profile used.
    pub profile: String,
    /// Catalog estimated joules (Estimated label only).
    pub estimated_j: f64,
    /// Package joules — **always None** on soft-ref / HTTP leaf without meter.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub measured_j: Option<f64>,
    /// True when no network / weights were used.
    pub offline: bool,
    /// Integration note + HF docs pointer.
    pub note: String,
}

impl ModelLastProposal {
    /// Honesty: soft-ref / HTTP path must not invent measured_j.
    pub fn honesty_ok(&self) -> bool {
        self.measured_j.is_none()
    }
}

/// Port toward residual Model LAST proposers.
pub trait ModelLastPort: Send + Sync {
    /// Propose under VoI / allow_model policy (caller still owns NI certify).
    fn propose(&self, req: &MolRequest) -> Result<ModelLastProposal>;
}

/// Runnable offline stub — labeled proposal; documents HF paths; no weights.
#[derive(Debug, Clone)]
pub struct StubModelLast {
    /// Documented profile (still stub inference).
    pub profile: ModelLastProfile,
}

impl Default for StubModelLast {
    fn default() -> Self {
        Self {
            profile: ModelLastProfile::Stub,
        }
    }
}

impl StubModelLast {
    /// Generic offline stub.
    pub fn new() -> Self {
        Self::default()
    }

    /// Documented Laya HF path (stub inference).
    pub fn laya_hf() -> Self {
        Self {
            profile: ModelLastProfile::LayaHf,
        }
    }

    /// Documented Jev / System One class (stub inference).
    pub fn jev_class() -> Self {
        Self {
            profile: ModelLastProfile::JevClass,
        }
    }

    /// Documented Decider HF path (stub inference).
    pub fn decider_hf() -> Self {
        Self {
            profile: ModelLastProfile::DeciderHf,
        }
    }

    /// From profile token.
    pub fn with_profile(profile: ModelLastProfile) -> Self {
        Self { profile }
    }
}

impl ModelLastPort for StubModelLast {
    fn propose(&self, req: &MolRequest) -> Result<ModelLastProposal> {
        if !req.budget.allow_model {
            return Err(MolError::ModelRefused(
                "Model LAST stub refused: allow_model=false".into(),
            ));
        }
        let q = req.query.to_ascii_lowercase();
        let residual = q.contains("residual propose")
            || q.contains("model propose")
            || q.contains("model_fallback")
            || q.contains("uncertified")
            || matches!(req.kind, mol_core::QueryKind::FreeForm);

        if !residual {
            return Err(MolError::ModelRefused(format!(
                "Model LAST stub: no residual pattern for '{}'",
                req.query
            )));
        }

        let text = format!(
            "MODEL_GENERATED_PROPOSAL (Model LAST {}; offline stub; no weights): residual for '{}'; docs={}; commit requires NI/EFA certificate — never launders to Deterministic; measured_j=None",
            self.profile.label(),
            req.query,
            self.profile.hf_or_docs()
        );
        Ok(ModelLastProposal {
            text,
            profile: self.profile.label().into(),
            estimated_j: MODEL_LAST_STUB_ESTIMATED_J,
            measured_j: None,
            offline: true,
            note: format!(
                "offline stub; HF/docs pointer {}; never invent measured_j",
                self.profile.hf_or_docs()
            ),
        })
    }
}

/// Optional OpenAI-compatible HTTP leaf (`/v1/chat/completions`).
///
/// Prefer local servers (llama.cpp, vLLM, Ollama OpenAI shim, Kev, …).
/// On transport failure returns a clear `ModelRefused` — does **not** invent answers
/// or `measured_j`.
#[derive(Debug, Clone)]
pub struct OpenAiCompatibleModelLast {
    /// Base or full chat-completions URL.
    pub endpoint: String,
    /// Model id sent to the API.
    pub model: String,
    /// Optional bearer token (`OPENAI_API_KEY` / local server key).
    pub api_key: Option<String>,
    /// HTTP timeout milliseconds.
    pub timeout_ms: u64,
    /// Documented profile label for receipts.
    pub profile: ModelLastProfile,
}

impl OpenAiCompatibleModelLast {
    /// Construct from endpoint URL (chat completions or `/v1` base).
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            model: "local".into(),
            api_key: std::env::var("OPENAI_API_KEY").ok().filter(|s| !s.is_empty()),
            timeout_ms: 30_000,
            profile: ModelLastProfile::Stub,
        }
    }

    /// Set model id.
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Set profile documentation label.
    pub fn with_profile(mut self, profile: ModelLastProfile) -> Self {
        self.profile = profile;
        self
    }

    /// Set API key.
    pub fn with_api_key(mut self, key: Option<String>) -> Self {
        self.api_key = key;
        self
    }

    fn chat_url(&self) -> String {
        let e = self.endpoint.trim_end_matches('/');
        if e.ends_with("/chat/completions") {
            e.to_string()
        } else if e.ends_with("/v1") {
            format!("{e}/chat/completions")
        } else {
            format!("{e}/v1/chat/completions")
        }
    }
}

impl ModelLastPort for OpenAiCompatibleModelLast {
    fn propose(&self, req: &MolRequest) -> Result<ModelLastProposal> {
        if !req.budget.allow_model {
            return Err(MolError::ModelRefused(
                "Model LAST endpoint refused: allow_model=false".into(),
            ));
        }

        let url = self.chat_url();
        let body = serde_json::json!({
            "model": self.model,
            "messages": [
                {
                    "role": "system",
                    "content": "You are a Mixture of Limits Model LAST residual proposer. Return a short typed proposal only. Do not invent package joules or measured_j."
                },
                {
                    "role": "user",
                    "content": req.query
                }
            ],
            "temperature": 0.0,
            "max_tokens": 256
        });

        let agent = ureq::AgentBuilder::new()
            .timeout(std::time::Duration::from_millis(self.timeout_ms))
            .build();

        let mut call = agent.post(&url).set("Content-Type", "application/json");
        if let Some(ref key) = self.api_key {
            call = call.set("Authorization", &format!("Bearer {key}"));
        }

        let resp = match call.send_json(body) {
            Ok(r) => r,
            Err(ureq::Error::Status(code, resp)) => {
                let detail = resp.into_string().unwrap_or_default();
                return Err(MolError::ModelRefused(format!(
                    "Model LAST endpoint HTTP {code}: {detail}; measured_j stays None"
                )));
            }
            Err(e) => {
                return Err(MolError::ModelRefused(format!(
                    "Model LAST endpoint transport failed ({e}); offline stub available without --endpoint; measured_j stays None"
                )));
            }
        };

        let v: serde_json::Value = resp
            .into_json()
            .map_err(|e| MolError::ModelRefused(format!("Model LAST JSON parse: {e}")))?;

        let content = v
            .pointer("/choices/0/message/content")
            .and_then(|c| c.as_str())
            .unwrap_or("")
            .trim()
            .to_string();

        if content.is_empty() {
            return Err(MolError::ModelRefused(
                "Model LAST endpoint returned empty content; measured_j stays None".into(),
            ));
        }

        let text = format!(
            "MODEL_GENERATED_PROPOSAL (Model LAST {}; openai-compatible endpoint; model={}): {}; commit requires NI/EFA certificate — never launders to Deterministic; measured_j=None",
            self.profile.label(),
            self.model,
            content
        );

        Ok(ModelLastProposal {
            text,
            profile: format!("{}_endpoint", self.profile.label()),
            estimated_j: MODEL_LAST_ENDPOINT_ESTIMATED_J,
            measured_j: None,
            offline: false,
            note: format!(
                "openai-compatible POST {url}; docs={}; catalog estimated_j only — never invent measured_j",
                self.profile.hf_or_docs()
            ),
        })
    }
}

/// Build the preferred Model LAST port: endpoint when set, else offline stub.
pub fn model_last_from_endpoint(
    endpoint: Option<&str>,
    profile: ModelLastProfile,
    model: Option<&str>,
) -> Box<dyn ModelLastPort> {
    match endpoint.map(str::trim).filter(|s| !s.is_empty()) {
        Some(url) => {
            let mut leaf = OpenAiCompatibleModelLast::new(url).with_profile(profile);
            if let Some(m) = model {
                leaf = leaf.with_model(m);
            } else {
                // Sensible default model id from profile docs.
                let m = match profile {
                    ModelLastProfile::LayaHf => "convaiinnovations/laya",
                    ModelLastProfile::DeciderHf => "Mapika/decider-4b",
                    ModelLastProfile::JevClass => "jev",
                    ModelLastProfile::Stub => "local",
                };
                leaf = leaf.with_model(m);
            }
            Box::new(leaf)
        }
        None => Box::new(StubModelLast::with_profile(profile)),
    }
}

/// Live path reserved marker (returns AdapterStub — prefer StubModelLast / endpoint).
pub fn live_model_last_stub(_req: &MolRequest) -> Result<ModelLastProposal> {
    Err(MolError::AdapterStub(
        "live Model LAST HF weights path deferred; use StubModelLast or --endpoint OpenAI-compatible".into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use mol_core::Budget;

    #[test]
    fn stub_proposes_offline_no_measured_j() {
        let leaf = StubModelLast::laya_hf();
        let mut b = Budget::demo().allow_model();
        b.max_j = mol_core::Joules::new(1.0);
        let p = leaf
            .propose(&MolRequest::new("residual propose approve ticket", b))
            .unwrap();
        assert!(p.offline);
        assert!(p.measured_j.is_none());
        assert!(p.honesty_ok());
        assert!(p.text.contains("MODEL_GENERATED_PROPOSAL"));
        assert!(p.text.contains("laya"));
    }

    #[test]
    fn stub_refuses_without_allow_model() {
        let leaf = StubModelLast::new();
        let req = MolRequest::new("residual propose x", Budget::coin_cell());
        assert!(leaf.propose(&req).is_err());
    }

    #[test]
    fn profile_parse() {
        assert_eq!(ModelLastProfile::parse("laya"), ModelLastProfile::LayaHf);
        assert_eq!(ModelLastProfile::parse("decider"), ModelLastProfile::DeciderHf);
        assert_eq!(ModelLastProfile::parse("jev"), ModelLastProfile::JevClass);
    }

    #[test]
    fn from_endpoint_none_is_stub() {
        let leaf = model_last_from_endpoint(None, ModelLastProfile::DeciderHf, None);
        let mut b = Budget::demo().allow_model();
        b.max_j = mol_core::Joules::new(1.0);
        let p = leaf
            .propose(&MolRequest::new("residual propose risk band", b))
            .unwrap();
        assert!(p.offline);
        assert!(p.measured_j.is_none());
    }
}
