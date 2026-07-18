# Session protocol v1 — method contracts

Status: experimental wire freeze for scaffold. Native method names use Session;
Codex naming exists only in the isolated mapping contract. All mutations require
an idempotency key and all requests require completed initialization.
[provenance: handoff §13, review D-SP.8..11, seed spec adapted Thread→Session]

## Inventory

| Method | Side | Params schema | Result | Idempotent | State/authority lock | Stable errors |
|---|---|---|---|---:|---|---|
| `initialize` | client | `initializeParams` | `initializeResult` | no | connection initialize gate | `already_initialized`, `protocol_version_unsupported` |
| `session/start` | client | `sessionStartParams` | Session | key | Tower registry + workspace authorization | `invalid_workspace`, `unauthorized`, `idempotency_conflict`, `tower_draining` |
| `session/resume` | client | `sessionTargetMutation` | Session | key | registry entry lifecycle | `session_not_found`, `session_not_dormant`, `idempotency_conflict` |
| `session/fork` | client | `sessionForkParams` | Session | key | source snapshot + destination registry | `session_not_found`, `invalid_workspace`, `fork_not_available` |
| `session/read` | client | `sessionReadParams` | snapshot/turns/items | read | snapshot boundary only | `session_not_found`, `page_too_large`, `history_unavailable` |
| `session/list` | client | `sessionListParams` | sessions/page cursor | read | registry snapshot | `invalid_cursor`, `page_too_large` |
| `session/archive` | client | `sessionTargetMutation` | operation result | key | registry lifecycle | `session_not_found`, `session_busy`, `idempotency_conflict` |
| `session/subscribe` | client | `subscribeParams` | subscription boundary | cursor | history epoch + event tap | `session_not_found`, `epoch_mismatch`, `cursor_too_old` |
| `session/unsubscribe` | client | subscriptionId | acknowledged | yes | connection subscription table | `subscription_not_found` |
| `turn/start` | client | `turnStartParams` | Turn | key | SessionActor turn queue | `session_not_found`, `session_archived`, `interaction_pending` |
| `turn/steer` | client | `turnSteerParams` | Item | key | named active Turn | `turn_not_active`, `steer_not_allowed`, `interaction_pending` |
| `turn/interrupt` | client | `turnInterruptParams` | operation result | key | named active Turn | `turn_not_found`, `turn_terminal` |
| `interaction/respond` | client | `interactionRespondParams` | operation result | key | interaction + controller lease | `interaction_not_found`, `controller_lease_required`, `interaction_already_resolved` |

“Authority lock” is an implementation ordering boundary, not a token scope. MVP
bearers are full-control and have no fine-grained scopes. [provenance: handoff R5]

## `session/start`

### Preconditions and canonicalization

1. Authentication and initialize gates pass before workspace lookup.
2. `workspaceRoot` is 1..4096 UTF-8 bytes, absolute after platform-aware parsing.
3. The runtime canonicalizes the path and applies folder trust/sandbox policy.
4. Symlink or ownership changes between authorization and actor creation abort.
5. `agentType` is optional; absence selects the runtime’s ordinary default.
6. `providerBinding` is the structured immutable public tuple
   `{providerId, credentialId, modelId, backend, bindingRevision}`. It contains
   identifiers only, rejects unknown fields, and never contains credentials.
7. The key is 8..128 bytes. Same canonical input replays the first result.
8. A created Session starts `starting`, then `ready` or `failed`; no success is
   returned before the registry has a stable Session ID and canonical file path.

Happy request:

```json
{"jsonrpc":"2.0","id":10,"method":"session/start","params":{"workspaceRoot":"/work/grok-goblin","agentType":"orchestrator","providerBinding":null,"idempotencyKey":"session-start-0010"}}
```

Happy result:

```json
{"jsonrpc":"2.0","id":10,"result":{"session":{"sessionId":"0198...","historyEpoch":"epoch_1","revision":"1","status":"ready","workspaceRoot":"/work/grok-goblin","title":null,"activeTurnId":null,"latestTurnId":null,"providerBinding":null,"createdAtMs":1784376000000,"updatedAtMs":1784376000000}}}
```

Invalid workspace:

```json
{"jsonrpc":"2.0","id":11,"error":{"code":-32020,"message":"The workspace cannot be opened.","data":{"code":"invalid_workspace","retryable":false,"field":"workspaceRoot"}}}
```

Unauthorized request returns the generic authority error before revealing path
existence:

```json
{"jsonrpc":"2.0","id":12,"error":{"code":-32001,"message":"Authentication required.","data":{"code":"unauthorized","retryable":false}}}
```

Named tests: `session_start_canonicalizes_before_actor_creation`,
`session_start_rejects_symlink_swap`, `session_start_idempotency_replays_result`,
and `unauthorized_start_does_not_reveal_workspace`.

## `session/resume`

Resume loads a dormant persisted Session into a resident actor. It does not
create a new Session ID or history epoch merely because residency changed.
Concurrent resumes with different keys converge on one actor; each operation
observes the same Session identity. Archived Sessions require an explicit future
unarchive contract and therefore return `session_archived`.

```json
{"jsonrpc":"2.0","id":20,"method":"session/resume","params":{"sessionId":"session_1","idempotencyKey":"resume-session-1"}}
```

Named test: `concurrent_resume_creates_one_resident_actor`.

## `session/fork`

Fork reads a consistent source snapshot and creates a new Session with a new ID,
new epoch and `parentSessionId` in internal metadata. It copies only canonical
session artifacts allowed by persistence policy, never Tower token/projection
state. Optional workspace override passes the same checks as `session/start`.

```json
{"jsonrpc":"2.0","id":21,"method":"session/fork","params":{"sessionId":"session_1","workspaceRoot":"/work/fork","idempotencyKey":"fork-session-1"}}
```

Named test: `fork_has_new_identity_and_consistent_source_boundary`.

## `session/read`

Read captures a Session snapshot revision and optionally returns Turns/Items no
newer than that boundary. `includeItems=true` implies `includeTurns=true`.
Large histories are paginated by the history contract; the response never
silently truncates. Provider keys, bearer values, hidden reasoning and private
tool state are redacted before byte-size accounting.

```json
{"jsonrpc":"2.0","id":22,"method":"session/read","params":{"sessionId":"session_1","includeTurns":true,"includeItems":false}}
```

Named test: `session_read_is_revision_consistent_and_redacted`.

## `session/list`

`pageSize` defaults to 50 and is 1..100. Cursor is opaque, instance-bound and
snapshot-bound; clients must not parse it. Filters apply before pagination.
Ordering is `updatedAtMs DESC, sessionId ASC`. A cursor from another Tower or
expired snapshot returns `invalid_cursor`, never an empty page.

```json
{"jsonrpc":"2.0","id":23,"method":"session/list","params":{"pageSize":50,"cursor":null,"includeArchived":false,"workspaceRoot":null}}
```

Named tests: `session_list_has_stable_tie_break_order` and
`foreign_instance_cursor_is_invalid`.

## `session/subscribe`

The result names `subscriptionId`, current `historyEpoch`,
`replayedThroughEventSeq` and `liveFromEventSeq`. Result delivery occurs only
after the processor has established the live tap and buffered the replay/live
boundary. Details and sequence diagram are in `events.md`.

All event-sequence and revision values are canonical decimal strings on the
wire. A client may open multiple subscriptions for one Session; every stream is
owned, routed and unsubscribed by `subscriptionId`.

```json
{"jsonrpc":"2.0","id":24,"method":"session/subscribe","params":{"sessionId":"session_1","historyEpoch":"epoch_1","afterEventSeq":"41"}}
```

Invalid cursor example:

```json
{"jsonrpc":"2.0","id":24,"error":{"code":-32012,"message":"The history epoch changed; read a new snapshot.","data":{"code":"epoch_mismatch","retryable":true,"currentHistoryEpoch":"epoch_2"}}}
```

## Turn concurrency rules

- A Session has one actor and one active foreground Turn.
- `turn/start` while another Turn is active queues according to existing runtime
  rules or returns `turn_already_active`; it never starts a parallel actor.
- `turn/steer` targets exactly the named active Turn. A steer becomes a
  user-message Item with kind `steer`; it cannot mutate a terminal Turn.
- `turn/interrupt` is idempotent. Repeating it after accepted interruption
  returns the original operation result. A new key against a terminal Turn
  returns `turn_terminal` with its safe status.
- A pending Interaction blocks operations the runtime cannot safely apply.

Start:

```json
{"jsonrpc":"2.0","id":30,"method":"turn/start","params":{"sessionId":"session_1","input":[{"type":"text","text":"Run tests"}],"idempotencyKey":"turn-start-0030"}}
```

Steer:

```json
{"jsonrpc":"2.0","id":31,"method":"turn/steer","params":{"sessionId":"session_1","turnId":"turn_1","input":[{"type":"text","text":"Only run the protocol tests"}],"idempotencyKey":"turn-steer-0031"}}
```

Interrupt:

```json
{"jsonrpc":"2.0","id":32,"method":"turn/interrupt","params":{"sessionId":"session_1","turnId":"turn_1","idempotencyKey":"turn-interrupt-0032"}}
```

Named tests: `turn_start_uses_existing_actor_queue`,
`steer_rejects_terminal_turn`, `interaction_blocks_conflicting_turn_mutation`,
and `interrupt_retry_returns_original_operation`.
