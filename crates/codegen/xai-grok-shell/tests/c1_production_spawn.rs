//! C1-J production-spawn seam + Medium finding fixes (F-1..F-5).
//!
//! These prove:
//! - The production spawn seam (`ProductionSpawner::with_real_spawn` /
//!   `ShellSessionActorRuntime::with_production_spawn`) routes a real
//!   resident `SessionHandle` when a real spawn function is injected, and
//!   returns `unsupported` enumerating the exact missing dependencies when
//!   none is injected (C1-G residual close).
//! - F-1: `steer_turn` synthesized `Item.event_seq` is a per-session
//!   monotonic sequence, not a wall-clock timestamp.
//! - F-2: `next_ordinal` seeds from `Summary.num_messages` on resume so
//!   ordinals do not collide across process restarts.
//! - F-3: when the actor mailbox is gone, `steer_turn`/`interrupt_turn`/
//!   `start_turn` return `session_closed` and clear the stale
//!   `current_prompt_id` so the turn-id guard stays honest.
//! - F-4: concurrent `ensure_resident` for the same session does not
//!   double-spawn (per-session async lock; the spawner is invoked once).
//!
//! RED-then-GREEN evidence is captured under
//! `.llms/execution/app-server-mcp-tower-corrective/tests/c1/`.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use agent_client_protocol as acp;
use async_trait::async_trait;
use tempfile::TempDir;
use tokio::sync::mpsc;
use xai_grok_app_server_protocol::{
    InputBlock, SessionResumeParams, SessionStartParams, TurnInterruptParams, TurnStartParams,
    TurnSteerParams,
};
use xai_grok_shell::app_server_runtime::{
    RealSpawnFn, ResidentHandle, SessionSpawner, ShellSessionActorRuntime,
};
use xai_grok_shell::session::commands::{PromptCompletionKind, PromptTurnOk, SessionCommand};
use xai_grok_shell::session::info::Info;
use xai_grok_shell::session::storage::{JsonlStorageAdapter, SessionUpdate, StorageAdapter};
use xai_grok_tower::GrokRuntimeFacade;

/// A real `cmd_tx` consumer that counts how many times `spawn` was invoked,
/// so the F-4 double-spawn test can assert exactly-once spawning.
struct CountingActorSpawner {
    root: std::path::PathBuf,
    spawn_calls: Arc<AtomicUsize>,
}

#[async_trait]
impl SessionSpawner for CountingActorSpawner {
    async fn spawn(
        &self,
        info: &Info,
        _model_id: &acp::ModelId,
    ) -> Result<ResidentHandle, xai_grok_tower::RuntimeError> {
        self.spawn_calls.fetch_add(1, Ordering::SeqCst);
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
                        if let Ok(mut g) = current_clone.lock() {
                            *g = Some(prompt_id.clone());
                        }
                        let notification = acp::SessionNotification::new(
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
                                &SessionUpdate::Acp(Box::new(notification)),
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
                    SessionCommand::Interject { text, .. } => {
                        tracing::debug!(?text, "counting consumer recorded interjection");
                    }
                    SessionCommand::Cancel { .. } => {
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
        })
    }
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

/// Build a `RealSpawnFn` closure backed by `CountingActorSpawner`, returning
/// the spawn-call counter alongside. This exercises the production seam
/// (`ProductionSpawner::with_real_spawn`) with a real offline `cmd_tx`
/// consumer — NOT `FakeRuntime`.
fn real_spawn_fn(
    root: std::path::PathBuf,
) -> (RealSpawnFn, Arc<AtomicUsize>) {
    let spawn_calls = Arc::new(AtomicUsize::new(0));
    let counter = spawn_calls.clone();
    let root_for_closure = root.clone();
    let closure: RealSpawnFn = Arc::new(move |info: Info, _model_id: acp::ModelId| {
        let counter = spawn_calls.clone();
        let root = root_for_closure.clone();
        Box::pin(async move {
            let spawner = CountingActorSpawner { root, spawn_calls: counter };
            // Delegate to the real `SessionSpawner` impl so the closure
            // exercises the same real `cmd_tx` consumer path the trait does.
            SessionSpawner::spawn(&spawner, &info, &_model_id).await
        })
    });
    (closure, counter)
}

#[tokio::test]
async fn c1_prod_spawn_seam_routes_real_resident_when_spawn_fn_injected() {
    // The production seam: with_production_spawn injects a RealSpawnFn. A
    // real offline cmd_tx consumer (NOT FakeRuntime) is wired through
    // ProductionSpawner::with_real_spawn. start_session -> start_turn must
    // obtain a real resident SessionHandle and route the prompt.
    let temp = TempDir::new().unwrap();
    let (real, _counter) = real_spawn_fn(temp.path().to_path_buf());
    let port = ShellSessionActorRuntime::with_production_spawn(
        temp.path().to_path_buf(),
        real,
    );
    let session_id = start_session(&port, "/work/prod-seam", "ps-1").await;

    let turn = port
        .start_turn(TurnStartParams {
            session_id: session_id.clone(),
            input: vec![InputBlock::Text {
                text: "hello".into(),
            }],
            idempotency_key: "ps-t-1".into(),
        })
        .await
        .expect("production seam routes a real resident handle");

    assert!(!turn.turn_id.is_empty());
    assert_eq!(turn.session_id, session_id);
    // Real disk side effect through the real cmd_tx consumer.
    let storage = JsonlStorageAdapter::with_root(temp.path().to_path_buf());
    let info = Info {
        id: acp::SessionId::new(session_id.clone()),
        cwd: "/work/prod-seam".to_string(),
    };
    let loaded = storage.load_session(&info).await.unwrap();
    assert!(
        !loaded.updates.is_empty(),
        "production seam produced a real disk side effect via the cmd_tx consumer"
    );
}

#[tokio::test]
async fn c1_prod_spawn_seam_without_spawn_fn_returns_unsupported_with_missing_deps() {
    // The default ProductionSpawner (no real spawn fn) must return
    // `unsupported` and the message must enumerate the exact missing
    // production dependencies so the BLOCKER is honest.
    let temp = TempDir::new().unwrap();
    let port = ShellSessionActorRuntime::new(temp.path().to_path_buf());
    let session_id = start_session(&port, "/work/no-spawn-fn", "ns-1").await;
    let err = port
        .start_turn(TurnStartParams {
            session_id,
            input: vec![InputBlock::Text {
                text: "hi".into(),
            }],
            idempotency_key: "ns-t".into(),
        })
        .await
        .unwrap_err();
    assert_eq!(err.code, "unsupported");
    // The message must name the concrete missing dependencies so the
    // BLOCKER is actionable (not a vague "needs creds").
    assert!(err.message.contains("credentials"), "msg: {}", err.message);
    assert!(err.message.contains("AgentDefinition"), "msg: {}", err.message);
    assert!(err.message.contains("spawn_session_on_thread"), "msg: {}", err.message);
    assert!(err.message.contains("C2-A"), "msg: {}", err.message);
}

#[tokio::test]
async fn c1_prod_spawn_seam_resume_re_residents_and_routes_turn() {
    // resume_session must re-resident via the production seam so a turn
    // routes after resume.
    let temp = TempDir::new().unwrap();
    let (real, _counter) = real_spawn_fn(temp.path().to_path_buf());
    let port = ShellSessionActorRuntime::with_production_spawn(
        temp.path().to_path_buf(),
        real,
    );
    let session_id = start_session(&port, "/work/prod-resume", "pr-1").await;
    let resumed = port
        .resume_session(SessionResumeParams {
            session_id: session_id.clone(),
            idempotency_key: "pr-2".into(),
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
            idempotency_key: "pr-t".into(),
        })
        .await
        .expect("turn routes after resume via production seam");
    assert!(!turn.turn_id.is_empty());
}

#[tokio::test]
async fn c1_f1_steer_turn_event_seq_is_monotonic_not_wall_clock() {
    // F-1 fix: steer_turn synthesized Item.event_seq must be a per-session
    // monotonic sequence (1, 2, 3, ...), not a wall-clock timestamp. We
    // cannot steer against a resolved turn (current_prompt_id cleared), so
    // we use a held-turn spawner to keep a turn running and steer it
    // multiple times, asserting event_seq strictly increases.
    let temp = TempDir::new().unwrap();
    let port = Arc::new(ShellSessionActorRuntime::with_spawner(
        temp.path().to_path_buf(),
        Arc::new(HeldTurnSpawner {
            root: temp.path().to_path_buf(),
        }),
    ));
    let session_id = start_session(&port, "/work/f1-monotonic", "f1-1").await;

    let port_for_turn = port.clone();
    let sid_for_turn = session_id.clone();
    let turn_handle = tokio::spawn(async move {
        port_for_turn
            .start_turn(TurnStartParams {
                session_id: sid_for_turn,
                input: vec![InputBlock::Text {
                    text: "run".into(),
                }],
                idempotency_key: "f1-t-1".into(),
            })
            .await
    });

    let resident = poll_until_running(&port, &session_id, std::time::Duration::from_secs(2))
        .await
        .expect("turn became running");
    let running_id = resident.current_turn().expect("running turn id");

    let mut seqs = Vec::new();
    for i in 0..3 {
        let item = port
            .steer_turn(TurnSteerParams {
                session_id: session_id.clone(),
                turn_id: running_id.clone(),
                input: vec![InputBlock::Text {
                    text: format!("steer-{i}"),
                }],
                idempotency_key: format!("f1-t-s{i}"),
            })
            .await
            .expect("steer returns Item with monotonic event_seq");
        seqs.push(item.event_seq.as_u64());
    }

    // Monotonic strictly increasing (not wall-clock ms which could collide
    // or be non-sequential).
    assert_eq!(seqs.len(), 3);
    assert!(seqs[0] < seqs[1], "event_seq must increase: {seqs:?}");
    assert!(seqs[1] < seqs[2], "event_seq must increase: {seqs:?}");

    let _ = port
        .interrupt_turn(TurnInterruptParams {
            session_id: session_id.clone(),
            turn_id: running_id.clone(),
            idempotency_key: "f1-t-done".into(),
        })
        .await
        .unwrap();
    let _ = turn_handle.await.unwrap().unwrap();
}

#[tokio::test]
async fn c1_f2_next_ordinal_seeds_from_summary_on_resume() {
    // F-2 fix: next_ordinal must seed from Summary.num_messages on resume so
    // ordinals do not collide across process restarts. We persist a
    // summary with num_messages > 0, then resume via a fresh runtime (same
    // disk), and assert the first turn's ordinal continues above the
    // persisted count.
    let temp = TempDir::new().unwrap();
    let root = temp.path().to_path_buf();

    // Phase 1: create a session and persist a summary with num_messages = 5
    // by writing updates directly through the storage adapter.
    let storage = JsonlStorageAdapter::with_root(root.clone());
    let info = Info {
        id: acp::SessionId::new("seeded-session".to_string()),
        cwd: "/work/f2-seed".to_string(),
    };
    let _summary = storage
        .init_session(&info, xai_grok_shell::session::persistence::default_model_id())
        .await
        .unwrap();
    // Append 5 user-message updates so num_messages advances. Use the real
    // storage adapter's append_update + recompute summary path.
    for i in 0..5 {
        let notification = acp::SessionNotification::new(
            info.id.clone(),
            acp::SessionUpdate::UserMessageChunk(acp::ContentChunk::new(acp::ContentBlock::Text(
                acp::TextContent::new(format!("user-msg-{i}")),
            ))),
        );
        storage
            .append_update(&info, &SessionUpdate::Acp(Box::new(notification)))
            .await
            .unwrap();
    }
    let summary = storage.load_summary(&info).await.unwrap();
    let persisted_num_messages = summary.num_messages;
    assert!(
        persisted_num_messages >= 1,
        "fixture must persist num_messages >= 1, got {persisted_num_messages}"
    );

    // Phase 2: a fresh runtime over the same disk resumes the session and
    // starts a turn. The ordinal must be > persisted_num_messages (seeded
    // from summary, not reset to 1).
    let (real, _counter) = real_spawn_fn(root.clone());
    let port = ShellSessionActorRuntime::with_production_spawn(root, real);
    let resumed = port
        .resume_session(SessionResumeParams {
            session_id: "seeded-session".to_string(),
            idempotency_key: "f2-resume".into(),
        })
        .await
        .unwrap();
    assert_eq!(resumed.session_id, "seeded-session");
    let turn = port
        .start_turn(TurnStartParams {
            session_id: "seeded-session".to_string(),
            input: vec![InputBlock::Text {
                text: "after-restart".into(),
            }],
            idempotency_key: "f2-t".into(),
        })
        .await
        .expect("turn routes after resume");
    assert!(
        turn.ordinal > persisted_num_messages as u64,
        "ordinal must seed from summary.num_messages ({persisted_num_messages}) on resume, got {}",
        turn.ordinal
    );
}

#[tokio::test]
async fn c1_f3_dead_actor_clears_stale_current_prompt_id_and_returns_session_closed() {
    // F-3 fix: when the actor mailbox is gone (consumer task dropped), a
    // fire-and-forget steer/interrupt must return `session_closed` AND clear
    // the stale `current_prompt_id` so the turn-id guard stays honest.
    let temp = TempDir::new().unwrap();
    let port = ShellSessionActorRuntime::with_spawner(
        temp.path().to_path_buf(),
        Arc::new(DropableActorSpawner {
            root: temp.path().to_path_buf(),
        }),
    );
    let session_id = start_session(&port, "/work/f3-dead-actor", "f3-1").await;

    // The DropableActorSpawner exposes a handle to drop the consumer (close
    // the mailbox). We can't access it directly through the port, so we
    // inject a spawner that records the cmd_tx and a kill switch.
    // Instead, use the resident's cmd_tx directly: drop the receiver by
    // dropping the resident handle's sender side is not possible from here.
    //
    // Approach: get the resident, manually set current_prompt_id to a stale
    // value, then drop the cmd_rx by... we cannot. So we use a spawner that
    // closes its receiver after the first prompt resolves, simulating actor
    // death.
    let turn = port
        .start_turn(TurnStartParams {
            session_id: session_id.clone(),
            input: vec![InputBlock::Text {
                text: "first".into(),
            }],
            idempotency_key: "f3-t-1".into(),
        })
        .await
        .unwrap();
    // After the first turn resolves, the DropableActorSpawner's consumer
    // task exits (it closes after one prompt), so the mailbox is now closed.

    // Manually plant a stale current_prompt_id to simulate the dead actor
    // never having cleared it (the hazard F-3 describes).
    {
        let resident = port.resident(&session_id).expect("resident exists");
        if let Ok(mut g) = resident.current_prompt_id.lock() {
            *g = Some(turn.turn_id.clone());
        }
    }

    // steer_turn against the stale id: the mailbox send fails (closed) →
    // session_closed AND the stale slot is cleared.
    let err = port
        .steer_turn(TurnSteerParams {
            session_id: session_id.clone(),
            turn_id: turn.turn_id.clone(),
            input: vec![InputBlock::Text {
                text: "steer-dead".into(),
            }],
            idempotency_key: "f3-t-2".into(),
        })
        .await
        .unwrap_err();
    assert_eq!(err.code, "session_closed");

    // The stale slot must be cleared so a subsequent steer against the same
    // id returns turn_not_found (not session_closed, and not a false match).
    let err2 = port
        .steer_turn(TurnSteerParams {
            session_id: session_id.clone(),
            turn_id: turn.turn_id.clone(),
            input: vec![InputBlock::Text {
                text: "steer-again".into(),
            }],
            idempotency_key: "f3-t-3".into(),
        })
        .await
        .unwrap_err();
    assert_eq!(
        err2.code, "turn_not_found",
        "stale current_prompt_id must be cleared after session_closed"
    );

    // interrupt_turn against the stale id also returns turn_not_found.
    let err3 = port
        .interrupt_turn(TurnInterruptParams {
            session_id,
            turn_id: turn.turn_id.clone(),
            idempotency_key: "f3-t-4".into(),
        })
        .await
        .unwrap_err();
    assert_eq!(err3.code, "turn_not_found");
}

#[tokio::test]
async fn c1_f4_concurrent_ensure_resident_does_not_double_spawn() {
    // F-4 fix: concurrent ensure_resident for the same session must not
    // double-spawn. The per-session async lock serializes the spawn; the
    // spawner is invoked exactly once.
    let temp = TempDir::new().unwrap();
    let spawn_calls = Arc::new(AtomicUsize::new(0));
    let spawner = Arc::new(CountingActorSpawner {
        root: temp.path().to_path_buf(),
        spawn_calls: spawn_calls.clone(),
    });
    let port = Arc::new(ShellSessionActorRuntime::with_spawner(
        temp.path().to_path_buf(),
        spawner,
    ));

    // Start the same session id many times concurrently via resume_session
    // against a pre-seeded session, so all calls hit ensure_resident for the
    // same session id concurrently.
    let storage = JsonlStorageAdapter::with_root(temp.path().to_path_buf());
    let info = Info {
        id: acp::SessionId::new("f4-session".to_string()),
        cwd: "/work/f4".to_string(),
    };
    let _ = storage
        .init_session(&info, xai_grok_shell::session::persistence::default_model_id())
        .await
        .unwrap();

    let mut handles = Vec::new();
    for _ in 0..8 {
        let port = port.clone();
        handles.push(tokio::spawn(async move {
            port.resume_session(SessionResumeParams {
                session_id: "f4-session".to_string(),
                idempotency_key: "f4-r".into(),
            })
            .await
        }));
    }
    for h in handles {
        let _ = h.await.unwrap();
    }

    // The spawner must have been invoked exactly once despite 8 concurrent
    // resume_session calls for the same session.
    let calls = spawn_calls.load(Ordering::SeqCst);
    assert_eq!(
        calls, 1,
        "ensure_resident must spawn exactly once for the same session (F-4), got {calls}"
    );
}

// --- helpers / held-turn spawner ---

/// A spawner whose consumer keeps the turn running (does not auto-resolve)
/// so steer/interrupt can target a live `current_prompt_id`. Used by F-1.
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
                        if let Ok(mut g) = current_clone.lock() {
                            *g = Some(prompt_id.clone());
                        }
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
        })
    }
}

/// A spawner whose consumer resolves exactly one Prompt then exits, closing
/// the mailbox (simulating actor death). Used by F-3.
struct DropableActorSpawner {
    root: std::path::PathBuf,
}

#[async_trait]
impl SessionSpawner for DropableActorSpawner {
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
            // Process exactly one Prompt, then exit (drop cmd_rx → mailbox closed).
            if let Some(cmd) = cmd_rx.recv().await {
                if let SessionCommand::Prompt {
                    prompt_id,
                    respond_to,
                    ..
                } = cmd
                {
                    if let Ok(mut g) = current_clone.lock() {
                        *g = Some(prompt_id.clone());
                    }
                    let storage = JsonlStorageAdapter::with_root(root.clone());
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
            // Receiver dropped here → mailbox closed. Subsequent sends fail.
        });
        Ok(ResidentHandle {
            cmd_tx,
            current_prompt_id,
            pending_interactions: None,
            delivery_hub: None,
        })
    }
}

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
