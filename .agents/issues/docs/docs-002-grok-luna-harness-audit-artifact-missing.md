id: docs-002
title: Grok Luna harness audit artifact is missing
category: docs
type: gap
status: OPEN
severity: MEDIUM
owner: Codex audit and evidence documentation
evidence_quality: confirmed
discovered_by: user and @code-review
discovered_during: codex-provider-finalization-follow-up-2026-07-17
discovered_at: 2026-07-17
evidence:
  - "Requested artifact `grok-luna-harness-audit-2026-07-17.md` was not found in the worktree"
  - "The artifact was not found under /home/guilherme during the review"
  - "The artifact was not present in `git ls-tree -r fork/goblin-multi-provider-codex`"
impact: "The requested audit cannot be reviewed, reproduced, or reconciled with implementation and release claims. Findings or acceptance criteria contained only in that document may be lost."
proposed_action: "Recover the original artifact from the producing session/worktree or recreate it from preserved evidence. Commit it under the canonical review/evidence directory and link it from TO_RELEASE.md if it is normative."
validation_notes: "Confirm the exact file exists in the branch, is linked from the relevant release/review index, identifies base/head, and maps every conclusion to current code and commands."

## Links

- Related issue: docs-001
- Existing available review: `.llms/reviews/code-audit-grok-goblin-codex-provider-finalization-2026-07-17.md`
