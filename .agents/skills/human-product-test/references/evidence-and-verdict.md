# Evidence and verdict rules

## Purpose

Product QA claims must be **reconstructible** after the session ends. Chat
narrative alone is not evidence.

## Evidence root

```text
.agents/evidence/product-qa/<YYYY-MM-DD>-<short-slug>/
```

Examples:

- `.agents/evidence/product-qa/2026-07-19-pager-model-picker/`
- `.agents/evidence/product-qa/2026-07-19-smoke-default/`

Create a `README.md` or use the report template as `REPORT.md` in that folder.

### What belongs here

- Command transcripts (`tee` outputs)
- TUI frames from `termctrl`: `l3-tui.txt`, `l3-tui.png` (preferred), optional SVG
- Optional recordings: `*.termctrl` (sensitive) and exported `l3-demo.mp4`
- tmux fallback pane dumps (`l3-tui-pane.txt`) when `termctrl` unavailable
- Headless stdout/stderr
- `cargo` / version lines
- Redacted auth diagnostics (no secrets)

### What must not appear

- Access tokens, refresh tokens, API keys
- OAuth user codes, device codes, full verification URLs with secrets
- Cookie headers
- Password material
- Truncated secret prefixes of length 4/8/12/20 of real tokens
- Unredacted `termctrl` recordings (`.termctrl` JSON Lines may include prompts and
  typed secrets) — scrub, avoid commit, or keep out of the repo evidence tree

If a tool dumps secrets, scrub before copying into the evidence tree.

### Preferred L3 artifact set

| Artifact | Strength |
| --- | --- |
| `l3-tui.png` + `l3-tui.txt` from `termctrl save` | Strong — default for `proven` |
| `termctrl show` text only | Acceptable if PNG failed |
| `l3-demo.mp4` from `termctrl video` | Optional demo / handoff |
| tmux `capture-pane` text | Weak fallback |

Screenshot-equivalent for this skill means **`termctrl save`/`show` first**, then
tmux if needed.

## `/tmp` policy

Scratch homes and intermediate logs under `/tmp/grok-oss-qa-…` are fine for
**runtime isolation**. Any row marked `proven` that matters for handoff must
also have a copy under `.agents/evidence/product-qa/…`.

## Verdict vocabulary

Use the same terms as [human-checklist.md](./human-checklist.md):

| Verdict | Use when |
| --- | --- |
| `proven` | Artifact shows expected behavior |
| `contradicted` | Artifact shows wrong behavior (product FAIL) |
| `incomplete` | Started; insufficient data |
| `weak` | Flaky or ambiguous |
| `missing` | Required by scope; not run |
| `blocked` | Environment/auth/tooling prevented run |
| `skipped` | Out of scope (reason required) |
| `n/a` | Not applicable |

### Promotion rules (forbidden)

- skip → proven  
- blocked → proven  
- missing → proven  
- weak → proven without re-run  

### Overall session rollup

| Overall | Condition |
| --- | --- |
| **PASS** | Every **in-scope required** row is `proven` (or justified `n/a`) |
| **PARTIAL** | Mix of proven and skipped/blocked non-critical; no `contradicted` on required |
| **FAIL** | Any in-scope required row is `contradicted` |
| **BLOCKED** | Cannot complete required layers due to env (build broken, no PTY tool for mandatory TUI, etc.) |

Required rows = those selected in triage (smoke profile or explicit list).

## Severity × confidence (findings)

When reporting product defects found during QA:

```text
[SEVERITY][Confidence] short title
```

| Severity | Meaning |
| --- | --- |
| `BLOCKER` | Product unusable for the scoped human path |
| `HIGH` | Major wrong behavior; likely ship blocker for that surface |
| `MEDIUM` | Degraded UX or secondary path |
| `LOW` | Polish / docs mismatch |

| Confidence | Meaning |
| --- | --- |
| `Confirmed` | Reproduced with artifact |
| `Likely` | Strong signal; one gap in proof |
| `Possible` | Incomplete observation |
| `Unverified` | Hypothesis only |

Link each finding to evidence path + optional `@issue-lifecycle` id.

## Mapping table (mandatory in report)

For each checklist ID in scope:

```text
| ID | Verdict | Command | Artifact | Notes |
```

Empty artifact + `proven` is invalid.

## Live vs offline honesty

| Kind | PASS requires |
| --- | --- |
| Offline CLI | Local process evidence only |
| Live inference | Network call succeeded + redacted durable log |
| Live OAuth | Login completed or typed failure with artifact |

If credentials absent: mark `skipped` or `blocked`, **not** pass.

Known anti-pattern (already tracked in fork testing issues): automated “live”
harnesses that exit 0 when they skipped the live path. Always read the log for
`skip`, `ignored`, `no credentials`, `not run`.

## Redaction quick checklist

Before saving evidence:

1. Search artifacts for `sk-`, `Bearer `, `eyJ`, `refresh_token`, `access_token`.
2. Search for known canary substrings if tests inject secrets.
3. Replace with `[REDACTED]` keeping structure of the log.

## Relationship to other skills

- `@session-evidence-gate` — can consume this report as plan evidence.
- `@delivery-report` — may cite the evidence folder.
- `@issue-lifecycle` — durable FAIL tracking beyond the evidence folder.
