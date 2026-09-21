fn register_plugin_path(config: &mut Value, dir: &str) -> bool {
    let Some(obj) = config.as_object_mut() else {
        return false;
    };
    let paths = obj
        .entry("plugins")
        .or_insert_with(|| json!({}))
        .as_object_mut()
        .and_then(|plugins| {
            plugins
                .entry("load")
                .or_insert_with(|| json!({}))
                .as_object_mut()
        })
        .map(|load| load.entry("paths").or_insert_with(|| json!([])))
        .and_then(|paths| paths.as_array_mut());

    let Some(paths) = paths else {
        return false;
    };
    if paths.iter().any(|p| p.as_str() == Some(dir)) {
        return false;
    }
    paths.push(json!(dir));
    true
}

impl AgentIntegration for OpenClawIntegration {
    /// `tool_result_persist` replaces the result of every built-in tool, which
    /// is the whole point of the plugin #628 shipped. It rewrites the message
    /// OpenClaw persists rather than the one already in flight, so the model
    /// reads OMNI's bytes on every turn that re-reads the transcript and the
    /// current turn still sees the raw output. Narrower than Claude Code's
    /// `Full`, and nowhere near the default `McpOnly` this inherited for two
    /// releases while `doctor` told the user there was no shell distill (#687).
    fn tier(&self) -> crate::agents::Tier {
        crate::agents::Tier::Full
    }

    fn id(&self) -> &'static str {
        "openclaw"
    }

    fn name(&self) -> &'static str {
        "OpenClaw"
    }

    fn install(&self, _exe_path: &str) -> anyhow::Result<()> {
        let dest = plugin_dir();
        fs::create_dir_all(&dest)?;

        crate::agent_report!(
            "  {} Downloading OpenClaw plugin files from GitHub...",
            "↓".cyan()
        );

        // Download key files
        for file in &[
            "openclaw.plugin.json",
            "index.ts",
            "package.json",
            "runtime-api.ts",
            "tsconfig.json",
        ] {
            let url = format!(
                "https://raw.githubusercontent.com/fajarhide/omni/main/plugins/openclaw/{}",
                file
            );
            let to = dest.join(file);

            let response = ureq::get(&url)
                .call()
                .map_err(|e| anyhow::anyhow!("Failed to download {}: {}", file, e))?;
            let mut dest_file =