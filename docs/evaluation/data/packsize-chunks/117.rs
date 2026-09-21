pub fn has_flag(args: &[String], flag: &str) -> bool {
    args.iter().any(|a| flag_name(a) == flag)
}

/// Is any of these spellings present? For a flag that grew alternatives.
///
/// `stats` had its own copy of this comparing whole arguments, which is how
/// `--hour=1` survived the first pass of #646: the source scan below looks for a
/// flag literal, and a helper comparing against a `&[&str]` parameter has none.
/// Deleting that copy is the fix, not widening the scan.