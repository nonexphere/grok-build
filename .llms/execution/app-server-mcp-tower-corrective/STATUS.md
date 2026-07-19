# STATUS — Corrective App Server / MCP / Tower

| Field | Value |
|---|---|
| Branch | `goblin-implement-epic-tree` |
| Wave | **Round-5 local fixes DONE** (R5-01..R5-10); checkpoint via commit (R5-11) |
| Date | 2026-07-19 |
| Composition | `ShellSessionActorRuntime::new` (no product echo — R5-01) |
| Product WS | `GROK_OSS_APP_SERVER=1` + `app-server-ws` |
| Product MCP | `GROK_OSS_MCP=http` / `GROK_OSS_MCP_HTTP=1` + `mcp-streamable-http` |

## Classification (post R5)

| Class | Items |
|---|---|
| **DONE (local)** | R5-01 product path no echo factory; R5-02 residency hard errors consistent on start/resume/idempotent; R5-03 Won/Existing cross-runtime claim; R5-04 unique rotating history_epoch; R5-05 gap replay + exact pagination; R5-06 MCP event buffer cap + expired cursor; R5-07 TTL eviction interrupts active turns; R5-08 no secret fingerprint in logs; R5-09 shared delivery hub + ask_user_question dual-wait; R5-10 ledger honesty |
| **EXTERNAL_CREDENTIAL** | Live `spawn_session_on_thread` with real credentials (turns in product without fixture) |
| **EXTERNAL** | TLS remote threat acceptance; npm publish |
| **PROCESS** | PR open (base `goblin`) when user requests |

## Product honesty

- Product composition is **storage-backed + fail-closed for turns** without a real spawn factory.
- `experimental_local_turn_spawn` is **test/fixture-only** (never product composition).
- `respond_interaction` delivers into the shared hub; live ask_user_question parks on that hub (R5-09). Full reverse-request dual-wait for every permission site remains proportional follow-on when real actor is product-wired.

## GREEN (Round-5)

- `r5_runtime_correctness` (3): cross-runtime idempotency, epoch rotate, gap replay  
- `r4_runtime_correctness` (4)  
- `c6_respond_interaction` (10)  
- `c7_conformance` (18)  
- `c1_shell_port` / `c1_production_spawn` / `c1_turn_lifecycle`  
- MCP streamable HTTP + `r5_mcp_event_buffer_cap` / `r5_mcp_ttl_eviction`  
- pager-bin `r4_startup_token_status` + composition initialize  
