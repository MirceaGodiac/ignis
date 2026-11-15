"""Dump the NASA test sheets (conditions, kavg, Dext, narrative comment) of TRAIN runs of a FLEX split.

usage: p2_flex_sheets.py <split-name: flex|flex_heptane> <out.md>
"""
import re
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
rows = []
extra_kinds = {}
for rid in ids:
    keys = {a["name"] for a in recs[rid]["aliases"]}
    files = [n for n in names if any(("_" + k + "_") in n or n.endswith("_" + k + ".txt") for k in keys)]
    sheet = [n for n in files if n.endswith("_TestSheet.txt")]
    kinds = sorted(n.rsplit("_", 1)[-1].replace(".txt", "") for n in files)
    extra_kinds[rid] = kinds
    txt = (RAW / sheet[0]).read_text(encoding="utf-8", errors="replace") if sheet else ""

    def grab(pat):
        m = re.search(pat, txt)
        return m.group(1).strip() if m else "?"

    comment = txt.split("COMMENTS:", 1)[1].strip() if "COMMENTS:" in txt else ""
    rows.append({
        "run": rid, "group": grp[rid], "files": kinds,
        "p_atm": grab(r"Pressure \(atm\) =\s*([\d.]+)"), "o2": grab(r"O2 Mole Fraction =\s*([\d.]+)"),
        "n2": grab(r"N2 Mole Fraction =\s*([\d.]+)"), "co2": grab(r"CO2 Mole Fraction =\s*([\d.]+)"),
        "he": grab(r"He Mole Fraction =\s*([\d.]+)"), "T": grab(r"Temperature \(K\) =\s*([\d.]+)"),
        "d0": grab(r"D0\s+\(mm\) =\s*([\d.]+)"), "kavg": grab(r"kavg\s+\(mm2/s\) =\s*([\d.]+)"),
        "fiber": grab(r"Fiber:\s*(\S+)"), "dext": grab(r"Dext\s+\(mm\) =\s*([\d.]+)"),
        "time": grab(r"Test Time:\s*([\d:]+)"), "comment": " ".join(comment.split()),
    })
rows.sort(key=lambda r: (r["group"], r["time"]))
md = [f"# Test sheets of the TRAIN runs of split `{name}` ({len(rows)} runs)", ""]
md.append("| run | time | P atm | O2 | N2 | CO2 | He | D0 mm | kavg mm2/s | Dext mm | extra files |")
md.append("|---|---|---|---|---|---|---|---|---|---|---|")
for r in rows:
    extra = ",".join(k for k in r["files"] if k not in ("DropletData", "TestSheet"))
    md.append(f"| {r['run'].replace('PSI-69:', '')} | {r['time']} | {r['p_atm']} | {r['o2']} | {r['n2']} | {r['co2']} | {r['he']} | {r['d0']} | {r['kavg']} | {r['dext']} | {extra} |")
md.append("")
for r in rows:
    md.append(f"**{r['run'].replace('PSI-69:', '')}** (O2 {r['o2']}, CO2 {r['co2']}, He {r['he']}, {r['p_atm']} atm, D0 {r['d0']}, k {r['kavg']}, Dext {r['dext']}): {r['comment']}")
    md.append("")
out.write_text("\n".join(md), encoding="utf-8")
print("written", out, "rows", len(rows))
