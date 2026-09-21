#!/usr/bin/env python3
"""Recompute every number in README.md from data/. No arguments."""
import json, pathlib, statistics as st

D = pathlib.Path(__file__).parent / "data"
load = lambda n: json.load(open(D / n))


def prf(truth, pred):
    tp = sum(1 for i in truth if truth[i] and pred[i])
    fp = sum(1 for i in truth if not truth[i] and pred[i])
    fn = sum(1 for i in truth if truth[i] and not pred[i])
    return (tp / (tp + fp) if tp + fp else 0.0, tp / (tp + fn) if tp + fn else 0.0)


def run(name, labels_file, jev_file, claude_file):
    lab = {int(k): v for k, v in load(labels_file)["labels"].items()}
    jev = load(jev_file)
    cl = {int(r["id"].split("_")[1]): r for r in load(claude_file)}
    print(f"\n=== {name} ({sum(lab.values())} of {len(lab)} positive) ===")
    print(f"{'':<20}{'precision':>10}{'recall':>8}")
    p, r = prf(lab, {i: cl[i]["answer"] == "yes" for i in lab})
    print(f"{'Claude, yes/no':<20}{p:>10.2f}{r:>8.2f}")
    for t in (0.5, 0.8):
        p, r = prf(lab, {i: jev[i] >= t for i in lab})
        print(f"{'Jev at ' + str(t):<20}{p:>10.2f}{r:>8.2f}")

    cp = [cl[i]["confidence"] for i in lab if lab[i]]
    cn = [cl[i]["confidence"] for i in lab if not lab[i]]
    jp = [jev[i] for i in lab if lab[i]]
    jn = [jev[i] for i in lab if not lab[i]]
    print(f"{'':<20}{'positives':>22}{'negatives':>22}{'overlap':>9}")
    print(f"{'Claude confidence':<20}{st.mean(cp):>10.1f} ({min(cp)} to {max(cp)}){st.mean(cn):>10.1f} ({min(cn)} to {max(cn)}){str(max(cn) >= min(cp)):>9}")
    print(f"{'Jev probability':<20}{st.mean(jp):>10.2f} ({min(jp):.2f} to {max(jp):.2f}){st.mean(jn):>7.2f} ({min(jn):.2f} to {max(jn):.2f}){str(max(jn) >= min(jp)):>9}")

    right = [cl[i]["confidence"] for i in lab if (cl[i]["answer"] == "yes") == lab[i]]
    wrong = [cl[i]["confidence"] for i in lab if (cl[i]["answer"] == "yes") != lab[i]]
    if wrong:
        print(f"Claude confidence when right {st.mean(right):.1f}, when wrong {st.mean(wrong):.1f}")


run("Experiment 1: .unwrap() or .expect(), regex ground truth",
    "labels-unwrap.json", "jev-unwrap.json", "claude-unwrap.json")
run("Experiment 2: silently swallows a failure, hand labels",
    "labels-swallow.json", "jev-swallow.json", "claude-swallow.json")
