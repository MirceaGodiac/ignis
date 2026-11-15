#!/usr/bin/env python3
"""Add burn conditions to every measured track and write BURNS.jsonl (no outcome field).

Conditions come from NASA's tables, never from the measurement, and each says its source:
  PSI-25  PSI-25_Experimental table_BASS-II.csv: date, GMT day, sample, material, flow restrictor,
          fan display, air display, calibrated initial/final O2 (other columns are not read);
          PSI-25_Reference Data_Fan calibration data.xlsx: FAN display -> U = 1.0066 x AIR for the
          burn's GMT day and restrictor count, linear in FAN between calibrated rows, never
          extrapolated below the lowest non-zero row (the zero-flow row is not used);
          the video overlay (from the track): FAN, AIR, RAD per second.
  PSI-26  PSI-26_Science Documents_As Run_Test Matrix.xlsx, sheet 2013: the rows whose approximate
          start time falls inside the video (date on videos = the file name's date): material,
          restrictor, sample distance, fan display, air display, O2, pressure (the observations
          column is not read).
Flow direction: from the matrix text when it says so (PSI-26 'opposed'), otherwise inferred from the
igniter position in the video (flow runs right to left in this camera: igniter at the right =
concurrent, at the left = opposed); the source is stated.

    python -I tools/wsp/media/fabric/enrich.py --root runs/dev/media/fabric [--only PSI-25]
"""
import argparse
import csv
import datetime as dt
import json
import re
from pathlib import Path

import openpyxl

REPO = Path(__file__).resolve().parents[4]
T25 = REPO / "data/raw/samples/PSI-25/PSI-25_Experimental table_BASS-II.csv"
CAL = REPO / "data/raw/samples/PSI-25/PSI-25_Reference Data_Fan calibration data.xlsx"
ASRUN = REPO / "runs/dev/data_ext/raw/samples/PSI-26/PSI-26_Science Documents_As Run_Test Matrix.xlsx"


def fnum(x):
    try:
        return float(x)
    except (TypeError, ValueError):
        return None


def table25():
    out = {}
    with open(T25, encoding="utf-8-sig") as f:
        for r in csv.DictReader(f):
            k = {kk.strip(): v for kk, v in r.items()}
            o2i = fnum(k.get("Calibrated  initial O2 % by vol") or k.get("Calibrated initial O2 % by vol"))
            o2f = fnum(k.get("Calibrated final O2 % by vol"))
            out[k["Test #"].strip()] = dict(date=k["Date"].strip(), gmt=k["GMT"].strip(), sample=k["Sample #"].strip(),
                                            material=" ".join(k["Fuel Sample Material"].split()),
                                            restrictor=k["Flow restrictor"].strip(), fan_display=k["Fan display"].strip(),
                                            air_display=k["Air display"].strip(),
                                            o2_initial=(o2i / 100 if o2i is not None and o2i > 1 else o2i),
                                            o2_final=(o2f / 100 if o2f is not None and o2f > 1 else o2f))
    return out


def calibration():
    """{gmt day (int): {restrictors (0/1/2): [(fan, U cm/s), ...]}}"""
    ws = openpyxl.load_workbook(CAL, data_only=True)["Fan Calibration data"]
    grid = [list(r) for r in ws.iter_rows(values_only=True)]
    ncol = max(len(r) for r in grid)
    # section rows: the example block labels them ('1 restrictor', '2 resrictor' in column D)
    sec1 = next(i for i, r in enumerate(grid) if "1 restrictor" in str(r[3]))
    sec2 = next(i for i, r in enumerate(grid) if "2 res" in str(r[3]))
    out = {}
    for c in range(ncol):
        if len(grid) < 4 or c + 2 >= ncol or grid[2][c] != "DATE":
            continue
        g = str(grid[3][c + 1] or "")
        m = re.search(r"20\d\d-0*(\d+)", g)
        if not m:
            continue
        day = int(m.group(1))
        runs, cur = [], None
        for r_i, r in enumerate(grid[5:], start=5):
            pot, fan = (r[c] if c < len(r) else None), (r[c + 1] if c + 1 < len(r) else None)
            air = r[c + 2] if c + 2 < len(r) else None
            if isinstance(pot, (int, float)) and isinstance(fan, (int, float)):
                if cur is None:
                    cur = (r_i, [])
                    runs.append(cur)
                cur[1].append((float(fan), fnum(air)))
            else:
                cur = None
        blk = {}
        for start, rows in runs:
            nres = 0 if start < sec1 else (1 if start < sec2 else 2)
            pts = sorted({(f, round(1.0066 * a_, 3)) for f, a_ in rows if a_ is not None and f > 38.0})
            if pts:
                blk.setdefault(nres, pts)
        out.setdefault(day, blk)
    return out


def u_of(fan, pts):
    if fan is None or not pts:
        return None
    if fan < pts[0][0] or fan > pts[-1][0]:
        return None
    for (f0, u0), (f1, u1) in zip(pts, pts[1:]):
        if f0 <= fan <= f1:
            return round(u0 + (u1 - u0) * (fan - f0) / (f1 - f0), 2) if f1 > f0 else u0
    return pts[-1][1] if fan == pts[-1][0] else None


def width_of(material):
    m = (material or "").lower()
    if "sphere" in m or "spherical" in m:
        mm = re.search(r"(\d+(?:\.\d+)?)\s*cm", m)
        return (float(mm.group(1)) / 100 if mm else None), "sphere diameter"
    mm = re.search(r"(\d+)/(\d+)\s*\"?", m)
    if "rod" in m and mm:
        return round(int(mm.group(1)) / int(mm.group(2)) * 0.0254, 5), "rod diameter"
    mm = re.search(r"(\d+(?:\.\d+)?)\"", m)
    if "rod" in m and mm:
        return round(float(mm.group(1)) * 0.0254, 5), "rod diameter"
    mm = re.search(r"(\d+(?:\.\d+)?)\s*cm", m)
    if mm:
        return float(mm.group(1)) / 100, "sample width"
    return None, None


def asrun26():
    wb = openpyxl.load_workbook(ASRUN, read_only=True, data_only=True)
    rows = []
    for i, r in enumerate(wb["2013"].iter_rows(values_only=True)):
        if i < 2 or r[1] is None or not isinstance(r[1], dt.datetime):
            continue
        times = re.findall(r"(?<![\d-])(\d{1,2}):(\d\d)", str(r[2] or ""))
        vd = r[1].date()
        g = re.search(r"(?:13-|GMT\s*)0*(\d{2,3})", str(r[2] or ""))
        if g and vd.month == 6:
            # rows filed under 2013-06-02 carry GMT 150: the video date is GMT day + 1 (display date)
            vd = dt.date(2013, 1, 1) + dt.timedelta(days=int(g.group(1)))
        rows.append(dict(video_date=vd, actual_date=(r[0].date() if isinstance(r[0], dt.datetime) else None),
                         start=(int(times[0][0]) * 60 + int(times[0][1])) if times else None, start_text=str(r[2]),
                         as_run_test=r[3], test=r[4], sample=r[5], material=r[6], igniter=r[7], restrictor=r[8],
                         sample_distance=r[9], fan_display=r[10], air_display=r[11], o2=fnum(r[27]), pressure_atm=fnum(r[28])))
    return rows


def enrich25(d, tab, cal):
    tid = d["run_id"].split(":", 1)[1]
    row = tab.get(tid)
    c = {"source": {}}
    if row:
        c.update(date=None, gmt_day=row["gmt"], sample=row["sample"], material=row["material"], restrictor=row["restrictor"],
                 fan_display_table=row["fan_display"], air_display_table=row["air_display"],
                 x_o2_initial=row["o2_initial"], x_o2_final=row["o2_final"], pressure_atm=None)
        try:
            c["date"] = dt.datetime.strptime(row["date"], "%m/%d/%Y").date().isoformat()
        except ValueError:
            c["date"] = row["date"]
        c["source"].update(date="NASA experiment table (actual date)", material="NASA experiment table",
                           x_o2="NASA experiment table (calibrated initial/final O2)", restrictor="NASA experiment table",
                           pressure="not in the table")
    w, wkind = width_of(c.get("material") or d.get("material"))
    c["width_m"], c["width_kind"] = w, wkind
    c["source"]["width"] = "nominal, parsed from the material text" if w else None
    # flow history from the overlay FAN with the day's calibration
    ovl = (d.get("overlay") or {}).get("series") or []
    pts = None
    try:
        day, nres = int(row["gmt"]), int(float(row["restrictor"]))
        pts = cal.get(day, {}).get(nres)
    except (TypeError, ValueError, KeyError):
        pass
    c["fan_calibration"] = pts
    c["flow"] = [{"t_s": o["t_s"], "fan": o["fan"], "air": o["air"], "u_cm_s": u_of(o["fan"], pts)} for o in ovl]
    c["source"]["flow"] = ("video overlay FAN (1 per second), converted with the fan calibration workbook for this GMT day and "
                           "restrictor count; u_cm_s null outside the calibrated rows" if pts else
                           "video overlay FAN and AIR (1 per second); no calibration block for this day and restrictor count")
    return c


def direction(d, c):
    mat = str(c.get("material") or "").lower()
    if any(k in mat for k in ("sphere", "candle", "wax")):
        return None, "not applicable to this sample geometry (no spread along a strip)"
    if "oppos" in mat:
        return "opposed", "NASA as-run matrix text"
    if "concurrent" in mat:
        return "concurrent", "NASA as-run matrix text"
    sd = d.get("spread_direction")
    if sd is None or d.get("igniter") is None or d.get("quality") == "failed":
        return None, "not known"
    return ("concurrent" if "smaller x" in sd else "opposed"), "inferred from the igniter position in the video (flow runs right to left in this camera)"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", required=True)
    ap.add_argument("--only", default=None)
    args = ap.parse_args()
    root = Path(args.root)
    tab, cal = table25(), calibration()
    ar = asrun26() if ASRUN.exists() else []
    burns = []
    for f in sorted((root / "tracks").rglob("*.json")):
        inv = "PSI-25" if "psi25" in f.parts else "PSI-26"
        if args.only and inv != args.only:
            continue
        d = json.loads(f.read_text(encoding="utf-8"))
        if inv == "PSI-25":
            c = enrich25(d, tab, cal)
        else:
            name = d["source"]["video"]
            m = re.search(r"(\d{4})(\d\d)(\d\d) (\d\d)[. ](\d\d)", name)
            c = {"source": {}}
            if m:
                vdate = dt.date(int(m.group(1)), int(m.group(2)), int(m.group(3)))
                vstart = int(m.group(4)) * 60 + int(m.group(5))
                dur = (d.get("record") or {}).get("duration_s") or 0
                hits = [r for r in ar if r["video_date"] == vdate and r["start"] is not None and vstart - 5 <= r["start"] <= vstart + dur / 60 + 5]
                c["date_on_video"] = vdate.isoformat()
                c["date"] = vdate.isoformat()
                c["actual_date_as_run"] = (hits[0]["actual_date"].isoformat() if hits and hits[0]["actual_date"] else None)
                c["source"]["date"] = ("date on the video file name, consistent within a session; the as-run actual date of the "
                                       "matched row is often one day earlier (actual_date_as_run)")
                c["as_run_matches"] = [{k: (str(v) if isinstance(v, (dt.date, dt.datetime)) else v) for k, v in r.items()
                                        if k not in ("video_date", "actual_date", "start")} for r in hits]
                one = hits[0] if len(hits) == 1 else None
                c.update(material=(one or {}).get("material"), restrictor=(one or {}).get("restrictor"),
                         x_o2_initial=((one or {}).get("o2")), pressure_atm=(one or {}).get("pressure_atm"),
                         fan_display_table=(one or {}).get("fan_display"), air_display_table=(one or {}).get("air_display"))
                c["source"]["as_run"] = (f"{len(hits)} as-run row(s) start inside this video; conditions filled only when exactly one does "
                                         "(match by approximate start time, GMT)")
            ovl = (d.get("overlay") or {}).get("series") or []
            ovl_ok = not any(str(x).startswith("R5") for x in d.get("qc_flags", []))
            c["flow"] = [{"t_s": o["t_s"], "fan": o["fan"], "air": o["air"], "u_cm_s": None} for o in ovl] if ovl_ok else []
            m_air = re.match(r"\s*(\d+(?:\.\d+)?)", str(c.get("air_display_table") or ""))
            c["u_cm_s_table_first"] = float(m_air.group(1)) if m_air else None
            c["source"]["flow"] = ("as-run matrix 'Air display' (cm/s, first value of the listed sequence; the sequence is the "
                                   "as-run flow history as text in air_display_table); "
                                   + ("video overlay FAN/AIR per second" if ovl_ok else
                                      "the video overlay could not be read (not registered in these 640x360 recordings)"))
            w, wkind = width_of(c.get("material"))
            c["width_m"], c["width_kind"] = w, wkind
        dirn, dsrc = direction(d, c)
        c["flow_direction"], c["source"]["flow_direction"] = dirn, dsrc
        ig = d.get("igniter") or {}
        c["igniter_on_t_s"], c["igniter_off_t_s"] = ig.get("on_t_s"), ig.get("off_t_s")
        c["source"]["igniter"] = "measured from the video (time origin = igniter off)"
        d["conditions"] = c
        f.write_text(json.dumps(d, indent=1), encoding="utf-8")
        flow0 = next((x["u_cm_s"] for x in c.get("flow", []) if x["t_s"] >= 0 and x["u_cm_s"] is not None), None)
        fan0 = next((x["fan"] for x in c.get("flow", []) if x["t_s"] >= 0 and x["fan"] is not None), None)
        burns.append({"investigation": inv, "burn_id": d["run_id"], "file_role": d.get("file_role", "primary"),
                      "date": c.get("date"), "partition": (d.get("partition") if inv == "PSI-25" else None),
                      "material": c.get("material") or d.get("material"), "flow_direction": dirn,
                      "width_m": c.get("width_m"), "width_kind": c.get("width_kind"), "x_o2_initial": c.get("x_o2_initial"),
                      "x_o2_final": c.get("x_o2_final"), "pressure_atm": c.get("pressure_atm"),
                      "fan_at_igniter_off": fan0, "u_cm_s_at_igniter_off": flow0 if inv == "PSI-25" else c.get("u_cm_s_table_first"),
                      "flow_source": ("overlay FAN + fan calibration" if inv == "PSI-25" else "as-run matrix Air display (first value)"),
                      "actual_date_as_run": c.get("actual_date_as_run"),
                      "as_run_tests": [h.get("as_run_test") for h in c.get("as_run_matches", [])] if inv == "PSI-26" else None,
                      "track": str(f.relative_to(root)).replace("\\", "/"), "quality": d.get("quality")})
    out = root / "BURNS.jsonl"
    keep = []
    if out.exists() and args.only:
        keep = [l for l in out.read_text(encoding="utf-8").splitlines() if json.loads(l)["investigation"] != args.only]
    out.write_text("\n".join(keep + [json.dumps(b) for b in burns]) + "\n", encoding="utf-8")
    print(f"{len(burns)} burns enriched")


if __name__ == "__main__":
    main()
