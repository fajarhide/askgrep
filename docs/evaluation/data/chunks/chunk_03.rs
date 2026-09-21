pub fn initialize_settings() -> anyhow::Result<(PathBuf, Value)> {
    let path = get_settings_path();

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut val = if path.exists() {
        let content = fs::read_to_string(&path)?;
        serde_json::from_str(&content).unwrap_or_else(|_| json!({}))
    } else {
        json!({})
    };

    install_omni_hooks(&mut val, ""); // Temp to ensure object exists
    Ok((path, val))
}
