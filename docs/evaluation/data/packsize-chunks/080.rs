fn print_help() {
    println!(
        "\n{} {}: Dependency context for one file",
        "omni".bold().cyan(),
        "context".bold().yellow()
    );
    println!(
        "Which files it imports, which import it, and whether this session keeps touching it."
    );
    println!();
    println!("Usage: omni context <file>");
    println!();
    super::print_flags(FLAGS);
    println!();
}

#[cfg(test)]
mod tests {
    use super::report;
    use crate::pipeline::SessionState;

    /// Greptile on #609. The graph resolves `./src/x.rs` and `src/x.rs` to one
    /// path; looking the session up by the argument as typed answers "Hot in
    /// session: no" for one spelling of the same file.
    #[test]
    fn the_hot_lookup_uses_the_path_the_graph_resolved() {
        let dir = tempfile::tempdir().expect("tempdir");
        let src = dir.path().join("src");
        std::fs::create_dir_all(&src).expect("mkdir");
        std::fs::write(src.join("thing.rs"), "// nothing to import\n").expect("write");

        let mut session = SessionState::new();
        // Recorded the way the tracker records it, resolved rather than as typed.
        let resolved = report(dir.path(), "src/thing.rs", None).expect("report");
        let name = resolved
            .lines()
            .next()
            .and_then(|l| l.strip_prefix("OMNI Context for "))
            .expect("the first line names the file")
            .to_string();
        session.hot_files.insert(name, 4);

        for spelling in ["src/thing.rs", "./src/thing.rs"] {
            let out = report(dir.path(), spelling, Some(&session)).expect("report");
            assert!(
                out.contains("Hot in session: yes (4x)"),
                "{spelling} reported the wrong hot status:\n{out}"
            );
        }
    }
}

#[cfg(test)]
mod tokens_tests {
    use crate::pipeline::{DistillResult, Route};
    use crate::store::sqlite::Store;

    /// A store holding one distilled call and one declined call, with sizes that
    /// cannot be confused: what survived distillation and what was handed back are
    /// deliberately different numbers.
    fn seeded() -> (Store, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open_path(&dir.path().join("omni.db")).unwrap();
        let row = |route: Route, input: usize, output: usize| DistillResult {
            output: