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
                                        || c.contains("--pre-hook"))
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
    /// Gemini carried its own copy of the exact-string test that #454 was filed
    /// against for Claude Code. Only one of the two was reported, so this is the
    /// half that was found by looking rather than by being told.
    #[test]
    fn reinstalling_from_another_path_moves_the_hook_rather_than_adding_one() {
        let mut val = json!({});
        install_omni_hooks(&mut val, "/repo/target/debug/omni");
        install_omni_hooks(&mut val, "/opt/homebrew/bin/omni");

        let hooks = val["hooks"].as_object().expect("hooks written");
        assert!(!hooks.is_empty());
        for (event, arr) in hooks {
            let ours: Vec<String> = arr
                .as_array()
                .expect("array")
                .iter()
                .flat_map(|m| m["hooks"].as_array().cloned().unwrap_or_default())
                .filter_map(|h| h["command"].as_str().map(str::to_string))
                .filter(|c| c.contains("omni"))
                .collect();
            assert_eq!(
                ours.len(),
                1,
                "{event} would run OMNI {} times: {ours:?}",
                ours.len()
            );
            assert!(
                ours[0].starts_with("/opt/homebrew/bin/omni"),
                "{event}: {ours:?}"
            );
        }
    }

    use super::*;

    /// #351: the matcher was `"Bash"`, which is Claude Code's tool na