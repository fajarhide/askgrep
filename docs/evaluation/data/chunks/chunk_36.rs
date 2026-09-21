pub fn install_omni_hooks(val: &mut Value, exe_path: &str) {
    let obj = match val.as_object_mut() {
        Some(o) => o,
        None => {
            *val = json!({});
            val.as_object_mut().unwrap()
        }
    };

    let hooks = obj
        .entry("hooks")
        .or_insert_with(|| json!({}))
        .as_object_mut()
        .unwrap();

    if exe_path.is_empty() {
        return;
    }

    let ensure_hook = |arr_val: &mut serde_json::Value, matcher: &str, hook_cmd: &str| {
        let arr = arr_val.as_array_mut().unwrap();
        // Ours is identified by binary and flag, never by the whole command, so
        // reinstalling from a different path moves the entry instead of adding a
        // second one that runs OMNI twice per call (#454).
        for v in arr.iter_mut() {
            if let Some(inner) = v.get_mut("hooks").and_then(|h| h.as_array_mut()) {
                for h in inner.iter_mut() {
                    if crate::agents::is_our_hook(
                        h.get("command").and_then(|c| c.as_str()),
                        hook_cmd,
                    ) {
                        if let Some(obj) = h.as_object_mut() {
                            obj.insert("command".to_string(), json!(hook_cmd));
                        }
                        return;
                    }
                }
            }
        }

        arr.push(json!({
            "matcher": matcher,
            "hooks": [
                {
                    "type": "command",
                    "command": hook_cmd
                }
            ]
        }));
    };

    let pre_cmd = format!("{} --pre-hook", exe_path);
    let post_cmd = format!("{} --post-hook", exe_path);

    // Gemini CLI uses BeforeTool / AfterTool (analogous to Claude's PreToolUse / PostToolUse)
    // Gemini matches on ITS OWN tool names, not Claude's. The shell tool is
    // `run_shell_command`; a matcher of "Bash" never fires, which is why this
    // integration recorded zero rows (#351). Same defect class as Cursor's
    // `afterFileEdit`: config written, host ignores it, doctor reports OK.
    ensure_hook(
        hooks.entry("BeforeTool").or_insert_with(|| json!([])),
        "run_shell_command",
        &pre_cmd,
    );
    ensure_hook(
        hooks.entry("AfterTool").or_insert_with(|| json!([])),
        "run_shell_command",
        &post_cmd,
    );
}
