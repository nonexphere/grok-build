//! Tower contracts. This scaffold deliberately does not create a second session runtime.
//!
//! Composition root (`xai-grok-pager-bin`) injects a Shell-backed
//! [`GrokRuntimeFacade`]. This crate never imports Shell.

pub mod budgets;
pub mod fake;
pub mod instance;
pub mod lease;
pub mod lifecycle;
pub mod lock;
pub mod metadata;
pub mod projection;
pub mod registry;
pub mod telemetry;
pub mod workspace;
pub use budgets::{ResourceBudgets, ResourceUsage, admit_resident, admit_turn};
pub use fake::FakeRuntime;
pub use instance::{
    InstanceDirectory, TOWER_INSTANCES_DIR, TowerHandle, TowerInstanceId, TowerInstanceIdError,
    instance_state_root,
};
pub use lease::{ControllerLease, LeaseTable};
pub use lifecycle::{DrainController, DrainState};
pub use lock::{
    INSTANCE_ENDPOINT_FILE, INSTANCE_LOCK_FILE, INSTANCE_METADATA_FILE, INSTANCE_TOKEN_FILE,
    InstanceLock, InstanceLockError,
};
pub use metadata::{Residency, SessionMetadata};
pub use projection::{
    contains_secret_canary, project_runtime_event, project_unknown_diagnostic, redact_text,
};
pub use registry::{ActorToken, ResidentSession, SessionRegistry};
pub use telemetry::LifecycleMetrics;

use async_trait::async_trait;
use xai_grok_app_server_protocol::{
    InteractionRequest, InteractionResponseParams, Item, Session, SessionArchiveParams,
    SessionForkParams, SessionReadParams, SessionReadResult, SessionResumeParams,
    SessionStartParams, SubscribeParams, Turn, TurnInterruptParams, TurnStartParams,
    TurnSteerParams, WireCounter,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeError {
    pub code: &'static str,
    pub message: String,
}

#[async_trait]
pub trait GrokRuntimeFacade: Send + Sync {
    /// Capabilities that are executable by this concrete runtime instance.
    /// Adapters must not advertise a method merely because the wire protocol
    /// knows about it; product composition can honestly narrow this set until
    /// a real actor factory is installed.
    fn capabilities(&self) -> RuntimeCapabilities {
        RuntimeCapabilities::all()
    }

    async fn list_sessions(&self) -> Result<Vec<Session>, RuntimeError>;
    async fn read_session(
        &self,
        params: SessionReadParams,
    ) -> Result<SessionReadResult, RuntimeError>;
    async fn start_session(&self, params: SessionStartParams) -> Result<Session, RuntimeError>;
    async fn resume_session(&self, params: SessionResumeParams) -> Result<Session, RuntimeError>;
    async fn fork_session(&self, params: SessionForkParams) -> Result<Session, RuntimeError>;
    async fn archive_session(&self, params: SessionArchiveParams) -> Result<(), RuntimeError>;
    async fn start_turn(&self, params: TurnStartParams) -> Result<Turn, RuntimeError>;
    async fn steer_turn(&self, params: TurnSteerParams) -> Result<Item, RuntimeError>;
    async fn interrupt_turn(&self, params: TurnInterruptParams) -> Result<(), RuntimeError>;
    async fn respond_interaction(
        &self,
        params: InteractionResponseParams,
    ) -> Result<(), RuntimeError>;
    async fn replay(&self, cursor: SubscribeParams) -> Result<ReplayPage, RuntimeError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeCapabilities {
    pub session_list: bool,
    pub session_read: bool,
    pub session_start: bool,
    pub session_resume: bool,
    pub session_fork: bool,
    pub session_archive: bool,
    pub session_subscribe: bool,
    pub turn_start: bool,
    pub turn_steer: bool,
    pub turn_interrupt: bool,
    pub interaction_respond: bool,
    pub item_lifecycle: bool,
    pub item_deltas: bool,
}

impl RuntimeCapabilities {
    pub const fn all() -> Self {
        Self {
            session_list: true,
            session_read: true,
            session_start: true,
            session_resume: true,
            session_fork: true,
            session_archive: true,
            session_subscribe: true,
            turn_start: true,
            turn_steer: true,
            turn_interrupt: true,
            interaction_respond: true,
            item_lifecycle: true,
            item_deltas: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReplayPage {
    pub events: Vec<RuntimeEvent>,
    pub replayed_through: WireCounter,
    pub next_cursor: Option<WireCounter>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeEvent {
    SessionChanged(Session),
    TurnChanged(Turn),
    ItemStarted(Item),
    ItemDelta {
        session_id: String,
        turn_id: String,
        item_id: String,
        revision: WireCounter,
        delta: String,
    },
    ItemCompleted(Item),
    InteractionRequested(InteractionRequest),
}

#[cfg(test)]
mod leader_characterization_tests {
    use super::*;

    /// Documents the Wave-0 ownership claim: Tower owns only facade/registry
    /// seams; it does not define a SessionActor type or import Shell.
    #[test]
    fn leader_characterization_tower_has_no_second_actor_type() {
        // Production sources under this crate must not define a SessionActor type.
        // (Comments may mention SessionActor as the Shell-owned authority.)
        let production_sources = [
            include_str!("lib.rs")
                .split("#[cfg(test)]")
                .next()
                .expect("production lib"),
            include_str!("registry.rs")
                .split("#[cfg(test)]")
                .next()
                .expect("production registry"),
            include_str!("instance.rs")
                .split("#[cfg(test)]")
                .next()
                .expect("production instance"),
        ];
        for src in production_sources {
            assert!(
                !src.contains("struct SessionActor") && !src.contains("enum SessionActor"),
                "Tower must not define SessionActor"
            );
        }
        let cargo = include_str!("../Cargo.toml");
        assert!(
            !cargo.contains("xai-grok-shell"),
            "Tower must not depend on Shell; composition root injects the adapter"
        );
        // Registry tokens are opaque and do not embed runtime state machines.
        let mut registry = SessionRegistry::new();
        let (token, created) = registry
            .get_or_insert_with("s1", |_| Ok(()))
            .expect("insert");
        assert!(created);
        assert_eq!(token.as_u64(), 1);
        assert_eq!(registry.get("s1"), Some(token));
    }
}
