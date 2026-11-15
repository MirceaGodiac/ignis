#!/usr/bin/env python3
"""Cases files of arms H and F on FLEX-2 (PROTOCOL 12.7) from the measurer's series. Only series whose
`quality` is `usable` enter. One file per fuel: `<out>/cases_<fuel>.jsonl`, in the generic format of
`tools/wsp/ext/transfer_droplet.py`. Never reads a test-day folder (`flex2_series_test*` is refused).

  --scale k   multiplies every diameter by k (a stated sensitivity only; 1 = the series as written)
  --shift s   adds s seconds to every time (a stated sensitivity only; 0 = as written)

Ambient: a series without a measured x_o2 / x_co2 / x_he takes the catalogue's recovered ambient only when
its class is `bracketed` (tools/wsp/ext/flex2_ambient.py); those conditions carry evidence ignis_derived,
the flags `ambient_recovered` and `ambient_class:bracketed`, and the case row `ambient_class`; they are
written to `cases_<fuel>_ambient_bracketed.jsonl`, apart from the measured ones, so any score reports them
apart. A series whose ambient is `unknown` is not written (the forward would refuse it).

    python -I tools/wsp/ext/flex2_cases.py --series runs/dev/media/droplet/flex2_series_v2 --out runs/dev/data_all/work/transfer_droplet/flex2_v2
"""
import argparse
import glob
import json
import sys
from collections import Counter
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
CAT = REPO / "runs/dev/data_all/work/atlas/flex2_tests.jsonl"
NAME = {"n-heptane": "heptane", "n-decane": "decane", "n-octane": "octane", "iso-octane": "isooctane"}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--series", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--scale", type=float, default=1.0)
    ap.add_argument("--shift", type=float, default=0.0)
    a = ap.parse_args()
    assert "test" not in Path(a.series).name, "test-day series are not read"
    cat = {json.loads(l)["test_id"]: json.loads(l) for l in CAT.read_text(encoding="utf-8").splitlines()}
    out = {}
    skipped = Counter()
    for part in ("train", "validation"):
        for p in sorted(glob.glob(str(Path(a.series) / part / "*.json"))):
            d = json.loads(Path(p).read_text(encoding="utf-8"))
            if d.get("quality") != "usable":
                continue
            assert d["partition"] == part
            tid = d["run_id"].split(":")[1]
            for o in d["observations"]:
                if o.get("value_si") is not None:
                    o["value_si"] *= a.scale
                o["time_s"] += a.shift
                o["available_at_s"] = o["time_s"]
            d["test"], d["fuel"] = tid, cat[tid]["fuel"]
            # ambient: measured on the day (as the series carries it), else the recovered ambient of the
            # catalogue (class `bracketed` only, flagged), else the series is not a case (the forward would refuse)
            cond = d["conditions"]
            missing = [k for k in ("x_o2", "x_co2", "x_he") if (cond.get(k) or {}).get("value") is None]
            d["ambient_class"] = "measured_same_day"
            if missing:
                c = cat[tid]
                if c.get("ambient_recovered_class") != "bracketed" or not c.get("ambient_recovered"):
                    skipped[c.get("ambient_recovered_class") or "unknown"] += 1
                    continue
                rec, ev = c["ambient_recovered"], c["ambient_recovered_evidence"]
                note = (f"recovered (bracketed): mean of gas readings {ev['before']['point']} ({ev['before']['day']}) and "
                        f"{ev['after']['point']} ({ev['after']['day']}); no reading on the test's day; "
                        f"uncertainty {json.dumps(c.get('ambient_recovered_uncertainty'))}")
                for k, key in (("x_o2", "O2"), ("x_co2", "CO2"), ("x_he", "He")):
                    cond[k] = dict(value=rec[key], unit="1", source="runs/dev/data_all/work/atlas/flex2_tests.jsonl",
                                   note=note, evidence="ignis_derived", flags=["ambient_recovered", "ambient_class:bracketed"])
                d["ambient_class"] = "bracketed"
            d["condition_summary"] = dict(x_o2=d["conditions"]["x_o2"]["value"], x_he=d["conditions"]["x_he"]["value"],
                                          pressure_atm=round(d["conditions"]["pressure"]["value"] / 101325, 3),
                                          fuel_text=d["conditions"]["fuel"]["value"])
            d["sensitivity"] = dict(scale=a.scale, shift_s=a.shift)
            out.setdefault((d["fuel"], d["ambient_class"]), []).append(d)
    Path(a.out).mkdir(parents=True, exist_ok=True)
    for (f, amb), rs in out.items():
        # cases with a recovered ambient go to their own file, so that every score reports them apart
        name = f"cases_{NAME[f]}.jsonl" if amb == "measured_same_day" else f"cases_{NAME[f]}_ambient_{amb}.jsonl"
        with (Path(a.out) / name).open("w", encoding="utf-8", newline="\n") as fh:
            for r in rs:
                fh.write(json.dumps(r, ensure_ascii=False) + "\n")
        print(f, amb, len(rs), dict(Counter((r["partition"], r["session_id"]) for r in rs)))
    if skipped:
        print("usable series not written (ambient not measured and not recovered):", dict(skipped))
    return 0


if __name__ == "__main__":
    sys.exit(main())
