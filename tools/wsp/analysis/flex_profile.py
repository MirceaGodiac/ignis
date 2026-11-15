"""Per-run physical profile of the TRAIN runs of a FLEX split from the raw DropletData + test sheet.

usage: p2_flex_profile.py <split> <out.json>
Analysis only: prints measurements; chooses no model parameter.
"""
import json
import math
import re
import statistics
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
import lib as p2lib

name, out = sys.argv[1], Path(sys.argv[2])
RAW = p2lib.ROOT / "data/raw/samples/PSI-69"
ids = p2lib.partition_ids(name, "train")
recs = p2lib.experiments(name, ids)
grp = p2lib.groups(name)
names = sorted(p.name for p in RAW.iterdir())


def files(rid, suffix):
    keys = {a["name"] for a in recs[rid]["aliases"]}
    return [n for n in names if n.endswith(suffix) and any(("_" + k + "_") in n for k in keys)]


def load(rid):
    f = files(rid, "_DropletData.txt")
    rows = []
    if not f:
        return rows
    for line in (RAW / f[0]).read_text(encoding="utf-8", errors="replace").splitlines()[1:]:
        p = re.split(r"[\t,; ]+", line.strip())
        try:
            v = [float(x) for x in p[:4]]
        except ValueError:
            continue
        if len(v) >= 2 and v[1] > 0 and all(math.isfinite(x) for x in v):
            rows.append(v + [0.0] * (4 - len(v)))
    return rows


def sheet(rid):
    f = files(rid, "_TestSheet.txt")
    txt = (RAW / f[0]).read_text(encoding="utf-8", errors="replace") if f else ""

    def g(pat):
        m = re.search(pat, txt)
        return float(m.group(1)) if m else None

    return {"d0": g(r"D0\s+\(mm\) =\s*([\d.]+)"), "kavg": g(r"kavg\s+\(mm2/s\) =\s*([\d.]+)"),
            "dext": g(r"Dext\s+\(mm\) =\s*([\d.]+)"), "p_atm": g(r"Pressure \(atm\) =\s*([\d.]+)"),
            "o2": g(r"O2 Mole Fraction =\s*([\d.]+)"), "co2": g(r"CO2 Mole Fraction =\s*([\d.]+)"),
            "he": g(r"He Mole Fraction =\s*([\d.]+)"), "n2": g(r"N2 Mole Fraction =\s*([\d.]+)")}


def slope(pts):
    """Least-squares slope of D^2 (mm2) on t (s): a measurement of the series, not a model fit."""
    n = len(pts)
    if n < 4:
        return None
    mt = sum(p[0] for p in pts) / n
    mq = sum(p[1] for p in pts) / n
    sxx = sum((p[0] - mt) ** 2 for p in pts)
    if sxx <= 0:
        return None
    return sum((p[0] - mt) * (p[1] - mq) for p in pts) / sxx


WINDOWS = [(0.2, 1.0), (1.0, 2.0), (2.0, 3.0), (3.0, 5.0), (5.0, 8.0), (8.0, 12.0), (12.0, 18.0), (18.0, 26.0), (26.0, 40.0)]
profiles = []
for rid in ids:
    rows = load(rid)
    sh = sheet(rid)
    prof = {"run": rid, "group": grp[rid], **sh, "n": len(rows)}
    if not rows:
        profiles.append(prof)
        continue
    t = [r[0] for r in rows]
    q = [r[1] ** 2 for r in rows]
    prof["t_first"], prof["t_last"] = t[0], t[-1]
    # held values: runs of identical consecutive diameters (tracker frozen)
    held = []
    i = 0
    while i < len(rows):
        j = i
        while j + 1 < len(rows) and abs(rows[j + 1][1] - rows[i][1]) < 1e-9:
            j += 1
        if j - i >= 2:
            held.append((round(t[i], 2), round(t[j], 2), round(rows[i][1], 3)))
        i = j + 1
    prof["held"] = held[:6]
    # gaps
    gaps = [(round(t[i], 2), round(t[i + 1], 2)) for i in range(len(t) - 1) if t[i + 1] - t[i] > 0.25]
    prof["gaps"] = gaps[:6]
    # pre-ignition reference diameter and early minimum (igniter glare)
    pre = [r[1] for r in rows if -1.4 <= r[0] <= -0.8]
    prof["d_pre"] = statistics.median(pre) if pre else None
    early = [r[1] for r in rows if -0.6 <= r[0] <= 0.12]
    prof["d_min_ign"] = min(early) if early else None
    post = [r[1] for r in rows if 0.15 <= r[0] <= 0.35]
    prof["d_015_035"] = statistics.median(post) if post else None
    # largest single-frame relative jump of D for t in [-0.6, 1.0]
    jumps = [(rows[i + 1][1] / rows[i][1] - 1, rows[i + 1][0]) for i in range(len(rows) - 1) if -0.6 <= rows[i][0] <= 1.0 and t[i + 1] - t[i] < 0.1]
    if jumps:
        j = max(jumps, key=lambda x: abs(x[0]))
        prof["max_jump"] = (round(j[0], 3), round(j[1], 2))
    # window slopes -> K (mm2/s)
    ks = {}
    for a, b in WINDOWS:
        pts = [(tt, qq) for tt, qq in zip(t, q) if a <= tt < b]
        if len(pts) >= max(6, int(0.5 * (b - a) * 30 * 0.4)):
            s = slope(pts)
            if s is not None:
                ks[f"{a:g}-{b:g}"] = round(-s, 3)
    prof["K"] = ks
    # drift speed over 0.2..1.2 s (mm/s)
    seg = [r for r in rows if 0.2 <= r[0] <= 1.2]
    if len(seg) >= 6:
        dt = seg[-1][0] - seg[0][0]
        prof["drift_mm_s"] = round(math.hypot(seg[-1][2] - seg[0][2], seg[-1][3] - seg[0][3]) / dt, 2) if dt > 0 else None
    # last diameter and the knee: last time the 1-s backward slope still exceeds 35 % of the 1-5 s rate
    prof["d_last"] = rows[-1][1]
    ref_pts = [(tt, qq) for tt, qq in zip(t, q) if 1.0 <= tt < 5.0]
    kref = -slope(ref_pts) if len(ref_pts) >= 20 and slope(ref_pts) is not None else None
    prof["k_1_5"] = round(kref, 3) if kref else None
    knee = None
    if kref and kref > 0.05:
        tt0 = 1.0
        while tt0 + 1.0 <= t[-1]:
            pts = [(a, b) for a, b in zip(t, q) if tt0 <= a < tt0 + 1.0]
            if len(pts) >= 10:
                s = slope(pts)
                if s is not None and -s < 0.35 * kref:
                    # require it to stay low for the next window too, when data exist
                    pts2 = [(a, b) for a, b in zip(t, q) if tt0 + 1.0 <= a < tt0 + 2.0]
                    s2 = slope(pts2) if len(pts2) >= 10 else None
                    if s2 is None or -s2 < 0.5 * kref:
                        knee = tt0
                        break
            tt0 += 0.25
    prof["t_knee"] = knee
    if knee is not None:
        near = [r[1] for r in rows if knee - 0.1 <= r[0] <= knee + 0.4]
        prof["d_knee"] = statistics.median(near) if near else None
        pts = [(a, b) for a, b in zip(t, q) if knee + 1.0 <= a]
        s = slope(pts) if len(pts) >= 20 else None
        prof["k_post"] = round(-s, 4) if s is not None else None
    profiles.append(prof)

out.write_text(json.dumps(profiles, indent=1), encoding="utf-8")
print("written", out, len(profiles))
