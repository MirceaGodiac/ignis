"""Conditions covered by the usable FLEX-2 series (v2, all batches), per fuel: counts and ranges only.

    python -I tools/wsp/media/flex2_coverage.py
"""
import json
from collections import defaultdict
from pathlib import Path

OUT = Path(__file__).resolve().parents[3] / "runs/dev/media/droplet"


def main():
    by = defaultdict(list)
    for p in sorted((OUT / "flex2_series_v2").rglob("*.json")):
        s = json.loads(p.read_text(encoding="utf-8"))
        if s.get("quality") != "usable":
            continue
        c = s["conditions"]
        by[c["fuel"]["value"]].append(dict(
            test=s["run_id"], batch=s.get("batch"), partition=s.get("partition"),
            o2=c["x_o2"]["value"], co2=c["x_co2"]["value"], he=c["x_he"]["value"],
            p=c["pressure"]["value"]))
    res = {}
    for f, rows in sorted(by.items()):
        known = [r for r in rows if r["o2"] is not None]

        def rng(k, rows=known, s=1.0):
            v = [r[k] * s for r in rows if r[k] is not None]
            return [round(min(v), 3), round(max(v), 3)] if v else None
        res[f] = dict(usable=len(rows), by_batch={b: sum(1 for r in rows if r["batch"] == b) for b in {r["batch"] for r in rows}},
                      by_partition={b: sum(1 for r in rows if r["partition"] == b) for b in {r["partition"] for r in rows}},
                      ambient_unknown=len(rows) - len(known), x_o2=rng("o2"), x_co2=rng("co2"), x_he=rng("he"),
                      pressure_atm=rng("p", rows, 1 / 101325))
    (OUT / "flex2_coverage.json").write_text(json.dumps(res, indent=1) + "\n", encoding="utf-8")
    print(json.dumps(res, indent=1))


if __name__ == "__main__":
    main()
