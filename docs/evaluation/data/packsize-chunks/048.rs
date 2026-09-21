fn strip_toml_section(config: &str, header: &str) -> String {
    let lines: Vec<&str> = config.lines().collect();
    let Some(start) = lines.iter().position(|l| l.trim_end() == header) else {
        return config.to_string();
    };
    let end = lines
        .iter()
        .enumerate()
        .skip(start + 1)
        .find(|(_, l)| l.starts_with('['))
        .map(|(i, _)| i)
        .unwrap_or(lines.len());

    let mut kept: Vec<&str> = Vec::with_capacity(lines.len());
    kept.extend_from_slice(&lines[..start]);
    kept.extend_from_slice(&lines[end..]);
    let mut out = kept.join("\n");
    if config.ends_with('\n') && !out.is_empty() {
        out.push('\n');
    }
    out
}

/// Removes only OMNI's own entry under `mcp_servers:`, and the key itself when
/// nothing else is left under it.
///
/// Deliberately not a block delete. `mcp_servers:` is a mapping the user shares
/// with every other server they have registered, so dropping the whole key to
/// uninstall one entry would take their servers with it. That is the same class
/// of defect as #377, where an installer that spliced a block in the wrong place
/// silently disabled every plugin the user had.
///
/// Returns `None` when there was nothing of ours to remove, so the caller can
/// tell "cleaned" from "was never there" and report the truth either way.