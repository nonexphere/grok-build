/**
 * Proposed Grok App Server v1 core declarations.
 * Production bindings should be generated from xai-grok-app-server-protocol.
 */
export type RequestId = string | number;
export type ThreadId = string;
export type TurnId = string;
export type ItemId = string;
export type InteractionId = string;
export type ApiBackend = "chat_completions" | "responses" | "messages";

export interface JsonRpcRequest<P = unknown> {
  jsonrpc: "2.0";
  id: RequestId;
  method: string;
  params: P;
}
export interface JsonRpcNotification<P = unknown> {
  jsonrpc: "2.0";
  method: string;
  params: P;
}
export interface JsonRpcSuccess<R = unknown> {
  jsonrpc: "2.0";
  id: RequestId;
  result: R;
}
export interface JsonRpcFailure {
  jsonrpc: "2.0";
  id: RequestId | null;
  error: { code: number; message: string; data?: unknown };
}

export interface ModelSelection {
  id: string;
  providerId: string | null;
  apiBackend: ApiBackend | null;
  reasoningEffort: string | null;
}
export type InputItem =
  | { type: "text"; text: string }
  | { type: "image"; url: string; mimeType: string | null; name: string | null }
  | { type: "localImage"; path: string; name: string | null }
  | { type: "mention"; name: string; path: string }
  | { type: "skill"; name: string; path: string | null };

export type ThreadStatus =
  | "starting" | "ready" | "running" | "waitingForInput"
  | "dormant" | "completed" | "failed" | "archived";
export type TurnStatus =
  | "queued" | "inProgress" | "waitingForApproval" | "waitingForInput"
  | "completed" | "failed" | "interrupted" | "declined";
export type ItemStatus =
  | "pending" | "inProgress" | "waitingForApproval" | "waitingForInput"
  | "completed" | "failed" | "declined" | "cancelled" | "backgrounded";

export interface ItemBase {
  id: ItemId;
  threadId: ThreadId;
  turnId: TurnId | null;
  type: string;
  status: ItemStatus;
  revision: number;
  createdAtMs: number;
  completedAtMs: number | null;
  metadata: Record<string, unknown>;
}
export interface UserMessageItem extends ItemBase {
  type: "userMessage";
  content: InputItem[];
  inputKind: "initial" | "steer" | "synthetic";
  clientUserMessageId: string | null;
}
export interface AgentMessageItem extends ItemBase {
  type: "agentMessage";
  content: string;
  format: "markdown" | "plain" | "json";
  modelId: string | null;
}
export interface CommandExecutionItem extends ItemBase {
  type: "commandExecution";
  command: string;
  argv: string[];
  cwd: string;
  aggregatedOutput: string;
  exitCode: number | null;
  durationMs: number | null;
  source: "agent" | "userShell" | "hook" | "subagent";
  backgroundTaskId: string | null;
}
export interface FileChange {
  path: string;
  kind: "add" | "modify" | "delete" | "rename";
  oldPath: string | null;
  diff: string | null;
  additions: number | null;
  deletions: number | null;
}
export interface FileChangeItem extends ItemBase {
  type: "fileChange";
  changes: FileChange[];
  summary: string | null;
  applied: boolean | null;
}
export interface PlanStep {
  id: string;
  title: string;
  description: string | null;
  status: "pending" | "inProgress" | "completed" | "blocked" | "cancelled";
  priority: "low" | "medium" | "high" | null;
}
export interface PlanItem extends ItemBase {
  type: "plan";
  content: string;
  steps: PlanStep[];
  approvalState: "notRequired" | "pending" | "accepted" | "declined" | null;
}
export interface SubagentItem extends ItemBase {
  type: "subagent";
  subagentId: string;
  childThreadId: ThreadId | null;
  subagentType: string;
  persona: string | null;
  description: string;
  worktreePath: string | null;
  modelId: string | null;
  runInBackground: boolean;
  progress: string | null;
  result: string | null;
}
export type ThreadItem =
  | UserMessageItem | AgentMessageItem | CommandExecutionItem
  | FileChangeItem | PlanItem | SubagentItem
  | (ItemBase & Record<string, unknown>);

export interface Turn {
  id: TurnId;
  threadId: ThreadId;
  ordinal: number;
  kind: string;
  status: TurnStatus;
  input: InputItem[];
  startedAtMs: number;
  completedAtMs: number | null;
  model: ModelSelection | null;
  promptOrigin: string | null;
  items: ThreadItem[];
  itemsView: "notLoaded" | "summary" | "full";
  usage: Record<string, unknown> | null;
  error: Record<string, unknown> | null;
  metadata: Record<string, unknown>;
}
export interface Thread {
  id: ThreadId;
  status: ThreadStatus;
  title: string | null;
  cwd: string;
  displayCwd: string | null;
  createdAtMs: number;
  updatedAtMs: number;
  model: ModelSelection | null;
  parentThreadId: ThreadId | null;
  activeTurnId: TurnId | null;
  latestTurnId: TurnId | null;
  metadata: Record<string, unknown>;
}

export interface ItemLifecycleParams {
  threadId: ThreadId;
  turnId: TurnId | null;
  eventSeq: number;
  timestampMs: number;
  item: ThreadItem;
}
export interface ItemDeltaParams {
  threadId: ThreadId;
  turnId: TurnId;
  itemId: ItemId;
  eventSeq: number;
  revision: number;
  delta: string;
  stream?: "stdout" | "stderr" | "content" | "reasoning";
  sequence?: number;
}
export interface ServerInteraction {
  interactionId: InteractionId;
  threadId: ThreadId;
  turnId: TurnId | null;
  itemId: ItemId;
  createdAtMs: number;
  expiresAtMs: number | null;
  reason: string | null;
  availableDecisions: string[];
}
export type ApprovalDecision =
  | "accept" | "acceptForTurn" | "acceptForSession" | "acceptAlways"
  | "decline" | "cancel";
