mod chunk;
mod jev;

use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use clap::Parser;

/// Dollars per million input tokens, from the published price for jev-1.13.
/// Output tokens are free, so they do not appear here.
const USD_PER_MTOK: f64 = 0.042;

#[derive(Parser)]
#[command(
    name = "askgrep",
    version,
    about = "grep for questions you cannot write as a pattern",
    after_help = "The question is spliced after \"The code in `state` \", so phrase it as a\npredicate: \"builds SQL by concatenating a request parameter.\"\n\nNeeds TYPESAFE_API_KEY. Get one at https://typesafe.ai"
)]
struct Args {
    /// What to look for, phrased as something the code does.
    question: String,

    /// Directory or file to search.
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Report chunks scoring at or above this.
    #[arg(short, long, default_value_t = 0.8)]
    threshold: f32,

    /// Count the chunks and price the sweep without calling the API.
    #[arg(long)]
    dry_run: bool,

    /// Concurrent requests. The published limit is 1,200 per minute.
    #[arg(short, long, default_value_t = 16)]
    jobs: usize,

    /// Ignore the on-disk cache and re-ask every chunk.
    #[arg(long)]
    no_cache: bool,

    /// One JSON object per hit, for piping.
    #[arg(long)]
    json: bool,
}

fn main() -> std::process::ExitCode {
    let args = Args::parse();
    let question = args.question.trim().to_string();

    let chunks = chunk::collect(&args.path, 40, 2_400);
    if chunks.is_empty() {
        eprintln!("askgrep: no readable source under {}", args.path.display());
        return std::process::ExitCode::from(2);
    }

    if args.dry_run {
        // Four bytes per token is the usual rule of thumb, plus roughly 60 tokens
        // of question and criteria on every request.
        let est: f64 = chunks.iter().map(|c| c.text.len() as f64 / 4.0 + 60.0).sum();
        println!("{} chunks", chunks.len());
        println!("~{:.0} input tokens", est);
        println!("~${:.4} at ${USD_PER_MTOK}/Mtok", est / 1e6 * USD_PER_MTOK);
        println!("\nCached answers are free, so a re-run over unchanged files costs nothing.");
        return std::process::ExitCode::SUCCESS;
    }

    let Ok(token) = std::env::var("TYPESAFE_API_KEY") else {
        eprintln!("askgrep: TYPESAFE_API_KEY is not set.");
        eprintln!("         export TYPESAFE_API_KEY=... and try again.");
        eprintln!("         Run with --dry-run to size a sweep without a key.");
        return std::process::ExitCode::from(2);
    };

    let client = jev::Client::new(token, !args.no_cache);
    let next = AtomicUsize::new(0);
    let results = std::sync::Mutex::new(Vec::new());
    let failed = std::sync::Mutex::new(Vec::new());

    std::thread::scope(|s| {
        for _ in 0..args.jobs.clamp(1, 64) {
            s.spawn(|| loop {
                let i = next.fetch_add(1, Ordering::Relaxed);
                let Some(c) = chunks.get(i) else { return };
                match client.score(&question, &c.text) {
                    Ok(p) if p >= args.threshold => results.lock().unwrap().push((p, c)),
                    Ok(_) => {}
                    Err(e) => failed.lock().unwrap().push((c.file.clone(), e)),
                }
            });
        }
    });

    let mut hits = results.into_inner().unwrap();
    hits.sort_by(|a, b| b.0.total_cmp(&a.0));

    // Print paths the way the user typed them, like ripgrep, so a hit can be
    // pasted straight back into an editor.
    let show = |p: &std::path::Path| -> String {
        p.strip_prefix(&args.path).unwrap_or(p).display().to_string()
    };

    for (p, c) in &hits {
        if args.json {
            println!(
                "{}",
                serde_json::json!({
                    "file": show(&c.file), "line": c.line, "score": p, "head": c.head()
                })
            );
        } else {
            println!("{}:{}  {p:.2}  {}", show(&c.file), c.line, c.head());
        }
    }

    let errs = failed.into_inner().unwrap();
    if !args.json {
        let tokens = client.tokens.load(Ordering::Relaxed);
        let cached = client.hits.load(Ordering::Relaxed);
        eprintln!(
            "\n{} hit{} in {} chunks  |  {} cached  |  {tokens} tokens  |  ${:.4}",
            hits.len(),
            if hits.len() == 1 { "" } else { "s" },
            chunks.len(),
            cached,
            tokens as f64 / 1e6 * USD_PER_MTOK
        );
    }
    if !errs.is_empty() {
        // Say how many chunks were never scored. A silent partial sweep would
        // look exactly like a clean one, which is the worst way to be wrong.
        eprintln!("{} chunk(s) failed, so this sweep is incomplete:", errs.len());
        for (f, e) in errs.iter().take(3) {
            eprintln!("  {}: {e}", f.display());
        }
        return std::process::ExitCode::from(1);
    }
    std::process::ExitCode::SUCCESS
}
