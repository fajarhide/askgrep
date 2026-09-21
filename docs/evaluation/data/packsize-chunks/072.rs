fn find_omni_references(val: &Value) -> bool {
    match val {
        Value::String(s) => s.to_lowercase().contains("omni"),
        Value::Array(arr) => arr.iter().any(find_omni_references),
        Value::Object(map) => map.values().any(find_omni_references),
        _ => false,
    }
}

/// Collect OMNI-related source strings from Pi settings.