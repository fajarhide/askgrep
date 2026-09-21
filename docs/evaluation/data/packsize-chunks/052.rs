pub fn hermes_default_config() -> crate::guard::config::AgentConfig {
    crate::guard::config::AgentConfig {
        mode: Some(crate::guard::config::DistillationMode::Efficient),
        enable_readfile_distillation: Some(true),
        enable_grep_distillation: Some(true),
        enable_webfetch_distillation: Some(true),
        pinned_files: Some(vec![
            "AGENTS.md".to_string(),
            ".omni/CONTEXT.md".to_string(),
        ]),
    }
}

/// Command patterns commonly issued by Hermes agent tool calls