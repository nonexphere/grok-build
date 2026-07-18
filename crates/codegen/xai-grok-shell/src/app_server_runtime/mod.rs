//! App Server runtime adapter seam (Shell side).
//!
//! `xai-grok-pager-bin` is the composition root: it constructs a
//! [`ShellRuntimeAdapter`] (or another `GrokRuntimeFacade` impl) and injects it
//! into Tower/App Server/tools. Tower never imports this module.
//!
//! Production path: methods must eventually forward to the existing
//! leader/`SessionActor` ownership model. Until that command mapping lands,
//! tests inject a faithful [`xai_grok_tower::FakeRuntime`]. Do **not** mix real
//! JSONL list/read with FakeRuntime mutations (split authority).

use std::sync::Arc;

use async_trait::async_trait;
use xai_grok_app_server_protocol::{
    InteractionResponseParams, Item, Session, SessionArchiveParams, SessionForkParams,
    SessionReadParams, SessionReadResult, SessionResumeParams, SessionStartParams, SessionStatus,
    SubscribeParams, Turn, TurnInterruptParams, TurnStartParams, TurnSteerParams, WireCounter,
};
use xai_grok_tower::{GrokRuntimeFacade, ReplayPage, RuntimeError, SessionRegistry};

/// Marker documenting ownership for characterization tests.
pub struct ShellRuntimeAdapterMarker;

impl ShellRuntimeAdapterMarker {
    pub const OWNER: &'static str = "xai-grok-shell";
    pub const INJECTED_AT: &'static str = "xai-grok-pager-bin";
}

/// Shell-owned facade injection handle.
///
/// Records residency via [`SessionRegistry`] so one opaque actor token is
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
        if self
            .registry
            .lock()
            .unwrap()
            .get(&params.session_id)
            .is_none()
        {
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

#[cfg(test)]
mod app_server_runtime_tests {
    use super::*;
    use xai_grok_app_server_protocol::{InputBlock, SessionStartParams, TurnStartParams};
    use xai_grok_tower::FakeRuntime;

    #[test]
    fn app_server_runtime_adapter_lives_in_shell_not_tower() {
        assert_eq!(ShellRuntimeAdapterMarker::OWNER, "xai-grok-shell");
        assert_eq!(ShellRuntimeAdapterMarker::INJECTED_AT, "xai-grok-pager-bin");
        let tower_cargo = include_str!("../../../xai-grok-tower/Cargo.toml");
        assert!(
            !tower_cargo.contains("xai-grok-shell"),
            "Tower must not depend on Shell"
        );
    }

    #[test]
    fn app_server_runtime_defines_no_session_actor_state_machine() {
        let src = include_str!("mod.rs");
        let production = src.split("#[cfg(test)]").next().unwrap();
        assert!(!production.contains("struct SessionActor"));
        assert!(!production.contains("enum SessionActor"));
    }

    #[tokio::test]
    async fn app_server_runtime_registers_one_actor_token_per_session() {
        let adapter = ShellRuntimeAdapter::inject(Arc::new(FakeRuntime::new()));
        let s = adapter
            .start_session(SessionStartParams {
                workspace_root: "/work".into(),
                agent_type: None,
                provider_binding: None,
                idempotency_key: "a1".into(),
            })
            .await
            .unwrap();
        let s2 = adapter
            .start_session(SessionStartParams {
                workspace_root: "/work".into(),
                agent_type: None,
                provider_binding: None,
                idempotency_key: "a1".into(),
            })
            .await
            .unwrap();
        assert_eq!(s.session_id, s2.session_id);
        assert_eq!(adapter.registry_len(), 1);
        let _ = adapter
            .start_turn(TurnStartParams {
                session_id: s.session_id.clone(),
                input: vec![InputBlock::Text {
                    text: "hi".into(),
                }],
                idempotency_key: "t1".into(),
            })
            .await
            .unwrap();
        assert_eq!(adapter.registry_len(), 1);
    }

    #[tokio::test]
    async fn single_actor_owns_turn_mutation() {
        let adapter = Arc::new(ShellRuntimeAdapter::inject(Arc::new(FakeRuntime::new())));
        let session = adapter
            .start_session(SessionStartParams {
                workspace_root: "/work".into(),
                agent_type: None,
                provider_binding: None,
                idempotency_key: "race-s".into(),
            })
            .await
            .unwrap();
        let mut handles = Vec::new();
        for i in 0..8 {
            let adapter = adapter.clone();
            let session_id = session.session_id.clone();
            handles.push(tokio::spawn(async move {
                adapter
                    .start_turn(TurnStartParams {
                        session_id,
                        input: vec![InputBlock::Text {
                            text: format!("m{i}"),
                        }],
                        idempotency_key: format!("race-t-{i}"),
                    })
                    .await
            }));
        }
        let mut ok = 0;
        for h in handles {
            if h.await.unwrap().is_ok() {
                ok += 1;
            }
        }
        assert_eq!(ok, 8);
        assert_eq!(adapter.registry_len(), 1);
    }

    #[test]
    fn project_active_session_row_is_dormant_metadata_only() {
        let s = project_active_session_row("sid", "/work", 1);
        assert_eq!(s.status, SessionStatus::Dormant);
        assert_eq!(s.workspace_root, "/work");
    }
}

#[cfg(test)]
mod multi_workspace_tests {
    use super::*;
    use xai_grok_app_server_protocol::SessionStartParams;
    use xai_grok_tower::FakeRuntime;

    #[tokio::test]
    async fn app_server_multi_workspace_stable_session_ids() {
        let adapter = ShellRuntimeAdapter::inject(Arc::new(FakeRuntime::new()));
        let a = adapter
            .start_session(SessionStartParams {
                workspace_root: "/work/a".into(),
                agent_type: None,
                provider_binding: None,
                idempotency_key: "ws-a".into(),
            })
            .await
            .unwrap();
        let b = adapter
            .start_session(SessionStartParams {
                workspace_root: "/work/b".into(),
                agent_type: None,
                provider_binding: None,
                idempotency_key: "ws-b".into(),
            })
            .await
            .unwrap();
        assert_ne!(a.session_id, b.session_id);
        assert_eq!(a.workspace_root, "/work/a");
        assert_eq!(b.workspace_root, "/work/b");
        assert_eq!(adapter.registry_len(), 2);
    }
}
