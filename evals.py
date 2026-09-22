#!/usr/bin/env python3
"""Every measurement in this repository, from committed data, in one table.

Runs offline. No API key, no network, nothing to spend. That is the point: a
claim nobody can re-derive is not a measurement, and this repository has shipped
two tables that drifted from the corpus they named before anyone noticed.

    python3 evals.py            # the scoreboard
    python3 evals.py --full     # every underlying table too
"""
import argparse, json, statistics as st, subprocess, sys
from pathlib import Path

HERE = Path(__file__).parent
EVAL = HERE / "docs/evaluation/data"
BENCH = HERE / "benchmarks/swe-bench/results/results.json"


def load(p: Path):
    return json.loads(p.read_text()) if p.exists() else None


def prf(truth, probs, thr):
    tp = sum(1 for t, p in zip(truth, probs) if t and p >= thr)
    fp = sum(1 for t, p in zip(truth, probs) if not t and p >= thr)
    fn = sum(1 for t, p in zip(truth, probs) if t and p < thr)
    return (tp / (tp + fp) if tp + fp else 0.0, tp / (tp + fn) if tp + fn else 0.0)


def rows():
    out = []

    d = load(EVAL / "packsize.json")
    if d:
        best = prf(d["truth"], d["runs"]["1"][0]["probabilities"], 0.8)[0]
        worst = prf(d["truth"], d["runs"]["20"][0]["probabilities"], 0.8)[0]
        out.append(("chunks per request", "`.unwrap()` detection, regex truth",
                    "20 per request", f"{best:.2f} precision", f"{worst:.2f}", d["n"], "win"))

    lab = load(EVAL / "labels-unwrap.json")
    jev = load(EVAL / "jev-unwrap.json")
    cl = load(EVAL / "claude-unwrap.json")
    if lab and jev and cl:
        t = [lab["labels"][str(i)] for i in range(len(jev))]
        conf = {int(r["id"].split("_")[1]): r["confidence"] for r in cl}
        jp = [jev[i] for i in range(len(t)) if t[i]]
        jn = [jev[i] for i in range(len(t)) if not t[i]]
        cp = [conf[i] for i in range(len(t)) if t[i]]
        cn = [conf[i] for i in range(len(t)) if not t[i]]
        out.append(("score separation", "`.unwrap()` detection, regex truth",
                    "chat model confidence",
                    "no overlap" if max(jn) < min(jp) else "overlaps",
                    "overlaps" if max(cn) >= min(cp) else "no overlap",
                    len(t), "win"))

    labs = load(EVAL / "labels-swallow.json")
    if labs:
        out.append(("open judgment", "silently swallowed errors, hand labels",
                    "chat model", "not usable", "not usable",
                    len(labs["labels"]), "void"))

    b = load(BENCH)
    if b:
        inst = b["instances"]
        a10 = st.mean(x["askgrep"]["recall@10"] for x in inst)
        b10 = st.mean(x["bm25"]["recall@10"] for x in inst)
        out.append(("retrieval", "SWE-bench Lite, gold = the real fix commit",
                    "BM25, no index", f"{a10:.2f} recall@10", f"{b10:.2f}",
                    len(inst), "loss"))
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--full", action="store_true")
    a = ap.parse_args()

    print("Unit tests")
    t = subprocess.run(["cargo", "test", "--quiet"], cwd=HERE,
                       capture_output=True, text=True)
    line = next((l for l in t.stdout.splitlines() if "test result" in l), "not run")
    print(f"  {line.strip()}\n")

    r = rows()
    if not r:
        sys.exit("no committed results found under docs/evaluation or benchmarks")

    w = max(len(x[0]) for x in r)
    print(f"{'measurement':<{w}}  {'against':<22}  {'askgrep':<16}  {'baseline':<12}  {'n':>4}  verdict")
    print("-" * (w + 70))
    for name, task, base, mine, theirs, n, verdict in r:
        print(f"{name:<{w}}  {base:<22}  {mine:<16}  {theirs:<12}  {n:>4}  {verdict}")

    won = sum(1 for x in r if x[6] == "win")
    lost = sum(1 for x in r if x[6] == "loss")
    void = sum(1 for x in r if x[6] == "void")
    print(f"\n{won} win, {lost} loss, {void} void. The void one had an answer key "
          f"written by a model\nfrom the same family as one of its subjects; "
          f"docs/evaluation says why it does not count.")

    if a.full:
        for script in ("docs/evaluation/score.py", "benchmarks/swe-bench/summarise.py"):
            print(f"\n{'=' * 70}\n{script}\n{'=' * 70}")
            p = subprocess.run([sys.executable, script], cwd=HERE, capture_output=True, text=True)
            print(p.stdout or p.stderr)


if __name__ == "__main__":
    main()
