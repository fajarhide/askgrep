pub fn has_any(args: &[String], flags: &[&str]) -> bool {
    flags.iter().any(|f| has_flag(args, f))
}

/// Did the caller ask for help?
///
/// `--help`, `-h`, or `help` as the **first argument to the subcommand**. Every
/// command used to test `help` anywhere in argv, so `omni query how do i get
/// help` printed the help page instead of answering, and routing that word
/// through `has_flag` made `help=notes.txt` do it too. Position is what separates
/// the subcommand from its payload.