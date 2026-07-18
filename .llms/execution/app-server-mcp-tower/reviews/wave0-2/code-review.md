# Independent code review — Waves 0–2 (+ early tools/MCP/WS)

**Date:** 2026-07-18  
**Branch:** `goblin-implement-epic-tree`  
**Reviewer:** independent subagent (read-only)  
**Primary triage:** implementer agent

## Verdict (post-triage)

| Wave | Verdict | Notes |
|---|---|---|
| 0 | PASS | Protocol freeze, leader single-winner, handshake, provider matrix |
| 1 | PARTIAL | Facade/registry/projection/fake real; **Shell `GrokRuntimeFacade` adapter still marker-only** |
| 2 | PASS (FakeRuntime) | Processor + in-process + stdio vertical slice |
| Tools/MCP/WS early | PARTIAL | Semantic core + ACL; WS not a full server; MCP no Streamable HTTP |

## Findings triage

| ID | Severity | Status |
|---|---|---|
| B-1 Shell adapter missing | Blocking for production | OPEN — next wave priority |
| H-1 bearer not constant-time | High | **FIXED** — `constant_time_eq` in websocket.rs |
| H-2 redaction suffix leak | High | **FIXED** — whole-token redaction |
| H-3 send mode ignored | High | **FIXED** — new_turn / steer_active branch |
| M-1 idempotency conflict | Medium | **FIXED** — digest compare in FakeRuntime |
| M-2..M-6 tool/wait/interaction gaps | Medium | OPEN / partial |
| M-7 stale STATUS | Medium | **FIXED** this update |

## Ownership

PASS: Tower/App Server/MCP do not depend on Shell. Single runtime authority remains the design invariant; production injection still pending.
