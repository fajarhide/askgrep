pub fn print_header(view: Option<&str>, scope: Option<&str>) {
    println!();
    print_rule();
    let title = match view {
        Some(v) => format!("OMNI · {v}"),
        None => "OMNI".to_string(),
    };
    match scope {
        Some(s) => println!(
            " {} {}",
            title.bold().bright_white(),
            format!("· {s}").bright_black()
        ),
        None => println!(" {}", title.bold().bright_white()),
    }
    print_rule();
}

/// A column separator built from the widths it sits under, so a header and its
/// rule cannot disagree.
///
/// `omni stats --detail` carried a five-group separator under a four-column
/// header because the `#` group was copied from the table above it, leaving a
/// 56-column rule under a 43-column header (#463).