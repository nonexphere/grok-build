//! Product ACP host vertical slice against the repository mock inference server.

use std::sync::Arc;

use agent_client_protocol as acp;
use serde_json::json;
use serial_test::serial;
use tempfile::TempDir;
use xai_grok_app_server_protocol::{
    InputBlock, InteractionResponseParams, SessionStartParams, SubscribeParams, TurnStartParams,
    WireCounter,
};
use xai_grok_shell::agent::config::Config as AgentConfig;
use xai_grok_shell::app_server_runtime::ShellSessionActorRuntime;
use xai_grok_shell::app_server_runtime::experimental_acp_resident_spawn;
use xai_grok_shell::app_server_runtime::spawn_acp_host;
use xai_grok_shell::session::commands::{PromptCompletionKind, SessionCommand};
use xai_grok_shell::session::info::Info;
use xai_grok_shell::session::storage::{JsonlStorageAdapter, StorageAdapter, UpdatesIterator};
use xai_grok_test_support::sse::chat_completions_reasoning_then_tool_call_events;
use xai_grok_test_support::{EnvGuard, MockInferenceServer, ScriptedResponse, SseEvent};
use xai_grok_tower::GrokRuntimeFacade;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn product_acp_host_runs_real_initialize_session_prompt_and_shutdown() {
    let server = MockInferenceServer::start().await.expect("mock server");
    server.set_response("host-response");
    let home = TempDir::new().expect("temp home");
    let cwd = TempDir::new().expect("temp cwd");
    let _home = EnvGuard::set("GROK_HOME", home.path());
    let _api_key = EnvGuard::set("XAI_API_KEY", "test-api-key");

    let mut config = AgentConfig::default();
    config.endpoints.xai_api_base_url = server.url();
    config.endpoints.models_base_url = Some(server.url());
    config.endpoints.models_list_url = Some(format!("{}/models", server.url()));
    let auth_manager = Arc::new(config.create_auth_manager());
    let mut host = spawn_acp_host(config, auth_manager, None, None).expect("spawn ACP host");

    let initialized = host
        .initialize(
            acp::InitializeRequest::new(acp::ProtocolVersion::V1)
                .client_capabilities(
                    acp::ClientCapabilities::new()
                        .fs(acp::FileSystemCapabilities::new())
                        .terminal(false),
                )
                .meta(
                    serde_json::json!({
                        "startupHints": {
                            "nonInteractive": true,
                            "skipGitStatus": true,
                            "skipProjectLayout": true
                        },
                        "clientType": "product-acp-host-test",
                        "clientVersion": "0.0.0-test"
                    })
                    .as_object()
                    .cloned(),
                ),
        )
        .await
        .expect("initialize");
    let api_key_method = initialized
        .auth_methods
        .iter()
        .find(|method| method.id().0.as_ref() == "xai.api_key")
        .expect("xai api key method");
    host.authenticate(
        acp::AuthenticateRequest::new(api_key_method.id().clone())
            .meta(serde_json::json!({"headless": true}).as_object().cloned()),
    )
    .await
    .expect("authenticate");

    let session = host
        .new_session(acp::NewSessionRequest::new(cwd.path().to_path_buf()).mcp_servers(vec![]))
        .await
        .expect("new session");
    let notifications = host.notifications();
    let mut live_notifications = notifications.subscribe();
    let storage = JsonlStorageAdapter::with_root(home.path().to_path_buf());
    let info = Info {
        id: session.session_id.clone(),
        cwd: cwd.path().to_string_lossy().into_owned(),
    };
    storage
        .init_session(&info, acp::ModelId::new("test-model"))
        .await
        .expect("init durable session");
    host.start_persistence(storage.clone(), info.clone())
        .expect("start persistence");
    host.prompt(acp::PromptRequest::new(
        session.session_id.clone(),
        vec![acp::ContentBlock::Text(acp::TextContent::new("hello"))],
    ))
    .await
    .expect("prompt");
    let live = tokio::time::timeout(std::time::Duration::from_secs(5), live_notifications.recv())
        .await
        .expect("live notification timeout")
        .expect("live notification channel");
    assert_eq!(live.session_id, session.session_id);

    assert!(
        notifications
            .snapshot()
            .iter()
            .any(|notification| notification.session_id == session.session_id),
        "real ACP prompt must emit a session notification"
    );
    host.cancel(acp::CancelNotification::new(session.session_id.clone()))
        .await
        .expect("cancel");
    drop(notifications);
    host.shutdown().await.expect("shutdown and join");
    let updates_path = storage.updates_file_path(&info).expect("updates path");
    let updates = UpdatesIterator::open(&updates_path)
        .expect("open updates")
        .expect("updates file");
    assert!(
        updates.count() > 0,
        "ACP notifications must reach durable JSONL"
    );
    assert!(
        server.request_count() > 0,
        "mock must observe real agent traffic"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn product_acp_host_round_trips_real_tool_permission() {
    let server = MockInferenceServer::start().await.expect("mock server");
    server.enqueue_response(
        "/v1/chat/completions",
        ScriptedResponse::sse(chat_completions_reasoning_then_tool_call_events(
            "checking permission",
            "call-permission-1",
            "write",
            r#"{"file_path":"permission-roundtrip.txt","content":"authorized"}"#,
            "test-model",
        )),
    );
    server.enqueue_response(
        "/v1/chat/completions",
        ScriptedResponse::sse(final_chat_text_events("authorized tool completed")),
    );
    let home = TempDir::new().expect("temp home");
    let cwd = TempDir::new().expect("temp cwd");
    let _home = EnvGuard::set("GROK_HOME", home.path());
    let _api_key = EnvGuard::set("XAI_API_KEY", "test-api-key");

    let mut config = AgentConfig::default();
    config.endpoints.xai_api_base_url = server.url();
    config.endpoints.models_base_url = Some(server.url());
    config.endpoints.models_list_url = Some(format!("{}/models", server.url()));
    let auth_manager = Arc::new(config.create_auth_manager());
    let host = spawn_acp_host(config, auth_manager, None, None).expect("spawn ACP host");
    let initialized = host
        .initialize(
            acp::InitializeRequest::new(acp::ProtocolVersion::V1).meta(
                serde_json::json!({
                    "startupHints": {
                        "nonInteractive": false,
                        "skipGitStatus": true,
                        "skipProjectLayout": true
                    },
                    "clientType": "product-acp-host-test",
                    "clientVersion": "0.0.0-test"
                })
                .as_object()
                .cloned(),
            ),
        )
        .await
        .expect("initialize");
    let api_key_method = initialized
        .auth_methods
        .iter()
        .find(|method| method.id().0.as_ref() == "xai.api_key")
        .expect("xai api key method");
    host.authenticate(
        acp::AuthenticateRequest::new(api_key_method.id().clone())
            .meta(serde_json::json!({"headless": true}).as_object().cloned()),
    )
    .await
    .expect("authenticate");
    let session = host
        .new_session(acp::NewSessionRequest::new(cwd.path().to_path_buf()).mcp_servers(vec![]))
        .await
        .expect("new session");
    let mut permissions = host.notifications().subscribe_permission_requests();
    let command = host.command_handle();
    let session_id = session.session_id.clone();
    let prompt = tokio::spawn(async move {
        command
            .prompt_with_context(
                acp::PromptRequest::new(
                    session_id,
                    vec![acp::ContentBlock::Text(acp::TextContent::new(
                        "read the README with permission",
                    ))],
                ),
                "turn-permission-1".into(),
            )
            .await
    });
    let request =
        match tokio::time::timeout(std::time::Duration::from_secs(10), permissions.recv()).await {
            Ok(Ok(request)) => request,
            other => panic!(
                "permission request timeout/broadcast failure: {other:?}; requests={:?}",
                server.request_log_summary()
            ),
        };
    assert_eq!(request.session_id, session.session_id.to_string());
    assert_eq!(request.turn_id.as_deref(), Some("turn-permission-1"));
    assert_eq!(request.tool_call_id.as_deref(), Some("call-permission-1"));
    assert_eq!(request.item_id.as_deref(), Some("tc_call-permission-1"));
    host.respond_permission(
        "call-permission-1".into(),
        xai_grok_shell::app_server_runtime::acp_host::AcpPermissionDecision::Selected(
            "allow-once".into(),
        ),
    )
    .await
    .expect("respond permission");
    let response = tokio::time::timeout(std::time::Duration::from_secs(20), prompt)
        .await
        .expect("prompt completion timeout")
        .expect("prompt task")
        .expect("prompt response");
    assert_eq!(response.stop_reason, acp::StopReason::EndTurn);
    assert!(
        server.request_count() >= 2,
        "tool outcome must trigger follow-up inference"
    );
    host.shutdown().await.expect("shutdown");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn product_facade_respond_interaction_round_trips_real_acp_permission() {
    let server = MockInferenceServer::start().await.expect("mock server");
    server.enqueue_response(
        "/v1/chat/completions",
        ScriptedResponse::sse(chat_completions_reasoning_then_tool_call_events(
            "checking facade permission",
            "call-facade-permission-1",
            "write",
            r#"{"file_path":"facade-roundtrip.txt","content":"authorized"}"#,
            "test-model",
        )),
    );
    server.enqueue_response(
        "/v1/chat/completions",
        ScriptedResponse::sse(final_chat_text_events("facade authorization completed")),
    );
    let home = TempDir::new().expect("temp home");
    let cwd = TempDir::new().expect("temp cwd");
    let _home = EnvGuard::set("GROK_HOME", home.path());
    let _api_key = EnvGuard::set("XAI_API_KEY", "test-api-key");
    let _api_base = EnvGuard::set("GROK_XAI_API_BASE_URL", server.url());
    let _models_base = EnvGuard::set("GROK_MODELS_BASE_URL", server.url());
    let _models_list = EnvGuard::set("GROK_MODELS_LIST_URL", format!("{}/models", server.url()));

    let runtime = Arc::new(ShellSessionActorRuntime::with_production_spawn(
        home.path().to_path_buf(),
        experimental_acp_resident_spawn(home.path().to_path_buf()),
    ));
    let session = runtime
        .start_session(SessionStartParams {
            workspace_root: cwd.path().to_string_lossy().into_owned(),
            agent_type: None,
            provider_binding: None,
            idempotency_key: "facade-permission-session-1".into(),
        })
        .await
        .expect("start product session");
    let session_id = session.session_id.clone();
    let turn_runtime = runtime.clone();
    let turn = tokio::spawn(async move {
        turn_runtime
            .start_turn(TurnStartParams {
                session_id: session_id.clone(),
                input: vec![InputBlock::Text {
                    text: "write a file with permission".into(),
                }],
                idempotency_key: "facade-permission-turn-1".into(),
            })
            .await
    });

    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(15);
    let mut responded = false;
    while tokio::time::Instant::now() < deadline {
        let Some(turn_id) = runtime
            .resident(&session.session_id)
            .and_then(|resident| resident.current_turn())
        else {
            tokio::time::sleep(std::time::Duration::from_millis(25)).await;
            continue;
        };
        match runtime
            .respond_interaction(InteractionResponseParams {
                session_id: session.session_id.clone(),
                turn_id,
                interaction_id: "call-facade-permission-1".into(),
                decision: "allow-once".into(),
                idempotency_key: "facade-permission-response-1".into(),
            })
            .await
        {
            Ok(()) => {
                responded = true;
                break;
            }
            Err(error) if error.code == "interaction_not_found" => {
                tokio::time::sleep(std::time::Duration::from_millis(25)).await;
            }
            Err(error) => panic!("facade interaction/respond failed: {error:?}"),
        }
    }
    assert!(
        responded,
        "facade must eventually observe and deliver ACP permission"
    );
    let completed = tokio::time::timeout(std::time::Duration::from_secs(20), turn)
        .await
        .expect("facade turn timeout")
        .expect("facade turn task")
        .expect("facade turn result");
    assert_eq!(
        completed.status,
        xai_grok_app_server_protocol::TurnStatus::Completed
    );
    assert!(
        server.request_count() >= 2,
        "facade decision must allow follow-up inference"
    );
}

fn final_chat_text_events(text: &str) -> Vec<SseEvent> {
    vec![
        SseEvent::data(
            json!({
                "id": "chatcmpl-final",
                "object": "chat.completion.chunk",
                "created": 1234567890,
                "model": "test-model",
                "choices": [{"index": 0, "delta": {"role": "assistant", "content": text}, "finish_reason": null}]
            })
            .to_string(),
        ),
        SseEvent::data(
            json!({
                "id": "chatcmpl-final",
                "object": "chat.completion.chunk",
                "created": 1234567890,
                "model": "test-model",
                "choices": [{"index": 0, "delta": {}, "finish_reason": "stop"}],
                "usage": {"prompt_tokens": 10, "completion_tokens": 5, "total_tokens": 15}
            })
            .to_string(),
        ),
        SseEvent::data("[DONE]"),
    ]
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn experimental_acp_resident_factory_routes_prompt_and_persists() {
    let server = MockInferenceServer::start().await.expect("mock server");
    server.set_response("resident-response");
    let home = TempDir::new().expect("temp home");
    let cwd = TempDir::new().expect("temp cwd");
    let _home = EnvGuard::set("GROK_HOME", home.path());
    let _api_key = EnvGuard::set("XAI_API_KEY", "test-api-key");
    let _api_base = EnvGuard::set("GROK_XAI_API_BASE_URL", server.url());
    let _models_base = EnvGuard::set("GROK_MODELS_BASE_URL", server.url());
    let _models_list = EnvGuard::set("GROK_MODELS_LIST_URL", format!("{}/models", server.url()));

    let session_id = acp::SessionId::new(uuid::Uuid::now_v7().to_string());
    let info = Info {
        id: session_id.clone(),
        cwd: cwd.path().to_string_lossy().into_owned(),
    };
    let root = home.path().to_path_buf();
    let factory = experimental_acp_resident_spawn(root.clone());
    let resident = factory(info.clone(), acp::ModelId::new("test-model"))
        .await
        .expect("real ACP resident");
    let (response_tx, response_rx) = tokio::sync::oneshot::channel();
    let (response_tx_2, response_rx_2) = tokio::sync::oneshot::channel();
    resident
        .cmd_tx
        .send(SessionCommand::prompt(
            "turn-1".into(),
            vec![acp::ContentBlock::Text(acp::TextContent::new("hello"))],
            response_tx,
        ))
        .expect("send prompt");
    resident
        .cmd_tx
        .send(SessionCommand::prompt(
            "turn-2".into(),
            vec![acp::ContentBlock::Text(acp::TextContent::new(
                "hello again",
            ))],
            response_tx_2,
        ))
        .expect("send queued prompt");
    let result = tokio::time::timeout(std::time::Duration::from_secs(20), response_rx)
        .await
        .expect("prompt timeout")
        .expect("prompt response");
    let ok = result.expect("prompt success");
    assert!(matches!(
        ok.completion_kind,
        PromptCompletionKind::Completed
    ));
    assert_eq!(ok.stop_reason, acp::StopReason::EndTurn);
    let second = tokio::time::timeout(std::time::Duration::from_secs(20), response_rx_2)
        .await
        .expect("second prompt timeout")
        .expect("second prompt response")
        .expect("second prompt success");
    assert!(matches!(
        second.completion_kind,
        PromptCompletionKind::Completed
    ));

    resident
        .cmd_tx
        .send(SessionCommand::Shutdown)
        .expect("shutdown");
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    let storage = JsonlStorageAdapter::with_root(root);
    let updates_path = storage.updates_file_path(&info).expect("updates path");
    assert!(
        updates_path.is_file(),
        "resident bridge must persist ACP updates"
    );
    assert!(
        server.request_count() > 0,
        "resident must use mock inference"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn experimental_acp_resident_cancel_is_observable_and_persists_terminal_update() {
    let server = MockInferenceServer::start().await.expect("mock server");
    server.set_response("slow-resident-response");
    server.set_chunk_delay(Some(std::time::Duration::from_millis(250)));
    let home = TempDir::new().expect("temp home");
    let cwd = TempDir::new().expect("temp cwd");
    let _home = EnvGuard::set("GROK_HOME", home.path());
    let _api_key = EnvGuard::set("XAI_API_KEY", "test-api-key");
    let _api_base = EnvGuard::set("GROK_XAI_API_BASE_URL", server.url());
    let _models_base = EnvGuard::set("GROK_MODELS_BASE_URL", server.url());
    let _models_list = EnvGuard::set("GROK_MODELS_LIST_URL", format!("{}/models", server.url()));

    let info = Info {
        id: acp::SessionId::new(uuid::Uuid::now_v7().to_string()),
        cwd: cwd.path().to_string_lossy().into_owned(),
    };
    let factory = experimental_acp_resident_spawn(home.path().to_path_buf());
    let resident = factory(info.clone(), acp::ModelId::new("test-model"))
        .await
        .expect("real ACP resident");
    let (response_tx, response_rx) = tokio::sync::oneshot::channel();
    resident
        .cmd_tx
        .send(SessionCommand::prompt(
            "cancelled-turn".into(),
            vec![acp::ContentBlock::Text(acp::TextContent::new("cancel me"))],
            response_tx,
        ))
        .expect("send prompt");
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    resident
        .cmd_tx
        .send(SessionCommand::Cancel {
            cancel_subagents: true,
            kill_background_tasks: false,
            rewind_if_pristine: false,
            trigger: Some("product-test".into()),
        })
        .expect("send cancel");
    let result = tokio::time::timeout(std::time::Duration::from_secs(10), response_rx)
        .await
        .expect("cancelled prompt timeout")
        .expect("cancelled prompt response")
        .expect("ACP prompt transport failure");
    assert!(matches!(
        result.completion_kind,
        PromptCompletionKind::Cancelled { .. }
    ));

    resident
        .cmd_tx
        .send(SessionCommand::Shutdown)
        .expect("shutdown");
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    let storage = JsonlStorageAdapter::with_root(home.path().to_path_buf());
    let updates_path = storage.updates_file_path(&info).expect("updates path");
    assert!(
        updates_path.is_file(),
        "cancelled resident must persist ACP updates"
    );
    assert!(
        UpdatesIterator::open(&updates_path)
            .expect("open updates")
            .expect("updates file")
            .count()
            > 0
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn acp_persisted_notifications_are_replayed_through_shell_facade() {
    let server = MockInferenceServer::start().await.expect("mock server");
    server.set_response("facade-replay-response");
    let home = TempDir::new().expect("temp home");
    let cwd = TempDir::new().expect("temp cwd");
    let _home = EnvGuard::set("GROK_HOME", home.path());
    let _api_key = EnvGuard::set("XAI_API_KEY", "test-api-key");
    let _api_base = EnvGuard::set("GROK_XAI_API_BASE_URL", server.url());
    let _models_base = EnvGuard::set("GROK_MODELS_BASE_URL", server.url());
    let _models_list = EnvGuard::set("GROK_MODELS_LIST_URL", format!("{}/models", server.url()));

    let info = Info {
        id: acp::SessionId::new(uuid::Uuid::now_v7().to_string()),
        cwd: cwd.path().to_string_lossy().into_owned(),
    };
    let factory = experimental_acp_resident_spawn(home.path().to_path_buf());
    let resident = factory(info.clone(), acp::ModelId::new("test-model"))
        .await
        .expect("real ACP resident");
    let (response_tx, response_rx) = tokio::sync::oneshot::channel();
    resident
        .cmd_tx
        .send(SessionCommand::prompt(
            "replay-turn".into(),
            vec![acp::ContentBlock::Text(acp::TextContent::new("replay me"))],
            response_tx,
        ))
        .expect("send prompt");
    response_rx
        .await
        .expect("prompt response")
        .expect("prompt success");

    let runtime = ShellSessionActorRuntime::new(home.path().to_path_buf());
    let mut page = None;
    for _ in 0..50 {
        let candidate = runtime
            .replay(SubscribeParams {
                session_id: info.id.to_string(),
                after_event_seq: WireCounter::new(0),
                history_epoch: None,
            })
            .await
            .expect("facade replay");
        if candidate.events.len() > 1 {
            page = Some(candidate);
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    let page = page.expect("ACP notification must become visible through facade replay");
    assert!(page.events.iter().any(|event| {
        matches!(
            event,
            xai_grok_tower::RuntimeEvent::ItemCompleted(_)
                | xai_grok_tower::RuntimeEvent::ItemDelta { .. }
        )
    }));
    resident
        .cmd_tx
        .send(SessionCommand::Shutdown)
        .expect("shutdown");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial]
async fn experimental_acp_resident_factory_rolls_back_on_invalid_session() {
    let server = MockInferenceServer::start().await.expect("mock server");
    let home = TempDir::new().expect("temp home");
    let _home = EnvGuard::set("GROK_HOME", home.path());
    let _api_key = EnvGuard::set("XAI_API_KEY", "test-api-key");
    let _api_base = EnvGuard::set("GROK_XAI_API_BASE_URL", server.url());
    let _models_base = EnvGuard::set("GROK_MODELS_BASE_URL", server.url());
    let _models_list = EnvGuard::set("GROK_MODELS_LIST_URL", format!("{}/models", server.url()));

    let info = Info {
        id: acp::SessionId::new(uuid::Uuid::now_v7().to_string()),
        cwd: "relative-cwd-is-invalid".into(),
    };
    let result = experimental_acp_resident_spawn(home.path().to_path_buf())(
        info,
        acp::ModelId::new("test-model"),
    )
    .await;
    assert!(result.is_err(), "invalid session bootstrap must roll back");
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}
