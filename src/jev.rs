//! One question, one chunk, one request.
//!
//! Packing several chunks into a single `state` is cheaper and measurably worse.
//! On 120 Rust functions against a regex ground truth, precision at p>=0.8 fell
//! from 0.85 at one chunk per request to 0.43 at twenty, and recall at p>=0.5
//! from 1.00 to 0.79. The saving was 1.7x. It is not worth it, so this file does
//! not offer the option.

use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use serde::Deserialize;

const ENDPOINT: &str = "https://api.typesafe.ai/v1/systemone";
pub const MODEL: &str = "jev-latest";

#[derive(Deserialize)]
struct Answer {
    noul: f32,
}

#[derive(Deserialize)]
struct Response {
    answers: std::collections::HashMap<String, Answer>,
    usage: Usage,
}

#[derive(Deserialize)]
pub struct Usage {
    pub input_tokens: u64,
}

fn body(question: &str, code: &str) -> serde_json::Value {
    serde_json::json!({
        "model": MODEL,
        "state": code,
        "questions": { "q": {
            "type": "noul",
            "instructions": format!("The code in `state` {question}"),
            "criteria": {
                "true": "The code does this.",
                "false": "The code does not do this."
            }
        }}
    })
}

/// FNV-1a over question and chunk. ponytail: a cache key, not a signature. If a
/// collision ever matters here, the cost of being wrong is one stale probability.
fn key(question: &str, code: &str) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in question.as_bytes().iter().chain(b"\0").chain(code.as_bytes()) {
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

pub struct Client {
    token: String,
    agent: ureq::Agent,
    pub cache: bool,
    pub tokens: AtomicUsize,
    pub hits: AtomicUsize,
}

impl Client {
    pub fn new(token: String, cache: bool) -> Self {
        Self {
            token,
            agent: ureq::AgentBuilder::new()
                .timeout(std::time::Duration::from_secs(60))
                .build(),
            cache,
            tokens: AtomicUsize::new(0),
            hits: AtomicUsize::new(0),
        }
    }

    pub fn score(&self, question: &str, code: &str) -> Result<f32, String> {
        let k = key(question, code);
        let path = if self.cache { cache_dir().map(|d| d.join(&k)) } else { None };
        if let Some(p) = &path {
            if let Ok(s) = std::fs::read_to_string(p) {
                if let Ok(v) = s.trim().parse::<f32>() {
                    self.hits.fetch_add(1, Ordering::Relaxed);
                    return Ok(v);
                }
            }
        }

        let mut last = String::new();
        for attempt in 0..3 {
            match self
                .agent
                .post(ENDPOINT)
                .set("Authorization", &format!("Bearer {}", self.token))
                .send_json(body(question, code))
            {
                Ok(resp) => {
                    let parsed: Response =
                        resp.into_json().map_err(|e| format!("bad response: {e}"))?;
                    self.tokens
                        .fetch_add(parsed.usage.input_tokens as usize, Ordering::Relaxed);
                    let v = parsed
                        .answers
                        .get("q")
                        .ok_or("response carried no answer for the question")?
                        .noul;
                    if let Some(p) = &path {
                        // Write through a temp file so a killed run cannot leave a
                        // half-written probability behind for the next one to read.
                        let tmp = p.with_extension("tmp");
                        if let Ok(mut f) = std::fs::File::create(&tmp) {
                            if write!(f, "{v}").is_ok() {
                                let _ = std::fs::rename(&tmp, p);
                            }
                        }
                    }
                    return Ok(v);
                }
                // 429 is the documented rate-limit code; the SDKs back off, so do we.
                Err(ureq::Error::Status(code, _)) if code == 429 || code >= 500 => {
                    last = format!("HTTP {code}");
                    std::thread::sleep(std::time::Duration::from_millis(400 << attempt));
                }
                Err(ureq::Error::Status(code, r)) => {
                    let detail = r.into_string().unwrap_or_default();
                    return Err(format!("HTTP {code}: {}", detail.trim()));
                }
                Err(e) => {
                    last = e.to_string();
                    std::thread::sleep(std::time::Duration::from_millis(400 << attempt));
                }
            }
        }
        Err(last)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_cache_key_separates_question_from_code() {
        // Without the separator, ("ab", "c") and ("a", "bc") would collide.
        assert_ne!(key("ab", "c"), key("a", "bc"));
        assert_eq!(key("q", "code"), key("q", "code"));
        assert_ne!(key("q", "code"), key("q", "codf"));
    }

    #[test]
    fn the_question_is_spliced_into_a_sentence_about_the_state() {
        let b = body("writes to disk without checking the error.", "fn x() {}");
        let i = b["questions"]["q"]["instructions"].as_str().unwrap();
        assert_eq!(i, "The code in `state` writes to disk without checking the error.");
        assert_eq!(b["state"], "fn x() {}");
        assert_eq!(b["questions"]["q"]["type"], "noul");
    }
}
