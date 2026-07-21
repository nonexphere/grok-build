# Programa 05 — Build, Release e Qualidade Operacional

Status: em progresso  
Owner: build/release + owners dos crates consumidores  
Objetivo: reduzir o custo de compilação e tornar os gates de distribuição reproduzíveis.

## Epics

- [v1-01-build-baseline-instrumentation](./v1-01-build-baseline-instrumentation/)
- [v1-02-dependency-feature-slicing](./v1-02-dependency-feature-slicing/)
- [v1-03-profiles-linker-cache-ci](./v1-03-profiles-linker-cache-ci/)
- [v1-04-dead-code-experimental-paths](./v1-04-dead-code-experimental-paths/)

Os epics deste programa não podem alterar comportamento público sem contract
test e benchmark antes/depois.
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
