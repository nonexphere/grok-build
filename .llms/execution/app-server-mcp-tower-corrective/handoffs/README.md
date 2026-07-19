# GLM handoffs — corrective App Server / MCP / Tower

| ID | File | Role | Model | Parallel? | Owns |
|---|---|---|---|---|---|
| **C0-A** | `HANDOFF-C0-A-requirement-matrix.md` | build / docs | glm-5.2 | yes with B,C | `.llms/**` matrix + reopen checkboxes |
| **C0-B** | `HANDOFF-C0-B-session-actor-map.md` | repo-explore | glm-5.2 | yes with A,C | SessionActor command map (read-only) |
| **C0-C** | `HANDOFF-C0-C-architecture-review.md` | review | glm-5.2 | after A/B preferred | C0 architecture GO/NO-GO |
| **C1-D** | `HANDOFF-C1-D-shell-port-impl.md` | build | glm-5.2 | **no** — after C0 GO | Shell SessionActor port + tests |
| **C1-E** | `HANDOFF-C1-E-code-review.md` | review | glm-5.2 | after D stable | independent code review |
| **C1-F** | `HANDOFF-C1-F-test-review.md` | review | glm-5.2 | after D stable | independent test review |
| **C1-G** | `HANDOFF-C1-G-turn-lifecycle.md` | build | glm-5.2 | **no** exclusive product writer | turn lifecycle via SessionHandle |
| **C1-H** | `HANDOFF-C1-H-code-review.md` | review | glm-5.2 | after G stable | code review of turn slice |
| **C1-I** | `HANDOFF-C1-I-test-review.md` | review | glm-5.2 | after G stable | test review of turn slice |
| **C3-A** | `HANDOFF-C3-A-ws-surface-map.md` | repo-explore | glm-5.2 | yes with C1-G / C4-A / C5-A | WS surface map only |
| **C3-B** | `HANDOFF-C3-B-ws-listener.md` | build | glm-5.2 | after C3-A; parallel C5-B | real WS listener |
| **C4-A** | `HANDOFF-C4-A-mcp-surface-map.md` | repo-explore | glm-5.2 | yes with C1-G / C3-A / C5-A | MCP surface map only |
| **C5-A** | `HANDOFF-C5-A-provider-surface-map.md` | repo-explore | glm-5.2 | yes with C1-G / C3-A / C4-A | provider surface map only |
| **C5-B** | `HANDOFF-C5-B-byok-providers.md` | build | glm-5.2 | parallel C1-G | BYOK OpenRouter/Groq/Cloudflare offline |

## Status legend (orchestrator) — 2026-07-18/19 wave

| ID | Status |
|---|---|
| C0-A..C1-F | **done** |
| C1-G | **done** (command routing REAL; production spawn PARTIAL) |
| C1-H/I | **done** PASS_WITH_FINDINGS |
| C3-A / C4-A / C5-A | **done** (maps) |
| C3-B | **done** real WS listener |
| C3-C/D | **done** PASS_WITH_FINDINGS |
| C4-B | **done** real Streamable HTTP |
| C4-C | **done** PASS |
| C4-D | **done** PASS_WITH_FINDINGS |
| C5-B | **done** offline BYOK |
| Next | C1 residual spawn factory · composition wiring · C2 · history · C6/C7 |

## Orchestration rules

1. Primary agent (orchestrator) owns integration, commits, and status claims.
2. Parallel only on **non-overlapping** ownership.
3. Never two writers on the same product module.
4. Reviewers never author the slice they review.
5. Ledger root: `.llms/execution/app-server-mcp-tower-corrective/`
6. All handoff subagents use model **`glm-5.2`** unless primary overrides.

## Spawn template

```text
model: glm-5.2
subagent_type: build | repo-explore | review
prompt: Read the handoff file completely and execute it. Report evidence paths.
```
