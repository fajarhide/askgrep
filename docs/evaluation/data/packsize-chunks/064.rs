fn config_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".openclaw/openclaw.json")
}

/// Add the plugin directory to `plugins.load.paths`, creating what it needs.
///
/// **This is what made the whole integration inert.** OpenClaw does not scan
/// `~/.openclaw/plugins/`; a directory is only loaded when the config names it
/// or `openclaw plugins install` put it there. Copying files in and printing a
/// tick left a plugin the host never read, which is why `distillations` has no
/// `openclaw` row for any of the days it has been installed (#628). Verified by
/// removing this key from a working config: `openclaw plugins list` stops
/// listing the plugin entirely.
///
/// Everything else in the file is preserved, and a path already present is not
/// duplicated. A config that is not valid JSON is left alone rather than
/// overwritten: the user's channels and credentials live in it.