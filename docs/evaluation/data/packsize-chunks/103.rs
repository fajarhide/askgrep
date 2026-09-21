pub fn run(args: &[String], store: &Store) -> Result<()> {
    // Only the leading token is checked. Free text is the feature here
    // (`omni goal build the auth module`) and a goal may legitimately contain a
    // flag (`omni goal set "ship with --release"`), but a *leading* unknown flag
    // was silently stored as the goal text by the catch-all arm below: `omni goal
    // --nonsense` set the goal to "--nonsense" and exited 0 (#151).
    if let Some(first) = args.first()
        && first.starts_with("--")
    {
        super::check_flags("goal", &args[..1], FLAGS)?;
    }

    let sub = args.first().map(|s| s.as_str()).unwrap_or("show");
    match sub {
        "set" => {
            let goal = args[1..].join(" ");
            cmd_set(goal.trim(), store)
        }
        "clear" | "unset" => cmd_clear(store),
        "show" | "--show" | "--status" => cmd_show(store),
        "--help" | "-h" | "help" => {
            print_help();
            Ok(())
        }
        _ => {
            // Treat the whole args as the goal text (e.g. `omni goal build the auth module`)
            let goal = args.join(" ");
            cmd_set(goal.trim(), store)
        }
    }
}
