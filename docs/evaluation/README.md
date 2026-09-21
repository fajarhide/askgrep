# What was measured, and where the method broke

Two experiments on the same 40 Rust functions, taken from a real codebase. Both
compare a calibrated classifier (Jev) against a chat model (Claude, via four
independent judges that never saw each other's answers or the answer key).

Everything here replays from `data/`. The second experiment's numbers should not
be quoted, and the reason is the most useful thing on this page.

## Experiment 0: how many chunks per request

**Question:** does this chunk contain a call to `.unwrap()` or `.expect(...)`?
Same regex ground truth as experiment 1, on 120 pinned chunks with 14 positives.

Batching is the obvious saving. TypeSafe measured it and published the result:
many questions about one shared document in a single call is 12.2x cheaper, 10x
faster, and changes no answer. askgrep cannot use that shape, because its
questions are about different functions, so batching means packing unrelated
chunks into one `state` array and pointing each question at an index.

| chunks per request | precision @0.8 | recall @0.5 | cost | seconds |
| --- | --- | --- | --- | --- |
| 1 | 1.00 / 1.00 | 1.00 / 1.00 | $0.0029 | 161 |
| 5 | 0.60 / 0.71 | 0.93 / 0.93 | $0.0019 | 30 |
| 20 | 0.45 / 0.48 | 0.86 / 0.79 | $0.0017 | 10 |

Two runs of each. Between repeats the probabilities move 0.012 (pack 1) to
0.036 (pack 20) on average, and at most 3 of 120 chunks cross the 0.8 threshold,
so the gap between one and twenty is not run-to-run noise.

Twenty per request is 1.7x cheaper and less than half of its hits are right.
Latency is not the reason to batch either: one chunk per request takes 161
seconds sequentially, and 10 seconds at 64 concurrent.

The chunks are committed in `data/packsize-chunks/`. They were not, the first
time this was measured, and the table then read 0.85 / 0.65 / 0.43 because the
source tree it sampled had moved between runs. The direction held; the numbers
did not.

## Experiment 1: a question a regex can answer

**Question:** does this chunk contain a call to `.unwrap()` or `.expect(...)`?

Ground truth is the regex `\.unwrap\(\)|\.expect\(`. It cannot take sides, which
is the whole reason this question was chosen first. 9 of 40 chunks are positive.

| | precision | recall |
| --- | --- | --- |
| Claude, yes/no | 1.00 | 1.00 |
| Jev at 0.5 | 1.00 | 1.00 |
| Jev at 0.8 | 1.00 | 1.00 |

Both are perfect. Neither was fooled by `.unwrap_or_else(`, `.unwrap_or(false)`,
or an `.expect("json")` buried inside a `#[cfg(test)]` block.

The difference is in the number each returns:

| | on true positives | on negatives | ranges overlap |
| --- | --- | --- | --- |
| Claude confidence | 97.9 (95 to 99) | 94.5 (80 to 99) | **yes** |
| Jev probability | 0.92 (0.80 to 0.99) | 0.10 (0.05 to 0.27) | **no** |

Claude's self-reported confidence is flat. A negative scored 99 and a positive
scored 95, so no threshold separates them and the number cannot rank anything.
Jev leaves an empty band between 0.27 and 0.80; any cut inside it is perfect.

That result is what `--threshold` and score-sorted output in askgrep are for.

## Experiment 2: a question a regex cannot answer

**Question:** does this code handle a failure or an absent value and continue
with a default, empty or fallback value, without logging, returning or
propagating it?

There is no mechanical ground truth for this, so the labels in
`data/labels-swallow.json` were written by hand and committed before either
model ran. Four chunks were flagged borderline at labelling time.

| | precision | recall |
| --- | --- | --- |
| Claude, yes/no | 0.95 | 0.95 |
| Jev at 0.5 | 0.71 | 0.95 |
| Jev at 0.8 | 0.78 | 0.67 |

Jev's clean separation does not survive: positives run 0.46 to 0.94, negatives
0.03 to 0.84, and they overlap. Claude's confidence, useless in experiment 1,
becomes informative here: it averages 81.0 where Claude was right and 58.5 where
it was wrong, and both of its errors sit at its two lowest scores.

## Why those numbers should not be quoted

**The labeller and one of the models are the same kind of thing.** These labels
were written by a Claude model. The Claude judges agreed with them 38 times out
of 40. That number cannot distinguish "Claude is accurate" from "two instances of
one model family share a prior", and nothing in this setup separates them.

**The disagreements point at the labels, not at Jev.** Jev scored 0.81 to 0.84 on
three chunks the labels call negative, and all three are the same shape:

```rust
if let Some(obj) = val.as_object_mut()
    && let Some(servers) = obj.get_mut("mcpServers").and_then(|v| v.as_object_mut())
{
    servers.remove("omni");
}
```

If the JSON is not the expected shape, this silently does nothing. Read against
the question as written, that is swallowing a failure, and Jev is probably right.
The labels were drawn on "is there an explicit fallback value", which is a
narrower question than the one that was asked. So some of Jev's eight false
positives are label errors, and 0.71 measures agreement with one person rather
than correctness.

A benchmark whose answer key comes from the same model family as one of its
subjects is not a benchmark. Fixing it needs a key built by something outside
that family, or several labellers with disagreements adjudicated in the open.

## What survives

**Question shape decides the outcome.** Lexical: both perfect, Jev's score
separates cleanly, Claude's confidence is flat. Semantic: both wobble, and they
wobble differently. Measuring one and concluding about the other is the mistake
this page exists to record.

**A chat model's confidence is not consistently useful.** Flat and worthless on
the easy question, informative on the hard one. That is the opposite of what a
single experiment would have suggested, in whichever direction it ran.

**The disagreements were worth more than the scores.** Ten of them exposed a real
ambiguity in the question itself, which no aggregate would have shown.

## Scope

40 chunks from one Rust codebase, one model each side, one session. Claude ran as
Claude Code subagents rather than through the API, so no token or latency figure
from that arm is comparable. Jev cost $0.0009 and $0.0010 for the two runs.

## Files

| | |
| --- | --- |
| `data/chunks/` | the 40 chunks, exactly as both models saw them |
| `data/labels-unwrap.json` | regex ground truth, experiment 1 |
| `data/labels-swallow.json` | hand labels, experiment 2, written before either run |
| `data/jev-*.json` | Jev's probability per chunk |
| `data/claude-*.json` | each judge's answer and confidence |
