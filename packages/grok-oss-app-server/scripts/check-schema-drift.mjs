import { readFile } from "node:fs/promises";

const schema = JSON.parse(await readFile(new URL("../../../crates/codegen/xai-grok-app-server-protocol/schemas/app-server.schema.json", import.meta.url), "utf8"));
const types = await readFile(new URL("../src/types.ts", import.meta.url), "utf8");
const requiredMirrors = {
  initializeParams: ["protocolVersion", "clientInfo", "capabilities"],
  protocolLimits: ["maxMessageBytes", "maxPageSize", "replayWindowEvents", "outboundQueueEvents", "initializeTimeoutMs"],
  session: ["sessionId", "historyEpoch", "revision", "status", "workspaceRoot", "createdAtMs", "updatedAtMs"],
  turn: ["turnId", "sessionId", "ordinal", "kind", "status", "revision", "createdAtMs", "completedAtMs"],
};
for (const [definition, fields] of Object.entries(requiredMirrors)) {
  if (!schema.$defs[definition]) throw new Error(`schema definition missing: ${definition}`);
  for (const field of fields) if (!types.includes(field)) throw new Error(`TypeScript mirror missing ${definition}.${field}`);
}
for (const status of schema.$defs.session.properties.status.enum) if (!types.includes(`"${status}"`)) throw new Error(`TypeScript SessionStatus missing ${status}`);
for (const status of schema.$defs.turn.properties.status.enum) if (!types.includes(`"${status}"`)) throw new Error(`TypeScript TurnStatus missing ${status}`);
console.log("Rust/check-in schema ↔ interim TypeScript critical-shape drift check passed");
