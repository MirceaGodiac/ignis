"""Print raw DropletData of TRAIN runs at reduced resolution.

usage: p2_flex_series.py <split> <t0> <t1> <step> <run-suffix> [...]
Columns: t, D (mm), D^2 (mm2), x, y (mm, zeroed position).
"""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
import lib as p2lib

name = sys.argv[1]
t0, t1, step = map(float, sys.argv[2:5])
RAW = p2lib.ROOT / "data/raw/samples/PSI-69"
ids = p2lib.partition_ids(name, "train")
recs = p2lib.experiments(name, ids)
names = sorted(p.name for p in RAW.iterdir())


def load(rid):
    keys = {a["name"] for a in recs[rid]["aliases"]}
    f = [n for n in names if n.endswith("_DropletData.txt") and any(("_" + k + "_") in n for k in keys)]
    rows = []
    for line in (RAW / f[0]).read_text(encoding="utf-8", errors="replace").splitlines()[1:]:
        p = line.split("\t")
        try:
            rows.append(tuple(float(x) for x in p[:4]))
        except ValueError:
            pass
    return rows


for suffix in sys.argv[5:]:
    match = [r for r in ids if suffix in r]
    assert match, f"{suffix}: not a train run of {name}"
    rid = match[0]
    rows = load(rid)
    print(f"== {rid}  n={len(rows)}  t=[{rows[0][0]:.2f},{rows[-1][0]:.2f}]")
    nxt = t0
    line = []
    for t, d, x, y in rows:
        if t < nxt or t > t1:
            continue
        line.append(f"{t:6.2f} {d:5.3f} {d*d:6.2f} ({x:5.1f},{y:5.1f})")
        nxt = t + step - 1e-6
    for i in range(0, len(line), 4):
        print("   " + " | ".join(line[i:i + 4]))
