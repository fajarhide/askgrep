fn collect_omni_sources_recursive(val: &Value, out: &mut Vec<String>) {
    match val {
        Value::String(s) if s.to_lowercase().contains("omni") => {
            out.push(s.clone());
        }
        Value::Array(arr) => {
            for v in arr {
                collect_omni_sources_recursive(v, out);
            }
        }
        Value::Object(map) => {
            for v in map.values() {
                collect_omni_sources_recursive(v, out);
            }
        }
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// PiIntegration
// ---------------------------------------------------------------------------

pub struct PiIntegration;

impl AgentIntegration for PiIntegration {
    /// Pi's `tool_result` handler returns replacement content, so the model
    /// reads distilled bytes on the turn it is on. It read the wrong key until
    /// #688 and applied nothing, which is why the default outlived the hook.
    /// Derived from the plugin by the test in `mod.rs` now, not remembered.
    fn tier(&self) -> crate::agents::Tier {
        crate::agents::Tier::Full
    }

    fn id(&self) -> &'static str {
        "pi"
    }

    fn name(&self) -> &'static str {
        "Pi Agent"
    }

    fn install(&self, _exe_path: &str) -> anyhow::Result<()> {
        let source = package_source();
        run_install(&source)
    }

    fn uninstall(&self) -> anyhow::Result<()> {
        // Pi does not expose a reliable package-removal command.
        // Print actionable manual cleanup instructions instead of editing
        // Pi settings blindly.
        crate::agent_report!(
            "  {} Pi does not expose a package-removal command.",
            "ℹ".blue()
        );
        crate::agent_report!("  {} To remove the OMNI Pi package manually:", "→".cyan());
        crate::agent_report!(
            "    1. Edit {} and remove the OMNI package entry.",
            pi_settings_path().display()
        );
        crate::agent_report!("    2. Restart Pi to apply the changes.\n");
        Ok(())
    }

    fn doctor_check(&self, fix_mode: bool, warnings: &mut Vec<String>) -> bool {
        crate::agent_report!("\n  {}", "Pi:".cyan());

        // 1. Pi binary
        if find_pi_binary().is_none() {
            crate::agent_report!(
                "   {:<15} {}",
                "Config:".bright_black(),
