fn retain_non_omni(hooks: &mut serde_json::Map<String, Value>) {
    for arr_val in hooks.values_mut() {
        if let Some(arr) = arr_val.as_array_mut() {
            arr.retain(|v| {
                v.get("command").and_then(|c| c.as_str()).is_none_or(|c| {
                    !(c.contains("omni")
                        && (c.contains("--pre-hook")
                            || c.contains("--post-hook")
                            || c.contains("--hook")))
                })
            });
        }
    }
}
