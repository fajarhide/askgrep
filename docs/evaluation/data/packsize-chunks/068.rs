fn package_source() -> String {
    std::env::var(PACKAGE_SOURCE_ENV).unwrap_or_else(|_| DEFAULT_PACKAGE_SOURCE.to_string())
}

// ---------------------------------------------------------------------------
// Pi binary detection
// ---------------------------------------------------------------------------

/// Locate the `pi` binary on `PATH`. Returns `None` if not found.