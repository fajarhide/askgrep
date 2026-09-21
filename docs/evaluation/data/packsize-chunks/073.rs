fn collect_omni_sources(val: &Value) -> Vec<String> {
    let mut sources = Vec::new();
    collect_omni_sources_recursive(val, &mut sources);
    sources
}
