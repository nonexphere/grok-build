id: testing-002
title: Branch diff check fails on added documentation
category: testing
type: gap
status: DONE
severity: LOW
owner: branch hygiene and documentation owners
evidence_quality: confirmed
discovered_by: "@code-review"
discovered_during: codex-provider-finalization-follow-up-2026-07-17
discovered_at: 2026-07-17
closed: 2026-07-17
evidence:
  - "Command `git diff --check goblin..HEAD` exited 2 on 2026-07-17"
  - ".llms/grok-build/app-server/*/README.md — added lines contain trailing whitespace"
  - "docs/architecture/byok-providers-onboarding/*.md and docs/runtime/turn-queue-subagents-and-followups.md — additional trailing-whitespace hits"
impact: "The repository's declared provider verification gate is red. Although most hits are outside Codex runtime, they prevent a clean readiness result for the branch as a whole."
proposed_action: "Remove unintended trailing whitespace from files reported by git diff --check without rewriting unrelated content or semantic Markdown line breaks that are intentional."
validation_notes: "Run `git diff --check goblin..HEAD`; it must exit 0. Review the resulting documentation-only diff to ensure no prose or formatting semantics were lost."

## Links

- Related skill gate: `.agents/skills/add-provider/SKILL.md` Verification
