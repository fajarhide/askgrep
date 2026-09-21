fn omni_home_dir() -> PathBuf {
    // Hermes' own tree stays where hermes puts it; OMNI's does not.
    crate::paths::config_home()
}
