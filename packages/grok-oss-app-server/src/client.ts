import type { InitializeParams, InitializeResult, Item, OperationResult, ProtocolEvent, Session, SessionListParams, SessionListResult, SessionReadResult, SessionStartParams, SubscribeParams, Turn } from "./types.js";

export interface JsonRpcTransport {
  send(message: string): Promise<void>;
  messages(): AsyncIterable<string>;
  close(): Promise<void>;
}

export class AppServerError extends Error {
  constructor(readonly rpcCode: number, message: string, readonly domainCode?: string, readonly retryable = false, readonly data?: unknown) { super(message); }
}

type Pending = { resolve(value: unknown): void; reject(reason: unknown): void };

export class AppServerClient {
  private nextId = 1;
  private pending = new Map<number, Pending>();
  private eventQueues = new Map<string, AsyncEventQueue<Item>>();
  private reader: Promise<void>;

  constructor(private readonly transport: JsonRpcTransport) { this.reader = this.readLoop(); }

  async initialize(params: InitializeParams): Promise<InitializeResult> {
    const result = await this.request<InitializeResult>("initialize", params);
    await this.notify("initialized", {});
    return result;
  }
  sessionStart(params: SessionStartParams): Promise<{ session: Session }> { return this.request("session/start", params); }
  sessionResume(sessionId: string, idempotencyKey: string): Promise<{ session: Session }> { return this.request("session/resume", { sessionId, idempotencyKey }); }
  sessionFork(sessionId: string, idempotencyKey: string, workspaceRoot: string | null = null): Promise<{ session: Session }> { return this.request("session/fork", { sessionId, idempotencyKey, workspaceRoot }); }
  sessionRead(sessionId: string, includeTurns = true, includeItems = true): Promise<SessionReadResult> { return this.request("session/read", { sessionId, includeTurns, includeItems }); }
  sessionList(params: SessionListParams = {}): Promise<SessionListResult> { return this.request("session/list", params); }
  sessionArchive(sessionId: string, idempotencyKey: string): Promise<OperationResult> { return this.request("session/archive", { sessionId, idempotencyKey }); }
  turnStart(sessionId: string, input: unknown[], idempotencyKey: string): Promise<{ turn: Turn }> { return this.request("turn/start", { sessionId, input, idempotencyKey }); }
  turnSteer(sessionId: string, turnId: string, input: unknown[], idempotencyKey: string): Promise<{ item: Item }> { return this.request("turn/steer", { sessionId, turnId, input, idempotencyKey }); }
  turnInterrupt(sessionId: string, turnId: string, idempotencyKey: string): Promise<OperationResult> { return this.request("turn/interrupt", { sessionId, turnId, idempotencyKey }); }

  async *subscribe(params: SubscribeParams): AsyncIterable<Item> {
    const queue = new AsyncEventQueue<Item>();
    this.eventQueues.set(params.sessionId, queue);
    try {
      await this.request("session/subscribe", params);
      for await (const item of queue) yield item;
    } finally {
      this.eventQueues.delete(params.sessionId);
    }
  }

  async request<T>(method: string, params: object): Promise<T> {
    const id = this.nextId++;
    const promise = new Promise<T>((resolve, reject) => this.pending.set(id, { resolve: value => resolve(value as T), reject }));
    await this.transport.send(JSON.stringify({ jsonrpc: "2.0", id, method, params }));
    return promise;
  }

  async notify(method: string, params: object): Promise<void> {
    await this.transport.send(JSON.stringify({ jsonrpc: "2.0", method, params }));
  }

  async close(): Promise<void> { await this.transport.close(); await this.reader; }

  private async readLoop(): Promise<void> {
    for await (const raw of this.transport.messages()) {
      const message = JSON.parse(raw) as Record<string, unknown>;
      if (typeof message.id === "number") {
        const pending = this.pending.get(message.id);
        if (!pending) continue;
        this.pending.delete(message.id);
        if (message.error) {
          const error = message.error as { code: number; message: string; data?: { code?: string; retryable?: boolean } };
          pending.reject(new AppServerError(error.code, error.message, error.data?.code, error.data?.retryable, error.data));
        } else pending.resolve(message.result);
      } else if (typeof message.method === "string") this.routeEvent(message as unknown as ProtocolEvent);
    }
    for (const pending of this.pending.values()) pending.reject(new Error("App Server transport closed"));
    this.pending.clear();
    for (const queue of this.eventQueues.values()) queue.end();
  }

  private routeEvent(event: ProtocolEvent): void {
    const sessionId = event.params.sessionId;
    const item = event.params.item;
    if (sessionId && item && typeof item === "object") this.eventQueues.get(sessionId)?.push(item as Item);
  }
}

class AsyncEventQueue<T> implements AsyncIterable<T> {
  private values: T[] = [];
  private waiters: Array<(value: IteratorResult<T>) => void> = [];
  private ended = false;
  push(value: T): void { const waiter = this.waiters.shift(); if (waiter) waiter({ value, done: false }); else this.values.push(value); }
  end(): void { this.ended = true; for (const waiter of this.waiters.splice(0)) waiter({ value: undefined, done: true }); }
  async *[Symbol.asyncIterator](): AsyncIterator<T> {
    while (true) {
      if (this.values.length) { yield this.values.shift()!; continue; }
      if (this.ended) return;
      const next = await new Promise<IteratorResult<T>>(resolve => this.waiters.push(resolve));
      if (next.done) return;
      yield next.value;
    }
  }
}
