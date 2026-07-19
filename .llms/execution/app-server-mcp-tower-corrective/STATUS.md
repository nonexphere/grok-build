# STATUS — Corrective App Server / MCP / Tower

| Field | Value |
|---|---|
| Contract | `.llms/tasks/20260718-correct-app-server-mcp-tower-execution.md` |
| Branch | `goblin-implement-epic-tree` |
| Wave | **C0 COMPLETE → C1 authorized (GO)** |
| Architecture review | `reviews/c0/architecture-review.md` — **GO for C1** |
| Matrix | `waves/c0-requirement-matrix.md` (120 tasks) |
| Command map | `waves/c0-session-actor-command-map.md` |
| Handoffs | `handoffs/` |

## GLM subagents

| ID | Status | Result |
|---|---|---|
| C0-A matrix | done | 77 PASS / 19 PARTIAL / 13 OPEN / 3 SKIP / 8 HUMAN; 19 reopened |
| C0-B command map | done | 11 facade methods → Shell file:fn |
| C0-C arch review (v1) | done | NO-GO (stale, pre-map) |
| C0-C arch review (v2) | done | **GO for C1** |
| C1-D shell port | **implemented — pending validation** | real `ShellSessionActorRuntime` over JSONL storage; composition root switched off FakeRuntime; actor-backed methods PARTIAL (`unsupported`) |
| C1-E/F reviews | staged | after D validated |

## Blocker for COMPLETE (not for C1 start)
Full product path still FakeRuntime until C1-D lands.

## C1-D status
- **Implemented:** real `ShellSessionActorRuntime` (`app_server_runtime/shell_session_actor_runtime.rs`)
  maps storage-backed facade methods to real Shell symbols; composition root
  (`app_server_composition.rs`) injects the real port (TempDir test seam).
  `FakeRuntime` retained for unit/conformance. Dormant `project_active_session_row`
  stub removed.
- **PARTIAL (honest gaps):** `start_turn`/`steer_turn`/`interrupt_turn`/
  `respond_interaction`/`archive_session` return `unsupported` (actor fixture
  gap / R6 product decision / R10 channel design). `read_session` turns/items
  (R2) and full `replay` projection (R11) deferred. See
  `waves/c1-shell-port.md` §7.
- **Validation:** tests written (`tests/c1_shell_port.rs` + composition tests).
  **PENDING execution** — this subagent had no command-execution tool; the
  fresh reviewer must run the commands in `tests/c1/README.md` and capture
  GREEN logs. Do not mark C1-D PASS until real-adapter GREEN is evidenced.

## Next
Fresh review of C1-D (C1-E code review / C1-F test review) with the validation
commands run, then actor-fixture follow-on for the PARTIAL turn/interaction
methods.
