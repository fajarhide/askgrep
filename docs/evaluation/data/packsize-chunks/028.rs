fn remove_mcp_server(val: &mut Value) {
    if let Some(obj) = val.as_object_mut()
        && let Some(servers) = obj.get_mut("mcpServers").and_then(|v| v.as_object_mut())
    {
        servers.remove("omni");
    }
}
