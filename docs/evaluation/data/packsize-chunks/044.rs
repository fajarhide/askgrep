fn configured_omni_plugin(config_path: &Path) -> Option<&'static str> {
    fs::read_to_string(config_path)
        .ok()
        .and_then(|config| config_mentions_omni_plugin(&config))
}

/// Adds a top-level YAML block without disturbing what is already there.
///
/// The previous version spliced the block in directly after the `plugins:` line.
/// `mcp_servers:` is itself a top-level key, so that ended the `plugins` mapping
/// and every plugin entry underneath became a child of `mcp_servers`. Loaded
/// with a real YAML parser, a config with two enabled plugins came back as
/// `plugins: None` and `mcp_servers: [omni, my-linter, my-formatter]`: the
/// installer silently disabled every plugin the user had (#377).
///
/// Top-level keys are order-independent, so appending is both correct and the
/// only placement that cannot capture someone else's entries.