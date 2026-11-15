"""Freeze the fresh blind n-heptane droplet batch (FLEX-2 2013-03-28, PSI-69 CO2 days of 2013).

Conditions, configuration and file sizes only: no frame is downloaded or opened and no outcome is read.
Writes docs/wsp/atlas/splits/heptane-fresh-blind.json, which also fixes the measurement method (tool and
its sha256, pixel scale and its source, time origin, acceptance rule, screens), so that nothing about
the measurement is chosen after a frame is seen.

    python -I tools/wsp/media/heptane_fresh_freeze.py

Test list (from RESULTS_HEPTANE_D3.md section 6, re-derived here from the raw text files):
  * PSI-68 (FLEX-2) day 2013-03-28: E087C01-C04, C06-C09, n-heptane (PSI data record "Heptane"),
    settings: no fibre, no flow; no gas reading that day (ambient carried, see AMBIENT_E087);
  * PSI-69 (FLEX) points of 2013 named C<day>H<nn>: days 021, 030, 039, 049 (23 points). Configuration
    from each test's MDCA settings (TP_PARAMETER_FIBER_USAGE_FLAG); ambient from the gas-reading points
    C<day>R01 (`_Mixture.txt`) of the same and neighbouring days; pressure from the chamber readings.
  * Fibre-supported points (settings flag Yes: 7 of the 23) are listed but NOT in the measured batch:
    configuration, decided before any frame (the heptane releases are free-droplet models).
"""
import csv
import hashlib
import json
import re
from datetime import datetime
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
RAW = REPO / "runs/dev/data_all/raw"
OUT = REPO / "docs/wsp/atlas/splits/heptane-fresh-blind.json"
TOOL = REPO / "tools/wsp/media/droplet_diameter.py"
PSI = 6894.757293168  # Pa per psi

E087 = ["E087C01", "E087C02", "E087C03", "E087C04", "E087C06", "E087C07", "E087C08", "E087C09"]
C2013 = ["C021H02", "C021H03", "C021H04", "C021H05",
         "C030H01", "C030H02", "C030H03", "C030H04", "C030H05", "C030H06", "C030H07",
         "C039H01", "C039H02", "C039H03", "C039H04", "C039H05", "C039H06", "C039H07", "C039H08",
         "C049H01", "C049H02", "C049H03", "C049H05"]


def psi69(tp, suffix):
    return RAW / "PSI-69/Raw Data" / f"PSI-69_Raw Data_{tp}{suffix}"


def psi68(tp, suffix):
    return RAW / "PSI-68/Raw Data" / f"{tp}{suffix}"


def text_path(tp, suffix):
    return psi68(tp, suffix) if tp.startswith("E") else psi69(tp, suffix)


def sha(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()


def settings(tp):
    p = text_path(tp, "_MDCA_Testpoint_Settings.txt")
    rows = list(csv.reader(p.open(encoding="utf-8", errors="replace")))
    return dict(zip([h.split("(")[0] for h in rows[0]], rows[1])), p


def timeline(tp):
    p = text_path(tp, ".txt")
    ev = {}
    for line in p.read_text(encoding="utf-8", errors="replace").splitlines():
        if ":" in line and line.startswith("AS "):
            k, v = line.split(":", 1)
            v = v.strip()
            try:
                ev[k.strip()] = datetime.strptime(v, "%Y-%m-%d %H:%M:%S.%f")
            except ValueError:
                pass
    return ev, p


def chamber_pressure(tp, t_ign, window_s=60.0):
    """Mean of PT16 (psia) and mean TM1 (C) over chamber-reading rows in the window_s before igniter on."""
    p = text_path(tp, "_Chamber_Readings.txt")
    rows = list(csv.reader(p.open(encoding="utf-8", errors="replace")))
    head = rows[0]
    i16, itm = head.index("FCU_MP_PT16(PSIA)"), head.index("FCU_MP_TM1(Deg C)")
    vals, tms = [], []
    for r in rows[1:]:
        try:
            t = datetime.strptime(r[0], "%Y-%m-%d %H:%M:%S.%f")
        except ValueError:
            continue
        dt = (t_ign - t).total_seconds()
        if 0 <= dt <= window_s:
            vals.append(float(r[i16]))
            tms.append(float(r[itm]))
    if not vals:
        return None, None, p, 0
    return sum(vals) / len(vals), sum(tms) / len(tms), p, len(vals)


def mixture(point):
    p = psi69(point, "_Mixture.txt") if point.startswith("C") else psi68(point, "_Mixture.txt")
    lines = [l for l in p.read_text(encoding="utf-8", errors="replace").splitlines() if l.strip()]
    head = lines[0].split(",")
    # one file (C049R01) has spaces instead of the comma after the time of day
    raw = re.sub(r"^(\S+)\s+(\d+:\d+(?::\d+)?)\s+(?=\d)", r"\1 \2,", lines[1].strip())
    vals = raw.split(",")
    d = {h: v for h, v in zip(head, vals)}
    return dict(point=point, file=str(p.relative_to(REPO)).replace("\\", "/"), sha256=sha(p),
                gc_time_as_written=d["Date/Time"],
                x_o2=float(d["O2"].rstrip("%")) / 100, x_co2=float(d["CO2"].rstrip("%")) / 100,
                x_n2=float(d["N2"].rstrip("%")) / 100, x_he=float(d["He"].rstrip("%")) / 100)


def reading_point_time(point):
    """The chamber-readings start of a gas-reading point (its own time, independent of the GC date)."""
    p = psi69(point, "_Chamber_Readings.txt") if point.startswith("C") else psi68(point, "_Chamber_Readings.txt")
    rows = list(csv.reader(p.open(encoding="utf-8", errors="replace")))
    t = rows[1][0]
    vals = [float(r[1]) for r in rows[1:] if len(r) > 1]
    return t, sum(vals) / len(vals)


# Ambient per day, decided from the reading points (values are copied from the files by mixture()).
DAY_AMBIENT = {
    "021": dict(points=["C021R01", "C024R01"], grade="bracketed",
                how="reading point C021R01 (chamber 16:59 GMT, 2.8 h before the first test; GC 13:14) and "
                    "C024R01 (GC dated 2013-01-21 20:45, after the last test of the day at 20:43; point run "
                    "2013-01-24). Both give O2 0.21, CO2 0.15: the value is the mean of the two."),
    "030": dict(points=["C030R01", "C031R01"], grade="conflicting",
                how="the two readings disagree on the diluent: C030R01 (point run 19:29 GMT, 48 min before the "
                    "first test; its GC line is dated 2013-01-25 00:50) gives CO2 0.50, N2 0.29; C031R01 (GC "
                    "dated 2013-01-31 00:17, two minutes after the last test of the session began; point run "
                    "2013-01-31 20:12) gives CO2 0.75, N2 0.04. Chamber pressure is 14.51 psia before and "
                    "14.53 psia after, so no venting is evident. O2 0.21 in both. Not resolved: the day is "
                    "measured but its CO2 fraction is carried as unresolved (both values given)."),
    "039": dict(points=["C039R01"], grade="single reading",
                how="reading point C039R01 (chamber 21:33 GMT, 20 min before the first test, 43.4 psia); its GC "
                    "line is dated 2013-02-01 03:42 (a week earlier; the 3 atm fill is not the 1 atm gas of "
                    "2013-01-31, so it is a fill made after that day). O2 0.2125, CO2 0.4938. No reading "
                    "after the session."),
    "049": dict(points=["C049R01", "C052R01"], grade="bracketed",
                how="C049R01 (GC 2013-02-18 15:28, point 17:09, 41 min before the first test) and C052R01 (GC "
                    "dated 2013-02-18 19:43, two minutes after the last test; point run 2013-02-21) agree: "
                    "O2 0.21, CO2 0.75. Mean of the two."),
}

AMBIENT_E087 = (
    "ASSUMPTION, not a reading: no gas reading on 2013-03-28. Carried from the last reading before it, "
    "E073R01 (PSI-68, 2013-03-14 04:52 GMT: O2 0.2113, N2 0.7887, no CO2, no He), 14 days earlier. "
    "Support: no test point or reading point of PSI-68 or PSI-69 is recorded between LCTF073 (2013-03-14 "
    "09:00) and E087C01 (2013-03-28 00:09); the chamber pressure of the first 2013-03-28 test (14.975 psia "
    "at 22.9 C) equals that of the last 2013-03-14 test (15.041 psia at 24.7 C) referred to 22.9 C by the "
    "ideal gas law (14.95 psia), i.e. no vent and refill is evident. A refill to the same pressure with "
    "another gas cannot be excluded from the files. The O2 fraction carries the consumption of the "
    "2013-03-14 burns (a few tenths of a percent at most) unknown.")


def main():
    tests = []
    for tp in E087 + C2013:
        st, st_p = settings(tp)
        ev, tl_p = timeline(tp)
        ign = ev.get("AS Burn to Completion/Igniter On")
        p_psia, t_c, ch_p, n = chamber_pressure(tp, ign)
        fibre = st["TP_PARAMETER_FIBER_USAGE_FLAG"].strip()
        flows = (st["TP_PARAMETER_FLOW_NOZZLE_1_SPEED"], st["TP_PARAMETER_FLOW_NOZZLE_2_SPEED"])
        inv = "PSI-68" if tp.startswith("E") else "PSI-69"
        date = ign.strftime("%Y-%m-%d")
        rec = dict(test_id=tp, investigation=inv, date=date, igniter_on_gmt=ign.isoformat(sep=" "),
                   fuel="n-heptane",
                   fuel_how=("PSI data record 'Heptane' (flex2_tests.jsonl)" if inv == "PSI-68" else
                             "test-point name C<day>H<nn> (FLEX naming: H = n-heptane; the C-series gas "
                             "readings record Heptane vapour traces only)"),
                   configuration="fibre-supported" if fibre == "Yes" else "free droplet",
                   configuration_how=f"{st_p.name}: TP_PARAMETER_FIBER_USAGE_FLAG={fibre}, flow nozzles {flows}",
                   planned_burn_ms=float(st["TP_PARAMETER_BURN_TO_COMPLETION_TIME"]),
                   igniter_on_ms=float(st["TP_PARAMETER_IGNITER_ON_TIME"]),
                   pressure=dict(value_pa=round(p_psia * PSI, 1), raw_psia=round(p_psia, 3), chamber_temp_c=round(t_c, 2),
                                 how=f"mean FCU_MP_PT16 over the {n} chamber-reading rows in the 60 s before "
                                     f"igniter on ({ch_p.name})", evidence="original_observation"),
                   archive=f"{tp}A.zip",
                   listing_file=f"{inv}_Raw Data_{tp}A.zip")
        if inv == "PSI-68":
            rec["ambient"] = dict(x_o2=0.2113, x_co2=0.0, x_n2=0.7887, x_he=0.0, grade="assumed (carried 14 days)",
                                  how=AMBIENT_E087, readings=[mixture("E073R01")], evidence="assumption")
        else:
            day = tp[1:4]
            a = DAY_AMBIENT[day]
            rd = [mixture(p) for p in a["points"]]
            for r, p in zip(rd, a["points"]):
                r["chamber_point_start"], r["chamber_point_mean_psia"] = reading_point_time(p)
            amb = dict(grade=a["grade"], how=a["how"], readings=rd, evidence="original_observation")
            if a["grade"] == "conflicting":
                amb.update(x_o2=round(sum(r["x_o2"] for r in rd) / len(rd), 4), x_co2=None, x_n2=None, x_he=0.0,
                           x_co2_alternatives=[round(r["x_co2"], 4) for r in rd], x_n2_alternatives=[round(r["x_n2"], 4) for r in rd])
            else:
                for k in ("x_o2", "x_co2", "x_n2", "x_he"):
                    amb[k] = round(sum(r[k] for r in rd) / len(rd), 4)
            rec["ambient"] = amb
        rec["text_sources"] = {k: dict(file=str(p.relative_to(REPO)).replace("\\", "/"), sha256=sha(p))
                               for k, p in (("settings", st_p), ("timeline", tl_p), ("chamber", ch_p))}
        tests.append(rec)
    listing = {}
    for inv in ("PSI-68", "PSI-69"):
        for line in (REPO / f"runs/dev/data_all/manifests/{inv}.listing.jsonl").open(encoding="utf-8"):
            r = json.loads(line)
            listing[r["file_name"]] = r
    for t in tests:
        row = listing.get(t["listing_file"])
        t["archive_bytes"] = row and row["size_bytes"]
        # measured batch: free droplets; order = conditions grade (air assumption first: plain air is the
        # most valuable regime, then bracketed CO2 days, then the single-reading and conflicting days)
        t["in_measured_batch"] = t["configuration"] == "free droplet"
    order = {"assumed (carried 14 days)": 0, "bracketed": 1, "single reading": 2, "conflicting": 3}
    tests.sort(key=lambda t: (not t["in_measured_batch"], order[t["ambient"]["grade"]], t["test_id"]))
    method = dict(
        tool="tools/wsp/media/droplet_diameter.py", tool_sha256=sha(TOOL),
        tool_frozen_by="runs/dev/media/droplet/FREEZE.json (same sha256; validated against NASA's processed "
                       "diameters on six FLEX train tests; used unchanged, never edited)",
        invocation="droplet_diameter.run(<staged dir>, investigation='FLEX', fov_mm=30.25, sheet_dir=...) "
                   "via tools/wsp/media/heptane_fresh_measure.py",
        pixel_scale=dict(
            rule="field of view 30.25 mm over the image width in pixels, passed explicitly (fov_mm=30.25); "
                 "33.85 px/mm on a 1024 px image, 16.93 on a 512 px binned image",
            source="FLEX HiBMs documented field of view 30.25 mm (TP15 section 3.1.2), verified against NASA's "
                   "processed diameters on six FLEX train tests (PILOT_REPORT.md); for FLEX-2 the requirements "
                   "document FLX-REQ-0001 4.1.56/4.1.57 (droplet view 3.0 cm, 30 um per pixel) and the "
                   "needle-arm width (53.9 px on 47 FLEX-2 tests, 54.4 px on FLEX) "
                   "(runs/dev/media/droplet/SCALE_FLEX2.md). The 2013 PSI-69 points use the same MDCA/CIR "
                   "hardware between FLEX-2 days. Not set from any droplet size, rate or expected diameter.",
            uncertainty="1.5 % (1 sigma) on the scale; worst case -7 % (SCALE_FLEX2.md section 3)",
            check="the tool's needle-arm ruler_check is recorded per test; it is a check, never used to rescale"),
        time_origin=dict(
            stamp="AS Igniter Off/Retract Start", rule="time_s = camera stamp - timeline 'AS Igniter Off/Retract "
            "Start' (as in FREEZE.json time_origin: NASA's processed FLEX t = 0 sits -0.31 to +0.41 s from it)"),
        acceptance=dict(
            rule="FREEZE.json acceptance, unchanged: clock check within 0.3 s, >= 30 usable (ok, soot) frames "
                 "after deployment + 0.1 s, coverage >= 0.6 of the measured span",
            screens="as flex2_batch_summary.py: 'failed' if the tool saw no droplet in > 95 % of post-deployment "
                    "frames; flex2_truncation_screen.py truncated or hijacked track; a visual check of the "
                    "contact sheets by the measurer (fibre or needle seen, track on another object, droplet "
                    "out of view) can only fail a series, never pass one the rule refused",
            exclusions="by measurement quality only, written with their reason; never by any model output"),
        output="runs/dev/heptane_fresh/series/<test>.json: liquid_diameter observations in m on the stated time "
               "base, per-frame tool flags, conditions as frozen here, quality and reasons")
    doc = dict(schema="ignis.atlas.batch/1", batch="heptane-fresh-blind", frozen_on="2026-11-15",
               frozen_before="any frame of these tests was downloaded or opened; no outcome of these tests exists "
                             "in the project (no NASA series, no test sheet)",
               purpose="blind diameter series for scoring a frozen n-heptane candidate against the adopted release",
               rule=__doc__.split("Test list")[1].split('"""')[0].strip(),
               counts=dict(tests=len(tests), measured_batch=sum(t["in_measured_batch"] for t in tests),
                           fibre_excluded=sum(not t["in_measured_batch"] for t in tests),
                           measured_batch_bytes=sum(t["archive_bytes"] for t in tests if t["in_measured_batch"])),
               method=method, tests=tests)
    OUT.write_text(json.dumps(doc, indent=1) + "\n", encoding="utf-8")
    for t in tests:
        a = t["ambient"]
        print(t["test_id"], t["date"], t["configuration"], round(t["pressure"]["value_pa"] / 101325, 3), "atm",
              a["x_o2"], a["x_co2"], a.get("x_co2_alternatives"), a["grade"], t["archive_bytes"])
    print(json.dumps(doc["counts"]))


if __name__ == "__main__":
    main()
