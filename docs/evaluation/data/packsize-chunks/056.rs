pub fn all_integrations() -> Vec<Box<dyn AgentIntegration>> {
    // The hosts that do more than write one JSON entry, then the six that do
    // exactly that, from `mcp_host::HOSTS` (#443). Order is what `omni doctor`
    // prints, so the ones with hooks come first.
    let mut all: Vec<Box<dyn AgentIntegration>> = vec![
        Box::new(claude::ClaudeIntegration),
        Box::new(cursor::CursorIntegration),
        Box::new(cline::ClineIntegration),
        Box::new(gemini::GeminiIntegration),
        Box::new(codex::CodexIntegration),
        Box::new(openclaw::OpenClawIntegration),
        Box::new(hermes::HermesIntegration),
        Box::new(pi::PiIntegration),
    ];
    all.extend(
        mcp_host::HOSTS
            .iter()
            .map(|h| Box::new(h) as Box<dyn AgentIntegration>)
            .collect::<Vec<_>>(),
    );
    all
}

#[cfg(test)]
mod fix_reporting_tests {
    /// #454's silent half: the state existed and every surface said [OK].
    #[test]
    fn counts_the_hooks_an_event_would_run() {
        let val = serde_json::json!({"hooks": {
            "PreToolUse": [
                {"hooks": [{"command": "/a/omni --pre-hook"}]},
                {"hooks": [{"command": "/b/omni --pre-hook"}]}
            ],
            "SessionStart": [{"hooks": [{"command": "/b/omni --session-start"}]}],
            "OtherTool": [{"hooks": [{"command": "/usr/bin/prettier --write"}]}]
        }});

        assert_eq!(
            super::duplicate_omni_hooks(&val),
            vec![("PreToolUse".to_string(), 2)],
            "only the doubled event is reported, and someone else's hook is not ours"
        );
    }

    use super::report_fix;

    /// #386: every site did `let _ = self.install(...)` and printed `[FIXED]`
    /// unconditionally. Against an unwritable home, `doctor --json --fix`
    /// answered `healthy: true`, `hooks installed` and zero warnings while
    /// nothing had been written, because in fix mode the `[FIXED]` line replaced
    /// the warning that would otherwise have been pushed.
    #[test]
    fn a_failed_repair_is_reported_and_warned_about() {
        let mut warnings = Vec::new();

        let (ok, printed) = crate::agents::output::capture(|| {
            report_fix(
                "Hooks:",
                "missing hooks installed",
                Err(anyhow::anyhow!("Permission denied (os error 13)")),
              