# Product QA report — grok-oss / Goblin

| Field | Value |
| --- | --- |
| Date | YYYY-MM-DD |
| Slug | short-kebab-case |
| Branch | |
| HEAD short SHA | |
| Binary path | e.g. `./target/debug/grok-oss` |
| Binary version line | paste |
| `GROK_OSS_HOME` (product) | scratch path |
| Dogfood host home? | no / yes (why) |
| Observability | termctrl CLI / termctrl mcp / tmux / none |
| Scope profile | smoke / auth / release / custom |
| Operator (agent/session) | |
| Overall verdict | PASS / PARTIAL / FAIL / BLOCKED |

## Intent

What the user asked to prove (1–3 sentences).

## Scope

### In scope (layers / surfaces)

- …

### Out of scope (with reason)

- …

## Preflight

| Check | Result |
| --- | --- |
| Repo root | |
| Dirty tree notes | |
| Disk/toolchain | |
| tmux available | yes/no |
| `termctrl` available | yes/no |
| `termctrl mcp` configured | yes/no |
| `ffmpeg` (for video) | yes/no |

## Build

```text
commands:
results:
artifact: .agents/evidence/product-qa/<slug>/b-version.txt
```

| ID | Verdict | Artifact | Notes |
| --- | --- | --- | --- |
| B1 | | | |
| B2 | | | |
| B3 | | | |
| B4 | | | |

## L0 — Automated (if any)

| ID | Verdict | Command | Artifact | Notes |
| --- | --- | --- | --- | --- |
| U1 | | | | |

## L1 — CLI

| ID | Verdict | Command | Artifact | Notes |
| --- | --- | --- | --- | --- |
| C1 | | | | |
| C2 | | | | |
| C3 | | | | |
| C7 | | | | |

## L2 — Headless

| ID | Verdict | Command | Artifact | Notes |
| --- | --- | --- | --- | --- |
| H1 | | | | |
| H5 | | | | |
| H6 | skipped/blocked/proven | | | auth? |

## L3 — TUI

| ID | Verdict | Method (`termctrl` / tmux) | Artifact | Notes |
| --- | --- | --- | --- | --- |
| T1 | | | | |
| T2 | | | l3-tui.txt | |
| T3 | | | l3-tui.png | preferred |
| T4 | | | | |
| T5 | | | | |
| T9 | skipped/proven | video | l3-demo.mp4 | optional |

Screen excerpt (optional, redacted):

```text
paste short excerpt of l3-tui.txt
```

## L4 — Auth (gated)

Authorization statement: _none / user allowed X_

| ID | Verdict | Artifact | Notes |
| --- | --- | --- | --- |
| A1 | | | |
| … | | | |

## L5 — Install

| ID | Verdict | Artifact | Notes |
| --- | --- | --- | --- |
| I1 | | | |
| I2 | | | |
| I3 | | | |

## Honesty guards

| ID | Verdict | Notes |
| --- | --- | --- |
| R1 | | no skip-as-pass |
| R2 | | evidence under `.agents/evidence/…` |
| R3 | | remaining surfaces listed below |
| R4 | | related issues |

## Findings

Use `[SEVERITY][Confidence] title` — evidence path — next action.

1. …

## Issues filed

| Path / id | Title |
| --- | --- |
| | |

## Remaining work

- …

## Conclusion

One paragraph: overall verdict, what a human can trust, what is still unproven.
