pub fn run(args: &[String]) -> anyhow::Result<()> {
    if super::wants_help(args) {
        print_help();
        return Ok(());
    }
    super::check_flags("doctor", args, FLAGS)?;

    if super::has_flag(args, "--json") {
        return run_json(args);
    }

    let fix_mode = super::has_flag(args, "--fix");
    let detail = super::has_flag(args, "--detail");

    let mut all_ok = true;
    let mut warnings: Vec<String> = Vec::new();
    println!();
    super::print_rule();
    println!(" {}: Installation Diagnostics", "OMNI Doctor".bold().cyan());
    super::print_rule();

    // 1. Binary Version
    let status = crate::guard::update::get_status();
    let version_info = match status {
        crate::guard::update::Status::Latest => {
            format!("omni v{} {}", env!("CARGO_PKG_VERSION"), "[LATEST]".green())
        }
        crate::guard::update::Status::UpdateAvailable(v) => format!(
            "omni v{} {} (Latest: {})",
            env!("CARGO_PKG_VERSION"),
            "[UPDATE]".yellow().bold(),
            v.green()
        ),
        crate::guard::update::Status::Ahead => format!(
            "omni v{} {}",
            env!("CARGO_PKG_VERSION"),
            "[AHEAD/RC]".blue().bold()
        ),
    };

    println!("  {:<15} {}", "Binary:".bright_black(), version_info);

    // #137: `[LATEST]` above answers "is there a newer release than mine". It
    // cannot see fixes that were never released, because then the newest
    // release *is* the running version, exactly the state #127 filed, where
    // six correctness fixes sat merged and unshipped while doctor said
    // `[LATEST]`. This is the other question, answered from the tree the binary
    // was built from.
    if unreleased_entries() > 0 {
        println!(
            "  {} {}",
            format!("[{} UNRELEASED]", unreleased_entries())
                .yellow()
                .bold(),
            "built into this binary, in no release. Cut a tag".bright_black()
        );
    }

    // 2. Config Dir (with actual write test for sandbox detection)
    let conf_dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".omni");
    if conf_dir.exists() {
        // Actual write test catches sandbox restrictions
        let test_file = conf_dir.join(".write_test");
        match fs::write(&test_file, "ok") {
            Ok(_) => {
               