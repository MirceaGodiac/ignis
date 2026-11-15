#!/usr/bin/env python3
"""SLICE (PSI-106) forecast cases from the investigators' summary log, and their score.

Source of the outcome: Appendix B "SLICE Summary Log" of `Raw Data/SLICE-RPT-002_Post Flight Report-.pdf`
(pdf pages 30-61, one or more pages per GMT day of 2012). Each line of the log is one flow condition of one
flame (ignition): burner tube, fuel bottle, FUEL display, FAN display, radiometer, ISS O2 / CO2 / pressure, and
the observation columns Attached / Lifting / Lifted / Out of view / Unsteady / Blow Off / No photos? / Fuel out?,
marked with an X, plus a free-text NOTES column. The X marks are vertical (rotated) characters; they are read
with their positions under the column headers.

Only the pages of train and validation days are parsed. Pages of test days (GMT 044, 051, 083 and the last page),
and of GMT 055 (a SPICE propane day the test catalogue does not list, held by the split's rule), are never opened.

A case is one logged flow condition. Targets (Brier score, yes or no):
  blow_off  the flame blows off at this condition, or during the adjustment noted on its line
            (X under "Blow Off", or the notes say blow off / blowout on this line)
  lifted    the flame is lifting or lifted at this condition (X under "Lifting" or "Lifted", or the notes say so)
An unmarked line is read as an attached, burning flame: the crew logged the next condition of the same flame,
so it burned there. Lines that end because the fuel ran out, and the two lines whose FAN display cannot be read
(outside 50-145), are kept with `eligible=False` and are not scored.

Inputs are computed from the log and from calibration workbooks only: fuel composition, burner inner diameter,
the fuel flow (FUEL display x gas correction factor K, `SLICE_MFC_Calib.xlsx` / Dotson Table 3.2 convention),
mean jet velocity, the coflow velocity (FAN display through that day's end-of-session fan calibration,
`SLICE fan calibration 20140828.xls`), and the cabin O2, CO2 and pressure of the log.

Writes, under runs/dev/data_all/work/transfer_slice/:
  cases_train.jsonl, cases_validation_inputs.jsonl, cases_validation_outcomes.jsonl (never printed),
  baselines_train.json

    python tools/wsp/ext/slice_cases.py     (needs pdfplumber and xlrd)
"""
import json
import math
import re
import statistics
import sys
from collections import defaultdict
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
RAW = REPO / "runs/dev/data_all/raw/PSI-106/Raw Data"
LOG = RAW / "SLICE-RPT-002_Post Flight Report-.pdf"
FANCAL = RAW / "SLICE fan calibration 20140828.xls"
SPLIT = REPO / "docs/wsp/atlas/splits/PSI-106.json"
OUT = REPO / "runs/dev/data_all/work/transfer_slice"

# GMT day of 2012 -> catalogue date (docs/wsp/atlas/splits/PSI-106.json) and pdf page indices (0-based) of the log.
# The page of each day was found from the GMT stamps printed on it; continuation pages follow their day.
DAYS = {
    "040": ("2012-02-09", [29, 30]), "048": ("2012-02-17", [33, 34]), "058": ("2012-02-27", [39, 40, 41]),
    "061": ("2012-03-01", [42, 43]), "065": ("2012-03-05", [44, 45]), "069": ("2012-03-09", [46, 47, 48, 49]),
    "072": ("2012-03-12", [50, 51]), "075": ("2012-03-15", [52, 53, 54]), "079": ("2012-03-19", [55, 56]),
}
NEVER_OPENED = {"044": [31, 32], "051": [35, 36], "055": [37, 38], "083": [57, 58, 59, 60]}
# Final report 2017, Table 6 and the bottle count of Table 4: GMT 061 burned the second 40 % CH4 bottle; the log
# writes "100% Methane" on that day's lines. The table is taken; the log's text is kept in `fuel_log_text`.
FUEL_OVERRIDE = {"061": ("40%", "Methane", "final report 2017 Table 6 and Table 4 (log says 100% Methane)")}
K = {"methane": 0.72, "ethylene": 0.60, "ethane": 0.50, "propane": 0.36, "nitrogen": 1.00}
MW = {"methane": 16.043, "ethylene": 28.054, "ethane": 30.069, "propane": 44.096, "nitrogen": 28.014}
# Final report 2017 Table 5: measured inner diameter ranges (mm); the midpoint is used.
BURNER_ID = {"0.4": (0.4064, 0.4191), "0.8": (0.7620, 0.7747), "1.6": (1.5875, 1.6002),
             "2.1": (2.1336, 2.1463), "3.2": (3.2258, 3.2385)}
OBS = ["attached", "lifting", "lifted", "out_of_view", "unsteady", "blow_off", "no_photos", "fuel_out"]
RHO_PER_MW = 101325.0 / (8.314462 * 294.15)  # g/m^3 per g/mol at 21 C, 1 atm (MFC standard conditions)
EV = {"blow_off": re.compile(r"blow\s*-?\s*(off|out)|blowoff|blowout", re.I),
      "lifted": re.compile(r"\blift", re.I),
      "fuel_out": re.compile(r"out of fuel|ran out|fuel out", re.I)}


def split_map():
    s = json.loads(SPLIT.read_text(encoding="utf-8"))["strata"]["all"]["groups"]
    return {g: part for part, gs in s.items() for g in gs}


def num(s):
    try:
        v = float(str(s).replace("`", "").rstrip("'"))
        return v if math.isfinite(v) else None
    except (TypeError, ValueError):
        return None


# ---------------------------------------------------------------- fan calibration

def fan_calibration():
    """{day: [(FAN display, u2 cm/s), ...]} from the 'SLICE summary' sheet (day-end blocks; 048 start+end mean)."""
    import xlrd

    s = xlrd.open_workbook(str(FANCAL)).sheet_by_name("SLICE summary")
    blocks = {"040": [(s.cell_value(r, 5), s.cell_value(r, 7)) for r in range(7, 13)]}
    for r in range(s.nrows):
        lab = str(s.cell_value(r, 3)).strip()
        m = re.match(r"^(\d+)-(end|start)$", lab)
        if not m:
            continue
        day = f"{int(m.group(1)):03d}"
        if day not in DAYS:
            continue
        pts = []
        for rr in range(r, min(r + 6, s.nrows)):
            f, u = s.cell_value(rr, 5), s.cell_value(rr, 7)
            if isinstance(f, float) and isinstance(u, float):
                pts.append((f, u))
        blocks[day + ":" + m.group(2)] = pts
    cal = {}
    for day in DAYS:
        if day == "040":
            cal[day] = sorted(blocks["040"])
            continue
        sets = [blocks[k] for k in (day + ":start", day + ":end") if k in blocks]
        if not sets:
            continue
        byf = defaultdict(list)
        for pts in sets:
            for f, u in pts:
                byf[f].append(u)
        cal[day] = sorted((f, statistics.fmean(u)) for f, u in byf.items())
    return cal


def interp(pts, x):
    if x is None or not pts:
        return None
    xs = [p[0] for p in pts]
    i = 1
    while i < len(pts) - 1 and x > xs[i]:
        i += 1
    (x0, y0), (x1, y1) = pts[i - 1], pts[i]
    return y0 + (y1 - y0) * (x - x0) / (x1 - x0)


# ---------------------------------------------------------------- log parsing

def page_rows(p, hdr_cache):
    hx = sorted({round(c["x0"]) for c in p.chars if not c["upright"] and 60 < c["top"] < 165})
    if len(hx) < 20:
        hx = hdr_cache[round(p.width)]
    hdr_cache[round(p.width)] = hx
    obsx = hx[-8:]
    fuelx = [x for x in hx if 700 < x < 730][0]
    fanx = [x for x in hx if 750 < x < 775][0]
    words = sorted((w for w in p.extract_words(extra_attrs=["upright"]) if w["upright"] and 158 < w["top"] < 1130),
                   key=lambda w: (w["top"], w["x0"]))
    marks = [c for c in p.chars if not c["upright"] and c["top"] > 158 and c["text"].strip() and c["x0"] > obsx[0] - 8]
    col = lambda lo, hi, pred=lambda w: True: [w for w in words if lo <= w["x0"] <= hi and pred(w)]
    tests = col(140, 168, lambda w: re.match(r"^\d+[A-Za-z]{0,2}\??$", w["text"]))
    burn = col(262, 280, lambda w: w["text"] in BURNER_ID)
    pct = col(195, 212, lambda w: w["text"].endswith("%"))
    fname = col(185, 200, lambda w: w["text"][:1].isalpha())
    fuelw = col(fuelx - 15, fuelx + 6, lambda w: num(w["text"]) is not None)
    fanw = col(fanx - 6, fanx + 6, lambda w: num(w["text"]) is not None)
    o2w, co2w, pw = col(890, 912), col(940, 960), col(990, 1010)
    timew = col(30, 130, lambda w: re.search(r"\d{1,2}:\d{2}", w["text"]))
    notes = col(1335, 3000)
    tinfo = {}
    for tw in tests:
        near = lambda lst, lo, hi: [w["text"] for w in lst if lo <= w["top"] - tw["top"] <= hi]
        tinfo[round(tw["top"])] = dict(test=tw["text"], burner=(near(burn, -4, 4) or [None])[0],
                                       pct=(near(pct, -14, 2) or [None])[0], fuel=(near(fname, 0, 12) or [None])[0])
    rows = []
    for fw in fuelw:
        t = fw["top"]
        prev = [w for w in tests if w["top"] <= t + 3]
        near = lambda lst, dy: ([w["text"] for w in lst if abs(w["top"] - t) <= dy] or [None])[0]
        m = {}
        for c in marks:
            if abs(c["top"] - t) <= 14:
                j = min(range(8), key=lambda k: abs(obsx[k] - c["x0"]))
                if abs(obsx[j] - c["x0"]) < 6 and c["text"].upper() == "X":
                    m[OBS[j]] = True
        tm = [w["text"] for w in timew if -2 <= w["top"] - t <= 20]
        rows.append(dict(top=round(t), test_info=tinfo[round(prev[-1]["top"])] if prev else None,
                         fuel_display=num(fw["text"]), fan_display=num(near(fanw, 4)),
                         o2=num(near(o2w, 4)), co2=num(near(co2w, 4)), p=num(near(pw, 4)),
                         time=tm[0] if tm else None, marks=m,
                         notes=" ".join(w["text"] for w in notes if -6 <= w["top"] - t <= 30)))
    return rows


def note_events(text):
    """Events named on a line's notes, with the FUEL display value written after '@' if any."""
    ev = {}
    for k, rx in EV.items():
        mm = rx.search(text)
        if mm:
            tail = text[mm.end():mm.end() + 30]
            v = re.search(r"@[^0-9]*?fuel\s*=\s*(\d+)", tail, re.I)
            ev[k] = num(v.group(1)) if v else True
    return ev


def read_cases():
    import pdfplumber

    parts = split_map()
    cal = fan_calibration()
    pdf = pdfplumber.open(str(LOG))
    hdr_cache = {}
    cases = []
    for day, (date, pages) in DAYS.items():
        part = parts.get(f"PSI-106:day:{date}")
        assert part in ("train", "validation"), (day, part)
        assert not set(pages) & {p for v in NEVER_OPENED.values() for p in v}
        cur = dict(test=None, burner=None, pct=None, fuel=None)
        env = dict(o2=None, co2=None, p=None)
        step = defaultdict(int)
        for pn in pages:
            for r in page_rows(pdf.pages[pn], hdr_cache):
                ti = r["test_info"]
                if ti and ti["test"] != cur.get("test_src"):
                    cur = dict(test=ti["test"], test_src=ti["test"], burner=ti["burner"] or cur["burner"],
                               pct=ti["pct"] or cur["pct"], fuel=ti["fuel"] or cur["fuel"])
                ok = dict(o2=(100, 250), co2=(0, 15), p=(650, 800))  # mmHg; other numbers there are typing slips
                for k in env:
                    v = r[k]
                    env[k] = v if (v is not None and ok[k][0] < v < ok[k][1]) else env[k]
                pct, fuel, fuel_src = cur["pct"], cur["fuel"], "summary log"
                if day in FUEL_OVERRIDE:
                    pct, fuel, fuel_src = FUEL_OVERRIDE[day]
                hc = (fuel or "").lower()
                x = (num((pct or "").rstrip("%")) or 0) / 100.0
                kmix = 1.0 / (x / K[hc] + (1 - x) / K["nitrogen"]) if hc in K else None
                mw = x * MW[hc] + (1 - x) * MW["nitrogen"] if hc in MW else None
                q = kmix * r["fuel_display"] if (kmix and r["fuel_display"] is not None) else None
                b = BURNER_ID.get(cur["burner"])
                d_mm = (b[0] + b[1]) / 2 if b else None
                u_jet = (q * 1e-6 / 60) / (math.pi * (d_mm * 1e-3) ** 2 / 4) if (q and d_mm) else None
                key = (cur["test"],)
                step[key] += 1
                fan = r["fan_display"] if (r["fan_display"] is not None and 50 <= r["fan_display"] <= 145) else None
                ev = note_events(r["notes"])
                m = r["marks"]
                blow = bool(m.get("blow_off")) or ("blow_off" in ev)
                lifted = bool(m.get("lifting") or m.get("lifted")) or ("lifted" in ev)
                fuel_out = bool(m.get("fuel_out")) or ("fuel_out" in ev)
                cases.append({
                    "case_id": f"PSI-106:{day}:p{pn + 1}:{cur['test']}:{step[key]}",
                    "investigation": "PSI-106", "day": day, "date": date, "group_id": f"PSI-106:day:{date}",
                    "partition": part,
                    "source": {"file": LOG.name, "pdf_page": pn + 1, "y": r["top"]},
                    "test": cur["test"], "step_in_test": step[key], "gmt_time": r["time"],
                    "inputs": {
                        "fuel_hydrocarbon": hc or None, "x_fuel": x if hc else None,
                        "fuel_source": fuel_src, "fuel_log_text": f"{cur['pct']} {cur['fuel']}",
                        "mw_jet_g_mol": mw, "burner_nominal_mm": num(cur["burner"]), "burner_id_mm": d_mm,
                        "fuel_display": r["fuel_display"], "k_mix": kmix, "q_jet_sccm": q,
                        "mdot_jet_mg_s": q * 1e-6 / 60 * mw * RHO_PER_MW * 1e3 if (q and mw) else None,
                        "u_jet_m_s": u_jet,
                        "fan_display": fan, "u_coflow_cm_s": interp(cal.get(day), fan),
                        "cabin_o2_mmhg": env["o2"], "cabin_co2_mmhg": env["co2"], "cabin_p_mmhg": env["p"],
                        "x_o2_cabin": (env["o2"] / env["p"]) if (env["o2"] and env["p"]) else None,
                    },
                    "outcome": {
                        "blow_off": int(blow), "lifted": int(lifted), "fuel_out": int(fuel_out),
                        "unsteady": int(bool(m.get("unsteady"))), "out_of_view": int(bool(m.get("out_of_view"))),
                        "eligible": (not fuel_out) and fan is not None and u_jet is not None,
                        "event_fuel_display": {k: v for k, v in ev.items() if not isinstance(v, bool)},
                        "marks": sorted(m), "notes": r["notes"],
                    },
                })
    return cases


# ---------------------------------------------------------------- scoring

def score(forecasts, outcomes, target="blow_off"):
    """Brier score. forecasts {case_id: p}; outcomes {case_id: {"day", target, "eligible"}}."""
    per_day, missing = defaultdict(list), []
    for cid, o in outcomes.items():
        if not o.get("eligible", True) or o.get(target) is None:
            continue
        p = forecasts.get(cid)
        if p is None:
            missing.append(cid)
            continue
        if not 0.0 <= p <= 1.0:
            raise ValueError(f"probability outside [0, 1] for {cid}")
        per_day[o["day"]].append((p - o[target]) ** 2)
    days = {d: statistics.fmean(v) for d, v in sorted(per_day.items())}
    allv = [x for v in per_day.values() for x in v]
    return {"target": target, "aggregate_day_mean": statistics.fmean(days.values()) if days else None,
            "pooled_mean": statistics.fmean(allv) if allv else None, "n": len(allv), "per_day": days,
            "missing": missing}


def baselines(train):
    res = {}
    out = {c["case_id"]: {"day": c["day"], **c["outcome"]} for c in train}
    el = [c for c in train if c["outcome"]["eligible"]]
    for tgt in ("blow_off", "lifted"):
        inc = statistics.fmean(c["outcome"][tgt] for c in el)
        res[f"{tgt}_incidence_pooled"] = score({c["case_id"]: inc for c in train}, out, tgt) | {"p": inc}
        lodo = {}
        for c in train:
            r = [d["outcome"][tgt] for d in el if d["day"] != c["day"]]
            lodo[c["case_id"]] = statistics.fmean(r)
        res[f"{tgt}_incidence_leave_one_day_out"] = score(lodo, out, tgt)
        byb = defaultdict(list)
        for c in el:
            byb[c["inputs"]["burner_nominal_mm"]].append(c["outcome"][tgt])
        res[f"{tgt}_incidence_by_burner"] = score(
            {c["case_id"]: statistics.fmean(byb[c["inputs"]["burner_nominal_mm"]]) if byb[c["inputs"]["burner_nominal_mm"]]
             else inc for c in train}, out, tgt) | {"p": {str(k): round(statistics.fmean(v), 4) for k, v in byb.items()}}
        res[f"{tgt}_persistence_never"] = score({c["case_id"]: 0.0 for c in train}, out, tgt)
    return res


def write(path, rows):
    with path.open("w", encoding="utf-8", newline="\n") as fh:
        for r in rows:
            fh.write(json.dumps(r, ensure_ascii=False) + "\n")


def main():
    cases = read_cases()
    train = [c for c in cases if c["partition"] == "train"]
    val = [c for c in cases if c["partition"] == "validation"]
    OUT.mkdir(parents=True, exist_ok=True)
    write(OUT / "cases_train.jsonl", train)
    write(OUT / "cases_validation_inputs.jsonl", [{k: v for k, v in c.items() if k != "outcome"} for c in val])
    write(OUT / "cases_validation_outcomes.jsonl",
          [{"case_id": c["case_id"], "day": c["day"], **c["outcome"]} for c in val])
    b = baselines(train)
    (OUT / "baselines_train.json").write_text(json.dumps(b, indent=1), encoding="utf-8")
    cnt = defaultdict(int)
    for c in train:
        cnt[c["day"]] += 1
    vc = defaultdict(int)
    for c in val:
        vc[c["day"]] += 1
    miss = sum(1 for c in train + val if c["inputs"]["u_coflow_cm_s"] is None or c["inputs"]["u_jet_m_s"] is None)
    print(f"train conditions {len(train)} by day {dict(cnt)}; validation conditions {len(val)} by day {dict(vc)}; "
          f"cases without u_jet or u_coflow {miss}")
    el = [c for c in train if c["outcome"]["eligible"]]
    print(f"train eligible {len(el)}: blow_off {sum(c['outcome']['blow_off'] for c in el)}, "
          f"lifted {sum(c['outcome']['lifted'] for c in el)}, flames {len({(c['day'], c['test']) for c in train})}")
    for k, v in b.items():
        print(f"{k:36s} Brier agg {v['aggregate_day_mean']:.4f}  pooled {v['pooled_mean']:.4f}  n {v['n']}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
