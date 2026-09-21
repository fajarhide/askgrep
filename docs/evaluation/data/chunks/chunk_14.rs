fn get_codex_dir() -> PathBuf {
    codex_dir_from(std::env::var_os("CODEX_HOME"), dirs::home_dir())
}

/// The choice itself, kept free of the environment so it can be tested without
/// a concurrently running test seeing the mutation.