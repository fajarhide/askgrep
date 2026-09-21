fn project_rule_path() -> PathBuf {
    PathBuf::from(".cursor/rules/omni.mdc")
}

/// True when the working directory looks like a checkout rather than a home
/// directory, which is the only place a project rule belongs.