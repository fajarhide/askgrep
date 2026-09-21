pub fn install_mcp_server(exe_path: &str) -> anyhow::Result<()> {
    let path = get_claude_json_path();
    let mut val = if path.exists() {
        let content = fs::read_to_string(&path)?;
        serde_json::from_str(&content).unwrap_or_else(|_| json!({}))
    } else {
        json!({})
    };

    let obj = val
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("Invalid .claude.json format"))?;

    let servers = obj
        .entry("mcpServers")
        .or_insert_with(|| json!({}))
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("mcpServers is not an object"))?;

    servers.insert(
        "omni".to_string(),
        json!({
            "type": "stdio",
            "command": exe_path,
            "args": ["--mcp"],
            "env": {
                "OMNI_AGENT_ID": "claude_code"
            },
        }),
    );

    if let Some(projects) = obj.get_mut("projects").and_then(|p| p.as_object_mut()) {
        for (_path, p_val) in projects.iter_mut() {
            if let Some(ps) = p_val.get_mut("mcpServers").and_then(|s| s.as_object_mut())
                && ps.contains_key("omni")
            {
                ps.insert(
                    "omni".to_string(),
                    json!({
                        "type": "stdio",
                        "command": exe_path,
                        "args": ["--mcp"],
                        "env": {
                            "OMNI_AGENT_ID": "claude_code"
                        },
                    }),
                );
            }
        }
    }

    let top_level_keys: Vec<String> = obj.keys().cloned().collect();
    for key in top_level_keys {
        if key != "mcpServers"
            && key != "projects"
            && let Some(inner_obj) = obj.get_mut(&key).and_then(|v| v.as_object_mut())
            && let Some(ps) = inner_obj
                .get_mut("mcpServers")
                .and_then(|s| s.as_object_mut())
            && ps.contains_key("omni")
        {
            ps.insert(
                "omni".to_string(),
                json!({
                    "type": "stdio",
                    "command": exe_path,
                    "args": ["--mcp"],
                    "env": {
                        "OMNI_AGENT_ID": "claude_code"
                    },
                }),
            );
        }
    }

    fs::write(&path, serde_json::to_string_pr