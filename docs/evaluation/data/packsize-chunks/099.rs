fn project_hash() -> String {
    let path = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "global".to_string());
    crate::agents::multiagent::project_hash(&path)
}

/// Set the project goal (overwrites previous).