# Tasks — in-process ACL and MCP parity

- [ ] `TA102-01` [D-TA.5] Implement fail-closed ACL in `xai-grok-tower-tools/src/acl.rs`; run `cargo test -p xai-grok-tower-tools acl`; accept orchestrator/default-deny/custom-opt-in matrix.
- [ ] `TA102-02` [D-TA.6] Register descriptors in-process through the runtime tool composition path; run `cargo test -p xai-grok-tower-tools in_process_registration`; accept direct facade calls with no JSON-RPC/MCP loop.
- [ ] `TA102-03` [D-TA.7,D-TA.8] Execute shared fixtures through MCP and in-process adapters; run `cargo test -p xai-grok-tower-tools -p xai-grok-mcp-server adapter_parity`; accept normalized equality for all nine tools.
- [ ] `TA102-04` [D-TA.9] Add dependency/composition assertion in Tower tools tests; run `cargo test -p xai-grok-tower-tools forbidden_hub`; accept no `tower_agent_hub` symbol or local self-MCP edge.
- [ ] `TA102-05` [D-SEC.12] Add ACL-before-lookup fixture in `xai-grok-tower-tools/tests/acl.rs`; run `cargo test -p xai-grok-tower-tools acl_does_not_leak_target`; accept identical deny for existing/missing target.
- [ ] `TA102-06` [D-TD.6] Validate protocol/Tower/tools/MCP packages; run `cargo test -p xai-grok-app-server-protocol -p xai-grok-tower -p xai-grok-tower-tools -p xai-grok-mcp-server`; accept the full vertical contract green.
