# Matriz de rastreabilidade

Esta matriz liga decisão humana a contrato, epic e prova planejada. Tasks
granulares vivem nos `tasks.md` de cada epic.

| Requisito | Contrato | Epic owner | Prova de aceite |
|---|---|---|---|
| Session canônico | `_shared/session-turn-item-identity.md` | `30/v1-01` | schema/TS/MCP sem `thread` nativo |
| Tower promove leader | `_shared/runtime-ownership.md` | `20/v1-01` | actor/registry ownership + connect-or-spawn tests |
| N sessions/workspaces | `_shared/tower-instance-lifecycle.md` | `20/v1-02` | multi-session integration |
| N Towers/machine | mesmo | `20/v1-03` | isolated endpoint/token/state smoke |
| T4 connect/spawn default | mesmo | `20/v1-03` | CLI lifecycle tests |
| WS early | `_shared/mcp-server-transport-cli.md`, protocol errors | `30/v1-04` | transport conformance WS |
| MCP remoto same release | `_shared/mcp-server-transport-cli.md` | `40/v1-01..02` | stdio + HTTP/SSE conformance |
| bearer full-control | `_shared/control-plane-security.md` | `30/v1-04`, `40/v1-02` | auth/redaction/threat tests |
| sem Origin/scopes/TLS mandatory | mesmo | `30/v1-04` | config/warning tests + docs |
| tools MUST | `_shared/tower-agent-tools.md` | `50/v1-01` | one contract suite por operação |
| in-process + MCP semantic parity | runtime/tools contracts | `50/v1-02` | differential conformance |
| ACL orchestrator default | security/tools | `50/v1-02` | allow/deny/config tests |
| local tool sem MCP loop | runtime ownership | `50/v1-02` | composition assertion |
| TS SDK real | `_shared/typescript-sdk.md`, `packages/grok-oss-app-server/` | `60/v1-01` | Node stdio/WS scripts + type/drift gates; browser limitation explicit |
| dashboard intocado | `_shared/ui-freeze.md` | `30/v2-01` | MVP regression ACP/roster |
| Goal fora do core | `_shared/goal-boundary.md` | `70/*` | DAG sem Goal v2 em release core |
| goal v1/v2 flags | `_shared/goal-boundary.md` | `70/v2-01..07` | future dual-version/rollback fixtures |
| BYOK OR/Groq/CF | `_shared/provider-contract.md` + onboarding docs | `10/v1-02..05` | onboarding→catalog→turn→logout |
| TDD | `TDD.md` | todos | RED/GREEN + package/conformance gate |
| gateways/voice só backlog | project docs | `80/v1-01`, `90/v1-01` | nenhum dependency inverso/core code task |

## Deepening de contratos e scaffolds (2026-07-18)

| Review | Artefato | Verificação |
|---|---|---|
| D-00 | `_shared/INDEX.md` | todo ID tem DONE/PARTIAL e partial justificado |
| D-CR | `_shared/crate-map.md`; seis crates Rust novas + package TS | package checks e DAG sem ciclo |
| D-SP | `30/.../contracts/*.md`; protocol schemas/goldens | serde roundtrip, named-definition validation, 4 JSONL |
| D-TW/D-RF | lifecycle + runtime facade | facade única e nenhum segundo actor |
| D-TA | tools contract + schema | 9 pares input/output e ACL fail-closed |
| D-SEC | security contract | matrix, threats, canaries e human remote gate |
| D-MCP/D-TR | MCP/transport/CLI contract | server distinto do client e co-start matrix |
| D-TS | TS contract + package privado | skeleton strict; publish é human gate |

Não há scaffold de Goal v2, Telegram, voice ou migração do dashboard.

## Issues persistentes incorporadas

| Issue | Epic que deve reconciliar | Resultado esperado |
|---|---|---|
| `data-001` Codex 401 FIFO fallback | `10/v1-01` | attempt-bound recovery provada ou status corrigido |
| `data-002` compaction cache key | `10/v1-01` | account-scoped key na composition real |
| `data-003` Codex-only login gate | `10/v1-01..02` | provider-driven gate sem regressão Codex |
| `docs-001` release contradiction | `10/v1-01` | uma única readiness matrix |
| `docs-002` harness audit artifact | `10/v1-01` | evidence durável ou claim removida |
| `operations-001` remote tip validation | `10/v1-01` | validação rotulada no scope real |
| `testing-001` live skip as pass | `10/v1-01` | skip/blocked nunca PASS |
| `testing-002` branch diff check | `10/v1-01` | gate reproduzível ou issue honesta |
| `ui-model-identity-system-prompt-label-sticky` | `10/v1-01` | harness label e model identity separados |
