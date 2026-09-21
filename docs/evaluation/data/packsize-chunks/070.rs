fn pi_settings_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".pi")
        .join("agent")
        .join("settings.json")
}

/// Return the project-local Pi settings path (`.pi/settings.json` in CWD).