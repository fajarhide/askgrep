pub fn print_rule() {
    println!("{}", "─".repeat(WIDTH).bright_black().bold());
}

/// The opening frame every report view draws: a rule, `OMNI · view · scope`,
/// a rule.
///
/// One printer rather than one per view, because three of them drifted. The
/// default and detail views drew the rule above and below; `context` still said
/// "OMNI Signal Report: Context"; `projects` and `rerun` drew no rule above at
/// all and folded the period label into a sentence, "OMNI Project Analytics,
/// last 30 days Breakdown" (#717). `view` is `None` for the default report,
/// `scope` is `None` where the view is not windowed.