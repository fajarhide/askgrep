fn aliases(spec: &str) -> impl Iterator<Item = &str> {
    spec.split(',')
        .filter_map(|part| part.split_whitespace().next())
}

/// The trailing entry every subcommand shares.
pub const HELP_FLAG: (&str, &str) = ("--help, -h", "Show this help message");

/// Render the `FLAGS:` block of a subcommand's help, `--help` included.