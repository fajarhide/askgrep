fn condense_agent_report(name: &str, tier: &str, report: &str, detail: bool) -> (String, usize) {
    if detail {
        // Nothing is being lined up in this mode, so the integration's own
        // heading and row widths are left exactly as it wrote them.
        let tier_row = format!("   {:<15} {}\n", "Distill tier:", tier);
        return (format!("{report}{tier_row}"), 0);
    }

    let body: Vec<&str> = report
        .lines()
        .skip_while(|l| l.trim().is_empty())
        .skip(1)
        .collect();
    let hidden = body.iter().filter(|l| l.contains("[OK]")).count();
    let kept = body
        .iter()
        .filter(|l| !l.contains("[OK]") && !l.trim().is_empty());

    // A host with no passing checks is not the same as one with none to run, and
    // "0 checks [OK]" reads like a failure. `Pi` with no config is the case.
    let count = if hidden == 0 {
        String::new()
    } else {
        let checks = if hidden == 1 { "check" } else { "checks" };
        format!("{:<9}{}", format!("{hidden} {checks}"), "[OK]".green())
    };

    // Padded before colouring: `{:<17}` counts the escape bytes otherwise, and
    // every coloured name would sit at its own column.
    let padded = format!("{name:<HOST_NAME_W$}");
    let row = format!("  {}{tier:<HOST_TIER_W$}{count}", padded.cyan());
    // A host with no count would otherwise end in the tier column's padding.
    let mut out = format!("{}\n", row.trim_end());
    // Shifted by one space rather than re-indented from the left, because a
    // report can nest: Pi prints `Duplicates:` at three and a bullet per source
    // at five, and trimming both to a fixed indent made the sources look like
    // siblings of the row that counts them.
    for line in kept {
        out.push_str(&format!(" {line}\n"));
    }
    (out, hidden)
}
