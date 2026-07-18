import { spawnSync } from "node:child_process";
import { readFile } from "node:fs/promises";
import ts from "typescript";

const rustCheck = spawnSync("cargo", ["run", "-q", "-p", "xai-grok-app-server-protocol", "--example", "generate-schema", "--", "--check"], {
  cwd: new URL("../../..", import.meta.url),
  encoding: "utf8",
});
if (rustCheck.status !== 0) throw new Error(rustCheck.stderr || "Rust-generated schema is stale");

const schema = JSON.parse(await readFile(new URL("../../../crates/codegen/xai-grok-app-server-protocol/schemas/app-server.schema.json", import.meta.url), "utf8"));
const sourceText = await readFile(new URL("../src/types.ts", import.meta.url), "utf8");
const source = ts.createSourceFile("types.ts", sourceText, ts.ScriptTarget.Latest, true, ts.ScriptKind.TS);
const declarations = new Map();
for (const node of source.statements) {
  if ((ts.isInterfaceDeclaration(node) || ts.isTypeAliasDeclaration(node)) && node.name) declarations.set(node.name.text, node);
}

function interfaceFields(name) {
  const node = declarations.get(name);
  if (!node || !ts.isInterfaceDeclaration(node)) throw new Error(`TypeScript interface missing: ${name}`);
  return new Map(node.members.filter(ts.isPropertySignature).map(member => [member.name.getText(source), member.type?.getText(source) ?? ""]));
}
function expectFields(name, expected) {
  const fields = interfaceFields(name);
  for (const [field, type] of Object.entries(expected)) {
    if (fields.get(field) !== type) throw new Error(`${name}.${field} must be ${type}; got ${fields.get(field) ?? "missing"}`);
  }
}
const wireCounter = declarations.get("WireCounter");
if (!wireCounter || !ts.isTypeAliasDeclaration(wireCounter) || wireCounter.type.getText(source) !== "string") throw new Error("WireCounter must be a string alias");

expectFields("ProviderBinding", { providerId: "string", credentialId: "string", modelId: "string", backend: "string", bindingRevision: "WireCounter" });
expectFields("Session", { revision: "WireCounter", providerBinding: "ProviderBinding | null" });
expectFields("Turn", { revision: "WireCounter", providerBinding: "ProviderBinding | null" });
expectFields("ItemBase", { revision: "WireCounter", eventSeq: "WireCounter" });
expectFields("SubscribeParams", { afterEventSeq: "WireCounter" });
expectFields("SubscribeResult", { replayedThroughEventSeq: "WireCounter", liveFromEventSeq: "WireCounter" });

for (const definition of ["wireCounter", "providerBinding", "session", "turn", "itemBase", "subscribeParams", "subscribeResult", "eventMeta"]) {
  if (!schema.$defs[definition]) throw new Error(`schema definition missing: ${definition}`);
}
if (schema.$defs.wireCounter.type !== "string") throw new Error("wireCounter schema must reject JSON numbers");
for (const field of ["providerId", "credentialId", "modelId", "backend", "bindingRevision"]) {
  if (!schema.$defs.providerBinding.required.includes(field)) throw new Error(`providerBinding.${field} must be required`);
}
if (schema.$defs.providerBinding.additionalProperties !== false) throw new Error("providerBinding must reject secret/unknown fields");
for (const status of schema.$defs.session.properties.status.enum) if (!sourceText.includes(`"${status}"`)) throw new Error(`SessionStatus missing ${status}`);
for (const status of schema.$defs.turn.properties.status.enum) if (!sourceText.includes(`"${status}"`)) throw new Error(`TurnStatus missing ${status}`);

console.log("Rust → generated schema snapshot → operational schema → TypeScript structural drift check passed");
