pub fn detect_agent_id() -> String {
    // Explicit override (set by agent-specific wrapper scripts)
    if let Ok(id) = std::env::var("OMNI_AGENT_ID") {
        return id;
    }
    // Claude Code first: it sets `TERM_PROGRAM`/`VSCODE_PID` too when run from
    // VS Code's integrated terminal, so checking it after the VSCode branch
    // below would label it `vscode`, trading a vague answer for a confidently
    // wrong one. Same reasoning as the Antigravity-before-VSCode ordering.
    if std::env::var("CLAUDECODE").is_ok() || std::env::var("CLAUDE_CODE_ENTRYPOINT").is_ok() {
        return "claude_code".to_string();
    }
    // Auto-detect from known env variables each agent sets
    if std::env::var("CURSOR_TRACE_ID").is_ok() || std::env::var("CURSOR_SESSION_ID").is_ok() {
        return "cursor".to_string();
    }
    if std::env::var("CLINE_TASK_ID").is_ok() {
        return "cline".to_string();
    }
    if std::env::var("CODEX_SESSION").is_ok() {
        return "codex_cli".to_string();
    }
    if std::env::var("WINDSURF_SESSION").is_ok() {
        return "windsurf".to_string();
    }
    if std::env::var("CONTINUE_SESSION_ID").is_ok() {
        return "vscode_continue".to_string();
    }
    if std::env::var("AIDER_SESSION").is_ok() {
        return "aider".to_string();
    }
    // Antigravity IDE detection (must be before VSCODE_PID: Antigravity is a VSCode fork)
    if std::env::var("ANTIGRAVITY_EDITOR_APP_ROOT").is_ok()
        || std::env::var("ANTIGRAVITY_SESSION").is_ok()
        || std::env::var("__CFBundleIdentifier")
            .map(|v| v.contains("antigravity"))
            .unwrap_or(false)
        || std::env::current_exe()
            .map(|p| p.to_string_lossy().contains("Antigravity"))
            .unwrap_or(false)
    {
        return "antigravity".to_string();
    }
    // VSCode Copilot detection
    if std::env::var("VSCODE_PID").is_ok()
        || std::env::var("TERM_PROGRAM")
            .map(|v| v == "vscode")
            .unwrap_or(false)
    {
        return "vscode".to_string();
    }
    // Default: terminal (generic shell usage)
    "terminal".to_string()
}

/// Sync current session state to the shared agent_sessions table