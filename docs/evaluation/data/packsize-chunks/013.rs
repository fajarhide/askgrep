fn hooks_awaiting_review(config: &str, hooks_path: &str) -> Vec<&'static str> {
    [
        ("pre_tool_use", "PreToolUse"),
        ("post_tool_use", "PostToolUse"),
        ("session_start", "SessionStart"),
    ]
    .into_iter()
    // TOML escapes backslashes inside a basic string, so a Windows key reads
    // `C:\\Users\\...` while `hooks_path` has single ones and the match never
    // fired: doctor reported all three hooks awaiting review on Windows even
    // when they were approved, and failed. The test used a POSIX literal, so CI
    // passed on windows-latest.
    .filter(|(wire, _)| {
        let escaped = hooks_path.replace('\\', "\\\\");
        !config.contains(&format!("{}:{}:", escaped, wire))
            && !config.contains(&format!("{}:{}:", hooks_path, wire))
    })
    .map(|(_, display)| display)
    .collect()
}
