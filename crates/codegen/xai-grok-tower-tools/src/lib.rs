//! Tool descriptors shared by in-process and MCP adapters. Semantics/ACL are
//! implemented only by `50-tower-agent-tools/v1-01..02` over the shared facade.

pub const TOWER_TOOL_NAMES: [&str; 9] = [
    "tower_agent_list",
    "tower_agent_start",
    "tower_agent_send",
    "tower_agent_history",
    "tower_agent_resume",
    "tower_agent_wait",
    "tower_agent_interrupt",
    "tower_agent_archive",
    "tower_agent_status",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TowerToolDescriptor {
    pub name: &'static str,
    pub description: &'static str,
    pub input_schema_ref: &'static str,
    pub output_schema_ref: &'static str,
}

pub const TOWER_TOOL_DESCRIPTORS: [TowerToolDescriptor; 9] = [
    descriptor(
        "tower_agent_list",
        "List Tower-managed Sessions with filters and pagination.",
        "tower-tools.schema.json#/$defs/tower_agent_list_input",
        "tower-tools.schema.json#/$defs/tower_agent_list_output",
    ),
    descriptor(
        "tower_agent_start",
        "Start a top-level Session in a validated workspace.",
        "tower-tools.schema.json#/$defs/tower_agent_start_input",
        "tower-tools.schema.json#/$defs/tower_agent_start_output",
    ),
    descriptor(
        "tower_agent_send",
        "Start a Turn or steer the named active Turn.",
        "tower-tools.schema.json#/$defs/tower_agent_send_input",
        "tower-tools.schema.json#/$defs/tower_agent_send_output",
    ),
    descriptor(
        "tower_agent_history",
        "Read redacted full or last Session history within byte limits.",
        "tower-tools.schema.json#/$defs/tower_agent_history_input",
        "tower-tools.schema.json#/$defs/tower_agent_history_output",
    ),
    descriptor(
        "tower_agent_resume",
        "Make a dormant Session resident without changing identity.",
        "tower-tools.schema.json#/$defs/tower_agent_resume_input",
        "tower-tools.schema.json#/$defs/tower_agent_resume_output",
    ),
    descriptor(
        "tower_agent_wait",
        "Wait after an event cursor without holding runtime locks.",
        "tower-tools.schema.json#/$defs/tower_agent_wait_input",
        "tower-tools.schema.json#/$defs/tower_agent_wait_output",
    ),
    descriptor(
        "tower_agent_interrupt",
        "Idempotently interrupt the named active Turn.",
        "tower-tools.schema.json#/$defs/tower_agent_interrupt_input",
        "tower-tools.schema.json#/$defs/tower_agent_interrupt_output",
    ),
    descriptor(
        "tower_agent_archive",
        "Archive a Session without deleting its transcript.",
        "tower-tools.schema.json#/$defs/tower_agent_archive_input",
        "tower-tools.schema.json#/$defs/tower_agent_archive_output",
    ),
    descriptor(
        "tower_agent_status",
        "Read a redacted Session status and residency summary.",
        "tower-tools.schema.json#/$defs/tower_agent_status_input",
        "tower-tools.schema.json#/$defs/tower_agent_status_output",
    ),
];

const fn descriptor(
    name: &'static str,
    description: &'static str,
    input_schema_ref: &'static str,
    output_schema_ref: &'static str,
) -> TowerToolDescriptor {
    TowerToolDescriptor {
        name,
        description,
        input_schema_ref,
        output_schema_ref,
    }
}

pub fn is_authorized(agent_type: &str, explicit_opt_in: bool) -> bool {
    agent_type == "orchestrator" || explicit_opt_in
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn acl_is_fail_closed_by_default() {
        assert!(!is_authorized("build", false));
        assert!(is_authorized("orchestrator", false));
    }
    #[test]
    fn contract_has_exactly_nine_unique_tools() {
        let mut names = TOWER_TOOL_NAMES.to_vec();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), 9);
        assert_eq!(
            TOWER_TOOL_DESCRIPTORS.map(|descriptor| descriptor.name),
            TOWER_TOOL_NAMES
        );
    }

    #[test]
    fn every_descriptor_resolves_exact_input_and_output_definition() {
        let schema: serde_json::Value = serde_json::from_str(include_str!(
            "../../xai-grok-app-server-protocol/schemas/tower-tools.schema.json"
        ))
        .unwrap();
        let definitions = schema["$defs"].as_object().unwrap();
        for descriptor in TOWER_TOOL_DESCRIPTORS {
            assert_eq!(
                descriptor.input_schema_ref,
                format!("tower-tools.schema.json#/$defs/{}_input", descriptor.name)
            );
            assert_eq!(
                descriptor.output_schema_ref,
                format!("tower-tools.schema.json#/$defs/{}_output", descriptor.name)
            );
            assert!(definitions.contains_key(&format!("{}_input", descriptor.name)));
            assert!(definitions.contains_key(&format!("{}_output", descriptor.name)));
        }
    }
}
