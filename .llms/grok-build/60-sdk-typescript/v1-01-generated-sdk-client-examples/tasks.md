# Tasks — generated TypeScript SDK and examples

- [ ] `TS101-01` [D-TS.1,D-TS.5] Freeze publish name, then implement generator/drift pipeline in `packages/grok-oss-app-server/scripts/`; run `npm --prefix packages/grok-oss-app-server run check:drift`; accept generated/interim types have zero critical-shape drift.
- [ ] `TS101-02` [D-TS.2,D-TS.3] Complete request client/reconnect iterator in `packages/grok-oss-app-server/src/client.ts`; run `npm --prefix packages/grok-oss-app-server run typecheck` plus client tests; accept ordered replay/live/reconnect with epoch validation.
- [ ] `TS101-03` [D-TS.4] Maintain Node stdio/WS examples under `packages/grok-oss-app-server/examples/`; run `npm --prefix packages/grok-oss-app-server run typecheck`; accept initialize→subscribe→turn→Items→close scripts compile.
- [ ] `TS101-04` [D-TS.6] Add runtime capability tests under `packages/grok-oss-app-server/test/`; run `npm --prefix packages/grok-oss-app-server test`; accept stdio/WS on Node, explicit browser bearer rejection and no token in URL.
- [ ] `TS101-05` [D-TS.7] Add typed error tests under `packages/grok-oss-app-server/test/`; run `npm --prefix packages/grok-oss-app-server test`; accept transport, JSON-RPC, epoch and resync errors remain distinct.
- [ ] [D-TS.1] `(HUMAN, product-decision, blocking: publish only)` approve name/publication; package stays private meanwhile.
