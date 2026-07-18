//! Bounded history replay helpers over facade `ReplayPage` results.

use xai_grok_app_server_protocol::{SubscribeParams, WireCounter};
use xai_grok_tower::{GrokRuntimeFacade, ReplayPage, RuntimeError};

/// Fetch successive replay pages until `next_cursor` is exhausted.
pub async fn replay_all_pages(
    runtime: &dyn GrokRuntimeFacade,
    session_id: &str,
    history_epoch: Option<String>,
    page_limit: usize,
) -> Result<Vec<ReplayPage>, RuntimeError> {
    let mut pages = Vec::new();
    let mut after = WireCounter::new(0);
    for _ in 0..page_limit {
        let page = runtime
            .replay(SubscribeParams {
                session_id: session_id.into(),
                after_event_seq: after.clone(),
                history_epoch: history_epoch.clone(),
            })
            .await?;
        let next = page.next_cursor.clone();
        let through = page.replayed_through.clone();
        pages.push(page);
        match next {
            Some(c) if c.as_u64() > after.as_u64() => {
                after = c;
            }
            Some(_) => break,
            None => break,
        }
        let _ = through;
    }
    Ok(pages)
}

#[cfg(test)]
mod cursor_semantics_tests {
    use super::*;
    use std::sync::Arc;
    use xai_grok_app_server_protocol::{
        InputBlock, SessionStartParams, TurnStartParams,
    };
    use xai_grok_tower::FakeRuntime;

    #[tokio::test]
    async fn cursor_semantics_epoch_mismatch_and_bounded_pages() {
        let rt = FakeRuntime::new();
        let session = rt
            .start_session(SessionStartParams {
                workspace_root: "/work".into(),
                agent_type: None,
                provider_binding: None,
                idempotency_key: "rp-1".into(),
            })
            .await
            .unwrap();
        for i in 0..3 {
            rt.start_turn(TurnStartParams {
                session_id: session.session_id.clone(),
                input: vec![InputBlock::Text {
                    text: format!("m{i}"),
                }],
                idempotency_key: format!("rp-t{i}"),
            })
            .await
            .unwrap();
        }
        let pages = replay_all_pages(&rt, &session.session_id, Some("epoch_1".into()), 5)
            .await
            .unwrap();
        assert!(!pages.is_empty());
        assert!(pages[0].replayed_through.as_u64() >= 1);

        let bad = rt
            .replay(SubscribeParams {
                session_id: session.session_id.clone(),
                after_event_seq: WireCounter::new(0),
                history_epoch: Some("epoch_stale".into()),
            })
            .await;
        assert_eq!(bad.unwrap_err().code, "epoch_mismatch");
    }

    #[tokio::test]
    async fn projection_rebuild_via_replay_is_stable_for_fake() {
        let rt = Arc::new(FakeRuntime::new());
        let session = rt
            .start_session(SessionStartParams {
                workspace_root: "/work".into(),
                agent_type: None,
                provider_binding: None,
                idempotency_key: "rb-1".into(),
            })
            .await
            .unwrap();
        let a = replay_all_pages(rt.as_ref(), &session.session_id, None, 3)
            .await
            .unwrap();
        let b = replay_all_pages(rt.as_ref(), &session.session_id, None, 3)
            .await
            .unwrap();
        assert_eq!(a.len(), b.len());
        assert_eq!(
            a[0].replayed_through.as_u64(),
            b[0].replayed_through.as_u64()
        );
    }
}
