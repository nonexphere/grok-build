# Epic E5 — Auth, scopes, TLS e MCP release hardening

Status: planejado/P1  
Escopo: ADICIONAR + REFACTORIZAR  
Owner: MCP/control-plane security  
Depende de: [E4 MCP parity](../v1-06-parity-multisession/)  
Consumidores: deployment, operadores e clientes remotos

## Tasks

- [ ] E5-01 implementar token create/list/revoke.
- [ ] E5-02 aplicar scopes por método, agent type e sessão.
- [ ] E5-03 testar corrida de revogação e sessões já estabelecidas.
- [ ] E5-04 rejeitar bearer inseguro em query string conforme política.
- [ ] E5-05 completar TLS/proxy TLS e bind seguro.
- [ ] E5-06 adicionar rate/payload/concurrency limits.
- [ ] E5-07 publicar threat model, runbook e human gate.

## Gate

Allow/deny/revoke é determinístico, fail-closed e validado em cenário remoto
TLS; cleartext não é promovido a GA.
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
