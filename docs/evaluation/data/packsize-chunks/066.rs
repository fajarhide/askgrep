pub fn emit_line(args: std::fmt::Arguments<'_>) {
    SINK.with(|sink| {
        let mut sink = sink.borrow_mut();
        match sink.as_mut() {
            Some(buf) => {
                use std::fmt::Write;
                // A formatting failure here would mean losing a diagnostic line,
                // never the document being built, so it cannot fail the run.
                let _ = writeln!(buf, "{}", args);
            }
            None => println!("{}", args),
        }
    });
}

/// Runs `f` with the report diverted away from stdout, returning its value and
/// whatever it would have printed.
///
/// Restores the previous sink even if `f` panics, because `doctor --fix` calls
/// into installers and a poisoned sink would silence every later report in the
/// process.