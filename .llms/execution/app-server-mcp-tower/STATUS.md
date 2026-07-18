# STATUS — App Server / MCP / Tower execution

| Field | Value |
|---|---|
| Task | `.llms/tasks/20260718-execute-app-server-mcp-tower-plan.md` |
| Protocol | `2026-07-18.experimental-v2` |
| Branch | `goblin-implement-epic-tree` |
| Wave | Wave 2 complete (FakeRuntime); Wave 1 PARTIAL; Wave 3–6 in progress |
| Current focus | **B-1** Shell `GrokRuntimeFacade` adapter + composition root wiring |
| Next action | Implement `impl GrokRuntimeFacade` in `xai-grok-shell/src/app_server_runtime/` |
| Worktree | `/home/guilherme/github/grok-goblin` |

## Proven green (local / FakeRuntime)

- Protocol: errors, envelope, transitions, goldens, schema check, 22 tests
- Tower: FakeRuntime, registry one-actor, projection redaction, budgets, workspace, lease, multi-instance dir, 14 tests
- App Server: processor, in-process, stdio NDJSON, websocket bearer+parity, controller lease, 12+ tests
- Tools: 9-tool invoke, ACL fail-closed, send mode, idempotency, 8 tests
- MCP: tools/list + tools/call over same core, 3 tests
- Shell: single-winner lock, handshake bytes, app_server_runtime marker, 2 adapter tests
- SDK: typecheck/test/drift
- `grok-oss` binary builds

## Open (blocking production path)

1. **Shell GrokRuntimeFacade adapter** (real SessionActor/leader forward) — B-1
2. Composition root inject in `xai-grok-pager-bin`
3. MCP Streamable HTTP + full WS server framing
4. Provider verticals 10/v1-02..05
5. Wave 6 security/ops hardening + TLS HUMAN gate
6. RF102-05 single_actor_owns_turn_mutation on real actor

## HUMAN gates (unchanged)

npm publish name, token CLI UX, missing-jsonrpc adapter, approval timeout, Tower CLI flags, non-loopback TLS+threat, live provider credentials.

## Reviews

- `.llms/execution/app-server-mcp-tower/reviews/wave0-2/code-review.md`
- `.llms/execution/app-server-mcp-tower/reviews/wave0-2/test-review.md`
