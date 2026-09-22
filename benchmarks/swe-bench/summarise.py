#!/usr/bin/env python3
"""Turn results.json into the table that goes in the README. No new numbers."""
import argparse, json, re, statistics as st
from pathlib import Path

ap = argparse.ArgumentParser()
ap.add_argument("results", nargs="?", default="results/results.json")
ap.add_argument("--readme", action="store_true",
                help="rewrite the block between the markers in the top-level README")
args = ap.parse_args()
d = json.loads(Path(args.results).read_text())
inst = d["instances"]
n = len(inst)

print(f"# SWE-bench Lite retrieval, {n} instances\n")
print(f"Issue text truncated to {d['issue_chars']} characters. "
      f"{d['seconds']/60:.0f} minutes of wall clock.\n")
print(f"| | recall@1 | recall@5 | recall@10 | MRR |")
print(f"| --- | --- | --- | --- | --- |")
for name in ("askgrep", "bm25"):
    r = [st.mean(x[name][f"recall@{k}"] for x in inst) for k in (1, 5, 10)]
    m = st.mean(x[name]["mrr"] for x in inst)
    print(f"| {name} | {r[0]:.2f} | {r[1]:.2f} | {r[2]:.2f} | {m:.3f} |")

print(f"\n| instance | gold | askgrep rank | bm25 rank | chunks | s |")
print(f"| --- | --- | --- | --- | --- | --- |")
for x in sorted(inst, key=lambda x: x["instance_id"]):
    g = ", ".join(x["gold"])[:44]
    a = x["askgrep"]["first_rank"] or "miss"
    b = x["bm25"]["first_rank"] or "miss"
    print(f"| {x['instance_id']} | {g} | {a} | {b} | {x['chunks']} | {x['seconds']:.0f} |")

won = sum(1 for x in inst if (x["askgrep"]["first_rank"] or 10**9) < (x["bm25"]["first_rank"] or 10**9))
tied = sum(1 for x in inst if (x["askgrep"]["first_rank"] or 10**9) == (x["bm25"]["first_rank"] or 10**9))
print(f"\naskgrep ranked the gold file higher on {won} of {n}, tied on {tied}, "
      f"lower on {n - won - tied}.")


if args.readme:
    # The README block is generated, never typed. A table nobody can regenerate
    # is how the last one drifted from the corpus it claimed to measure.
    rows = []
    for name, label in (("askgrep", "askgrep"), ("bm25", "BM25, no index")):
        r = [st.mean(x[name][f"recall@{k}"] for x in inst) for k in (1, 5, 10)]
        m = st.mean(x[name]["mrr"] for x in inst)
        rows.append(f"| {label} | {r[0]:.2f} | {r[1]:.2f} | {r[2]:.2f} | {m:.3f} |")
    block = "\n".join([
        f"Given a GitHub issue from SWE-bench Lite, find the files the real fix",
        f"changed. The gold answer is the fix commit, so nobody had to write a label.",
        "",
        f"| over {n} instances | recall@1 | recall@5 | recall@10 | MRR |",
        "| --- | --- | --- | --- | --- |",
        *rows,
        "",
        f"{n} instances from pallets/flask and psf/requests, the two smallest",
        f"repositories in the set. Issue text truncated to {d['issue_chars']} characters,",
        "because askgrep repeats the question once per chunk. Every file is searched,",
        "docs and tests included.",
        "",
        "[benchmarks/swe-bench](benchmarks/swe-bench) has the runner, the raw results",
        "and what the two choices above do to the number.",
    ])
    readme = Path(__file__).parent.parent.parent / "README.md"
    text = readme.read_text()
    new, count = re.subn(
        r"(<!-- swe-bench:start -->\n).*?(\n<!-- swe-bench:end -->)",
        lambda m: m.group(1) + block + m.group(2), text, flags=re.S)
    if not count:
        raise SystemExit("markers <!-- swe-bench:start --> / :end not found in README.md")
    readme.write_text(new)
    print(f"README updated from {args.results}")
