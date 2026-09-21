fn in_a_project() -> bool {
    PathBuf::from(".git").exists()
}

/// Writes the rule when run inside a project, prints it otherwise.
///
/// Printing alone was the cheap option and it does not work: the user has to
/// notice a hint, copy it, and create a file, and the tool sits unused when they
/// do not. `omni_run` is the *only* way a shell command is distilled on Cursor
/// (#349, #351), so an unused tool is the whole feature not working.
///
/// Only inside a project, because `.cursor/rules/` is per-project and OMNI has
/// no business writing into a home directory or an unrelated folder. Uninstall
/// removes it again.