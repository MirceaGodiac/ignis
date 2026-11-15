"""Near-duplicate audit of PSI-69 DropletData: one physical video analysed twice and filed under two ids.

Two files are 'near duplicates' when their droplet position tracks (x, y rounded to 0.1 mm) agree on
at least 90 % of the samples in t = 0.5..6 s and both tracks actually move. Position is not an outcome
of interest and nothing is printed for runs outside train except the fact of a match.
"""
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
import lib as p2lib

RAW = p2lib.ROOT / "data/raw/samples/PSI-69"
names = sorted(p.name for p in RAW.iterdir() if p.name.endswith("_DropletData.txt"))
recs = [json.loads(l) for l in (p2lib.ROOT / "data/work/records/flex/experiments.jsonl").open(encoding="utf-8")]
_, part_m = p2lib.split("flex")
_, part_h = p2lib.split("flex_heptane")
file_run = {}
for r in recs:
    for a in r["aliases"]:
        for n in names:
            if ("_" + a["name"] + "_") in n:
                file_run[n] = r


def track(n):
    out = {}
    for line in (RAW / n).read_text(encoding="utf-8", errors="replace").splitlines()[1:]:
        p = line.split("\t")
        try:
            t, x, y = float(p[0]), float(p[2]), float(p[3])
        except (ValueError, IndexError):
            continue
        if 0.5 <= t <= 6.0:
            out[round(t, 2)] = (x, y)
    return out


tracks = {n: track(n) for n in names}
pairs = []
for i, a in enumerate(names):
    ta = tracks[a]
    if len(ta) < 60:
        continue
    span_a = max(abs(v[0]) + abs(v[1]) for v in ta.values())
    if span_a < 2.0:
        continue
    for b in names[i + 1:]:
        tb = tracks[b]
        common = set(ta) & set(tb)
        if len(common) < 60:
            continue
        same = sum(1 for k in common if abs(ta[k][0] - tb[k][0]) <= 0.15 and abs(ta[k][1] - tb[k][1]) <= 0.15)
        if same >= 0.9 * len(common):
            pairs.append((a, b, same, len(common)))


def describe(n):
    r = file_run.get(n)
    if r is None:
        return "?"
    rid = r["id"]
    p = part_m.get(rid) or part_h.get(rid) or "not in a cohort"
    fuel = str(r["conditions"].get("fuel", {}).get("raw_value"))
    return (rid if p in ("train", "validation") else "<a %s run>" % p) + f" [{fuel}, {p}]"


print("files", len(names), "near-duplicate pairs", len(pairs))
for a, b, same, n in pairs:
    print(f"  {describe(a)}  ~~  {describe(b)}   ({same}/{n} identical positions)")
