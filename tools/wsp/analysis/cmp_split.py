"""Splits one `ignis eval compare` output into two aggregate tables by the data root a case was sealed on.

Why it exists. The evaluator keys a sealed forecast by (run, prefix hash). A release that was sealed
on the shared records and on the private copy of the data root (whose prefixes carry one more
condition or series) therefore has two cases per run and cutoff, and `ignis eval compare` reports
them as one table. This script takes that comparison and the comparison made when only the private
root had been sealed, and prints the aggregate of each half.

It reads per-case rows of a held-out partition but prints only aggregates: the cluster-equal mean of
each metric per release (the evaluator's estimate: mean over clusters of the cluster means), for
event models the integrated Brier score (the mean over cases of the per-case contribution, which is
the number the results tables quote), and the paired difference to a reference with a cluster
bootstrap. The half that the evaluator scored alone is recomputed and must equal the evaluator's own
numbers, which is the check that the split is exact. The check holds because every run appears once
in each half with the same outcome: the censoring weights of the doubled set equal those of a half.

    python tools/wsp/analysis/cmp_split.py --mixed mixed.json --private private.json [--reference ID] [--out out.json]
"""
import argparse
import json
import random
import sys
from collections import defaultdict


def cluster_mean(rows, metric):
    by = defaultdict(list)
    for r in rows:
        if r["scored"] and metric in r["metrics"] and r["metrics"][metric] is not None:
            by[r["cluster"]].append(r["metrics"][metric])
    means = {k: sum(v) / len(v) for k, v in by.items()}
    return (sum(means.values()) / len(means) if means else None), len(means), sum(len(v) for v in by.values())


def case_mean(rows, metric):
    v = [r["metrics"][metric] for r in rows if r["admitted"] and r["metrics"].get(metric) is not None]
    return sum(v) / len(v) if v else None


def paired(rows_ref, rows_other, metric, n_boot, seed):
    ref = {(r["run_id"], r["prefix_hash"]): r for r in rows_ref}
    by = defaultdict(list)
    for r in rows_other:
        a = ref.get((r["run_id"], r["prefix_hash"]))
        if a is None or not (a["scored"] and r["scored"]):
            continue
        x, y = a["metrics"].get(metric), r["metrics"].get(metric)
        if x is None or y is None:
            continue
        by[a["cluster"]].append(y - x)
    means = [sum(v) / len(v) for _, v in sorted(by.items())]
    if not means:
        return None
    est = sum(means) / len(means)
    k = len(means)
    if k < 2:
        return {"estimate": est, "ci": None, "n_clusters": k}
    rng = random.Random(seed)
    boots = sorted(sum(means[rng.randrange(k)] for _ in range(k)) / k for _ in range(n_boot))
    lo, hi = boots[int(0.025 * n_boot)], boots[min(n_boot - 1, int(0.975 * n_boot))]
    return {"estimate": est, "ci": [lo, hi], "n_clusters": k, "n_cases": sum(len(v) for v in by.values())}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--mixed", required=True, help="comparison after both data roots were sealed")
    ap.add_argument("--private", required=True, help="comparison made when only the private root had been sealed")
    ap.add_argument("--reference", default=None, help="release id (prefix) of the paired reference; default: the comparison's")
    ap.add_argument("--n-boot", type=int, default=4000)
    ap.add_argument("--seed", type=int, default=20261114)
    ap.add_argument("--out", default=None)
    a = ap.parse_args()
    mixed = json.load(open(a.mixed, encoding="utf-8"))
    priv = json.load(open(a.private, encoding="utf-8"))
    rels = mixed["releases"]
    if set(rels) != set(priv["releases"]):
        sys.exit("the two comparisons are not of the same releases")
    reference = mixed["reference"]
    if a.reference:
        hit = [r for r in rels if r.startswith(a.reference) or r.split(":")[-1].startswith(a.reference)]
        if len(hit) != 1:
            sys.exit(f"--reference matches {len(hit)} releases")
        reference = hit[0]
    priv_keys = {(c["run_id"], c["prefix_hash"]) for c in priv["cases"][rels[0]]}
    halves = {"private": {}, "shared": {}}
    for rid in rels:
        rows = mixed["cases"][rid]
        halves["private"][rid] = [c for c in rows if (c["run_id"], c["prefix_hash"]) in priv_keys]
        halves["shared"][rid] = [c for c in rows if (c["run_id"], c["prefix_hash"]) not in priv_keys]
    metrics = sorted(mixed["per_release"][rels[0]]["metrics"].keys())
    out = {"mixed": a.mixed, "private": a.private, "reference": reference, "halves": {}}
    worst = 0.0
    for half, per in halves.items():
        n = len(per[rels[0]])
        runs = len({c["run_id"] for c in per[rels[0]]})
        block = {"n_cases": n, "n_runs": runs, "per_release": {}, "paired_vs_reference": {}}
        print(f"\n{half} root: {n} cases of {runs} runs")
        for rid in rels:
            row = {}
            for m in metrics:
                est, k, cnt = cluster_mean(per[rid], m)
                row[m] = {"estimate": est, "n_clusters": k, "n_cases": cnt}
                if half == "private":
                    tool = priv["per_release"][rid]["metrics"][m]["estimate"]
                    if est is not None and tool is not None:
                        worst = max(worst, abs(est - tool))
            ib = {}
            for m in metrics:
                if not m.startswith("ibs_contrib["):
                    continue
                cause = m[len("ibs_contrib["):-1]
                ib[cause] = case_mean(per[rid], m)
                if half == "private":
                    tool = priv["per_release"][rid].get("events", {}).get("causes", {}).get(cause, {}).get("integrated_brier")
                    if tool is not None and ib[cause] is not None:
                        worst = max(worst, abs(ib[cause] - tool))
            if ib:
                row["integrated_brier"] = ib
            block["per_release"][rid] = row
            print("  " + rid[:20] + "  cluster-equal: " + "  ".join(f"{m}={row[m]['estimate']:.4f}" for m in metrics if row[m]["estimate"] is not None))
            if ib:
                print("  " + " " * 20 + "  integrated Brier: " + "  ".join(f"{c}={v:.4f}" for c, v in ib.items() if v is not None))
        for rid in rels:
            if rid == reference:
                continue
            d = {m: paired(per[reference], per[rid], m, a.n_boot, a.seed) for m in metrics}
            block["paired_vs_reference"][rid] = d
            print("  " + rid[:20] + " minus " + reference[:20] + ": " + "  ".join(
                f"{m}={v['estimate']:+.4f} [{v['ci'][0]:+.4f}, {v['ci'][1]:+.4f}]" if v and v["ci"] else f"{m}=n/a"
                for m, v in d.items()))
        out["halves"][half] = block
    print(f"\ncheck: largest difference between this script and the evaluator on the private half: {worst:.2e}")
    out["check_max_abs_difference_to_evaluator"] = worst
    if worst > 1e-9:
        sys.exit("the private half does not reproduce the evaluator's numbers: the split is not exact")
    if a.out:
        json.dump(out, open(a.out, "w", encoding="utf-8"), indent=1)


if __name__ == "__main__":
    main()
