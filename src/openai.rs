//! Any OpenAI-compatible chat endpoint: OpenAI, Groq, OpenRouter, vLLM, Ollama.
//!
//! A chat model writes words, not probabilities, so this backend asks for a
//! single token and reads the token distribution instead of the text. The
//! probability of "yes" against "no" at position one is a real distribution the
//! model already computed, which is far better calibrated than asking it to
//! rate its own confidence in prose.
//!
//! An endpoint that serves no logprobs falls back to the one word it wrote, and
//! scores are then only ever 0.0 or 1.0. `--threshold` cannot rank anything in
//! that mode, which `id()` records so those scores never share a cache with the
//! graded ones.

use std::sync::atomic::{AtomicUsize, Ordering};

use serde::Deserialize;

use crate::backend::{backoff, Backend};

#[derive(Deserialize)]
struct TopLogprob {
    token: String,
    logprob: f64,
}

#[derive(Deserialize)]
struct ContentLogprob {
    top_logprobs: Vec<TopLogprob>,
}

#[derive(Deserialize)]
struct Logprobs {
    content: Vec<ContentLogprob>,
}

#[derive(Deserialize)]
struct Message {
    content: Option<String>,
}

#[derive(Deserialize)]
struct Choice {
    message: Message,
    logprobs: Option<Logprobs>,
}

#[derive(Deserialize)]
struct Usage {
    prompt_tokens: u64,
}

#[derive(Deserialize)]
struct Response {
    choices: Vec<Choice>,
    usage: Option<Usage>,
}

pub struct OpenAiCompat {
    token: String,
    model: String,
    base_url: String,
    agent: ureq::Agent,
    tokens: AtomicUsize,
    /// Set once an endpoint has rejected the logprobs fields, so the rest of the
    /// sweep does not pay a failed request per chunk to learn the same thing.
    no_logprobs: std::sync::atomic::AtomicBool,
}

impl OpenAiCompat {
    pub fn new(token: String, model: String, base_url: String, agent: ureq::Agent) -> Self {
        Self {
            token,
            model,
            base_url: base_url.trim_end_matches('/').to_string(),
            agent,
            tokens: AtomicUsize::new(0),
            no_logprobs: std::sync::atomic::AtomicBool::new(false),
        }
    }

    fn body(&self, question: &str, code: &str, logprobs: bool) -> serde_json::Value {
        let mut b = serde_json::json!({
            "model": self.model,
            "temperature": 0,
            "max_tokens": 1,
            "messages": [
                { "role": "system",
                  "content": "You judge one piece of code. Reply with exactly one word: yes or no." },
                { "role": "user",
                  "content": format!(
                      "```\n{code}\n```\n\nStatement: the code above {question}\n\nIs the statement true?") }
            ]
        });
        if logprobs {
            b["logprobs"] = serde_json::json!(true);
            b["top_logprobs"] = serde_json::json!(10);
        }
        b
    }
}

/// P(yes) against P(no) over the first token's distribution.
///
/// Only the two words are counted, and they are renormalised against each other,
/// so probability the model spent on punctuation or a stray capital does not
/// drag a confident yes toward the middle.
fn probability_of_yes(tops: &[TopLogprob]) -> Option<f32> {
    let (mut yes, mut no) = (0.0f64, 0.0f64);
    for t in tops {
        let w = t.token.trim().to_ascii_lowercase();
        if w == "yes" || w == "y" {
            yes += t.logprob.exp();
        } else if w == "no" || w == "n" {
            no += t.logprob.exp();
        }
    }
    if yes + no <= 0.0 {
        return None;
    }
    Some((yes / (yes + no)) as f32)
}

impl Backend for OpenAiCompat {
    fn id(&self) -> String {
        // The mode is part of the key: a binary 1.0 and a graded 1.0 mean
        // different things and must not be served for one another.
        let mode = if self.no_logprobs.load(Ordering::Relaxed) {
            "binary"
        } else {
            "logprobs"
        };
        format!("openai:{}:{}:{mode}", self.base_url, self.model)
    }

    fn tokens(&self) -> usize {
        self.tokens.load(Ordering::Relaxed)
    }

    fn score(&self, question: &str, code: &str) -> Result<f32, String> {
        let url = format!("{}/chat/completions", self.base_url);
        let mut last = String::new();
        for attempt in 0..3 {
            let want_logprobs = !self.no_logprobs.load(Ordering::Relaxed);
            match self
                .agent
                .post(&url)
                .set("Authorization", &format!("Bearer {}", self.token))
                .send_json(self.body(question, code, want_logprobs))
            {
                Ok(resp) => {
                    let parsed: Response =
                        resp.into_json().map_err(|e| format!("bad response: {e}"))?;
                    if let Some(u) = parsed.usage {
                        self.tokens
                            .fetch_add(u.prompt_tokens as usize, Ordering::Relaxed);
                    }
                    let choice = parsed
                        .choices
                        .first()
                        .ok_or("response carried no choices")?;
                    if let Some(p) = choice
                        .logprobs
                        .as_ref()
                        .and_then(|l| l.content.first())
                        .and_then(|c| probability_of_yes(&c.top_logprobs))
                    {
                        return Ok(p);
                    }
                    self.no_logprobs.store(true, Ordering::Relaxed);
                    let said = choice
                        .message
                        .content
                        .as_deref()
                        .unwrap_or("")
                        .trim()
                        .to_ascii_lowercase();
                    return Ok(if said.starts_with('y') { 1.0 } else { 0.0 });
                }
                // A 400 here is usually the endpoint refusing logprobs. Drop them
                // once and let the next attempt answer in binary rather than
                // failing the whole sweep.
                Err(ureq::Error::Status(400, r)) if want_logprobs => {
                    self.no_logprobs.store(true, Ordering::Relaxed);
                    last = format!("HTTP 400: {}", r.into_string().unwrap_or_default().trim());
                }
                Err(ureq::Error::Status(c, _)) if c == 429 || c >= 500 => {
                    last = format!("HTTP {c}");
                    backoff(attempt);
                }
                Err(ureq::Error::Status(c, r)) => {
                    return Err(format!(
                        "HTTP {c}: {}",
                        r.into_string().unwrap_or_default().trim()
                    ))
                }
                Err(e) => {
                    last = e.to_string();
                    backoff(attempt);
                }
            }
        }
        Err(last)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lp(token: &str, p: f64) -> TopLogprob {
        TopLogprob {
            token: token.into(),
            logprob: p.ln(),
        }
    }

    #[test]
    fn reads_the_distribution_not_the_word() {
        let p = probability_of_yes(&[lp("yes", 0.6), lp("no", 0.2)]).unwrap();
        assert!(
            (p - 0.75).abs() < 1e-4,
            "0.6 against 0.2 renormalises to 0.75, got {p}"
        );
    }

    #[test]
    fn mass_spent_elsewhere_does_not_drag_a_confident_answer_down() {
        // Punctuation and a capital take most of the mass; the verdict is still
        // an overwhelming yes and must not read as 0.09.
        let p = probability_of_yes(&[lp("yes", 0.09), lp("\n", 0.8), lp("no", 0.01)]).unwrap();
        assert!(p > 0.85, "expected a confident yes, got {p}");
    }

    #[test]
    fn a_distribution_naming_neither_word_is_no_answer() {
        assert!(probability_of_yes(&[lp("maybe", 0.9)]).is_none());
    }

    #[test]
    fn the_mode_is_part_of_the_cache_key() {
        let c = OpenAiCompat::new(
            "t".into(),
            "m".into(),
            "https://x/v1/".into(),
            ureq::AgentBuilder::new().build(),
        );
        assert_eq!(c.id(), "openai:https://x/v1:m:logprobs");
        c.no_logprobs.store(true, Ordering::Relaxed);
        assert_eq!(
            c.id(),
            "openai:https://x/v1:m:binary",
            "a binary score must not reuse a graded one"
        );
    }

    #[test]
    fn the_question_reads_as_a_sentence() {
        let c = OpenAiCompat::new(
            "t".into(),
            "m".into(),
            "https://x/v1".into(),
            ureq::AgentBuilder::new().build(),
        );
        let b = c.body(
            "writes to disk without checking the error.",
            "fn x() {}",
            true,
        );
        let u = b["messages"][1]["content"].as_str().unwrap();
        assert!(
            u.contains("Statement: the code above writes to disk without checking the error."),
            "{u}"
        );
        assert!(u.contains("fn x() {}"));
        assert_eq!(b["top_logprobs"], 10);
        assert_eq!(c.body("q", "c", false).get("top_logprobs"), None);
    }
}
