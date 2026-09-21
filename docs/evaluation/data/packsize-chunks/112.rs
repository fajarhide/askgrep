pub fn column_rule(widths: &[usize]) -> String {
    widths
        .iter()
        .map(|w| "─".repeat(*w))
        .collect::<Vec<_>>()
        .join(" ")
}

/// The flags one subcommand accepts, as `(spec, description)`, where `spec` is
/// the flag and any aliases exactly as help should show them: `"--today, -d"`.
///
/// Both the help printer and the argument check read this one list, so a flag
/// cannot be accepted without being documented or documented without being
/// accepted, the drift that made `omni stats`'s own footer advertise a
/// `--detail` its `--help` never mentioned (#151).
pub type Flags = &'static [(&'static str, &'static str)];

/// The individual flags of a `(spec, _)` entry, with any value placeholder
/// dropped: `"--today, -d"` → `--today`, `-d`; `"--validate <file.toml>"` →
/// `--validate`.