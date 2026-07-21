# Epic E8 — Grafo de dependências e feature slicing

Status: planejado  
Escopo: REFACTORIZAR  
Owner: owners de Shell, Pager e composition root  
Depende de: [E0 build baseline](../v1-01-build-baseline-instrumentation/)  
Consumidores: [E9 profiles/linker/cache/CI](../v1-03-profiles-linker-cache-ci/), CI e desenvolvimento local

## Objetivo

Reduzir recompilações e units desnecessárias sem remover capacidade necessária
do produto nem alterar contratos de segurança.

## Tasks

- [ ] E8-01 gerar grafo de dependências/feature por target crítico.
- [ ] E8-02 separar dependências de teste, examples e clients de produção.
- [ ] E8-03 avaliar features default de `xai-grok-pager-bin`.
- [ ] E8-04 avaliar isolamento do composition root Shell/Pager.
- [ ] E8-05 localizar dependências nativas pesadas e duplicações de versão.
- [ ] E8-06 revisar três bins com o mesmo `src/main.rs`.
- [ ] E8-07 aplicar apenas cortes que preservem os gates funcionais.
- [ ] E8-08 comparar tempo, memória, tamanho e número de crates antes/depois.

## Acceptance criteria

Cada redução tem benchmark, justificativa de compatibilidade e todos os gates
App Server/MCP/Tower e product smoke continuam verdes.

## Riscos

Não usar `default-features = false` ou remover dependência nativa sem validar
platforms, sandbox, auth e release.
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
