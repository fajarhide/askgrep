pub fn build_knowledge_context(store: &Store, project_path: &str) -> Option<String> {
    let proj_hash = project_hash(project_path);
    let knowledge = store.get_project_knowledge(&proj_hash);

    if knowledge.is_empty() {
        return None;
    }

    let mut ctx = "\n📚 Project Knowledge (learned across sessions):\n".to_string();
    for (key, value, confidence) in &knowledge {
        if *confidence >= 0.7 {
            ctx.push_str(&format!("  • [{key}]: {value}\n"));
        }
    }
    Some(ctx)
}

/// Auto-learn project patterns and save to project_knowledge
/// Called periodically during session to build up semantic memory