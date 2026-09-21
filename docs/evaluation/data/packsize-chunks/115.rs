pub fn print_flag_group(title: &str, flags: &[&(&str, &str)]) {
    // Sized to the longest entry rather than a fixed width, which
    // `--all-commands` and `--validate <file.toml>` both overflow.
    let width = flags.iter().map(|(spec, _)| spec.len()).max().unwrap_or(0);

    println!("\n{}", title.bold().bright_white());
    for (spec, description) in flags {
        println!("  {} {}", format!("{spec:<width$}").cyan(), description);
    }
}

/// Reject any `--flag` this subcommand does not accept.
///
/// clap cannot do this for us. Every subcommand is declared `trailing_var_arg`
/// with a `Vec<String>` catch-all and each module then re-parses raw argv by
/// hand, so clap is never told the valid set and nothing can detect a value
/// outside it. Untouched, `omni stats --detial` silently ran the default
/// overview and exited 0, the user asked for one mode, got another, and the
/// output said nothing about the flag being ignored (#151).
///
/// Long `--flags` are always checked. A single-letter `-x` is checked only when
/// the subcommand declares at least one short flag, so free-form text keeps
/// passing through (`omni remember "build with -O2"`, `omni engram list`).
/// The flag an argument names, ignoring any `=value` attached to it.
///
/// `check_flags` accepts `--flag=value` and validates the name alone, so every
/// consumer that then compares the whole argument silently stops routing it:
/// `omni reset --openclaw=1` passed validation, matched nothing, and dropped into
/// the interactive menu with the integration still installed. One function so the
/// accepted form and the routed form cannot disagree.