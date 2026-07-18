# 50 — Tower Agent Tools

## O que é

Contrato e implementação compartilhada da família `tower_agent_*`, exposta ao
orchestrator in-process e a clients MCP.

## Estado atual

Subagents depth=1 e operations de leader/roster existem em superfícies
fragmentadas; não há tools first-class de peer top-level Session.

## Issues conhecidos

- agent role ACL ainda não protege Tower operations;
- history/wait/redaction não têm contrato único;
- tools MCP e internas podem divergir se implementadas separadamente.

## Epics

- [v1-01-tool-contract-and-facade](./v1-01-tool-contract-and-facade/)
- [v1-02-in-process-acl-mcp-parity](./v1-02-in-process-acl-mcp-parity/)
- [v2-01-peer-messaging-study](./v2-01-peer-messaging-study/)

