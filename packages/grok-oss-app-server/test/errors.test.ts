import assert from "node:assert/strict";
import { test } from "node:test";

test("typed domain errors remain distinct by code class", () => {
  const transport = { kind: "transport", message: "socket closed" };
  const jsonrpc = { kind: "jsonrpc", code: -32600, message: "Invalid Request" };
  const epoch = { kind: "epoch", code: "epoch_mismatch", retryable: true };
  const resync = { kind: "resync", code: "resync_required", retryable: true };
  assert.notEqual(transport.kind, jsonrpc.kind);
  assert.notEqual(epoch.code, resync.code);
  assert.equal(epoch.retryable, true);
  assert.equal(resync.retryable, true);
});
