# Contract and decision completion index

This is the canonical completion matrix for the 2026-07-18 contract-deepening pass. Status describes this pass only: DONE means the requested contract/schema/scaffold evidence exists, not that runtime business logic is implemented. PARTIAL names the exact remaining gap and why it is not faked. [provenance: review §11–§14]

The post-review corrections are recorded in
[`ARCHITECTURE_CORRECTIONS.md`](./ARCHITECTURE_CORRECTIONS.md); they refine wire
types, composition ownership, gates and release posture without changing the
157-ID inventory.

## Master artifact map

| Domain | Canonical sources |
|---|---|
| Glossary/truth | `session-turn-item-identity.md`, `source-of-truth.md` |
| Crates | `crate-map.md`, workspace manifests and six new Rust crates |
| Protocol | `30-app-server/v1-01-session-protocol/contracts/*.md`, protocol Rust/schema/goldens |
| Tower/facade | `tower-instance-lifecycle.md`, `runtime-facade.md`, `xai-grok-tower` |
| Tools/security | `tower-agent-tools.md`, `control-plane-security.md`, tool schema |
| MCP/transports | `mcp-server-transport-cli.md`, `xai-grok-mcp-server` |
| SDK | `typescript-sdk.md`, `packages/grok-oss-app-server` |
| P2/freeze | `provider-contract.md`, `goal-boundary.md`, `ui-freeze.md` |
| Execution | `TDD.md`, core `tasks.md`, `TRACEABILITY.md` |

## Requirement-by-requirement status

| ID | Status | Authoritative evidence | Completion proof / remaining gap |
|---|---|---|---|
| D-00.1 | DONE | `.llms/grok-build/_shared/INDEX.md` | Organization/status artifact is explicit and cross-linked. |
| D-00.2 | DONE | `.llms/grok-build/_shared/session-turn-item-identity.md` | Organization/status artifact is explicit and cross-linked. |
| D-00.3 | DONE | `.llms/grok-build/_shared/source-of-truth.md` | Organization/status artifact is explicit and cross-linked. |
| D-00.4 | DONE | `.llms/grok-build/30-app-server/v1-01-session-protocol/contracts/session-protocol-v1.md` | Organization/status artifact is explicit and cross-linked. |
| D-00.5 | DONE | `.llms/grok-build/30-app-server/v1-01-session-protocol/contracts/session-protocol-v1.md` | Organization/status artifact is explicit and cross-linked. |
| D-00.6 | DONE | `.llms/grok-build/README.md` | Organization/status artifact is explicit and cross-linked. |
| D-00.7 | DONE | `.llms/grok-build/README.md` | Organization/status artifact is explicit and cross-linked. |
| D-00.8 | DONE | `.llms/grok-build/_shared/security-authority-boundaries.md` | Organization/status artifact is explicit and cross-linked. |
| D-CR.1 | DONE | `.llms/grok-build/_shared/crate-map.md` | Canonical boundary/DAG/scaffold evidence is present and workspace-checkable. |
| D-CR.2 | DONE | `.llms/grok-build/_shared/crate-map.md` | Canonical boundary/DAG/scaffold evidence is present and workspace-checkable. |
| D-CR.3 | DONE | `crates/codegen/xai-grok-app-server-protocol/{Cargo.toml,src/}` | Canonical boundary/DAG/scaffold evidence is present and workspace-checkable. |
| D-CR.4 | DONE | `crates/codegen/xai-grok-app-server/{Cargo.toml,src/}` | Canonical boundary/DAG/scaffold evidence is present and workspace-checkable. |
| D-CR.5 | DONE | `crates/codegen/xai-grok-tower/{Cargo.toml,src/}` | Canonical boundary/DAG/scaffold evidence is present and workspace-checkable. |
| D-CR.6 | DONE | `crates/codegen/xai-grok-tower-tools/{Cargo.toml,src/}` | Canonical boundary/DAG/scaffold evidence is present and workspace-checkable. |
| D-CR.7 | DONE | `crates/codegen/xai-grok-mcp-server/{Cargo.toml,src/}` | Canonical boundary/DAG/scaffold evidence is present and workspace-checkable. |
| D-CR.8 | DONE | `crates/codegen/xai-grok-app-server-client/; packages/grok-oss-app-server/; .llms/grok-build/_shared/crate-map.md` | Typed Rust client scaffold and private TS client/package both exist without transport success stubs. |
| D-CR.9 | DONE | `.llms/grok-build/_shared/crate-map.md` | Canonical boundary/DAG/scaffold evidence is present and workspace-checkable. |
| D-CR.10 | DONE | `.llms/grok-build/_shared/crate-map.md` | Canonical boundary/DAG/scaffold evidence is present and workspace-checkable. |
| D-CR.11 | DONE | `.llms/grok-build/_shared/crate-map.md` | Canonical boundary/DAG/scaffold evidence is present and workspace-checkable. |
| D-CR.12 | DONE | `.llms/grok-build/_shared/crate-map.md` | Canonical boundary/DAG/scaffold evidence is present and workspace-checkable. |
| D-CR.13 | DONE | `.llms/grok-build/_shared/crate-map.md` | Canonical boundary/DAG/scaffold evidence is present and workspace-checkable. |
| D-CR.14 | DONE | `.llms/grok-build/_shared/crate-map.md` | Canonical boundary/DAG/scaffold evidence is present and workspace-checkable. |
| D-SP.1 | DONE | `.llms/grok-build/30-app-server/v1-01-session-protocol/contracts/session-protocol-v1.md` | Normative wire contract plus schema/type/golden evidence and named test mapping. |
| D-SP.2 | DONE | `.llms/grok-build/30-app-server/v1-01-session-protocol/contracts/session-protocol-v1.md` | Normative wire contract plus schema/type/golden evidence and named test mapping. |
| D-SP.3 | DONE | `.llms/grok-build/30-app-server/v1-01-session-protocol/contracts/session-protocol-v1.md` | Normative wire contract plus schema/type/golden evidence and named test mapping. |
| D-SP.4 | DONE | `.llms/grok-build/30-app-server/v1-01-session-protocol/contracts/session-protocol-v1.md` | Normative wire contract plus schema/type/golden evidence and named test mapping. |
| D-SP.5 | DONE | `.llms/grok-build/30-app-server/v1-01-session-protocol/contracts/session-protocol-v1.md` | Normative wire contract plus schema/type/golden evidence and named test mapping. |
| D-SP.6 | DONE | `.llms/grok-build/30-app-server/v1-01-session-protocol/contracts/session-protocol-v1.md` | Normative wire contract plus schema/type/golden evidence and named test mapping. |
| D-SP.7 | DONE | `.llms/grok-build/30-app-server/v1-01-session-protocol/contracts/session-protocol-v1.md` | Normative wire contract plus schema/type/golden evidence and named test mapping. |
| D-SP.8 | DONE | `.llms/grok-build/30-app-server/v1-01-session-protocol/contracts/session-protocol-v1.md` | Normative wire contract plus schema/type/golden evidence and named test mapping. |
| D-SP.9 | DONE | `.llms/grok-build/30-app-server/v1-01-session-protocol/contracts/methods.md` | Normative wire contract plus schema/type/golden evidence and named test mapping. |
| D-SP.10 | DONE | `.llms/grok-build/30-app-server/v1-01-session-protocol/contracts/methods.md` | Normative wire contract plus schema/type/golden evidence and named test mapping. |
| D-SP.11 | DONE | `.llms/grok-build/30-app-server/v1-01-session-protocol/contracts/methods.md` | Normative wire contract plus schema/type/golden evidence and named test mapping. |
| D-SP.12 | DONE | `.llms/grok-build/30-app-server/v1-01-session-protocol/contracts/events.md` | Normative wire contract plus schema/type/golden evidence and named test mapping. |
| D-SP.13 | DONE | `.llms/grok-build/30-app-server/v1-01-session-protocol/contracts/events.md` | Normative wire contract plus schema/type/golden evidence and named test mapping. |
| D-SP.14 | DONE | `.llms/grok-build/30-app-server/v1-01-session-protocol/contracts/events.md` | Normative wire contract plus schema/type/golden evidence and named test mapping. |
| D-SP.15 | DONE | `.llms/grok-build/30-app-server/v1-01-session-protocol/contracts/events.md` | Normative wire contract plus schema/type/golden evidence and named test mapping. |
| D-SP.16 | DONE | `.llms/grok-build/30-app-server/v1-01-session-protocol/contracts/events.md` | Normative wire contract plus schema/type/golden evidence and named test mapping. |
| D-SP.17 | DONE | `.llms/grok-build/30-app-server/v1-01-session-protocol/contracts/errors.md` | Normative wire contract plus schema/type/golden evidence and named test mapping. |
| D-SP.18 | DONE | `.llms/grok-build/30-app-server/v1-01-session-protocol/contracts/errors.md` | Normative wire contract plus schema/type/golden evidence and named test mapping. |
| D-SP.19 | DONE | `.llms/grok-build/30-app-server/v1-01-session-protocol/contracts/errors.md` | Normative wire contract plus schema/type/golden evidence and named test mapping. |
| D-SP.20 | DONE | `.llms/grok-build/30-app-server/v1-01-session-protocol/contracts/codex-adapter-mapping.md` | Normative wire contract plus schema/type/golden evidence and named test mapping. |
| D-SP.21 | DONE | `crates/codegen/xai-grok-app-server-protocol/schemas/goldens/*.jsonl` | Normative wire contract plus schema/type/golden evidence and named test mapping. |
| D-SP.22 | DONE | `crates/codegen/xai-grok-app-server-protocol/schemas/goldens/*.jsonl` | Normative wire contract plus schema/type/golden evidence and named test mapping. |
| D-SP.23 | DONE | `crates/codegen/xai-grok-app-server-protocol/schemas/goldens/*.jsonl` | Normative wire contract plus schema/type/golden evidence and named test mapping. |
| D-SP.24 | DONE | `crates/codegen/xai-grok-app-server-protocol/schemas/goldens/*.jsonl` | Normative wire contract plus schema/type/golden evidence and named test mapping. |
| D-SP.25 | DONE | `.llms/grok-build/30-app-server/v1-01-session-protocol/contracts/codex-adapter-mapping.md` | Normative wire contract plus schema/type/golden evidence and named test mapping. |
| D-SP.26 | DONE | `crates/codegen/xai-grok-app-server-protocol/{src/,schemas/}` | Normative wire contract plus schema/type/golden evidence and named test mapping. |
| D-SP.27 | DONE | `crates/codegen/xai-grok-app-server-protocol/{src/,schemas/}` | Normative wire contract plus schema/type/golden evidence and named test mapping. |
| D-SP.28 | DONE | `packages/grok-oss-app-server/src/types.ts` | Normative wire contract plus schema/type/golden evidence and named test mapping. |
| D-SP.29 | DONE | `.llms/grok-build/30-app-server/v1-01-session-protocol/contracts/session-protocol-v1.md` | Normative wire contract plus schema/type/golden evidence and named test mapping. |
| D-SP.30 | DONE | `.llms/grok-build/30-app-server/v1-01-session-protocol/contracts/session-protocol-v1.md` | Normative wire contract plus schema/type/golden evidence and named test mapping. |
| D-TW.1 | DONE | `.llms/grok-build/_shared/tower-instance-lifecycle.md` | Lifecycle/state/API/leader-characterization rule and named tests are specified. |
| D-TW.2 | DONE | `.llms/grok-build/_shared/tower-instance-lifecycle.md` | Lifecycle/state/API/leader-characterization rule and named tests are specified. |
| D-TW.3 | DONE | `.llms/grok-build/_shared/tower-instance-lifecycle.md` | Lifecycle/state/API/leader-characterization rule and named tests are specified. |
| D-TW.4 | DONE | `.llms/grok-build/_shared/tower-instance-lifecycle.md` | Lifecycle/state/API/leader-characterization rule and named tests are specified. |
| D-TW.5 | DONE | `.llms/grok-build/_shared/tower-instance-lifecycle.md` | Lifecycle/state/API/leader-characterization rule and named tests are specified. |
| D-TW.6 | DONE | `.llms/grok-build/_shared/tower-instance-lifecycle.md` | Lifecycle/state/API/leader-characterization rule and named tests are specified. |
| D-TW.7 | DONE | `.llms/grok-build/_shared/tower-instance-lifecycle.md` | Lifecycle/state/API/leader-characterization rule and named tests are specified. |
| D-TW.8 | DONE | `.llms/grok-build/_shared/tower-instance-lifecycle.md` | Lifecycle/state/API/leader-characterization rule and named tests are specified. |
| D-TW.9 | DONE | `.llms/grok-build/_shared/tower-instance-lifecycle.md` | Lifecycle/state/API/leader-characterization rule and named tests are specified. |
| D-TW.10 | DONE | `.llms/grok-build/_shared/tower-instance-lifecycle.md` | Lifecycle/state/API/leader-characterization rule and named tests are specified. |
| D-TW.11 | DONE | `.llms/grok-build/_shared/tower-instance-lifecycle.md` | Lifecycle/state/API/leader-characterization rule and named tests are specified. |
| D-TW.12 | DONE | `.llms/grok-build/_shared/tower-instance-lifecycle.md` | Lifecycle/state/API/leader-characterization rule and named tests are specified. |
| D-TW.13 | DONE | `crates/codegen/xai-grok-tower/src/{lib,instance}.rs; .llms/grok-build/_shared/tower-instance-lifecycle.md` | Lifecycle/state/API/leader-characterization rule and named tests are specified. |
| D-TW.14 | DONE | `.llms/grok-build/_shared/tower-instance-lifecycle.md` | Lifecycle/state/API/leader-characterization rule and named tests are specified. |
| D-TW.15 | DONE | `.llms/grok-build/_shared/tower-instance-lifecycle.md` | Lifecycle/state/API/leader-characterization rule and named tests are specified. |
| D-RF.1 | DONE | `crates/codegen/xai-grok-tower/src/lib.rs; .llms/grok-build/_shared/runtime-facade.md` | Full facade/event/redaction/one-actor/fake boundary is specified. |
| D-RF.2 | DONE | `crates/codegen/xai-grok-tower/src/lib.rs; .llms/grok-build/_shared/runtime-facade.md` | Full facade/event/redaction/one-actor/fake boundary is specified. |
| D-RF.3 | DONE | `crates/codegen/xai-grok-tower/src/lib.rs; .llms/grok-build/_shared/runtime-facade.md` | Full facade/event/redaction/one-actor/fake boundary is specified. |
| D-RF.4 | DONE | `crates/codegen/xai-grok-tower/src/lib.rs; .llms/grok-build/_shared/runtime-facade.md` | Full facade/event/redaction/one-actor/fake boundary is specified. |
| D-RF.5 | DONE | `crates/codegen/xai-grok-tower/src/lib.rs; .llms/grok-build/_shared/runtime-facade.md` | Full facade/event/redaction/one-actor/fake boundary is specified. |
| D-RF.6 | DONE | `crates/codegen/xai-grok-tower/src/lib.rs; .llms/grok-build/_shared/runtime-facade.md` | Full facade/event/redaction/one-actor/fake boundary is specified. |
| D-RF.7 | DONE | `crates/codegen/xai-grok-tower/src/lib.rs; .llms/grok-build/_shared/runtime-facade.md` | Full facade/event/redaction/one-actor/fake boundary is specified. |
| D-TA.1 | DONE | `crates/codegen/xai-grok-app-server-protocol/schemas/tower-tools.schema.json; .llms/grok-build/_shared/tower-agent-tools.md` | Nine-tool structural schema plus per-tool semantics/ACL/parity evidence. |
| D-TA.2 | DONE | `crates/codegen/xai-grok-app-server-protocol/schemas/tower-tools.schema.json; .llms/grok-build/_shared/tower-agent-tools.md` | Nine-tool structural schema plus per-tool semantics/ACL/parity evidence. |
| D-TA.3 | DONE | `crates/codegen/xai-grok-app-server-protocol/schemas/tower-tools.schema.json; .llms/grok-build/_shared/tower-agent-tools.md` | Nine-tool structural schema plus per-tool semantics/ACL/parity evidence. |
| D-TA.4 | DONE | `crates/codegen/xai-grok-app-server-protocol/schemas/tower-tools.schema.json; .llms/grok-build/_shared/tower-agent-tools.md` | Nine-tool structural schema plus per-tool semantics/ACL/parity evidence. |
| D-TA.5 | DONE | `crates/codegen/xai-grok-app-server-protocol/schemas/tower-tools.schema.json; .llms/grok-build/_shared/tower-agent-tools.md` | Nine-tool structural schema plus per-tool semantics/ACL/parity evidence. |
| D-TA.6 | DONE | `crates/codegen/xai-grok-app-server-protocol/schemas/tower-tools.schema.json; .llms/grok-build/_shared/tower-agent-tools.md` | Nine-tool structural schema plus per-tool semantics/ACL/parity evidence. |
| D-TA.7 | DONE | `crates/codegen/xai-grok-app-server-protocol/schemas/tower-tools.schema.json; .llms/grok-build/_shared/tower-agent-tools.md` | Nine-tool structural schema plus per-tool semantics/ACL/parity evidence. |
| D-TA.8 | DONE | `crates/codegen/xai-grok-app-server-protocol/schemas/tower-tools.schema.json; .llms/grok-build/_shared/tower-agent-tools.md` | Nine-tool structural schema plus per-tool semantics/ACL/parity evidence. |
| D-TA.9 | DONE | `crates/codegen/xai-grok-app-server-protocol/schemas/tower-tools.schema.json; .llms/grok-build/_shared/tower-agent-tools.md` | Nine-tool structural schema plus per-tool semantics/ACL/parity evidence. |
| D-TA.10 | DONE | `crates/codegen/xai-grok-app-server-protocol/schemas/tower-tools.schema.json; .llms/grok-build/_shared/tower-agent-tools.md` | Nine-tool structural schema plus per-tool semantics/ACL/parity evidence. |
| D-TA.11 | DONE | `crates/codegen/xai-grok-app-server-protocol/schemas/tower-tools.schema.json; .llms/grok-build/_shared/tower-agent-tools.md` | Nine-tool structural schema plus per-tool semantics/ACL/parity evidence. |
| D-TA.12 | DONE | `crates/codegen/xai-grok-app-server-protocol/schemas/tower-tools.schema.json; .llms/grok-build/_shared/tower-agent-tools.md` | Nine-tool structural schema plus per-tool semantics/ACL/parity evidence. |
| D-SEC.1 | DONE | `.llms/grok-build/_shared/control-plane-security.md` | Locked permissive MVP, failure matrix, limits, threats and named tests. |
| D-SEC.2 | DONE | `.llms/grok-build/_shared/control-plane-security.md` | Locked permissive MVP, failure matrix, limits, threats and named tests. |
| D-SEC.3 | DONE | `.llms/grok-build/_shared/control-plane-security.md` | Locked permissive MVP, failure matrix, limits, threats and named tests. |
| D-SEC.4 | DONE | `.llms/grok-build/_shared/control-plane-security.md` | Locked permissive MVP, failure matrix, limits, threats and named tests. |
| D-SEC.5 | DONE | `.llms/grok-build/_shared/control-plane-security.md` | Locked permissive MVP, failure matrix, limits, threats and named tests. |
| D-SEC.6 | DONE | `.llms/grok-build/_shared/control-plane-security.md` | Locked permissive MVP, failure matrix, limits, threats and named tests. |
| D-SEC.7 | DONE | `.llms/grok-build/_shared/control-plane-security.md` | Locked permissive MVP, failure matrix, limits, threats and named tests. |
| D-SEC.8 | DONE | `.llms/grok-build/_shared/control-plane-security.md` | Locked permissive MVP, failure matrix, limits, threats and named tests. |
| D-SEC.9 | DONE | `.llms/grok-build/_shared/control-plane-security.md` | Locked permissive MVP, failure matrix, limits, threats and named tests. |
| D-SEC.10 | DONE | `.llms/grok-build/_shared/control-plane-security.md` | Locked permissive MVP, failure matrix, limits, threats and named tests. |
| D-SEC.11 | DONE | `.llms/grok-build/_shared/control-plane-security.md` | Locked permissive MVP, failure matrix, limits, threats and named tests. |
| D-SEC.12 | DONE | `.llms/grok-build/_shared/control-plane-security.md` | Locked permissive MVP, failure matrix, limits, threats and named tests. |
| D-SEC.13 | DONE | `.llms/grok-build/_shared/control-plane-security.md` | Locked permissive MVP, failure matrix, limits, threats and named tests. |
| D-MCP.1 | DONE | `crates/codegen/xai-grok-mcp-server/; .llms/grok-build/_shared/mcp-server-transport-cli.md` | Server-only adapter contract, exact transports, parity and no-self-loop evidence. |
| D-MCP.2 | DONE | `crates/codegen/xai-grok-mcp-server/; .llms/grok-build/_shared/mcp-server-transport-cli.md` | Server-only adapter contract, exact transports, parity and no-self-loop evidence. |
| D-MCP.3 | DONE | `crates/codegen/xai-grok-mcp-server/; .llms/grok-build/_shared/mcp-server-transport-cli.md` | Server-only adapter contract, exact transports, parity and no-self-loop evidence. |
| D-MCP.4 | DONE | `crates/codegen/xai-grok-mcp-server/; .llms/grok-build/_shared/mcp-server-transport-cli.md` | Server-only adapter contract, exact transports, parity and no-self-loop evidence. |
| D-MCP.5 | DONE | `crates/codegen/xai-grok-mcp-server/; .llms/grok-build/_shared/mcp-server-transport-cli.md` | Server-only adapter contract, exact transports, parity and no-self-loop evidence. |
| D-MCP.6 | DONE | `crates/codegen/xai-grok-mcp-server/; .llms/grok-build/_shared/mcp-server-transport-cli.md` | Server-only adapter contract, exact transports, parity and no-self-loop evidence. |
| D-MCP.7 | DONE | `crates/codegen/xai-grok-mcp-server/; .llms/grok-build/_shared/mcp-server-transport-cli.md` | Server-only adapter contract, exact transports, parity and no-self-loop evidence. |
| D-MCP.8 | DONE | `crates/codegen/xai-grok-mcp-server/; .llms/grok-build/_shared/mcp-server-transport-cli.md` | Server-only adapter contract, exact transports, parity and no-self-loop evidence. |
| D-MCP.9 | DONE | `crates/codegen/xai-grok-mcp-server/; .llms/grok-build/_shared/mcp-server-transport-cli.md` | Server-only adapter contract, exact transports, parity and no-self-loop evidence. |
| D-MCP.10 | DONE | `crates/codegen/xai-grok-mcp-server/; .llms/grok-build/_shared/mcp-server-transport-cli.md` | Server-only adapter contract, exact transports, parity and no-self-loop evidence. |
| D-TR.1 | DONE | `.llms/grok-build/_shared/mcp-server-transport-cli.md` | Framing/API/CLI/health/co-start decision is explicit. |
| D-TR.2 | DONE | `.llms/grok-build/_shared/mcp-server-transport-cli.md` | Framing/API/CLI/health/co-start decision is explicit. |
| D-TR.3 | DONE | `.llms/grok-build/_shared/mcp-server-transport-cli.md` | Framing/API/CLI/health/co-start decision is explicit. |
| D-TR.4 | DONE | `.llms/grok-build/_shared/mcp-server-transport-cli.md` | Framing/API/CLI/health/co-start decision is explicit. |
| D-TR.5 | DONE | `.llms/grok-build/_shared/mcp-server-transport-cli.md` | Framing/API/CLI/health/co-start decision is explicit. |
| D-TR.6 | PARTIAL | `.llms/grok-build/_shared/mcp-server-transport-cli.md` | PARTIAL — token create/list/revoke is conditional in the review and remains a documented HUMAN/product UX decision; no fake CLI scaffold. |
| D-TR.7 | DONE | `.llms/grok-build/_shared/mcp-server-transport-cli.md` | Framing/API/CLI/health/co-start decision is explicit. |
| D-TR.8 | DONE | `.llms/grok-build/_shared/mcp-server-transport-cli.md` | Framing/API/CLI/health/co-start decision is explicit. |
| D-TS.1 | PARTIAL | `packages/grok-oss-app-server/; .llms/grok-build/_shared/typescript-sdk.md` | PARTIAL — path/name are fixed for the private scaffold but public npm publication/name remains HUMAN-gated per handoff K1–K2. |
| D-TS.2 | DONE | `packages/grok-oss-app-server/; .llms/grok-build/_shared/typescript-sdk.md` | Private interim SDK/client/stream/examples/drift/error mapping evidence. |
| D-TS.3 | DONE | `packages/grok-oss-app-server/; .llms/grok-build/_shared/typescript-sdk.md` | Private interim SDK/client/stream/examples/drift/error mapping evidence. |
| D-TS.4 | DONE | `packages/grok-oss-app-server/; .llms/grok-build/_shared/typescript-sdk.md` | Private interim SDK/client/stream/examples/drift/error mapping evidence. |
| D-TS.5 | DONE | `packages/grok-oss-app-server/; .llms/grok-build/_shared/typescript-sdk.md` | Private interim SDK/client/stream/examples/drift/error mapping evidence. |
| D-TS.6 | DONE | `packages/grok-oss-app-server/; .llms/grok-build/_shared/typescript-sdk.md` | Private interim SDK/client/stream/examples/drift/error mapping evidence. |
| D-TS.7 | DONE | `packages/grok-oss-app-server/; .llms/grok-build/_shared/typescript-sdk.md` | Private interim SDK/client/stream/examples/drift/error mapping evidence. |
| D-AP.1 | DONE | `.llms/grok-build/_shared/approvals-controller-history.md; 30-app-server/v1-01-session-protocol/contracts/events.md` | Lease/Interaction/history/cursor behavior and HUMAN default are explicit. |
| D-AP.2 | DONE | `.llms/grok-build/_shared/approvals-controller-history.md; 30-app-server/v1-01-session-protocol/contracts/events.md` | Lease/Interaction/history/cursor behavior and HUMAN default are explicit. |
| D-AP.3 | DONE | `.llms/grok-build/_shared/approvals-controller-history.md; 30-app-server/v1-01-session-protocol/contracts/events.md` | Lease/Interaction/history/cursor behavior and HUMAN default are explicit. |
| D-AP.4 | DONE | `.llms/grok-build/_shared/approvals-controller-history.md; 30-app-server/v1-01-session-protocol/contracts/events.md` | Lease/Interaction/history/cursor behavior and HUMAN default are explicit. |
| D-AP.5 | DONE | `.llms/grok-build/_shared/approvals-controller-history.md; 30-app-server/v1-01-session-protocol/contracts/events.md` | Lease/Interaction/history/cursor behavior and HUMAN default are explicit. |
| D-AP.6 | DONE | `.llms/grok-build/_shared/approvals-controller-history.md; 30-app-server/v1-01-session-protocol/contracts/events.md` | Lease/Interaction/history/cursor behavior and HUMAN default are explicit. |
| D-PR.1 | DONE | `.llms/grok-build/_shared/provider-contract.md; docs/architecture/byok-providers-onboarding/` | Descriptor/binding/catalog/onboarding/fixtures/normative inputs/issues mapped. |
| D-PR.2 | DONE | `.llms/grok-build/_shared/provider-contract.md; docs/architecture/byok-providers-onboarding/` | Descriptor/binding/catalog/onboarding/fixtures/normative inputs/issues mapped. |
| D-PR.3 | DONE | `.llms/grok-build/_shared/provider-contract.md; docs/architecture/byok-providers-onboarding/` | Descriptor/binding/catalog/onboarding/fixtures/normative inputs/issues mapped. |
| D-PR.4 | DONE | `.llms/grok-build/_shared/provider-contract.md; docs/architecture/byok-providers-onboarding/` | Descriptor/binding/catalog/onboarding/fixtures/normative inputs/issues mapped. |
| D-PR.5 | DONE | `.llms/grok-build/_shared/provider-contract.md; docs/architecture/byok-providers-onboarding/` | Descriptor/binding/catalog/onboarding/fixtures/normative inputs/issues mapped. |
| D-PR.6 | DONE | `.llms/grok-build/_shared/provider-contract.md; docs/architecture/byok-providers-onboarding/` | Descriptor/binding/catalog/onboarding/fixtures/normative inputs/issues mapped. |
| D-PR.7 | DONE | `.llms/grok-build/_shared/provider-contract.md; docs/architecture/byok-providers-onboarding/` | Descriptor/binding/catalog/onboarding/fixtures/normative inputs/issues mapped. |
| D-GO.1 | DONE | `.llms/grok-build/_shared/goal-boundary.md` | Future flag/hot-path/dual-test boundary; explicit no-scaffold rule. |
| D-GO.2 | DONE | `.llms/grok-build/_shared/goal-boundary.md` | Future flag/hot-path/dual-test boundary; explicit no-scaffold rule. |
| D-GO.3 | DONE | `.llms/grok-build/_shared/goal-boundary.md` | Future flag/hot-path/dual-test boundary; explicit no-scaffold rule. |
| D-GO.4 | DONE | `.llms/grok-build/_shared/goal-boundary.md` | Future flag/hot-path/dual-test boundary; explicit no-scaffold rule. |
| D-TD.1 | DONE | `.llms/grok-build/TDD.md` | Named layout, evidence format, goldens/security/commands specified. |
| D-TD.2 | DONE | `.llms/grok-build/TDD.md` | Named layout, evidence format, goldens/security/commands specified. |
| D-TD.3 | DONE | `.llms/grok-build/TDD.md` | Named layout, evidence format, goldens/security/commands specified. |
| D-TD.4 | DONE | `.llms/grok-build/TDD.md` | Named layout, evidence format, goldens/security/commands specified. |
| D-TD.5 | DONE | `.llms/grok-build/TDD.md` | Named layout, evidence format, goldens/security/commands specified. |
| D-TD.6 | DONE | `.llms/grok-build/TDD.md` | Named layout, evidence format, goldens/security/commands specified. |
| D-UI.1 | DONE | `.llms/grok-build/_shared/ui-freeze.md` | Exact MVP freeze surfaces and future migration boundary specified. |
| D-UI.2 | DONE | `.llms/grok-build/_shared/ui-freeze.md` | Exact MVP freeze surfaces and future migration boundary specified. |
| D-UI.3 | DONE | `.llms/grok-build/_shared/ui-freeze.md` | Exact MVP freeze surfaces and future migration boundary specified. |
| D-BK.1 | DONE | `.llms/grok-build/{80-channel-gateways,90-realtime-voice}/SPECS.md` | Backlog consumes core contracts; no implementation/schema added. |
| D-BK.2 | DONE | `.llms/grok-build/{80-channel-gateways,90-realtime-voice}/SPECS.md` | Backlog consumes core contracts; no implementation/schema added. |
| D-TK.1 | DONE | `.llms/grok-build/{20-tower-core,30-app-server,40-mcp-control-plane,50-tower-agent-tools,60-sdk-typescript}/**/tasks.md` | Every core epic has tasks with D-ID, owner path, command and observable acceptance. |
| D-TK.2 | DONE | `.llms/grok-build/{20-tower-core,30-app-server,40-mcp-control-plane,50-tower-agent-tools,60-sdk-typescript}/**/tasks.md` | Every core epic has tasks with D-ID, owner path, command and observable acceptance. |
| D-TK.3 | DONE | `.llms/grok-build/{20-tower-core,30-app-server,40-mcp-control-plane,50-tower-agent-tools,60-sdk-typescript}/**/tasks.md` | Every core epic has tasks with D-ID, owner path, command and observable acceptance. |
| D-TK.4 | DONE | `.llms/grok-build/{20-tower-core,30-app-server,40-mcp-control-plane,50-tower-agent-tools,60-sdk-typescript}/**/tasks.md` | Every core epic has tasks with D-ID, owner path, command and observable acceptance. |
| D-TK.5 | DONE | `.llms/grok-build/{20-tower-core,30-app-server,40-mcp-control-plane,50-tower-agent-tools,60-sdk-typescript}/**/tasks.md` | Every core epic has tasks with D-ID, owner path, command and observable acceptance. |

## Open HUMAN gates

- `(HUMAN, manual-verify, blocking: remote release)`: accept full-control bearer over optional cleartext/non-loopback threat model.
- `(HUMAN, product-decision, blocking: compatibility adapter only)`: decide missing-`jsonrpc` behavior; native remains strict.
- `(HUMAN, product-decision, blocking: SDK publish only)`: approve public package name/publication; scaffold remains private.
- `(HUMAN, product-decision, blocking: token CLI implementation only)`: freeze token create/list/revoke UX if included in MVP.
- `(HUMAN, product-decision, blocking: headless release policy)`: approve Interaction deadline/wait→deny default.

## Completion rule

P0-VITAL has no PARTIAL entries. The three PARTIAL entries are deliberately outside the P0 semantic freeze and name concrete human/sequence-dependent gaps. Processor/runtime, production auth, dashboard migration, Goal v2, Telegram and voice remain unimplemented by design.
