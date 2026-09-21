pub fn remove_omni_hooks(val: &mut Value) {
    if let Some(obj) = val.as_object_mut()
        && let Some(hooks) = obj.get_mut("hooks").and_then(|h| h.as_object_mut())
    {
        for (_key, arr_val) in hooks.iter_mut() {
            if let Some(arr) = arr_val.as_array_mut() {
                arr.retain(|v| {
                    if let Some(inner) = v.get("hooks").and_then(|h| h.as_array()) {
                        !inner.iter().any(|h| {
                            h.get("command").and_then(|c| c.as_str()).is_some_and(|c| {
                                c.contains("omni")
                                    && (c.contains("--hook")
                                        || c.contains("--post-hook")
                                        || c.contains("--pre-hook")
                                        || c.contains("--pre-compact"))
                            })
                        })
                    } else {
                        true
                    }
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// #351: an upgrade must remove the `PreToolUse` / `PostToolUse` /
    /// `PreCompact` entries an earlier version wrote. Cline never emits those
    /// names, so they were config the host ignored while doctor called them
    /// installed. Another tool's hooks in the same file must survive.
    #[test]
    fn install_purges_the_hook_names_cline_never_emits() {
        let mut val = json!({
            "mcpServers": {},
            "hooks": {
                "PreToolUse": [{ "matcher": "Bash", "hooks": [{ "type": "command", "command": "/old/omni --pre-hook" }] }],
                "PostToolUse": [{ "matcher": "Bash", "hooks": [{ "type": "command", "command": "/old/omni --post-hook" }] }],
                "TaskStart": [{ "matcher": "", "hooks": [{ "type": "command", "command": "/other/tool.sh" }] }]
            }
        });

        remove_omni_hooks(&mut val);
        let dumped = serde_json::to_string(&val).expect("json");

        assert!(
            !dumped.contains("--pre-hook"),
            "stale hook survived: {dumped}"
        );
        assert!(
            !dumped.contains("--post-hook"),
            "stale hook survived: {dumped}"
        );
        assert!(
            dumped.contains("/other/tool.sh"),
            "another tool's hook must survive: {dumped}"
        );