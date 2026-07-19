# Corrective blockers (reconciled Round-5)

| ID | Class | Description |
|---|---|---|
| EXT-TLS | EXTERNAL | TLS + threat acceptance for non-loopback production (D-SEC.13) |
| EXT-CREDS | EXTERNAL_CREDENTIAL | Live model/provider credentials for full `spawn_session_on_thread` on product path |
| EXT-NPM | EXTERNAL | npm publish/naming |
| PROC-PR | PROCESS | User-authorized PR to base `goblin` (never `main`) when ready |

## Closed local blockers (Round-4 + Round-5)

- Secret logging raw token, WS empty-bearer fail-close, tower fail-fast  
- Durable + **cross-runtime** idempotency (Won/Existing)  
- Unique rotating `history_epoch`; gap-safe replay pagination  
- MCP max sessions, TTL **with interrupt**, event buffer cap  
- Product path **no offline echo** (R5-01)  
- Residency hard-error consistency (R5-02)  
- `respond_interaction` hub shared with live actor reverse-request park (R5-09 ask_user_question)  

## Not a local open item

- Full live-model product turns require EXT-CREDS — not fakeable as DONE.
