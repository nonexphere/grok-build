# Contract conformance and capability truth

**Fonte de verdade.** Este contrato impede divergência entre Rust protocol, JSON Schema, Tower tools, App Server, MCP, SDK e comportamento product-wired.

Proveniência: [provenance: user-input, skill-output, code, doc-tree, inferred].

## Precedência e geração

Rust protocol types e explicit semantic contracts são a fonte estrutural. Deles são gerados:

- App Server operational schema;
- nove input/output schemas Tower;
- MCP tools/list descriptors com schemas resolvíveis;
- TypeScript declarations;
- golden requests, responses e errors.

Código handwritten pode implementar clients/adapters, mas não redefinir enums, required fields, defaults, errors, operation results ou capabilities.

## Validação obrigatória

Todo adapter valida input antes de lookup/efeito e output antes da conformance release gate. Unknown fields, missing required, invalid conditional fields, oversized payloads e cursor/epoch inválidos falham com erro estável.

Nenhum adapter pode:

- usar default para campo required;
- ignorar model, provider binding, sandbox, filters, cursor ou bounds;
- inventar agentType, residency, epoch, event sequence ou status;
- reduzir structured input ao primeiro text block;
- hardcode safe-looking placeholder como verdade de produto.

## Error envelope

Erros de domínio preservam code, message segura, retryable, operationId opcional e safe details typed. MCP usa isError structured content; App Server usa JSON-RPC error data; in-process usa o mesmo enum typed. A equivalência é semântica e testada.

## Capability negotiation

Initialize resulta de uma capability registry ligada ao composition root. Build features, runtime dependencies e release mode determinam capabilities. FakeRuntime e testes não alteram a matriz de produto.

## Differential conformance

Uma fixture canônica executa success/error para cada method/tool em:

- runtime facade;
- App Server in-process;
- App Server stdio;
- App Server WebSocket;
- MCP stdio;
- MCP Streamable HTTP;
- tools in-process;
- SDK TypeScript.

A comparação inclui IDs, states, errors, retryability, ordering, epoch, cursors, redaction e idempotency. Transport-only fields podem ser normalizados por regra declarada.

## CI gates

- generated artifacts byte-for-byte;
- zero unresolved external $ref em MCP tools/list;
- schema-valid outputs;
- capability matrix product-wired;
- no zero-test filters;
- fake conformance e product integration reportados separadamente;
- skip/blocked nunca conta como pass.

