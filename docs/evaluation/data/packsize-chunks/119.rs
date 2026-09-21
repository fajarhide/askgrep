pub fn wants_help(args: &[String]) -> bool {
    has_flag(args, "--help") || has_flag(args, "-h") || args.get(2).is_some_and(|a| a == "help")
}

/// The value given to a flag, as `--flag value` or `--flag=value`.
///
/// The `=` form used to be accepted by `check_flags` and then never found, so
/// `omni dashboard --port=8080` bound the default port and said nothing, and
/// `omni patterns --tool=bash` filtered on nothing. Silently doing something
/// other than what the argument said is worse here than for a boolean flag: the
/// command succeeds and the answer is about something else.