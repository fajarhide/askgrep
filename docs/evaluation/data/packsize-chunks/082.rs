pub fn run(args: &[String], store: &Store) -> Result<()> {
    if super::has_flag(args, "--help") || super::has_flag(args, "-h") {
        print_help();
        return Ok(());
    }
    super::check_flags("dashboard", args, FLAGS)?;

    // `--port=8080` used to bind `DEFAULT_PORT` and say nothing: `check_flags`
    // accepted the argument and the lookup below never matched it (#646).
    let port = super::flag_value(args, "--port")
        .and_then(|v| v.parse::<u16>().ok())
        .unwrap_or(DEFAULT_PORT);

    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, port))?;
    println!(
        "\n  {} http://127.0.0.1:{port}",
        "OMNI dashboard".bold().bright_white()
    );
    println!("  {}\n", "Ctrl-C to stop".bright_black());

    for stream in listener.incoming() {
        let Ok(stream) = stream else { continue };
        // One connection at a time on purpose: this is one reader looking at
        // their own machine, and a thread pool would be machinery for a load
        // that does not exist.
        if let Err(e) = serve(stream, store) {
            eprintln!("  {} {e}", "request failed:".bright_black());
        }
    }
    Ok(())
}
