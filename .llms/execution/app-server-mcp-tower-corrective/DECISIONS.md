# DECISIONS — Corrective App Server / MCP / Tower

| ID | Decision | Rationale | Status |
|---|---|---|---|
| D-R6 | `archive_session` = reversible **hide** via `archived.flag` (not delete) | Review residual C7-B F-1 + R6: keep data on disk, project `SessionStatus::Archived`; default list filters archived | **DONE** (2026-07-19) |
| D-R10 | `respond_interaction` is a delivery channel into shared hub + pending map; no second permission engine | R5-09: hub shared via `SessionHandle::interaction_delivery_hub`; ask_user_question dual-waits hub vs ACP | **DONE** local (permission reverse-request dual-wait proportional follow-on when product wires real actor) |
| D-SPAWN | Product composition uses `ShellSessionActorRuntime::new` (no echo); `experimental_local_turn_spawn` is test/fixture-only; full `spawn_session_on_thread` is EXTERNAL-cred | R5-01: never present offline echo as product | **DONE** local; EXTERNAL for live model |
| D-ENV-TOWER | Precedence: explicit > `GROK_OSS_TOWER` > legacy `GROK_TOWER_INSTANCE` > `default` | Aligns with GOBLIN/AGENTS canonical env name | **DONE** |
| D-WS-COMPOSE | Product WS via `GROK_OSS_APP_SERVER=1` + feature `app-server-ws` on existing `agent serve` | Minimal CLI surface; loopback default | **DONE** (TLS HUMAN) |
| D-MCP-COMPOSE | Product MCP HTTP via `GROK_OSS_MCP_HTTP=1` + feature `mcp-streamable-http` | Distinct env from WS; fail-closed bearer | **DONE** (TLS HUMAN) |
| D-BYOK | Register OpenRouter/Groq/Cloudflare offline AuthProviders | Spec verticals; live smoke opt-in only | **DONE** offline; composition Turn PARTIAL |
| D-HIST | Single shared projector over `updates.jsonl` for read_session + replay | No second buffer | **DONE** with honest PARTIAL for events Shell never writes |
| D-TLS | Cleartext non-loopback experimental/unsafe only | D-SEC.13 HUMAN threat acceptance | **HUMAN** |
| D-NPM | npm publish/naming | External | **HUMAN** |
| D-LIVE-PROV | Live provider tests never PASS when creds missing | SKIP only | **POLICY** |
