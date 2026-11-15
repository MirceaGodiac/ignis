#!/usr/bin/env python3
"""Chamber history of FLEX-2 (PSI-68) from the text tier only: for every rack point (test points, gas
reading points, calibration points) its time, the chamber pressure and temperature before the igniter came
on, and for gas reading points the analysed composition. Conditions only: a test point's chamber file is
read up to its igniter-on stamp and no further; no droplet series, image, radiometer trace or outcome is
read. Writes `runs/dev/data_all/work/atlas/flex2_chamber_history.jsonl`, one row per point, in time order.

    python -I tools/wsp/ext/flex2_chamber_history.py
"""
import csv
import datetime
import json
import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
RAW = REPO / "runs/dev/data_all/raw"
FILES = REPO / "runs/dev/data_all/work/atlas/files.jsonl"
MANIFEST = RAW / "MANIFEST.jsonl"
OUT = REPO / "runs/dev/data_all/work/atlas/flex2_chamber_history.jsonl"
PSIA = 6894.757293168


def header(path):
    head = {}
    with path.open(encoding="utf-8", errors="replace") as f:
        for line in f:
            if line.startswith("MDCA MP Time"):
                break
            if ":" in line and not line[:1].isspace():
                k, v = line.split(":", 1)
                head[k.strip()] = v.strip()
    return head


def stamp(s):
    try:
        return datetime.datetime.fromisoformat(s) if s else None
    except ValueError:
        return None


def chamber(path, until):
    """Rows of a chamber file stamped before `until` (all rows when `until` is None): first and last stamp,
    mean of the four pressure transducers and of the temperatures, per window."""
    rows = []
    with path.open(encoding="utf-8", errors="replace") as f:
        head = [h.strip() for h in next(csv.reader([f.readline()]))]
        ip = [i for i, h in enumerate(head) if re.fullmatch(r"FCU_MP_PT\d+\(PSIA\)", h)]
        it = [i for i, h in enumerate(head) if re.fullmatch(r"FCU_MP_TM\d+\(Deg C\)", h)]
        for line in f:
            c = line.rstrip("\n").split(",")
            t = stamp(c[0].strip().strip('"'))
            if t is None:
                continue
            if until is not None and t >= until:
                break
            try:
                rows.append((t, [float(c[i]) for i in ip], [float(c[i]) for i in it]))
            except (ValueError, IndexError):
                continue
    if not rows:
        return None
    names = [head[i] for i in ip]
    first, last = rows[0], rows[-1]
    win = [r for r in rows if r[0] >= last[0] - datetime.timedelta(seconds=120)]
    return dict(first=first[0].isoformat(sep=" "), last=last[0].isoformat(sep=" "), n=len(rows), transducers=names,
                p_first_psia=first[1], p_last_psia=last[1],
                p_window_psia=[sum(r[1][k] for r in win) / len(win) for k in range(len(names))],
                t_window_c=[sum(r[2][k] for r in win) / len(win) for k in range(len(it))] if it else [])


def mixture(path):
    rows = list(csv.reader(path.open(encoding="utf-8", errors="replace")))
    if len(rows) < 2:
        return None
    h = [x.strip() for x in rows[0]]
    r = dict(zip(h, [x.strip() for x in rows[1]]))
    fr = {}
    for k, v in r.items():
        if k != "Date/Time":
            try:
                fr[k] = float(v.rstrip("%")) / 100.0
            except ValueError:
                fr[k] = None
    return dict(date_time=r.get("Date/Time"), fractions=fr)


def main():
    local = {}
    for line in MANIFEST.open(encoding="utf-8"):
        r = json.loads(line)
        if r["investigation"] == "PSI-68" and r.get("status") in ("ok", "reused"):
            local[r["key"].split("/")[-1]] = RAW / r["local_path"]
    pts = {}
    for line in FILES.open(encoding="utf-8"):
        if '"PSI-68"' not in line:
            continue
        r = json.loads(line)
        if r["investigation"] != "PSI-68" or not r.get("test_id") or r["file_name"] not in local:
            continue
        pts.setdefault((r["test_id"], r["test_id_scheme"]), []).append(local[r["file_name"]])
    out = []
    for (tid, scheme), paths in pts.items():
        def find(rx):
            for p in paths:
                n = re.sub(r"^PSI-68_Raw Data_", "", p.name)
                if re.fullmatch(rx, n, re.I):
                    return p
            return None
        tl = find(re.escape(tid) + r"\.txt")
        ch = find(re.escape(tid) + r"_Chamber_Readings\d*\.txt")
        mx = find(re.escape(tid) + r"_mixture\.txt")
        h = header(tl) if tl else {}
        ign = stamp(h.get("AS Burn to Completion/Igniter On"))
        start = stamp(h.get("AS Started")) or stamp(h.get("CC Setup Started"))
        row = dict(point=tid, scheme=scheme, igniter_on=ign.isoformat(sep=" ") if ign else None,
                   started=start.isoformat(sep=" ") if start else None, planned_burn_ms=h.get("Burn to Completion (milliseconds)"))
        if ch:
            # a test point is read only up to its igniter-on stamp; other points (no flame) whole
            c = chamber(ch, ign if scheme == "cir_test_point" else None)
            if c is None and scheme == "cir_test_point" and ign is None:
                c = None
            row["chamber"] = c
        if mx:
            row["mixture"] = mixture(mx)
        t = row["igniter_on"] or row["started"] or ((row.get("chamber") or {}).get("first"))
        row["time"] = t
        out.append(row)
    out.sort(key=lambda r: (r["time"] or "9999", r["point"]))
    with OUT.open("w", encoding="utf-8", newline="\n") as f:
        for r in out:
            f.write(json.dumps(r, ensure_ascii=False) + "\n")
    print(len(out), "points;", sum(1 for r in out if r.get("chamber")), "with chamber rows;",
          sum(1 for r in out if r.get("mixture")), "with a mixture")
    return 0


if __name__ == "__main__":
    sys.exit(main())
