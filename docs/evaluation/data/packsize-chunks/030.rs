pub fn install_omni_hooks(exe_path: &str) -> anyhow::Result<()> {
    install_omni_hooks_at(&get_hooks_path(), exe_path)
}

/// The path is a parameter so the tests drive this without setting `HOME`.
/// `cargo` runs tests in parallel and a `set_var` here would decide where an
/// unrelated test writes, which is the failure mode `CONTRIBUTING.md` calls out.