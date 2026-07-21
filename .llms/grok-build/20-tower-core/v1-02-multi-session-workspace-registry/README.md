# Epic v1-02 — Registry multi-session e multi-workspace
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
Estimativa: 2–4 semanas
Depende de: `../v1-01-leader-characterization-promotion/`
Habilita: `v1-03`, `30/v1-02`, `50/v1-01`
Skills relacionadas: `@implementation-loop`, `@code-review`
Proveniência: [provenance: user-input, workspace, doc-tree, code, inferred]

## Arquitetura

Formaliza resident/dormant/archived Session registry sobre roster/files e
permite `start(workspace)` em qualquer path autorizado pela Tower.

## Escopo

### ADICIONAR
- typed registry API, atomic pending loads, workspace/session metadata, fork/resume/archive.

### REFACTORIZAR
- `x.ai/sessions/*` e dashboard passam a observar o registry sem migração de protocolo.

### REMOVER
- qualquer implicit assumption Tower=cwd atual.

### MANTÉM
- arquivos de sessão como verdade e subagents depth=1.

## Contratos

- [Tower lifecycle](../../_shared/tower-instance-lifecycle.md)
- [runtime ownership](../../_shared/runtime-ownership.md)

## TODO checklist

- [ ] Characterization resident/dormant roster e session files
- [ ] RED para concurrent duplicate resume/start
- [ ] Implementar shared pending load/unique handle
- [ ] Implementar workspace canonicalization e permission boundary
- [ ] Definir start/resume/fork/archive/status typed operations
- [ ] Testar N sessions em workspaces distintos no mesmo processo
- [ ] Preservar dashboard `x.ai/sessions/list|changed`
- [ ] Medir current/peak resource telemetry sem enforcement
- [ ] Testar no hard cap e resource bound do registry
- [ ] Atualizar runbook/SPECS

## Riscos e incertezas

- **[HIGH][Confirmed] actor duplicado:** atomic registry/pending future.
- **[HIGH][Confirmed] path authority:** canonicalization + existing sandbox/policy.
- **[MEDIUM][Possible] resource exhaustion sem cap:** bounds internos + telemetry, sem quota claim.
