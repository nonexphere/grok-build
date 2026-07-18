import assert from "node:assert/strict";
import test from "node:test";
import { AppServerClient, AppServerError, type JsonRpcTransport, WebSocketTransport } from "../src/index.js";

class FakeTransport implements JsonRpcTransport {
  sent: Record<string, unknown>[] = [];
  private values: string[] = [];
  private wake: (() => void) | undefined;
  private ended = false;
  async send(raw: string): Promise<void> { this.sent.push(JSON.parse(raw)); }
  respond(value: unknown): void { this.values.push(JSON.stringify(value)); this.wake?.(); this.wake = undefined; }
  async *messages(): AsyncIterable<string> { while (!this.ended) { if (this.values.length) yield this.values.shift()!; else await new Promise<void>(resolve => { this.wake = resolve; }); } }
  async close(): Promise<void> { this.ended = true; this.wake?.(); }
}

test("initialize sends initialized notification only after success", async () => {
  const transport = new FakeTransport();
  const client = new AppServerClient(transport);
  const pending = client.initialize({ protocolVersion: "2026-07-18.experimental-v2", clientInfo: { name: "test", version: "1" }, capabilities: { interactions: false, reconnect: true, experimental: [] } });
  await new Promise(resolve => setImmediate(resolve));
  transport.respond({ jsonrpc: "2.0", id: 1, result: { protocolVersion: "2026-07-18.experimental-v2", serverInfo: { name: "grok-oss", version: "test" }, serverInstanceId: "default", capabilities: { sessions: { list: true, read: true, start: true, resume: true, fork: true, archive: true, subscribe: true }, turns: { start: true, steer: true, interrupt: true }, items: { lifecycle: true, deltas: true }, interactions: { approvals: true, questions: true, mcpElicitation: true }, experimental: [] }, limits: { maxMessageBytes: 1048576, maxPageSize: 100, replayWindowEvents: 10000, outboundQueueEvents: 1024, initializeTimeoutMs: 10000 } } });
  await pending;
  assert.equal(transport.sent[0]?.method, "initialize");
  assert.equal(transport.sent[1]?.method, "initialized");
  await client.close();
});

test("domain errors retain code and retryability", async () => {
  const transport = new FakeTransport();
  const client = new AppServerClient(transport);
  const pending = client.request("session/read", { sessionId: "missing" });
  transport.respond({ jsonrpc: "2.0", id: 1, error: { code: -32010, message: "Not found", data: { code: "session_not_found", retryable: false } } });
  await assert.rejects(pending, (error: unknown) => error instanceof AppServerError && error.domainCode === "session_not_found" && !error.retryable);
  await client.close();
});

test("WebSocket transport forbids credentials or query parameters in URL", () => {
  assert.throws(() => new WebSocketTransport("ws://user@example.test/?token=secret", "bearer"), /forbidden/i);
});

test("subscriptions for the same session route independently by subscriptionId", async () => {
  const transport = new FakeTransport();
  const client = new AppServerClient(transport);
  const first = client.subscribe({ sessionId: "session_1", historyEpoch: "epoch_1", afterEventSeq: "0" })[Symbol.asyncIterator]();
  const second = client.subscribe({ sessionId: "session_1", historyEpoch: "epoch_1", afterEventSeq: "0" })[Symbol.asyncIterator]();
  const firstNext = first.next();
  const secondNext = second.next();
  await new Promise(resolve => setImmediate(resolve));
  transport.respond({ jsonrpc: "2.0", id: 1, result: { subscriptionId: "sub_1", historyEpoch: "epoch_1", replayedThroughEventSeq: "0", liveFromEventSeq: "1" } });
  transport.respond({ jsonrpc: "2.0", id: 2, result: { subscriptionId: "sub_2", historyEpoch: "epoch_1", replayedThroughEventSeq: "0", liveFromEventSeq: "1" } });
  await new Promise(resolve => setImmediate(resolve));
  const item = { itemId: "item_1", sessionId: "session_1", turnId: "turn_1", type: "agent_message", status: "completed", revision: "1", eventSeq: "1", createdAtMs: 1, text: "one" };
  transport.respond({ jsonrpc: "2.0", method: "item/completed", params: { subscriptionId: "sub_2", sessionId: "session_1", historyEpoch: "epoch_1", eventSeq: "1", item } });
  assert.deepEqual((await secondNext).value, item);
  const firstRace = await Promise.race([firstNext.then(() => "unexpected"), new Promise(resolve => setTimeout(() => resolve("pending"), 20))]);
  assert.equal(firstRace, "pending");
  transport.respond({ jsonrpc: "2.0", method: "item/completed", params: { subscriptionId: "sub_1", sessionId: "session_1", historyEpoch: "epoch_1", eventSeq: "1", item } });
  assert.deepEqual((await firstNext).value, item);
  await first.return?.();
  await second.return?.();
  await client.close();
});
