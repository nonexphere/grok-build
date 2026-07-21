# Epic E12 — Release readiness e validação final

Status: planejado/P0  
Escopo: ADICIONAR + REFACTORIZAR  
Owner: release captain + owners dos programas 20–60  
Depende de: [E1](../../20-tower-core/v1-09-product-runtime-vertical-completion/), [E2](../../20-tower-core/v1-10-lifecycle-recovery-hardening/), [E3](../v1-10-product-contract-capability-ga/), [E4](../../40-mcp-control-plane/v1-06-parity-multisession/), [E5](../../40-mcp-control-plane/v1-07-security-scopes-tls-ga/), [E6](../../50-tower-agent-tools/v1-04-nine-tools-product-ga/), [E7](../../60-sdk-typescript/v1-03-generated-sdk-black-box-regeneration/), [E8/E9/E10](../../05-build-and-release/), [E11](../../20-tower-core/v1-11-observability-fault-testing/)  
Consumidores: distribuição `grok-oss`

## Tasks

- [ ] E12-01 executar completion audit contra este documento e `COMPLETION_COVERAGE`.
- [ ] E12-02 validar cargo fmt/check/clippy/test e todos os targets necessários.
- [ ] E12-03 validar build debug/release/release-dist e plataformas suportadas.
- [ ] E12-04 validar CI cache, `--locked`, artifacts e required checks.
- [ ] E12-05 executar smoke humano do binário `grok-oss`.
- [ ] E12-06 revisar segurança, secrets, permissões e documentação.
- [ ] E12-07 registrar gaps externos, human gates e rollback.
- [ ] E12-08 emitir verdict PASS/PARTIAL/BLOCKED com evidência.

## Gate

Nenhum requisito fica implícito: cada item é proven, blocked, deferred ou
superseded com owner, evidência e condição de encerramento.
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
