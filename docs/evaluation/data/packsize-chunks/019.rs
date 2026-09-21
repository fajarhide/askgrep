pub fn remove_omni_hooks() -> anyhow::Result<()> {
    let hooks_path = get_codex_dir().join("hooks.json");
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
        for (_key, arr_val) in hooks.iter_mut() {
            if let Some(arr) = arr_val.as_array_mut() {
                // Entries live one level down in a matcher group's `hooks` array,
                // and older installs wrote the command at the top level. Uninstall
                // has to find both, or it leaves behind exactly the orphan that
                // bricked `config.toml` in #351.
                arr.retain(|v| {
                    let flat = v.get("command").and_then(|c| c.as_str());
                    let nested = v
                        .get("hooks")
                        .and_then(|h| h.as_array())
                        .map(|inner| {
                            inner
                                .iter()
                                .filter_map(|h| h.get("command").and_then(|c| c.as_str()))
                                .any(is_omni_hook_command)
                        })
                        .unwrap_or(false);

                    !(flat.is_some_and(is_omni_hook_command) || nested)
                });
            }
        }
    }

    fs::write(&hooks_path, serde_json::to_string_pretty(&val)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {

    /// #369: dedupe matched the exact command string, so moving the binary left
    /// the previous entry in place. Both fired, every command was distilled
    /// twice and recorded twice. Reproduced by repointing a live install from a
    /// build directory to a stable copy.
    #[test]
    fn replaces_an_entry_that_points_at_an_older_binary_path() {
        let mut arr = json!([{
            "hooks": [{ "type": "command", "command": "/old/path/omni --pre-hook" }]
        }]);

        ensure_hook_entry(&mut arr, "/new/path/omni --pre-hook");

        let arr = arr.as_array().unwrap();
        assert_eq!(arr.len(), 1, "the old path survived: {arr:?}");
        assert_eq!(arr[0]["hooks"][0]["command"], "/new/path/omni --pre-hook");
    