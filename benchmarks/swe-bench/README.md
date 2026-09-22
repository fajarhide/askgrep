# SWE-bench Lite retrieval

Given a GitHub issue, find the files that have to change to fix it.

The gold answer is the set of files in the real fix commit, so the ground truth
costs nothing and cannot take sides. That is the point. Everything in
`docs/evaluation` is precision on a question chosen because a regex could check
it, which says nothing about what a sweep misses on real work, and a miss is the
failure that matters: four findings with the fifth silently left out reads as an
all clear.

The design is borrowed from [ck](https://github.com/BeaconBay/ck), which runs the
same dataset for the same reason.

## Running it

```sh
export TYPESAFE_API_KEY=...
python3 run.py --repo pallets/flask --repo psf/requests
python3 summarise.py
```

`run.py` fetches the 300 Lite instances once, shallow clones each repository, and
checks out the instance's base commit before searching. Searching a later tree
would be searching a repository where somebody already fixed the bug.

BM25 runs on the same instances with no index and no dependency. If plain lexical
search wins, that is the most useful thing this repository can publish about
itself, and the table says so either way.

## Two choices that shape the number

**The issue text is truncated to 400 characters.** askgrep sends one request per
chunk, so the question is repeated once per chunk: on flask that is 7,280 copies,
and the full statement costs 4.5x more than all the code it is asked about.
Truncation is not only thrift. Nobody types 2,000 characters into a search box
either, and the cut is stated so the number means something.

**Every file in the repository is searched, including docs and tests.** On flask
that is 4,245 of 7,280 chunks in `docs/`, where a gold file never lives. Excluding
them would raise the score and would be measuring a tool that already knows where
to look.

## Cost

Roughly one to two US cents per instance at $0.042 per million input tokens, and
two to four minutes of wall clock. The full 300 would be hours and tens of
dollars, which is why the published run names its instance count in the table
header rather than rounding up into a claim about the tool.
