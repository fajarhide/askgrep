fn install_mcp_server(val: &mut Value, exe_path: &str) {
    let obj = match val.as_object_mut() {
        Some(o) => o,
        None => {
            *val = json!({});
            val.as_object_mut().unwrap()
        }
    };
    let servers = obj
        .entry("mcpServers")
        .or_insert_with(|| json!({}))
        .as_object_mut()
        .unwrap();
    servers.insert(
        "omni".to_string(),
        json!({
            "type": "stdio", "command": exe_path, "args": ["--mcp"],
            "env": { "OMNI_AGENT_ID": "cursor" }
        }),
    );
}
