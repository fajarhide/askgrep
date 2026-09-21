fn remove_omni_mcp_server(config: &str) -> Option<String> {
    let lines: Vec<&str> = config.lines().collect();
    let start = lines
        .iter()
        .position(|l| l.trim_end() == "mcp_servers:" || l.trim_end() == "mcp:")?;

    // The block runs to the next top-level key. Blank lines belong to it, so a
    // trailing blank does not end it early.
    let mut end = lines.len();
    for (i, line) in lines.iter().enumerate().skip(start + 1) {
        if !line.trim().is_empty() && !line.starts_with([' ', '\t']) {
            end = i;
            break;
        }
    }

    let indent = |l: &str| l.len() - l.trim_start().len();
    let child_indent = lines[start + 1..end]
        .iter()
        .find(|l| !l.trim().is_empty())
        .map(|l| indent(l))?;

    let omni_at = lines[start + 1..end]
        .iter()
        .position(|l| indent(l) == child_indent && l.trim() == "omni:")
        .map(|i| start + 1 + i)?;

    let mut omni_end = end;
    for (i, line) in lines
        .iter()
        .enumerate()
        .skip(omni_at + 1)
        .take(end - omni_at - 1)
    {
        if !line.trim().is_empty() && indent(line) <= child_indent {
            omni_end = i;
            break;
        }
    }

    let mut kept: Vec<&str> = Vec::with_capacity(lines.len());
    kept.extend_from_slice(&lines[..start + 1]);
    kept.extend_from_slice(&lines[start + 1..omni_at]);
    kept.extend_from_slice(&lines[omni_end..]);

    // If the mapping is now empty, the key is noise, so it goes too.
    let still_has_children = kept
        .iter()
        .skip(start + 1)
        .take_while(|l| l.trim().is_empty() || l.starts_with([' ', '\t']))
        .any(|l| !l.trim().is_empty());
    if !still_has_children {
        kept.remove(start);
    }

    let mut out = kept.join("\n");
    if config.ends_with('\n') {
        out.push('\n');
    }
    Some(out)
}
