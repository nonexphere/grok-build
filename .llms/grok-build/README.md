# grok-oss — árvore de planos e épicos

> **Escopo deste snapshot:** planejamento documental; nenhum código de produto é
> autorizado por esta árvore. Aprovação para executar cada epic é separada.

| Campo | Valor |
|---|---|
| Snapshot | `d0ea385c9d0e7335da9f831a3031fca6e3f2bd67` + dirty audited worktree |
| Data | 2026-07-19 (`America/Sao_Paulo`) |
| Branch | `goblin-implement-epic-tree` |
| Produto público | `grok-oss` · `~/.grok-oss` · `@brasalabs/grok-oss` |
| Fontes humanas | handoff §13–14 + transcrição de 2026-07-18 |

Esta árvore ordena as próximas atualizações do fork em uma plataforma com uma
**Tower** multi-session, App Server Session/Turn/Item, MCP remoto e tools
`tower_agent_*`. O leader, o registry de sessões, o `SessionActor`, ACP e os
arquivos de sessão existentes são promovidos e reutilizados; não se cria um
segundo runtime. [provenance: user-input, code, inferred]

## Programas

| Ordem | Programa | Papel | Estado do código | Release core |
|---:|---|---|---|---|
| 05 | [Build e Release](./05-build-and-release/) | baseline, grafo, cache, CI e limpeza | baseline em execução; otimizações planejadas | **MUST** |
| 10 | [Providers](./10-providers/) | Codex/multi-auth + BYOK | Codex avançado; BYOK manual | paralelo controlado |
| 20 | [Tower Core](./20-tower-core/) | daemon, registry e instâncias | supervisor existe; actor product-wired falta | **MUST** |
| 30 | [App Server](./30-app-server/) | Session/Turn/Item + stdio/WS | protocol/listeners existem; capabilities reais parciais | **MUST** |
| 40 | [MCP Control Plane](./40-mcp-control-plane/) | MCP stdio + Streamable HTTP/SSE | HTTP existe; schema/stdio/auth release incompletos | **MUST** |
| 50 | [Tower Agent Tools](./50-tower-agent-tools/) | `tower_agent_*` e ACL | nove tools existem; semântica product incompleta | **MUST** |
| 60 | [SDK TypeScript](./60-sdk-typescript/) | schema, client e scripts | package/interim mirror existe; black-box falta | **MUST** |
| 70 | [Goal Runtime](./70-goal-runtime/) | goal legado v1 + runtime v2 | v1 existe; v2 é plano | futuro |
| 80 | [Channel Gateways](./80-channel-gateways/) | Telegram primeiro | backlog | fora do core |
| 90 | [Realtime Voice](./90-realtime-voice/) | full duplex + barge-in | ditado/STT parcial | fora do core |

## Roadmap e waves

| Wave | Epics | Pode paralelizar | Gate de saída |
|---|---|---|---|
| 0 | `10/v1-01`, `20/v1-01`, `30/v1-01` | os três | baseline honesto, ownership e contratos congelados |
| 1 | `10/v1-02`, `20/v1-02`, `30/v1-02` | providers e runtime | provider seam + uma Tower/registry + facade única |
| 2 | `30/v1-03` | não | primeira vertical slice local in-process/stdio completa |
| 3 | `20/v1-03`, `30/v1-05..06`, `50/v1-01` | parcialmente | lifecycle multi-instance, replay/interactions e tools in-process |
| 4 | `30/v1-04`, depois `40/v1-01` | parcialmente | WebSocket local primeiro; MCP somente após descriptors/facade |
| 5 | `50/v1-02`, `60/v1-01`, `10/v1-03..05` | sim | ACL/paridade, SDK real, BYOK por provider |
| 6 | `30/v1-07`, `40/v1-02`, `20/v1-04` | hardening conjunto | remote/TLS classification, conformance, runbooks e release evidence |
| 7+ | Goal v2, dashboard client, gateways e voz | fora do MVP core | programas separados e explicitamente aprovados |

### Corrective completion waves (2026-07-19)

| Wave | Epics | Gate |
|---|---|---|
| C1 | `20/v1-06` | actor real, shared runtime, readiness truth |
| C2 | `20/v1-07`, `30/v1-05..06` | lifecycle/metadata/replay/interactions canônicos |
| C3 | `50/v1-03`, `30/v1-09` | nove tools e capabilities sem placeholders |
| C4 | `40/v1-04`, `60/v1-02` | MCP/SDK por clientes black-box reais |
| C5 | `40/v1-05` | token lifecycle, scopes e TLS remote GA |
| C6 | `30/v1-07`, `20/v1-04` | conformance/load/security/cleanup/release final |

### Build and readiness completion waves (2026-07-21)

| Wave | Epics | Gate |
|---|---|---|
| B0 | `05/v1-01` | baseline frio, warm, incremental, release e CI |
| B1 | `20/v1-09`, `20/v1-10`, `30/v1-10` | runtime real, lifecycle e capability truth |
| B2 | `40/v1-06`, `50/v1-04` | MCP parity e nove tools product-backed |
| B3 | `40/v1-07`, `60/v1-03`, `20/v1-11` | segurança, SDK, carga e fault testing |
| B4 | `05/v1-02`, `05/v1-03`, `05/v1-04` | compilação, CI, cache e dívida terminal |
| B5 | `30/v1-11` | release verdict completo |

## Status e estimativas dos epics

| Programa | Epic | Status | Estimativa |
|---|---|---|---|
| 10 | `v1-01-codex-readiness-hygiene` | rascunho | 1–2 sem. |
| 10 | `v1-02-api-key-provider-foundation` | rascunho | 2–4 sem. |
| 10 | `v1-03-openrouter-onboarding` | rascunho | 1–2 sem. |
| 10 | `v1-04-groq-onboarding` | rascunho | 1–2 sem. |
| 10 | `v1-05-cloudflare-onboarding` | rascunho | 2–3 sem. |
| 20 | `v1-01-leader-characterization-promotion` | rascunho | 2–3 sem. |
| 20 | `v1-02-multi-session-workspace-registry` | rascunho | 2–4 sem. |
| 20 | `v1-03-multi-instance-daemon-modes` | rascunho | 2–3 sem. |
| 20 | `v1-04-operations-hardening` | rascunho | 1–3 sem. |
| 20 | `v1-05-tower-supervisor` | concluído localmente | entregue |
| 20 | `v1-06-canonical-session-actor-runtime` | rascunho/P0 | 2–4 sem. |
| 20 | `v1-07-lifecycle-metadata-recovery` | rascunho/P0 | 2–3 sem. |
| 30 | `v1-01-session-protocol` | rascunho | 2–3 sem. |
| 30 | `v1-02-runtime-facade-projection` | rascunho | 2–4 sem. |
| 30 | `v1-03-core-in-process-stdio` | rascunho | 2–4 sem. |
| 30 | `v1-04-websocket-remote-auth` | rascunho | 2–3 sem. |
| 30 | `v1-05-history-replay` | rascunho | 2–4 sem. |
| 30 | `v1-06-approvals-control` | rascunho | 2–3 sem. |
| 30 | `v1-07-release-hardening` | rascunho | 2–4 sem. |
| 30 | `v1-08-tower-product-runtime` | concluído localmente | entregue |
| 30 | `v1-09-capability-contract-product-conformance` | rascunho/P0 | 2–4 sem. |
| 30 | `v2-01-dashboard-client-migration` | rascunho/backlog | 2–4 sem. |
| 40 | `v1-01-server-transports` | rascunho | 2–4 sem. |
| 40 | `v1-02-remote-security-conformance` | rascunho | 1–3 sem. |
| 40 | `v1-03-tower-product-runtime` | concluído localmente | entregue |
| 40 | `v1-04-mcp-contract-transport-completion` | rascunho/P0 | 2–4 sem. |
| 40 | `v1-05-token-scopes-tls-release` | rascunho/P1 | 2–4 sem. |
| 50 | `v1-01-tool-contract-and-facade` | rascunho | 2–3 sem. |
| 50 | `v1-02-in-process-acl-mcp-parity` | rascunho | 2–3 sem. |
| 50 | `v1-03-nine-tool-semantic-completion` | rascunho/P0 | 2–4 sem. |
| 50 | `v2-01-peer-messaging-study` | rascunho/backlog | 1–2 sem. |
| 60 | `v1-01-generated-sdk-client-examples` | rascunho | 2–3 sem. |
| 60 | `v1-02-generated-sdk-black-box-ga` | rascunho/P1 | 2–3 sem. |
| 70 | `v1-01-legacy-characterization` | rascunho/backlog | 1–3 sem. |
| 70 | `v2-01-domain-foundation` | rascunho/backlog | 2–4 sem. |
| 70 | `v2-02-persistence-leases-accounting` | rascunho/backlog | 2–4 sem. |
| 70 | `v2-03-runtime-continuation` | rascunho/backlog | 2–4 sem. |
| 70 | `v2-04-tools-verification` | rascunho/backlog | 2–4 sem. |
| 70 | `v2-05-task-graph-subagents` | rascunho/backlog | 2–4 sem. |
| 70 | `v2-06-clients-projections` | rascunho/backlog | 2–4 sem. |
| 70 | `v2-07-recovery-rollout` | rascunho/backlog | 2–4 sem. |
| 80 | `v1-01-telegram-bridge-backlog` | rascunho/backlog | 2–4 sem. |
| 90 | `v1-01-full-duplex-backlog` | rascunho/backlog | 3–4 sem. |

### Corrective completion chain

- [20/v1-06 canonical SessionActor](20-tower-core/v1-06-canonical-session-actor-runtime/)
- [20/v1-07 lifecycle, metadata and recovery](20-tower-core/v1-07-lifecycle-metadata-recovery/)
- [30/v1-09 capability and product conformance](30-app-server/v1-09-capability-contract-product-conformance/)
- [40/v1-04 MCP contract and transports](40-mcp-control-plane/v1-04-mcp-contract-transport-completion/)
- [40/v1-05 token scopes and TLS](40-mcp-control-plane/v1-05-token-scopes-tls-release/)
- [50/v1-03 all nine tool semantics](50-tower-agent-tools/v1-03-nine-tool-semantic-completion/)
- [60/v1-02 generated SDK black-box GA](60-sdk-typescript/v1-02-generated-sdk-black-box-ga/)

### Complementary execution epics

- [E0 — build baseline](05-build-and-release/v1-01-build-baseline-instrumentation/)
- [E1 — product runtime](20-tower-core/v1-09-product-runtime-vertical-completion/)
- [E2 — lifecycle/recovery](20-tower-core/v1-10-lifecycle-recovery-hardening/)
- [E3 — App Server GA contract](30-app-server/v1-10-product-contract-capability-ga/)
- [E4 — MCP parity](40-mcp-control-plane/v1-06-parity-multisession/)
- [E5 — MCP security](40-mcp-control-plane/v1-07-security-scopes-tls-ga/)
- [E6 — nine tools GA](50-tower-agent-tools/v1-04-nine-tools-product-ga/)
- [E7 — SDK regeneration](60-sdk-typescript/v1-03-generated-sdk-black-box-regeneration/)
- [E8 — dependency slicing](05-build-and-release/v1-02-dependency-feature-slicing/)
- [E9 — profiles/linker/CI](05-build-and-release/v1-03-profiles-linker-cache-ci/)
- [E10 — dead code](05-build-and-release/v1-04-dead-code-experimental-paths/)
- [E11 — observability/faults](20-tower-core/v1-11-observability-fault-testing/)
- [E12 — release readiness](30-app-server/v1-11-release-readiness/)

### Grafo de dependências

```text
10-providers/v1-01 ─→ v1-02 ─────────────→ v1-03 OpenRouter
                                  ├───────→ v1-04 Groq
                                  └───────→ v1-05 Cloudflare

20-tower/v1-01 ─→ v1-02 ─→ v1-03 ───────────────────────→ v1-04 hardening
       │              │
       └────→ 30-app/v1-01 ─→ v1-02 ─→ v1-03 ─→ 50-tools/v1-01 ─┬→ 30-app/v1-04 ─→ v1-05 ─┐
                                                   │             │                  └────→ v1-06 ─┤
                                                   │             └→ 40-mcp/v1-01 ─→ v1-02       │
                                                   └────────────────→ 50-tools/v1-02             │
                                                                                                  │
                                                60-sdk/v1-01 ◄────────────────────────────────────┘

30/v1-05 + 30/v1-06 + 40/v1-02 + 50/v1-02 + 60/v1-01 ─→ 30/v1-07
30/v1-07 ─→ 30/v2-01 (dashboard, futuro), 80/v1-01, 90/v1-01
Goal v1 characterization ─→ Goal v2-01..07 (fora da critical path core)
```

Não há dependência do App Server/Tower sobre Goal v2. App Server apenas
inventaria hot paths e preserva uma facade onde o goal legado v1 e o futuro v2
possam ser selecionados por flag. [provenance: user-input]

## Contratos compartilhados

- [glossário e identidade](./_shared/session-turn-item-identity.md)
- [precedência das fontes de verdade](./_shared/source-of-truth.md)
- [índice completo D-*](./_shared/INDEX.md)
- [mapa de crates e dependências](./_shared/crate-map.md)
- [ownership do runtime](./_shared/runtime-ownership.md)
- [runtime facade](./_shared/runtime-facade.md)
- [Tower e lifecycle de instâncias](./_shared/tower-instance-lifecycle.md)
- [autenticação e threat model](./_shared/control-plane-security.md)
- [ordenação, replay e idempotência](./_shared/identity-event-ordering.md)
- [tools Tower](./_shared/tower-agent-tools.md)
- [MCP, transportes e CLI](./_shared/mcp-server-transport-cli.md)
- [approvals/controller/history](./_shared/approvals-controller-history.md)
- [SDK TypeScript](./_shared/typescript-sdk.md)
- [product runtime readiness](./_shared/product-runtime-readiness.md)
- [contract conformance/capability truth](./_shared/contract-conformance-capability-truth.md)
- [completion coverage](./COMPLETION_COVERAGE.md)
- [providers](./_shared/provider-contract.md)
- [Goal futuro](./_shared/goal-boundary.md)
- [freeze dashboard/ACP](./_shared/ui-freeze.md)
- [método TDD](./TDD.md)
- [rastreabilidade](./TRACEABILITY.md)

## Estrutura da documentação

```text
.llms/grok-build/
├── README.md
├── TDD.md
├── TRACEABILITY.md
├── _shared/
├── 10-providers/
├── 20-tower-core/
├── 30-app-server/
├── 40-mcp-control-plane/
├── 50-tower-agent-tools/
├── 60-sdk-typescript/
├── 70-goal-runtime/
├── 80-channel-gateways/
└── 90-realtime-voice/
```

Cada programa contém `README.md`, `SPECS.md`, `VISION.md` e epics
`vN-NN-kebab-case/`. Epic com mais de 15 tasks ou contrato local possui
`tasks.md`; contratos usados por dois ou mais programas vivem em `_shared/`.

## Princípios

1. **Session é canônico** — `thread` aparece somente ao citar/mapear Codex.
2. **Uma semantic core** — in-process, stdio, WebSocket e MCP usam a mesma facade.
3. **Sem segundo `SessionActor`** — Tower promove leader/registry existentes.
4. **MCP remoto e WebSocket early** — ambos pertencem ao MVP, não ao polish final.
5. **Segurança honesta** — bearer full-control sobre `http://`/`ws://` remoto é risco alto documentado, não escondido.
6. **ACL mínima** — só `orchestrator` acessa Tower por default; configuração pode ampliar.
7. **Dashboard congelado no MVP** — ACP/leader/roster continuam até programa v2 explícito.
8. **Compatibilidade seletiva** — dual-version obrigatória somente no Goal futuro.
9. **TDD por comportamento** — todo epic muda comportamento via Red-Green-Refactor e conformance real.
10. **Estado factual** — `planejado` não significa implementado; `concluído` exige evidência.

## Decisões de design

### DD-01 — numeração global por programa e local por epic

- **Decisão:** programas `10..90`; epics `vN-01-*` ordenáveis.
- **Rejeitado:** manter pastas antigas sem sequência, pois escondiam a ordem de PRs.
- **Status:** aceito. [provenance: user-input]

### DD-02 — Tower promove o leader

- **Decisão:** generalizar `connect_or_spawn`, leader server, roster e session
  lifecycle; nova Tower somente por flag explícita.
- **Rejeitado:** daemon paralelo, por duplicar ownership e sessões.
- **Status:** aceito. [provenance: user-input, code]

### DD-03 — segurança local permissiva; remote GA exige hardening

- **Decisão:** local/experimental preserva bearer full-control e cleartext
  explícito; produção remota exige token lifecycle, scopes e TLS/proxy
  verificado. Origin/pairing/multi-user continuam fora do core.
- **Rejeitado:** chamar warning+cleartext de remote-ready ou quebrar o modo
  local antes de uma migração de token.
- **Status:** completion target; scope UX/TLS têm gates humanos.
  [provenance: user-input, conversation, skill-output]

### DD-04 — MCP local não faz loop em si mesmo

- **Decisão:** orchestrator local chama tools in-process; config MCP serve para
  Towers externas e clientes externos.
- **Rejeitado:** auto-injetar a Tower local como seu próprio MCP server.
- **Status:** aceito. [provenance: user-input, inferred]

### DD-05 — BYOK estende o control plane multi-auth

- **Decisão:** API-key providers reutilizam credential store, binding imutável,
  catálogo credential-scoped e request-time bearer; custom TOML permanece.
- **Rejeitado:** wizard que grava secret em TOML e um terceiro sistema de auth.
- **Status:** recomendado e planejado. [provenance: doc-tree, inferred]

## Propostas residuais e decisões humanas

- `[PROPOSED]` SDK em `packages/grok-oss-app-server`, Node + browser WebSocket,
  sem publicação npm até o protocolo estabilizar.
- `[PROPOSED]` key MCP externa `grok-oss-tower` ou `tower-<instance-id>`.
- `(HUMAN)` aceitar explicitamente o threat model antes de habilitar bind público em uma release — type: `manual-verify`; blocking: release remoto, não planejamento.
- `(HUMAN)` fornecer credenciais para smokes live de providers e npm quando chegar a execução — type: `credential`; blocking: somente gates live/publish.

## Regras de status e execução

- Todos os epics novos começam `rascunho`; passam a `planejado` após review do
  plano; nenhum está `em progresso` por existir em disco.
- Cada epic deve caber em 1–4 semanas. Se não couber, o executor o divide antes
  de código, preservando contratos e dependências.
- Código só começa após autorização explícita e segue [TDD.md](./TDD.md).
- PRs de produto usam feature branch `goblin-*` e base `goblin`, nunca `main`.
- Findings duráveis usam `@issue-lifecycle`; execução usa `@execute-plan` e
  itens coerentes usam `@implementation-loop`.
