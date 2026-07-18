# Tower agent tools contract

[provenance: handoff §13.3/§13.6, review D-TA.*, runtime facade contract]

Nine tools form one semantic API. Their JSON input/output definitions live in
`crates/codegen/xai-grok-app-server-protocol/schemas/tower-tools.schema.json`.
In-process and MCP registrations use those same descriptors and the same
`GrokRuntimeFacade`; neither adapter may reinterpret success or errors.

## Operations

| Tool | Purpose | Mutating | Idempotency |
|---|---|---:|---|
| `tower_agent_list` | list visible local sessions | no | n/a |
| `tower_agent_start` | create a top-level peer session | yes | required |
| `tower_agent_send` | start/send a turn to a peer | yes | required |
| `tower_agent_history` | read full or last history with cursor/byte cap | no | cursor-based |
| `tower_agent_resume` | restore an archived/idle peer handle | yes | required |
| `tower_agent_wait` | poll/replay events after a cursor | no | cursor-based |
| `tower_agent_interrupt` | interrupt named active turn | yes | required |
| `tower_agent_archive` | detach/archive peer without deleting transcript | yes | required |
| `tower_agent_status` | obtain state/revision | no | n/a |

Every output is a structured object. Mutations return `operationId`, state and
target identity; textual “done” is never success. Retrying a mutation with the
same key and canonical-equivalent input returns the original result. Reusing a
key with different input returns `idempotency_conflict`.

## Per-tool wire contracts

The checked-in JSON Schema is normative for field optionality and limits. The
descriptions below lock semantic behavior not expressible in structural schema.

### `tower_agent_list`

Filters: workspaceRoot, agentType, status and includeArchived. `pageSize`
defaults 50, maximum 100. Cursor is opaque, Tower-instance/filter-bound and
orders rows by updatedAt descending then Session ID. Result rows expose Session
ID, agent type, workspace, status, residency, active Turn, timestamp and a
redacted safe summary. Invalid/foreign cursor is an error, not an empty result.

### `tower_agent_start`

Requires workspaceRoot, agentType and idempotencyKey. Model, provider binding
and sandbox mode are optional overrides; parent values are defaults only. The
runtime validates workspace/trust/sandbox and agent profile before creating the
one actor. Result identifies operation and Session. It never returns provider
credentials. Retrying same input/key returns the original Session.

### `tower_agent_send`

Accepts structured input blocks and explicit mode `new_turn|steer_active`.
`new_turn` requires null/absent turnId and uses actor queue rules;
`steer_active` requires exact active turnId. The tool never guesses from current
state. Pending Interaction or terminal target returns a stable error. Input is
bounded by the shared 1 MiB message limit.

### `tower_agent_history`

Mode `full` pages forward after eventSeq; mode `last` returns at most lastItems
ending at the current boundary. `maxBytes` is mandatory and at most 1 MiB.
Output states historyEpoch, nextEventSeq, truncated and redacted. Bearers,
provider secrets, environment secrets, hidden reasoning and private tool fields
are removed before byte accounting. Epoch/cursor mismatch requires resnapshot.

### `tower_agent_interrupt`

Targets exact Session + Turn. First valid call requests actor interruption and
returns an operation. Same key/input returns it again. A new key after terminal
state returns `turn_not_active` with safe state; it never interrupts a newer Turn.

### `tower_agent_resume`

Only dormant active Sessions can become resident. Concurrent resumes converge
on one SessionActor. Resume preserves Session ID/epoch unless continuity was
independently invalidated. Archived/dead Sessions are not silently recreated.

### `tower_agent_archive`

Archive removes the Session from default active listings and drains/detaches its
actor. Canonical transcript remains. It is not delete/purge. Busy-session policy
must either reject or explicitly interrupt according to runtime configuration;
the adapter never chooses ad hoc.

### `tower_agent_status`

Returns the same safe row shape as list, including residency and active Turn.
It performs no resume and reveals existence only after ACL/authority validation.

### `tower_agent_wait`

Waits after a named epoch/eventSeq for event, terminal state, Interaction,
timeout or resync. Timeout is 1..300000 ms. It holds no registry/actor lock while
awaiting. Output includes wakeReason and next cursor; timeout is successful empty
observation, not `operation_timeout`.

## ACL

Default allowlist is exactly agent type `orchestrator`. Built-in `build`,
`explore`, `review` and unspecified agents are denied. A custom agent may opt in
with explicit `tower_access = true`; inheritance, capability `all`, prompt text
or model name never implies access. ACL is evaluated before argument lookup can
reveal target existence. External MCP clients are authorized by Tower bearer,
not by an agent type; their full-control risk is covered by the security contract.

| Agent type | list/status/history/wait | start/send/resume/interrupt/archive | Reason |
|---|---:|---:|---|
| `orchestrator` | allow | allow | locked default control profile |
| `build` | deny | deny | ordinary execution agent |
| `review` | deny | deny | read capability does not imply Tower authority |
| `repo-explore` / `explore` | deny | deny | exploration remains session-local |
| `architect` | deny | deny | planning role is not control authority |
| `general` / unknown | deny | deny | fail-closed default |
| custom `tower_access=true` | allow | allow | explicit profile opt-in |

## Stable errors

`tower_acl_denied`, `invalid_arguments`, `session_not_found`,
`turn_not_found`, `session_archived`, `turn_not_active`, `interaction_pending`,
`idempotency_conflict`, `cursor_too_old`, `epoch_mismatch`, `resync_required`,
`operation_timeout`, `tower_draining`, `runtime_unavailable`, `internal_error`.
Errors include `retryable` and safe details, never secrets or hidden target data.

## Registration and parity

Local orchestrators receive descriptors directly from
`xai-grok-tower-tools`. The product MUST NOT inject its own MCP server into its
local MCP client config. `xai-grok-mcp-server` exposes the identical descriptors
to external clients. Differential tests execute each fixture against both
adapters and compare normalized output/error objects.

Required tests: `all_nine_descriptors_have_input_and_output_schema`,
`builtin_non_orchestrators_are_denied`, `custom_explicit_opt_in_is_allowed`,
`acl_does_not_leak_target_existence`, `mutation_retry_returns_original_result`,
`mcp_and_in_process_outputs_are_equal`, and `local_composition_has_no_mcp_loop`.

MCP descriptors use exactly the nine names, per-tool human descriptions above,
and the corresponding `$defs/<name>_input` as `inputSchema`. If the MCP library
supports output schema, it uses `$defs/<name>_output`; otherwise conformance
validates returned structured content against it.

Parity cases for every tool: happy result; invalid field; unauthorized/ACL deny;
not-found after authorization; idempotent retry where applicable; runtime
unavailable; oversized input; redaction canary; and normalized error equality.

## Limits and future peer messaging

`wait.timeoutMs` is 1..300000; send text and total tool input fit the global
1 MiB limit. MVP has no inbox, broadcast or multi-host host bridge. A future
peer-messaging epic consumes session identity, cursor, ACL and facade contracts;
it cannot add a parallel registry or a `tower_agent_hub`.

## Examples

```json
{"tool":"tower_agent_start","arguments":{"workspaceRoot":"/work/a","agentType":"build","idempotencyKey":"start-a"}}
```

```json
{"operationId":"op_01","state":"completed","sessionId":"session_01"}
```

```json
{"tool":"tower_agent_wait","arguments":{"sessionId":"session_01","afterEventSeq":"12","timeoutMs":30000}}
```

### Multi-session swarm orchestration

```json
{"tool":"tower_agent_start","arguments":{"workspaceRoot":"/work/a","agentType":"build","model":null,"providerBinding":null,"sandboxMode":null,"idempotencyKey":"start-worker-a"}}
{"tool":"tower_agent_start","arguments":{"workspaceRoot":"/work/b","agentType":"review","model":null,"providerBinding":null,"sandboxMode":null,"idempotencyKey":"start-reviewer-b"}}
{"tool":"tower_agent_send","arguments":{"sessionId":"session_a","input":[{"type":"text","text":"Implement the protocol fixture."}],"mode":"new_turn","turnId":null,"idempotencyKey":"send-worker-a"}}
{"tool":"tower_agent_send","arguments":{"sessionId":"session_b","input":[{"type":"text","text":"Review the protocol contract."}],"mode":"new_turn","turnId":null,"idempotencyKey":"send-reviewer-b"}}
{"tool":"tower_agent_wait","arguments":{"sessionId":"session_a","historyEpoch":"epoch_a","afterEventSeq":"0","timeoutMs":30000}}
{"tool":"tower_agent_wait","arguments":{"sessionId":"session_b","historyEpoch":"epoch_b","afterEventSeq":"0","timeoutMs":30000}}
```

The orchestrator polls each Session independently; no cross-Session event order
or direct peer inbox is implied. Follow-up uses another `tower_agent_send` via
the Tower control plane.
