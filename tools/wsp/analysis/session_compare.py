"""Session-grouped comparison of two releases on a development partition (the selection rule of
docs/wsp/PROTOCOL.md, section 7).

usage: session_compare.py <feedback.json> <profile.json> <index-A> <index-B> [--skip run,run,…]

A is the candidate, B the release it would replace. Prints, per session, the mean CRPS of both over
the cases both scored, and then the rule: A replaces B when its median CRPS over all common cases is
not worse and it is better or level (within 1 um) in at least 60 % of the sessions. The rule is read
on train; validation is reported, never used to choose. Nothing here writes a model parameter.
"""
import json
import statistics
import sys
from collections import defaultdict

fb = json.load(open(sys.argv[1], encoding="utf-8"))
prof = {r["run"]: r for r in json.load(open(sys.argv[2], encoding="utf-8"))}
ia, ib = int(sys.argv[3]), int(sys.argv[4])
skip = set()
if "--skip" in sys.argv:
    skip = {s for s in sys.argv[sys.argv.index("--skip") + 1].split(",") if s}


def cases(i):
    return {(c["run_id"], c["cutoff_s"]): c["metrics"]["crps"] * 1e6
            for c in fb["releases"][i]["cases"] if "crps" in c.get("metrics", {})}


a, b = cases(ia), cases(ib)
common = [k for k in a if k in b and not any(s in k[0] for s in skip)]
by = defaultdict(list)
for k in common:
    by[prof.get(k[0], {}).get("group", "?")].append(k)
print("A =", fb["releases"][ia]["model_name"], fb["releases"][ia]["release_id"][7:15],
      "  B =", fb["releases"][ib]["model_name"], fb["releases"][ib]["release_id"][7:15],
      "  common cases", len(common), " skipped runs", sorted(skip))
better = level = worse = 0
for g in sorted(by):
    ma = statistics.mean(a[k] for k in by[g])
    mb = statistics.mean(b[k] for k in by[g])
    tag = "A" if ma < mb - 1.0 else ("=" if abs(ma - mb) <= 1.0 else "B")
    better += tag == "A"
    level += tag == "="
    worse += tag == "B"
    print("  %-28s n=%3d  A %6.1f  B %6.1f  %s" % (g.split(":")[-1], len(by[g]), ma, mb, tag))
med_a = statistics.median(a[k] for k in common)
med_b = statistics.median(b[k] for k in common)
mean_a = statistics.mean(a[k] for k in common)
mean_b = statistics.mean(b[k] for k in common)
n = better + level + worse
share = (better + level) / n if n else 0.0
print("median CRPS  A %.1f  B %.1f   mean  A %.1f  B %.1f" % (med_a, med_b, mean_a, mean_b))
print("sessions: A better %d, level %d, B better %d  (A better or level in %.0f %%)" % (better, level, worse, 100 * share))
print("rule (median not worse and >= 60 %% of sessions better or level): %s" % ("A REPLACES B" if med_a <= med_b and share >= 0.6 else "B STAYS"))
