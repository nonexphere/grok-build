# GLM handoffs — corrective App Server / MCP / Tower

| ID | File | Role | Model | Parallel? | Owns |
|---|---|---|---|---|---|
| **C0-A** | `HANDOFF-C0-A-requirement-matrix.md` | build / docs | glm-5.2 | yes with B,C | `.llms/**` matrix + reopen checkboxes |
| **C0-B** | `HANDOFF-C0-B-session-actor-map.md` | repo-explore | glm-5.2 | yes with A,C | SessionActor command map (read-only) |
| **C0-C** | `HANDOFF-C0-C-architecture-review.md` | review | glm-5.2 | after A/B preferred; may start with audit only | C0 architecture GO/NO-GO |
| **C1-D** | `HANDOFF-C1-D-shell-port-impl.md` | build | glm-5.2 | **no** — after C0 GO | Shell SessionActor port + tests |
| **C1-E** | `HANDOFF-C1-E-code-review.md` | review | glm-5.2 | after D stable | independent code review |
| **C1-F** | `HANDOFF-C1-F-test-review.md` | review | glm-5.2 | after D stable | independent test review |

## Orchestration rules

1. Primary agent owns integration, commits, and status claims.
2. Parallel only on **non-overlapping** ownership (C0-A/B/C).
3. Never two writers on the same product module.
4. Reviewers never author the slice they review.
5. Ledger root: `.llms/execution/app-server-mcp-tower-corrective/`

## Spawn template

```text
model: glm-5.2
subagent_type: build | repo-explore | review
prompt: Read handoff file completely and execute. Report evidence paths.
```
