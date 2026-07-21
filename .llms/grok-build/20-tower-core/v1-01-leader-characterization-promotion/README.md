# Epic v1-01 — Caracterizar e promover o leader
Owner: Tower core/runtime owners
Escopo: conforme a seção Escopo deste epic

## Revisão de implementação

Este epic só pode ser executado quando cada task tiver owner, arquivos ou
contrato afetado, pré-condição, comando de validação e evidência esperada.
Alterações de comportamento exigem Red-Green-Refactor; alterações de contrato
exigem contract test e atualização da matriz de rastreabilidade.

### Gate mínimo

- [ ] dependências e links deste epic foram verificados;
- [ ] interfaces, schemas, estados, erros e compatibilidade estão definidos;
- [ ] caminho fake/conformance está separado do caminho product-backed;
- [ ] testes unitários, integração, black-box e segurança foram classificados;
- [ ] timeout, cancelamento, retry, restart e falhas parciais foram tratados;
- [ ] observabilidade, limites de recurso e redaction foram especificados;
- [ ] comando reproduzível e artefato de evidência foram registrados;
- [ ] bloqueios humanos/externos possuem owner e condição de desbloqueio;
- [ ] status do epic foi reconciliado com `TRACEABILITY.md` e `COMPLETION_COVERAGE.md`.
Status: rascunho
Prioridade: lançamento-bloqueante
Estimativa: 2–3 semanas
Depende de: nenhuma
Habilita: `v1-02`, `30-app-server/v1-01`
Skills relacionadas: `@repository-exploration`, `@implementation-loop`, `@code-review`
Proveniência: [provenance: user-input, workspace, doc-tree, code, inferred]

## Arquitetura

Caracteriza `connect_or_spawn`, `run_leader`, `run_leader_server`, locks,
protocol routing, roster e SessionActor ownership; extrai uma Tower facade sem
alterar TUI/ACP behavior.

## Escopo

### ADICIONAR
- characterization tests, ownership map, Tower instance types e facade seam.

### REFACTORIZAR
- nomenclatura/entrypoint leader internamente atrás de compat adapter, preservando wire.

### REMOVER
- nenhum leader/ACP path nesta fase.

### MANTÉM
- connect-or-spawn default, dashboard roster e session persistence.

## Contratos

- [Tower lifecycle](../../_shared/tower-instance-lifecycle.md)
- [runtime ownership](../../_shared/runtime-ownership.md)

## TODO checklist

- [ ] RED/characterization de connect, spawn, lock handoff e reconnect — Follow @repository-exploration
- [ ] Mapear owners e consumers de leader/socket/roster/session lifecycle
- [ ] Provar exatamente um SessionActor por loaded Session
- [ ] Definir Tower facade/instance identity sem novo daemon
- [ ] Adaptar entrypoints preservando ACP bytes e dashboard
- [ ] Testar binary replacement/readiness/relaunch behavior
- [ ] Rodar shell/pager focused tests e `cargo fmt --check`
- [ ] Documentar decisões e hot paths de Goal tocados

## Riscos e incertezas

- **[HIGH][Confirmed] duplicate control plane:** reuse e characterization gate.
- **[HIGH][Likely] lock/reconnect regression:** deterministic integration tests.
- **UNVERIFIED:** nomes finais de modules/crates só após ownership spike.
