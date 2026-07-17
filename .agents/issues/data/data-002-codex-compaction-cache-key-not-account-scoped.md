id: data-002
title: Codex compaction prompt cache key is not account scoped
category: data
type: contract-drift
status: DONE
severity: HIGH
owner: sampling types and shell compaction
evidence_quality: confirmed
discovered_by: "@code-review"
discovered_during: codex-provider-finalization-follow-up-2026-07-17
discovered_at: 2026-07-17
closed: 2026-07-17
evidence:
  - "crates/codegen/xai-grok-shell/src/session/helpers/session_compact.rs:468-499 — compaction pre-populates prompt_cache_key from session and agent identity before calling the sampler"
  - "crates/codegen/xai-grok-sampling-types/src/conversation.rs:2800-2812 — prompt_cache_key_for_compaction calls derive_prompt_cache_key without provider or credential"
  - "crates/codegen/xai-grok-sampling-types/src/conversation.rs:2414-2426 — account-scoped ensure returns immediately when the pre-populated key is non-empty"
  - "crates/codegen/xai-grok-sampler/src/client.rs:1967-1974 — the sampler has provider and credential identity but cannot replace the pre-populated key"
impact: "Two Codex credentials using the same session/agent identity can send the same cache-affinity key during compaction, violating account isolation and the documented PC6/PC10 provider contract."
proposed_action: "Either leave compaction prompt_cache_key unset and let the sampler derive it from its binding, or extend the compaction helper to receive provider and opaque credential identity. Ensure normal turns and compaction share the same account-scoping policy."
validation_notes: "At the real shell-to-sampler composition boundary, create identical compaction requests for two credential IDs in one session and assert different gpc_ keys. Assert same credential/session is stable and non-Codex omits the key."

## Links

- Remediation contract: `CODEX_AUDIT_REMEDIATION_PLAN.md` item 1.5
- Release claim: `TO_RELEASE.md` PC10
- Related: data-001
