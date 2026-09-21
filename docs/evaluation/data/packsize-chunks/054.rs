pub fn is_hermes_agent(agent_id: &str) -> bool {
    let id = agent_id.to_lowercase();
    id == "hermes" || id.starts_with("hermes-") || id.contains("hermes")
}

#[cfg(test)]
mod tests {
    /// The defect #445 filed: an install that lives only in config.yaml was
    /// reported as uninstalled while staying fully configured.
    #[test]
    fn removes_the_omni_server_and_leaves_the_users_own() {
        let config = "plugins:\n  - name: my-linter\n\nmcp_servers:\n  omni:\n    command: \"/usr/local/bin/omni\"\n    args: [\"--mcp\"]\n    env:\n      OMNI_AGENT_ID: \"hermes\"\n  their-server:\n    command: \"/usr/bin/other\"\n";

        let out = super::remove_omni_mcp_server(config).expect("ours was there");

        assert!(!out.contains("omni:"), "our entry must go: {out}");
        assert!(!out.contains("OMNI_AGENT_ID"), "and its children: {out}");
        assert!(out.contains("their-server:"), "theirs must stay: {out}");
        assert!(
            out.contains("mcp_servers:"),
            "the key still has an entry: {out}"
        );
        assert!(
            out.contains("my-linter"),
            "unrelated blocks are untouched: {out}"
        );
    }

    #[test]
    fn drops_the_key_when_omni_was_its_only_entry() {
        let config = "plugins:\n  - name: my-linter\n\nmcp_servers:\n  omni:\n    command: \"/usr/local/bin/omni\"\n    args: [\"--mcp\"]\n";

        let out = super::remove_omni_mcp_server(config).expect("ours was there");

        assert!(
            !out.contains("mcp_servers:"),
            "an empty mapping is noise: {out}"
        );
        assert!(out.contains("my-linter"), "{out}");
    }

    #[test]
    fn reports_nothing_to_remove_rather_than_rewriting_the_file() {
        let config = "plugins:\n  - name: my-linter\n";

        assert!(super::remove_omni_mcp_server(config).is_none());
    }

    #[test]
    fn strips_our_toml_section_and_stops_at_the_next_one() {
        let config = "[core]\nmode = \"balanced\"\n\n[agents.hermes]\nmode = \"aggressive\"\nenable_grep_distillation = true\n\n[agents.pi]\nmode = \"balanced\"\n";

        let out = super::strip_toml_section(config, "[agents.hermes]");

        assert!(!out.contains("[agents.hermes]"), "{out}");
        assert!(!out.contains("aggressive"), "its keys go with it: {out}");
        assert!(out.contains("[agents.pi]"), "the next section stays: {out}");
        a