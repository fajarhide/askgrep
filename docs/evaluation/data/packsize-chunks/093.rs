fn unreleased_entries() -> usize {
    option_env!("OMNI_UNRELEASED_ENTRIES")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

/// One line for the MCP section, phrased so a missing tool is a setting rather
/// than a mystery.
///
/// It names the id it resolved because the caller passes whatever
/// `detect_agent_id` returned, which in a plain terminal is `terminal` and not
/// the host the reader is thinking of. `keep_everything` comes from
/// `policy::override_is_on`, the same read the served router makes, so the line
/// cannot offer the override as a remedy while the override is already on.
pub(crate) fn mcp_tool_line(agent_id: &str, keep_everything: bool) -> String {
    let total = crate::mcp::policy::ALL_TOOL_COUNT;
    let (active, note) = if keep_everything {
        (total, "OMNI_MCP_TOOLS=all is set")
    } else {
        (
            crate::mcp::policy::active_tools(agent_id).len(),
            "OMNI_MCP_TOOLS=all restores the rest",
        )
    };
    format!("  MCP tools:      {active} of {total} advertised to {agent_id} ({note})")
}
