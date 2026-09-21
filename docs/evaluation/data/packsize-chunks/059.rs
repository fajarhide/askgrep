pub fn sync_session(store: &Store, session: &Arc<Mutex<SessionState>>) {
    let Ok(state) = session.lock() else { return };

    let project_path = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    let proj_hash = project_hash(&project_path);
    let agent_id = detect_agent_id();
    let state_json = serde_json::to_string(&*state).unwrap_or_else(|_| "{}".to_string());

    store.sync_agent_session(&agent_id, &state.session_id, &proj_hash, &state_json);
}

/// Get peer agents and merge their context into the current session summary