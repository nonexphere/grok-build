# Tower Agent Tools — SPECS

## 1. Surface

`list`, `start`, `send`, `history`, `interrupt`, `resume`, `archive`, `status`,
`wait`; schemas e errors no contrato compartilhado.

## 2. Execution

Uma Rust facade typed executa operations. MCP e tool runtime apenas convertem
input/output. Wait usa subscription/cursor e timeout, sem polling agressivo.

## 3. ACL

Default allow `orchestrator`; deny demais roles. Config é carregada pela Tower,
não mutável pelo modelo. Bearer client externo é gate distinto.

## 4. v2

Estuda messaging direto peer↔peer preservando ACL e mailbox, sem bloquear v1.

## 5. Semantic completion

Nenhuma tool usa placeholder/default para campo obrigatório. Todas preservam
metadata, structured input, epoch/cursor, state transitions, bounds, errors e
output schema conforme o contrato compartilhado.
