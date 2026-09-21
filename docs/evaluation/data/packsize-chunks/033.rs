pub fn remove_omni_hooks() -> anyhow::Result<()> {
    let hooks_path = get_hooks_path();
    if !hooks_path.exists() {
        return Ok(());
    }
    let content = fs::read_to_string(&hooks_path)?;
    let Ok(mut val) = serde_json::from_str::<Value>(&content) else {
        return Ok(());
    };

    if let Some(obj) = val.as_object_mut()
        && let Some(hooks) = obj.get_mut("hooks").and_then(|h| h.as_object_mut())
    {
        retain_non_omni(hooks);
    }

    fs::write(&hooks_path, serde_json::to_string_pretty(&val)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_mcp_server_is_idempotent() {
        let mut val = json!({});
        install_mcp_server(&mut val, "/usr/local/bin/omni");
        install_mcp_server(&mut val, "/usr/local/bin/omni");
        let servers = val
            .get("mcpServers")
            .and_then(|v| v.as_object())
            .expect("mcpServers exists");
        assert_eq!(servers.len(), 1);
        assert!(servers.contains_key("omni"));
    }

    #[test]
    fn remove_mcp_server_removes_only_omni() {
        let mut val = json!({ "mcpServers": { "omni": {"command": "/usr/local/bin/omni", "args": ["--mcp"]}, "other": {"command": "other"} } });
        remove_mcp_server(&mut val);
        let servers = val
            .get("mcpServers")
            .and_then(|v| v.as_object())
            .expect("mcpServers exists");
        assert!(!servers.contains_key("omni"));
        assert!(servers.contains_key("other"));
    }

    /// The old version of this built its own JSON and then asserted on the JSON it
    /// had just built, so it passed while `--post-hook` sat on `afterFileEdit` and
    /// received nothing for the life of the integration (#340). Drive the real
    /// writer and assert the event each command lands on.
    #[test]
    fn registers_the_post_hook_on_the_event_that_carries_output() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("hooks.json");
        install_omni_hooks_at(&path, "/usr/local/bin/omni").expect("install");

        let installed = installed_hook_events(&path);
        for (event, flag) in REQUIRED_HOOKS {
            assert!(
                installed.iter().any(|(e, f)| e == event && f == flag),
                "{flag} must be registered on {event}, got {installed:?}"
            );
        }
        asse