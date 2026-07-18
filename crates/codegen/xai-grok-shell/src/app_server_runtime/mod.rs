//! App Server runtime adapter seam.
//!
//! Composition root injects a type implementing [`xai_grok_tower::GrokRuntimeFacade`]
//! that forwards to the existing leader/`SessionActor` path. This module owns the
//! Shell-side boundary only — Tower never imports Shell.

/// Marker documenting that production adapters live in Shell and are injected
/// at `xai-grok-pager-bin`. Unit tests characterize the ownership claim.
pub struct ShellRuntimeAdapterMarker;

impl ShellRuntimeAdapterMarker {
    pub const OWNER: &'static str = "xai-grok-shell";
    pub const INJECTED_AT: &'static str = "xai-grok-pager-bin";
}

#[cfg(test)]
mod app_server_runtime_tests {
    use super::*;

    #[test]
    fn app_server_runtime_adapter_lives_in_shell_not_tower() {
        assert_eq!(ShellRuntimeAdapterMarker::OWNER, "xai-grok-shell");
        assert_eq!(ShellRuntimeAdapterMarker::INJECTED_AT, "xai-grok-pager-bin");
        let tower_cargo = include_str!("../../../xai-grok-tower/Cargo.toml");
        assert!(
            !tower_cargo.contains("xai-grok-shell"),
            "Tower must not depend on Shell"
        );
    }

    #[test]
    fn app_server_runtime_defines_no_session_actor_state_machine() {
        let src = include_str!("mod.rs");
        let production = src.split("#[cfg(test)]").next().unwrap();
        assert!(!production.contains("struct SessionActor"));
        assert!(!production.contains("enum SessionActor"));
    }
}
