fn initialize_mcp_config() -> anyhow::Result<(PathBuf, Value)> {
    let mcp_path = get_mcp_path();
    if let Some(parent) = mcp_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let val = if mcp_path.exists() {
        let content = fs::read_to_string(&mcp_path)?;
        serde_json::from_str(&content).unwrap_or_else(|_| json!({}))
    } else {
        json!({})
    };
    Ok((mcp_path, val))
}
