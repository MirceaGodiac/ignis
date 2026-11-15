import json, statistics, sys
for f in sys.argv[1:]:
    fb = json.load(open(f))
    print(f)
    for r in fb["releases"]:
        cs = [c for c in r["cases"] if "crps" in c["metrics"]]
        v = sorted(c["metrics"]["crps"] * 1e6 for c in cs)
        trim = v[:int(len(v) * 0.95)]
        by = {}
        for c in cs:
            by.setdefault(c["cutoff_s"], []).append(c["metrics"]["crps"] * 1e6)
        print("  %-18s %s n=%d refused=%d mean %.1f median %.1f trimmed95 %.1f cov50 %.2f cov90 %.2f | by cutoff %s" % (
            r["model_name"], r["release_id"][7:15], len(v), len(r["refused"]), statistics.mean(v), statistics.median(v),
            statistics.mean(trim), statistics.mean(c["metrics"]["coverage@0.5"] for c in cs),
            statistics.mean(c["metrics"]["coverage@0.9"] for c in cs),
            {k: round(statistics.mean(x), 1) for k, x in sorted(by.items())}))
