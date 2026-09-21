fn install_omni_hooks_at(hooks_path: &PathBuf, exe_path: &str) -> anyhow::Result<()> {
    if let Some(parent) = hooks_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut val = if hooks_path.exists() {
        let content = fs::read_to_string(hooks_path)?;
        serde_json::from_str(&content).unwrap_or_else(|_| json!({}))
    } else {
        json!({})
    };

    let obj = match val.as_object_mut() {
        Some(o) => o,
        None => {
            val = json!({});
            val.as_object_mut().unwrap()
        }
    };

    let hooks = obj
        .entry("hooks")
        .or_insert_with(|| json!({}))
        .as_object_mut()
        .unwrap();

    let pre_cmd = format!("{} --pre-hook", exe_path);
    let post_cmd = format!("{} --post-hook", exe_path);
    let hook_cmd = format!("{} --hook", exe_path);

    // Drop every OMNI entry first, then re-add. Two things this buys: an install
    // after the binary moved leaves one command rather than two, and the stale
    // `afterFileEdit` registration from before #340 is purged on upgrade instead
    // of sitting in the file forever, firing on edits that carry no output.
    retain_non_omni(hooks);

    let ensure_hook = |arr_val: &mut Value, cmd: &str| {
        let arr = arr_val.as_array_mut().unwrap();
        for v in arr.iter() {
            if v.get("command").and_then(|c| c.as_str()) == Some(cmd) {
                return;
            }
        }
        arr.push(json!({ "command": cmd }));
    };

    // `postToolUse` is Cursor's analogue of Claude Code's `PostToolUse`, and it
    // is the only one of these that carries command output. `afterFileEdit`,
    // registered here until #340, fires on a file write: the distiller was handed
    // nothing for the entire life of the integration, which is why this
    // installation has 9,857 `claude_code` rows and zero `cursor` ones.
    for (event, cmd) in [
        ("beforeShellExecution", &pre_cmd),
        ("postToolUse", &post_cmd),
        // Failed commands carry the error payload OMNI reads for #120.
        ("postToolUseFailure", &hook_cmd),
        // Cursor has no PreCompact, and `stop` is the closest thing to
        // SessionEnd: the flush that lets the next session start informed.
        ("stop", &hook_cmd),
    ] {
        ensure_hook(hooks.entry(event).or_insert_with(|| json!([])), cmd);
    }

    fs::write(hooks