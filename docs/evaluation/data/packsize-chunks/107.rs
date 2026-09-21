fn non_interactive_host() -> anyhow::Result<&'static str> {
    let agent = crate::agents::multiagent::detect_agent_id();
    init_id_for_agent(&agent).ok_or_else(|| {
        anyhow::anyhow!(
            "no terminal to prompt on, and the host here reads as `{agent}`. \
             Name one instead: `omni init --claude`, every host with \
             `omni init --all`, or see them all with `omni init --help`."
        )
    })
}

/// Which halves of the Claude Code install a flag set asks for, as (hooks, mcp).
///
/// #757. `--hook` has said "Only install hooks" since it shipped and installed
/// the MCP server as well, which is the shape of #151: a flag the parser accepts
/// and the code ignores. On this host the hooks are what shortens output and the
/// MCP server is a convenience whose tool definitions sit in the prefix of every
/// request, so one without the other is a preference somebody can hold.
///
/// Asking for both, by naming the host or by naming both halves, gets both.