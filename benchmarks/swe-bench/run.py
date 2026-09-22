#!/usr/bin/env python3
"""Retrieval benchmark: given a GitHub issue, find the files that have to change.

The gold answer is the set of files in the real fix commit, so the ground truth
costs nothing and cannot take sides. This is the measurement askgrep was missing:
everything else in docs/evaluation is precision on a question chosen because a
regex could check it, which says nothing about what a sweep misses on real work.

BM25 runs on the same instances. If plain lexical search wins, that is the most
useful thing this repository can publish about itself.
"""
import argparse, json, math, re, subprocess, sys, time, urllib.request
from collections import Counter
from pathlib import Path

HERE = Path(__file__).parent
REPOS = HERE / "repos"
RESULTS = HERE / "results"
HF = ("https://datasets-server.huggingface.co/rows?dataset=princeton-nlp"
      "%2FSWE-bench_Lite&config=default&split=test&offset={}&length=100")

# The issue text is repeated in every request, once per chunk. On flask that is
# 7,280 copies, and the full statement costs 4.5x more than all the code it is
# asked about. Truncating is not only thrift: nobody types 2,000 characters into
# a search box either. The cut is stated here so the number means something.
ISSUE_CHARS = 400


def load_instances():
    f = HERE / "instances.json"
    if f.exists():
        return json.loads(f.read_text())
    rows = []
    for off in (0, 100, 200):
        with urllib.request.urlopen(HF.format(off), timeout=120) as r:
            rows += [x["row"] for x in json.load(r)["rows"]]
    keep = ["instance_id", "repo", "base_commit", "problem_statement", "patch"]
    rows = [{k: x[k] for k in keep} for x in rows]
    f.write_text(json.dumps(rows, indent=1))
    return rows


def gold_files(patch: str) -> list[str]:
    return sorted(set(re.findall(r"^\+\+\+ b/(\S+)", patch, re.M)))


def checkout(repo: str, commit: str) -> Path:
    d = REPOS / repo.replace("/", "__")
    if not d.exists():
        REPOS.mkdir(parents=True, exist_ok=True)
        subprocess.run(["git", "clone", "-q", "--filter=blob:none",
                        f"https://github.com/{repo}.git", str(d)], check=True)
    # Detached at the instance's base commit: searching a later tree would be
    # searching a repository where somebody already fixed the bug.
    subprocess.run(["git", "-C", str(d), "checkout", "-q", "--detach", commit], check=True)
    return d


def question(statement: str) -> str:
    one = " ".join(statement.split())[:ISSUE_CHARS]
    return f"has to be changed to fix this bug report: {one}"


def askgrep_rank(tree: Path, statement: str, jobs: int) -> tuple[list[str], dict]:
    """Best chunk score per file, files ranked high to low."""
    p = subprocess.run(
        ["askgrep", question(statement), str(tree), "--json", "-t", "0", "-j", str(jobs)],
        capture_output=True, text=True)
    best: dict[str, float] = {}
    for line in p.stdout.splitlines():
        if not line.strip():
            continue
        h = json.loads(line)
        f = h["file"]
        best[f] = max(best.get(f, 0.0), h["score"])
    ranked = sorted(best, key=lambda f: -best[f])
    return ranked, {"chunks": len(p.stdout.splitlines()), "stderr": p.stderr.strip()[-200:]}


def bm25_rank(tree: Path, statement: str) -> list[str]:
    """Plain BM25 over file contents. No dependency, no index, no excuses."""
    tok = lambda s: re.findall(r"[a-z_][a-z0-9_]{2,}", s.lower())
    docs: dict[str, Counter] = {}
    for f in tree.rglob("*"):
        if not f.is_file() or ".git" in f.parts:
            continue
        try:
            text = f.read_text(errors="ignore")
        except OSError:
            continue
        if len(text) > 2_000_000:
            continue
        docs[str(f.relative_to(tree))] = Counter(tok(text))
    if not docs:
        return []
    N = len(docs)
    avg = sum(sum(c.values()) for c in docs.values()) / N
    df = Counter()
    for c in docs.values():
        df.update(c.keys())
    k1, b = 1.5, 0.75
    q = tok(statement)
    scores = {}
    for name, c in docs.items():
        dl = sum(c.values()) or 1
        s = 0.0
        for t in q:
            if t not in c:
                continue
            idf = math.log(1 + (N - df[t] + 0.5) / (df[t] + 0.5))
            s += idf * c[t] * (k1 + 1) / (c[t] + k1 * (1 - b + b * dl / avg))
        scores[name] = s
    return sorted(scores, key=lambda f: -scores[f])


def metrics(ranked: list[str], gold: list[str]) -> dict:
    pos = {f: i + 1 for i, f in enumerate(ranked)}
    first = min((pos[g] for g in gold if g in pos), default=None)
    return {
        "mrr": 1.0 / first if first else 0.0,
        "first_rank": first,
        **{f"recall@{k}": float(all(pos.get(g, 10**9) <= k for g in gold)) for k in (1, 5, 10)},
    }


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--repo", action="append", help="only these repos, e.g. pallets/flask")
    ap.add_argument("--limit", type=int)
    ap.add_argument("-j", "--jobs", type=int, default=32)
    ap.add_argument("--out", default=str(RESULTS / "results.json"))
    a = ap.parse_args()

    rows = load_instances()
    if a.repo:
        rows = [r for r in rows if r["repo"] in a.repo]
    if a.limit:
        rows = rows[: a.limit]
    print(f"{len(rows)} instances", file=sys.stderr)

    out, t0 = [], time.time()
    for i, r in enumerate(rows, 1):
        gold = gold_files(r["patch"])
        tree = checkout(r["repo"], r["base_commit"])
        t = time.time()
        ranked, info = askgrep_rank(tree, r["problem_statement"], a.jobs)
        ak = metrics(ranked, gold)
        bm = metrics(bm25_rank(tree, r["problem_statement"]), gold)
        out.append({"instance_id": r["instance_id"], "repo": r["repo"], "gold": gold,
                    "chunks": info["chunks"], "seconds": round(time.time() - t, 1),
                    "askgrep": ak, "bm25": bm})
        print(f"  [{i}/{len(rows)}] {r['instance_id']:<26} "
              f"askgrep r@10={ak['recall@10']:.0f} rank={ak['first_rank']}  "
              f"bm25 r@10={bm['recall@10']:.0f} rank={bm['first_rank']}  "
              f"{info['chunks']} chunks {time.time()-t:.0f}s", file=sys.stderr, flush=True)

    RESULTS.mkdir(parents=True, exist_ok=True)
    Path(a.out).write_text(json.dumps(
        {"issue_chars": ISSUE_CHARS, "seconds": round(time.time() - t0, 1),
         "instances": out}, indent=1))
    print(f"\nwrote {a.out}", file=sys.stderr)


if __name__ == "__main__":
    main()
