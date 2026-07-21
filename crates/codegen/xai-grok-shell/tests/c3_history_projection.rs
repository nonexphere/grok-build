//! C3-F canonical history / RuntimeEvent projection tests (R2 / R11).
//!
//! These exercise the shared `project_updates` projector (via the real
//! [`ShellSessionActorRuntime`] facade) against real `updates.jsonl` fixtures
//! built with the JSONL storage adapter. They prove:
//!
//! - `read_session` projects `Turn`/`Item` from `updates.jsonl` (R2) — turns
//!   bounded on `UserMessageChunk`, items include user/agent/thought/tool/
//!   plan bodies.
//! - `replay` projects a richer `RuntimeEvent` lifecycle (R11) — tool call
//!   `ItemStarted`→`ItemCompleted` correlated via `tool_call_id`, reasoning
//!   chunks, plans — where Shell writes the data.
//! - Honest PARTIAL: `TurnChanged` is never emitted (Shell writes no turn
//!   lifecycle); `InteractionRequested` is never projected (in-memory only);
//!   xAI extension updates are skipped.
//!
//! RED-then-GREEN evidence is captured under
//! `.llms/execution/app-server-mcp-tower-corrective/tests/c3/`.

use agent_client_protocol as acp;
use tempfile::TempDir;
use xai_grok_app_server_protocol::{
    ItemBody, ItemStatus, SessionReadParams, SessionStartParams, SubscribeParams, TurnStatus,
    WireCounter,
};
use xai_grok_shell::app_server_runtime::ShellSessionActorRuntime;
use xai_grok_shell::session::info::Info;
use xai_grok_shell::session::storage::{JsonlStorageAdapter, SessionUpdate, StorageAdapter};
use xai_grok_tower::{GrokRuntimeFacade, RuntimeEvent};

fn real_port(temp: &TempDir) -> ShellSessionActorRuntime {
    ShellSessionActorRuntime::new(temp.path().to_path_buf())
}

/// Start a real session and return its id + the resolved cwd.
async fn start(_temp: &TempDir, port: &ShellSessionActorRuntime, cwd: &str) -> String {
    port.start_session(SessionStartParams {
        workspace_root: cwd.into(),
        agent_type: None,
        provider_binding: None,
        idempotency_key: format!("c3-{cwd}"),
    })
    .await
    .unwrap()
    .session_id
}

/// Append an ACP update to the session's `updates.jsonl` via the storage
/// adapter (the real path the actor uses).
async fn append_acp(temp: &TempDir, session_id: &str, cwd: &str, update: acp::SessionUpdate) {
    let storage = JsonlStorageAdapter::with_root(temp.path().to_path_buf());
    let info = Info {
        id: acp::SessionId::new(session_id.to_string()),
        cwd: cwd.to_string(),
    };
    let notification =
        acp::SessionNotification::new(acp::SessionId::new(session_id.to_string()), update);
    storage
        .append_update(&info, &SessionUpdate::Acp(Box::new(notification)))
        .await
        .unwrap();
}

fn user_chunk(text: &str) -> acp::SessionUpdate {
    acp::SessionUpdate::UserMessageChunk(acp::ContentChunk::new(acp::ContentBlock::Text(
        acp::TextContent::new(text.to_string()),
    )))
}

fn agent_chunk(text: &str) -> acp::SessionUpdate {
    acp::SessionUpdate::AgentMessageChunk(acp::ContentChunk::new(acp::ContentBlock::Text(
        acp::TextContent::new(text.to_string()),
    )))
}

fn thought_chunk(text: &str) -> acp::SessionUpdate {
    acp::SessionUpdate::AgentThoughtChunk(acp::ContentChunk::new(acp::ContentBlock::Text(
        acp::TextContent::new(text.to_string()),
    )))
}

fn tool_call(tool_call_id: &str, title: &str, status: acp::ToolCallStatus) -> acp::SessionUpdate {
    let mut tc = acp::ToolCall::new(acp::ToolCallId::new(tool_call_id), title);
    tc = tc.status(status);
    acp::SessionUpdate::ToolCall(tc)
}

fn tool_call_update(
    tool_call_id: &str,
    status: acp::ToolCallStatus,
    title: Option<String>,
) -> acp::SessionUpdate {
    let mut fields = acp::ToolCallUpdateFields::new();
    fields.status = Some(status);
    fields.title = title;
    acp::SessionUpdate::ToolCallUpdate(acp::ToolCallUpdate::new(
        acp::ToolCallId::new(tool_call_id),
        fields,
    ))
}

fn plan(entries: Vec<acp::PlanEntry>) -> acp::SessionUpdate {
    acp::SessionUpdate::Plan(acp::Plan::new(entries))
}

// ---------------------------------------------------------------------------
// R2 — read_session Turn/Item projection
// ---------------------------------------------------------------------------

#[tokio::test]
async fn c3_read_session_projects_turns_and_items_from_updates_jsonl() {
    let temp = TempDir::new().unwrap();
    let port = real_port(&temp);
    let cwd = "/work/read-turns";
    let sid = start(&temp, &port, cwd).await;
    // Seed: user message → agent chunk → thought chunk.
    append_acp(&temp, &sid, cwd, user_chunk("hello")).await;
    append_acp(&temp, &sid, cwd, agent_chunk("hi back")).await;
    append_acp(&temp, &sid, cwd, thought_chunk("thinking")).await;

    let result = port
        .read_session(SessionReadParams {
            session_id: sid.clone(),
            include_turns: true,
            include_items: true,
        })
        .await
        .unwrap();

    // R2 REAL: turns projected from user-message boundaries.
    assert_eq!(result.turns.len(), 1, "one turn per user message");
    assert_eq!(result.turns[0].ordinal, 1);
    assert_eq!(result.turns[0].status, TurnStatus::Completed);
    // R2 REAL: items projected (user + agent + thought).
    assert_eq!(result.items.len(), 3, "user + agent + thought items");
}

#[tokio::test]
async fn c3_read_session_turn_boundaries_on_each_user_message() {
    let temp = TempDir::new().unwrap();
    let port = real_port(&temp);
    let cwd = "/work/read-multi-turn";
    let sid = start(&temp, &port, cwd).await;
    append_acp(&temp, &sid, cwd, user_chunk("turn 1")).await;
    append_acp(&temp, &sid, cwd, agent_chunk("reply 1")).await;
    append_acp(&temp, &sid, cwd, user_chunk("turn 2")).await;
    append_acp(&temp, &sid, cwd, agent_chunk("reply 2")).await;

    let result = port
        .read_session(SessionReadParams {
            session_id: sid,
            include_turns: true,
            include_items: false,
        })
        .await
        .unwrap();

    assert_eq!(result.turns.len(), 2, "two turns bounded on user messages");
    assert_eq!(result.turns[0].ordinal, 1);
    assert_eq!(result.turns[1].ordinal, 2);
    // turn_ids are distinct and synthesized from the user-message line number.
    assert_ne!(result.turns[0].turn_id, result.turns[1].turn_id);
}

#[tokio::test]
async fn c3_read_session_items_include_tool_call_and_reasoning_bodies() {
    let temp = TempDir::new().unwrap();
    let port = real_port(&temp);
    let cwd = "/work/read-bodies";
    let sid = start(&temp, &port, cwd).await;
    append_acp(&temp, &sid, cwd, user_chunk("run a tool")).await;
    append_acp(&temp, &sid, cwd, thought_chunk("planning")).await;
    append_acp(
        &temp,
        &sid,
        cwd,
        tool_call("tc-1", "Read file", acp::ToolCallStatus::InProgress),
    )
    .await;

    let result = port
        .read_session(SessionReadParams {
            session_id: sid,
            include_turns: false,
            include_items: true,
        })
        .await
        .unwrap();

    // user + thought + tool call = 3 items
    assert_eq!(result.items.len(), 3);
    let has_reasoning = result
        .items
        .iter()
        .any(|i| matches!(i.body, ItemBody::ReasoningSummary { .. }));
    assert!(
        has_reasoning,
        "reasoning chunk projected to ReasoningSummary"
    );
    let has_tool = result
        .items
        .iter()
        .any(|i| matches!(i.body, ItemBody::ToolCall { .. }));
    assert!(has_tool, "tool call projected to ToolCall body");
    let tool_item = result
        .items
        .iter()
        .find(|i| matches!(i.body, ItemBody::ToolCall { .. }))
        .unwrap();
    assert_eq!(tool_item.status, ItemStatus::InProgress);
    // item_id correlates via tool_call_id (tc_{tool_call_id}).
    assert_eq!(tool_item.item_id, "tc_tc-1");
}

#[tokio::test]
async fn c3_read_session_items_include_plan_body() {
    let temp = TempDir::new().unwrap();
    let port = real_port(&temp);
    let cwd = "/work/read-plan";
    let sid = start(&temp, &port, cwd).await;
    append_acp(&temp, &sid, cwd, user_chunk("plan it")).await;
    let entries = vec![acp::PlanEntry::new(
        "step one",
        acp::PlanEntryPriority::High,
        acp::PlanEntryStatus::Pending,
    )];
    append_acp(&temp, &sid, cwd, plan(entries)).await;

    let result = port
        .read_session(SessionReadParams {
            session_id: sid,
            include_turns: false,
            include_items: true,
        })
        .await
        .unwrap();

    let has_plan = result
        .items
        .iter()
        .any(|i| matches!(i.body, ItemBody::Plan { .. }));
    assert!(has_plan, "plan projected to Plan body");
}

#[tokio::test]
async fn c3_read_session_empty_updates_returns_empty_turns_and_items() {
    let temp = TempDir::new().unwrap();
    let port = real_port(&temp);
    let sid = start(&temp, &port, "/work/read-empty").await;

    let result = port
        .read_session(SessionReadParams {
            session_id: sid,
            include_turns: true,
            include_items: true,
        })
        .await
        .unwrap();

    assert!(result.turns.is_empty(), "no updates → no turns");
    assert!(result.items.is_empty(), "no updates → no items");
}

#[tokio::test]
async fn c3_read_session_include_flags_respected() {
    let temp = TempDir::new().unwrap();
    let port = real_port(&temp);
    let cwd = "/work/read-flags";
    let sid = start(&temp, &port, cwd).await;
    append_acp(&temp, &sid, cwd, user_chunk("hi")).await;
    append_acp(&temp, &sid, cwd, agent_chunk("yo")).await;

    let turns_only = port
        .read_session(SessionReadParams {
            session_id: sid.clone(),
            include_turns: true,
            include_items: false,
        })
        .await
        .unwrap();
    assert!(!turns_only.turns.is_empty());
    assert!(
        turns_only.items.is_empty(),
        "include_items=false → no items"
    );

    let items_only = port
        .read_session(SessionReadParams {
            session_id: sid,
            include_turns: false,
            include_items: true,
        })
        .await
        .unwrap();
    assert!(
        items_only.turns.is_empty(),
        "include_turns=false → no turns"
    );
    assert!(!items_only.items.is_empty());
}

// ---------------------------------------------------------------------------
// R11 — replay RuntimeEvent projection
// ---------------------------------------------------------------------------

#[tokio::test]
async fn c3_replay_projects_tool_call_lifecycle_correlated_via_tool_call_id() {
    let temp = TempDir::new().unwrap();
    let port = real_port(&temp);
    let cwd = "/work/replay-tc";
    let sid = start(&temp, &port, cwd).await;
    append_acp(&temp, &sid, cwd, user_chunk("do it")).await;
    append_acp(
        &temp,
        &sid,
        cwd,
        tool_call("tc-42", "Read", acp::ToolCallStatus::InProgress),
    )
    .await;
    append_acp(
        &temp,
        &sid,
        cwd,
        tool_call_update("tc-42", acp::ToolCallStatus::Completed, None),
    )
    .await;

    let page = port
        .replay(SubscribeParams {
            session_id: sid,
            after_event_seq: WireCounter::new(0),
            history_epoch: None,
        })
        .await
        .unwrap();

    // event 0 = SessionChanged snapshot; event 1 = user ItemCompleted;
    // event 2 = tool ItemStarted; event 3 = tool ItemCompleted.
    let started = page
        .events
        .iter()
        .find(|e| matches!(e, RuntimeEvent::ItemStarted(i) if i.item_id == "tc_tc-42"))
        .expect("ItemStarted for tc-42");
    let completed = page
        .events
        .iter()
        .find(|e| matches!(e, RuntimeEvent::ItemCompleted(i) if i.item_id == "tc_tc-42"))
        .expect("ItemCompleted for tc-42");
    // Same item_id → correlated lifecycle (not two unrelated items).
    let start_id = match started {
        RuntimeEvent::ItemStarted(i) => i.item_id.clone(),
        _ => unreachable!(),
    };
    let done_id = match completed {
        RuntimeEvent::ItemCompleted(i) => i.item_id.clone(),
        _ => unreachable!(),
    };
    assert_eq!(start_id, done_id, "ItemStarted/ItemCompleted share item_id");
    let done_item = match completed {
        RuntimeEvent::ItemCompleted(i) => i,
        _ => unreachable!(),
    };
    assert_eq!(done_item.status, ItemStatus::Completed);
}

#[tokio::test]
async fn c3_replay_projects_agent_thought_chunk_as_item_completed() {
    let temp = TempDir::new().unwrap();
    let port = real_port(&temp);
    let cwd = "/work/replay-thought";
    let sid = start(&temp, &port, cwd).await;
    append_acp(&temp, &sid, cwd, user_chunk("think")).await;
    append_acp(&temp, &sid, cwd, thought_chunk("reasoning here")).await;

    let page = port
        .replay(SubscribeParams {
            session_id: sid,
            after_event_seq: WireCounter::new(0),
            history_epoch: None,
        })
        .await
        .unwrap();

    let has_thought = page.events.iter().any(|e| {
        matches!(
            e,
            RuntimeEvent::ItemCompleted(i)
                if matches!(i.body, ItemBody::ReasoningSummary { .. })
        )
    });
    assert!(
        has_thought,
        "AgentThoughtChunk → ItemCompleted(ReasoningSummary)"
    );
}

#[tokio::test]
async fn c3_replay_projects_agent_message_chunk_as_item_delta() {
    let temp = TempDir::new().unwrap();
    let port = real_port(&temp);
    let cwd = "/work/replay-delta";
    let sid = start(&temp, &port, cwd).await;
    append_acp(&temp, &sid, cwd, user_chunk("q")).await;
    append_acp(&temp, &sid, cwd, agent_chunk("streaming-delta")).await;

    let page = port
        .replay(SubscribeParams {
            session_id: sid,
            after_event_seq: WireCounter::new(0),
            history_epoch: None,
        })
        .await
        .unwrap();

    let has_delta = page
        .events
        .iter()
        .any(|e| matches!(e, RuntimeEvent::ItemDelta { delta, .. } if delta == "streaming-delta"));
    assert!(has_delta, "AgentMessageChunk → ItemDelta");
}

#[tokio::test]
async fn c3_replay_projects_plan_as_item_completed() {
    let temp = TempDir::new().unwrap();
    let port = real_port(&temp);
    let cwd = "/work/replay-plan";
    let sid = start(&temp, &port, cwd).await;
    append_acp(&temp, &sid, cwd, user_chunk("plan")).await;
    let entries = vec![acp::PlanEntry::new(
        "only step",
        acp::PlanEntryPriority::Medium,
        acp::PlanEntryStatus::InProgress,
    )];
    append_acp(&temp, &sid, cwd, plan(entries)).await;

    let page = port
        .replay(SubscribeParams {
            session_id: sid,
            after_event_seq: WireCounter::new(0),
            history_epoch: None,
        })
        .await
        .unwrap();

    let has_plan = page.events.iter().any(|e| {
        matches!(
            e,
            RuntimeEvent::ItemCompleted(i) if matches!(i.body, ItemBody::Plan { .. })
        )
    });
    assert!(has_plan, "Plan → ItemCompleted(Plan)");
}

// ---------------------------------------------------------------------------
// Honest PARTIAL — events Shell never writes
// ---------------------------------------------------------------------------

#[tokio::test]
async fn c3_replay_honest_partial_no_turn_changed_emitted() {
    // Shell writes no turn lifecycle events. The projector must NOT
    // synthesize TurnChanged — that would be inventing data Shell never
    // wrote. Honest PARTIAL: turn boundaries are available via
    // read_session.turns (inferred), but replay emits no TurnChanged.
    let temp = TempDir::new().unwrap();
    let port = real_port(&temp);
    let cwd = "/work/replay-no-turn";
    let sid = start(&temp, &port, cwd).await;
    append_acp(&temp, &sid, cwd, user_chunk("a")).await;
    append_acp(&temp, &sid, cwd, agent_chunk("b")).await;
    append_acp(&temp, &sid, cwd, user_chunk("c")).await;

    let page = port
        .replay(SubscribeParams {
            session_id: sid,
            after_event_seq: WireCounter::new(0),
            history_epoch: None,
        })
        .await
        .unwrap();

    let has_turn_changed = page
        .events
        .iter()
        .any(|e| matches!(e, RuntimeEvent::TurnChanged(_)));
    assert!(
        !has_turn_changed,
        "TurnChanged must NOT be emitted — Shell writes no turn lifecycle"
    );
}

#[tokio::test]
async fn c3_replay_honest_partial_no_interaction_requested_emitted() {
    // Shell interaction requests are in-memory only (pending_interaction.rs);
    // they are never persisted to updates.jsonl. The projector must NOT
    // synthesize InteractionRequested.
    let temp = TempDir::new().unwrap();
    let port = real_port(&temp);
    let cwd = "/work/replay-no-interaction";
    let sid = start(&temp, &port, cwd).await;
    append_acp(&temp, &sid, cwd, user_chunk("need approval")).await;
    append_acp(&temp, &sid, cwd, agent_chunk("working")).await;

    let page = port
        .replay(SubscribeParams {
            session_id: sid,
            after_event_seq: WireCounter::new(0),
            history_epoch: None,
        })
        .await
        .unwrap();

    let has_interaction = page
        .events
        .iter()
        .any(|e| matches!(e, RuntimeEvent::InteractionRequested(_)));
    assert!(
        !has_interaction,
        "InteractionRequested must NOT be projected — Shell keeps it in-memory only"
    );
}

#[tokio::test]
async fn c3_replay_honest_partial_skips_xai_extension_updates() {
    // xAI extension updates (RewindMarker, AutoCompact, Subagent*, etc.)
    // have no RuntimeEvent representation. They are skipped honestly.
    let temp = TempDir::new().unwrap();
    let port = real_port(&temp);
    let cwd = "/work/replay-xai";
    let sid = start(&temp, &port, cwd).await;
    append_acp(&temp, &sid, cwd, user_chunk("before xai")).await;
    // Append a xAI extension update (MemoryFlushStarted).
    let storage = JsonlStorageAdapter::with_root(temp.path().to_path_buf());
    let info = Info {
        id: acp::SessionId::new(sid.clone()),
        cwd: cwd.to_string(),
    };
    let xai_notif = xai_grok_shell::extensions::notification::SessionNotification {
        session_id: acp::SessionId::new(sid.clone()),
        update: xai_grok_shell::extensions::notification::SessionUpdate::MemoryFlushStarted,
        meta: None,
    };
    storage
        .append_update(&info, &SessionUpdate::Xai(Box::new(xai_notif)))
        .await
        .unwrap();
    append_acp(&temp, &sid, cwd, agent_chunk("after xai")).await;

    let page = port
        .replay(SubscribeParams {
            session_id: sid,
            after_event_seq: WireCounter::new(0),
            history_epoch: None,
        })
        .await
        .unwrap();

    // The xAI update contributes no RuntimeEvent; only snapshot + user +
    // agent delta appear. The projector does not panic on xAI updates.
    assert!(
        page.events.len() >= 2,
        "snapshot + at least one projected event"
    );
    // No item references the xAI extension.
    let all_items_have_real_bodies = page.events.iter().all(|e| {
        matches!(
            e,
            RuntimeEvent::SessionChanged(_)
                | RuntimeEvent::ItemStarted(_)
                | RuntimeEvent::ItemDelta { .. }
                | RuntimeEvent::ItemCompleted(_)
        )
    });
    assert!(all_items_have_real_bodies, "xAI extension updates skipped");
}

// ---------------------------------------------------------------------------
// Cursor / crash-restart cases
// ---------------------------------------------------------------------------

#[tokio::test]
async fn c3_replay_cursor_beyond_end_returns_empty_page() {
    // Stale cursor: after_event_seq beyond the event stream returns an empty
    // page with replayed_through at total (no next cursor). This is the
    // crash/restart "cursor stale" case — the client asked for events past
    // the end and gets an honest empty page, not a synthesized replay.
    let temp = TempDir::new().unwrap();
    let port = real_port(&temp);
    let cwd = "/work/replay-stale";
    let sid = start(&temp, &port, cwd).await;
    append_acp(&temp, &sid, cwd, user_chunk("only")).await;

    let page = port
        .replay(SubscribeParams {
            session_id: sid,
            after_event_seq: WireCounter::new(10_000),
            history_epoch: None,
        })
        .await
        .unwrap();

    assert!(page.events.is_empty(), "stale cursor → empty page");
    assert!(page.next_cursor.is_none(), "no next cursor past end");
    // replayed_through reflects the total event count (snapshot + 1).
    assert!(page.replayed_through.as_u64() >= 1);
}

#[tokio::test]
async fn c3_replay_snapshot_always_event_zero() {
    // Regardless of updates, event 0 is the SessionChanged snapshot projected
    // from the real summary. Crash/restart: a fresh replay always begins with
    // the canonical snapshot so the client can resync.
    let temp = TempDir::new().unwrap();
    let port = real_port(&temp);
    let cwd = "/work/replay-snap";
    let sid = start(&temp, &port, cwd).await;

    let page = port
        .replay(SubscribeParams {
            session_id: sid.clone(),
            after_event_seq: WireCounter::new(0),
            history_epoch: None,
        })
        .await
        .unwrap();

    assert!(
        matches!(page.events.first(), Some(RuntimeEvent::SessionChanged(_))),
        "event 0 is the SessionChanged snapshot"
    );
}

#[tokio::test]
async fn c3_read_session_crash_mid_turn_inferred_completed_partial() {
    // Crash-mid-turn case: a user message and a partial agent chunk are
    // persisted, but no completion marker exists (Shell writes none). The
    // projector infers turn status Completed from persistence — this is the
    // documented PARTIAL: crash-mid-turn is not detected, so the last turn
    // is reported Completed even if the actor crashed mid-stream. The test
    // documents this inference honestly (it is the safest reversible read;
    // the alternative — reporting InProgress forever — is worse).
    let temp = TempDir::new().unwrap();
    let port = real_port(&temp);
    let cwd = "/work/crash-mid-turn";
    let sid = start(&temp, &port, cwd).await;
    append_acp(&temp, &sid, cwd, user_chunk("start")).await;
    append_acp(&temp, &sid, cwd, agent_chunk("partial...")).await;
    // No completion marker — simulate crash mid-turn (no more updates).

    let result = port
        .read_session(SessionReadParams {
            session_id: sid,
            include_turns: true,
            include_items: true,
        })
        .await
        .unwrap();

    assert_eq!(result.turns.len(), 1);
    // PARTIAL: inferred Completed from persistence (crash-mid-turn not
    // detected). This assertion documents the inference — not a claim that
    // the turn truly completed.
    assert_eq!(result.turns[0].status, TurnStatus::Completed);
    assert!(
        !result.items.is_empty(),
        "partial agent chunk still projected"
    );
}
