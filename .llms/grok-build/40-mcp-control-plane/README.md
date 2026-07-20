# 40 — MCP Control Plane

## O que é

MCP server da Tower para clients locais e remotos. O crate `xai-grok-mcp`
atual é client; este programa adiciona server sem confundir os papéis.

## Estado atual

MCP client possui stdio/HTTP/OAuth/liveness; não há servidor de controle de
sessions nem Streamable HTTP/SSE da Tower.

## Issues conhecidos

- risco de duplicar semantic core de App Server;
- remote bearer permissivo exige threat model e limits reais;
- SSE legado vs Streamable HTTP precisa compat explícita.

## Epics

- [v1-01-server-transports](./v1-01-server-transports/)
- [v1-02-remote-security-conformance](./v1-02-remote-security-conformance/)
- [v1-03-tower-product-runtime](./v1-03-tower-product-runtime/) — supervisor combinado
- [v1-04-mcp-contract-transport-completion](./v1-04-mcp-contract-transport-completion/) — schema/stdio/HTTP completos
- [v1-05-token-scopes-tls-release](./v1-05-token-scopes-tls-release/) — auth lifecycle e remote release
