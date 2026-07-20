# Tasks — generated SDK black-box GA

- [ ] TS102-01 [F-07] Generate all protocol declarations, method params/results, errors, capabilities and event unions from Rust/schema; accept types.ts is generated and carries no handwritten mirror claim.
- [ ] TS102-02 [F-07] Add clean-room delete/regenerate/check gate; run npm run generate && git diff --exit-code on generated paths in CI.
- [ ] TS102-03 [F-10] Make client feature behavior depend on initialize capabilities and reject unavailable methods before unsafe assumptions.
- [ ] TS102-04 [WS] Run Node WebSocket client against real grok-oss tower with bearer; exercise initialize/start/send/subscribe/history/interrupt/archive.
- [ ] TS102-05 [STDIO] Run Node stdio client against real product subprocess; assert stdout framing, EOF drain and stderr diagnostics.
- [ ] TS102-06 [RECONNECT] Test AbortSignal, transport close, pending request rejection, reconnect, epoch mismatch, resync and duplicate suppression.
- [ ] TS102-07 [ERRORS] Differentially compare typed SDK errors with direct WS/App Server fixtures including retryability and operationId.
- [ ] TS102-08 [SECURITY] Assert tokens never enter URL/log/storage; browser WS remains explicit unsupported until safe handshake.
- [ ] TS102-09 [PACKAGE] Verify exports, ESM/Node versions, examples, package contents and no accidental private/generated source omission.
- [ ] TS102-10 [MCP] Add a separate independent MCP example/client smoke consuming resolvable Tower tool schemas without redefining App Server protocol.
- [ ] TS102-11 [HUMAN] Approve package name/publication only after protocol freeze; local/private completion is otherwise valid.

