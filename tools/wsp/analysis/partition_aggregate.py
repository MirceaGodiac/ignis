"""Aggregate comparison of two releases in one `wsp-dev feedback` file, with runs left out by id.

usage: partition_aggregate.py <feedback.json> <split-name> <index-A> <index-B> [--skip run,run,...]

Written for the registered n-heptane look of PROTOCOL 13.7, where one validation run was studied
beforehand and must not enter the aggregate. It prints aggregates only: counts, the session-weighted
mean (mean of session means, the evaluator's estimate) and the case mean of CRPS, of the MAE of the
median and of the coverage of the 50 % and 90 % bands over the cases both releases scored, and how
many sessions are lower, level (within 1 um) or higher for A. No per-case and no per-session value is
printed, and the runs left out are never read past their id. Nothing here writes a model parameter.
"""
import json
import statistics
import sys
from collections import defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
import lib  # noqa: E402

fb = json.load(open(sys.argv[1], encoding="utf-8"))
name = sys.argv[2]
ia, ib = int(sys.argv[3]), int(sys.argv[4])
skip = set()
if "--skip" in sys.argv:
    skip = {s for s in sys.argv[sys.argv.index("--skip") + 1].split(",") if s}
assert fb["partition"] in ("train", "validation"), "development partitions only"
group = lib.groups(name)
METRICS = ("crps", "mae_median", "coverage@0.5", "coverage@0.9")


def cases(i):
    out = {}
    for c in fb["releases"][i]["cases"]:
        if any(s in c["run_id"] for s in skip):
            continue
        m = c.get("metrics") or {}
        if "crps" in m:
            out[(c["run_id"], c["cutoff_s"])] = m
    return out


a, b = cases(ia), cases(ib)
common = sorted(k for k in a if k in b)
by = defaultdict(list)
for k in common:
    by[group[k[0]]].append(k)
ra, rb = fb["releases"][ia], fb["releases"][ib]
print("file %s  partition %s  horizon %s s  cutoffs %s" % (sys.argv[1], fb["partition"], fb["horizon_s"], fb["cutoffs_s"]))
print("A = %s %s   B = %s %s" % (ra["model_name"], ra["release_id"][7:15], rb["model_name"], rb["release_id"][7:15]))
print("runs left out: %d (%s)   common cases %d in %d sessions   A only %d   B only %d" % (
    len(skip), ", ".join(sorted(skip)) or "none", len(common), len(by), len(a) - len(common), len(b) - len(common)))
for m in METRICS:
    scale = 1e6 if m in ("crps", "mae_median") else 1.0
    sa = statistics.mean(statistics.mean(a[k][m] for k in ks) for ks in by.values()) * scale
    sb = statistics.mean(statistics.mean(b[k][m] for k in ks) for ks in by.values()) * scale
    ca = statistics.mean(a[k][m] for k in common) * scale
    cb = statistics.mean(b[k][m] for k in common) * scale
    unit = "um" if scale > 1 else ""
    line = "%-13s session-weighted  A %7.2f  B %7.2f  (A - B %+6.2f) %s   case mean  A %7.2f  B %7.2f" % (m, sa, sb, sa - sb, unit, ca, cb)
    if scale > 1:
        lower = level = higher = 0
        for ks in by.values():
            d = (statistics.mean(a[k][m] for k in ks) - statistics.mean(b[k][m] for k in ks)) * scale
            lower += d < -1.0
            higher += d > 1.0
            level += abs(d) <= 1.0
        line += "   sessions: A lower %d, level %d, higher %d" % (lower, level, higher)
    print(line)
