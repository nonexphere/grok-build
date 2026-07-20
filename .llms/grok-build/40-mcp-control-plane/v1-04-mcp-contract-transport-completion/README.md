# Epic v1-04 — Contrato e transportes MCP completos

Status: rascunho
Prioridade: P0 lançamento-bloqueante
Estimativa: 2–4 semanas
Depende de: ../v1-03-tower-product-runtime/, ../../50-tower-agent-tools/v1-03-nine-tool-semantic-completion/
Habilita: 40/v1-05, 30/v1-07
Skills relacionadas: @implementation-loop, @code-review, @human-product-test
Proveniência: [provenance: user-input, skill-output, code, doc-tree]

## Objetivo

Fechar validação, descriptors, error mapping, MCP stdio product path e Streamable HTTP/SSE contra a semantic core real.

## Escopo

### ADICIONAR

- input/output JSON Schema resolvível em tools/list;
- validation antes de dispatch e output conformance;
- stdio launcher real;
- HTTP POST/GET/DELETE/session/TTL/rebind suite product-backed;
- independent MCP client smoke.

### REFACTORIZAR

- ToolError/RuntimeError para envelope canônico;
- HTTP/stdio usam o mesmo dispatch typed;
- bearer posture segue o contrato de segurança.

### REMOVER

- $ref relativo sem resolução;
- parsing ad hoc/default silencioso no transport;
- import/helper stdio morto e parity helper-only.

### MANTÉM

- crate server separado do MCP client;
- self-loop local proibido.

## Contratos

- [MCP transport](../../_shared/mcp-server-transport-cli.md)
- [conformance](../../_shared/contract-conformance-capability-truth.md)
- [Tower tools](../../_shared/tower-agent-tools.md)

## Gate de saída

Cliente MCP independente descobre schemas válidos e executa as nove tools via stdio e HTTP real com resultados/erros equivalentes, incluindo SSE/reconnect/TTL.

