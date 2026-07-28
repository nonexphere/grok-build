# Tower Core — SPECS

## 1. Ownership

Tower reutiliza leader process, session registry e SessionActor. Não contém
protocol semantics específicas de MCP/SDK.

## 2. Instances

Instance possui ID, endpoints, state dir, token ref e lifecycle. Default
connect-or-spawn; `--new-tower`/`--tower-id`/endpoint explícito cria ou escolhe
outra instância. [PROPOSED nomes de flags]

## 3. Sessions

Registry suporta resident/dormant/archived e workspace por Session. Sem hard
cap MVP; métricas atual/pico são best-effort sem payload sensível.

## 4. Modes

Daemon completo por default; flags permitem App Server-only, MCP-only ou ambos,
sempre sobre o mesmo registry/facade.

## 5. Validação

Actor uniqueness, multi-client/session/workspace/instance, restart/reconnect e
resource bounds conforme [TDD](../TDD.md).

