fn configured_omni_mcp(config_path: &Path) -> bool {
    fs::read_to_string(config_path)
        .map(|config| config_mentions_omni_mcp(&config))
        .unwrap_or(false)
}
