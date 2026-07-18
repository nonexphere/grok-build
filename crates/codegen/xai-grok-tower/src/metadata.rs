//! Session metadata index entries (not transcript copies).

use xai_grok_app_server_protocol::{ProviderBinding, SessionStatus, WireCounter};

/// Points at canonical session files; never embeds full transcripts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionMetadata {
    pub session_id: String,
    pub history_epoch: String,
    pub revision: WireCounter,
    pub status: SessionStatus,
    pub workspace_root: String,
    pub canonical_session_path: String,
    pub title: Option<String>,
    pub active_turn_id: Option<String>,
    pub latest_turn_id: Option<String>,
    pub provider_binding: Option<ProviderBinding>,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
    pub residency: Residency,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Residency {
    Resident,
    Dormant,
    Archived,
}

#[cfg(test)]
mod session_metadata_tests {
    use super::*;

    #[test]
    fn session_metadata_points_to_path_not_transcript() {
        let meta = SessionMetadata {
            session_id: "session_1".into(),
            history_epoch: "epoch_1".into(),
            revision: 1.into(),
            status: SessionStatus::Ready,
            workspace_root: "/work".into(),
            canonical_session_path: "/home/u/.grok-oss/sessions/session_1.jsonl".into(),
            title: None,
            active_turn_id: None,
            latest_turn_id: None,
            provider_binding: None,
            created_at_ms: 1,
            updated_at_ms: 1,
            residency: Residency::Dormant,
        };
        assert!(meta.canonical_session_path.ends_with("session_1.jsonl"));
        // Metadata struct fields are identifiers/paths only — no messages field.
        let debug = format!("{meta:?}");
        assert!(!debug.contains("transcript"));
        assert!(!debug.contains("messages"));
    }
}
