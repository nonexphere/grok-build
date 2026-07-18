# Tasks — v1-07 release hardening

- [x] `AS107-01` [D-TD.2,D-MCP.7] Run one black-box suite against in-process, stdio, WS and MCP; command `cargo test -p xai-grok-app-server -p xai-grok-mcp-server conformance`; accept identical normalized cases.
- [x] `AS107-02` [D-SEC.11,D-SEC.12] Run complete threat/security suite; command `cargo test -p xai-grok-app-server -p xai-grok-mcp-server security`; accept all documented scenarios have a named passing test.
- [x] `AS107-03` [D-SP.21..24,D-TS.5] Run schema/golden/TS drift gates; command `cargo test -p xai-grok-app-server-protocol && npm --prefix packages/grok-oss-app-server run typecheck`; accept no binding drift.
- [x] `AS107-04` [D-TW.15] Run drain/restart/reconnect composition tests; command `cargo test -p xai-grok-tower -p xai-grok-app-server restart`; accept stable Session IDs and correct epoch behavior.
- [x] `AS107-05` [D-UI.1..3] Validate frozen ACP/roster/dashboard surfaces in shell/pager packages; run `./scripts/run-rust-test-gate.sh roster cargo test -p xai-grok-shell roster` and `./scripts/run-rust-test-gate.sh dashboard cargo test -p xai-grok-pager dashboard`; accept no MVP dashboard/TUI migration or protocol behavior change.
- [x] `AS107-06` [D-TD.3] Produce delivery evidence with every RED/GREEN, skip and blocked gate; run `rg -n 'RED|GREEN|SKIP|BLOCKED' .llms/grok-build/30-app-server/v1-07-release-hardening`; accept skip never labeled PASS and no unresolved P0 contract drift.
- [ ] [D-SEC.13] `(HUMAN, manual-verify, blocking: remote release)` execute public-bind threat checklist and record explicit acceptance.
