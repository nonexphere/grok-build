# Epic v1-03 — MCP no supervisor Tower
Owner: MCP/control-plane owners
Escopo: conforme a seção Escopo deste epic

Status: parcial — transportes/supervisor concluídos; runtime de turnos ainda pendente
Escopo: REFACTORIZAR + ADICIONAR
Depende de: `../../20-tower-core/v1-05-tower-supervisor/`
Contrato: [supervisor compartilhado](../../_shared/tower-command-runtime.md)

## Objetivo

Tornar o MCP Streamable HTTP uma unidade controlável pelo `grok-oss tower`,
com bind independente, auth compartilhada, `tools/list` verificável e shutdown
coordenado.

## Tasks

> Gate de prontidão: MCP está validado contra a facade e os transportes, mas a
> vertical de mutação continua limitada pelo runtime shell-backed. `tools/list`
> e initialize não são evidência de um Tower product-ready enquanto a factory
> ACP/SessionActor não estiver integrada e testada com o mock de inferência.

- [x] Expor factory de listener que aceite bind/secret/Tower ID validados.
- [x] Retornar handle/join observável ao supervisor sem bloquear o runtime.
- [x] Preservar `initialize`, `Mcp-Session-Id`, `tools/list` e ACL.
- [x] Testar MCP-only e combined com `--no-app-server`.
- [x] Testar falha de bind e limpeza de sessão/tarefa.
- [x] Testar SIGINT/SIGTERM sem orphan listener.
- [x] Registrar evidência HTTP real de initialize/tools/list/401.
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
