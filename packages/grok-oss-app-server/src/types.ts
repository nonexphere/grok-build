/** Interim hand-authored mirror. Rust/schema generation replaces this file. */
export type JsonRpcId = string | number;
export interface ClientInfo { name: string; version: string }
export interface ClientCapabilities { interactions: boolean; reconnect: boolean; experimental: string[] }
export interface InitializeParams { protocolVersion: "2026-07-18.experimental-v1"; clientInfo: ClientInfo; capabilities: ClientCapabilities }
export interface ProtocolLimits { maxMessageBytes: 1048576; maxPageSize: 100; replayWindowEvents: 10000; outboundQueueEvents: 1024; initializeTimeoutMs: 10000 }
export interface ServerCapabilities {
  sessions: { list: boolean; read: boolean; start: boolean; resume: boolean; fork: boolean; archive: boolean; subscribe: boolean };
  turns: { start: boolean; steer: boolean; interrupt: boolean };
  items: { lifecycle: boolean; deltas: boolean };
  interactions: { approvals: boolean; questions: boolean; mcpElicitation: boolean };
  experimental: string[];
}
export interface InitializeResult { protocolVersion: string; serverInfo: ClientInfo; serverInstanceId: string; capabilities: ServerCapabilities; limits: ProtocolLimits }
export type SessionStatus = "starting" | "ready" | "running" | "waiting_for_input" | "dormant" | "completed" | "failed" | "archived";
export interface Session { sessionId: string; historyEpoch: string; revision: number; status: SessionStatus; workspaceRoot: string; title: string | null; activeTurnId: string | null; latestTurnId: string | null; providerBinding: string | null; createdAtMs: number; updatedAtMs: number }
export type TurnStatus = "queued" | "in_progress" | "waiting_for_approval" | "waiting_for_input" | "completed" | "failed" | "interrupted" | "declined";
export interface Turn { turnId: string; sessionId: string; ordinal: number; kind: "user" | "steer" | "resume" | "synthetic"; status: TurnStatus; revision: number; createdAtMs: number; completedAtMs: number | null }
export interface ItemBase { itemId: string; sessionId: string; turnId: string; status: string; revision: number; eventSeq: number; createdAtMs: number }
export type Item = ItemBase & (
  | { type: "user_message"; content: InputBlock[] }
  | { type: "agent_message"; text: string }
  | { type: "tool_call"; toolName: string; arguments: unknown }
  | { type: "tool_result"; toolName: string; output: unknown; isError: boolean }
  | { type: "command_execution"; command: string; argv: string[]; cwd: string }
  | { type: "file_change"; changes: Record<string, unknown>[] }
  | { type: "plan"; content: string; steps: Record<string, unknown>[] }
  | { type: "subagent"; subagentId: string; agentType: string; description: string }
  | { type: "mcp_tool_call"; server: string; toolName: string; arguments: unknown }
  | { type: "reasoning_summary"; summary: string }
  | { type: "hook"; hookName: string; phase: string; safeSummary: string }
  | { type: "background_task"; taskId: string; safeSummary: string }
  | { type: "compaction"; safeSummary: string }
  | { type: "provider_error"; providerId: string; code: string; safeMessage: string }
  | { type: "error"; code: string; message: string }
  | { type: "interaction_request"; interactionId: string; prompt: string; choices: string[] }
  | { type: "extension"; extensionType: string; payload: unknown }
);
export type InputBlock = { type: "text"; text: string } | { type: "mention" | "skill"; name: string; path: string | null };
export interface RpcErrorData { code: string; retryable: boolean; operationId?: string; [key: string]: unknown }
export interface JsonRpcError { code: number; message: string; data?: RpcErrorData }
export interface SessionStartParams { workspaceRoot: string; agentType?: string | null; providerBinding?: string | null; idempotencyKey: string }
export interface SessionListParams { pageSize?: number; cursor?: string | null; includeArchived?: boolean; workspaceRoot?: string | null }
export interface SessionListResult { sessions: Session[]; nextCursor: string | null }
export interface SessionReadResult { session: Session; turns: Turn[]; items: Item[] }
export interface SubscribeParams { sessionId: string; historyEpoch?: string | null; afterEventSeq: number }
export interface OperationResult { operationId: string; accepted: boolean }
export interface ProtocolEvent { method: string; params: { sessionId?: string; historyEpoch?: string; eventSeq?: number; [key: string]: unknown } }
