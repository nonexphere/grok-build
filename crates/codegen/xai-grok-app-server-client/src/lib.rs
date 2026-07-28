//! Typed client boundary owned by App Server v1-03 and SDK conformance work.
//! No transport implementation or fake success behavior lives in this scaffold.

use std::pin::Pin;

use async_trait::async_trait;
use futures::Stream;
use xai_grok_app_server_protocol::{
    InitializeParams, InitializeResult, Item, OperationResult, Session, SessionArchiveParams,
    SessionForkParams, SessionListParams, SessionListResult, SessionReadParams, SessionReadResult,
    SessionResumeParams, SessionStartParams, SubscribeParams, Turn, TurnInterruptParams,
    TurnStartParams, TurnSteerParams,
};

pub type ItemStream = Pin<Box<dyn Stream<Item = Result<Item, ClientError>> + Send>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientError {
    pub rpc_code: Option<i64>,
    pub domain_code: &'static str,
    pub message: String,
    pub retryable: bool,
}

#[async_trait]
pub trait AppServerClient: Send + Sync {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult, ClientError>;
    async fn list_sessions(
        &self,
        params: SessionListParams,
    ) -> Result<SessionListResult, ClientError>;
    async fn read_session(
        &self,
        params: SessionReadParams,
    ) -> Result<SessionReadResult, ClientError>;
    async fn start_session(&self, params: SessionStartParams) -> Result<Session, ClientError>;
    async fn resume_session(&self, params: SessionResumeParams) -> Result<Session, ClientError>;
    async fn fork_session(&self, params: SessionForkParams) -> Result<Session, ClientError>;
    async fn archive_session(
        &self,
        params: SessionArchiveParams,
    ) -> Result<OperationResult, ClientError>;
    async fn start_turn(&self, params: TurnStartParams) -> Result<Turn, ClientError>;
    async fn steer_turn(&self, params: TurnSteerParams) -> Result<Item, ClientError>;
    async fn interrupt_turn(
        &self,
        params: TurnInterruptParams,
    ) -> Result<OperationResult, ClientError>;
    async fn subscribe(&self, params: SubscribeParams) -> Result<ItemStream, ClientError>;
    async fn close(&self) -> Result<(), ClientError>;
}
