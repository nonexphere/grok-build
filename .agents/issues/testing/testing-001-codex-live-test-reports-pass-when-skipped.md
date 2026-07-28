id: testing-001
title: Codex live model test reports pass when protocol is skipped
category: testing
type: gap
status: OPEN
severity: MEDIUM
owner: Codex live validation
evidence_quality: confirmed
discovered_by: "@code-review"
discovered_during: codex-provider-finalization-follow-up-2026-07-17
discovered_at: 2026-07-17
evidence:
  - "crates/codegen/xai-grok-multi-auth/tests/live_codex_models.rs:6-11 — the test returns Ok when RUN_LIVE_CODEX is not 1, so cargo reports it as passed"
  - ".llms/evidence/pc8-live-2026-07-17.md — records only a one-shot two-turn observation and lacks the full PC8 evidence pack"
  - "CODEX_100_PERCENT_GOAL.md:206-220 — PC8 requires repeated turns, a negative control, and persisted redacted SSE/usage evidence"
impact: "Automated output can be misread as live protocol proof although no credential, network request, model listing, or prompt-cache behavior ran. Full prompt-cache support remains under-evidenced."
proposed_action: "Mark credential-dependent tests #[ignore] or use a harness that reports an explicit skipped state outside normal pass counts. Add a reproducible gated PC8 probe with session/request correlation, key invariance, third turn, negative control, and durable redacted raw usage/SSE artifacts."
validation_notes: "Default cargo test output must show the live test as ignored/skipped, not passed. With the explicit live gate and authorized ephemeral credentials, the probe must produce complete artifacts and prove both cache hit and expected negative-control drop."

## Links

- Release status: `TO_RELEASE.md` PC8 PARTIAL
- Goal: `CODEX_100_PERCENT_GOAL.md` PC8
- Related issue: docs-001
