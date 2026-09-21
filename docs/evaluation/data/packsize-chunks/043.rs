fn config_mentions_omni_mcp(config: &str) -> bool {
    let has_mcp_section = config.contains("mcp_servers:") || config.contains("mcp:");
    let has_omni_server = config.contains("omni:");
    let has_omni_command = config.contains("--mcp") || config.contains("OMNI_AGENT_ID");
    has_mcp_section && has_omni_server && has_omni_command
}
