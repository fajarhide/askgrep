fn find_pi_binary() -> Option<PathBuf> {
    let candidates = ["pi", "pi-cli"];
    for candidate in candidates {
        if let Ok(output) = Command::new(candidate).arg("--version").output()
            && output.status.success()
        {
            return Some(PathBuf::from(candidate));
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Pi settings helpers (read-only)
// ---------------------------------------------------------------------------

/// Return the default global Pi settings path.