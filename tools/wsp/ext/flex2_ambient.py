#!/usr/bin/env python3
"""Recovered ambient of the FLEX-2 (PSI-68) test points that have no gas reading on their day. Conditions
only, from the text tier: the gas readings (`<R>_Mixture.txt`) and the chamber history built by
`flex2_chamber_history.py` (pressure and temperature before ignition; no droplet series, rate or outcome).

It adds fields to `runs/dev/data_all/work/atlas/flex2_tests.jsonl` and never changes a measured one:

  ambient_recovered            {O2, He, CO2, N2} or null
  ambient_recovered_class      bracketed | carried_adjacent | unknown   (measured rows: measured_same_day)
  ambient_recovered_uncertainty  {o2_abs, he_abs, p_wrong} from the method's error on measured days
  ambient_recovered_evidence   the readings used, their days and compositions, the pressure levels
  ambient_candidates           for `unknown`: the nearest readings before and after, for information

Classes (the order in which they are tried):
  bracketed         no plausible reading on the day; the nearest plausible readings before and after,
                    each within MAX_GAP_DAYS, agree (|dO2| <= 0.01, |dHe| <= 0.05), and the test's pressure
                    level is that of the reading days or of a day between with no evidence of a change.
                    Value: the mean of the two.
  carried_adjacent  not bracketed; a plausible reading within ADJ_DAYS days on one side, with the same
                    pressure level (+-2 %) and no pressure-level change between. Its error on measured days is
                    too large (a refill to the same pressure is invisible in the snapshots), so it is NOT
                    assigned: such a test is `unknown` with the value kept in `ambient_recovered_evidence`.
  unknown           otherwise.
"Planned in the matrix" is not a class here: PSI-68's test matrix and SRD give only the envelope
(O2 0.17/0.21/0.25/0.30, N2 or He, 0.5-3 atm), never a per-test atmosphere.

The error of each rule is measured by leave-one-out on the days that DO have a reading (each reading day
predicted from the others by the same rule), and is written to
`runs/dev/data_all/work/atlas/flex2_ambient_method_error.json`.

    python -I tools/wsp/ext/flex2_chamber_history.py
    python -I tools/wsp/ext/flex2_ambient.py
"""
import datetime
import json
import statistics
import sys
from collections import Counter, OrderedDict
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
ATLAS = REPO / "runs/dev/data_all/work/atlas"
HIST = ATLAS / "flex2_chamber_history.jsonl"
CAT = ATLAS / "flex2_tests.jsonl"
ERR = ATLAS / "flex2_ambient_method_error.json"
MAX_GAP_DAYS = 45
ADJ_DAYS = 3
P_TOL = 0.02
KEYS = ("O2", "He", "CO2", "N2")


def date(s):
    return datetime.date.fromisoformat(s)


def plausible(fr):
    s = sum(fr.get(k) or 0.0 for k in ("O2", "N2", "He", "CO2", "Xe", "H20"))
    return 0.97 <= s <= 1.03 and (fr.get("O2") or 0.0) >= 0.05


def load_days():
    days = OrderedDict()
    for line in HIST.open(encoding="utf-8"):
        r = json.loads(line)
        if r["time"]:
            days.setdefault(r["time"][:10], []).append(r)
    out = []
    for d, rs in days.items():
        reading = None
        for r in rs:
            m = r.get("mixture")
            if m and plausible(m["fractions"]):
                reading = dict(point=r["point"], time=r["time"], comp={k: m["fractions"].get(k) or 0.0 for k in KEYS},
                               file_date=m.get("date_time"))
                break
        ps = sorted(r["chamber"]["p_window_psia"][0] for r in rs
                    if r.get("chamber") and r["chamber"]["p_window_psia"] and r["scheme"] in ("cir_test_point", "cir_gas_reading_point"))
        out.append(dict(day=d, reading=reading, p_level=statistics.median(ps) if ps else None,
                        p_range=[ps[0], ps[-1]] if ps else None))
    return out


def same_level(a, b):
    return a is not None and b is not None and abs(a / b - 1) <= P_TOL


def agree(a, b):
    return abs(a["O2"] - b["O2"]) <= 0.01 and abs(a["He"] - b["He"]) <= 0.05


def level_path_ok(days, i0, i1, p_test):
    """No pressure-level change between day indices i0..i1 (inclusive) and the test's level: every day with
    a pressure level in that span sits at the test's level (days without a level do not count)."""
    for d in days[i0:i1 + 1]:
        if d["p_level"] is not None and p_test is not None and not same_level(d["p_level"], p_test):
            return False
    return True


def recover(days, idx, p_test, exclude_self=False):
    """Recover the ambient of day index idx from the other days. Returns (class, comp, evidence)."""
    d0 = date(days[idx]["day"])
    before = [i for i in range(idx - 1, -1, -1) if days[i]["reading"]]
    after = [i for i in range(idx + 1, len(days)) if days[i]["reading"]]
    ib = before[0] if before else None
    ia = after[0] if after else None
    gb = (d0 - date(days[ib]["day"])).days if ib is not None else None
    ga = (date(days[ia]["day"]) - d0).days if ia is not None else None
    ev = dict(before=None if ib is None else dict(day=days[ib]["day"], point=days[ib]["reading"]["point"], comp=days[ib]["reading"]["comp"],
                                                  p_level=days[ib]["p_level"], gap_days=gb),
              after=None if ia is None else dict(day=days[ia]["day"], point=days[ia]["reading"]["point"], comp=days[ia]["reading"]["comp"],
                                                 p_level=days[ia]["p_level"], gap_days=ga),
              p_level_test=p_test)
    if ib is not None and ia is not None and gb <= MAX_GAP_DAYS and ga <= MAX_GAP_DAYS:
        cb, ca = days[ib]["reading"]["comp"], days[ia]["reading"]["comp"]
        if agree(cb, ca) and (p_test is None or level_path_ok(days, ib, ia, p_test) or
                              same_level(days[ib]["p_level"], p_test) or same_level(days[ia]["p_level"], p_test)):
            return "bracketed", {k: (cb[k] + ca[k]) / 2 for k in KEYS}, ev
    for i, g, lo, hi in ((ib, gb, ib, idx), (ia, ga, idx, ia)):
        if i is not None and g <= ADJ_DAYS and p_test is not None and same_level(days[i]["p_level"], p_test) \
                and level_path_ok(days, lo, hi, p_test):
            return "carried_adjacent", dict(days[i]["reading"]["comp"]), ev
    return "unknown", None, ev


def method_error(days):
    rows = {"bracketed": [], "carried_adjacent": [], "unknown": 0}
    for i, d in enumerate(days):
        if not d["reading"]:
            continue
        saved = d["reading"]
        d["reading"] = None  # leave this day out
        cls, comp, ev = recover(days, i, d["p_level"])
        d["reading"] = saved
        if cls == "unknown":
            rows["unknown"] += 1
            continue
        rows[cls].append(dict(day=d["day"], o2_err=comp["O2"] - saved["comp"]["O2"], he_err=comp["He"] - saved["comp"]["He"],
                              co2_err=comp["CO2"] - saved["comp"]["CO2"]))
    out = dict(rule=f"leave one reading day out, recover it from the others; bracket gap <= {MAX_GAP_DAYS} d, adjacent <= {ADJ_DAYS} d, "
                    f"pressure level +-{P_TOL:.0%}", unknown=rows["unknown"])
    for cls in ("bracketed", "carried_adjacent"):
        v = rows[cls]
        if not v:
            out[cls] = dict(n=0)
            continue
        o = sorted(abs(x["o2_err"]) for x in v)
        h = sorted(abs(x["he_err"]) for x in v)
        wrong = [x for x in v if abs(x["o2_err"]) > 0.01 or abs(x["he_err"]) > 0.05]
        ok = [abs(x["o2_err"]) for x in v if x not in wrong]
        out[cls] = dict(n=len(v), wrong=len(wrong), p_wrong=len(wrong) / len(v),
                        o2_abs_median=statistics.median(o), o2_abs_max=o[-1], he_abs_max=h[-1],
                        o2_abs_max_when_right=max(ok) if ok else None,
                        co2_abs_max=max(abs(x["co2_err"]) for x in v),
                        wrong_cases=[dict(day=x["day"], o2_err=round(x["o2_err"], 4), he_err=round(x["he_err"], 4)) for x in wrong])
    return out


def main():
    days = load_days()
    idx = {d["day"]: i for i, d in enumerate(days)}
    err = method_error(days)
    ERR.write_text(json.dumps(err, indent=1) + "\n", encoding="utf-8")
    cat = [json.loads(l) for l in CAT.read_text(encoding="utf-8").splitlines()]
    psia = 6894.757293168
    counts = Counter()
    for r in cat:
        for k in ("ambient_recovered", "ambient_recovered_class", "ambient_recovered_uncertainty", "ambient_recovered_evidence",
                  "ambient_candidates"):
            r.pop(k, None)
        if r.get("ambient"):
            r["ambient_recovered_class"] = "measured_same_day"
            counts[("measured_same_day", r.get("fuel"))] += 1
            continue
        if not r.get("date") or r["date"] not in idx:
            r.update(ambient_recovered=None, ambient_recovered_class="unknown",
                     ambient_recovered_evidence=dict(reason="no date or no chamber history for the day"))
            counts[("unknown", r.get("fuel"))] += 1
            continue
        p_test = r["pressure_pa"] / psia if r.get("pressure_pa") else days[idx[r["date"]]]["p_level"]
        cls, comp, ev = recover(days, idx[r["date"]], p_test)
        if cls == "carried_adjacent":
            # the rule is measured and rejected: wrong on 4 of 9 measured days (a fill between two days at the
            # same pressure is invisible in the snapshots); its value is kept as a candidate only
            ev["adjacent_candidate"] = dict(comp=comp, note="carried_adjacent rule, wrong on measured days; not used")
            cls, comp = "unknown", None
        r["ambient_recovered_class"] = cls
        r["ambient_recovered"] = comp
        r["ambient_recovered_evidence"] = ev
        if cls == "unknown":
            r["ambient_candidates"] = [x for x in (ev["before"], ev["after"]) if x]
        else:
            e = err[cls]
            r["ambient_recovered_uncertainty"] = dict(
                p_wrong=round(e["p_wrong"], 3), o2_abs_if_right=e["o2_abs_max_when_right"], o2_abs_max=e["o2_abs_max"],
                he_abs_max=e["he_abs_max"], from_measured_days=e["n"],
                meaning="p_wrong: share of measured days the same rule got wrong (|dO2| > 0.01 or |dHe| > 0.05); "
                        "o2_abs_if_right: the largest O2 error on the days it got right")
        counts[(cls, r.get("fuel"))] += 1
    with CAT.open("w", encoding="utf-8", newline="\n") as f:
        for r in cat:
            f.write(json.dumps(r, ensure_ascii=False) + "\n")
    print(json.dumps(err, indent=1))
    for k, v in sorted(counts.items(), key=lambda kv: (kv[0][0], str(kv[0][1]))):
        print(k, v)
    return 0


if __name__ == "__main__":
    sys.exit(main())
