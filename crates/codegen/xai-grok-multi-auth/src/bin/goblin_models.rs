//! Lightweight `goblin models --codex` helper: lists OpenAI Codex models
//! using multi-provider credentials under GROK_HOME / ~/.grok.
//!
//! Also invoked by the `goblin` wrapper when the full pager binary has not
//! yet been rebuilt with multi-provider model listing.

#[tokio::main]
async fn main() {
    let home = std::env::var_os("GROK_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|h| std::path::PathBuf::from(h).join(".grok"))
        })
        .unwrap_or_else(|| std::path::PathBuf::from(".grok"));

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
