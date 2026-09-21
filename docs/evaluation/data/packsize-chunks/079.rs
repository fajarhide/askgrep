fn counted(n: u64, noun: &str) -> String {
    let n = crate::cli::stats::format_number(n);
    if n == "1" {
        format!("{n} {noun}")
    } else {
        format!("{n} {noun}s")
    }
}

/// Split from the printing so the arithmetic can be driven directly. The defect
/// this whole area keeps producing is a ratio taken over the wrong population, and
/// a test that has to reach through a terminal cannot see one.
pub(crate) fn tokens_report(
    store: &crate::store::sqlite::Store,
    label: &str,
    since: i64,
) -> Result<String> {
    use crate::cli::stats::format_bytes;
    use std::fmt::Write as _;

    let t = store.engine_totals(since)?;
    // What reached OMNI: what the distiller was given, plus what it declined to
    // touch. Not `distilled_output`, which is what survived distillation rather
    // than what was handed back, and reading it as the second is how the first
    // draft of this report mislabelled 3.7 MB.
    let seen = t.distilled_input + t.declined_bytes;
    let mut out = String::new();

    writeln!(out, "\n Where the tokens went · {label}\n")?;

    if t.distilled_calls == 0 && t.declined_calls == 0 && t.folds == 0 {
        writeln!(out, "   nothing recorded yet in this window\n")?;
        return Ok(out);
    }

    writeln!(
        out,
        "   {:<24}{:>10}   {}",
        "tool output OMNI saw",
        format_bytes(seen),
        counted(t.distilled_calls + t.declined_calls, "call")
    )?;
    // One line per engine, each percentage against its own base, exactly as #665
    // settled it for `omni stats`. A single "removed" figure under the total was
    // the first draft and it subtracts across populations: the ledger folds
    // payloads whose bytes are not inside `distilled_input`, so the two may be
    // summed and may never share a denominator.
    let distilled_pct = t
        .distilled_pct()
        .map(|p| format!("{p:.0}% of what it was given"))
        .unwrap_or_else(|| "of what it was given".to_string());
    writeln!(
        out,
        "     {:<22}{:>10}   {:<26} {}",
        "distilled",
        format_bytes(t.distilled_saved()),
        distilled_pct,
        counted(t.distilled_calls, "call")
    )?;
    let fold_pct = t
        .fold_pct()
        .map(|p| format!("{p:.0}% of what it folded"))
        .unwrap_or_else(|| "of what it folded".to_string());
    writeln!(
        out,
        