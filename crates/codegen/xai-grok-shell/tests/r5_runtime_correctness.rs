//! Round-5 runtime correctness tests (R5-03 cross-runtime idempotency,
//! R5-04 epoch rotation, R5-05 replay gap + pagination).

use tempfile::TempDir;
use xai_grok_app_server_protocol::{
    SessionStartParams, SubscribeParams, WireCounter,
};
use xai_grok_shell::app_server_runtime::{
    experimental_local_turn_spawn, ShellSessionActorRuntime,
};
use xai_grok_shell::session::info::Info;
use xai_grok_tower::GrokRuntimeFacade;

#[tokio::test]
async fn r5_cross_runtime_concurrent_idempotency_collapses_to_winner() {
    // R5-03: two independent runtimes sharing the same root must collapse
    // concurrent same-key starts to one session id (Won / Existing).
    let temp = TempDir::new().unwrap();
    let root = temp.path().to_path_buf();
    let key = "cross-runtime-idemp";
    let mut handles = Vec::new();
    for i in 0..4 {
        let root = root.clone();
        let key = key.to_string();
        handles.push(tokio::spawn(async move {
            let rt = ShellSessionActorRuntime::with_production_spawn(
                root.clone(),
                experimental_local_turn_spawn(root),
            );
            rt.start_session(SessionStartParams {
                workspace_root: "/work/cross-rt".into(),
                agent_type: None,
                provider_binding: None,
                idempotency_key: key,
            })
            .await
            .map(|s| (i, s.session_id))
        }));
    }
    let mut ids = Vec::new();
    for h in handles {
        let (i, sid) = h.await.unwrap().expect("start ok");
        ids.push((i, sid));
    }
    let first = ids[0].1.clone();
    for (i, sid) in &ids {
        assert_eq!(
            sid, &first,
            "runtime {i} returned different session id under concurrent claim"
        );
    }
}

#[tokio::test]
async fn r5_history_epoch_is_unique_and_rotates() {
    // R5-04: new sessions get unique epochs; rotate invalidates prior cursors.
    let temp = TempDir::new().unwrap();
    let root = temp.path().to_path_buf();
    let rt = ShellSessionActorRuntime::with_production_spawn(
        root.clone(),
        experimental_local_turn_spawn(root),
    );
    let a = rt
        .start_session(SessionStartParams {
            workspace_root: "/work/epoch-a".into(),
            agent_type: None,
            provider_binding: None,
            idempotency_key: "ep-a".into(),
        })
        .await
        .unwrap();
    let b = rt
        .start_session(SessionStartParams {
            workspace_root: "/work/epoch-b".into(),
            agent_type: None,
            provider_binding: None,
            idempotency_key: "ep-b".into(),
        })
        .await
        .unwrap();
    assert!(a.history_epoch.starts_with("epoch_"));
    assert!(b.history_epoch.starts_with("epoch_"));
    assert_ne!(a.history_epoch, b.history_epoch, "sessions must not share epoch");

    let info = Info {
        id: agent_client_protocol::SessionId::new(a.session_id.clone()),
        cwd: "/work/epoch-a".into(),
    };
    let old = a.history_epoch.clone();
    let new_epoch = rt.rotate_history_epoch(&info);
    assert_ne!(old, new_epoch);
    let err = rt
        .replay(SubscribeParams {
            session_id: a.session_id.clone(),
            after_event_seq: WireCounter::new(0),
            history_epoch: Some(old),
        })
        .await
        .unwrap_err();
    assert_eq!(err.code, "epoch_mismatch");
    let ok = rt
        .replay(SubscribeParams {
            session_id: a.session_id.clone(),
            after_event_seq: WireCounter::new(0),
            history_epoch: Some(new_epoch),
        })
        .await
        .unwrap();
    assert!(!ok.events.is_empty());
}

#[tokio::test]
async fn r5_replay_gap_and_exact_pagination() {
    // R5-05: non-projectable physical line between valid events must not drop
    // later events; pagination next_cursor only when more remain.
    let temp = TempDir::new().unwrap();
    let root = temp.path().to_path_buf();
    let rt = ShellSessionActorRuntime::with_production_spawn(
        root.clone(),
        experimental_local_turn_spawn(root.clone()),
    );
    let s = rt
        .start_session(SessionStartParams {
            workspace_root: "/work/gap".into(),
            agent_type: None,
            provider_binding: None,
            idempotency_key: "gap-1".into(),
        })
        .await
        .unwrap();

    // Seed two projectable agent chunks with a corrupt JSONL line between them.
    use agent_client_protocol as acp;
    use xai_grok_shell::session::storage::{JsonlStorageAdapter, SessionUpdate, StorageAdapter};
    let storage = JsonlStorageAdapter::with_root(root);
    let info = Info {
        id: acp::SessionId::new(s.session_id.clone()),
        cwd: "/work/gap".into(),
    };
    let mk = |text: &str| {
        SessionUpdate::Acp(Box::new(acp::SessionNotification::new(
            info.id.clone(),
            acp::SessionUpdate::AgentMessageChunk(acp::ContentChunk::new(acp::ContentBlock::Text(
                acp::TextContent::new(text.to_string()),
            ))),
        )))
    };
    storage.append_update(&info, &mk("before-gap")).await.unwrap();
    // Physical non-projectable line (invalid JSON) creates a seq gap risk if
    // cursor used vector index instead of canonical event_seq.
    let updates_path = storage
        .archived_flag_file(&info)
        .parent()
        .unwrap()
        .join("updates.jsonl");
    {
        use std::io::Write;
        let mut f = std::fs::OpenOptions::new()
            .append(true)
            .open(&updates_path)
            .unwrap();
        writeln!(f, "{{not valid json").unwrap();
    }
    storage.append_update(&info, &mk("after-gap")).await.unwrap();

    let page0 = rt
        .replay(SubscribeParams {
            session_id: s.session_id.clone(),
            after_event_seq: WireCounter::new(0),
            history_epoch: None,
        })
        .await
        .unwrap();
    // Snapshot + before + after (corrupt line skipped, later event kept).
    assert!(
        page0.events.len() >= 3,
        "must retain events after non-projectable gap, got {}",
        page0.events.len()
    );
    // Exact page at end: no next_cursor when all events fit.
    assert!(
        page0.next_cursor.is_none(),
        "full final page must not advertise next_cursor"
    );
    let through = page0.replayed_through.as_u64();
    let page1 = rt
        .replay(SubscribeParams {
            session_id: s.session_id.clone(),
            after_event_seq: WireCounter::new(through),
            history_epoch: None,
        })
        .await
        .unwrap();
    assert!(page1.events.is_empty(), "exclusive cursor past end is empty");
}
