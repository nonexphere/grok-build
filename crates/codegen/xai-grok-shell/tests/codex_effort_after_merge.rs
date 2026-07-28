//! Integration: CLI `--effort` must stamp Codex catalog entries merged after
//! the multi-provider report is injected (P0.3 / skeptic gap).
//!
//! Shell `--lib` unit tests are currently unbuildable upstream (WorkspaceOps
//! / EnvVarGuard cfg(test) deps). This integration target compiles the lib
//! without cfg(test) and drives the shipped catalog functions directly.

#![cfg(feature = "native-multi-provider-auth")]

use std::collections::BTreeSet;

use indexmap::IndexMap;
use xai_grok_auth::{CredentialId, ProviderModel};
use xai_grok_multi_auth::cli::{CodexAccountModels, CodexModelsReport};
use xai_grok_multi_auth::provider_model_key::format_provider_model_key;
use xai_grok_sampling_types::ReasoningEffort;
use xai_grok_shell::agent::config::{Config, ModelEntry, ResolvedCredentials, sampling_config_for_model};
use xai_grok_shell::agent::models::{
    clear_codex_merge_report_override_for_test, merge_codex_report_into_catalog,
    resolve_model_catalog, set_codex_merge_report_override_for_test,
    stamp_reasoning_effort_overrides,
};

fn sample_report(credential_id: CredentialId, slug: &str) -> CodexModelsReport {
    CodexModelsReport {
        accounts: vec![CodexAccountModels {
            alias: "default".into(),
            email: Some("test@example.com".into()),
            credential_id,
            account_id: Some("acct-chatgpt-1".into()),
            models: vec![ProviderModel {
                id: slug.into(),
                display_name: "GPT-5.6-Luna".into(),
                description: Some("Fast coding".into()),
                context_window: Some(272_000),
                priority: 0,
                capabilities: BTreeSet::new(),
                raw_metadata: serde_json::Value::Null,
            }],
            error: None,
        }],
    }
}

#[test]
fn resolve_model_catalog_stamps_cli_effort_on_merged_codex_entry() {
    let credential_id = CredentialId::from_uuid(
        uuid::Uuid::parse_str("019f6a33-60f7-78c1-97e7-27fc1ccfc525").unwrap(),
    );
    let provider = xai_grok_auth::ProviderId::new_unchecked("codex");
    let catalog_key = format_provider_model_key(&provider, credential_id, "gpt-5.6-luna");

    set_codex_merge_report_override_for_test(sample_report(credential_id, "gpt-5.6-luna"));

    let cfg = Config {
        reasoning_effort_override: Some(ReasoningEffort::High),
        ..Config::default()
    };

    // Empty prefetched: Codex key only appears after merge_codex_provider_models.
    let catalog = resolve_model_catalog(&cfg, Some(IndexMap::new()));
    clear_codex_merge_report_override_for_test();

    let entry = catalog
        .get(&catalog_key)
        .unwrap_or_else(|| panic!("expected merged Codex key {catalog_key}"));

    assert!(entry.info.supports_reasoning_effort);
    assert_eq!(
        entry.info.reasoning_effort,
        Some(ReasoningEffort::High),
        "CLI --effort high must override merge default Medium on Codex entry"
    );
    assert!(
        entry
            .info
            .reasoning_efforts
            .iter()
            .any(|o| o.value == ReasoningEffort::High)
    );
    assert!(entry.api_key.is_none(), "no OAuth snapshot in api_key");

    let sc = sampling_config_for_model(
        entry,
        ResolvedCredentials {
            api_key: None,
            base_url: entry.info.base_url.clone(),
            auth_type: xai_chat_state::AuthType::SessionToken,
            auth_scheme: entry.info.auth_scheme,
        },
        None,
        None,
        None,
        None,
    );
    assert_eq!(
        sc.reasoning_effort,
        Some(ReasoningEffort::High),
        "sampling_config_for_model must carry CLI effort for Codex"
    );
}

#[test]
fn stamp_after_merge_replaces_medium_default() {
    let credential_id = CredentialId::from_uuid(
        uuid::Uuid::parse_str("aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee").unwrap(),
    );
    let provider = xai_grok_auth::ProviderId::new_unchecked("codex");
    let key = format_provider_model_key(&provider, credential_id, "gpt-5.6-luna");

    let mut catalog: IndexMap<String, ModelEntry> = IndexMap::new();
    merge_codex_report_into_catalog(&mut catalog, &sample_report(credential_id, "gpt-5.6-luna"));
    assert_eq!(
        catalog[&key].info.reasoning_effort,
        Some(ReasoningEffort::Medium),
        "merge baseline is Medium"
    );

    let cfg = Config {
        reasoning_effort_override: Some(ReasoningEffort::Xhigh),
        ..Config::default()
    };
    stamp_reasoning_effort_overrides(&cfg, &mut catalog);
    assert_eq!(
        catalog[&key].info.reasoning_effort,
        Some(ReasoningEffort::Xhigh),
        "stamp after merge must replace Medium with CLI override"
    );
}
