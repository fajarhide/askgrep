pub fn check_status(val: &Value, exe_path: &str) -> (bool, bool, bool) {
    let hooks = match val.get("hooks").and_then(|v| v.as_object()) {
        Some(h) => h,
        None => return (false, false, false),
    };

    let check = |event: &str| -> bool {
        if let Some(arr) = hooks.get(event).and_then(|v| v.as_array()) {
            for v in arr {
                if let Some(inner_arr) = v.get("hooks").and_then(|v2| v2.as_array()) {
                    for hook_def in inner_arr {
                        if let Some(cmd) = hook_def.get("command").and_then(|c| c.as_str())
                            && cmd.contains(exe_path)
                            && (cmd.contains("--hook")
                                || cmd.contains("--post-hook")
                                || cmd.contains("--pre-hook")
                                || cmd.contains("--session-start")
                                || cmd.contains("--pre-compact"))
                        {
                            return true;
                        }
                    }
                }
            }
        }
        false
    };

    (
        check("PostToolUse"),
        check("SessionStart"),
        check("PreCompact"),
    )
}
