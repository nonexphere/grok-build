# Epic v1-01 — MCP server local e remoto early
Owner: MCP/control-plane owners
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
Depende de: `../../20-tower-core/v1-03-multi-instance-daemon-modes/`, `../../30-app-server/v1-03-core-in-process-stdio/`, `../../50-tower-agent-tools/v1-01-tool-contract-and-facade/`
Habilita: `v1-02`, clients externos e Towers externas
Skills relacionadas: `@implementation-loop`, `@code-review`
Proveniência: [provenance: user-input, workspace, doc-tree, code, inferred]

## Arquitetura

Novo MCP server adapter chama a Tower tools facade. Entrega stdio/local e
Streamable HTTP/SSE remoto no mesmo MVP; não reutiliza o MCP client como server.

## Escopo

### ADICIONAR
- server initialize/list/call, stdio framing, HTTP/SSE sessions, health e shutdown.

### REFACTORIZAR
- compartilhar tipos/transport primitives úteis com `xai-grok-mcp` sem misturar roles.

### REMOVER
- nenhuma funcionalidade MCP client.

### MANTÉM
- semantics na facade Tower e bearer common gate.

## Contratos

- [Tower tools](../../_shared/tower-agent-tools.md)
- [control-plane security](../../_shared/control-plane-security.md)

## TODO checklist

- [ ] RED server initialize/tool list/call via in-memory transport
- [ ] Definir crate/module boundary client vs server
- [ ] Implementar MCP stdio lifecycle/framing
- [ ] Implementar Streamable HTTP + SSE compatibility/reconnect
- [ ] Integrar tool registry facade sem logic fork
- [ ] Co-host modes app-only/MCP-only/both
- [ ] Testar remote early, not local-only
- [ ] Testar shutdown/half-close/slow client/message limits
- [ ] Conformance MCP result vs in-process tool result
- [ ] Docs client config local/remote e `[PROPOSED]` server key

## Riscos e incertezas

- **[HIGH][Confirmed] semantic fork:** differential conformance.
- **[HIGH][Likely] streaming lifecycle:** protocol fixtures + reconnect tests.
- **UNVERIFIED:** versão MCP SDK/crate escolhida na implementação.
