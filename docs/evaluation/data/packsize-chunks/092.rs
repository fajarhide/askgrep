fn format_time_ago(ts: u64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    if ts >= now {
        return "just now".to_string();
    }
    let diff = now - ts;
    if diff < 60 {
        format!("{} seconds ago", diff)
    } else if diff < 3600 {
        format!("{} minutes ago", diff / 60)
    } else if diff < 86400 {
        format!("{} hours ago", diff / 3600)
    } else {
        format!("{} days ago", diff / 86400)
    }
}

#[derive(serde::Serialize)]
pub struct DoctorJson {
    pub version: String,
    pub healthy: bool,
    pub checks: Vec<DoctorCheck>,
    pub fix_available: bool,
    /// What the per-agent reports said that a check name cannot carry.
    ///
    /// These were collected and dropped. A Codex install whose hooks are
    /// installed but awaiting review (#367) pushes its warning here and nowhere
    /// else, so `--json` reported `hooks: installed`, `healthy: true`, and gave a
    /// program no way to learn the hooks were being skipped.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
}

#[derive(serde::Serialize)]
pub struct DoctorCheck {
    pub name: String,
    pub ok: bool,
    pub message: String,
}

/// Changelog entries this build carries that no release contains (#137).
///
/// Counted by `build.rs` from the `## [Unreleased]` section of the tree the
/// binary was compiled from, so a properly cut release reports 0 and says
/// nothing. Parsing cannot fail into a false alarm: an unreadable or malformed
/// value means "nothing to report", never "something is wrong".