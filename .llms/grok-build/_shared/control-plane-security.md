# Control Plane Security and Threat Model

**Fonte de verdade.** Tower Core é dona do bind/token lifecycle; App Server e
MCP aplicam o mesmo gate; runtime Grok continua dono de sandbox e permissions.

## Contrato MVP travado

| Dimensão | Regra |
|---|---|
| Auth HTTP/WS/MCP | `Authorization: Bearer <token>` |
| Poder do token | full control; sem scopes finos |
| Rede | loopback, LAN ou internet; bind não-loopback requer flag explícita |
| Transporte | `http://` e `ws://` permitidos; TLS não obrigatório |
| Origin | sem allowlist no MVP |
| Redaction | bearer, provider keys e secrets nunca em logs/history/events |
| Revogação | token revogável/rotacionável; conexão revalidada conforme contrato |

## Threat model honesto

Bearer em cleartext atravessando rede pode ser capturado. Como o token tem
controle total, captura equivale a controlar sessions, turns, tools e arquivos
dentro das permissões do processo. Ausência de Origin allowlist também permite
clientes browser se conectarem se obtiverem token/endpoint. Estas são decisões
humanas de MVP, não propriedades seguras por default. [provenance: user-input]

Mitigações obrigatórias que não contradizem o contrato:

- default de listen loopback; exposição pública sempre explícita;
- warning forte ao usar bind não-loopback sem TLS proxy;
- tokens de alta entropia, arquivo owner-only, rotation/revoke e comparação constant-time;
- nunca aceitar token em query string, argv persistido, log ou payload de erro;
- limites de frame/request/history/queue e backpressure;
- redaction de secrets inclusive prefixes/suffixes canary;
- structured audit metadata sem payload secreto;
- runtime sandbox/hooks/approvals continuam vigentes após autenticação.

## ACL do modelo

Auth do client e ACL do agent são gates independentes. Tools in-process usam
agent type: `orchestrator` allow por default; todos os demais deny. Config pode
adicionar tipos, mas uma session não altera sua própria ACL.

## Pós-MVP

TLS nativo/proxy helpers, Origin allowlist, scopes, pairing, expiração curta e
rate policies são hardening futuro. Não devem ser documentados como existentes
no MVP.

