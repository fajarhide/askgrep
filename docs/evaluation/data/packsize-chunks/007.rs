pub fn install_hooks(exe_path: &str) -> anyhow::Result<()> {
    let (path, mut val) = initialize_settings()?;
    let _ = backup_settings(&path);

    install_omni_hooks(&mut val, exe_path);
    fs::write(&path, serde_json::to_string_pretty(&val)?)?;
    crate::agent_report!(
        "  {} {} installed in Claude settings",
        "✓".green(),
        "Hooks".bold()
    );
    Ok(())
}
