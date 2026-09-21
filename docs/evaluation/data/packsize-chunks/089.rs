fn truncate_lines(s: &str, max_lines: usize) -> Vec<String> {
    let lines: Vec<&str> = s.lines().collect();
    let mut result = Vec::new();

    for &line in lines.iter().take(max_lines) {
        let truncated = crate::util::text::display_truncate_with_ellipsis(line, super::WIDTH - 5);
        result.push(truncated);
    }

    if lines.len() > max_lines {
        result.push(format!(
            "{} ... ({} more lines) ...",
            "---".bright_black(),
            lines.len() - max_lines
        ));
    }

    result
}
