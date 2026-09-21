fn strip_omni_server(config: &str) -> String {
    let mut out = String::with_capacity(config.len());
    let mut skip = false;
    for line in config.lines() {
        let t = line.trim();
        if t.starts_with('[') {
            skip = t == "[mcp_servers.omni]" || t.starts_with("[mcp_servers.omni.");
        }
        if !skip {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

/// Codex reads its config from `$CODEX_HOME` and only falls back to `~/.codex`.
/// Installing to the fallback while the host is using the override writes a
/// config Codex never loads, and `doctor` then reports the file it wrote rather
/// than the file in use. Seen on a machine where a launcher exported
/// `CODEX_HOME` to its own runtime directory: `~/.codex/config.toml` held the
/// MCP server, the live home did not, and OMNI reported healthy either way.
/// The OMNI hook events Codex has no trust record for, and will therefore skip.
///
/// Codex 0.144.6 trusts hooks one entry at a time, keyed
/// `<hooks.json path>:<event>:<group>:<index>` under `[hooks.state]`, with the
/// entry's own hash. An entry it has not been shown is ignored, and `codex exec`
/// prints nothing about it, so a config that reads as installed does nothing at
/// all. Proved by deleting one trust record: the hook stopped firing, and came
/// back when the record was restored (#359).
///
/// OMNI deliberately does not write these records. The hash exists so a human
/// approves each command before Codex runs it, and a tool that grants itself
/// that approval removes the only thing standing between "can write a config
/// file" and "can execute anything".