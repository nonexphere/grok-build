# Session protocol v1 — events, replay and interactions

[provenance: handoff §13, review D-SP.12..16/D-AP.1..5]

## Notification payloads

Every lifecycle notification carries `sessionId`, `historyEpoch`, `eventSeq` and
`timestampMs`. Item deltas additionally carry `turnId`, `itemId`, item revision,
delta text and stream. Lifecycle names are:

| Notification | Payload | Terminal? | Coalescible? |
|---|---|---:|---:|
| `session/created` | EventMeta + Session | no | no |
| `session/updated` | EventMeta + full Session | maybe | only superseded adjacent updates |
| `session/archived` | EventMeta + full Session | yes for residency | no |
| `turn/created` | EventMeta + Turn | no | no |
| `turn/updated` | EventMeta + full Turn | maybe | only non-terminal adjacent updates |
| `item/started` | EventMeta + full Item | no | no |
| `item/delta` | EventMeta + target/revision/delta/stream | no | adjacent same target+stream only |
| `item/completed` | EventMeta + full final Item | yes | no |
| `interaction/requested` | EventMeta + Interaction | no | no |
| `interaction/resolved` | EventMeta + resolution | yes | no |
| `subscription/resyncRequired` | subscription + safe cursor reason | yes for subscription | no |
| `server/draining` | instance ID + deadline | no | no |

Coalescing preserves the final content and revision, and the emitted eventSeq
represents the coalesced envelope. The server never drops lifecycle or terminal
events. Hidden reasoning is never a raw delta; only `reasoning_summary` approved
by the runtime projector may cross the protocol.

## Ordering model

- `eventSeq`: strictly increasing `u64` per Session/historyEpoch.
- Item `revision`: strictly increasing per Item, independent of eventSeq.
- Session/Turn revisions: strictly increasing per entity.
- `historyEpoch`: opaque identity of continuity. Restart changes it only when
  replay continuity cannot be proven; rebuild that preserves event identity may
  retain it.
- Cross-Session order has no meaning. Concurrent Sessions can interleave on a
  connection while each Session remains ordered.

Invalid examples:

```text
epoch_1: eventSeq 41, 43       INVALID: gap 42, reconnect/resync required
epoch_1: item revision 7, 6    INVALID: stale item update, ignore and diagnose
epoch_1 cursor used in epoch_2 INVALID: never compare eventSeq across epochs
```

## Snapshot-then-live algorithm

```text
subscribe(session, requested_epoch, after):
  authorize session without leaking its existence
  tap = registry.attach_live_tap(session)
  boundary = tap.current_event_seq()
  snapshot = history.read_epoch_and_revision(session)
  if requested_epoch != null && requested_epoch != snapshot.epoch:
      detach tap; return epoch_mismatch(snapshot.epoch)
  replay = history.read(after + 1 .. boundary)
  if replay has a gap or exceeds retention/byte limit:
      detach tap; return cursor_too_old/resync_required
  emit replay in strict eventSeq order
  emit tap.buffered_where(seq > boundary), deduplicated by seq
  return subscription boundary; continue live
```

```mermaid
sequenceDiagram
  participant C as Client
  participant P as Processor
  participant H as History
  participant L as Live tap
  C->>P: session/subscribe(epoch E, after 41)
  P->>L: attach; capture boundary 45
  P->>H: validate E; read 42..45
  H-->>P: events 42..45
  P-->>C: events 42..45
  L-->>P: buffered 46..47
  P-->>C: events 46..47
  P-->>C: result(liveFromEventSeq=48)
```

Named tests: `subscribe_has_no_snapshot_live_gap`,
`subscribe_deduplicates_boundary_event`, `epoch_mismatch_requires_snapshot`,
`slow_subscriber_receives_resync_not_silent_drop`.

## Interaction/server-request model

Interactions cover approval, question and MCP elicitation. The server sends a
JSON-RPC request with its own request ID plus an Interaction payload. The
`interactionId` is stable across transport retries; the JSON-RPC request ID is
connection-local correlation only.

```json
{"jsonrpc":"2.0","id":"server:interaction:91","method":"interaction/request","params":{"interactionId":"interaction_91","sessionId":"session_1","turnId":"turn_4","itemId":"item_8","kind":"approval","prompt":"Allow command execution?","choices":["accept","decline"],"expiresAtMs":1784377000000}}
```

Controller lease states:

```text
UNOWNED -> HELD(connection, lease_revision, deadline)
HELD -> RENEWED(same controller)
HELD -> RELEASED(disconnect/explicit/expiry)
HELD -> RESOLVED(one terminal decision)
RELEASED -> HELD(other eligible controller)
```

Only the current controller may answer. Disconnect releases the lease but never
auto-allows. Explicit policy may auto-deny; otherwise the interaction remains
pending for another controller until deadline. Duplicate same-key responses
return the original resolution; conflicting responses return
`interaction_already_resolved`.

Named tests: `request_id_is_not_interaction_id`,
`controller_disconnect_never_auto_allows`, `only_lease_holder_can_respond`, and
`interaction_resolution_is_exactly_once`.

## History source and projection

MVP authority remains canonical session JSONL/files. Projection SQLite, when
introduced in `30/v1-05`, is rebuildable and may index eventSeq/cursors but does
not become execution truth. If projection and session files disagree, rebuild
or return `history_unavailable`; never invent missing events. Cursor tokens bind
Tower instance, Session, epoch, filter set, boundary and expiry.
