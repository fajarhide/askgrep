pub fn cmd_set(goal: &str, store: &Store) -> Result<()> {
    if goal.trim().len() < 5 {
        anyhow::bail!("Goal is too short. Describe your project objective clearly.");
    }
    if goal.len() > 500 {
        anyhow::bail!("Goal too long (max 500 chars). Keep it concise.");
    }
    let ph = project_hash();
    store.upsert_project_knowledge(&ph, GOAL_KEY, goal, 1.0);
    println!("{} Goal set: {}", "✓".green(), goal.bright_white());
    println!(
        "  {} OMNI will inject this goal at the start of every new session.",
        "→".bright_black()
    );
    Ok(())
}

/// Display the current project goal.