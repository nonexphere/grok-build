id: data-001
title: Codex 401 recovery falls back to a different request stamp
category: data
type: bug
status: DONE
severity: HIGH
owner: multi-provider auth and shell sampling recovery
evidence_quality: confirmed
discovered_by: "@code-review"
discovered_during: codex-provider-finalization-follow-up-2026-07-17
discovered_at: 2026-07-17
closed: 2026-07-17
evidence:
  - "crates/codegen/xai-grok-shell/src/auth/multi_provider_resolve.rs:82-95 — a missing attempt id or missing exact stamp falls back to take_stamp_for_recovery(), which is FIFO"
  - "crates/codegen/xai-grok-sampler/src/client.rs:633-684 — the sampler now owns attempt_id per request, so the remaining fallback is no longer required by the production composition path"
  - ".agents/skills/add-provider/SKILL.md:236-242 — the provider contract forbids FIFO/last-wins recovery and treats a multi-provider 401 without attempt id as an invariant breach"
impact: "Concurrent Codex requests can recover a 401 using another request's credential generation stamp. This can refresh or invalidate the wrong generation/account and makes the A1 PASS claim unsafe under an invariant breach."
proposed_action: "Fail closed in multi-provider recovery when attempt_id is absent or take_attempt(id) misses. Retain FIFO only behind an explicitly separate legacy non-multi-provider API, if that compatibility path is still required."
validation_notes: "Add composition tests for (1) 401 without attempt_id and (2) unknown/already-consumed attempt_id while another stamp is queued. Both must refuse recovery and must leave the unrelated stamp untouched. Keep the inverted concurrent HTTP 401 test green."

## Links

- Review: `.llms/reviews/code-audit-grok-goblin-codex-provider-finalization-2026-07-17.md`
- Release claim: `TO_RELEASE.md` A1
- Related skill: `.agents/skills/add-provider/SKILL.md`
