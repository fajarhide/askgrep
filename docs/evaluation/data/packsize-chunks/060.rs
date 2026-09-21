pub fn build_peer_context(store: &Store, project_path: &str) -> Option<String> {
    let proj_hash = project_hash(project_path);
    let my_agent = detect_agent_id();
    let peers = store.get_active_agents_for_project(&proj_hash, &my_agent);

    if peers.is_empty() {
        return None;
    }

    let mut ctx = format!(
        "\n🤝 Multi-Agent Context ({} peer{} active on this project):\n",
        peers.len(),
        if peers.len() == 1 { "" } else { "s" }
    );

    for peer in &peers {
        let age_mins = (chrono::Utc::now().timestamp() - peer.last_active) / 60;
        let age = if age_mins < 60 {
            format!("{age_mins}m ago")
        } else {
            format!("{}h ago", age_mins / 60)
        };

        if let Ok(peer_state) = serde_json::from_str::<SessionState>(&peer.state_json) {
            let task = peer_state
                .inferred_task
                .as_deref()
                .unwrap_or("unknown task");

            let mut hot: Vec<(&String, &u32)> = peer_state.hot_files.iter().collect();
            hot.sort_by_key(|(_, c)| std::cmp::Reverse(*c));
            let top_files: Vec<&str> = hot.iter().take(3).map(|(f, _)| f.as_str()).collect();

            ctx.push_str(&format!("  [{age}] {agent}: {task}", agent = peer.agent_id));
            if !top_files.is_empty() {
                ctx.push_str(&format!(" | files: {}", top_files.join(", ")));
            }
            if let Some(err) = peer_state.active_errors.first() {
                let short = err.chars().take(60).collect::<String>();
                ctx.push_str(&format!(" | ⚠ {short}"));
            }
            ctx.push('\n');
        } else {
            ctx.push_str(&format!("  [{age}] {}: active\n", peer.agent_id));
        }
    }

    Some(ctx)
}

/// Inject project-level knowledge into session start summary