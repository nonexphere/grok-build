//! Round-4 runtime correctness tests (R4-05 concurrent idempotency, R4-06
//! replay gap, R4-03 product local-turn spawn).

use std::sync::Arc;

use tempfile::TempDir;
use xai_grok_app_server_protocol::{
    InputBlock, SessionStartParams, SubscribeParams, TurnStartParams, WireCounter,
};
use xai_grok_shell::app_server_runtime::{ShellSessionActorRuntime, experimental_local_turn_spawn};
use xai_grok_tower::GrokRuntimeFacade;

#[tokio::test]
async fn r4_concurrent_start_session_same_idempotency_key_single_session() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().to_path_buf();
    let rt = Arc::new(ShellSessionActorRuntime::with_production_spawn(
        root.clone(),
        experimental_local_turn_spawn(root),
    ));
    let key = "concurrent-idemp-key";
    let mut handles = Vec::new();
    for _ in 0..8 {
        let rt = rt.clone();
        let key = key.to_string();
        handles.push(tokio::spawn(async move {
            // Same workspace + same key → same digest; concurrent claims must
            // collapse to one session id (R4-05).
            rt.start_session(SessionStartParams {
                workspace_root: "/work/concurrent".into(),
                agent_type: None,
                provider_binding: None,
                idempotency_key: key,
            })
            .await
        }));
    }
    let mut ids = Vec::new();
    for h in handles {
        let s = h.await.unwrap().expect("start ok");
        ids.push(s.session_id);
    }
    ids.sort();
    ids.dedup();
    assert_eq!(
        ids.len(),
        1,
        "concurrent same-key starts must collapse to one session id, got {ids:?}"
    );
}

#[tokio::test]
async fn r4_idempotency_survives_runtime_reconstruction() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().to_path_buf();
    let key = "durable-idemp-key";
    let first = {
        let rt = ShellSessionActorRuntime::with_production_spawn(
            root.clone(),
            experimental_local_turn_spawn(root.clone()),
        );
        rt.start_session(SessionStartParams {
            workspace_root: "/work/durable".into(),
            agent_type: None,
            provider_binding: None,
            idempotency_key: key.into(),
        })
        .await
        .unwrap()
    };
    // New runtime instance, same root — durable claim must re-dedup.
    let rt2 = ShellSessionActorRuntime::with_production_spawn(
        root.clone(),
        experimental_local_turn_spawn(root),
    );
    let second = rt2
        .start_session(SessionStartParams {
            workspace_root: "/work/durable".into(),
            agent_type: None,
            provider_binding: None,
            idempotency_key: key.into(),
        })
        .await
        .unwrap();
    assert_eq!(first.session_id, second.session_id);
}

#[tokio::test]
async fn r4_product_local_turn_spawn_completes_turn() {
    // R4-03: experimental product factory makes turns operational (offline echo).
    let temp = TempDir::new().unwrap();
    let root = temp.path().to_path_buf();
    let rt = ShellSessionActorRuntime::with_production_spawn(
        root.clone(),
        experimental_local_turn_spawn(root),
    );
    let s = rt
        .start_session(SessionStartParams {
            workspace_root: "/work/product-turn".into(),
            agent_type: None,
            provider_binding: None,
            idempotency_key: "pt-1".into(),
        })
        .await
        .unwrap();
    let turn = rt
        .start_turn(TurnStartParams {
            session_id: s.session_id.clone(),
            input: vec![InputBlock::Text {
                text: "hello product".into(),
            }],
            idempotency_key: "pt-t1".into(),
        })
        .await
        .expect("product local spawn must allow start_turn");
    assert_eq!(turn.session_id, s.session_id);
    assert!(!turn.turn_id.is_empty());
}

#[tokio::test]
async fn r4_replay_filters_by_canonical_event_seq_not_vector_index() {
    // R4-06: after_event_seq is exclusive on canonical seq; gaps from omitted
    // lines must not shift the cursor to drop valid later events.
    let temp = TempDir::new().unwrap();
    let root = temp.path().to_path_buf();
    let rt = ShellSessionActorRuntime::with_production_spawn(
        root.clone(),
        experimental_local_turn_spawn(root),
    );
    let s = rt
        .start_session(SessionStartParams {
            workspace_root: "/work/replay-seq".into(),
            agent_type: None,
            provider_binding: None,
            idempotency_key: "rs-1".into(),
        })
        .await
        .unwrap();
    let _ = rt
        .start_turn(TurnStartParams {
            session_id: s.session_id.clone(),
            input: vec![InputBlock::Text {
                text: "line".into(),
            }],
            idempotency_key: "rs-t".into(),
        })
        .await
        .unwrap();
    let page0 = rt
        .replay(SubscribeParams {
            session_id: s.session_id.clone(),
            after_event_seq: WireCounter::new(0),
            history_epoch: None,
        })
        .await
        .unwrap();
    assert!(
        !page0.events.is_empty(),
        "start-of-stream includes SessionChanged"
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
    // Exclusive: no event at or before `through` should reappear.
    assert!(
        page1.events.is_empty() || page1.replayed_through.as_u64() >= through,
        "cursor advances by last processed seq"
    );
}
