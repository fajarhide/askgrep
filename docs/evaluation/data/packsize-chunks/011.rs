fn declares_transport(config: &str) -> bool {
    let mut in_parent = false;
    for line in config.lines() {
        let t = line.trim();
        if t.starts_with('[') {
            in_parent = t == "[mcp_servers.omni]";
            continue;
        }
        if in_parent && t.starts_with("command") {
            return true;
        }
    }
    false
}

/// Removes `[mcp_servers.omni]` **and every `[mcp_servers.omni.*]` sub-table**.
///
/// The old version stopped skipping at the next `[`, so a sub-table survived its
/// own parent. Codex then read a server with no transport and refused to start,
/// which is a working install turned into a dead one by running uninstall (#351).