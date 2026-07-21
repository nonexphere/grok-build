# Epic E0 — Build baseline e instrumentação

Status: em progresso  
Escopo: ADICIONAR  
Owner: build/release  
Depende de: nenhum  
Consumidores: E8, E9 e todos os epics de runtime

## Objetivo

Criar uma medição reproduzível de compilação, link, memória, tamanho de artefato
e recompilação incremental para distinguir gargalo real de percepção local.

## Tasks

- [ ] E0-01 registrar toolchain, target, CPU, RAM, linker, Cargo config e SHA.
- [ ] E0-02 medir cold/warm/incremental para App Server, MCP, Tower, Shell e `grok-oss`.
- [ ] E0-03 gerar `cargo --timings` por cenário e preservar os relatórios.
- [ ] E0-04 medir build scripts, crates nativos, link e pico de memória.
- [ ] E0-05 medir tamanho, símbolos, debug info e startup do binário.
- [ ] E0-06 documentar comandos idempotentes para reproduzir a matriz.
- [ ] E0-07 adicionar gate que falhe quando o baseline não puder ser coletado.

## Acceptance criteria

Há três execuções por cenário, artefatos versionáveis fora do source gerado,
comparação estatística e nenhuma meta de performance sem evidência.

## Validação

`cargo build --timings`, `cargo check`, `cargo test` por crate, `/usr/bin/time -v`
e inspeção do artefato final.
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
