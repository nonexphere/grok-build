//! `grok`/`goblin models` subcommand.

use anyhow::Result;
use tokio_util::sync::CancellationToken;
use xai_grok_shell::agent::config::Config as AgentConfig;
use xai_grok_shell::cli_models::{AuthStatus, list_models};

use crate::client_identity::{PAGER_CLIENT_TYPE, PAGER_CLIENT_VERSION};

pub async fn list_available_models(agent_config: &AgentConfig) -> Result<()> {
    match AuthStatus::resolve(agent_config) {
        AuthStatus::ApiKey => println!("You are using XAI_API_KEY."),
        AuthStatus::LoggedIn(host) => println!("You are logged in with {}.", host),
        AuthStatus::ModelCredentials(model) => {
            println!("Model '{model}' is using its own API key.");
        }
        AuthStatus::DeploymentKey => println!("You are authenticated via deployment key."),
        AuthStatus::NotAuthenticated => println!("You are not authenticated."),
    }
    println!();

    let cancel = CancellationToken::new();
    let spawned = crate::acp::spawn::spawn_grok_shell(agent_config.clone(), &cancel, None).await?;

    let state = list_models(&spawned.channel.tx, PAGER_CLIENT_TYPE, PAGER_CLIENT_VERSION).await?;

    println!("Default model: {}", state.current_model_id.0);
    println!();
    println!("Available models (xAI / shell):");
    for m in state.available_models {
        if m.model_id == state.current_model_id {
            println!("  * {} (default)", m.model_id.0);
        } else {
            println!("  - {}", m.model_id.0);
        }
    }

    cancel.cancel();

    // Goblin fork: also list Codex models when multi-provider credentials exist.
    let home = std::env::var_os("GROK_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| dirs::home_dir().map(|h| h.join(".grok")))
        .unwrap_or_else(|| std::path::PathBuf::from(".grok"));
    match xai_grok_multi_auth::cli::list_codex_models(&home).await {
        Ok(report) if !report.accounts.is_empty() => {
            println!();
            print!(
                "{}",
                xai_grok_multi_auth::cli::format_codex_models_report(&report)
            );
        }
        Ok(_) => {
            // No Codex credentials — stay quiet so upstream-looking output is clean.
        }
        Err(e) if e.contains("GROK_DISABLE_CODEX_AUTH") => {
            println!("\n(Codex models hidden: {e})");
        }
        Err(e) => {
            eprintln!("\nCodex models unavailable: {e}");
        }
    }

    Ok(())
}
