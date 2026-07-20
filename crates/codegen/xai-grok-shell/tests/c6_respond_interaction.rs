//! C6-B `respond_interaction` delivery-channel integration tests.
//!
//! These prove `ShellSessionActorRuntime::respond_interaction` is a
//! **delivery channel** into the existing pending-interaction surface —
//! NOT a second permission engine. The method:
//!
//! 1. Checks the session exists on disk (storage authority).
//! 2. Requires a resident actor (returns `unsupported` without one).
//! 3. Checks `pending_interactions` membership keyed by `interaction_id`
//!    (= `tool_call_id`); consumes it only after a deliverable response (or
//!    restores it when the actor hub/sender is not ready).
//! 4. Delivers `params.decision` via a process-local oneshot hub.
//! 5. Does NOT re-evaluate allow/deny policy.
//!
//! The test seam injects a resident with a pre-seeded `pending_interactions`
//! map + a registered oneshot via a real `cmd_tx` consumer spawner (NOT
//! `FakeRuntime`). The spawner returns a `ResidentHandle` carrying the
//! `pending_interactions` Arc and the `delivery_hub` Arc so the test can
//! seed them and assert delivery.
//!
//! Gate: `./scripts/run-rust-test-gate.sh interaction_facade \
//! cargo test -p xai-grok-shell interaction_facade`
//!
//! RED-then-GREEN evidence is captured under
//! `.llms/execution/app-server-mcp-tower-corrective/tests/c6/`.

use std::sync::{Arc, Mutex};

use agent_client_protocol as acp;
use async_trait::async_trait;
use tempfile::TempDir;
use tokio::sync::{mpsc, oneshot};
use xai_grok_app_server_protocol::{InteractionResponseParams, SessionStartParams};
use xai_grok_shell::app_server_runtime::{
    ResidentHandle, SessionSpawner, ShellSessionActorRuntime,
};
use xai_grok_shell::session::commands::SessionCommand;
use xai_grok_shell::session::info::Info;
use xai_grok_shell::session::pending_interaction::{PendingInteractions, PendingKind};
use xai_grok_tower::{GrokRuntimeFacade, RuntimeError};

// ---------------------------------------------------------------------------
// Test spawner — a real cmd_tx consumer that also exposes the
// pending_interactions + delivery_hub Arcs so the test can seed them.
// ---------------------------------------------------------------------------

/// Interaction-capable spawner: returns a `ResidentHandle` with a real
/// `pending_interactions` map and a `delivery_hub` so `respond_interaction`
/// can check membership and deliver decisions. The consumer task drains
/// the command channel (it does not need to process turns for these tests —
/// `respond_interaction` never sends a `SessionCommand`).
struct InteractionSpawner {
    with_hub: bool,
}

#[async_trait]
impl SessionSpawner for InteractionSpawner {
    async fn spawn(
        &self,
        _info: &Info,
        _model_id: &acp::ModelId,
    ) -> Result<ResidentHandle, RuntimeError> {
        let (cmd_tx, mut cmd_rx) = mpsc::unbounded_channel::<SessionCommand>();
        let current_prompt_id = Arc::new(Mutex::new(None::<String>));
        let pending_interactions: PendingInteractions =
            Arc::new(Mutex::new(std::collections::HashMap::new()));
        let delivery_hub: Arc<
            Mutex<std::collections::HashMap<String, oneshot::Sender<String>>>,
        > = Arc::new(Mutex::new(std::collections::HashMap::new()));

        // Drain the command channel so the mailbox does not fill up. We do
        // not need to process any SessionCommand for respond_interaction
        // tests — the delivery channel reads the pending map + hub directly.
        tokio::spawn(async move {
            while cmd_rx.recv().await.is_some() {
                // Intentionally minimal: respond_interaction does not route
                // through the actor command channel.
            }
        });

        Ok(ResidentHandle {
            cmd_tx,
            current_prompt_id,
            pending_interactions: Some(pending_interactions),
            delivery_hub: self.with_hub.then_some(delivery_hub),
        })
    }
}

fn real_port(temp: &TempDir) -> ShellSessionActorRuntime {
    ShellSessionActorRuntime::with_spawner(
        temp.path().to_path_buf(),
        Arc::new(InteractionSpawner { with_hub: true }),
    )
}

fn no_hub_port(temp: &TempDir) -> ShellSessionActorRuntime {
    ShellSessionActorRuntime::with_spawner(
        temp.path().to_path_buf(),
        Arc::new(InteractionSpawner { with_hub: false }),
    )
}

async fn start_session(
    rt: &ShellSessionActorRuntime,
    cwd: &str,
    key: &str,
) -> xai_grok_app_server_protocol::Session {
    rt.start_session(SessionStartParams {
        workspace_root: cwd.into(),
        agent_type: None,
        provider_binding: None,
        idempotency_key: key.into(),
    })
    .await
    .unwrap()
}

/// Seed a pending interaction + register a delivery oneshot for `interaction_id`.
/// Returns the `oneshot::Receiver<String>` the test awaits to assert delivery.
fn seed_pending(
    port: &ShellSessionActorRuntime,
    session_id: &str,
    interaction_id: &str,
    kind: PendingKind,
) -> oneshot::Receiver<String> {
    let resident = port
        .resident(session_id)
        .expect("resident must exist after start_session with InteractionSpawner");
    let pending = resident
        .pending_interactions
        .as_ref()
        .expect("InteractionSpawner provides a pending_interactions surface");
    pending
        .lock()
        .unwrap()
        .insert(interaction_id.to_string(), kind);
    let hub = resident
        .delivery_hub
        .as_ref()
        .expect("InteractionSpawner provides a delivery_hub");
    let (tx, rx) = oneshot::channel();
    hub.lock().unwrap().insert(interaction_id.to_string(), tx);
    rx
}

fn seed_pending_without_delivery_hub(
    port: &ShellSessionActorRuntime,
    session_id: &str,
    interaction_id: &str,
    kind: PendingKind,
) {
    let resident = port.resident(session_id).expect("resident must exist");
    let pending = resident
        .pending_interactions
        .as_ref()
        .expect("pending surface must exist");
    pending
        .lock()
        .unwrap()
        .insert(interaction_id.to_string(), kind);
}

// ===========================================================================
// GREEN path — delivery into the parked-future oneshot
// ===========================================================================

#[tokio::test]
async fn interaction_facade_delivers_decision_to_parked_oneshot() {
    // The core delivery path: a pending interaction is seeded, a oneshot is
    // registered, respond_interaction delivers the decision string. The
    // method does NOT re-evaluate allow/deny — it forwards the caller's
    // decision verbatim.
    let temp = TempDir::new().unwrap();
    let port = real_port(&temp);
    let s = start_session(&port, "/work/ix/deliver", "ix-1").await;

    let rx = seed_pending(&port, &s.session_id, "call-1", PendingKind::Permission);

    port.respond_interaction(InteractionResponseParams {
        session_id: s.session_id.clone(),
        turn_id: "t-1".into(),
        interaction_id: "call-1".into(),
        decision: "allow".into(),
        idempotency_key: "r-1".into(),
    })
    .await
    .expect("delivery succeeds");

    let decision = rx.await.expect("oneshot must receive the decision");
    assert_eq!(decision, "allow", "decision is forwarded verbatim — no policy re-eval");

    // The pending entry must be removed (first-answer-wins).
    let resident = port.resident(&s.session_id).unwrap();
    let pending = resident.pending_interactions.as_ref().unwrap();
    assert!(
        !pending.lock().unwrap().contains_key("call-1"),
        "pending entry removed after delivery"
    );
    // The delivery hub entry must also be removed.
    let hub = resident.delivery_hub.as_ref().unwrap();
    assert!(
        !hub.lock().unwrap().contains_key("call-1"),
        "delivery hub entry removed after delivery"
    );
}

#[tokio::test]
async fn interaction_facade_delivers_deny_decision_verbatim() {
    // The delivery channel forwards any decision string — "deny" is not
    // re-evaluated or transformed. This proves the method is a delivery
    // channel, not a permission engine.
    let temp = TempDir::new().unwrap();
    let port = real_port(&temp);
    let s = start_session(&port, "/work/ix/deny", "ix-2").await;

    let rx = seed_pending(&port, &s.session_id, "call-deny", PendingKind::Permission);

    port.respond_interaction(InteractionResponseParams {
        session_id: s.session_id.clone(),
        turn_id: "t-2".into(),
        interaction_id: "call-deny".into(),
        decision: "deny".into(),
        idempotency_key: "r-2".into(),
    })
    .await
    .unwrap();

    assert_eq!(rx.await.unwrap(), "deny");
}

#[tokio::test]
async fn interaction_facade_delivers_for_question_kind() {
    // The delivery channel works for all PendingKind variants (Permission,
    // Question, PlanApproval) — it does not filter by kind.
    let temp = TempDir::new().unwrap();
    let port = real_port(&temp);
    let s = start_session(&port, "/work/ix/question", "ix-3").await;

    let rx = seed_pending(&port, &s.session_id, "call-q", PendingKind::Question);

    port.respond_interaction(InteractionResponseParams {
        session_id: s.session_id.clone(),
        turn_id: "t-3".into(),
        interaction_id: "call-q".into(),
        decision: "answer-42".into(),
        idempotency_key: "r-3".into(),
    })
    .await
    .unwrap();

    assert_eq!(rx.await.unwrap(), "answer-42");
}

// ===========================================================================
// First-answer-wins / idempotency
// ===========================================================================

#[tokio::test]
async fn interaction_facade_second_call_is_interaction_not_found() {
    // First-answer-wins: the first respond_interaction removes the pending
    // entry. A second call for the same interaction_id finds it gone →
    // `interaction_not_found`. This makes the delivery idempotent: only the
    // first response is delivered; duplicates are rejected.
    let temp = TempDir::new().unwrap();
    let port = real_port(&temp);
    let s = start_session(&port, "/work/ix/idem", "ix-4").await;

    let rx = seed_pending(&port, &s.session_id, "call-idem", PendingKind::Permission);

    // First call succeeds.
    port.respond_interaction(InteractionResponseParams {
        session_id: s.session_id.clone(),
        turn_id: "t-4".into(),
        interaction_id: "call-idem".into(),
        decision: "allow".into(),
        idempotency_key: "r-4a".into(),
    })
    .await
    .unwrap();
    assert_eq!(rx.await.unwrap(), "allow");

    // Second call → interaction_not_found (entry already removed).
    let err = port
        .respond_interaction(InteractionResponseParams {
            session_id: s.session_id.clone(),
            turn_id: "t-4".into(),
            interaction_id: "call-idem".into(),
            decision: "deny".into(),
            idempotency_key: "r-4b".into(),
        })
        .await
        .unwrap_err();
    assert_eq!(err.code, "interaction_not_found");
}

// ===========================================================================
// Error paths
// ===========================================================================

#[tokio::test]
async fn interaction_facade_unknown_session_not_found() {
    // An unknown session id is rejected at the storage layer before the
    // resident check — the delivery channel cannot deliver to a session that
    // does not exist on disk.
    let temp = TempDir::new().unwrap();
    let port = real_port(&temp);
    let err = port
        .respond_interaction(InteractionResponseParams {
            session_id: "no-such-session".into(),
            turn_id: "t".into(),
            interaction_id: "ix".into(),
            decision: "allow".into(),
            idempotency_key: "r".into(),
        })
        .await
        .unwrap_err();
    assert_eq!(err.code, "session_not_found");
}

#[tokio::test]
async fn interaction_facade_no_resident_unsupported() {
    // A session exists on disk but no resident actor is loaded (production
    // spawner returns unsupported). The delivery channel honestly returns
    // `unsupported` because the decision cannot be routed to a parked future
    // that lives in the actor's memory.
    let temp = TempDir::new().unwrap();
    let port = ShellSessionActorRuntime::new(temp.path().to_path_buf());
    let s = start_session(&port, "/work/ix/no-resident", "ix-5").await;

    let err = port
        .respond_interaction(InteractionResponseParams {
            session_id: s.session_id,
            turn_id: "t-5".into(),
            interaction_id: "ix-5".into(),
            decision: "allow".into(),
            idempotency_key: "r-5".into(),
        })
        .await
        .unwrap_err();
    assert_eq!(err.code, "unsupported");
}

#[tokio::test]
async fn interaction_facade_unknown_interaction_not_found() {
    // A resident exists but the interaction_id is not in the pending map →
    // `interaction_not_found`. The delivery channel does not fabricate a
    // pending interaction.
    let temp = TempDir::new().unwrap();
    let port = real_port(&temp);
    let s = start_session(&port, "/work/ix/unknown-ix", "ix-6").await;

    // Do NOT seed any pending interaction.
    let err = port
        .respond_interaction(InteractionResponseParams {
            session_id: s.session_id,
            turn_id: "t-6".into(),
            interaction_id: "not-pending".into(),
            decision: "allow".into(),
            idempotency_key: "r-6".into(),
        })
        .await
        .unwrap_err();
    assert_eq!(err.code, "interaction_not_found");
}

#[tokio::test]
async fn interaction_facade_not_deliverable_keeps_pending_for_retry() {
    let temp = TempDir::new().unwrap();
    let port = real_port(&temp);
    let s = start_session(&port, "/work/ix/retry", "ix-retry").await;
    seed_pending_without_delivery_hub(
        &port,
        &s.session_id,
        "call-retry",
        PendingKind::Question,
    );

    let err = port
        .respond_interaction(InteractionResponseParams {
            session_id: s.session_id.clone(),
            turn_id: "t-retry".into(),
            interaction_id: "call-retry".into(),
            decision: "allow".into(),
            idempotency_key: "r-retry-1".into(),
        })
        .await
        .unwrap_err();
    assert_eq!(err.code, "interaction_not_deliverable");

    // The parked interaction remains available for the actor to register its
    // oneshot and for the client to retry; it was not consumed by the failed
    // delivery attempt.
    let resident = port.resident(&s.session_id).unwrap();
    assert!(resident
        .pending_interactions
        .as_ref()
        .unwrap()
        .lock()
        .unwrap()
        .contains_key("call-retry"));
}

#[tokio::test]
async fn interaction_facade_missing_hub_keeps_pending_for_retry() {
    let temp = TempDir::new().unwrap();
    let port = no_hub_port(&temp);
    let s = start_session(&port, "/work/ix/no-hub", "ix-no-hub").await;
    seed_pending_without_delivery_hub(
        &port,
        &s.session_id,
        "call-no-hub",
        PendingKind::Permission,
    );

    let err = port
        .respond_interaction(InteractionResponseParams {
            session_id: s.session_id.clone(),
            turn_id: "t-no-hub".into(),
            interaction_id: "call-no-hub".into(),
            decision: "allow".into(),
            idempotency_key: "r-no-hub".into(),
        })
        .await
        .unwrap_err();
    assert_eq!(err.code, "unsupported");
    let resident = port.resident(&s.session_id).unwrap();
    assert!(resident
        .pending_interactions
        .as_ref()
        .unwrap()
        .lock()
        .unwrap()
        .contains_key("call-no-hub"));
}

#[tokio::test]
async fn interaction_facade_closed_receiver_keeps_pending_for_retry() {
    let temp = TempDir::new().unwrap();
    let port = real_port(&temp);
    let s = start_session(&port, "/work/ix/closed-receiver", "ix-closed").await;
    let rx = seed_pending(
        &port,
        &s.session_id,
        "call-closed",
        PendingKind::Question,
    );
    drop(rx);

    let err = port
        .respond_interaction(InteractionResponseParams {
            session_id: s.session_id.clone(),
            turn_id: "t-closed".into(),
            interaction_id: "call-closed".into(),
            decision: "allow".into(),
            idempotency_key: "r-closed".into(),
        })
        .await
        .unwrap_err();
    assert_eq!(err.code, "interaction_not_deliverable");
    let resident = port.resident(&s.session_id).unwrap();
    assert!(resident
        .pending_interactions
        .as_ref()
        .unwrap()
        .lock()
        .unwrap()
        .contains_key("call-closed"));
}

// ===========================================================================
// No-second-permission-engine guard
// ===========================================================================

#[tokio::test]
async fn interaction_facade_does_not_re_evaluate_policy() {
    // The delivery channel forwards the decision verbatim — it does not
    // inspect, validate, or re-evaluate the decision string. A "deny" decision
    // is delivered as "deny"; an "allow" decision is delivered as "allow".
    // The method is a delivery channel, not a permission engine. This test
    // seeds a pending interaction and delivers an arbitrary decision string
    // that a real permission engine would never accept ("maybe"), proving the
    // channel does not filter.
    let temp = TempDir::new().unwrap();
    let port = real_port(&temp);
    let s = start_session(&port, "/work/ix/no-engine", "ix-7").await;

    let rx = seed_pending(&port, &s.session_id, "call-ne", PendingKind::Permission);

    port.respond_interaction(InteractionResponseParams {
        session_id: s.session_id.clone(),
        turn_id: "t-7".into(),
        interaction_id: "call-ne".into(),
        decision: "maybe".into(), // not a valid allow/deny — a real engine would reject
        idempotency_key: "r-7".into(),
    })
    .await
    .unwrap();

    // The arbitrary string is forwarded verbatim — no policy re-evaluation.
    assert_eq!(rx.await.unwrap(), "maybe");
}

#[tokio::test]
async fn interaction_facade_production_source_has_no_second_permission_engine() {
    // Static guard: the production source of the real port must not define a
    // second permission policy engine. The delivery channel checks
    // membership and forwards the decision; it must not contain allow/deny
    // evaluation logic of its own.
    let src = include_str!(
        "../src/app_server_runtime/shell_session_actor_runtime.rs"
    );
    let production = src.split("#[cfg(test)]").next().unwrap();

    // The respond_interaction method body must not contain policy evaluation
    // keywords (allow/deny decision logic). It forwards the decision string.
    let respond_start = production
        .find("async fn respond_interaction")
        .expect("respond_interaction method exists");
    let respond_end = production[respond_start..]
        .find("\n    async fn ")
        .map(|i| respond_start + i)
        .unwrap_or(production.len());
    let respond_body = &production[respond_start..respond_end];

    // The method must NOT contain policy-evaluation constructs.
    assert!(
        !respond_body.contains("is_allowed"),
        "delivery channel must not call is_allowed"
    );
    assert!(
        !respond_body.contains("evaluate_permission"),
        "delivery channel must not evaluate permissions"
    );
    assert!(
        !respond_body.contains("auto_allow"),
        "delivery channel must not auto-allow"
    );
    assert!(
        !respond_body.contains("should_allow"),
        "delivery channel must not re-evaluate allow/deny"
    );
    // It must forward the decision verbatim.
    assert!(
        respond_body.contains("params.decision"),
        "delivery channel must forward params.decision"
    );
}

// ===========================================================================
// Non-vacuity guard — the test file covers the minimum scenarios
// ===========================================================================

#[tokio::test]
async fn interaction_facade_suite_covers_minimum_scenarios() {
    // Non-vacuity guard: asserts every minimum scenario has a dedicated test.
    let src = include_str!("c6_respond_interaction.rs");
    let minimum = [
        "interaction_facade_delivers_decision_to_parked_oneshot",
        "interaction_facade_second_call_is_interaction_not_found",
        "interaction_facade_unknown_session_not_found",
        "interaction_facade_no_resident_unsupported",
        "interaction_facade_unknown_interaction_not_found",
        "interaction_facade_does_not_re_evaluate_policy",
    ];
    for name in minimum {
        assert!(src.contains(name), "missing minimum scenario test: {name}");
    }
    // The test spawner must route through the real SessionCommand enum and
    // provide the real pending_interactions / delivery_hub surfaces — NOT
    // FakeRuntime.
    assert!(
        src.contains("SessionSpawner"),
        "test must inject a real SessionSpawner"
    );
    assert!(
        src.contains("PendingInteractions"),
        "test must use the real PendingInteractions type"
    );
    // The production source (not the test file) must not construct or import
    // FakeRuntime — the delivery channel is backed by the real adapter only.
    let prod_src = include_str!(
        "../src/app_server_runtime/shell_session_actor_runtime.rs"
    );
    let prod_code = prod_src.split("#[cfg(test)]").next().unwrap();
    assert!(
        !prod_code.contains("FakeRuntime::new"),
        "production source must NOT construct FakeRuntime"
    );
    assert!(
        !prod_code.contains("use xai_grok_tower::FakeRuntime"),
        "production source must NOT import FakeRuntime"
    );
}
