pub fn project_hash(path: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(path.trim_end_matches('/').as_bytes());
    hex::encode(&hasher.finalize()[..8])
}

/// Detect the current agent from environment.
///
/// This runs on the `omni exec` and pipe paths, where there is no JSON payload
/// to inspect. The ids it returns **must match**
/// `hooks::normalize::detect_agent_id`, which names the same agents from the
/// payload shape on the hook path, otherwise one agent's work splits across two
/// rows in `omni stats` and neither row is the true total (#160).