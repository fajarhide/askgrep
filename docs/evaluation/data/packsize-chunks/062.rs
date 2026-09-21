pub fn auto_learn_project_patterns(
    store: &Store,
    project_path: &str,
    session: &Arc<Mutex<SessionState>>,
) {
    let proj_hash = project_hash(project_path);
    let Ok(state) = session.lock() else { return };

    // Learn: which toolchain this project uses
    for (toolchain, version) in &state.toolchain_hints {
        store.upsert_project_knowledge(
            &proj_hash,
            &format!("toolchain_{toolchain}"),
            version,
            0.95,
        );
    }

    // Learn: most accessed files (high hit count → important to project)
    let mut hot: Vec<(&String, &u32)> = state.hot_files.iter().collect();
    hot.sort_by_key(|(_, c)| std::cmp::Reverse(*c));
    for (file, count) in hot.iter().take(5) {
        if **count >= 3 {
            let confidence = (**count as f32 / 10.0).clamp(0.6, 0.9);
            store.upsert_project_knowledge(
                &proj_hash,
                &format!("hot_file_{}", file.replace('/', "_")),
                file,
                confidence,
            );
        }
    }

    // Learn: persistent error patterns (errors that keep recurring)
    for err in state.active_errors.iter().take(3) {
        let short = crate::util::text::safe_slice(err, 80);
        store.upsert_project_knowledge(
            &proj_hash,
            &format!(
                "recurring_error_{}",
                crate::util::text::safe_slice(short, 20)
            ),
            short,
            0.6,
        );
    }
}

#[cfg(test)]
mod tests {
    /// #437: four implementations, and the three that did not trim split a
    /// project's memory in two the moment a path arrived with a trailing slash.
    #[test]
    fn addresses_a_path_the_same_with_or_without_a_trailing_slash() {
        assert_eq!(project_hash("/foo/bar"), project_hash("/foo/bar/"));
        assert_eq!(project_hash("/foo/bar"), project_hash("/foo/bar///"));
        assert_ne!(project_hash("/foo/bar"), project_hash("/foo/baz"));
    }

    /// Every other site delegates, so this is the check that they still do.
    #[test]
    fn every_caller_agrees_on_one_address() {
        assert_eq!(
            project_hash("/repo/"),
            crate::hooks::session_end::compute_project_hash_for_test("/repo/")
        );
    }

    use super::*;
    use tempfile::tempdir;

    fn get_store() -> (Arc<Store>, tempfile::TempDir) {
        let dir = tempdir()