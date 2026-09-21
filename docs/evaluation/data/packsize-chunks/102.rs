pub fn cmd_clear(store: &Store) -> Result<()> {
    let ph = project_hash();
    // Overwrite with empty string effectively removes the value from recall,
    // but keeps the row (upsert). Using confidence=0 marks it stale.
    match store.get_knowledge(&ph, GOAL_KEY) {
        Some(_) => {
            store.upsert_project_knowledge(&ph, GOAL_KEY, "", 0.0);
            println!("{} Goal cleared.", "✓".green());
        }
        None => {
            println!("  {} No goal was set.", "ℹ".blue());
        }
    }
    Ok(())
}

/// Entry point for `omni goal [set|show|clear] ...`
/// Read by both `print_help` and the guard below (#151).
const FLAGS: super::Flags = &[("--show, --status", "Print the current goal")];
