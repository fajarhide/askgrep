fn print_help() {
    println!(
        "\n{} {}: North Star context pinning",
        "omni".bold().cyan(),
        "goal".bold().yellow()
    );
    println!("\n{}", "USAGE:".bold().bright_white());
    println!(
        "  omni {} {}",
        "goal".cyan(),
        "[SUBCOMMAND] [TEXT]".bright_black()
    );
    println!("\n{}", "SUBCOMMANDS:".bold().bright_white());
    println!(
        "  {: <12} Set the project goal (default)",
        "set <text>".cyan()
    );
    println!("  {: <12} Display current goal", "show".cyan());
    println!("  {: <12} Remove current goal", "clear".cyan());
    println!("\n{}", "EXAMPLES:".bold().bright_white());
    println!(
        "  omni goal set 'Build OAuth2 integration for the API'  {}",
        "# Set a goal".bright_black()
    );
    println!(
        "  omni goal show                                         {}",
        "# Display current goal".bright_black()
    );
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tempfile::tempdir;

    fn get_store() -> (Arc<Store>, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("omni.db");
        (Arc::new(Store::open_path(&db_path).unwrap()), dir)
    }

    #[test]
    fn set_and_show_goal() {
        let (store, _dir) = get_store();
        cmd_set("Build the auth module", &store).unwrap();
        let ph = project_hash();
        let g = store.get_knowledge(&ph, GOAL_KEY);
        assert_eq!(g.as_deref(), Some("Build the auth module"));
    }

    #[test]
    fn set_empty_goal_returns_error() {
        let (store, _dir) = get_store();
        let res = cmd_set("hi", &store);
        assert!(res.is_err());
    }

    #[test]
    fn clear_removes_goal() {
        let (store, _dir) = get_store();
        cmd_set("Ship v1.0 before Friday", &store).unwrap();
        cmd_clear(&store).unwrap();
        // After clear, confidence=0 so get_knowledge still returns empty string
        // (row exists but is semantically cleared)
        let ph = project_hash();
        let g = store.get_knowledge(&ph, GOAL_KEY);
        assert!(g.map(|v| v.is_empty()).unwrap_or(true));
    }

    #[test]
    fn show_prints_no_goal_when_unset() {
        let (store, _dir) = get_store();
        // Should not panic
        assert!(cmd_show(&store).is_ok());
    }
}
