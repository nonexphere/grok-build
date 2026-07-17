# Grok Build — Goal Runtime e App Server

Snapshot de planejamento: `b189869` em 2026-07-16.

Esta árvore transforma as especificações em `changes/` em dois programas de
entrega coordenados. O **Goal Runtime** torna `/goal` um runtime transacional,
verificável e recuperável. O **App Server** expõe o runtime Grok a múltiplos
clientes através de um protocolo Thread/Turn/Item sem duplicar o `SessionActor`.

## Roadmap

```text
Goal:  caracterização+domínio → persistência → runtime → tools+verificação
                         └────→ tasks+subagents ───────→ clientes → rollout

App:   arquitetura+protocolo → facade+projeção → core ─→ histórico
                                             └────────→ approvals
               histórico + approvals ─→ daemon ─→ TUI ─→ ecossistema+GA

Integração: Goal clientes/projeções ────────────────→ App ecossistema+GA
```

O Goal Runtime pode ser entregue e operar sem o App Server. O App Server
consome o `GoalService` e seus eventos por uma facade; nunca passa a ser fonte
de verdade do lifecycle do goal.

## Grupos e épicos

| Grupo | Epic | Status | Dependências principais |
|---|---|---|---|
| Goal Runtime | [v1-characterization-domain](./goal-runtime/v1-characterization-domain/) | rascunho | nenhuma |
| Goal Runtime | [v1-persistence-leases-accounting](./goal-runtime/v1-persistence-leases-accounting/) | rascunho | characterization-domain |
| Goal Runtime | [v1-runtime-continuation](./goal-runtime/v1-runtime-continuation/) | rascunho | persistence-leases-accounting |
| Goal Runtime | [v1-tools-verification](./goal-runtime/v1-tools-verification/) | rascunho | runtime-continuation |
| Goal Runtime | [v1-task-graph-subagents](./goal-runtime/v1-task-graph-subagents/) | rascunho | runtime-continuation, tools-verification |
| Goal Runtime | [v1-clients-projections](./goal-runtime/v1-clients-projections/) | rascunho | tools-verification, task-graph-subagents |
| Goal Runtime | [v1-recovery-rollout](./goal-runtime/v1-recovery-rollout/) | rascunho | todos os Goal v1 anteriores |
| App Server | [v1-architecture-protocol](./app-server/v1-architecture-protocol/) | rascunho | nenhuma |
| App Server | [v1-runtime-facade-projection](./app-server/v1-runtime-facade-projection/) | rascunho | architecture-protocol |
| App Server | [v1-core-in-process](./app-server/v1-core-in-process/) | rascunho | runtime-facade-projection |
| App Server | [v1-history-replay](./app-server/v1-history-replay/) | rascunho | core-in-process |
| App Server | [v1-approvals-control](./app-server/v1-approvals-control/) | rascunho | core-in-process |
| App Server | [v1-daemon-transports-security](./app-server/v1-daemon-transports-security/) | rascunho | history-replay, approvals-control |
| App Server | [v1-tui-migration](./app-server/v1-tui-migration/) | rascunho | daemon-transports-security |
| App Server | [v1-ecosystem-ga](./app-server/v1-ecosystem-ga/) | rascunho | TUI migration, Goal clients/projections |

## Contratos compartilhados

- [runtime-ownership.md](./_shared/runtime-ownership.md)
- [identity-event-ordering.md](./_shared/identity-event-ordering.md)
- [leases-idempotency.md](./_shared/leases-idempotency.md)
- [security-authority-boundaries.md](./_shared/security-authority-boundaries.md)

## Rastreabilidade das especificações

| Fonte | Seções/fases | Épico responsável |
|---|---|---|
| Goal spec | §3–5.2, Phase 0–1 | `goal-runtime/v1-characterization-domain` |
| Goal spec | §5.3, §10, Phase 2 | `goal-runtime/v1-persistence-leases-accounting` |
| Goal spec | §5.4, §9, Phase 3 | `goal-runtime/v1-runtime-continuation` |
| Goal spec | §4, §5.5–5.6, Phase 4–5 | `goal-runtime/v1-tools-verification` |
| Goal spec | §3.5, §6.4–6.7, Phase 6 | `goal-runtime/v1-task-graph-subagents` |
| Goal spec | §7, §11.3–11.6, Phase 7 | `goal-runtime/v1-clients-projections` |
| Goal spec | §6.7, §11, Phase 8, §12.3–12.5 | `goal-runtime/v1-recovery-rollout` |
| App spec | §2–7, Phase 0–1 | `app-server/v1-architecture-protocol` |
| App spec | §3–6, §12.1, Phase 2 | `app-server/v1-runtime-facade-projection` |
| App spec | §7, §12.2–12.3, Phase 3 | `app-server/v1-core-in-process` |
| App spec | §7.11, §13, Phase 4 | `app-server/v1-history-replay` |
| App spec | §7.9–7.10, §10, Phase 5 | `app-server/v1-approvals-control` |
| App spec | §8–10, Phase 6 | `app-server/v1-daemon-transports-security` |
| App spec | §14, Phase 7 | `app-server/v1-tui-migration` |
| App spec | §15–24, Phase 8–9 | `app-server/v1-ecosystem-ga` |

## Capacidade e duração

- Cada épico foi desenhado para **1–4 semanas de calendário** com ownership
  claro; estimativas em person-weeks das specs continuam válidas.
- Épicos com workstreams grandes (`ecosystem-ga`, recovery e TUI) exigem
  execução paralela para respeitar essa janela; sem staffing, devem ser
  replanejados antes de iniciar, não prolongados silenciosamente.
- Foundations dos dois grupos podem iniciar em paralelo. Integrações somente
  começam após gates das dependências.

## Regras de execução e encerramento

1. `rascunho → planejado` exige decisões humanas bloqueantes resolvidas ou
   branches explicitamente adiadas.
2. `em progresso → concluído` exige todas as tasks obrigatórias, Gate de saída,
   validação e delivery evidence.
3. Finding novo é persistido via `@issue-lifecycle`; não vira TODO nu em código.
4. Mudança de contrato atualiza a fonte canônica e seus consumers no mesmo epic.
5. Nenhum executor reduz MUST/MUST NOT das fontes em `changes/` sem decisão humana.

## Estrutura

```text
.llms/grok-build/
├── README.md
├── _shared/
├── goal-runtime/
│   ├── README.md
│   ├── SPECS.md
│   ├── VISION.md
│   └── v1-*/
└── app-server/
    ├── README.md
    ├── SPECS.md
    ├── VISION.md
    └── v1-*/
```

## Princípios

1. **Uma autoridade por estado** — `GoalRuntime` governa goals; runtime Grok governa execução; App Server governa protocolo e clientes.
2. **Sem segundo SessionActor** — transportes e adapters compartilham um único registry/runtime facade.
3. **Fail-closed em autoridade** — conclusão, approvals, recovery e permissões nunca avançam por ausência de evidência.
4. **Estado durável antes de projeção** — UI, ACP, JSONL e Thread/Turn/Item são projeções, não lifecycle truth.
5. **Compatibilidade por adapters** — formatos antigos são aceitos em bordas versionadas, não contaminam o domínio novo.
6. **Local e seguro por padrão** — remote control permanece desligado e exige autenticação, scopes e Origin/TLS.
7. **TUI parity é release gate** — nenhuma migração estável pode reduzir comportamento ou desempenho materialmente.
8. **Evidência sobre alegações** — testes devem atingir entrypoints reais, incluindo concorrência, crash e replay.

## Decisões de design

### DD-1: uma árvore, dois grupos

- **Decisão:** manter Goal Runtime e App Server sob uma raiz, com épicos separados.
- **Contexto:** pertencem ao mesmo repositório e compartilham runtime, eventos, TUI, subagents e worktrees.
- **Rejeitado:** duas árvores independentes, pois duplicariam contratos e esconderiam dependências de integração.
- **Status:** aceito. [provenance: user-input, inferred]

### DD-2: dependência unidirecional

- **Decisão:** o App Server pode consumir `GoalService`; o Goal Runtime não depende do App Server.
- **Rejeitado:** tornar o daemon necessário para `/goal`, porque quebraria TUI/headless e recovery local.
- **Status:** aceito. [provenance: doc-tree]

### DD-3: 15 épicos em vez de uma pasta por fase original

- **Decisão:** agrupar fases adjacentes com o mesmo gate e ownership.
- **Rejeitado:** 19 microépicos que fragmentariam vertical slices e excederiam o limite operacional do planejamento.
- **Status:** aceito. [provenance: skill-output, inferred]

## Fontes

- `changes/grok-build-goal-runtime-technical-spec (1).md`
- `changes/grok_app_server_spec_bundle/grok_app_server_plan_and_spec.md`
- bundle de schema, TypeScript e exemplos em `changes/grok_app_server_spec_bundle/`

**Próximo passo:** resolver decisões humanas dos épicos iniciais e executar
primeiro as duas foundations em paralelo; não iniciar integrações antes de seus
gates.
