//! Faithful in-memory runtime used by contract/conformance tests.
//!
//! This is not a second production actor: production injects a Shell adapter.
//! The fake mirrors facade semantics (ids, revisions, epochs, idempotency).

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use async_trait::async_trait;
use xai_grok_app_server_protocol::{
    InputBlock, InteractionRequest, InteractionResponseParams, Item, ItemBody, ItemStatus,
    Session, SessionArchiveParams, SessionForkParams, SessionReadParams, SessionReadResult,
    SessionResumeParams, SessionStartParams, SessionStatus, SubscribeParams, Turn, TurnInterruptParams,
    TurnKind, TurnStartParams, TurnStatus, TurnSteerParams, WireCounter,
};

use crate::{
    admit_resident, admit_turn, project_runtime_event, GrokRuntimeFacade, ReplayPage, ResourceBudgets,
    ResourceUsage, RuntimeError, RuntimeEvent, SessionRegistry,
};

#[derive(Debug, Default)]
struct FakeState {
    sessions: HashMap<String, Session>,
    turns: HashMap<String, Vec<Turn>>,
    items: HashMap<String, Vec<Item>>,
    events: HashMap<String, Vec<RuntimeEvent>>,
    /// idempotency_key -> (result_id, canonical_input_digest)
    idempotency: HashMap<String, (String, String)>,
    next_event_seq: HashMap<String, u64>,
    usage: ResourceUsage,
    budgets: ResourceBudgets,
    registry: SessionRegistry,
}

pub struct FakeRuntime {
    state: Mutex<FakeState>,
    clock_ms: AtomicU64,
    id_counter: AtomicU64,
}

impl Default for FakeRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl FakeRuntime {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(FakeState {
                budgets: ResourceBudgets::default(),
                ..FakeState::default()
            }),
            clock_ms: AtomicU64::new(1_784_376_000_000),
            id_counter: AtomicU64::new(1),
        }
    }

    pub fn with_budgets(budgets: ResourceBudgets) -> Self {
        let rt = Self::new();
        rt.state.lock().unwrap().budgets = budgets;
        rt
    }

    fn next_id(&self, prefix: &str) -> String {
        let n = self.id_counter.fetch_add(1, Ordering::SeqCst);
        format!("{prefix}_{n}")
    }

    fn now(&self) -> u64 {
        self.clock_ms.fetch_add(1, Ordering::SeqCst)
    }

    fn push_event(state: &mut FakeState, session_id: &str, event: RuntimeEvent) {
        let seq = state.next_event_seq.entry(session_id.to_owned()).or_insert(0);
        *seq += 1;
        state
            .events
            .entry(session_id.to_owned())
            .or_default()
            .push(project_runtime_event(event));
    }
}

#[async_trait]
impl GrokRuntimeFacade for FakeRuntime {
    async fn list_sessions(&self) -> Result<Vec<Session>, RuntimeError> {
        let state = self.state.lock().unwrap();
        let mut sessions: Vec<_> = state.sessions.values().cloned().collect();
        sessions.sort_by(|a, b| a.session_id.cmp(&b.session_id));
        Ok(sessions)
    }

    async fn read_session(
        &self,
        params: SessionReadParams,
    ) -> Result<SessionReadResult, RuntimeError> {
        let state = self.state.lock().unwrap();
        let session = state
            .sessions
            .get(&params.session_id)
            .cloned()
            .ok_or_else(|| RuntimeError {
                code: "session_not_found",
                message: "Session was not found.".into(),
            })?;
        let turns = if params.include_turns {
            state
                .turns
                .get(&params.session_id)
                .cloned()
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        let items = if params.include_items {
            state
                .items
                .get(&params.session_id)
                .cloned()
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        Ok(SessionReadResult {
            session,
            turns,
            items,
        })
    }

    async fn start_session(&self, params: SessionStartParams) -> Result<Session, RuntimeError> {
        let mut state = self.state.lock().unwrap();
        let digest = format!(
            "{}|{:?}",
            params.workspace_root, params.provider_binding
        );
        if let Some((existing_id, prev_digest)) = state.idempotency.get(&params.idempotency_key) {
            if prev_digest != &digest {
                return Err(RuntimeError {
                    code: "idempotency_conflict",
                    message: "The idempotency key was already used with different input.".into(),
                });
            }
            return state
                .sessions
                .get(existing_id)
                .cloned()
                .ok_or_else(|| RuntimeError {
                    code: "internal_error",
                    message: "idempotent session missing".into(),
                });
        }
        if params.workspace_root.is_empty() || !params.workspace_root.starts_with('/') {
            return Err(RuntimeError {
                code: "invalid_workspace",
                message: "The workspace cannot be opened.".into(),
            });
        }
        admit_resident(&state.budgets, &state.usage)?;
        let session_id = self.next_id("session");
        let now = self.now();
        let session = Session {
            session_id: session_id.clone(),
            history_epoch: "epoch_1".into(),
            revision: WireCounter::new(1),
            status: SessionStatus::Ready,
            workspace_root: params.workspace_root,
            title: None,
            active_turn_id: None,
            latest_turn_id: None,
            provider_binding: params.provider_binding,
            created_at_ms: now,
            updated_at_ms: now,
        };
        state
            .registry
            .get_or_insert_with(&session_id, |_| Ok(()))?;
        state.usage.record_resident(1);
        state.sessions.insert(session_id.clone(), session.clone());
        state
            .idempotency
            .insert(params.idempotency_key, (session_id.clone(), digest));
        Self::push_event(
            &mut state,
            &session_id,
            RuntimeEvent::SessionChanged(session.clone()),
        );
        Ok(session)
    }

    async fn resume_session(&self, params: SessionResumeParams) -> Result<Session, RuntimeError> {
        let mut state = self.state.lock().unwrap();
        let status = state
            .sessions
            .get(&params.session_id)
            .map(|s| s.status.clone())
            .ok_or_else(|| RuntimeError {
                code: "session_not_found",
                message: "Session was not found.".into(),
            })?;
        if matches!(status, SessionStatus::Archived) {
            return Err(RuntimeError {
                code: "invalid_state",
                message: "Session is archived.".into(),
            });
        }
        if matches!(status, SessionStatus::Dormant) {
            admit_resident(&state.budgets, &state.usage)?;
            state.usage.record_resident(1);
            state
                .registry
                .get_or_insert_with(&params.session_id, |_| Ok(()))?;
        }
        let session = state
            .sessions
            .get_mut(&params.session_id)
            .expect("session present");
        if matches!(status, SessionStatus::Dormant) {
            session.status = SessionStatus::Ready;
        }
        session.updated_at_ms = self.now();
        let out = session.clone();
        Self::push_event(
            &mut state,
            &params.session_id,
            RuntimeEvent::SessionChanged(out.clone()),
        );
        Ok(out)
    }

    async fn fork_session(&self, params: SessionForkParams) -> Result<Session, RuntimeError> {
        let source = {
            let state = self.state.lock().unwrap();
            state
                .sessions
                .get(&params.session_id)
                .cloned()
                .ok_or_else(|| RuntimeError {
                    code: "session_not_found",
                    message: "Session was not found.".into(),
                })?
        };
        self.start_session(SessionStartParams {
            workspace_root: params
                .workspace_root
                .unwrap_or_else(|| source.workspace_root.clone()),
            agent_type: None,
            provider_binding: source.provider_binding.clone(),
            idempotency_key: params.idempotency_key,
        })
        .await
    }

    async fn archive_session(&self, params: SessionArchiveParams) -> Result<(), RuntimeError> {
        let mut state = self.state.lock().unwrap();
        let status = state
            .sessions
            .get(&params.session_id)
            .map(|s| s.status.clone())
            .ok_or_else(|| RuntimeError {
                code: "session_not_found",
                message: "Session was not found.".into(),
            })?;
        if !matches!(status, SessionStatus::Archived) {
            if state.registry.remove(&params.session_id).is_some() {
                state.usage.record_resident(-1);
            }
            let session = state
                .sessions
                .get_mut(&params.session_id)
                .expect("session present");
            session.status = SessionStatus::Archived;
            session.updated_at_ms = self.now();
        }
        Ok(())
    }

    async fn start_turn(&self, params: TurnStartParams) -> Result<Turn, RuntimeError> {
        let mut state = self.state.lock().unwrap();
        let turn_digest = format!("{:?}|{}", params.input, params.session_id);
        if let Some((existing_id, prev_digest)) = state.idempotency.get(&params.idempotency_key) {
            if prev_digest != &turn_digest {
                return Err(RuntimeError {
                    code: "idempotency_conflict",
                    message: "The idempotency key was already used with different input.".into(),
                });
            }
            let turns = state.turns.get(&params.session_id).ok_or_else(|| RuntimeError {
                code: "internal_error",
                message: "idempotent turn missing".into(),
            })?;
            return turns
                .iter()
                .find(|t| &t.turn_id == existing_id)
                .cloned()
                .ok_or_else(|| RuntimeError {
                    code: "internal_error",
                    message: "idempotent turn missing".into(),
                });
        }
        let session_status = state
            .sessions
            .get(&params.session_id)
            .map(|s| s.status.clone())
            .ok_or_else(|| RuntimeError {
                code: "session_not_found",
                message: "Session was not found.".into(),
            })?;
        if matches!(session_status, SessionStatus::Archived) {
            return Err(RuntimeError {
                code: "invalid_state",
                message: "Session is archived.".into(),
            });
        }
        admit_turn(&state.budgets, &state.usage)?;
        let turn_id = self.next_id("turn");
        let now = self.now();
        let ordinal = state
            .turns
            .get(&params.session_id)
            .map(|t| t.len() as u64 + 1)
            .unwrap_or(1);
        let provider_binding = state
            .sessions
            .get(&params.session_id)
            .and_then(|s| s.provider_binding.clone());
        let turn = Turn {
            turn_id: turn_id.clone(),
            session_id: params.session_id.clone(),
            provider_binding,
            revision: WireCounter::new(1),
            status: TurnStatus::InProgress,
            kind: TurnKind::User,
            ordinal,
            created_at_ms: now,
            completed_at_ms: None,
        };
        {
            let session = state
                .sessions
                .get_mut(&params.session_id)
                .expect("session present");
            session.status = SessionStatus::Running;
            session.active_turn_id = Some(turn_id.clone());
            session.latest_turn_id = Some(turn_id.clone());
            session.updated_at_ms = now;
            session.revision = WireCounter::new(session.revision.as_u64() + 1);
        }
        state.usage.record_turn(1);
        state
            .turns
            .entry(params.session_id.clone())
            .or_default()
            .push(turn.clone());
        state
            .idempotency
            .insert(params.idempotency_key.clone(), (turn_id.clone(), turn_digest));

        let user_item = Item {
            item_id: self.next_id("item"),
            session_id: params.session_id.clone(),
            turn_id: turn_id.clone(),
            event_seq: WireCounter::new(0),
            revision: WireCounter::new(1),
            status: ItemStatus::Completed,
            created_at_ms: now,
            body: ItemBody::UserMessage {
                content: params.input.clone(),
            },
        };
        let agent_text = input_text(&params.input);
        let agent_item = Item {
            item_id: self.next_id("item"),
            session_id: params.session_id.clone(),
            turn_id: turn_id.clone(),
            event_seq: WireCounter::new(0),
            revision: WireCounter::new(1),
            status: ItemStatus::Completed,
            created_at_ms: now,
            body: ItemBody::AgentMessage {
                text: format!("echo: {agent_text}"),
            },
        };
        state
            .items
            .entry(params.session_id.clone())
            .or_default()
            .extend([user_item.clone(), agent_item.clone()]);

        Self::push_event(
            &mut state,
            &params.session_id,
            RuntimeEvent::TurnChanged(turn.clone()),
        );
        Self::push_event(
            &mut state,
            &params.session_id,
            RuntimeEvent::ItemStarted(user_item.clone()),
        );
        Self::push_event(
            &mut state,
            &params.session_id,
            RuntimeEvent::ItemCompleted(user_item),
        );
        Self::push_event(
            &mut state,
            &params.session_id,
            RuntimeEvent::ItemStarted(agent_item.clone()),
        );
        Self::push_event(
            &mut state,
            &params.session_id,
            RuntimeEvent::ItemDelta {
                session_id: params.session_id.clone(),
                turn_id: turn_id.clone(),
                item_id: agent_item.item_id.clone(),
                revision: WireCounter::new(1),
                delta: format!("echo: {agent_text}"),
            },
        );
        Self::push_event(
            &mut state,
            &params.session_id,
            RuntimeEvent::ItemCompleted(agent_item),
        );

        // Complete turn immediately in the fake (deterministic vertical slice).
        // Return the *completed* snapshot so Fake and real adapters agree on
        // the start_turn status (C7-B F-2). The earlier TurnChanged event still
        // carries InProgress for replay lifecycle fidelity.
        let completed_at = self.now();
        if let Some(t) = state
            .turns
            .get_mut(&params.session_id)
            .and_then(|turns| turns.iter_mut().find(|t| t.turn_id == turn_id))
        {
            t.status = TurnStatus::Completed;
            t.completed_at_ms = Some(completed_at);
            t.revision = WireCounter::new(2);
        }
        if let Some(session) = state.sessions.get_mut(&params.session_id) {
            session.status = SessionStatus::Ready;
            session.active_turn_id = None;
            session.updated_at_ms = completed_at;
        }
        state.usage.record_turn(-1);
        let mut completed_turn = turn;
        completed_turn.status = TurnStatus::Completed;
        completed_turn.completed_at_ms = Some(completed_at);
        completed_turn.revision = WireCounter::new(2);
        Ok(completed_turn)
    }

    async fn steer_turn(&self, params: TurnSteerParams) -> Result<Item, RuntimeError> {
        let text = input_text(&params.input);
        Ok(Item {
            item_id: self.next_id("item"),
            session_id: params.session_id,
            turn_id: params.turn_id,
            event_seq: WireCounter::new(0),
            revision: WireCounter::new(1),
            status: ItemStatus::Completed,
            created_at_ms: self.now(),
            body: ItemBody::UserMessage {
                content: vec![InputBlock::Text { text }],
            },
        })
    }

    async fn interrupt_turn(&self, params: TurnInterruptParams) -> Result<(), RuntimeError> {
        let mut state = self.state.lock().unwrap();
        let turn = state
            .turns
            .get_mut(&params.session_id)
            .and_then(|turns| turns.iter_mut().find(|t| t.turn_id == params.turn_id))
            .ok_or_else(|| RuntimeError {
                code: "turn_not_found",
                message: "Turn was not found.".into(),
            })?;
        turn.status = TurnStatus::Interrupted;
        turn.completed_at_ms = Some(self.now());
        if let Some(session) = state.sessions.get_mut(&params.session_id) {
            session.active_turn_id = None;
            session.status = SessionStatus::Ready;
        }
        Ok(())
    }

    async fn respond_interaction(
        &self,
        _params: InteractionResponseParams,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    async fn replay(&self, cursor: SubscribeParams) -> Result<ReplayPage, RuntimeError> {
        let state = self.state.lock().unwrap();
        if !state.sessions.contains_key(&cursor.session_id) {
            return Err(RuntimeError {
                code: "session_not_found",
                message: "Session was not found.".into(),
            });
        }
        if let Some(expected) = &cursor.history_epoch {
            let session = state.sessions.get(&cursor.session_id).unwrap();
            if &session.history_epoch != expected {
                return Err(RuntimeError {
                    code: "epoch_mismatch",
                    message: "History epoch does not match.".into(),
                });
            }
        }
        let after = cursor.after_event_seq.as_u64();
        let all = state
            .events
            .get(&cursor.session_id)
            .cloned()
            .unwrap_or_default();
        let page_size = 100usize;
        let events: Vec<_> = all.into_iter().skip(after as usize).take(page_size).collect();
        let replayed_through = after + events.len() as u64;
        let next_cursor = if (replayed_through as usize)
            < state
                .events
                .get(&cursor.session_id)
                .map(|e| e.len())
                .unwrap_or(0)
        {
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

fn input_text(input: &[InputBlock]) -> String {
    input
        .iter()
        .filter_map(|b| match b {
            InputBlock::Text { text } => Some(text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ")
}

// Silence unused InteractionRequest import warning if not used yet.
#[allow(dead_code)]
fn _interaction_type_anchor() -> Option<InteractionRequest> {
    None
}

#[cfg(test)]
mod fake_conformance_tests {
    use super::*;

    #[tokio::test]
    async fn fake_conformance_session_turn_item_idempotency() {
        let rt = FakeRuntime::new();
        let session = rt
            .start_session(SessionStartParams {
                workspace_root: "/work".into(),
                agent_type: Some("build".into()),
                provider_binding: None,
                idempotency_key: "start-1".into(),
            })
            .await
            .unwrap();
        let again = rt
            .start_session(SessionStartParams {
                workspace_root: "/work".into(),
                agent_type: Some("build".into()),
                provider_binding: None,
                idempotency_key: "start-1".into(),
            })
            .await
            .unwrap();
        assert_eq!(session.session_id, again.session_id);

        let turn = rt
            .start_turn(TurnStartParams {
                session_id: session.session_id.clone(),
                input: vec![InputBlock::Text {
                    text: "hello".into(),
                }],
                idempotency_key: "turn-1".into(),
            })
            .await
            .unwrap();
        let turn2 = rt
            .start_turn(TurnStartParams {
                session_id: session.session_id.clone(),
                input: vec![InputBlock::Text {
                    text: "hello".into(),
                }],
                idempotency_key: "turn-1".into(),
            })
            .await
            .unwrap();
        assert_eq!(turn.turn_id, turn2.turn_id);

        let page = rt
            .replay(SubscribeParams {
                session_id: session.session_id.clone(),
                after_event_seq: WireCounter::new(0),
                history_epoch: Some("epoch_1".into()),
            })
            .await
            .unwrap();
        assert!(!page.events.is_empty());
        assert!(page.replayed_through.as_u64() >= 1);
    }

    #[tokio::test]
    async fn facade_shape_covers_session_turn_interaction_replay() {
        let rt = FakeRuntime::new();
        let session = rt
            .start_session(SessionStartParams {
                workspace_root: "/work".into(),
                agent_type: None,
                provider_binding: None,
                idempotency_key: "s".into(),
            })
            .await
            .unwrap();
        let _ = rt.list_sessions().await.unwrap();
        let _ = rt
            .read_session(SessionReadParams {
                session_id: session.session_id.clone(),
                include_turns: true,
                include_items: true,
            })
            .await
            .unwrap();
        let turn = rt
            .start_turn(TurnStartParams {
                session_id: session.session_id.clone(),
                input: vec![InputBlock::Text {
                    text: "x".into(),
                }],
                idempotency_key: "t".into(),
            })
            .await
            .unwrap();
        let _ = rt
            .steer_turn(TurnSteerParams {
                session_id: session.session_id.clone(),
                turn_id: turn.turn_id.clone(),
                input: vec![InputBlock::Text {
                    text: "y".into(),
                }],
                idempotency_key: "st".into(),
            })
            .await
            .unwrap();
        let _ = rt
            .interrupt_turn(TurnInterruptParams {
                session_id: session.session_id.clone(),
                turn_id: turn.turn_id.clone(),
                idempotency_key: "i".into(),
            })
            .await
            .unwrap();
        let _ = rt
            .respond_interaction(InteractionResponseParams {
                session_id: session.session_id.clone(),
                turn_id: turn.turn_id,
                interaction_id: "ix".into(),
                decision: "allow".into(),
                idempotency_key: "r".into(),
            })
            .await
            .unwrap();
        let _ = rt
            .replay(SubscribeParams {
                session_id: session.session_id.clone(),
                after_event_seq: 0.into(),
                history_epoch: None,
            })
            .await
            .unwrap();
        let _ = rt
            .archive_session(SessionArchiveParams {
                session_id: session.session_id,
                idempotency_key: "a".into(),
            })
            .await
            .unwrap();
    }
}
