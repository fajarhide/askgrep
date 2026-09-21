pub fn hermes_command_patterns() -> Vec<&'static str> {
    vec![
        "terminal", "hermes", "shell", "python", "node", "npm", "pip",
    ]
}

/// Check if a given agent_id looks like Hermes