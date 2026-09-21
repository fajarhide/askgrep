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
        for v in arr.iter_mut() {
            let installed = v
                .get("hooks")
                .and_then(|h| h.as_array())
                .is_some_and(|inner| {
                    inner.iter().any(|h| {
                        crate::agents::is_our_hook(
                            h.get("command").and_then(|c| c.as_str()),
                            hook_cmd,
                        )
                    })
                });
            if installed {
                // Bring the path up to date as well as the matcher. Matching on
                // the exact string meant a reinstall from a different binary
                // matched nothing and pushed a second entry, so OMNI ran twice
                // per call (#454).
                if let Some(inner) = v
                    .get_mut("hooks")
                    .and_then(|h| h.as_array_mut())
                    .and_then(|inner| {
                        inner.iter_mut().find(|h| {
                            crate::agents::is_our_hook(
                                h.get("command").and_then(|c| c.as_str()),
                                hook_cmd,
                            )
                        })
                    })
                    .and_then(|h| h.as_object_mut())
                {
                    inner.insert("command".to_string(), json!(hook_cmd));
                }
                // Already installed, but the matcher still has to be brought up to
                // date. Returning here unconditionally is why widening it in #172
                // would have reached new installs only: every existing settings
                // file names `Bash` and nothing would have rewritten it.
                if let Some(obj) = v.as_object_mut() {
                    obj.insert("matcher".to_string(), json!(matcher));
            