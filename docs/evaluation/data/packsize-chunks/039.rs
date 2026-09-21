fn omni_config_path() -> PathBuf {
    crate::paths::config_file()
}

/// Comprehensive startup validation for Hermes integration.
///
/// Checks: config.yaml (MCP + compression), plugin files, OMNI binary
/// availability, and OMNI config presence. Returns `None` when all
/// checks pass, or a formatted diagnostics string that gets injected
/// into the Hermes session-start context so the agent can self-heal.