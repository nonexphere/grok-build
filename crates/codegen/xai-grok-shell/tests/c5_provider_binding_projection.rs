//! C5-C `ProviderBinding` projection integration tests.
//!
//! These prove `ShellSessionActorRuntime` projects the identifier-only
//! `ProviderBinding` carried by `SessionStartParams` onto `Session` / `Turn`
//! rows and persists it durably as a sidecar JSON file under the session
//! directory — **without** persisting any secret material (no api_key, no
//! tokens, no auth headers).
//!
//! Contract (handoff C5-C):
//! - `start_session` with a `ProviderBinding` → `read_session` returns the
//!   same identifiers on the `Session` row.
//! - The sidecar (`provider_binding.json`) contains ONLY the structured
//!   identifier fields (`provider_id` / `credential_id` / `model_id` /
//!   `backend` / `binding_revision`); no `api_key` / `token` / `secret`.
//! - `list_sessions` / `resume_session` / `fork_session` / `start_turn` /
//!   `replay` all project the persisted binding.
//! - A session started without a binding continues to project `None`
//!   (no regression on c1/c6/c7).
//!
//! RED-then-GREEN evidence is captured under
//! `.llms/execution/app-server-mcp-tower-corrective/tests/c5/`.

use std::sync::Arc;

use agent_client_protocol as acp;
use async_trait::async_trait;
use tempfile::TempDir;
use tokio::sync::mpsc;
use xai_grok_app_server_protocol::{
    InputBlock, ProviderBinding, SessionReadParams, SessionResumeParams, SessionStartParams,
    SubscribeParams, TurnStartParams, WireCounter,
};
use xai_grok_shell::app_server_runtime::{
    ResidentHandle, SessionSpawner, ShellSessionActorRuntime,
};
use xai_grok_shell::session::commands::{PromptCompletionKind, PromptTurnOk, SessionCommand};
use xai_grok_shell::session::info::Info;
use xai_grok_shell::session::storage::{JsonlStorageAdapter, StorageAdapter};
use xai_grok_tower::{GrokRuntimeFacade, RuntimeEvent};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// A minimal real `cmd_tx` consumer so `start_turn` can route through a real
/// `SessionCommand::Prompt` (mirrors c1_turn_lifecycle). It resolves the
/// prompt as `Completed` and persists an `AgentMessageChunk` to
/// `updates.jsonl` so `read_session` turns are non-empty. NOT `FakeRuntime`.
struct TurnSpawner {
    root: std::path::PathBuf,
}

#[async_trait]
impl SessionSpawner for TurnSpawner {
    async fn spawn(
        &self,
        info: &Info,
        _model_id: &acp::ModelId,
    ) -> Result<ResidentHandle, xai_grok_tower::RuntimeError> {
        let (cmd_tx, mut cmd_rx) = mpsc::unbounded_channel::<SessionCommand>();
        let current_prompt_id = Arc::new(std::sync::Mutex::new(None::<String>));
        let storage = JsonlStorageAdapter::with_root(self.root.clone());
        let info_clone = info.clone();
        let current_clone = current_prompt_id.clone();
        tokio::spawn(async move {
            while let Some(cmd) = cmd_rx.recv().await {
                if let SessionCommand::Prompt {
                    prompt_id,
                    respond_to,
                    ..
                } = cmd
                {
                    if let Ok(mut g) = current_clone.lock() {
                        *g = Some(prompt_id.clone());
                    }
                    // Persist a real UserMessageChunk boundary first so the
                    // shared projector infers a turn (Shell writes no
                    // explicit turn-start event; turn boundaries come from
                    // UserMessageChunk). Then append the agent reply.
                    let user_note = acp::SessionNotification::new(
                        info_clone.id.clone(),
                        acp::SessionUpdate::UserMessageChunk(acp::ContentChunk::new(
                            acp::ContentBlock::Text(acp::TextContent::new(format!(
                                "user-prompt-for-{prompt_id}"
                            ))),
                        )),
                    );
                    let _ = storage
                        .append_update(
                            &info_clone,
                            &xai_grok_shell::session::storage::SessionUpdate::Acp(Box::new(
                                user_note,
                            )),
                        )
                        .await;
                    let agent_note = acp::SessionNotification::new(
                        info_clone.id.clone(),
                        acp::SessionUpdate::AgentMessageChunk(acp::ContentChunk::new(
                            acp::ContentBlock::Text(acp::TextContent::new(format!(
                                "agent-reply-for-{prompt_id}"
                            ))),
                        )),
                    );
                    let _ = storage
                        .append_update(
                            &info_clone,
                            &xai_grok_shell::session::storage::SessionUpdate::Acp(Box::new(
                                agent_note,
                            )),
                        )
                        .await;
                    let _ = respond_to.send(Ok(PromptTurnOk {
                        stop_reason: acp::StopReason::EndTurn,
                        total_tokens: 0,
                        turn_snapshot: None,
                        completion_kind: PromptCompletionKind::Completed,
                        structured_output: None,
                        usage: None,
                    }));
                    if let Ok(mut g) = current_clone.lock() {
                        *g = None;
                    }
                }
            }
        });
        Ok(ResidentHandle {
            cmd_tx,
            current_prompt_id,
            pending_interactions: None,
            delivery_hub: None,
        })
    }
}

fn real_port(temp: &TempDir) -> ShellSessionActorRuntime {
    ShellSessionActorRuntime::new(temp.path().to_path_buf())
}

fn port_with_turn_spawner(temp: &TempDir) -> ShellSessionActorRuntime {
    ShellSessionActorRuntime::with_spawner(
        temp.path().to_path_buf(),
        Arc::new(TurnSpawner {
            root: temp.path().to_path_buf(),
        }),
    )
}

/// A canonical identifier-only binding used across tests. Contains NO secret
/// material — only `provider_id` / `credential_id` / `model_id` / `backend` /
/// `binding_revision`.
fn sample_binding() -> ProviderBinding {
    ProviderBinding {
        provider_id: "openrouter".into(),
        credential_id: "work".into(),
        model_id: "anthropic/claude-3.5-sonnet".into(),
        backend: "chat_completions".into(),
        binding_revision: WireCounter::new(1),
    }
}

/// Start a session with an explicit binding (or `None`).
async fn start_with(
    port: &ShellSessionActorRuntime,
    cwd: &str,
    key: &str,
    binding: Option<ProviderBinding>,
) -> xai_grok_app_server_protocol::Session {
    port.start_session(SessionStartParams {
        workspace_root: cwd.into(),
        agent_type: None,
        provider_binding: binding,
        idempotency_key: key.into(),
    })
    .await
    .unwrap()
}

/// Resolve the on-disk `Info` for a session id (scan summaries).
async fn info_for(_temp: &TempDir, session_id: &str, cwd: &str) -> Info {
    Info {
        id: acp::SessionId::new(session_id.to_string()),
        cwd: cwd.to_string(),
    }
}

// ===========================================================================
// start_session → read_session projects the binding
// ===========================================================================

#[tokio::test]
async fn c5_start_with_binding_read_session_returns_same_identifiers() {
    let temp = TempDir::new().unwrap();
    let port = real_port(&temp);
    let binding = sample_binding();
    let s = start_with(&port, "/work/c5/read", "c5-read", Some(binding.clone())).await;

    assert_eq!(
        s.provider_binding.as_ref().expect("binding projected on start"),
        &binding,
        "start_session returns the same identifier-only binding it was given"
    );

    let result = port
        .read_session(SessionReadParams {
            session_id: s.session_id.clone(),
            include_turns: false,
            include_items: false,
        })
        .await
        .unwrap();
    let read_binding = result
        .session
        .provider_binding
        .expect("read_session projects the persisted binding");
    assert_eq!(read_binding, binding);
    // Identifier-only: every field is a structured identifier, never a secret.
    assert_eq!(read_binding.provider_id, "openrouter");
    assert_eq!(read_binding.credential_id, "work");
    assert_eq!(read_binding.model_id, "anthropic/claude-3.5-sonnet");
    assert_eq!(read_binding.backend, "chat_completions");
    assert_eq!(read_binding.binding_revision, WireCounter::new(1));
}

#[tokio::test]
async fn c5_sidecar_json_contains_no_secret_material() {
    // The durable sidecar (`provider_binding.json`) must contain ONLY the
    // structured identifier fields — never `api_key`, `token`, `secret`,
    // `authorization`, or any credential material.
    let temp = TempDir::new().unwrap();
    let port = real_port(&temp);
    let binding = sample_binding();
    let s = start_with(&port, "/work/c5/sidecar", "c5-sidecar", Some(binding)).await;

    let info = info_for(&temp, &s.session_id, "/work/c5/sidecar").await;
    let storage = JsonlStorageAdapter::with_root(temp.path().to_path_buf());
    let sidecar = storage.provider_binding_file(&info);
    assert!(sidecar.is_file(), "provider_binding.json sidecar exists");
    let bytes = std::fs::read_to_string(&sidecar).unwrap();
    // Structured identifiers present.
    assert!(bytes.contains("\"providerId\""));
    assert!(bytes.contains("\"credentialId\""));
    assert!(bytes.contains("\"modelId\""));
    assert!(bytes.contains("\"backend\""));
    assert!(bytes.contains("\"bindingRevision\""));
    // No secret material — by field name and by value.
    assert!(!bytes.to_lowercase().contains("api_key"));
    assert!(!bytes.to_lowercase().contains("apikey"));
    assert!(!bytes.to_lowercase().contains("token"));
    assert!(!bytes.to_lowercase().contains("secret"));
    assert!(!bytes.to_lowercase().contains("authorization"));
    assert!(!bytes.to_lowercase().contains("bearer"));
    // The structured ProviderBinding deserializes back cleanly.
    let roundtrip: ProviderBinding = serde_json::from_str(&bytes).unwrap();
    assert_eq!(roundtrip, sample_binding());
}

// ===========================================================================
// list_sessions / resume_session / fork_session project the binding
// ===========================================================================

#[tokio::test]
async fn c5_list_sessions_projects_persisted_binding() {
    let temp = TempDir::new().unwrap();
    let port = real_port(&temp);
    let binding = sample_binding();
    let s = start_with(&port, "/work/c5/list", "c5-list", Some(binding.clone())).await;

    let sessions = port.list_sessions().await.unwrap();
    let row = sessions
        .iter()
        .find(|r| r.session_id == s.session_id)
        .expect("session appears in list");
    assert_eq!(
        row.provider_binding.as_ref().expect("binding on list row"),
        &binding,
    );
}

#[tokio::test]
async fn c5_resume_session_projects_persisted_binding() {
    let temp = TempDir::new().unwrap();
    let port = real_port(&temp);
    let binding = sample_binding();
    let s = start_with(&port, "/work/c5/resume", "c5-resume", Some(binding.clone())).await;

    let resumed = port
        .resume_session(SessionResumeParams {
            session_id: s.session_id.clone(),
            idempotency_key: "c5-resume-1".into(),
        })
        .await
        .unwrap();
    assert_eq!(
        resumed.provider_binding.as_ref().expect("binding on resume"),
        &binding,
    );
}

#[tokio::test]
async fn c5_fork_session_inherits_parent_binding() {
    let temp = TempDir::new().unwrap();
    let port = real_port(&temp);
    let binding = sample_binding();
    let s = start_with(&port, "/work/c5/fork", "c5-fork", Some(binding.clone())).await;

    let forked = port
        .fork_session(xai_grok_app_server_protocol::SessionForkParams {
            session_id: s.session_id.clone(),
            idempotency_key: "c5-fork-1".into(),
            workspace_root: None,
        })
        .await
        .unwrap();
    // The forked session inherits the parent's identifier-only binding via
    // the copied sidecar.
    assert_eq!(
        forked
            .provider_binding
            .as_ref()
            .expect("fork inherits parent binding"),
        &binding,
    );
    // The fork's sidecar file exists on disk (durable, not just in-memory).
    let fork_info = info_for(&temp, &forked.session_id, "/work/c5/fork").await;
    let storage = JsonlStorageAdapter::with_root(temp.path().to_path_buf());
    assert!(storage.provider_binding_file(&fork_info).is_file());
}

// ===========================================================================
// start_turn + read_session turns project the binding
// ===========================================================================

#[tokio::test]
async fn c5_start_turn_projects_session_binding_on_turn() {
    let temp = TempDir::new().unwrap();
    let port = port_with_turn_spawner(&temp);
    let binding = sample_binding();
    let s = start_with(&port, "/work/c5/turn", "c5-turn", Some(binding.clone())).await;

    let turn = port
        .start_turn(TurnStartParams {
            session_id: s.session_id.clone(),
            input: vec![InputBlock::Text {
                text: "hello".into(),
            }],
            idempotency_key: "c5-turn-1".into(),
        })
        .await
        .expect("start_turn routes through real cmd_tx");
    assert_eq!(
        turn.provider_binding.as_ref().expect("binding on turn"),
        &binding,
    );
}

#[tokio::test]
async fn c5_read_session_turns_carry_session_binding() {
    let temp = TempDir::new().unwrap();
    let port = port_with_turn_spawner(&temp);
    let binding = sample_binding();
    let s = start_with(&port, "/work/c5/turns", "c5-turns", Some(binding.clone())).await;

    // Drive a real turn so `updates.jsonl` has a UserMessageChunk boundary.
    port.start_turn(TurnStartParams {
        session_id: s.session_id.clone(),
        input: vec![InputBlock::Text {
            text: "first prompt".into(),
        }],
        idempotency_key: "c5-turns-1".into(),
    })
    .await
    .unwrap();

    let result = port
        .read_session(SessionReadParams {
            session_id: s.session_id.clone(),
            include_turns: true,
            include_items: true,
        })
        .await
        .unwrap();
    assert!(
        !result.turns.is_empty(),
        "turn projection is non-empty after a real turn"
    );
    for turn in &result.turns {
        assert_eq!(
            turn.provider_binding.as_ref().expect("binding on every turn"),
            &binding,
            "every inferred turn carries the session's identifier-only binding"
        );
    }
}

// ===========================================================================
// replay SessionChanged snapshot projects the binding
// ===========================================================================

#[tokio::test]
async fn c5_replay_session_changed_snapshot_projects_binding() {
    let temp = TempDir::new().unwrap();
    let port = real_port(&temp);
    let binding = sample_binding();
    let s = start_with(&port, "/work/c5/replay", "c5-replay", Some(binding.clone())).await;

    let page = port
        .replay(SubscribeParams {
            session_id: s.session_id.clone(),
            after_event_seq: WireCounter::new(0),
            history_epoch: None,
        })
        .await
        .unwrap();
    let snapshot = page
        .events
        .iter()
        .find_map(|e| match e {
            RuntimeEvent::SessionChanged(session) => Some(session.clone()),
            _ => None,
        })
        .expect("SessionChanged snapshot is event 0");
    assert_eq!(
        snapshot
            .provider_binding
            .as_ref()
            .expect("binding on replay snapshot"),
        &binding,
    );
}

// ===========================================================================
// Idempotent re-start projects the persisted binding
// ===========================================================================

#[tokio::test]
async fn c5_idempotent_restart_projects_persisted_binding() {
    let temp = TempDir::new().unwrap();
    let port = real_port(&temp);
    let binding = sample_binding();
    let s = start_with(&port, "/work/c5/idem", "c5-idem", Some(binding.clone())).await;

    // Same idempotency key + same digest → same session, with the binding
    // re-projected from the persisted sidecar (no re-write).
    let s2 = start_with(&port, "/work/c5/idem", "c5-idem", Some(binding.clone())).await;
    assert_eq!(s.session_id, s2.session_id);
    assert_eq!(
        s2.provider_binding.as_ref().expect("binding on idempotent restart"),
        &binding,
    );
}

// ===========================================================================
// No regression: a session started without a binding projects None
// ===========================================================================

#[tokio::test]
async fn c5_session_without_binding_projects_none_everywhere() {
    let temp = TempDir::new().unwrap();
    let port = real_port(&temp);
    let s = start_with(&port, "/work/c5/none", "c5-none", None).await;

    assert!(
        s.provider_binding.is_none(),
        "no binding on start when none given"
    );
    let result = port
        .read_session(SessionReadParams {
            session_id: s.session_id.clone(),
            include_turns: true,
            include_items: true,
        })
        .await
        .unwrap();
    assert!(
        result.session.provider_binding.is_none(),
        "no binding on read_session when none persisted"
    );
    for turn in &result.turns {
        assert!(
            turn.provider_binding.is_none(),
            "no binding on inferred turns when none persisted"
        );
    }
    // No sidecar file is created when no binding was given.
    let info = info_for(&temp, &s.session_id, "/work/c5/none").await;
    let storage = JsonlStorageAdapter::with_root(temp.path().to_path_buf());
    assert!(
        !storage.provider_binding_file(&info).exists(),
        "no sidecar file when no binding was given"
    );
}
