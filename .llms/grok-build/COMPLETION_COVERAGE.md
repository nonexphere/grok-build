# Completion coverage — App Server + MCP + Tower

Snapshot de refinamento: 2026-07-19, branch goblin-implement-epic-tree.

Esta matriz é o ledger normativo do goal de planejamento. Cobertura significa contrato + epic owner + task executável + gate observável. Não significa código concluído.

| Requirement/gap | Contrato | Epic owner | Gate terminal |
|---|---|---|---|
| F-01 actor real product-wired | _shared/product-runtime-readiness.md | 20/v1-06, 20/v1-08 | vertical binário start→send→wait→history |
| F-02 agentType/residency/filters/overrides | lifecycle + conformance | 20/v1-07, 50/v1-03 | schema-valid canonical rows |
| F-03 epoch/cursor/wait real | identity ordering + tools | 30/v1-05, 50/v1-03 | history→wait/restart/rebind |
| F-04 archive/resume state truth | lifecycle | 20/v1-07 | transition/property/crash suite |
| F-05 error envelope/retryability | conformance | 30/v1-09, 40/v1-04, 50/v1-03 | differential errors all adapters |
| F-06 MCP schema resolvível | conformance + MCP transport | 40/v1-04 | independent MCP client compiles schemas |
| F-07 source generation/SDK drift | conformance + TS SDK | 60/v1-02 | clean regeneration + black-box SDK |
| F-08 fake versus product evidence | product readiness | 20/v1-06, 30/v1-09 | separate CI jobs and real vertical |
| F-09 TLS/remote release | security | 40/v1-05 | TLS proxy/process smoke + human gate |
| F-10 capability truth | capability truth | 30/v1-09 | initialize matrix equals executable paths |
| canonical actor uniqueness | runtime ownership | 20/v1-06 | one actor under concurrent start/resume |
| provider/model/sandbox binding | product readiness | 20/v1-06 | immutable per-Turn binding and policy |
| interactions product delivery | approvals + readiness | 30/v1-09 | parked actor, reconnect, deny/timeout |
| nine tools complete | tower-agent-tools | 50/v1-03 | every parity case for every tool |
| MCP HTTP lifecycle/SSE | MCP transport | 40/v1-04 | POST/GET/DELETE, TTL, rebind, resync |
| MCP stdio product launcher | MCP transport | 40/v1-04 | real subprocess framing/EOF/noise |
| token create/list/revoke/scopes | security + CLI | 40/v1-05 | scoped allow/deny/revocation race |
| cleartext query bearer conflict | security | 40/v1-05 | URL token rejected in secure mode |
| SDK truly generated | TS contract | 60/v1-02 | delete/regenerate/clean diff |
| SDK against real listeners | TS contract | 60/v1-02 | Node stdio/WS + reconnect/abort |
| observability/resource bounds | lifecycle/security | 20/v1-04, 30/v1-07 | load/fault/secret canary |
| dead/placeholder paths | capability truth | 30/v1-09 | inventory + removal/justification |
| release evidence | all contracts | 30/v1-07 | all dependencies green; human gates explicit |

## New corrective epics

- [20/v1-06-canonical-session-actor-runtime](20-tower-core/v1-06-canonical-session-actor-runtime/)
- [20/v1-07-lifecycle-metadata-recovery](20-tower-core/v1-07-lifecycle-metadata-recovery/)
- [30/v1-09-capability-contract-product-conformance](30-app-server/v1-09-capability-contract-product-conformance/)
- [40/v1-04-mcp-contract-transport-completion](40-mcp-control-plane/v1-04-mcp-contract-transport-completion/)
- [40/v1-05-token-scopes-tls-release](40-mcp-control-plane/v1-05-token-scopes-tls-release/)
- [50/v1-03-nine-tool-semantic-completion](50-tower-agent-tools/v1-03-nine-tool-semantic-completion/)
- [60/v1-02-generated-sdk-black-box-ga](60-sdk-typescript/v1-02-generated-sdk-black-box-ga/)
- [20/v1-08-product-session-host](20-tower-core/v1-08-product-session-host/)

## Completion invariant

Cada row acima deve apontar para ao menos uma task com owner path, comando e acceptance observável. O gate final é blocked se qualquer row estiver missing, partial, contradictory ou somente fake-backed.
