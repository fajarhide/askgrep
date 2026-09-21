fn print_help() {
    println!(
        "\n{} {}: A local dashboard over the numbers omni stats prints",
        "omni".bold().cyan(),
        "dashboard".bold().yellow()
    );
    println!("\n{}", "USAGE:".bold().bright_white());
    println!("  omni {} {}", "dashboard".cyan(), "[FLAGS]".bright_black());
    println!();
    super::print_flags(FLAGS);
    println!("Binds 127.0.0.1 only. Ctrl-C to stop.\n");
}
