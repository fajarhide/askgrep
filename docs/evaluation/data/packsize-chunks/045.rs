fn append_top_level_block(config: &str, block: &str) -> String {
    let mut out = String::with_capacity(config.len() + block.len() + 1);
    out.push_str(config);
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(block);
    out
}
