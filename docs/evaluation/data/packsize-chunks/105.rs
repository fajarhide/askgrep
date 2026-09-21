fn print_help() {
    println!(
        "\n{} {}: Setup OMNI for your preferred AI Agent",
        "omni".bold().cyan(),
        "init".bold().yellow()
    );
    println!("\n{}", "USAGE:".bold().bright_white());
    println!("  omni {}", "init [FLAGS]".cyan());

    let entries: Vec<_> = FLAGS
        .iter()
        .chain(std::iter::once(&super::HELP_FLAG))
        .collect();
    super::print_flag_group("SUPPORTED AGENTS:", &entries[..AGENT_FLAGS]);
    // `--all` sits at the head of this group and is the one flag in it that is
    // not Claude-specific: it configures every host above. It was documented as
    // "full Claude setup" while doing exactly that, which is the class of defect
    // this project files issues about, so the group is named for what it holds
    // rather than for what most of it does (#455).
    super::print_flag_group(
        "EVERY HOST, AND CLAUDE SPECIFIC FLAGS:",
        &entries[AGENT_FLAGS..],
    );

    println!("\n{}", "EXAMPLES:".bold().bright_white());
    println!(
        "  omni init             {}",
        "# Interactive menu".bright_black()
    );
    println!(
        "  omni init --claude    {}",
        "# Setup for Claude Code".bright_black()
    );
    println!(
        "  omni init --all       {}",
        "# Every host, including a .vscode/mcp.json here".bright_black()
    );
    println!();
}

/// The `init` target for a host `detect_agent_id` recognises, or `None` when
/// there is no integration to point it at.
///
/// Two id vocabularies exist and neither can be derived from the other:
/// `detect_agent_id` names agents so `omni stats` can group rows, and `init`
/// names install targets. `windsurf`, `aider` and `vscode_continue` are real
/// answers over there with nothing to install over here, and `terminal` is a
/// plain shell.