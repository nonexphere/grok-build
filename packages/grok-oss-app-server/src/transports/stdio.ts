import { spawn, type ChildProcessWithoutNullStreams } from "node:child_process";
import { createInterface } from "node:readline";
import type { JsonRpcTransport } from "../client.js";

export class StdioTransport implements JsonRpcTransport {
  private readonly child: ChildProcessWithoutNullStreams;
  constructor(command = "grok-oss", args = ["app-server", "--stdio"]) {
    this.child = spawn(command, args, { stdio: ["pipe", "pipe", "pipe"] });
    this.child.stderr.pipe(process.stderr);
  }
  async send(message: string): Promise<void> { if (!this.child.stdin.write(`${message}\n`)) await new Promise<void>(resolve => this.child.stdin.once("drain", resolve)); }
  async *messages(): AsyncIterable<string> { for await (const line of createInterface({ input: this.child.stdout })) yield line; }
  async close(): Promise<void> { this.child.stdin.end(); await new Promise<void>((resolve, reject) => this.child.once("exit", code => code === 0 ? resolve() : reject(new Error(`grok-oss exited ${code}`)))); }
}
