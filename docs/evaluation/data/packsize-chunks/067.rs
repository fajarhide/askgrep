pub fn capture<T>(f: impl FnOnce() -> T) -> (T, String) {
    let previous = SINK.with(|s| s.borrow_mut().replace(String::new()));
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
    let captured = SINK.with(|s| s.borrow_mut().take()).unwrap_or_default();
    SINK.with(|s| *s.borrow_mut() = previous);

    match result {
        Ok(value) => (value, captured),
        Err(payload) => std::panic::resume_unwind(payload),
    }
}

/// Emits one line of a `doctor` report. Same arguments as `println!`.
#[macro_export]
macro_rules! agent_report {
    () => { $crate::agents::output::emit_line(format_args!("")) };
    ($($arg:tt)*) => { $crate::agents::output::emit_line(format_args!($($arg)*)) };
}

#[cfg(test)]
mod tests {
    use super::*;

    /// #353: `omni doctor --json | jq .` failed on line 1 because the agent
    /// reports were already on stdout by the time the document was written.
    #[test]
    fn keeps_the_report_out_of_stdout_while_capturing() {
        let (returned, captured) = capture(|| {
            agent_report!("Claude Code:");
            agent_report!("  hooks {}", "[OK]");
            true
        });

        assert!(returned);
        assert_eq!(captured, "Claude Code:\n  hooks [OK]\n");
    }

    /// Capturing is scoped: once it ends the report goes back to stdout, so a
    /// later human-facing `doctor` in the same process still prints.
    #[test]
    fn stops_capturing_once_the_scope_ends() {
        let (_, first) = capture(|| agent_report!("inside"));
        let (_, second) = capture(|| ());

        assert_eq!(first, "inside\n");
        assert!(second.is_empty(), "second capture saw {second:?}");
    }

    /// A panicking integration must not leave the sink installed, or every
    /// later report in the process disappears.
    #[test]
    fn restores_the_sink_when_the_body_panics() {
        let panicked = std::panic::catch_unwind(|| capture(|| panic!("boom")));
        assert!(panicked.is_err());

        let (_, after) = capture(|| agent_report!("still captured"));
        assert_eq!(after, "still captured\n");
    }
}
