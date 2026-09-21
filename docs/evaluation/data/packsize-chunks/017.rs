fn ensure_hook_entry(arr_val: &mut Value, cmd: &str) {
    let Some(arr) = arr_val.as_array_mut() else {
        return;
    };

    // Drop every prior entry of ours, in either shape and at any path, before
    // adding the current one. Matching on the exact command was not enough:
    // moving the binary (a dev build to a stable copy, or a Homebrew upgrade)
    // left the old entry in place, so the hook ran twice per command and every
    // distillation was recorded twice (#369). Third-party entries are untouched.
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

    arr.push(json!({
        "hooks": [{ "type": "command", "command": cmd, "timeout": 10 }]
    }));
}

/// True for a hook command this tool installed, in either shape it has written.