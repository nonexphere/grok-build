//! Lightweight models helper: lists OpenAI Codex models using multi-provider
//! credentials under GROK_OSS_HOME / GROK_HOME / ~/.grok-oss.
//!
//! Also invoked by the PATH wrapper when the full pager binary has not
//! yet been rebuilt with multi-provider model listing.

#[tokio::main]
async fn main() {
    let home = xai_grok_multi_auth::token_resolve::grok_home();

    match xai_grok_multi_auth::cli::list_codex_models(&home).await {
        Ok(report) => {
            print!(
                "{}",
                xai_grok_multi_auth::cli::format_codex_models_report(&report)
            );
            if report.accounts.is_empty() {
                std::process::exit(1);
            }
            if report.accounts.iter().any(|a| a.error.is_some()) {
                std::process::exit(2);
            }
        }
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    }
}
