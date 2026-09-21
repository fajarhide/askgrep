fn has_valid_omni_server(val: &Value) -> bool {
    val.get("mcpServers")
        .and_then(|v| v.as_object())
        .and_then(|servers| servers.get("omni"))
        .is_some_and(|omni| {
            omni.get("command").and_then(|v| v.as_str()).is_some()
                && omni
                    .get("args")
                    .and_then(|v| v.as_array())
                    .is_some_and(|args| args.iter().any(|arg| arg.as_str() == Some("--mcp")))
        })
}
