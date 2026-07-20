# Tasks — token scopes and TLS release

- [ ] MCP105-01 [AUTH] Freeze token record/schema: safe ID, hash, scopes, created/revoked/expiry metadata and instance binding; validate no raw secret persistence beyond secure token material.
- [ ] MCP105-02 [AUTH] Implement one-time create, safe list, revoke, rotate and revoke-connections in shared auth core/CLI; run named lifecycle tests and accept no token in argv/log/default output.
- [ ] MCP105-03 [SCOPES] Define read, session-write, turn-control, interaction-control, admin and full-control scopes; add method/tool matrix and default-deny tests before target lookup.
- [ ] MCP105-04 [PARITY] Apply identical authn/authz to App Server WS and MCP HTTP/SSE; run differential authorized/denied/revoked tests.
- [ ] MCP105-05 [MIGRATION] Specify legacy single bearer migration/rollback and instance-bound token compatibility; accept no silent privilege expansion.
- [ ] MCP105-06 [URL] Reject query/cookie/URL credentials in secure mode and update startup/docs; run token_in_url_rejected across HTTP/WS clients.
- [ ] MCP105-07 [TLS] Choose process TLS or freeze trusted reverse proxy headers/health contract; add TLS/WSS integration smoke with certificate verification.
- [ ] MCP105-08 [AUDIT] Add safe token fingerprint/scope/decision fields and canary tests across logs/errors/metrics.
- [ ] MCP105-09 [ABUSE] Test brute force timing, revoked connection, rotation races, slowloris, rate/concurrency bounds and cross-instance tokens.
- [ ] MCP105-10 [HUMAN] Record scope UX decision, TLS threat acceptance and authorized remote smoke; without evidence remote release remains blocked.

