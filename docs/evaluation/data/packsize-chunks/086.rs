fn page(store: &Store) -> String {
    let (sessions, median, longest, compacted) = store.session_lifetime(0);
    let periods = store.multi_period_stats().unwrap_or_default();
    let reasons = store.passthrough_reasons(0);

    let lifetime = if sessions == 0 {
        "<p class=\"muted\">Not measurable yet: no session has been closed by a host that reports one.</p>".to_string()
    } else {
        format!(
            "<p class=\"big\">{median}</p><p class=\"muted\">commands in the median closed session, {longest} in the longest, across {sessions} sessions. {compacted} ended at a compaction.</p>"
        )
    };

    let mut rows = String::new();
    for (label, calls, _input, _output, raw_tokens, filtered_tokens) in &periods {
        if *calls == 0 && label != "All Time" {
            continue;
        }
        let pct = if *raw_tokens > 0 {
            100.0 * (1.0 - *filtered_tokens as f64 / *raw_tokens as f64)
        } else {
            0.0
        };
        // `216444` was reaching the page raw while the CLI printed `216K` from
        // this very function, which is public and two files away (#463).
        rows.push_str(&format!(
            "<tr><td>{}</td><td class=\"n\">{calls}</td><td class=\"n\">{}</td><td class=\"n\">{}</td><td class=\"n\">{pct:.1}%</td></tr>",
            escape(label),
            super::stats::format_compact(*raw_tokens),
            super::stats::format_compact(*filtered_tokens)
        ));
    }

    // Top Commands and Agent Distribution, from the same two calls that feed the
    // CLI tables, so the footer's "same figures as omni stats" stays true.
    let mut commands = String::new();
    if let Ok(raw) = store.filter_breakdown(0) {
        for (name, calls, pct, _) in super::stats::group_and_calculate_stats(raw, 0)
            .iter()
            .filter(|(_, _, pct, _)| *pct > 0.0)
            .take(10)
        {
            commands.push_str(&format!(
                "<tr><td>{}</td><td class=\"n\">{calls}</td><td class=\"n\">{pct:.1}%</td></tr>",
                escape(name)
            ));
        }
    }
    if commands.is_empty() {
        commands.push_str(
            "<tr><td class=\"muted\" colspan=\"3\">no command has saved anything yet</td></tr>",
        );
    }

    let mut agents = String::new();
    if let Ok(rows) = store.get_agent_breakdown(0) {
        let total: u64 = rows.iter().map(|r| 