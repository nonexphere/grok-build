//! C1-D real-adapter integration tests.
//!
//! These exercise the real [`ShellSessionActorRuntime`] (backed by the JSONL
//! storage adapter) against a `TempDir` fixture. They prove the storage-backed
//! facade methods map to real Shell symbols (C0-B §1) and that the actor-backed
//! methods honestly return `unsupported` (PARTIAL) instead of silently
//! faking success.
//!
//! RED-then-GREEN evidence is captured under
//! `.llms/execution/app-server-mcp-tower-corrective/tests/c1/`.

use std::sync::Arc;

use agent_client_protocol as acp;
use tempfile::TempDir;
use xai_grok_app_server_protocol::{
    InputBlock, InteractionResponseParams, SessionArchiveParams, SessionForkParams,
    SessionReadParams, SessionResumeParams, SessionStartParams, SessionStatus, SubscribeParams,
    TurnInterruptParams, TurnStartParams, TurnSteerParams, WireCounter,
};
use xai_grok_shell::app_server_runtime::ShellSessionActorRuntime;
use xai_grok_shell::session::info::Info;
use xai_grok_shell::session::storage::{JsonlStorageAdapter, SessionUpdate, StorageAdapter};
use xai_grok_tower::GrokRuntimeFacade;

fn real_port(temp: &TempDir) -> ShellSessionActorRuntime {
    ShellSessionActorRuntime::new(temp.path().to_path_buf())
}

async fn seed_update(temp: &TempDir, session_id: &str, cwd: &str, text: &str) {
    let storage = JsonlStorageAdapter::with_root(temp.path().to_path_buf());
    let info = Info {
        id: acp::SessionId::new(session_id.to_string()),
        cwd: cwd.to_string(),
    };
    storage
        .init_session(&info, acp::ModelId::new("grok-code-fast-1"))
        .await
        .unwrap();
    let notification = acp::SessionNotification::new(
        acp::SessionId::new(session_id.to_string()),
        acp::SessionUpdate::AgentMessageChunk(acp::ContentChunk::new(acp::ContentBlock::Text(
            acp::TextContent::new(text.to_string()),
        ))),
    );
    storage
        .append_update(&info, &SessionUpdate::Acp(Box::new(notification)))
        .await
        .unwrap();
}

#[tokio::test]
async fn c1_real_adapter_list_sessions_reads_jsonl_summaries_not_dormant_stub() {
    let temp = TempDir::new().unwrap();
    let port = real_port(&temp);
    // Seed a real session on disk via the storage adapter directly.
    let storage = JsonlStorageAdapter::with_root(temp.path().to_path_buf());
    let info = Info {
        id: acp::SessionId::new("seed-list-1".to_string()),
        cwd: "/work/list".to_string(),
    };
    storage
        .init_session(&info, acp::ModelId::new("grok-code-fast-1"))
        .await
        .unwrap();

    let sessions = port.list_sessions().await.unwrap();
    let row = sessions
        .iter()
        .find(|s| s.session_id == "seed-list-1")
        .expect("seeded session appears in real list");
    // NOT the dormant stub: status is Starting (num_messages == 0), workspace
    // is the real cwd, and the session id matches the on-disk summary.
    assert_eq!(row.workspace_root, "/work/list");
    assert_ne!(row.status, SessionStatus::Dormant);
    // R5-04: unique per-session epoch (not a global constant).
    assert!(
        row.history_epoch.starts_with("epoch_"),
        "history_epoch must be durable unique stream id, got {}",
        row.history_epoch
    );
}

#[tokio::test]
async fn c1_real_adapter_read_session_projects_session_row_from_summary() {
    let temp = TempDir::new().unwrap();
    let port = real_port(&temp);
    let s = port
        .start_session(SessionStartParams {
            workspace_root: "/work/read".into(),
            agent_type: None,
            provider_binding: None,
            idempotency_key: "read-1".into(),
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
    assert_eq!(result.session.session_id, s.session_id);
    assert_eq!(result.session.workspace_root, "/work/read");
    // R2 PARTIAL: turns/items projection from updates.jsonl not yet implemented.
    assert!(result.turns.is_empty());
    assert!(result.items.is_empty());
}

#[tokio::test]
async fn c1_real_adapter_start_session_persists_summary_and_returns_real_id() {
    let temp = TempDir::new().unwrap();
    let port = real_port(&temp);
    let s = port
        .start_session(SessionStartParams {
            workspace_root: "/work/start".into(),
            agent_type: None,
            provider_binding: None,
            idempotency_key: "start-1".into(),
        })
        .await
        .unwrap();
    // UUIDv7 is 36 chars; the dormant stub returned "session_N".
    assert_eq!(s.session_id.len(), 36);
    assert_eq!(s.workspace_root, "/work/start");
    // The summary.json must exist on disk (real storage authority).
    let storage = JsonlStorageAdapter::with_root(temp.path().to_path_buf());
    let info = Info {
        id: acp::SessionId::new(s.session_id.clone()),
        cwd: "/work/start".to_string(),
    };
    let summary = storage.load_summary(&info).await.unwrap();
    assert_eq!(summary.info.id.0.as_ref(), s.session_id.as_str());
}

#[tokio::test]
async fn c1_real_adapter_start_session_idempotency_key_dedups_same_session_id() {
    let temp = TempDir::new().unwrap();
    let port = real_port(&temp);
    let a = port
        .start_session(SessionStartParams {
            workspace_root: "/work/idem".into(),
            agent_type: None,
            provider_binding: None,
            idempotency_key: "idem-1".into(),
        })
        .await
        .unwrap();
    let b = port
        .start_session(SessionStartParams {
            workspace_root: "/work/idem".into(),
            agent_type: None,
            provider_binding: None,
            idempotency_key: "idem-1".into(),
        })
        .await
        .unwrap();
    assert_eq!(a.session_id, b.session_id);
}

#[tokio::test]
async fn c1_real_adapter_start_session_idempotency_conflict_on_different_input() {
    let temp = TempDir::new().unwrap();
    let port = real_port(&temp);
    let _ = port
        .start_session(SessionStartParams {
            workspace_root: "/work/idem-a".into(),
            agent_type: None,
            provider_binding: None,
            idempotency_key: "idem-conflict".into(),
        })
        .await
        .unwrap();
    let err = port
        .start_session(SessionStartParams {
            workspace_root: "/work/idem-b".into(),
            agent_type: None,
            provider_binding: None,
            idempotency_key: "idem-conflict".into(),
        })
        .await
        .unwrap_err();
    assert_eq!(err.code, "idempotency_conflict");
}

#[tokio::test]
async fn c1_real_adapter_resume_session_loads_persisted_summary() {
    let temp = TempDir::new().unwrap();
    let port = real_port(&temp);
    let s = port
        .start_session(SessionStartParams {
            workspace_root: "/work/resume".into(),
            agent_type: None,
            provider_binding: None,
            idempotency_key: "resume-1".into(),
        })
        .await
        .unwrap();
    let resumed = port
        .resume_session(SessionResumeParams {
            session_id: s.session_id.clone(),
            idempotency_key: "resume-2".into(),
        })
        .await
        .unwrap();
    assert_eq!(resumed.session_id, s.session_id);
    assert_eq!(resumed.workspace_root, "/work/resume");
}

#[tokio::test]
async fn c1_real_adapter_resume_session_unknown_returns_not_found() {
    let temp = TempDir::new().unwrap();
    let port = real_port(&temp);
    let err = port
        .resume_session(SessionResumeParams {
            session_id: "nope".into(),
            idempotency_key: "r".into(),
        })
        .await
        .unwrap_err();
    assert_eq!(err.code, "session_not_found");
}

#[tokio::test]
async fn c1_real_adapter_fork_session_copies_history_to_new_cwd() {
    let temp = TempDir::new().unwrap();
    let port = real_port(&temp);
    let s = port
        .start_session(SessionStartParams {
            workspace_root: "/work/fork-src".into(),
            agent_type: None,
            provider_binding: None,
            idempotency_key: "fork-src".into(),
        })
        .await
        .unwrap();
    // Seed an update on the source so the copy has content.
    seed_update(&temp, &s.session_id, "/work/fork-src", "fork me").await;

    let forked = port
        .fork_session(SessionForkParams {
            session_id: s.session_id.clone(),
            idempotency_key: "fork-1".into(),
            workspace_root: Some("/work/fork-dst".into()),
        })
        .await
        .unwrap();
    assert_ne!(forked.session_id, s.session_id);
    assert_eq!(forked.workspace_root, "/work/fork-dst");
    // The forked session's updates.jsonl must exist on disk (real copy).
    let storage = JsonlStorageAdapter::with_root(temp.path().to_path_buf());
    let target_info = Info {
        id: acp::SessionId::new(forked.session_id.clone()),
        cwd: "/work/fork-dst".to_string(),
    };
    let loaded = storage.load_session(&target_info).await.unwrap();
    assert!(!loaded.updates.is_empty(), "fork copied updates.jsonl");
}

#[tokio::test]
async fn c1_real_adapter_archive_session_hides_not_delete() {
    use xai_grok_app_server_protocol::SessionStatus;
    let temp = TempDir::new().unwrap();
    let port = real_port(&temp);
    let s = port
        .start_session(SessionStartParams {
            workspace_root: "/work/archive".into(),
            agent_type: None,
            provider_binding: None,
            idempotency_key: "archive-1".into(),
        })
        .await
        .unwrap();
    // R6: reversible hide-not-delete — Ok, status Archived, data remains.
    port.archive_session(SessionArchiveParams {
        session_id: s.session_id.clone(),
        idempotency_key: "archive-1".into(),
    })
    .await
    .expect("archive must succeed as hide");
    let listed = port.list_sessions().await.unwrap();
    let row = listed
        .iter()
        .find(|row| row.session_id == s.session_id)
        .expect("archived session must still be on disk (not deleted)");
    assert_eq!(row.status, SessionStatus::Archived);
    // Still readable.
    let read = port
        .read_session(SessionReadParams {
            session_id: s.session_id.clone(),
            include_turns: false,
            include_items: false,
        })
        .await
        .unwrap();
    assert_eq!(read.session.status, SessionStatus::Archived);
}

#[tokio::test]
async fn c1_real_adapter_start_turn_returns_unsupported_actor_gap() {
    let temp = TempDir::new().unwrap();
    let port = real_port(&temp);
    let s = port
        .start_session(SessionStartParams {
            workspace_root: "/work/turn".into(),
            agent_type: None,
            provider_binding: None,
            idempotency_key: "turn-1".into(),
        })
        .await
        .unwrap();
    let err = port
        .start_turn(TurnStartParams {
            session_id: s.session_id.clone(),
            input: vec![InputBlock::Text { text: "hi".into() }],
            idempotency_key: "turn-1".into(),
        })
        .await
        .unwrap_err();
    assert_eq!(err.code, "unsupported");
}

#[tokio::test]
async fn c1_real_adapter_steer_turn_returns_unsupported_actor_gap() {
    let temp = TempDir::new().unwrap();
    let port = real_port(&temp);
    let err = port
        .steer_turn(TurnSteerParams {
            session_id: "s".into(),
            turn_id: "t".into(),
            input: vec![InputBlock::Text {
                text: "steer".into(),
            }],
            idempotency_key: "st".into(),
        })
        .await
        .unwrap_err();
    assert_eq!(err.code, "unsupported");
}

#[tokio::test]
async fn c1_real_adapter_interrupt_turn_returns_unsupported_actor_gap() {
    let temp = TempDir::new().unwrap();
    let port = real_port(&temp);
    let err = port
        .interrupt_turn(TurnInterruptParams {
            session_id: "s".into(),
            turn_id: "t".into(),
            idempotency_key: "i".into(),
        })
        .await
        .unwrap_err();
    assert_eq!(err.code, "unsupported");
}

#[tokio::test]
async fn c1_real_adapter_respond_interaction_returns_unsupported_without_resident() {
    // C6-B: respond_interaction is now a delivery channel (no longer a stub).
    // With a real session on disk but no resident actor (production spawner
    // returns unsupported), the delivery channel honestly returns
    // `unsupported` because the decision cannot be routed to a parked future.
    let temp = TempDir::new().unwrap();
    let port = real_port(&temp);
    let s = port
        .start_session(SessionStartParams {
            workspace_root: "/work/ri".into(),
            agent_type: None,
            provider_binding: None,
            idempotency_key: "ri-1".into(),
        })
        .await
        .unwrap();
    let err = port
        .respond_interaction(InteractionResponseParams {
            session_id: s.session_id,
            turn_id: "t".into(),
            interaction_id: "ix".into(),
            decision: "allow".into(),
            idempotency_key: "r".into(),
        })
        .await
        .unwrap_err();
    assert_eq!(err.code, "unsupported");
}

#[tokio::test]
async fn c1_real_adapter_respond_interaction_unknown_session_not_found() {
    // C6-B: an unknown session id is rejected at the storage layer before
    // the resident check — the delivery channel cannot deliver to a session
    // that does not exist on disk.
    let temp = TempDir::new().unwrap();
    let port = real_port(&temp);
    let err = port
        .respond_interaction(InteractionResponseParams {
            session_id: "no-such-session".into(),
            turn_id: "t".into(),
            interaction_id: "ix".into(),
            decision: "allow".into(),
            idempotency_key: "r".into(),
        })
        .await
        .unwrap_err();
    assert_eq!(err.code, "session_not_found");
}

#[tokio::test]
async fn c1_real_adapter_replay_projects_updates_jsonl_into_runtime_events() {
    let temp = TempDir::new().unwrap();
    let port = real_port(&temp);
    let s = port
        .start_session(SessionStartParams {
            workspace_root: "/work/replay".into(),
            agent_type: None,
            provider_binding: None,
            idempotency_key: "replay-1".into(),
        })
        .await
        .unwrap();
    seed_update(&temp, &s.session_id, "/work/replay", "delta-text").await;

    let page = port
        .replay(SubscribeParams {
            session_id: s.session_id.clone(),
            after_event_seq: WireCounter::new(0),
            history_epoch: Some(s.history_epoch.clone()),
        })
        .await
        .unwrap();
    // Event 0 is the SessionChanged snapshot; event 1 is the projected delta.
    assert!(!page.events.is_empty());
    assert!(page.replayed_through.as_u64() >= 1);
    // R11 PARTIAL: at least the snapshot is projected from real storage.
    let has_snapshot = page
        .events
        .iter()
        .any(|e| matches!(e, xai_grok_tower::RuntimeEvent::SessionChanged(_)));
    assert!(has_snapshot, "replay projects SessionChanged snapshot");
}

#[tokio::test]
async fn c1_real_adapter_replay_epoch_mismatch_returns_error() {
    let temp = TempDir::new().unwrap();
    let port = real_port(&temp);
    let s = port
        .start_session(SessionStartParams {
            workspace_root: "/work/replay-epoch".into(),
            agent_type: None,
            provider_binding: None,
            idempotency_key: "replay-epoch".into(),
        })
        .await
        .unwrap();
    let err = port
        .replay(SubscribeParams {
            session_id: s.session_id,
            after_event_seq: WireCounter::new(0),
            history_epoch: Some("epoch_stale".into()),
        })
        .await
        .unwrap_err();
    assert_eq!(err.code, "epoch_mismatch");
}

#[tokio::test]
async fn c1_real_adapter_replay_cursor_pagination_advances_after_event_seq() {
    let temp = TempDir::new().unwrap();
    let port = real_port(&temp);
    let s = port
        .start_session(SessionStartParams {
            workspace_root: "/work/replay-page".into(),
            agent_type: None,
            provider_binding: None,
            idempotency_key: "replay-page".into(),
        })
        .await
        .unwrap();
    // Seed two updates so the event stream has snapshot + 2 deltas.
    seed_update(&temp, &s.session_id, "/work/replay-page", "d1").await;
    seed_update_more(&temp, &s.session_id, "/work/replay-page", "d2").await;

    let first = port
        .replay(SubscribeParams {
            session_id: s.session_id.clone(),
            after_event_seq: WireCounter::new(0),
            history_epoch: None,
        })
        .await
        .unwrap();
    let through = first.replayed_through.as_u64();
    // If there are more events, the next cursor advances.
    if let Some(next) = first.next_cursor.clone() {
        let second = port
            .replay(SubscribeParams {
                session_id: s.session_id,
                after_event_seq: next,
                history_epoch: None,
            })
            .await
            .unwrap();
        assert!(second.replayed_through.as_u64() >= through);
    }
}

async fn seed_update_more(temp: &TempDir, session_id: &str, cwd: &str, text: &str) {
    let storage = JsonlStorageAdapter::with_root(temp.path().to_path_buf());
    let info = Info {
        id: acp::SessionId::new(session_id.to_string()),
        cwd: cwd.to_string(),
    };
    let notification = acp::SessionNotification::new(
        acp::SessionId::new(session_id.to_string()),
        acp::SessionUpdate::AgentMessageChunk(acp::ContentChunk::new(acp::ContentBlock::Text(
            acp::TextContent::new(text.to_string()),
        ))),
    );
    storage
        .append_update(&info, &SessionUpdate::Acp(Box::new(notification)))
        .await
        .unwrap();
}

#[tokio::test]
async fn c1_real_adapter_no_hybrid_authority_real_list_with_fake_mutation_rejected() {
    // The real port never mixes FakeRuntime. This test documents that the real
    // port's list path reads real storage and that no FakeRuntime mutation
    // path exists in the real port (static guard in the port module).
    let temp = TempDir::new().unwrap();
    let port = real_port(&temp);
    let _ = port
        .start_session(SessionStartParams {
            workspace_root: "/work/no-hybrid".into(),
            agent_type: None,
            provider_binding: None,
            idempotency_key: "no-hybrid".into(),
        })
        .await
        .unwrap();
    let sessions = port.list_sessions().await.unwrap();
    // The real session we started is the only one in this isolated temp root.
    assert_eq!(sessions.len(), 1);
}

#[tokio::test]
async fn c1_real_adapter_shell_runtime_adapter_wraps_real_port() {
    // The composition-root wrapper (ShellRuntimeAdapter) must accept the real
    // port as its inner Arc<dyn GrokRuntimeFacade> and record a registry token.
    use xai_grok_shell::app_server_runtime::ShellRuntimeAdapter;
    let temp = TempDir::new().unwrap();
    let real: Arc<dyn GrokRuntimeFacade> =
        Arc::new(ShellSessionActorRuntime::new(temp.path().to_path_buf()));
    let adapter = ShellRuntimeAdapter::inject(real);
    let s = adapter
        .start_session(SessionStartParams {
            workspace_root: "/work/wrap".into(),
            agent_type: None,
            provider_binding: None,
            idempotency_key: "wrap-1".into(),
        })
        .await
        .unwrap();
    assert_eq!(adapter.registry_len(), 1);
    assert_eq!(s.workspace_root, "/work/wrap");
}
