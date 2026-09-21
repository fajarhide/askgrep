pub fn cmd_show(store: &Store) -> Result<()> {
    let ph = project_hash();
    match store.get_knowledge(&ph, GOAL_KEY) {
        Some(goal) => {
            println!(
                "\n{} {}: Current goal\n",
                "omni".bold().cyan(),
                "goal".bold().yellow()
            );
            println!("  {}", goal.bright_white());
            println!();
        }
        None => {
            println!(
                "  {} No goal set. Use {} to set one.",
                "ℹ".blue(),
                "omni goal set '<your goal>'".bright_cyan()
            );
        }
    }
    Ok(())
}

/// Clear (forget) the current project goal.