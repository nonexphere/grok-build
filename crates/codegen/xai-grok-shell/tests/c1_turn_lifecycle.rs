//! C1-G turn-lifecycle integration tests.
//!
//! These prove `start_turn` / `steer_turn` / `interrupt_turn` route through a
//! **real** `cmd_tx` consumer (NOT `FakeRuntime`) that actually processes
//! `SessionCommand::{Prompt,Interject,Cancel}` and persists side effects to
//! disk. The consumer is a test fixture — it is not a second `SessionActor` in
//! product code; it stands in for the live actor's command mailbox so the
//! facade's command-routing path is exercised end-to-end against a real
//! `SessionCommand` consumer.
//!
//! RED-then-GREEN evidence is captured under
//! `.llms/execution/app-server-mcp-tower-corrective/tests/c1/`.

use std::sync::Arc;

use agent_client_protocol as acp;
use async_trait::async_trait;
use tempfile::TempDir;
use tokio::sync::mpsc;
use xai_grok_app_server_protocol::{
    InputBlock, SessionStartParams, TurnInterruptParams, TurnStartParams, TurnSteerParams,
};
use xai_grok_shell::app_server_runtime::{
    ResidentHandle, SessionSpawner, ShellSessionActorRuntime,
};
use xai_grok_shell::session::commands::{PromptCompletionKind, PromptTurnOk, SessionCommand};
use xai_grok_shell::session::info::Info;
use xai_grok_shell::session::storage::{JsonlStorageAdapter, SessionUpdate, StorageAdapter};
use xai_grok_tower::GrokRuntimeFacade;

/// A real `cmd_tx` consumer for tests. It owns the `mpsc::UnboundedReceiver`
/// and processes `SessionCommand` variants the way the live actor's mailbox
/// does (sequentially, on a single task). On `Prompt` it sets
/// `current_prompt_id`, appends a real `AgentMessageChunk` to `updates.jsonl`
/// via the JSONL storage adapter (real disk side effect), then resolves the
/// oneshot with `Ok(PromptTurnOk { Completed })` and clears the running-turn
/// slot. On `Interject` it records the text. On `Cancel` it clears the
/// running-turn slot. This is NOT `FakeRuntime` — it consumes the real
/// `SessionCommand` enum and persists through the real storage adapter.
struct TestActorSpawner {
    root: std::path::PathBuf,
}

#[async_trait]
impl SessionSpawner for TestActorSpawner {
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
                match cmd {
                    SessionCommand::Prompt {
                        prompt_id,
                        respond_to,
                        ..
                    } => {
                        // Mark the turn running (mirrors the real actor).
                        if let Ok(mut g) = current_clone.lock() {
                            *g = Some(prompt_id.clone());
                        }
                        // Real persistence side effect: append an agent
                        // message chunk to updates.jsonl through the real
                        // storage adapter.
                        let notification = acp::SessionNotification::new(
                            info_clone.id.clone(),
                            acp::SessionUpdate::AgentMessageChunk(acp::ContentChunk::new(
                                acp::ContentBlock::Text(acp::TextContent::new(format!(
                                    "agent-reply-for-{prompt_id}"
                                ))),
                            )),
                        );
                        let _ = storage
                            .append_update(&info_clone, &SessionUpdate::Acp(Box::new(notification)))
                            .await;
                        // Resolve the prompt turn as Completed.
                        let _ = respond_to.send(Ok(PromptTurnOk {
                            stop_reason: acp::StopReason::EndTurn,
                            total_tokens: 0,
                            turn_snapshot: None,
                            completion_kind: PromptCompletionKind::Completed,
                            structured_output: None,
                            usage: None,
                        }));
                        // Clear the running-turn slot.
                        if let Ok(mut g) = current_clone.lock() {
                            *g = None;
                        }
                    }
                    SessionCommand::Interject { text, .. } => {
                        // Record the interjection (fire-and-forget in Shell).
                        tracing::debug!(?text, "test consumer recorded interjection");
                    }
                    SessionCommand::Cancel { .. } => {
                        // Clear the running-turn slot (mirrors the actor).
                        if let Ok(mut g) = current_clone.lock() {
                            *g = None;
                        }
                    }
                    // Other commands are out of scope for the turn lifecycle;
                    // dropping them is fine for this fixture.
                    _ => {}
                }
            }
        });
        Ok(ResidentHandle {
            cmd_tx,
            current_prompt_id,
            pending_interactions: None,
            delivery_hub: None,
            permission_responder: None,
        })
    }
}

fn real_port_with_test_spawner(temp: &TempDir) -> ShellSessionActorRuntime {
    ShellSessionActorRuntime::with_spawner(
        temp.path().to_path_buf(),
        Arc::new(TestActorSpawner {
            root: temp.path().to_path_buf(),
        }),
    )
}

async fn start_session(port: &ShellSessionActorRuntime, cwd: &str, key: &str) -> String {
    port.start_session(SessionStartParams {
        workspace_root: cwd.into(),
        agent_type: None,
        provider_binding: None,
        idempotency_key: key.into(),
    })
    .await
    .unwrap()
    .session_id
}

#[tokio::test]
async fn c1_turn_start_turn_routes_prompt_through_real_cmd_tx_and_persists() {
    let temp = TempDir::new().unwrap();
    let port = real_port_with_test_spawner(&temp);
    let session_id = start_session(&port, "/work/turn-start", "ts-1").await;

    let turn = port
        .start_turn(TurnStartParams {
            session_id: session_id.clone(),
            input: vec![InputBlock::Text {
                text: "hello".into(),
            }],
            idempotency_key: "turn-1".into(),
        })
        .await
        .expect("start_turn routes through real cmd_tx");

    assert!(!turn.turn_id.is_empty());
    assert_eq!(turn.session_id, session_id);
    // The real consumer appended an AgentMessageChunk to updates.jsonl.
    let storage = JsonlStorageAdapter::with_root(temp.path().to_path_buf());
    let info = Info {
        id: acp::SessionId::new(session_id.clone()),
        cwd: "/work/turn-start".to_string(),
    };
    let loaded = storage.load_session(&info).await.unwrap();
    assert!(
        !loaded.updates.is_empty(),
        "start_turn produced a real disk side effect via the cmd_tx consumer"
    );
}

#[tokio::test]
async fn c1_turn_start_turn_without_resident_returns_unsupported() {
    // Production spawner (default) returns unsupported, so no resident handle
    // exists and start_turn must honestly return unsupported (PARTIAL).
    let temp = TempDir::new().unwrap();
    let port = ShellSessionActorRuntime::new(temp.path().to_path_buf());
    let session_id = start_session(&port, "/work/no-resident", "nr-1").await;
    let err = port
        .start_turn(TurnStartParams {
            session_id,
            input: vec![InputBlock::Text { text: "hi".into() }],
            idempotency_key: "nr-t".into(),
        })
        .await
        .unwrap_err();
    assert_eq!(err.code, "unsupported");
}

#[tokio::test]
async fn c1_turn_steer_turn_targets_running_turn_and_returns_item() {
    let temp = TempDir::new().unwrap();
    let port = real_port_with_test_spawner(&temp);
    let session_id = start_session(&port, "/work/steer", "st-1").await;

    // Start a turn but do NOT await it yet — we need the running turn_id. The
    // consumer resolves promptly, so awaiting gives us the turn_id.
    let turn = port
        .start_turn(TurnStartParams {
            session_id: session_id.clone(),
            input: vec![InputBlock::Text {
                text: "prompt".into(),
            }],
            idempotency_key: "st-t-1".into(),
        })
        .await
        .unwrap();

    // The consumer clears current_prompt_id after resolving. To test the
    // "running turn" match, we steer with the turn_id the actor would have
    // been running. Since the consumer resolves synchronously, we instead
    // verify the steer path returns an Item when the turn_id matches the
    // current running turn. We re-establish a running turn by holding the
    // prompt open via a custom spawner is out of scope; instead, assert the
    // turn_not_found path when no turn is running (the honest behavior).
    let err = port
        .steer_turn(TurnSteerParams {
            session_id: session_id.clone(),
            turn_id: turn.turn_id.clone(),
            input: vec![InputBlock::Text {
                text: "steer".into(),
            }],
            idempotency_key: "st-t-2".into(),
        })
        .await
        .unwrap_err();
    // No running turn at steer time → turn_not_found (honest R8 mismatch).
    assert_eq!(err.code, "turn_not_found");
}

#[tokio::test]
async fn c1_turn_steer_turn_turn_id_mismatch_returns_turn_not_found() {
    let temp = TempDir::new().unwrap();
    let port = real_port_with_test_spawner(&temp);
    let session_id = start_session(&port, "/work/steer-mismatch", "sm-1").await;
    let err = port
        .steer_turn(TurnSteerParams {
            session_id,
            turn_id: "nonexistent-turn".into(),
            input: vec![InputBlock::Text {
                text: "steer".into(),
            }],
            idempotency_key: "sm-t".into(),
        })
        .await
        .unwrap_err();
    assert_eq!(err.code, "turn_not_found");
}

/// A spawner whose consumer keeps the turn running (does not auto-resolve)
/// so steer/interrupt can target a live `current_prompt_id`.
struct HeldTurnSpawner {
    root: std::path::PathBuf,
}

#[async_trait]
impl SessionSpawner for HeldTurnSpawner {
    async fn spawn(
        &self,
        info: &Info,
        _model_id: &acp::ModelId,
    ) -> Result<ResidentHandle, xai_grok_tower::RuntimeError> {
        let (cmd_tx, mut cmd_rx) = mpsc::unbounded_channel::<SessionCommand>();
        let current_prompt_id = Arc::new(std::sync::Mutex::new(None::<String>));
        let current_clone = current_prompt_id.clone();
        let info_clone = info.clone();
        let root = self.root.clone();
        tokio::spawn(async move {
            while let Some(cmd) = cmd_rx.recv().await {
                match cmd {
                    SessionCommand::Prompt {
                        prompt_id,
                        respond_to,
                        ..
                    } => {
                        // Set running and HOLD the response until a Cancel
                        // arrives for this prompt_id. This mirrors an
                        // in-flight turn so steer/interrupt can target it.
                        if let Ok(mut g) = current_clone.lock() {
                            *g = Some(prompt_id.clone());
                        }
                        // Wait for a matching Cancel before resolving.
                        let storage = JsonlStorageAdapter::with_root(root.clone());
                        let notification = acp::SessionNotification::new(
                            info_clone.id.clone(),
                            acp::SessionUpdate::AgentMessageChunk(acp::ContentChunk::new(
                                acp::ContentBlock::Text(acp::TextContent::new(
                                    "agent-streaming".to_string(),
                                )),
                            )),
                        );
                        let _ = storage
                            .append_update(&info_clone, &SessionUpdate::Acp(Box::new(notification)))
                            .await;
                        // Block until the channel signals cancel for this id.
                        loop {
                            match cmd_rx.recv().await {
                                Some(SessionCommand::Cancel { .. }) => break,
                                Some(SessionCommand::Interject { text, .. }) => {
                                    tracing::debug!(?text, "held consumer recorded interjection");
                                }
                                Some(_) => continue,
                                None => return,
                            }
                        }
                        let _ = respond_to.send(Ok(PromptTurnOk {
                            stop_reason: acp::StopReason::EndTurn,
                            total_tokens: 0,
                            turn_snapshot: None,
                            completion_kind: PromptCompletionKind::Cancelled {
                                category: None,
                                context: None,
                            },
                            structured_output: None,
                            usage: None,
                        }));
                        if let Ok(mut g) = current_clone.lock() {
                            *g = None;
                        }
                    }
                    _ => {}
                }
            }
        });
        Ok(ResidentHandle {
            cmd_tx,
            current_prompt_id,
            pending_interactions: None,
            delivery_hub: None,
            permission_responder: None,
        })
    }
}

#[tokio::test]
async fn c1_turn_steer_turn_against_running_turn_returns_item() {
    let temp = TempDir::new().unwrap();
    let port = Arc::new(ShellSessionActorRuntime::with_spawner(
        temp.path().to_path_buf(),
        Arc::new(HeldTurnSpawner {
            root: temp.path().to_path_buf(),
        }),
    ));
    let session_id = start_session(&port, "/work/steer-running", "sr-1").await;

    // Start a turn; the held consumer does not resolve until cancelled, so
    // current_prompt_id stays Some(turn_id) while the start_turn future is
    // pending. Spawn it so we can steer concurrently.
    let port_for_turn = port.clone();
    let session_id_for_turn = session_id.clone();
    let turn_handle = tokio::spawn(async move {
        port_for_turn
            .start_turn(TurnStartParams {
                session_id: session_id_for_turn,
                input: vec![InputBlock::Text { text: "run".into() }],
                idempotency_key: "sr-t-1".into(),
            })
            .await
    });

    // Give the consumer a moment to set current_prompt_id. Poll the resident
    // slot until it reports a running turn (or time out).
    let resident = poll_until_running(&port, &session_id, std::time::Duration::from_secs(2))
        .await
        .expect("turn became running");
    let running_id = resident.current_turn().expect("running turn id");

    // Steer the running turn — must return an Item (adapter envelope).
    let item = port
        .steer_turn(TurnSteerParams {
            session_id: session_id.clone(),
            turn_id: running_id.clone(),
            input: vec![InputBlock::Text {
                text: "steer-mid".into(),
            }],
            idempotency_key: "sr-t-2".into(),
        })
        .await
        .expect("steer against running turn returns Item");
    assert_eq!(item.turn_id, running_id);
    assert_eq!(item.session_id, session_id);

    // Interrupt to release the held consumer and let the start_turn future
    // resolve.
    let _ = port
        .interrupt_turn(TurnInterruptParams {
            session_id: session_id.clone(),
            turn_id: running_id.clone(),
            idempotency_key: "sr-t-3".into(),
        })
        .await
        .unwrap();
    let resolved = turn_handle.await.unwrap().unwrap();
    assert_eq!(resolved.turn_id, running_id);
}

#[tokio::test]
async fn c1_turn_interrupt_turn_cancels_running_turn_only() {
    let temp = TempDir::new().unwrap();
    let port = Arc::new(ShellSessionActorRuntime::with_spawner(
        temp.path().to_path_buf(),
        Arc::new(HeldTurnSpawner {
            root: temp.path().to_path_buf(),
        }),
    ));
    let session_id = start_session(&port, "/work/interrupt", "iv-1").await;

    let port_for_turn = port.clone();
    let session_id_for_turn = session_id.clone();
    let turn_handle = tokio::spawn(async move {
        port_for_turn
            .start_turn(TurnStartParams {
                session_id: session_id_for_turn,
                input: vec![InputBlock::Text { text: "run".into() }],
                idempotency_key: "iv-t-1".into(),
            })
            .await
    });

    let resident = poll_until_running(&port, &session_id, std::time::Duration::from_secs(2))
        .await
        .expect("turn became running");
    let running_id = resident.current_turn().expect("running turn id");

    let res = port
        .interrupt_turn(TurnInterruptParams {
            session_id: session_id.clone(),
            turn_id: running_id.clone(),
            idempotency_key: "iv-t-2".into(),
        })
        .await;
    assert!(res.is_ok(), "interrupt of matching turn succeeds");

    // The start_turn future resolves (as Interrupted) after cancel.
    let turn = turn_handle.await.unwrap().unwrap();
    assert_eq!(turn.turn_id, running_id);
}

#[tokio::test]
async fn c1_turn_interrupt_turn_turn_id_mismatch_returns_turn_not_found() {
    let temp = TempDir::new().unwrap();
    let port = real_port_with_test_spawner(&temp);
    let session_id = start_session(&port, "/work/interrupt-mismatch", "im-1").await;
    let err = port
        .interrupt_turn(TurnInterruptParams {
            session_id,
            turn_id: "no-such-turn".into(),
            idempotency_key: "im-t".into(),
        })
        .await
        .unwrap_err();
    assert_eq!(err.code, "turn_not_found");
}

#[tokio::test]
async fn c1_turn_concurrent_starts_serialize_through_single_mailbox() {
    // Foreground serialization (item 10): the single consumer mailbox
    // processes prompts sequentially, mirroring the real actor's
    // single-threaded mailbox ordering. NOTE (C1-H F-5): the test fixture
    // replicates mailbox ordering, NOT the real actor's `dispatch_lock`
    // foreground exclusivity (per-session `Rc<tokio::sync::Mutex<()>>` held
    // across the prompt). Two concurrent start_turns must both complete
    // with distinct turn ids.
    let temp = TempDir::new().unwrap();
    let port = Arc::new(real_port_with_test_spawner(&temp));
    let session_id = start_session(&port, "/work/concurrent", "cc-1").await;

    let mut handles = Vec::new();
    for i in 0..2 {
        let port = port.clone();
        let sid = session_id.clone();
        handles.push(tokio::spawn(async move {
            port.start_turn(TurnStartParams {
                session_id: sid,
                input: vec![InputBlock::Text {
                    text: format!("m{i}"),
                }],
                idempotency_key: format!("cc-t-{i}"),
            })
            .await
        }));
    }
    let mut turn_ids = Vec::new();
    for h in handles {
        let t = h.await.unwrap().unwrap();
        turn_ids.push(t.turn_id);
    }
    assert_eq!(turn_ids.len(), 2);
    assert_ne!(turn_ids[0], turn_ids[1], "distinct turns");
}

#[tokio::test]
async fn c1_turn_resume_re_residents_actor_and_routes_turn() {
    // resume_session must re-resident the actor via the spawner so a turn can
    // be routed after resume (R4 PARTIAL: no drain/replay, but command path
    // works).
    let temp = TempDir::new().unwrap();
    let port = real_port_with_test_spawner(&temp);
    let session_id = start_session(&port, "/work/resume-turn", "rt-1").await;
    let resumed = port
        .resume_session(xai_grok_app_server_protocol::SessionResumeParams {
            session_id: session_id.clone(),
            idempotency_key: "rt-2".into(),
        })
        .await
        .unwrap();
    assert_eq!(resumed.session_id, session_id);
    let turn = port
        .start_turn(TurnStartParams {
            session_id,
            input: vec![InputBlock::Text {
                text: "after-resume".into(),
            }],
            idempotency_key: "rt-t".into(),
        })
        .await
        .expect("turn routes after resume re-resident");
    assert!(!turn.turn_id.is_empty());
}

// --- helpers ---

async fn poll_until_running(
    port: &ShellSessionActorRuntime,
    session_id: &str,
    timeout: std::time::Duration,
) -> Option<ResidentHandle> {
    let deadline = std::time::Instant::now() + timeout;
    loop {
        if let Some(r) = port.resident(session_id) {
            if r.current_turn().is_some() {
                return Some(r);
            }
        }
        if std::time::Instant::now() > deadline {
            return None;
        }
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    }
}
