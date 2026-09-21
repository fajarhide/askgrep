fn configured_compression_in_config(config_path: &Path) -> bool {
    fs::read_to_string(config_path)
        .map(|config| configured_compression(&config))
        .unwrap_or(false)
}

/// The plugin source with the binary path substituted in.
///
/// The path goes in as a JSON string rather than raw text. JSON string syntax is
/// a subset of Python's, so this escapes the two characters that would otherwise
/// produce a file Python cannot read: a quote closes the literal, and a Windows
/// path's `\U` is a unicode escape (`C:\Users\…` is a syntax error, not a path).