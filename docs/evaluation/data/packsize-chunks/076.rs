pub fn run_context(args: &[String], session: Option<&SessionState>) -> Result<()> {
    if super::wants_help(args) {
        print_help();
        return Ok(());
    }
    super::check_flags("context", args, FLAGS)?;

    if super::has_flag(args, "--tokens") {
        return run_tokens(args);
    }

    let Some(file_path) = args.iter().skip(2).find(|a| !a.starts_with('-')) else {
        print_help();
        return Ok(());
    };

    let cwd = std::env::current_dir()?;
    println!("{}", report(&cwd, file_path, session)?);
    Ok(())
}

/// The report itself, so the CLI and any future caller cannot render it two ways.
///
/// The session is read here rather than by the caller because the hot-file lookup
/// has to use the path the graph resolved, not the one that was typed. `omni
/// context ./src/main.rs` and `omni context src/main.rs` name the same file, and
/// looking up the raw argument answers "Hot in session: no" for one of them.