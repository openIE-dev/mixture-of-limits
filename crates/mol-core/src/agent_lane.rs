//! Agent Lane–class isolation — no shared cookies/profile/storage with opaque bots.
//!
//! Verified Nova pattern (`web-browser/docs/agent-lane.md`):
//! - `AppLane::{Human,Agent}` with separate profile root
//! - Isolated cookie jar for Agent Lane
//! - Honest gap in Nova MVP: localStorage/IndexedDB not yet partitioned
//!
//! MoL **real session path** (this module): logical cookie / profile / storage
//! partitions are separate per lane; host invoke requires lane provenance +
//! typed [`GrantReceipt`] (keyword confirm alone is insufficient); cross-lane
//! share is refused; successful invoke stamps [`AgentLaneReceipt`].

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::floor::{Floor, FloorKind};

/// Human browsing vs agent actuation lane.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppLane {
    /// Human session (cookies/profile for people).
    Human,
    /// Agent session — must not share human cookies/profile/high-risk surfaces.
    Agent,
}

impl AppLane {
    /// Wire label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Human => "human",
            Self::Agent => "agent",
        }
    }
}

impl fmt::Display for AppLane {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// Logical partition surface inside a lane session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PartitionSurface {
    /// Cookie jar.
    Cookies,
    /// Profile root.
    Profile,
    /// Web storage (localStorage / IndexedDB–class).
    Storage,
}

impl PartitionSurface {
    /// Wire label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Cookies => "cookies",
            Self::Profile => "profile",
            Self::Storage => "storage",
        }
    }
}

impl fmt::Display for PartitionSurface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// One logical partition root — never aliased across lanes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LanePartition {
    /// Owning lane.
    pub lane: AppLane,
    /// Surface.
    pub surface: PartitionSurface,
    /// Logical root id (e.g. `lane:agent/cookies`).
    pub root_id: String,
}

impl LanePartition {
    /// Build a lane-scoped root id.
    pub fn for_lane(lane: AppLane, surface: PartitionSurface) -> Self {
        Self {
            lane,
            surface,
            root_id: format!("lane:{}/{}", lane.label(), surface.label()),
        }
    }

    /// True when this partition belongs to `lane`.
    pub fn belongs_to(&self, lane: AppLane) -> bool {
        self.lane == lane
    }
}

/// Isolation posture for an agent (or human) lane.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentIsolationPolicy {
    /// Which lane.
    pub lane: AppLane,
    /// Separate profile root (Nova: `~/.nova/profiles/agent`).
    pub separate_profile: bool,
    /// Separate cookie jar.
    pub separate_cookies: bool,
    /// Separate web storage (Nova honest gap: often still false).
    pub separate_storage: bool,
    /// Anti-pattern: allow sharing session with opaque frontier-vendor bots.
    pub allow_shared_opaque_bot: bool,
}

impl AgentIsolationPolicy {
    /// Strict Agent Lane (cookies+profile+storage isolated).
    pub fn strict_agent() -> Self {
        Self {
            lane: AppLane::Agent,
            separate_profile: true,
            separate_cookies: true,
            separate_storage: true,
            allow_shared_opaque_bot: false,
        }
    }

    /// Nova-MVP-shaped agent: cookies+profile yes, storage not yet.
    pub fn nova_mvp_agent() -> Self {
        Self {
            lane: AppLane::Agent,
            separate_profile: true,
            separate_cookies: true,
            separate_storage: false,
            allow_shared_opaque_bot: false,
        }
    }

    /// Broken / opaque-bot anti-pattern (must refuse).
    pub fn opaque_shared_bot() -> Self {
        Self {
            lane: AppLane::Agent,
            separate_profile: false,
            separate_cookies: false,
            separate_storage: false,
            allow_shared_opaque_bot: true,
        }
    }

    /// Human lane default.
    pub fn human() -> Self {
        Self {
            lane: AppLane::Human,
            separate_profile: true,
            separate_cookies: true,
            separate_storage: true,
            allow_shared_opaque_bot: false,
        }
    }

    /// Isolation law — Agent lane must isolate cookies+profile; opaque share refused.
    ///
    /// Storage partition is **required** on the real [`AgentLaneSession`] path
    /// (`session.check()`), but not on Nova-MVP policy (honest gap).
    pub fn check(&self) -> Result<(), Floor> {
        if self.allow_shared_opaque_bot {
            return Err(Floor::new(
                "agent_isolation_opaque_bot",
                FloorKind::AgentIsolation,
                "refuse: allow_shared_opaque_bot — frontier-vendor opaque multi-tool agent loops forbidden",
            ));
        }
        if self.lane == AppLane::Agent {
            if !self.separate_profile {
                return Err(Floor::new(
                    "agent_isolation_profile",
                    FloorKind::AgentIsolation,
                    "Agent Lane requires separate_profile (no shared human profile)",
                ));
            }
            if !self.separate_cookies {
                return Err(Floor::new(
                    "agent_isolation_cookies",
                    FloorKind::AgentIsolation,
                    "Agent Lane requires separate_cookies (no shared human cookie jar)",
                ));
            }
        }
        Ok(())
    }
}

/// Real Agent Lane session — separate logical cookie / profile / storage partitions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentLaneSession {
    /// Session id (logical; not a browser process handle).
    pub session_id: String,
    /// Lane this session belongs to.
    pub lane: AppLane,
    /// Cookie jar partition.
    pub cookies: LanePartition,
    /// Profile partition.
    pub profile: LanePartition,
    /// Storage partition (required on real path — closes Nova honest gap in MoL law).
    pub storage: LanePartition,
    /// Isolation policy mirrored from partitions.
    pub policy: AgentIsolationPolicy,
}

impl AgentLaneSession {
    /// Open a strict Agent Lane session (all three surfaces partitioned).
    pub fn open_agent(session_id: impl Into<String>) -> Self {
        let session_id = session_id.into();
        let lane = AppLane::Agent;
        Self {
            session_id,
            lane,
            cookies: LanePartition::for_lane(lane, PartitionSurface::Cookies),
            profile: LanePartition::for_lane(lane, PartitionSurface::Profile),
            storage: LanePartition::for_lane(lane, PartitionSurface::Storage),
            policy: AgentIsolationPolicy::strict_agent(),
        }
    }

    /// Open a Human Lane session (all three surfaces partitioned).
    pub fn open_human(session_id: impl Into<String>) -> Self {
        let session_id = session_id.into();
        let lane = AppLane::Human;
        Self {
            session_id,
            lane,
            cookies: LanePartition::for_lane(lane, PartitionSurface::Cookies),
            profile: LanePartition::for_lane(lane, PartitionSurface::Profile),
            storage: LanePartition::for_lane(lane, PartitionSurface::Storage),
            policy: AgentIsolationPolicy::human(),
        }
    }

    /// Partition roots must be lane-owned and mutually distinct across surfaces.
    pub fn check(&self) -> Result<(), Floor> {
        self.policy.check()?;
        if self.lane == AppLane::Agent && !self.policy.separate_storage {
            return Err(Floor::new(
                "agent_isolation_storage",
                FloorKind::AgentIsolation,
                "real Agent Lane session requires separate_storage partition",
            ));
        }
        for part in [&self.cookies, &self.profile, &self.storage] {
            if !part.belongs_to(self.lane) {
                return Err(Floor::new(
                    "agent_isolation_cross_lane",
                    FloorKind::AgentIsolation,
                    format!(
                        "partition {} root '{}' does not belong to lane {}",
                        part.surface, part.root_id, self.lane
                    ),
                ));
            }
        }
        if self.cookies.root_id == self.profile.root_id
            || self.cookies.root_id == self.storage.root_id
            || self.profile.root_id == self.storage.root_id
        {
            return Err(Floor::new(
                "agent_isolation_partition_collapse",
                FloorKind::AgentIsolation,
                "cookie/profile/storage partition roots must be distinct",
            ));
        }
        Ok(())
    }

    /// Refuse any attempt to share a partition root across lanes.
    pub fn refuse_cross_lane_share(
        &self,
        other: &AgentLaneSession,
        surface: PartitionSurface,
    ) -> Result<(), Floor> {
        if self.lane == other.lane {
            return Ok(());
        }
        let mine = self.partition(surface);
        let theirs = other.partition(surface);
        if mine.root_id == theirs.root_id {
            return Err(Floor::new(
                "agent_isolation_cross_lane_share",
                FloorKind::AgentIsolation,
                format!(
                    "refuse cross-lane share of {} (root '{}') between {} and {}",
                    surface, mine.root_id, self.lane, other.lane
                ),
            ));
        }
        // Even distinct roots: attaching the other lane's root into this session is refuse.
        Err(Floor::new(
            "agent_isolation_cross_lane_share",
            FloorKind::AgentIsolation,
            format!(
                "refuse cross-lane share of {} between {} session '{}' and {} session '{}'",
                surface, self.lane, self.session_id, other.lane, other.session_id
            ),
        ))
    }

    /// Partition for a surface.
    pub fn partition(&self, surface: PartitionSurface) -> &LanePartition {
        match surface {
            PartitionSurface::Cookies => &self.cookies,
            PartitionSurface::Profile => &self.profile,
            PartitionSurface::Storage => &self.storage,
        }
    }

    /// Stamp receipt projection (no grant yet).
    pub fn to_receipt(&self) -> AgentLaneReceipt {
        AgentLaneReceipt {
            lane: self.lane,
            separate_profile: self.policy.separate_profile,
            separate_cookies: self.policy.separate_cookies,
            separate_storage: self.policy.separate_storage,
            allow_shared_opaque_bot: self.policy.allow_shared_opaque_bot,
            session_id: Some(self.session_id.clone()),
            cookies_root: Some(self.cookies.root_id.clone()),
            profile_root: Some(self.profile.root_id.clone()),
            storage_root: Some(self.storage.root_id.clone()),
            grant_id: None,
        }
    }
}

/// Lane provenance attached to a host invoke (WebMCP-class fail-closed).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LaneProvenance {
    /// Session id that originated the invoke.
    pub session_id: String,
    /// Lane.
    pub lane: AppLane,
    /// Partition root the invoke is scoped to (usually cookies or storage).
    pub partition_root: String,
}

impl LaneProvenance {
    /// Build provenance from a live session + surface.
    pub fn from_session(session: &AgentLaneSession, surface: PartitionSurface) -> Self {
        Self {
            session_id: session.session_id.clone(),
            lane: session.lane,
            partition_root: session.partition(surface).root_id.clone(),
        }
    }
}

/// Host capability invoked through the Agent Lane host bridge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostCapability {
    /// `document.modelContext` execute-class.
    ModelContextExecute,
    /// Navigation / actuation.
    Navigate,
    /// Read from lane storage partition.
    StorageRead,
    /// Write to lane storage partition.
    StorageWrite,
}

impl HostCapability {
    /// Wire label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::ModelContextExecute => "model_context_execute",
            Self::Navigate => "navigate",
            Self::StorageRead => "storage_read",
            Self::StorageWrite => "storage_write",
        }
    }
}

impl fmt::Display for HostCapability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// Keyword-only confirm (e.g. typing "confirm") — **insufficient** alone.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeywordConfirm {
    /// Keyword string ("confirm", "yes", …).
    pub keyword: String,
}

impl KeywordConfirm {
    /// Common affirmative keywords (still insufficient without grant).
    pub fn is_affirmative(&self) -> bool {
        let k = self.keyword.trim().to_ascii_lowercase();
        matches!(k.as_str(), "confirm" | "yes" | "y" | "ok" | "approve")
    }
}

/// Typed grant receipt — required for consequential host invoke.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrantReceipt {
    /// Stable grant id.
    pub grant_id: String,
    /// Session the grant was issued to.
    pub session_id: String,
    /// Lane the grant binds.
    pub lane: AppLane,
    /// Capability covered.
    pub capability: HostCapability,
    /// Monotonic issue sequence (logical clock; not wall time).
    pub issued_seq: u64,
}

impl GrantReceipt {
    /// Issue a grant for a session + capability.
    pub fn issue(
        session: &AgentLaneSession,
        capability: HostCapability,
        issued_seq: u64,
    ) -> Self {
        Self {
            grant_id: format!(
                "grant:{}:{}:{}",
                session.session_id,
                capability.label(),
                issued_seq
            ),
            session_id: session.session_id.clone(),
            lane: session.lane,
            capability,
            issued_seq,
        }
    }

    /// True when this grant covers the session + capability.
    pub fn covers(&self, session: &AgentLaneSession, capability: HostCapability) -> bool {
        self.session_id == session.session_id
            && self.lane == session.lane
            && self.capability == capability
    }
}

/// Host invoke request (fail-closed without lane provenance + grant).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostInvokeRequest {
    /// Capability to invoke.
    pub capability: HostCapability,
    /// Target session (must already be partition-checked).
    pub session: AgentLaneSession,
    /// Lane provenance — **required**; missing → refuse.
    pub provenance: Option<LaneProvenance>,
    /// Keyword confirm alone is **not** enough.
    pub keyword_confirm: Option<KeywordConfirm>,
    /// Typed grant receipt — **required** for allow.
    pub grant: Option<GrantReceipt>,
}

/// Decision from [`host_invoke`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostInvokeDecision {
    /// Allowed — receipt stamped with partitions + grant id.
    Allow {
        /// Agent Lane receipt stamp.
        receipt: AgentLaneReceipt,
        /// Grant id that authorized the invoke.
        grant_id: String,
    },
    /// Refused — floor + partial receipt stamp for audit.
    Refuse {
        /// Binding floor.
        floor: Floor,
        /// Agent Lane receipt (session partitions when known).
        receipt: AgentLaneReceipt,
    },
}

impl HostInvokeDecision {
    /// True on allow.
    pub fn is_allow(&self) -> bool {
        matches!(self, Self::Allow { .. })
    }

    /// True on refuse.
    pub fn is_refuse(&self) -> bool {
        matches!(self, Self::Refuse { .. })
    }

    /// Floor when refused.
    pub fn floor(&self) -> Option<&Floor> {
        match self {
            Self::Refuse { floor, .. } => Some(floor),
            Self::Allow { .. } => None,
        }
    }

    /// Stamped Agent Lane receipt (both arms).
    pub fn receipt(&self) -> &AgentLaneReceipt {
        match self {
            Self::Allow { receipt, .. } | Self::Refuse { receipt, .. } => receipt,
        }
    }
}

/// Fail-closed host invoke for Agent Lane.
///
/// Law:
/// 1. Session partitions must check (cookies/profile/storage separate; no cross-lane).
/// 2. Lane provenance required — else `provenance_missing_lane`.
/// 3. Provenance must match session lane + session_id + a real partition root.
/// 4. Keyword confirm alone is insufficient — need [`GrantReceipt`].
/// 5. Grant must cover session + capability.
/// 6. On allow, stamp [`AgentLaneReceipt`] with partition roots + grant_id.
pub fn host_invoke(req: &HostInvokeRequest) -> HostInvokeDecision {
    let mut receipt = req.session.to_receipt();

    if let Err(floor) = req.session.check() {
        return HostInvokeDecision::Refuse { floor, receipt };
    }

    let Some(ref prov) = req.provenance else {
        let floor = Floor::new(
            "provenance_missing_lane",
            FloorKind::ProvenanceMissing,
            "fail-closed: host invoke requires lane provenance (session_id + lane + partition_root)",
        );
        return HostInvokeDecision::Refuse { floor, receipt };
    };

    if prov.session_id != req.session.session_id || prov.lane != req.session.lane {
        let floor = Floor::new(
            "agent_isolation_provenance_mismatch",
            FloorKind::AgentIsolation,
            format!(
                "lane provenance session/lane ({}/{}) does not match invoke session ({}/{})",
                prov.session_id, prov.lane, req.session.session_id, req.session.lane
            ),
        );
        return HostInvokeDecision::Refuse { floor, receipt };
    }

    let roots = [
        req.session.cookies.root_id.as_str(),
        req.session.profile.root_id.as_str(),
        req.session.storage.root_id.as_str(),
    ];
    if !roots.contains(&prov.partition_root.as_str()) {
        let floor = Floor::new(
            "agent_isolation_cross_lane",
            FloorKind::AgentIsolation,
            format!(
                "provenance partition_root '{}' is not owned by session '{}' — no cross-lane share",
                prov.partition_root, req.session.session_id
            ),
        );
        return HostInvokeDecision::Refuse { floor, receipt };
    }

    // Keyword confirm is never sufficient by itself.
    let grant = match &req.grant {
        None => {
            let detail = if req.keyword_confirm.as_ref().is_some_and(|k| k.is_affirmative()) {
                "keyword confirm insufficient — host invoke requires typed GrantReceipt"
            } else {
                "host invoke requires typed GrantReceipt (keyword confirm alone never enough)"
            };
            let floor = Floor::new(
                "grant_receipt_required",
                FloorKind::AgentIsolation,
                detail,
            );
            return HostInvokeDecision::Refuse { floor, receipt };
        }
        Some(g) => g,
    };

    if !grant.covers(&req.session, req.capability) {
        let floor = Floor::new(
            "grant_receipt_mismatch",
            FloorKind::AgentIsolation,
            format!(
                "grant '{}' does not cover session '{}' capability {}",
                grant.grant_id, req.session.session_id, req.capability
            ),
        );
        return HostInvokeDecision::Refuse { floor, receipt };
    }

    receipt.grant_id = Some(grant.grant_id.clone());
    HostInvokeDecision::Allow {
        grant_id: grant.grant_id.clone(),
        receipt,
    }
}

/// Receipt projection of agent lane isolation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentLaneReceipt {
    /// Lane.
    pub lane: AppLane,
    /// Separate profile.
    pub separate_profile: bool,
    /// Separate cookies.
    pub separate_cookies: bool,
    /// Separate storage (may be false on Nova-MVP policy — honest gap).
    pub separate_storage: bool,
    /// Opaque-bot share allowed (must be false for commit).
    pub allow_shared_opaque_bot: bool,
    /// Session id when stamped from a real [`AgentLaneSession`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// Logical cookies partition root.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cookies_root: Option<String>,
    /// Logical profile partition root.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile_root: Option<String>,
    /// Logical storage partition root.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub storage_root: Option<String>,
    /// Grant id that authorized a host invoke (when applicable).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grant_id: Option<String>,
}

impl From<&AgentIsolationPolicy> for AgentLaneReceipt {
    fn from(p: &AgentIsolationPolicy) -> Self {
        Self {
            lane: p.lane,
            separate_profile: p.separate_profile,
            separate_cookies: p.separate_cookies,
            separate_storage: p.separate_storage,
            allow_shared_opaque_bot: p.allow_shared_opaque_bot,
            session_id: None,
            cookies_root: None,
            profile_root: None,
            storage_root: None,
            grant_id: None,
        }
    }
}

impl From<&AgentLaneSession> for AgentLaneReceipt {
    fn from(s: &AgentLaneSession) -> Self {
        s.to_receipt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strict_agent_ok() {
        assert!(AgentIsolationPolicy::strict_agent().check().is_ok());
        assert!(AgentIsolationPolicy::nova_mvp_agent().check().is_ok());
    }

    #[test]
    fn opaque_bot_refused() {
        let err = AgentIsolationPolicy::opaque_shared_bot()
            .check()
            .unwrap_err();
        assert_eq!(err.kind, FloorKind::AgentIsolation);
        assert_eq!(err.id.as_str(), "agent_isolation_opaque_bot");
    }

    #[test]
    fn agent_without_cookies_refused() {
        let mut p = AgentIsolationPolicy::strict_agent();
        p.separate_cookies = false;
        let err = p.check().unwrap_err();
        assert_eq!(err.id.as_str(), "agent_isolation_cookies");
    }

    #[test]
    fn session_partitions_distinct() {
        let s = AgentLaneSession::open_agent("s1");
        assert!(s.check().is_ok());
        assert_ne!(s.cookies.root_id, s.storage.root_id);
        assert!(s.policy.separate_storage);
    }

    #[test]
    fn cross_lane_share_refused() {
        let human = AgentLaneSession::open_human("h1");
        let agent = AgentLaneSession::open_agent("a1");
        let err = agent
            .refuse_cross_lane_share(&human, PartitionSurface::Cookies)
            .unwrap_err();
        assert_eq!(err.id.as_str(), "agent_isolation_cross_lane_share");
    }

    #[test]
    fn host_invoke_needs_provenance() {
        let session = AgentLaneSession::open_agent("a1");
        let grant = GrantReceipt::issue(&session, HostCapability::ModelContextExecute, 1);
        let d = host_invoke(&HostInvokeRequest {
            capability: HostCapability::ModelContextExecute,
            session,
            provenance: None,
            keyword_confirm: Some(KeywordConfirm {
                keyword: "confirm".into(),
            }),
            grant: Some(grant),
        });
        assert!(d.is_refuse());
        assert_eq!(
            d.floor().map(|f| f.id.as_str()),
            Some("provenance_missing_lane")
        );
    }

    #[test]
    fn keyword_confirm_insufficient() {
        let session = AgentLaneSession::open_agent("a1");
        let prov = LaneProvenance::from_session(&session, PartitionSurface::Cookies);
        let d = host_invoke(&HostInvokeRequest {
            capability: HostCapability::Navigate,
            session,
            provenance: Some(prov),
            keyword_confirm: Some(KeywordConfirm {
                keyword: "confirm".into(),
            }),
            grant: None,
        });
        assert!(d.is_refuse());
        assert_eq!(
            d.floor().map(|f| f.id.as_str()),
            Some("grant_receipt_required")
        );
    }

    #[test]
    fn grant_plus_provenance_allows_and_stamps() {
        let session = AgentLaneSession::open_agent("a1");
        let prov = LaneProvenance::from_session(&session, PartitionSurface::Storage);
        let grant = GrantReceipt::issue(&session, HostCapability::StorageWrite, 7);
        let d = host_invoke(&HostInvokeRequest {
            capability: HostCapability::StorageWrite,
            session: session.clone(),
            provenance: Some(prov),
            keyword_confirm: None,
            grant: Some(grant.clone()),
        });
        assert!(d.is_allow());
        let r = d.receipt();
        assert_eq!(r.session_id.as_deref(), Some("a1"));
        assert_eq!(r.cookies_root.as_deref(), Some("lane:agent/cookies"));
        assert_eq!(r.profile_root.as_deref(), Some("lane:agent/profile"));
        assert_eq!(r.storage_root.as_deref(), Some("lane:agent/storage"));
        assert_eq!(r.grant_id.as_deref(), Some(grant.grant_id.as_str()));
        assert!(r.separate_cookies && r.separate_profile && r.separate_storage);
    }

    #[test]
    fn foreign_partition_root_refused() {
        let session = AgentLaneSession::open_agent("a1");
        let mut prov = LaneProvenance::from_session(&session, PartitionSurface::Cookies);
        prov.partition_root = "lane:human/cookies".into();
        let grant = GrantReceipt::issue(&session, HostCapability::Navigate, 1);
        let d = host_invoke(&HostInvokeRequest {
            capability: HostCapability::Navigate,
            session,
            provenance: Some(prov),
            keyword_confirm: None,
            grant: Some(grant),
        });
        assert!(d.is_refuse());
        assert_eq!(
            d.floor().map(|f| f.id.as_str()),
            Some("agent_isolation_cross_lane")
        );
    }
}
