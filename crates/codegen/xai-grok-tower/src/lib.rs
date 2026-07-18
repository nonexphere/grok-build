//! Tower contracts. This scaffold deliberately does not create a second session runtime.

pub mod instance;
pub use instance::{TowerHandle, TowerInstanceId, TowerInstanceIdError};

use async_trait::async_trait;
use xai_grok_app_server_protocol::{
    InteractionRequest, InteractionResponseParams, Item, Session, SessionArchiveParams,
    SessionForkParams, SessionReadParams, SessionReadResult, SessionResumeParams,
    SessionStartParams, SubscribeParams, Turn, TurnInterruptParams, TurnStartParams,
    TurnSteerParams,
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
    async fn replay(&self, cursor: SubscribeParams) -> Result<Vec<RuntimeEvent>, RuntimeError>;
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
        revision: u64,
        delta: String,
    },
    ItemCompleted(Item),
    InteractionRequested(InteractionRequest),
}
