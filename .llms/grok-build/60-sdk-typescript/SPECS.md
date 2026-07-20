# SDK TypeScript — SPECS

## 1. Fonte

Rust protocol types geram JSON Schema e declarations. Handwritten client usa
tipos gerados; CI falha em drift.

## 2. Client

Initialize, reconnect, request correlation, notifications, async item stream,
interrupt e typed errors. `[PROPOSED]` Node e browser WS no mesmo package.

## 3. Examples

Script inicia/resume Session, envia Turn, lê Items/history e interrompe. MCP
example fica no programa MCP, não duplica SDK protocol.

## 4. Distribution

`[PROPOSED] packages/grok-oss-app-server`; uso local no MVP, publish npm após freeze.

## 5. Completion target

Types e method bindings são gerados, não mirror manual. Clean regeneration é
determinística e o client Node passa start/send/stream/history/interrupt contra
stdio e WebSocket product-wired antes de GA.
