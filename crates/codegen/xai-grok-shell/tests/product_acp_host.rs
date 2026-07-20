//! Product ACP host vertical slice against the repository mock inference server.

use std::sync::Arc;

use agent_client_protocol as acp;
use serial_test::serial;
use tempfile::TempDir;
use xai_grok_shell::agent::config::Config as AgentConfig;
use xai_grok_shell::app_server_runtime::spawn_acp_host;
use xai_grok_shell::app_server_runtime::experimental_acp_resident_spawn;
use xai_grok_shell::session::info::Info;
use xai_grok_shell::session::commands::{PromptCompletionKind, SessionCommand};
use xai_grok_shell::session::storage::{JsonlStorageAdapter, StorageAdapter, UpdatesIterator};
use xai_grok_test_support::{EnvGuard, MockInferenceServer};

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
    assert!(updates.count() > 0, "ACP notifications must reach durable JSONL");
    assert!(server.request_count() > 0, "mock must observe real agent traffic");
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
            vec![acp::ContentBlock::Text(acp::TextContent::new("hello again"))],
            response_tx_2,
        ))
        .expect("send queued prompt");
    let result = tokio::time::timeout(std::time::Duration::from_secs(20), response_rx)
        .await
        .expect("prompt timeout")
        .expect("prompt response");
    let ok = result.expect("prompt success");
    assert!(matches!(ok.completion_kind, PromptCompletionKind::Completed));
    assert_eq!(ok.stop_reason, acp::StopReason::EndTurn);
    let second = tokio::time::timeout(std::time::Duration::from_secs(20), response_rx_2)
        .await
        .expect("second prompt timeout")
        .expect("second prompt response")
        .expect("second prompt success");
    assert!(matches!(second.completion_kind, PromptCompletionKind::Completed));

    resident.cmd_tx.send(SessionCommand::Shutdown).expect("shutdown");
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    let storage = JsonlStorageAdapter::with_root(root);
    let updates_path = storage.updates_file_path(&info).expect("updates path");
    assert!(updates_path.is_file(), "resident bridge must persist ACP updates");
    assert!(server.request_count() > 0, "resident must use mock inference");
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
