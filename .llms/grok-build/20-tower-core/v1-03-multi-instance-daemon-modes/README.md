# Epic v1-03 — Multi-instance e modos do daemon
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
Depende de: `../v1-02-multi-session-workspace-registry/`, `../../30-app-server/v1-03-core-in-process-stdio/`
Habilita: WS/MCP co-hosting e operações
Skills relacionadas: `@implementation-loop`, `@code-review`
Proveniência: [provenance: user-input, workspace, doc-tree, code, inferred]

## Arquitetura

Entrega N Towers/machine isoladas por instance ID, endpoint, token e state dir;
default conecta/spawna e nova instância exige flag. Modes: app-server, MCP ou
ambos (daemon completo default).

## Escopo

### ADICIONAR
- instance config/discovery, CLI lifecycle, endpoint collision checks e mode composition.

### REFACTORIZAR
- leader singleton paths para instance-scoped, preservando default compat.

### REMOVER
- implicit process-wide endpoint/state path.

### MANTÉM
- TUI default connect-or-spawn e dashboard ACP.

## Contratos

- [Tower lifecycle](../../_shared/tower-instance-lifecycle.md)
- [runtime ownership](../../_shared/runtime-ownership.md)

## TODO checklist

- [ ] Definir instance ID/state/endpoints/token refs
- [ ] RED default connect, absent spawn e explicit new Tower
- [ ] Implementar instance-scoped locks/socket/ports
- [ ] Implementar app-only, MCP-only e combined composition flags
- [ ] Testar duas Towers e Sessions no mesmo workspace sem cross-talk
- [ ] Testar endpoint/state collision fail-loud
- [ ] Testar restart/reconnect e graceful shutdown
- [ ] Documentar `[PROPOSED]` CLI names e migration

## Riscos e incertezas

- **[HIGH][Confirmed] cross-instance data leak:** explicit namespaces + tests.
- **[MEDIUM][Likely] UX ambiguity:** default instance deterministic; explicit flags elsewhere.
- **Human decision required:** aprovar nomes finais de flags — type: product-decision — blocking: CLI freeze.
