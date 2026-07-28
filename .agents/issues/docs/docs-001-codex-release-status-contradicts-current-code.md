id: docs-001
title: Codex release status contradicts current implementation evidence
category: docs
type: contract-drift
status: OPEN
severity: MEDIUM
owner: Codex release documentation
evidence_quality: confirmed
discovered_by: "@code-review"
discovered_during: codex-provider-finalization-follow-up-2026-07-17
discovered_at: 2026-07-17
evidence:
  - "TO_RELEASE.md:3,38,45,51 — declares the offline Codex path, A1, and PC10 PASS despite open data-001 and data-002"
  - "CODEX_AUDIT_REMEDIATION_PLAN.md:15-28 — still labels many already-remediated historical findings as open, conflicting with TO_RELEASE.md"
  - "TO_RELEASE.md:55 — says only external/1.0 deferrals remain, while confirmed in-scope correctness findings remain"
impact: "Maintainers and release reviewers cannot identify the canonical completion state. The branch may be merged or released based on PASS claims that exceed the current evidence."
proposed_action: "Change A1, PC10, and the offline production-ready summary to PARTIAL until data-001/data-002 close. Add a dated current-status section or archive/supersede the stale remediation matrix so one canonical status exists per requirement."
validation_notes: "Search all Codex goal, remediation, progress, README, review, and release documents. Each requirement must have one compatible status and every PASS must map to current symbol, test, command result, and commit."

## Links

- Related issues: data-001, data-002, testing-001, docs-002
- Canonical release inventory: `TO_RELEASE.md`
