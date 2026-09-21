fn installed_hook_events(hooks_path: &PathBuf) -> Vec<(String, String)> {
    let Ok(content) = fs::read_to_string(hooks_path) else {
        return Vec::new();
    };
    let Ok(val) = serde_json::from_str::<Value>(&content) else {
        return Vec::new();
    };
    let Some(hooks) = val.get("hooks").and_then(|h| h.as_object()) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for (event, arr) in hooks {
        for entry in arr.as_array().into_iter().flatten() {
            let Some(cmd) = entry.get("command").and_then(|c| c.as_str()) else {
                continue;
            };
            if !cmd.contains("omni") {
                continue;
            }
            // Longest flag first: `--post-hook` also contains `--hook`.
            for flag in ["--pre-hook", "--post-hook", "--hook"] {
                if cmd.contains(flag) {
                    out.push((event.clone(), flag.to_string()));
                    break;
                }
            }
        }
    }
    out
}
