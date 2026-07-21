# Epic E11 — Observabilidade, limites e fault testing

Status: planejado/P1  
Escopo: ADICIONAR + REFACTORIZAR  
Owner: runtime/platform  
Depende de: [E1 product runtime](../v1-09-product-runtime-vertical-completion/), [E2 lifecycle](../v1-10-lifecycle-recovery-hardening/), [E3 App Server](../../30-app-server/v1-10-product-contract-capability-ga/), [E4 MCP parity](../../40-mcp-control-plane/v1-06-parity-multisession/), [E6 tools](../../50-tower-agent-tools/v1-04-nine-tools-product-ga/)  
Consumidores: operação, CI e release

## Tasks

- [ ] E11-01 instrumentar latency, queue, retry, timeout, reconnect e memory.
- [ ] E11-02 adicionar health/readiness/liveness sem secrets.
- [ ] E11-03 testar carga multi-session e backpressure.
- [ ] E11-04 injetar falhas de provider, actor, storage e transport.
- [ ] E11-05 adicionar secret canary para logs, errors, events e files.
- [ ] E11-06 testar graceful shutdown, crash recovery e cleanup.
- [ ] E11-07 definir SLOs, alertas e runbooks.

## Gate

Falhas são observáveis, bounded e recuperáveis; nenhum teste encontra secret
em saída pública ou persistência indevida.
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
