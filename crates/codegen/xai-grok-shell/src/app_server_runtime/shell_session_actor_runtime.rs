//! Shell-owned `GrokRuntimeFacade` backed by the real JSONL storage adapter.
//!
//! This is the C1-D real port. It maps every facade method to an *existing*
//! Shell symbol (see `waves/c0-session-actor-command-map.md` §1) so the
//! composition root can switch off `FakeRuntime` for the experimental product
//! path without introducing a second `SessionActor`, a second permission
//! engine, a second replay buffer, or a hybrid Fake+JSONL authority.
//!
//! ## Authority
//! One authority per session: the real `JsonlStorageAdapter`. Reads and
//! writes go through the same on-disk `summary.json` / `updates.jsonl` /
//! `chat_history.jsonl` layout that the live `SessionActor` uses. `FakeRuntime`
//! is NOT mixed in here (corrective contract §2 / audit F-13).
//!
//! ## What is real vs PARTIAL (C1-D)
//! - **REAL (storage-backed):** `list_sessions`, `read_session` (session row),
//!   `start_session` (persists `summary.json` via `init_session`),
//!   `resume_session` (loads `summary.json`), `fork_session`
//!   (`copy_session_data`), `replay` (projects `updates.jsonl`).
//! - **PARTIAL (actor fixture gap):** `start_turn`, `steer_turn`,
//!   `interrupt_turn` require a live `SessionActor` (`!Send`, dedicated thread
//!   + `LocalSet` + auth/credentials/tool-context). The adapter returns
//!   `unsupported` for these and documents the gap in `waves/c1-shell-port.md`.
//! - **PARTIAL (product decision pending):** `archive_session` (R6) — only
//!   destructive `delete_session` or disk-keeping `close_session_explicit`
//!   exist; mapping `archive` → `delete` is data loss. Returns `unsupported`
//!   (safest reversible no-op stub) until the product decision lands.
//! - **PARTIAL (design pending):** `respond_interaction` (R10) — no
//!   `SessionCommand` exists for this; the parked oneshot is resolved via the
//!   leader's ACP response forwarding. Returns `unsupported` until the
//!   delivery-channel design is implemented.
//! - **PARTIAL (projection):** `read_session` Turn/Item projection (R2) and
//!   `replay` full `RuntimeEvent` projection (R11) — Shell has no first-class
//!   `Turn`/`Item`/`RuntimeEvent` projector; the adapter builds a minimal one
//!   over `updates.jsonl`. Full lifecycle projection is deferred.
//!
//! `MvpAgent`/`SessionActor` are NOT reinvented here. The turn lifecycle
//! forwards to the existing `SessionCommand::{Prompt,Interject,Cancel}` once
//! the actor fixture is wired (Wave C1 follow-on).

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use async_trait::async_trait;
use xai_grok_app_server_protocol::{
    InteractionResponseParams, Item, ItemBody, ItemStatus, Session, SessionArchiveParams,
    SessionForkParams, SessionReadParams, SessionReadResult, SessionResumeParams,
    SessionStartParams, SessionStatus, SubscribeParams, Turn, TurnInterruptParams, TurnStartParams,
    TurnSteerParams, WireCounter,
};
use xai_grok_tower::{GrokRuntimeFacade, ReplayPage, RuntimeError, RuntimeEvent};

use crate::session::info::Info;
use crate::session::persistence::{Summary, default_model_id};
use crate::session::storage::{JsonlStorageAdapter, StorageAdapter, UpdatesIterator};

/// History epoch used by the real port for sessions that have no compaction
/// generation yet. Shell's `Summary` has no epoch field; this stable constant
/// matches the Fake conformance value and is documented as a placeholder until
/// a real epoch concept lands in Shell (audit F-13 root cause was a synthetic
/// *per-row* epoch — this is a single stable value, not synthetic per-row).
const HISTORY_EPOCH: &str = "epoch_1";

/// Replay page size (events per page). Matches the Fake conformance bound.
const REPLAY_PAGE_SIZE: usize = 100;

/// Shell-owned `GrokRuntimeFacade` backed by the real JSONL storage adapter.
///
/// Construct with [`ShellSessionActorRuntime::new`] (product uses `grok_home()`;
/// tests pass a `TempDir` root). The adapter is `Send + Sync` because
/// `JsonlStorageAdapter` is `Clone + Send + Sync` and all actor state lives on
/// the actor's own thread (not held here).
pub struct ShellSessionActorRuntime {
    storage: JsonlStorageAdapter,
    /// `idempotency_key -> (session_id, input_digest)` for start-session dedup.
    /// Mirrors `FakeRuntime`'s idempotency contract without mixing authorities.
    idempotency: Mutex<HashMap<String, (String, String)>>,
}

impl ShellSessionActorRuntime {
    /// Build a real port rooted at `root` (product: `grok_home()`; tests: TempDir).
    pub fn new(root: PathBuf) -> Self {
        Self {
            storage: JsonlStorageAdapter::with_root(root),
            idempotency: Mutex::new(HashMap::new()),
        }
    }

    /// Build a real port over an explicit storage adapter (test seam).
    pub fn with_storage(storage: JsonlStorageAdapter) -> Self {
        Self {
            storage,
            idempotency: Mutex::new(HashMap::new()),
        }
    }

    /// Compute the input digest for start-session idempotency (matches Fake).
    fn start_digest(params: &SessionStartParams) -> String {
        format!("{}|{:?}", params.workspace_root, params.provider_binding)
    }

    /// Find a session's `Info` by id across all workspaces (O(n) scan).
    ///
    /// `SessionResumeParams` / `SubscribeParams` carry no `cwd`, but the JSONL
    /// layout is `{root}/sessions/{urlencoded(cwd)}/{session_id}/`. Resolving
    /// the cwd requires reading the summary, which requires the path, which
    /// requires the cwd — so we scan all summaries and match by id (R4).
    async fn find_info(&self, session_id: &str) -> Result<Info, RuntimeError> {
        let summaries = self
            .storage
            .list_sessions(None)
            .await
            .map_err(io_err_to_runtime)?;
        for s in summaries {
            if s.info.id.0.as_ref() == session_id {
                return Ok(s.info);
            }
        }
        Err(RuntimeError {
            code: "session_not_found",
            message: "Session was not found.".into(),
        })
    }
}

fn io_err_to_runtime(e: std::io::Error) -> RuntimeError {
    RuntimeError {
        code: "storage_error",
        message: e.to_string(),
    }
}

/// Project a Shell `Summary` into a protocol `Session` (C0-B §1.1 R1).
///
/// This replaces the dormant `project_active_session_row` stub: status,
/// workspace, timestamps, and title come from the real on-disk summary, not
/// hardcoded `epoch_1`/revision 0/`Dormant`.
fn project_summary_to_session(summary: &Summary) -> Session {
    let title = summary.display_title();
    let title = if title.is_empty() {
        None
    } else {
        Some(title.to_string())
    };
    // Shell `Summary` has no live-status field; a persisted session with no
    // actor resident is `Ready` (resumable). `Running`/`WaitingForInput`
    // require the live actor (PARTIAL — not represented here).
    let status = if summary.num_messages == 0 {
        SessionStatus::Starting
    } else {
        SessionStatus::Ready
    };
    let revision = WireCounter::new(summary.num_messages.max(1) as u64);
    Session {
        session_id: summary.info.id.0.to_string(),
        history_epoch: HISTORY_EPOCH.to_string(),
        revision,
        status,
        workspace_root: summary.info.cwd.clone(),
        title,
        active_turn_id: None,
        latest_turn_id: None,
        // Shell `Summary` carries `current_model_id` but not a full
        // `ProviderBinding` (credential_id/backend require actor-side
        // resolution). PARTIAL — left None until provider binding flows
        // through `SessionStartParams` to the actor.
        provider_binding: None,
        created_at_ms: summary.created_at.timestamp_millis() as u64,
        updated_at_ms: summary.updated_at.timestamp_millis() as u64,
    }
}

/// Minimal `updates.jsonl` line → `RuntimeEvent` projector (R11 PARTIAL).
///
/// Projects the common ACP streaming chunks into `Item*` events. Full
/// `Turn`/`ToolCall`/`Interaction` lifecycle projection is deferred — this is
/// a projection over existing data, NOT a second replay buffer.
fn project_update_to_event(session_id: &str, seq: u64, update: &crate::session::storage::SessionUpdate) -> Option<RuntimeEvent> {
    use agent_client_protocol as acp;
    let item_id = format!("item_{seq}");
    let turn_id = format!("turn_{seq}");
    let event_seq = WireCounter::new(seq);
    match update {
        crate::session::storage::SessionUpdate::Acp(n) => match &n.update {
            acp::SessionUpdate::AgentMessageChunk(chunk) => {
                if let acp::ContentBlock::Text(text) = &chunk.content {
                    Some(RuntimeEvent::ItemDelta {
                        session_id: session_id.to_string(),
                        turn_id,
                        item_id,
                        revision: WireCounter::new(1),
                        delta: text.text.clone(),
                    })
                } else {
                    None
                }
            }
            acp::SessionUpdate::UserMessageChunk(chunk) => {
                if let acp::ContentBlock::Text(text) = &chunk.content {
                    let item = Item {
                        item_id,
                        session_id: session_id.to_string(),
                        turn_id,
                        event_seq,
                        revision: WireCounter::new(1),
                        status: ItemStatus::Completed,
                        created_at_ms: 0,
                        body: ItemBody::UserMessage {
                            content: vec![xai_grok_app_server_protocol::InputBlock::Text {
                                text: text.text.clone(),
                            }],
                        },
                    };
                    Some(RuntimeEvent::ItemCompleted(item))
                } else {
                    None
                }
            }
            _ => None,
        },
        crate::session::storage::SessionUpdate::Xai(_) => None,
    }
}

#[async_trait]
impl GrokRuntimeFacade for ShellSessionActorRuntime {
    async fn list_sessions(&self) -> Result<Vec<Session>, RuntimeError> {
        let summaries = self
            .storage
            .list_sessions(None)
            .await
            .map_err(io_err_to_runtime)?;
        Ok(summaries.iter().map(project_summary_to_session).collect())
    }

    async fn read_session(
        &self,
        params: SessionReadParams,
    ) -> Result<SessionReadResult, RuntimeError> {
        let info = self.find_info(&params.session_id).await?;
        let summary = self
            .storage
            .load_summary(&info)
            .await
            .map_err(io_err_to_runtime)?;
        let session = project_summary_to_session(&summary);
        // R2 PARTIAL: Shell has no first-class Turn/Item projection. Returns
        // empty until the updates.jsonl → Turn/Item projector lands (shares
        // the R11 projector surface).
        let turns: Vec<Turn> = Vec::new();
        let items: Vec<Item> = Vec::new();
        Ok(SessionReadResult {
            session,
            turns,
            items,
        })
    }

    async fn start_session(&self, params: SessionStartParams) -> Result<Session, RuntimeError> {
        let digest = Self::start_digest(&params);
        // Idempotency-key dedup (R3): same key + same digest → same session.
        // Drop the MutexGuard before any await so the future stays Send.
        let existing = {
            let guard = self.idempotency.lock().unwrap();
            guard.get(&params.idempotency_key).cloned()
        };
        if let Some((existing_id, prev_digest)) = existing {
            if prev_digest != digest {
                return Err(RuntimeError {
                    code: "idempotency_conflict",
                    message: "The idempotency key was already used with different input.".into(),
                });
            }
            let info = self.find_info(&existing_id).await?;
            let summary = self
                .storage
                .load_summary(&info)
                .await
                .map_err(io_err_to_runtime)?;
            return Ok(project_summary_to_session(&summary));
        }
        if params.workspace_root.is_empty() || !params.workspace_root.starts_with('/') {
            return Err(RuntimeError {
                code: "invalid_workspace",
                message: "The workspace cannot be opened.".into(),
            });
        }
        // REAL storage authority: writes summary.json via init_session.
        // NOTE: no SessionActor is spawned (PARTIAL — actor fixture gap).
        let session_id = uuid::Uuid::now_v7().to_string();
        let info = Info {
            id: agent_client_protocol::SessionId::new(session_id.clone()),
            cwd: params.workspace_root.clone(),
        };
        let summary = self
            .storage
            .init_session(&info, default_model_id())
            .await
            .map_err(io_err_to_runtime)?;
        self.idempotency
            .lock()
            .unwrap()
            .insert(params.idempotency_key, (session_id, digest));
        Ok(project_summary_to_session(&summary))
    }

    async fn resume_session(&self, params: SessionResumeParams) -> Result<Session, RuntimeError> {
        // R4: resolve cwd via scan (params carry no cwd).
        let info = self.find_info(&params.session_id).await?;
        let summary = self
            .storage
            .load_summary(&info)
            .await
            .map_err(io_err_to_runtime)?;
        // PARTIAL: no actor drain/replay — returns the persisted session row.
        Ok(project_summary_to_session(&summary))
    }

    async fn fork_session(&self, params: SessionForkParams) -> Result<Session, RuntimeError> {
        let source_info = self.find_info(&params.session_id).await?;
        let source_summary = self
            .storage
            .load_summary(&source_info)
            .await
            .map_err(io_err_to_runtime)?;
        let new_cwd = params
            .workspace_root
            .clone()
            .unwrap_or_else(|| source_summary.info.cwd.clone());
        let new_session_id = uuid::Uuid::now_v7().to_string();
        let target_info = Info {
            id: agent_client_protocol::SessionId::new(new_session_id.clone()),
            cwd: new_cwd.clone(),
        };
        // REAL symbol: `copy_session_data` (the primitive `fork_session`
        // calls internally). No second authority.
        let options = crate::session::storage::CopySessionOptions {
            parent_session_id: Some(params.session_id.clone()),
            session_kind: Some("fork".to_string()),
            ..Default::default()
        };
        self.storage
            .copy_session_data(&source_info, &target_info, options)
            .await
            .map_err(io_err_to_runtime)?;
        let summary = self
            .storage
            .load_summary(&target_info)
            .await
            .map_err(io_err_to_runtime)?;
        Ok(project_summary_to_session(&summary))
    }

    async fn archive_session(&self, params: SessionArchiveParams) -> Result<(), RuntimeError> {
        // R6 PARTIAL: product decision pending. The only existing destructive
        // symbol is `delete_session` (irreversible); mapping `archive` →
        // `delete` is data loss. Safest reversible interpretation: no-op stub
        // returning `unsupported` (review §5.1). Do NOT silently delete.
        let _ = params;
        Err(RuntimeError {
            code: "unsupported",
            message: "archive_session semantics undecided (R6); no-op stub until product decision.".into(),
        })
    }

    async fn start_turn(&self, _params: TurnStartParams) -> Result<Turn, RuntimeError> {
        // PARTIAL: requires a live SessionActor (SessionCommand::Prompt +
        // dispatch_lock). Actor fixture gap — documented in waves/c1-shell-port.md.
        Err(RuntimeError {
            code: "unsupported",
            message: "start_turn requires a live SessionActor (C1-D PARTIAL: actor fixture gap).".into(),
        })
    }

    async fn steer_turn(&self, _params: TurnSteerParams) -> Result<Item, RuntimeError> {
        // PARTIAL: SessionCommand::Interject + current_prompt_id match.
        Err(RuntimeError {
            code: "unsupported",
            message: "steer_turn requires a live SessionActor (C1-D PARTIAL: actor fixture gap).".into(),
        })
    }

    async fn interrupt_turn(&self, _params: TurnInterruptParams) -> Result<(), RuntimeError> {
        // PARTIAL: SessionCommand::Cancel + current_prompt_id match.
        Err(RuntimeError {
            code: "unsupported",
            message: "interrupt_turn requires a live SessionActor (C1-D PARTIAL: actor fixture gap).".into(),
        })
    }

    async fn respond_interaction(&self, _params: InteractionResponseParams) -> Result<(), RuntimeError> {
        // R10 PARTIAL: no SessionCommand exists; parked oneshot is resolved via
        // the leader's ACP response forwarding. Delivery-channel design pending.
        Err(RuntimeError {
            code: "unsupported",
            message: "respond_interaction delivery channel not implemented (R10 PARTIAL).".into(),
        })
    }

    async fn replay(&self, cursor: SubscribeParams) -> Result<ReplayPage, RuntimeError> {
        let info = self.find_info(&cursor.session_id).await?;
        let summary = self
            .storage
            .load_summary(&info)
            .await
            .map_err(io_err_to_runtime)?;
        if let Some(expected) = &cursor.history_epoch {
            if expected.as_str() != HISTORY_EPOCH {
                return Err(RuntimeError {
                    code: "epoch_mismatch",
                    message: "History epoch does not match.".into(),
                });
            }
        }
        // Build the event stream: event 0 = SessionChanged snapshot; events
        // 1..N = projected updates.jsonl lines (R11 PARTIAL projector).
        let mut all_events: Vec<RuntimeEvent> =
            vec![RuntimeEvent::SessionChanged(project_summary_to_session(&summary))];
        if let Some(path) = self.storage.updates_file_path(&info) {
            if let Ok(Some(mut iter)) = UpdatesIterator::open(&path) {
                let mut seq = 1u64;
                while let Some(Ok(update)) = iter.next() {
                    if let Some(event) = project_update_to_event(&cursor.session_id, seq, &update) {
                        all_events.push(event);
                    }
                    seq += 1;
                }
            }
        }
        let after = cursor.after_event_seq.as_u64() as usize;
        let total = all_events.len();
        if after >= total {
            return Ok(ReplayPage {
                events: Vec::new(),
                replayed_through: WireCounter::new(total as u64),
                next_cursor: None,
            });
        }
        let end = (after + REPLAY_PAGE_SIZE).min(total);
        let events = all_events[after..end].to_vec();
        let replayed_through = end as u64;
        let next_cursor = if end < total {
            Some(WireCounter::new(replayed_through))
        } else {
            None
        };
        Ok(ReplayPage {
            events,
            replayed_through: WireCounter::new(replayed_through),
            next_cursor,
        })
    }
}

#[cfg(test)]
mod port_invariant_tests {
    use super::*;

    /// The real port must not define a SessionActor type (Tower guard mirrors).
    #[test]
    fn shell_session_actor_runtime_defines_no_session_actor() {
        let src = include_str!("shell_session_actor_runtime.rs");
        let production = src.split("#[cfg(test)]").next().unwrap();
        assert!(!production.contains("struct SessionActor"));
        assert!(!production.contains("enum SessionActor"));
    }

    /// The real port must not construct or import FakeRuntime (no hybrid authority).
    /// Doc comments may mention FakeRuntime by name; this checks for actual
    /// code usage (`FakeRuntime::new` / `use ...FakeRuntime`), not prose.
    #[test]
    fn shell_session_actor_runtime_does_not_use_fake_runtime() {
        let src = include_str!("shell_session_actor_runtime.rs");
        let production = src.split("#[cfg(test)]").next().unwrap();
        assert!(
            !production.contains("FakeRuntime::new")
                && !production.contains("use xai_grok_tower::FakeRuntime")
                && !production.contains(": FakeRuntime"),
            "real port must not mix FakeRuntime authority"
        );
    }
}
