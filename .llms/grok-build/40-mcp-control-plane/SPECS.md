# MCP Control Plane — SPECS

## 1. Server

Expõe MCP stdio/local e Streamable HTTP com SSE compatibility quando necessária.
Initialize/capabilities, tool list/call, health e shutdown usam a Tower facade.

## 2. Auth e rede

HTTP exige bearer full-control. Cleartext e bind remoto são permitidos conforme
[`control-plane-security`](../_shared/control-plane-security.md).

## 3. Tools

Espelha [`tower-agent-tools`](../_shared/tower-agent-tools.md); nenhuma tool
possui lógica divergente.

## 4. Validação

Mesmos fixtures e outcomes do in-process path, mais framing/reconnect/
backpressure/auth/size conformance.

## 5. Completion target

Descriptors publicam schemas resolvíveis, input/output são validados e errors
preservam retryability/operationId. Stdio e Streamable HTTP passam por cliente
MCP independente. Remote GA adiciona token lifecycle, scopes e TLS; o bearer
full-control cleartext permanece somente modo experimental/local.
