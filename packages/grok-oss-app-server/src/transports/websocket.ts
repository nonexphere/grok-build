import WebSocket from "ws";
import type { JsonRpcTransport } from "../client.js";

export class WebSocketTransport implements JsonRpcTransport {
  private readonly socket: WebSocket;
  private readonly queue = new MessageQueue();
  private readonly opened: Promise<void>;

  constructor(url: string, token: string) {
    if (!token) throw new Error("GROK_OSS_TOWER_TOKEN is required");
    if (new URL(url).username || new URL(url).password || new URL(url).search) throw new Error("Credentials/query parameters are forbidden in the WebSocket URL");
    this.socket = new WebSocket(url, "grok-oss.app-server.experimental-v1", { headers: { Authorization: `Bearer ${token}` }, maxPayload: 1048576 });
    this.opened = new Promise((resolve, reject) => { this.socket.once("open", resolve); this.socket.once("error", reject); });
    this.socket.on("message", (data, binary) => { if (binary) this.queue.fail(new Error("Binary App Server frame")); else this.queue.push(data.toString()); });
    this.socket.once("close", () => this.queue.end());
    this.socket.once("error", error => this.queue.fail(error));
  }

  async send(message: string): Promise<void> { await this.opened; await new Promise<void>((resolve, reject) => this.socket.send(message, error => error ? reject(error) : resolve())); }
  messages(): AsyncIterable<string> { return this.queue; }
  async close(): Promise<void> { if (this.socket.readyState === WebSocket.CLOSED) return; await new Promise<void>(resolve => { this.socket.once("close", () => resolve()); this.socket.close(1000); }); }
}

class MessageQueue implements AsyncIterable<string> {
  private values: string[] = [];
  private waiters: Array<(value: IteratorResult<string>) => void> = [];
  private terminalError: unknown;
  private ended = false;
  push(value: string): void { const waiter = this.waiters.shift(); if (waiter) waiter({ value, done: false }); else this.values.push(value); }
  end(): void { this.ended = true; for (const waiter of this.waiters.splice(0)) waiter({ value: undefined, done: true }); }
  fail(error: unknown): void { this.terminalError = error; this.end(); }
  async *[Symbol.asyncIterator](): AsyncIterator<string> { while (true) { if (this.values.length) { yield this.values.shift()!; continue; } if (this.terminalError) throw this.terminalError; if (this.ended) return; const next = await new Promise<IteratorResult<string>>(resolve => this.waiters.push(resolve)); if (next.done) { if (this.terminalError) throw this.terminalError; return; } yield next.value; } }
}
