//! Live smoke: list Codex models using credentials under ~/.grok-oss.
//! Ignored in CI by default unless RUN_LIVE_CODEX=1.

#[tokio::test]
async fn live_list_codex_models_from_home() {
    if std::env::var("RUN_LIVE_CODEX").ok().as_deref() != Some("1") {
        eprintln!("skip: set RUN_LIVE_CODEX=1 to run");
        return;
    }
    let home = xai_grok_multi_auth::token_resolve::grok_home();
    let report = xai_grok_multi_auth::cli::list_codex_models(&home)
        .await
        .expect("list_codex_models");
    let text = xai_grok_multi_auth::cli::format_codex_models_report(&report);
    println!("{text}");
    assert!(
        !report.accounts.is_empty(),
        "expected at least one Codex credential under {home:?}"
    );
    let total: usize = report.accounts.iter().map(|a| a.models.len()).sum();
    assert!(
        total > 0,
        "expected models from Codex API; report={text}"
    );
    assert!(
        report.accounts.iter().all(|a| a.error.is_none()),
        "errors in report: {text}"
    );
}
