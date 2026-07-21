# Epic E9 — Profiles, linker, cache e CI

Status: planejado  
Escopo: ADICIONAR + REFACTORIZAR  
Owner: build/release  
Depende de: [E0 build baseline](../v1-01-build-baseline-instrumentation/), [E8 dependency slicing](../v1-02-dependency-feature-slicing/)  
Consumidores: CI, release e desenvolvimento local

## Tasks

- [ ] E9-01 comparar profiles dev, conformance, product-integration e release.
- [ ] E9-02 medir `lto`, `codegen-units`, incremental, debug e panic.
- [ ] E9-03 avaliar `sccache`/rustc-wrapper e chaves de cache corretas.
- [ ] E9-04 avaliar mold/lld por target, sem impor ferramenta ausente.
- [ ] E9-05 separar jobs fake, product, release e cross-target.
- [ ] E9-06 evitar rebuild duplicado entre jobs e features.
- [ ] E9-07 adicionar cargo timings e diagnóstico de memória ao CI.
- [ ] E9-08 auditar build scripts, downloads, checksum e modo offline.
- [ ] E9-09 validar `--locked`, artifact reuse e required checks no GitHub.

## Acceptance criteria

CI reproduz os gates, usa cache seguro, diferencia falha de código de timeout
de infraestrutura e publica evidência de build sem expor secrets.
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
