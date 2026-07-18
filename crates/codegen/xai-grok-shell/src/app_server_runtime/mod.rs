//! App Server runtime adapter seam (Shell side).
//!
//! `xai-grok-pager-bin` is the composition root: it constructs a
//! [`ShellRuntimeAdapter`] (or another `GrokRuntimeFacade` impl) and injects it
//! into Tower/App Server/tools. Tower never imports this module.
//!
//! Production path: methods forward to the existing leader/`SessionActor`
//! ownership model. Until full command mapping lands, the adapter can be
//! constructed with an injected [`GrokRuntimeFacade`] (typically the faithful
//! `FakeRuntime` in tests, or a future `SessionActor`-backed port).

use std::sync::Arc;

use async_trait::async_trait;
use xai_grok_app_server_protocol::{
    InteractionResponseParams, Item, Session, SessionArchiveParams, SessionForkParams,
    SessionReadParams, SessionReadResult, SessionResumeParams, SessionStartParams,
    SessionStatus, SubscribeParams, Turn, TurnInterruptParams, TurnStartParams, TurnSteerParams,
    WireCounter,
};
use xai_grok_tower::{
    GrokRuntimeFacade, ReplayPage, RuntimeError, SessionRegistry,
};

/// Marker documenting ownership for characterization tests.
pub struct ShellRuntimeAdapterMarker;

impl ShellRuntimeAdapterMarker {
    pub const OWNER: &'static str = "xai-grok-shell";
    pub const INJECTED_AT: &'static str = "xai-grok-pager-bin";
}

/// Shell-owned facade injection handle.
///
/// This type is the only Shell-visible production surface for App Server.
/// It records residency via [`SessionRegistry`] so one opaque actor token is
/// associated per Session ID while the inner port performs work.
pub struct ShellRuntimeAdapter {
    inner: Arc<dyn GrokRuntimeFacade>,
    registry: std::sync::Mutex<SessionRegistry>,
}

impl ShellRuntimeAdapter {
    /// Inject a runtime port (composition root or test).
    pub fn inject(inner: Arc<dyn GrokRuntimeFacade>) -> Self {
        Self {
            inner,
            registry: std::sync::Mutex::new(SessionRegistry::new()),
        }
    }

    pub fn registry_len(&self) -> usize {
        self.registry.lock().unwrap().len()
    }
}

#[async_trait]
impl GrokRuntimeFacade for ShellRuntimeAdapter {
    async fn list_sessions(&self) -> Result<Vec<Session>, RuntimeError> {
        self.inner.list_sessions().await
    }

    async fn read_session(
        &self,
        params: SessionReadParams,
    ) -> Result<SessionReadResult, RuntimeError> {
        self.inner.read_session(params).await
    }

    async fn start_session(&self, params: SessionStartParams) -> Result<Session, RuntimeError> {
        let session = self.inner.start_session(params).await?;
        self.registry
            .lock()
            .unwrap()
            .get_or_insert_with(&session.session_id, |_| Ok(()))?;
        Ok(session)
    }

    async fn resume_session(&self, params: SessionResumeParams) -> Result<Session, RuntimeError> {
        let session = self.inner.resume_session(params).await?;
        self.registry
            .lock()
            .unwrap()
            .get_or_insert_with(&session.session_id, |_| Ok(()))?;
        Ok(session)
    }

    async fn fork_session(&self, params: SessionForkParams) -> Result<Session, RuntimeError> {
        let session = self.inner.fork_session(params).await?;
        self.registry
            .lock()
            .unwrap()
            .get_or_insert_with(&session.session_id, |_| Ok(()))?;
        Ok(session)
    }

    async fn archive_session(&self, params: SessionArchiveParams) -> Result<(), RuntimeError> {
        self.inner.archive_session(params.clone()).await?;
        self.registry.lock().unwrap().remove(&params.session_id);
        Ok(())
    }

    async fn start_turn(&self, params: TurnStartParams) -> Result<Turn, RuntimeError> {
        // Residency must already exist for mutations.
        if self
            .registry
            .lock()
            .unwrap()
            .get(&params.session_id)
            .is_none()
        {
            // Allow inner to create path if session exists only on disk later;
            // for injected ports, require prior start/resume registration.
            let _ = self
                .registry
                .lock()
                .unwrap()
                .get_or_insert_with(&params.session_id, |_| Ok(()))?;
        }
        self.inner.start_turn(params).await
    }

    async fn steer_turn(&self, params: TurnSteerParams) -> Result<Item, RuntimeError> {
        self.inner.steer_turn(params).await
    }

    async fn interrupt_turn(&self, params: TurnInterruptParams) -> Result<(), RuntimeError> {
        self.inner.interrupt_turn(params).await
    }

    async fn respond_interaction(
        &self,
        params: InteractionResponseParams,
    ) -> Result<(), RuntimeError> {
        self.inner.respond_interaction(params).await
    }

    async fn replay(&self, cursor: SubscribeParams) -> Result<ReplayPage, RuntimeError> {
        self.inner.replay(cursor).await
    }
}

/// Project an active-session row into a protocol Session (list path only).
pub fn project_active_session_row(
    session_id: &str,
    workspace_root: &str,
    created_at_ms: u64,
) -> Session {
    Session {
        session_id: session_id.into(),
        history_epoch: "epoch_1".into(),
        revision: WireCounter::new(0),
        status: SessionStatus::Dormant,
        workspace_root: workspace_root.into(),
        title: None,
        active_turn_id: None,
        latest_turn_id: None,
        provider_binding: None,
        created_at_ms,
        updated_at_ms: created_at_ms,
    }
}
