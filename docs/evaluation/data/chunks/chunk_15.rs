fn codex_dir_from(codex_home: Option<std::ffi::OsString>, home: Option<PathBuf>) -> PathBuf {
    match codex_home {
        Some(dir) if !dir.is_empty() => PathBuf::from(dir),
        _ => home.unwrap_or_else(|| PathBuf::from(".")).join(".codex"),
    }
}
