# Canonical glossary and identity

This glossary is normative for all programs. Terms include an anti-definition
because several adjacent runtime concepts are superficially similar.
[provenance: handoff §13.0, runtime code, review D-00.2]

| Term | Definition | Explicitly is not |
|---|---|---|
| Session | Canonical persisted conversation/work unit with stable ID, workspace and ordered Turns | a transport connection, UI tab, subagent child, Codex-named object, or resident actor |
| Turn | One user/steer/resume/synthetic round within exactly one Session | a JSON-RPC request, whole Session, tool call, or Tower operation |
| Item | Typed observable unit within a Turn, revisioned and projected to clients | raw internal event, hidden reasoning, arbitrary log line, or provider credential |
| Tower | Named daemon/control plane that promotes the existing leader and hosts a registry of Sessions | a separate inbox/hub tool, second runtime, agent type, or MCP client |
| Tower instance | One isolated Tower identity with state root, endpoint, token, lock and epoch | a Session or operating-system-wide singleton |
| Resident | Session currently backed by one live authoritative SessionActor | persisted transcript, “recently listed”, or proof a Turn is active |
| Dormant | Persisted resumable Session without a resident actor | archived, deleted, failed, or a process zombie |
| Archived | Session detached from active management but whose canonical transcript remains | delete, purge, dormant eviction, or terminal Turn |
| Controller | Authenticated connection holding the current interaction lease | model agent, bearer token itself, Tower owner, or permanent administrator |
| Controller lease | Revisioned, expiring right to answer a specific Interaction | token scope, filesystem lock, actor ownership, or permission policy |
| Interaction | Server-initiated approval, question or MCP elicitation with stable identity and one resolution | notification-only prompt, JSON-RPC request ID, user message, or Tower inbox |
| IdempotencyKey | Client-selected mutation key scoped to authority/method and canonical input | request ID, Session ID, retry counter, deduplication of read methods |
| eventSeq | Strict per-Session/per-historyEpoch delivery order | Item revision, wall-clock time, global Tower order, or database row ID |
| historyEpoch | Opaque continuity identity for replay cursors | process PID, protocol version, Session ID, or monotonically ordered number |
| revision | Per-entity mutation counter | eventSeq, optimistic lease revision across entities, or timestamp |
| AgentType | Runtime profile name used by capability/ACL policy | model ID, provider, bearer authority, Session role, or free-form prompt text |
| CredentialId | Opaque identity of stored provider credentials | credential value, catalog slug, bearer token, or AgentType |
| ProviderBinding | Immutable structured reference `{providerId, credentialId, modelId, backend, bindingRevision}` captured by a Session and snapshotted for every Turn | secret storage, mutable alias, opaque string, or control-plane bearer |
| Runtime facade | Single typed port from adapters to existing leader/SessionActor behavior | second actor, wire protocol, persistence authority, or canned-success fake |
| Projection | Rebuildable client/history representation derived from canonical runtime/session files | execution truth, credential store, SessionActor state, or authoritative transcript |

## Identity rules

Session IDs preserve the existing persisted Session UUID where available. Turn
IDs reuse stable prompt/run identity where the runtime exposes one. Item IDs are
derived from stable source identity + history epoch + kind; rebuild cannot
renumber already published identities. IDs are opaque on the wire and bounded to
128 bytes.

Native methods use `session/*`, `turn/*`, `item/*`. Codex terminology may appear
only in the isolated compatibility mapping file and citations; it never enters
native Rust type names, MCP tools, SDK API or product-facing examples.

`eventSeq`, entity `revision`, binding revision and replay cursor counters are
internal `u64` values serialized as canonical decimal strings. JSON numbers are
invalid for these fields so JavaScript cannot silently lose precision.
