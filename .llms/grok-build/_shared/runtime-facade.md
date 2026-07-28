# GrokRuntimeFacade contract

[provenance: handoff §13.1/§13.9, shell SessionActor/leader evidence, review D-RF.*]

The facade is the single semantic boundary used by App Server, MCP server and
in-process Tower tools. It wraps the existing leader/session registry and
`SessionActor`; it never instantiates an alternative actor.

## Required trait surface

```rust
#[async_trait]
pub trait GrokRuntimeFacade: Send + Sync {
    async fn list_sessions(&self) -> Result<Vec<Session>, RuntimeError>;
    async fn read_session(&self, params: SessionReadParams) -> Result<SessionReadResult, RuntimeError>;
    async fn start_session(&self, params: SessionStartParams) -> Result<Session, RuntimeError>;
    async fn resume_session(&self, params: SessionResumeParams) -> Result<Session, RuntimeError>;
    async fn fork_session(&self, params: SessionForkParams) -> Result<Session, RuntimeError>;
    async fn archive_session(&self, params: SessionArchiveParams) -> Result<(), RuntimeError>;
    async fn start_turn(&self, params: TurnStartParams) -> Result<Turn, RuntimeError>;
    async fn steer_turn(&self, params: TurnSteerParams) -> Result<Item, RuntimeError>;
    async fn interrupt_turn(&self, params: TurnInterruptParams) -> Result<(), RuntimeError>;
    async fn respond_interaction(&self, params: InteractionResponseParams) -> Result<(), RuntimeError>;
    async fn replay(&self, cursor: SubscribeParams) -> Result<ReplayPage, RuntimeError>;
}
```

`ReplayPage` is byte/event bounded and contains `events`,
`replayed_through` and optional `next_cursor`; an unbounded `Vec` is forbidden.
The scaffold declares this full shape and has no implementation. Streaming live
events will add a separate subscription receiver without changing replay cursor
semantics; no method returns canned success.

The composition root is `xai-grok-pager-bin`: it constructs the Shell adapter
and injects it. Tower never imports Shell, and Shell never delegates actor
semantics back through App Server or MCP.

## Mapping to existing operations

| Facade operation | Existing owner | Rule |
|---|---|---|
| list/read/start/resume/fork/archive session | leader roster + session-file discovery | preserve canonical session identity |
| start/steer/interrupt turn | `SessionHandle` / `SessionActor` commands | one command, one actor owner |
| respond interaction | permission/elicitation path | controller lease checked before actor call |
| replay/live subscription | leader event fan-out + projection | attach snapshot boundary before live stream |

## Event projection

| Runtime event | Protocol Item/notification |
|---|---|
| user prompt accepted | `user_message` item |
| assistant delta/final | coalesced `agent_message` item updates |
| tool start/result | `tool_call` / `tool_result` |
| permission or elicitation | `interaction_request` |
| turn state change | `turn/updated` notification |
| runtime failure | redacted `error` item |

Provider credentials, bearer tokens, raw auth headers and environment secrets
are removed before projection. Unknown runtime events become a redacted
`unsupported_runtime_event` diagnostic; they are not silently dropped.

## Concurrency and test doubles

Exactly one facade adapter may hold each `SessionHandle`; cloneable handles send
commands but do not duplicate actor state. A fake MUST model revisions, event
sequence, epoch changes, idempotency replay, interaction blocking and interrupt
races. A hashmap returning canned success is not conformant.

Named tests: `single_actor_owns_turn_mutation`,
`all_adapters_observe_identical_item_projection`,
`runtime_secrets_never_cross_projection`, and
`fake_matches_epoch_and_idempotency_semantics`.
