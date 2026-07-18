import { AppServerClient, WebSocketTransport } from "../src/index.js";

const client = new AppServerClient(new WebSocketTransport("ws://127.0.0.1:8787", process.env.GROK_OSS_TOWER_TOKEN ?? ""));
await client.initialize({ protocolVersion: "2026-07-18.experimental-v2", clientInfo: { name: "ws-example", version: "1" }, capabilities: { interactions: true, reconnect: true, experimental: [] } });
const { session } = await client.sessionStart({ workspaceRoot: process.cwd(), agentType: "orchestrator", idempotencyKey: crypto.randomUUID() });
const events = client.subscribe({ sessionId: session.sessionId, historyEpoch: session.historyEpoch, afterEventSeq: "0" });
await client.turnStart(session.sessionId, [{ type: "text", text: "Report repository status." }], crypto.randomUUID());
for await (const item of events) if (item.type === "agent_message" && item.status === "completed") break;
await client.close();
