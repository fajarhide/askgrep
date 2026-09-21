//! What askgrep needs from a model, and nothing more.
//!
//! One question, one chunk, one probability. Everything else (prose, reasoning,
//! tool calls) is something askgrep would have to throw away, so it is not in
//! the trait. That keeps a calibrated classifier and a chat model behind the
//! same three methods.

use std::io::Write;
use std::path::PathBuf;

pub trait Backend: Sync {
    /// Probability that the answer to `question` about `code` is yes, 0.0 to 1.0.
    fn score(&self, question: &str, code: &str) -> Result<f32, String>;

    /// Goes into the cache key, so switching model or backend cannot serve back
    /// a score the other one produced.
    fn id(&self) -> String;

    /// Input tokens spent so far. Output is not counted: Jev does not bill it,
    /// and askgrep asks for one token back from a chat model.
    fn tokens(&self) -> usize;

    /// USD per million input tokens, when it is known for certain. `None` prints
    /// no cost rather than a made-up one.
    fn price_per_mtok(&self) -> Option<f64> {
        None
    }
}

/// FNV-1a over backend id, question and chunk. ponytail: a cache key, not a
/// signature. The cost of a collision here is one stale probability.
fn key(id: &str, question: &str, code: &str) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in id
        .as_bytes()
        .iter()
        .chain(b"\0")
        .chain(question.as_bytes())
        .chain(b"\0")
        .chain(code.as_bytes())
    {
        h ^= *b as u64;
        h = h.wrapping_mul(0x1000_0000_01b3);
    }
    format!("{h:016x}")
}

fn cache_dir() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache")))?;
    let d = base.join("askgrep");
    std::fs::create_dir_all(&d).ok()?;
    Some(d)
}

/// Reads a cached score, or asks the backend and stores the answer.
///
/// Returns whether the answer came from cache so the footer can say how much of
/// a sweep was free.
pub fn score_cached(
    backend: &dyn Backend,
    enabled: bool,
    question: &str,
    code: &str,
) -> Result<(f32, bool), String> {
    let path = if enabled {
        cache_dir().map(|d| d.join(key(&backend.id(), question, code)))
    } else {
        None
    };
    if let Some(p) = &path {
        if let Some(v) = std::fs::read_to_string(p)
            .ok()
            .and_then(|s| s.trim().parse::<f32>().ok())
        {
            return Ok((v, true));
        }
    }
    let v = backend.score(question, code)?;
    if let Some(p) = &path {
        // Written through a temp file so a killed run cannot leave half a number
        // behind for the next run to parse.
        let tmp = p.with_extension("tmp");
        if let Ok(mut f) = std::fs::File::create(&tmp) {
            if write!(f, "{v}").is_ok() {
                let _ = std::fs::rename(&tmp, p);
            }
        }
    }
    Ok((v, false))
}

/// Sleeps before retry `attempt`, counting from zero.
pub fn backoff(attempt: u32) {
    std::thread::sleep(std::time::Duration::from_millis(400u64 << attempt));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_cache_key_separates_its_three_parts() {
        // Without separators these would collide pairwise.
        assert_ne!(key("a", "bc", "d"), key("ab", "c", "d"));
        assert_ne!(key("a", "b", "cd"), key("a", "bc", "d"));
        assert_eq!(key("a", "b", "c"), key("a", "b", "c"));
    }

    #[test]
    fn switching_backend_invalidates_the_cache() {
        assert_ne!(key("jev:jev-latest", "q", "code"), key("openai:gpt-4o", "q", "code"));
    }
}
