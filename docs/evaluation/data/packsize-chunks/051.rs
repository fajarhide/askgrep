fn render_plugin(exe_path: &str) -> String {
    let literal = serde_json::to_string(exe_path).unwrap_or_else(|_| "\"omni\"".to_string());
    include_str!("../../plugins/hermes/__init__.py").replace("\"{{OMNI_BIN}}\"", &literal)
}

impl AgentIntegration for HermesIntegration {
    /// `transform_tool_result` fires for every tool, not only the terminal, so
    /// this is the host with the widest reach of any: it is what finally runs
    /// the Read, Grep and WebFetch distillers that Claude Code's Bash-only
    /// matcher never reaches (#172). #628 wired it and left the tier at the
    /// default, so `doctor` reported "no shell distill" for it (#687).
    fn tier(&self) -> crate::agents::Tier {
        crate::agents::Tier::Full
    }

    fn id(&self) -> &'static str {
        "hermes"
    }

    fn name(&self) -> &'static str {
        "Hermes Agent"
    }

    fn install(&self, exe_path: &str) -> anyhow::Result<()> {
        let mut actions = Vec::new();
        let mut warnings = Vec::new();

        let dest = plugin_dir();
        fs::create_dir_all(&dest)?;

        let plugin_yaml_content = r#"name: omni-signal-engine
version: "1.0"
description: OMNI Signal Engine integration for Hermes Agent hooks
"#;

        // The plugin is a real Python file in `plugins/hermes/`, compiled by
        // CI, rather than a Rust string literal. The literal that used to live
        // here opened with five quote characters, so every `__init__.py` OMNI
        // wrote was a syntax error and the plugin never loaded once (#628).
        let init_py_content = render_plugin(exe_path);

        fs::write(dest.join("plugin.yaml"), plugin_yaml_content)?;
        fs::write(dest.join("__init__.py"), init_py_content)?;
        actions.push(format!(
            "{} Installed Hermes plugin to ~/.hermes/plugins/omni-signal-engine/",
            "✓".green()
        ));

        let config_path = hermes_config_path();
        let requires_manual_plugin_step = !fs::metadata(&config_path)
            .ok()
            .map(|meta| meta.is_file())
            .unwrap_or(false);

        if requires_manual_plugin_step {
            actions.push(format!(
                "{} Run {} to enable the OMNI plugin",
                "→".cyan(),
                "hermes plugins enable omni-signal-engine".bright_black()
            ));
            warnings.push(
                "Hermes config not