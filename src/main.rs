mod backend;
mod chunk;
mod jev;
mod openai;

use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use clap::{Parser, ValueEnum};

use backend::Backend;

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum Which {
    /// TypeSafe System One. Returns a calibrated probability directly.
    Jev,
    /// Any OpenAI-compatible chat endpoint, scored from first-token logprobs.
    Openai,
}

#[derive(Parser)]
#[command(
    name = "askgrep",
    version,
    about = "grep for questions you cannot write as a pattern",
    after_help = "The question is spliced after \"the code ...\", so phrase it as something\nthe code does: \"builds SQL by concatenating a request parameter.\"\n\nBackends read TYPESAFE_API_KEY or OPENAI_API_KEY. --dry-run needs neither."
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

    /// Which model answers. Default: jev if TYPESAFE_API_KEY is set, else openai.
    #[arg(short, long)]
    backend: Option<Which>,

    /// Model name for the chosen backend.
    #[arg(short, long)]
    model: Option<String>,

    /// OpenAI-compatible base URL, for Groq, OpenRouter, vLLM or Ollama.
    #[arg(long, env = "ASKGREP_BASE_URL", default_value = "https://api.openai.com/v1")]
    base_url: String,

    /// Count the chunks and price the sweep without calling anything.
    #[arg(long)]
    dry_run: bool,

    /// Concurrent requests.
    #[arg(short, long, default_value_t = 16)]
    jobs: usize,

    /// Ignore the on-disk cache and re-ask every chunk.
    #[arg(long)]
    no_cache: bool,

    /// One JSON object per hit, for piping.
    #[arg(long)]
    json: bool,
}

/// Picks the backend and reads its key, or explains what is missing.
fn build(args: &Args) -> Result<Box<dyn Backend>, String> {
    let ts = std::env::var("TYPESAFE_API_KEY").ok().filter(|k| !k.is_empty());
    let oa = std::env::var("OPENAI_API_KEY").ok().filter(|k| !k.is_empty());
    let which = match args.backend {
        Some(w) => w,
        None if ts.is_some() => Which::Jev,
        None if oa.is_some() => Which::Openai,
        None => {
            return Err("no key found. Set TYPESAFE_API_KEY or OPENAI_API_KEY.\n       \
                        Run with --dry-run to size a sweep without one."
                .into())
        }
    };
    let agent = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(60))
        .build();
    Ok(match which {
        Which::Jev => Box::new(jev::Jev::new(
            ts.ok_or("backend jev needs TYPESAFE_API_KEY")?,
            args.model.clone().unwrap_or_else(|| jev::DEFAULT_MODEL.into()),
            agent,
        )),
        Which::Openai => Box::new(openai::OpenAiCompat::new(
            // A local endpoint (Ollama, vLLM) accepts any key, so an empty one
            // is not an error here, only a missing OPENAI_API_KEY for a remote.
            oa.unwrap_or_default(),
            args.model.clone().unwrap_or_else(|| "gpt-4o-mini".into()),
            args.base_url.clone(),
            agent,
        )),
    })
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
        // of instructions on every request.
        let est: f64 = chunks.iter().map(|c| c.text.len() as f64 / 4.0 + 60.0).sum();
        println!("{} chunks", chunks.len());
        println!("~{est:.0} input tokens");
        println!("~${:.4} on jev at $0.042/Mtok", est / 1e6 * 0.042);
        println!("\nCached answers are free, so a re-run over unchanged files costs nothing.");
        return std::process::ExitCode::SUCCESS;
    }

    let backend = match build(&args) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("askgrep: {e}");
            return std::process::ExitCode::from(2);
        }
    };

    let next = AtomicUsize::new(0);
    let cached = AtomicUsize::new(0);
    let results = std::sync::Mutex::new(Vec::new());
    let failed = std::sync::Mutex::new(Vec::new());

    std::thread::scope(|s| {
        for _ in 0..args.jobs.clamp(1, 64) {
            s.spawn(|| loop {
                let i = next.fetch_add(1, Ordering::Relaxed);
                let Some(c) = chunks.get(i) else { return };
                match backend::score_cached(backend.as_ref(), !args.no_cache, &question, &c.text) {
                    Ok((p, hit)) => {
                        if hit {
                            cached.fetch_add(1, Ordering::Relaxed);
                        }
                        if p >= args.threshold {
                            results.lock().unwrap().push((p, c));
                        }
                    }
                    Err(e) => failed.lock().unwrap().push((c.file.clone(), e)),
                }
            });
        }
    });

    let mut hits = results.into_inner().unwrap();
    hits.sort_by(|a, b| b.0.total_cmp(&a.0));

    // Print paths the way they were typed, like ripgrep, so a hit pastes back
    // into an editor.
    // Stripping a file path against itself leaves nothing, and a hit that prints
    // as `:1` cannot be pasted anywhere.
    let show = |p: &std::path::Path| -> String {
        let rel = p.strip_prefix(&args.path).unwrap_or(p);
        if rel.as_os_str().is_empty() { p.display().to_string() } else { rel.display().to_string() }
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
        let tokens = backend.tokens();
        let cost = match backend.price_per_mtok() {
            Some(r) => format!("  |  ${:.4}", tokens as f64 / 1e6 * r),
            None => String::new(),
        };
        eprintln!(
            "\n{} hit{} in {} chunks  |  {} cached  |  {tokens} tokens{cost}",
            hits.len(),
            if hits.len() == 1 { "" } else { "s" },
            chunks.len(),
            cached.load(Ordering::Relaxed),
        );
    }
    if !errs.is_empty() {
        // Say how many chunks were never scored. A silent partial sweep looks
        // exactly like a clean one, which is the worst way to be wrong.
        eprintln!("{} chunk(s) failed, so this sweep is incomplete:", errs.len());
        for (f, e) in errs.iter().take(3) {
            eprintln!("  {}: {e}", f.display());
        }
        return std::process::ExitCode::from(1);
    }
    std::process::ExitCode::SUCCESS
}
