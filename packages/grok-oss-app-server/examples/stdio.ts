import { AppServerClient, StdioTransport } from "../src/index.js";

const client = new AppServerClient(new StdioTransport());
await client.initialize({ protocolVersion: "2026-07-18.experimental-v1", clientInfo: { name: "stdio-example", version: "1" }, capabilities: { interactions: true, reconnect: true, experimental: [] } });
const { session } = await client.sessionStart({ workspaceRoot: process.cwd(), idempotencyKey: crypto.randomUUID() });
const events = client.subscribe({ sessionId: session.sessionId, historyEpoch: session.historyEpoch, afterEventSeq: 0 });
await client.turnStart(session.sessionId, [{ type: "text", text: "Say hello." }], crypto.randomUUID());
for await (const item of events) if (item.type === "agent_message" && item.status === "completed") break;
await client.close();
