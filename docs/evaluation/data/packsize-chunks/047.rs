fn configured_compression(config: &str) -> bool {
    let has_compression = config.contains("compression:");
    let has_enabled = config.contains("enabled: true") || config.contains("enabled:true");
    has_compression && has_enabled
}

/// Drops one `[section]` and its keys from a TOML document.
///
/// Line based on purpose: `omni_config.toml` is hand-edited and round-tripping
/// it through a parser would reformat everything a user wrote around our
/// section. The block ends at the next `[` in column zero.