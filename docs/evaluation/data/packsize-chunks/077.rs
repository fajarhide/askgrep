pub fn report(
    cwd: &std::path::Path,
    file_path: &str,
    session: Option<&SessionState>,
) -> Result<String> {
    let graph = graph::indexer::build_graph(cwd)?;
    let ctx = graph.context_for(file_path);
    let hot_count = session
        .and_then(|s| s.hot_files.get(&ctx.file_path).copied())
        .unwrap_or(0);

    let list = |items: &[String]| {
        if items.is_empty() {
            "none detected".to_string()
        } else {
            items.iter().take(8).cloned().collect::<Vec<_>>().join(", ")
        }
    };

    Ok(format!(
        "OMNI Context for {}\nImports: {}\nImported by: {}\nHot in session: {}\n",
        ctx.file_path,
        list(&ctx.imports),
        list(&ctx.imported_by),
        if hot_count > 0 {
            format!("yes ({hot_count}x)")
        } else {
            "no".to_string()
        }
    ))
}

/// `omni context --tokens`: what OMNI saw, what it removed, and what it declined.
///
/// #612 came from someone saying "I thought it was just me, because of polluted
/// context". That is a guess they cannot check, and OMNI is the one thing in the
/// stack already holding the answer for the part it can see.
///
/// **What it deliberately does not report.** #612 asks first for the share of
/// context that is tool output against everything else. OMNI cannot answer that and
/// must not appear to: it records what passed through its hook, and never sees the
/// prompts, the system block, the tool definitions or the assistant's own text. A
/// denominator built from what OMNI happens to hold would read as "share of your
/// context" while meaning "share of the part OMNI touched", which is the kind of
/// figure this project exists to refuse. The closing line says so instead.
///
/// Read only, and no new store query: `engine_totals`, `passthrough_reasons` and
/// `filter_breakdown` already existed, the first two built for #665 and #672.