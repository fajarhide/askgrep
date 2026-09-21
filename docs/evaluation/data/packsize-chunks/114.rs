pub fn print_flags(flags: Flags) {
    let entries: Vec<_> = flags.iter().chain(std::iter::once(&HELP_FLAG)).collect();
    print_flag_group("FLAGS:", &entries);
}

/// One titled group, for a command whose flags read better split up
/// (`omni init` separates its agents from its Claude-specific flags).