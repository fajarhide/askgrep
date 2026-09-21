pub fn validate_startup() -> Option<String> {
    let mut warnings: Vec<&str> = Vec::new();

    // ── 1. Hermes config.yaml ──
    let config_path = hermes_home_dir().join("config.yaml");
    if let Ok(config_str) = fs::read_to_string(&config_path) {
        // MCP server registration
        if !config_str.contains("mcp_servers:") || !config_str.contains("omni:") {
            warnings.push(
                "OMNI MCP server is NOT registered in ~/.hermes/config.yaml. \
                 27 MCP tools (omni_retrieve, omni_loop_memory, omni_knowledge, …) \
                 will be unavailable. Run `omni init --hermes` to fix.",
            );
        }
        // Compression bridge
        if !config_str.contains("compression:") || !config_str.contains("enabled: true") {
            warnings.push(
                "Hermes compression is NOT enabled. Context Pressure warnings \
                 from OMNI will be surfaced but Hermes will not act on them. \
                 Run `omni init --hermes` to fix.",
            );
        }
    } else {
        warnings.push("Could not find ~/.hermes/config.yaml. Is Hermes installed?");
    }

    // ── 2. Plugin scaffold ──
    let plugin_init = plugin_dir().join("__init__.py");
    if !plugin_init.exists() {
        warnings.push(
            "OMNI Hermes plugin (`__init__.py`) is missing. \
             Pre/Post hooks will not execute. Run `omni init --hermes` to install.",
        );
    }

    // ── 3. OMNI binary reachable ──
    #[allow(clippy::collapsible_if)]
    if let Ok(exe) = std::env::current_exe() {
        if !exe.exists() {
            warnings.push("OMNI binary path does not exist on disk. Hooks will fail at runtime.");
        }
    }

    // ── 4. OMNI config for Hermes ──
    let omni_cfg = omni_config_path();
    if omni_cfg.exists() {
        #[allow(clippy::collapsible_if)]
        if let Ok(content) = fs::read_to_string(&omni_cfg) {
            if !content.contains("[agents.hermes]") {
                warnings.push(
                    "~/.omni/config.toml exists but has no [agents.hermes] section. \
                     Hermes-optimized defaults (Efficient mode, pinned files) are inactive. \
                     Run `omni init --hermes` to add them.",
                );
            }
        }
    } else {
        warnings.push(
            "~/.omni/config.toml does not exist. OMNI is using built