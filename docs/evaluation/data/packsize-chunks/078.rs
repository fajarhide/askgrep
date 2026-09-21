fn run_tokens(args: &[String]) -> Result<()> {
    let store = match crate::store::sqlite::Store::open() {
        Ok(s) => s,
        Err(e) => {
            println!("no database yet: {e}");
            return Ok(());
        }
    };
    let (label, since) = super::stats::scope(args);
    print!("{}", tokens_report(&store, label, since)?);
    Ok(())
}

/// "1 call", not "1 calls". Six lines of this report end in a count and a noun,
/// and a window holding one of something is the common case on the short windows.