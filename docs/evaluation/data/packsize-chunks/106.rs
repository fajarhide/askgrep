fn init_id_for_agent(agent_id: &str) -> Option<&'static str> {
    match agent_id {
        "claude_code" => Some("claude"),
        "cursor" => Some("cursor"),
        "cline" => Some("cline"),
        "codex_cli" => Some("codex"),
        "antigravity" => Some("antigravity"),
        "vscode" => Some("vscode"),
        _ => None,
    }
}

/// Which host to configure when there is no terminal to ask on.
///
/// `omni init` is the line the README, the Homebrew caption, the landing page and
/// `install.sh` all print, and the audience for this tool is agents, which run it
/// with no tty. `dialoguer` can only fail there, so the one documented setup
/// command exited 1 on `IO error: not a terminal` and named no remedy (#528).
///
/// The host running the command is the host to configure, and OMNI already reads
/// that from the environment on the exec and pipe paths. Anything it cannot name
/// is an error that lists the flags, rather than a guess: installing into a host
/// nobody asked for is the worse failure of the two.