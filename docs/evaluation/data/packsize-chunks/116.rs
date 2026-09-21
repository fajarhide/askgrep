pub fn flag_name(arg: &str) -> &str {
    arg.split('=').next().unwrap_or(arg)
}

/// Is this flag present, in either accepted spelling?
///
/// `check_flags` validates `--flag=value` on the name alone, so a caller
/// comparing the whole argument accepts the input and then behaves as if the
/// flag were absent. Every command in this CLI had that shape: `omni reset
/// --openclaw=1` dropped into the interactive menu with the plugin installed,
/// and `omni init --openclaw=1` exited 0 having installed nothing (#646).
///
/// Use this rather than `args.iter().any(|a| a == "--flag")`. There is a test in
/// this module that fails on the raw form, because sixty of them is what it took
/// to notice.