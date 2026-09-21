fn install_cursor_rule() -> anyhow::Result<()> {
    if !in_a_project() {
        crate::agent_report!(
            "\n  {} Not in a project, so the {} rule was not written.",
            "!".yellow().bold(),
            "omni_run".bold()
        );
        crate::agent_report!(
            "    Cursor cannot rewrite built-in tool output, so run `omni init --cursor`"
        );
        crate::agent_report!(
            "    from a repository, or save this as {}:\n",
            ".cursor/rules/omni.mdc".bold()
        );
        for line in CURSOR_RULE.lines() {
            crate::agent_report!("      {}", line.bright_black());
        }
        return Ok(());
    }

    let path = project_rule_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, CURSOR_RULE)?;
    crate::agent_report!(
        "  {} Wrote {} so the agent reaches for {}",
        "✓".green(),
        ".cursor/rules/omni.mdc".bold(),
        "omni_run".bold()
    );
    crate::agent_report!(
        "    {}",
        "(Cursor cannot rewrite built-in tool output; that tool is the only distill path here)"
            .bright_black()
    );
    Ok(())
}

/// The event each hook has to sit under for Cursor to feed it anything, so
/// `doctor` can assert the wiring rather than the presence of a substring.
const REQUIRED_HOOKS: &[(&str, &str)] = &[
    ("beforeShellExecution", "--pre-hook"),
    ("postToolUse", "--post-hook"),
    ("postToolUseFailure", "--hook"),
    ("stop", "--hook"),
];

/// `(event, flag)` for every OMNI command registered in the file.