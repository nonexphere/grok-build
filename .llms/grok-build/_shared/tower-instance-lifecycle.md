# Tower instance and session lifecycle

Tower is the promoted leader/registry boundary. A Tower instance has a stable
`instance_id`, endpoint, bearer-token file and state root. It may host many
sessions and workspaces; a machine may run many explicitly isolated Towers.

## State layout and permissions

```text
~/.grok-oss/towers/<instance-id>/
├── instance.json          # 0600, atomic replace
├── control.token          # 0600, 256-bit random bearer
├── app-server.sock        # Unix 0600 when available
├── sessions/              # mapping metadata, not duplicate transcripts
├── projections/           # rebuildable state only
└── logs/                  # redacted structured logs
```

Default `instance_id` is `default`; `--tower <id>` selects another. IDs match
`[a-z0-9][a-z0-9._-]{0,63}`. Symlinks in the Tower root or token path are
rejected. Session transcripts remain in the existing canonical session-file
location; Tower metadata stores IDs and paths but never copies authoritative
conversation state.
[provenance: handoff §13.1/§13.11, shell leader/session storage code, review D-TW.*]

## Default instance selection algorithm

1. Explicit CLI `--tower <id>` wins.
2. Otherwise `GROK_OSS_TOWER` is used if present and valid.
3. Otherwise select literal `default`; there is no “last used” mutable pointer.
4. Resolve `GROK_OSS_HOME` through existing product path policy, append
   `towers/<id>`, and reject symlink/non-owned unsafe components.
5. Derive the Unix socket from the instance root; where a path-length limit
   applies, use a collision-resistant hash of the canonical root under the
   existing secure runtime directory and retain the full root in metadata.
6. Open `<root>/instance.lock` without following symlinks. The lock protects
   spawn/metadata publication, not the whole daemon lifetime.
7. Validate metadata instance ID, canonical state root, endpoint owner/mode,
   protocol version and health response before returning a handle.

## Connect-or-spawn state machine

```text
DISCOVER -> CONNECTING -> READY
   |           | failure
   | absent    v
   +------> LOCKING -> SPAWNING -> HEALTHCHECK -> READY
                  |       |             |
                  +-------+-------------+-> FAILED (actionable error)
READY -> DRAINING -> STOPPED
```

Discovery reads metadata, validates owner/perms, and probes health. Only one
contender obtains the byte-range/file lock. Losers retry discovery with bounded
jitter. A stale PID alone never authorizes deletion: endpoint health, process
identity and lock ownership must agree. Spawn timeout is 15s; health probes use
250ms initial backoff capped at 2s. Failure preserves diagnostic metadata and
never reports READY.

Failure codes: `tower_metadata_invalid`, `tower_permissions_unsafe`,
`tower_lock_timeout`, `tower_spawn_failed`, `tower_health_timeout`,
`tower_endpoint_in_use`, `tower_version_incompatible`, `tower_draining`.

## Multi-instance isolation

Instance IDs produce disjoint roots, endpoints, locks, tokens, metrics labels
and projection databases. A caller must name the selected Tower or accept
`default`; ambient “last used Tower” state is forbidden. Cross-instance session
attach is rejected unless a later explicit import protocol exists.

## Session lifecycle

```text
STARTING -> IDLE <-> RUNNING -> AWAITING_INTERACTION
   |          |          |             |
   v          v          v             v
 FAILED     ARCHIVED   INTERRUPTING -> IDLE
```

Session state is derived from the canonical actor and files. Tower registry
rows are indexes and may be rebuilt. Archive detaches active control and closes
subscriptions after a terminal notification; it does not delete transcripts.
Parent cancellation does not archive peer sessions by default.

Residency is orthogonal to durable lifecycle:

| Durable state | Residency | Meaning |
|---|---|---|
| active | resident | exactly one live actor/handle in this Tower |
| active | dormant | canonical files exist and may be resumed; no actor |
| archived | dormant | retained transcript, excluded from default list |
| failed | resident or dormant | last operation failed; safe summary retained |
| dead | none | metadata points to irrecoverable/missing canonical state; diagnostic only |

Idle release changes resident→dormant without changing Session status, ID,
history epoch or transcript. Archive is explicit and never an eviction synonym.

## Workspace, symlinks and sandbox

Workspace roots are canonicalized once, retained as display + resolved paths,
and checked against sandbox/trust policy before actor creation. A symlink race
between authorization and actor start fails closed. Peers inherit parent cwd,
sandbox, model and provider binding only as defaults; explicit overrides pass
the same validation as CLI-created sessions.

## Residency and resource policy

MVP has no hard global session cap. Idle actors may be released while session
identity remains discoverable; reopening reconstructs through existing session
files. Telemetry records current/peak Towers, sessions, active turns, resident
actors, queue depth, spawn latency and failed lifecycle transitions. Future
knobs (`max_resident_sessions`, `idle_release_after`) are inert until specified;
they MUST NOT silently change MVP behavior.

“No product session cap” does not mean unbounded resource admission. The runtime
MUST enforce safety budgets for resident actors, active Turns, pending loads,
connection queues, replay bytes, file descriptors and minimum free disk. A
budget refusal is explicit, retryable where appropriate, observable, and never
archives or deletes a Session. Defaults are derived conservatively from process
and OS capacity; configured values are instance-scoped. Dormant persisted
Sessions remain listable even when a residency budget is exhausted.

Soft telemetry records current and peak values for: Tower instances, registered
Sessions, resident actors, dormant Sessions, active Turns, pending Interactions,
per-connection outbound queue depth, replay buffer bytes, process RSS, open file
descriptors, tasks/threads, connect/spawn latency and lifecycle failures. Labels
are bounded (`instance_id`, transition, outcome); Session IDs/workspaces are logs
only when safe and are never metric labels. Peak values reset only at process
start and are included in authenticated readiness diagnostics.

## Existing leader protocol preservation list

Promotion does not reinterpret existing ACP/leader bytes. Characterization
fixtures freeze:

- length/framing and JSON-RPC envelope accepted by leader transport;
- `initialize`/capability negotiation order used by ACP clients;
- existing connection handshake, leader discovery metadata and socket mode;
- roster snapshot/delta method names and field casing;
- `session/new`, `session/load`, prompt and cancel request shapes;
- error envelope codes/messages relied on by pager/dashboard;
- EOF, reconnect, ping/health and stale-leader recovery behavior;
- stdout/stderr separation for stdio agent paths.

The new native App Server does not reuse those bytes by coincidence: it calls a
thin adapter behind the same actor/registry. Any intentional leader byte change
belongs to a separate compatibility migration, not Tower promotion.

## Public Rust API sketch

```rust
pub struct TowerInstanceId(/* validated */ String);
pub struct TowerHandle { /* cloneable command sender, no mutable actor */ }

impl TowerHandle {
    pub async fn connect_or_spawn(config: TowerConfig) -> Result<Self, TowerError>;
    pub async fn list_sessions(&self, filter: SessionFilter) -> Result<Vec<SessionSummary>, TowerError>;
    pub async fn open_session(&self, request: OpenSession) -> Result<SessionHandle, TowerError>;
    pub async fn resume_session(&self, id: SessionId) -> Result<SessionHandle, TowerError>;
    pub async fn archive_session(&self, id: SessionId, key: IdempotencyKey) -> Result<(), TowerError>;
    pub async fn drain(&self, deadline: Instant) -> Result<DrainReport, TowerError>;
}
```

`SessionHandle` is the existing runtime handle or a narrow wrapper around it;
Tower never exposes mutable SessionActor internals.

## Actor safety and shutdown

Registry maps IDs to handles, never mutable actor structs. Each actor serializes
turn mutations; registry and projection readers may be concurrent. Shutdown
enters DRAINING, rejects new session/turn starts, allows a configurable 10s
grace, interrupts remaining turns, flushes projection cursors and atomically
marks STOPPED. Crash restart changes `epoch`, preserves session IDs, and forces
subscribers to replay or resnapshot.

## Characterization gates

Before promotion, byte-level fixtures lock existing leader discovery and
handshake: `leader_metadata_bytes_are_preserved`,
`connect_or_spawn_has_single_winner`, `stale_pid_does_not_delete_live_endpoint`,
`two_towers_isolate_token_endpoint_and_state`,
`archive_preserves_canonical_session_file`, and
`restart_changes_epoch_without_changing_session_id`.

Planned file/assertion placement:

| File | Assertion names |
|---|---|
| `xai-grok-shell/src/leader/protocol.rs` tests | `leader_initialize_fixture_bytes`, `roster_delta_fixture_bytes` |
| `xai-grok-shell/src/leader/client.rs` tests | `client_reconnect_preserves_handshake_order` |
| `xai-grok-shell/src/leader/lock.rs` tests | `connect_or_spawn_has_single_winner`, `stale_pid_is_not_sufficient` |
| `xai-grok-tower/tests/lifecycle.rs` | `two_instances_are_fully_isolated`, `restart_epoch_semantics` |

## Runtime/thread-safety allocation

The async runtime owns connection tasks, registry command processing and actor
command senders. Each SessionActor remains its single mutation owner. Blocking
filesystem discovery runs outside actor/event hot paths. Subscription writer
tasks own their bounded queue and cannot await while holding registry locks.
TowerHandle clones are `Send + Sync` command senders; SessionActor is not made
shared merely to satisfy an adapter.
