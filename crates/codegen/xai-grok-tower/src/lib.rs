//! Tower contracts. This scaffold deliberately does not create a second session runtime.
//!
//! Composition root (`xai-grok-pager-bin`) injects a Shell-backed
//! [`GrokRuntimeFacade`]. This crate never imports Shell.

pub mod budgets;
pub mod fake;
pub mod instance;
pub mod lease;
pub mod metadata;
pub mod projection;
pub mod registry;
pub mod workspace;
pub use budgets::{
    admit_resident, admit_turn, ResourceBudgets, ResourceUsage,
};
pub use fake::FakeRuntime;
pub use instance::{InstanceDirectory, TowerHandle, TowerInstanceId, TowerInstanceIdError};
pub use lease::{ControllerLease, LeaseTable};
pub use metadata::{Residency, SessionMetadata};
pub use projection::{
    contains_secret_canary, project_runtime_event, project_unknown_diagnostic, redact_text,
};
pub use registry::{ActorToken, ResidentSession, SessionRegistry};

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
