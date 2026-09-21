//! TypeSafe System One. Returns a calibrated probability directly, which is the
//! shape askgrep wants, so this backend does no conversion.
//!
//! One chunk per request, deliberately. Packing several into one `state` is
//! cheaper and measurably worse: on 120 Rust functions scored against a regex
//! that answers the same question exactly, precision at p>=0.8 fell from 0.85 at
//! one chunk per request to 0.43 at twenty, and recall at p>=0.5 from 1.00 to
//! 0.79, for a 1.7x saving. So the option is not offered.

use std::sync::atomic::{AtomicUsize, Ordering};

use serde::Deserialize;

use crate::backend::{backoff, Backend};

const ENDPOINT: &str = "https://api.typesafe.ai/v1/systemone";
pub const DEFAULT_MODEL: &str = "jev-latest";

/// Published price for jev-1.13 input tokens. Output is free.
const USD_PER_MTOK: f64 = 0.042;

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
struct Usage {
    input_tokens: u64,
}

pub struct Jev {
    token: String,
    model: String,
    agent: ureq::Agent,
    tokens: AtomicUsize,
}

impl Jev {
    pub fn new(token: String, model: String, agent: ureq::Agent) -> Self {
        Self { token, model, agent, tokens: AtomicUsize::new(0) }
    }

    fn body(&self, question: &str, code: &str) -> serde_json::Value {
        serde_json::json!({
            "model": self.model,
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
}

impl Backend for Jev {
    fn id(&self) -> String {
        format!("jev:{}", self.model)
    }

    fn tokens(&self) -> usize {
        self.tokens.load(Ordering::Relaxed)
    }

    fn price_per_mtok(&self) -> Option<f64> {
        Some(USD_PER_MTOK)
    }

    fn score(&self, question: &str, code: &str) -> Result<f32, String> {
        let mut last = String::new();
        for attempt in 0..3 {
            match self
                .agent
                .post(ENDPOINT)
                .set("Authorization", &format!("Bearer {}", self.token))
                .send_json(self.body(question, code))
            {
                Ok(resp) => {
                    let parsed: Response =
                        resp.into_json().map_err(|e| format!("bad response: {e}"))?;
                    self.tokens
                        .fetch_add(parsed.usage.input_tokens as usize, Ordering::Relaxed);
                    return Ok(parsed
                        .answers
                        .get("q")
                        .ok_or("response carried no answer for the question")?
                        .noul);
                }
                // 429 is the documented rate-limit code.
                Err(ureq::Error::Status(c, _)) if c == 429 || c >= 500 => {
                    last = format!("HTTP {c}");
                    backoff(attempt);
                }
                Err(ureq::Error::Status(c, r)) => {
                    return Err(format!("HTTP {c}: {}", r.into_string().unwrap_or_default().trim()))
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

    #[test]
    fn the_question_is_spliced_into_a_sentence_about_the_state() {
        let j = Jev::new("t".into(), "jev-latest".into(), ureq::AgentBuilder::new().build());
        let b = j.body("writes to disk without checking the error.", "fn x() {}");
        assert_eq!(
            b["questions"]["q"]["instructions"],
            "The code in `state` writes to disk without checking the error."
        );
        assert_eq!(b["state"], "fn x() {}");
        assert_eq!(b["questions"]["q"]["type"], "noul");
        assert_eq!(j.id(), "jev:jev-latest");
    }
}
