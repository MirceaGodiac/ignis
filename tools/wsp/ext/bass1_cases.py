"""Cases for BASS (PSI-26, slug bass1) and the non-fabric burns of BASS-II (PSI-25, slug bass2_solids).

Both investigations burned solid samples in the small flow duct of the Microgravity Science Glovebox. There is no
sensor file: the outcomes are words in NASA's as-run sheets. This script reads them, codes two outcomes with fixed
text rules, and writes the case files of docs/wsp/feedback/bass1_feasibility.md:

    runs/dev/data_all/work/transfer_bass1/cases_train.jsonl               train days: inputs and outcome
    runs/dev/data_all/work/transfer_bass1/cases_validation_inputs.jsonl   validation days: inputs only
    runs/dev/data_all/work/transfer_bass1/cases_validation_outcomes.jsonl kept apart, never printed
    runs/dev/data_all/work/transfer_bass1/fan_curve.json                  the frozen fan-to-speed map

Targets (one record per burn):
    quench  the duct air speed below which the flame goes out when the flow is lowered (the low-speed quenching
            branch of the flammability map), an interval [u_lo, u_hi] in cm/s: the flame burned at u_hi and was
            out at u_lo. u_lo = 0 when it went out only when the fan was turned off, or burned to completion.
            Score: interval CRPS from quantiles (2 x mean interval pinball loss at 5, 25, 50, 75, 95 %).
    sustain did the sample ignite and keep burning after the igniter was switched off (1) or not (0).
            Score: Brier.

Groups: PSI-26 2013 burns by the frozen video days (docs/wsp/atlas/splits/PSI-26-days.json; the as-run date is
one day before the video date); PSI-26 2012 burns, which have no video, by docs/wsp/feedback/bass1_PSI-26-2012-days.json;
PSI-25 solids by docs/wsp/atlas/splits/PSI-25-solids-days.json. All three were frozen from dates only, before
any outcome of their held days was read. A row of a test day is counted from its date and nothing else of it is read.

    python tools/wsp/ext/bass1_cases.py build        # writes the case files, prints train and counts only
    python tools/wsp/ext/bass1_cases.py baselines    # fits the baseline constants on train, scores train
    python tools/wsp/ext/bass1_cases.py show-train   # train burns: text beside the coded outcome (for review)
    python tools/wsp/ext/bass1_cases.py score --target quench --forecasts F.json --split train|validation
    python tools/wsp/ext/bass1_cases.py selftest
"""
import argparse
import datetime as dt
import json
import math
import re
import statistics
import sys
from collections import Counter, defaultdict
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
RAW = REPO / "runs/dev/data_all/raw"
OUT = REPO / "runs/dev/data_all/work/transfer_bass1"
AS_RUN_26 = RAW / "PSI-26/Science Documents/PSI-26_Science Documents_As Run_Test Matrix.xlsx"
MATRIX_25 = RAW / "PSI-25/Science Documents/PSI-25_Science Documents_BASS-II_Test.Matrices.xlsx"
REPORT_25 = RAW / "PSI-25/Reports/PSI-25_Reports_BASS-II Summary Report.pdf"
APPENDIX_PAGES = range(111, 122)  # Appendix A of the summary report: the as-run matrices with observations
FANCAL_25 = RAW / "PSI-25/Reference Data/PSI-25_Reference Data_Fan calibration data.xlsx"
SPLIT_26 = REPO / "docs/wsp/atlas/splits/PSI-26-days.json"
SPLIT_26_2012 = REPO / "docs/wsp/feedback/bass1_PSI-26-2012-days.json"
SPLIT_25 = REPO / "docs/wsp/atlas/splits/PSI-25-solids-days.json"

QUANTILES = (0.05, 0.25, 0.5, 0.75, 0.95)
FAN_OFF = 38.0  # fan display (volts x 10) at pot 0: the fan is stopped, no forced flow
# as-run date of the 2013 sheet -> date in the video names (column 'Date on videos and images'; the 1 June
# burns are in videos named 31 May, as the measurer's BURNS.jsonl maps them)
VIDEO_DAY_2013 = {"2013-04-04": "2013-04-05", "2013-04-05": "2013-04-06", "2013-04-09": "2013-04-10",
                  "2013-04-12": "2013-04-13", "2013-04-16": "2013-04-17", "2013-04-23": "2013-04-24",
                  "2013-04-26": "2013-04-27", "2013-04-29": "2013-04-30", "2013-05-02": "2013-05-03",
                  "2013-05-15": "2013-05-16", "2013-05-17": "2013-05-18", "2013-06-01": "2013-05-31"}


# ----------------------------------------------------------------------------------------------- partitions
def _groups(path):
    g = json.loads(path.read_text(encoding="utf-8"))["groups"]
    return {d: p for p, ds in g.items() for d in ds}


def partition_of(inv, date):
    """train / validation / test from the date alone; 'held' for a date no frozen split names."""
    if date is None:
        return "held"
    if inv == "PSI-26":
        if date.startswith("2012"):
            return _groups(SPLIT_26_2012).get(date, "held")
        return _groups(SPLIT_26).get(VIDEO_DAY_2013.get(date, ""), "held")
    return _groups(SPLIT_25).get(date, "held")


def norm_date(v):
    if isinstance(v, dt.datetime):
        return v.strftime("%Y-%m-%d").replace("2103-", "2013-")  # one row of the 2013 sheet is typed 2103
    m = re.match(r"\s*(\d{1,2})/(\d{1,2})/\D?(\d{2,4})", str(v or ""))
    if not m:
        return None
    y = int(m.group(3))
    return "%04d-%02d-%02d" % (2000 + y if y < 100 else y, int(m.group(1)), int(m.group(2)))


# ----------------------------------------------------------------------------------------------- fan curve
def _day_label(v):
    if isinstance(v, dt.datetime):
        return v.strftime("%Y-%m-%d")
    m = re.match(r"(\d{1,2})/(\d{1,2})/(\d{4})", str(v or ""))
    return "%s-%02d-%02d" % (m.group(3), int(m.group(1)), int(m.group(2))) if m else None


def calibrations():
    """The BASS-II fan calibrations (Reference Data workbook).

    Each day has a Pot / Fan Display / Air Display column triple; down a triple, the runs that start again at
    pot 10 are the 0, 1 and 2 restrictor blocks. U = 1.0066 x air display (the workbook's note). The campaign
    curve is the median over days per pot; below pot 1 the speed is linear in the fan display from (38, 0) to
    the lowest reading at pot >= 1 (the anemometer does not resolve the lowest settings); a day's own block is used when it has at least four readings and its
    highest reading is within a factor 0.6 to 1.6 of the campaign's at the same pot (one day, 29 July, carries a
    block whose values belong to another restrictor count). Curves are [(fan display, U cm/s), ...] from (38, 0)."""
    import openpyxl
    ws = openpyxl.load_workbook(FANCAL_25, data_only=True).worksheets[0]
    days = defaultdict(dict)
    pooled = defaultdict(lambda: defaultdict(list))
    fan_of_pot = defaultdict(list)
    for c in range(2, ws.max_column + 1):
        if ws.cell(6, c).value != "Pot":
            continue
        day = _day_label(ws.cell(3, c + 1).value)
        block, last, pts = 0, None, defaultdict(dict)
        for r in range(7, ws.max_row + 1):
            pot, fan, air = (ws.cell(r, c + k).value for k in range(3))
            if not isinstance(pot, (int, float)):
                continue
            if last is not None and pot > last:
                block += 1
            last = pot
            if isinstance(fan, (int, float)):
                fan_of_pot[float(pot)].append(float(fan))
            if isinstance(air, (int, float)) and block <= 2 and pot > 0:
                pts[block][float(pot)] = 1.0066 * float(air)
        for b, d in pts.items():
            for pot, u in d.items():
                pooled[b][pot].append(u)
            if day:
                days[day][b] = d  # a second triple of the same day (07-07 end, 07-31 p.m.) replaces the first
    pot_fan = sorted((p, statistics.median(v)) for p, v in fan_of_pot.items())
    fan_at = dict(pot_fan)

    def to_curve(d):
        # below pot 1 the anemometer reads 0 to 4 cm/s with a one-minute lag; there the speed is taken linear in
        # the fan display between the fan stopped (38, 0) and the lowest reading at pot >= 1
        xy = sorted([(FAN_OFF, 0.0)] + [(fan_at.get(p, FAN_OFF + 8.4 * p), u) for p, u in d.items() if p >= 1])
        mono = []
        for x, u in xy:
            if mono and x <= mono[-1][0]:
                continue
            mono.append((x, max(u, mono[-1][1] if mono else 0.0)))
        return mono

    campaign = {b: {p: statistics.median(us) for p, us in d.items() if len(us) >= 3} for b, d in pooled.items()}
    day_curves = {}
    for day, blocks in sorted(days.items()):
        for b, d in blocks.items():
            if len([p for p in d if p >= 1]) < 4 or b not in campaign:
                continue
            ptop = max(d)
            ref = campaign[b].get(ptop)
            if ref and 0.6 <= d[ptop] / ref <= 1.6:
                day_curves.setdefault(day, {})[str(b)] = to_curve(d)
    return {"campaign": {str(b): to_curve(d) for b, d in sorted(campaign.items())}, "days": day_curves,
            "fan_display_by_pot": pot_fan, "source": str(FANCAL_25.relative_to(REPO)),
            "rule": calibrations.__doc__.strip()}


def curve_for(inv, date, nres, fc):
    """The fan-to-speed curve of a burn and its source."""
    key = str(nres if nres in (0, 1, 2) else 2)
    if inv == "PSI-25":
        earlier = [d for d in fc["days"] if d <= (date or "") and key in fc["days"][d]]
        if earlier:
            d = max(earlier)
            return fc["days"][d][key], "BASS-II fan calibration of %s, %s restrictor(s)" % (d, key)
    return fc["campaign"][key], "median of the BASS-II 2014 fan calibrations, %s restrictor(s)%s" % (
        key, " (no calibration of the 2012-2013 BASS days is on disk; same duct and fan)" if inv == "PSI-26" else "")


def interp(x, xy):
    if x <= xy[0][0]:
        return xy[0][1]
    for (x0, y0), (x1, y1) in zip(xy, xy[1:]):
        if x <= x1:
            return y0 + (y1 - y0) * (x - x0) / (x1 - x0) if x1 > x0 else y1
    (x0, y0), (x1, y1) = xy[-2], xy[-1]
    return y1 + (y1 - y0) * (x - x1) / (x1 - x0)


def pot_to_fan(pot, fc):
    return interp(pot, fc["fan_display_by_pot"])


def fan_to_u(fan, curve):
    return max(0.0, interp(fan, curve))


def u_to_fan(u, curve):
    inv = []
    for x, y in curve:
        if not inv or y > inv[-1][0]:
            inv.append((y, x))
    return interp(u, inv)


# ----------------------------------------------------------------------------------------------- readers
def read_psi26():
    """Rows of the sheet 'as run matrix' (2012 and 2013). A row of a test or unsplit day yields only its date."""
    import openpyxl
    ws = openpyxl.load_workbook(AS_RUN_26, data_only=True, read_only=True)["as run matrix"]
    rows = list(ws.iter_rows(values_only=True))
    hdr = [str(h or "") for h in rows[2]]
    col = {k: next(i for i, h in enumerate(hdr) if h.startswith(k)) for k in (
        "Actual Date", "As run test no.", "Test no.", "Sample no.", "material", "Igniter", "Flow restrictor",
        "Sample distance", "Fan display", "Air display", "GN2 Pot", "O2 %", "pressure", "CO2", "Test Observations")}
    out, seen = [], Counter()
    for k, r in enumerate(rows[3:]):
        if not r or r[col["Actual Date"]] is None:
            continue
        date = norm_date(r[col["Actual Date"]])
        part = partition_of("PSI-26", date)
        rec = {"investigation": "PSI-26", "date": date, "partition": part, "row": k + 4}
        g = lambda key: r[col[key]]  # noqa: E731
        if g("material") is None or g("Test no.") is None:
            continue  # setup, dilution check, soot cleaning: not a burn (read from the material and test cells)
        if part in ("test", "held"):
            out.append(rec)  # counted from its date; nothing else of the row is kept
            continue
        test = str(g("Test no.")).strip()
        seen[(date, test)] += 1
        rec.update({
            "case_id": "PSI-26:%s:%s%s" % (date, test, "" if seen[(date, test)] == 1 else "#%d" % seen[(date, test)]),
            "as_run": str(g("As run test no.")), "test": test, "sample": str(g("Sample no.")).strip(),
            "material_raw": str(g("material")).strip(), "igniter": str(g("Igniter") or ""),
            "restrictors_raw": g("Flow restrictor"), "distance": str(g("Sample distance") or ""),
            "fan": str(g("Fan display") or ""), "air": str(g("Air display") or ""), "gn2": str(g("GN2 Pot") or ""),
            "x_o2": g("O2 %"), "p_atm": g("pressure"), "co2_mmhg": g("CO2"), "config_raw": "",
            "obs": str(g("Test Observations") or "")})
        out.append(rec)
    return out


def appendix_records():
    """Appendix A of the BASS-II summary report, row by row: {test id: (configuration cell, observations)}."""
    import pdfplumber
    recs = {}
    with pdfplumber.open(REPORT_25) as pdf:
        for pno in APPENDIX_PAGES:
            cur = None
            for tb in pdf.pages[pno - 1].extract_tables():
                for row in tb:
                    cells = [(x or "").replace("\n", " ").strip() for x in row]
                    if len(cells) > 5 and re.fullmatch(r"[BFPMOT]\d{1,2}[ab]?", cells[5]):
                        last = max(i for i, x in enumerate(cells) if x)
                        cfg = next((x for x in cells if x in ("Opposed", "Concurrent", "Stagnation", "?")), "")
                        cur = {"config": cfg, "obs_col": last, "obs": [cells[last]]}
                        recs[cells[5]] = cur
                    elif cur is not None and len(cells) > cur["obs_col"] and cells[cur["obs_col"]]:
                        cur["obs"].append(cells[cur["obs_col"]])
    return {k: (v["config"], " ".join(v["obs"])) for k, v in recs.items()}


def read_psi25():
    """Non-fabric rows of the BASS-II test matrices (every PI sheet; SIBAL rows are the fabric split's)."""
    import openpyxl
    wb = openpyxl.load_workbook(MATRIX_25, data_only=True, read_only=True)
    out, app = [], None
    for ws in wb.worksheets:
        rows = list(ws.iter_rows(values_only=True))
        for r in rows[1:]:
            if not r or r[2] is None:
                continue
            date = norm_date(r[0])
            part = partition_of("PSI-25", date)
            rec = {"investigation": "PSI-25", "date": date, "partition": part, "row": ws.title}
            if "sibal" in str(r[4]).lower():
                continue  # fabric: the project's fabric split owns it (read from the material cell)
            if part in ("test", "held"):
                out.append(rec)  # counted from its date; nothing else of the row is kept
                continue
            if app is None:
                app = appendix_records()
            tid = str(r[2]).strip()
            cfg, obs = app.get(tid, ("", ""))
            rec.update({
                "case_id": "PSI-25:%s" % tid, "as_run": "", "test": tid, "sample": str(r[3]).strip(),
                "material_raw": str(r[4]).strip(), "igniter": "", "restrictors_raw": r[5], "distance": "",
                "fan": str(r[6] if r[6] is not None else ""), "air": str(r[7] if r[7] is not None else ""), "gn2": "",
                "x_o2": (r[10] / 100.0) if isinstance(r[10], (int, float)) else None, "p_atm": None,
                "co2_mmhg": (r[12] * 7.6) if isinstance(r[12], (int, float)) else None,  # vol% at 1 atm -> mmHg
                "config_raw": cfg, "obs": obs})
            out.append(rec)
    return out


# ----------------------------------------------------------------------------------------------- inputs
INCH = 0.0254


def geometry(rec):
    """Material class and sizes from the material words (and, for PSI-26, the SRD's test-number ranges)."""
    m = rec["material_raw"].lower().replace("2ide", "wide").replace("blac ", "black ")
    g = {"material_class": None, "fuel": None, "shape": None, "diameter_m": None, "width_m": None,
         "thickness_m": None, "sides_burning": None, "rod_colour": None}
    num = lambda pat: (float(re.search(pat, m).group(1)) if re.search(pat, m) else None)  # noqa: E731
    if "sibal" in m:
        g.update(material_class="fabric_sibal", fuel="cotton-fiberglass fabric", shape="flat",
                 width_m=(num(r"(\d)\s*cm") or 2) / 100)
    elif "nomex" in m:
        g.update(material_class="fabric_nomex", fuel="Nomex", shape="flat", width_m=0.02 if "2 cm" in m or rec["investigation"] == "PSI-25" else None)
    elif "ultem" in m:
        g.update(material_class="ultem_sheet", fuel="Ultem", shape="flat")
    elif "candle" in m:
        g.update(material_class="candle", fuel="Japan wax" if "japan" in m else "paraffin", shape="candle in tube")
    elif "wax" in m:
        g.update(material_class="wax_slab", fuel="paraffin on fiberglass", shape="flat", width_m=0.01,
                 thickness_m=0.002, sides_burning=1)  # SRD: test points 74-77, 1-cm width, 2-mm thick, one-sided
    elif "rod" in m:
        frac = re.search(r"(\d)/(\d)", m)
        d = (int(frac.group(1)) / int(frac.group(2))) if frac else num(r"(0\.\d+)\s*\"")
        g.update(material_class="pmma_rod", fuel="PMMA", shape="cylinder", diameter_m=d * INCH if d else None,
                 rod_colour="black" if "black" in m else "clear")
    elif "spher" in m and "section" in m:
        g.update(material_class="pmma_spherical_section", fuel="PMMA", shape="spherical section", diameter_m=0.04)
    elif "sphere" in m or (rec["investigation"] == "PSI-26" and re.fullmatch(r"pmma\s*[12]\s*cm", m.strip())):
        g.update(material_class="pmma_sphere", fuel="PMMA", shape="sphere", diameter_m=(num(r"(\d)\s*cm") or 2) / 100)
    elif "pmma" in m or "film" in m or "slab" in m or "micron" in m:
        thick = num(r"(\d+)\s*micron") or num(r"(\d+)\s*µm")
        thick = thick * 1e-6 if thick else None
        if thick is None:
            mm = num(r"(\d+(?:\.\d+)?)\s*mm")
            thick = mm / 1000 if mm else (0.002 if rec["investigation"] == "PSI-26" else None)
        width = num(r"(\d)\s*cm(?:\s*wide|\s*2ide|,|\s|$)") or num(r"(\d)\s*cm")
        if width is None and rec["investigation"] == "PSI-26":
            width = 1  # SRD: 'Thick Flat, 1-cm Width, 2-mm Thick PMMA, One-Sided', test points 66-73
        sides = 1 if ("1 sided" in m or "one-sided" in m or "1 sided" in m.replace("-", " ")) else (
            2 if "2 sided" in m else (1 if rec["investigation"] == "PSI-26" else 2))
        g.update(material_class="pmma_film" if thick is not None and thick < 0.0005 else "pmma_slab", fuel="PMMA",
                 shape="flat", width_m=width / 100 if width else None, thickness_m=thick, sides_burning=sides)
    return g


def configuration(rec, geo):
    words = " ".join((rec["material_raw"], rec["igniter"], rec["distance"], rec["config_raw"])).lower()
    if "wake" in words:
        return "wake"
    for k in ("opposed", "concurrent", "stagnation"):
        if k in words or ("oppsed" in words and k == "opposed"):
            return k
    if geo["shape"] in ("sphere", "candle in tube"):
        return "forward stagnation"
    return None


def restrictors(rec):
    v = rec["restrictors_raw"]
    if isinstance(v, (int, float)):
        return int(v), "as run"
    return 2, "assumed (cell empty or unreadable; 2 is the setting of almost every low-flow burn)"


# ----------------------------------------------------------------------------------------------- flow settings
NUM = re.compile(r"(\d*\.\d+|\d+)")


def _numbers(cell):
    """Numbers of a cell with their surroundings; clock times (a number next to ':') are dropped."""
    s = cell.lower().replace("pots", "pot")
    out = []
    for m in NUM.finditer(s):
        if s[m.end():m.end() + 1] == ":" or s[max(0, m.start() - 1):m.start()] == ":":
            continue
        before, after = s[max(0, m.start() - 6):m.start()], s[m.end():m.end() + 6]
        inside = s[:m.start()].count("(") > s[:m.start()].count(")")
        out.append((float(m.group(1)), before, after, inside, "." in m.group(1), s[max(0, m.start() - 1):m.start()] == "<"))
    return s, out


def fan_cell_steps(cell, fc, curve):
    """Settings in the fan cell, as duct speed U (cm/s). >= 36 is a fan display, 0 the fan stopped, <= 12 a pot,
    except a whole number <= 12 in a cell whose other values are pots below 1: that is an air speed in cm/s
    (the BASS-II rod sheets write 'ign 5, 0.36, 0.32, ...')."""
    s, nums = _numbers(cell)
    small_pots = any(n[0] < 1 and n[4] for n in nums)
    steps = []
    for v, before, after, inside, dec, lt in nums:
        if v >= 36:
            steps.append(("fan %.4g" % v, fan_to_u(v, curve)))
        elif v == 0:
            steps.append(("fan off", 0.0))
        elif v <= 12:
            if not dec and small_pots and "pot" not in before and not after.strip().startswith("pot"):
                steps.append(("air %.3g cm/s" % v, 1.0066 * v))
            else:
                steps.append(("pot %.3g" % v, fan_to_u(pot_to_fan(v, fc), curve)))
    return steps


def air_cell_steps(cell, fc, curve):
    """Settings in the air cell: values after or before 'pot', or in brackets, are pots; the others air readings."""
    s, nums = _numbers(cell)
    steps = []
    for v, before, after, inside, dec, lt in nums:
        if "pot" in before or after.strip().startswith("pot") or inside or s.strip().startswith("pot"):
            if v <= 12:
                steps.append(("pot %.3g" % v, fan_to_u(pot_to_fan(v, fc), curve)))
        elif v <= 60:
            steps.append(("air %s%.3g cm/s" % ("<" if lt else "", v), 1.0066 * (v / 2 if lt else v)))
    return steps


def steps_of(rec, fc, curve, prefer_air=False):
    """The flow settings of a burn in the order written, as U (cm/s). The burn's own anemometer reading is used
    for a step when the fan and air cells list the same number of steps and the reading is at least 2 cm/s
    (the anemometer is slow and coarse below that); otherwise the fan setting through the fan curve."""
    f = fan_cell_steps(rec["fan"], fc, curve)
    a = air_cell_steps(rec["air"], fc, curve)
    a_plain = [x for x in a if x[0].startswith("air")]
    a_pot = [x for x in a if x[0].startswith("pot")]
    if prefer_air and a_plain:
        return a_plain + a_pot
    if f and len(f) == len(a_plain):
        return [(fa[0] + " / " + aa[0], aa[1]) if aa[1] >= 2 else fa for fa, aa in zip(f, a_plain)] + a_pot
    if f:
        return f + a_pot
    return a


# ----------------------------------------------------------------------------------------------- inputs
def inputs_of(rec, fc, prior):
    geo = geometry(rec)
    nres, nres_src = restrictors(rec)
    curve, curve_src = curve_for(rec["investigation"], rec["date"], nres, fc)
    st = steps_of(rec, fc, curve)
    x = rec["x_o2"] if isinstance(rec["x_o2"], (int, float)) else None
    p = rec["p_atm"] if isinstance(rec["p_atm"], (int, float)) else None
    return {
        "case_id": rec["case_id"], "investigation": rec["investigation"], "date": rec["date"],
        "day_group": rec["date"], "test": rec["test"], "sample": rec["sample"], "material_raw": rec["material_raw"],
        **geo, "configuration": configuration(rec, geo),
        "x_o2_initial": round(x, 4) if x is not None else None,
        "x_o2_source": ("as-run 'O2 % by volume' (fraction)" if rec["investigation"] == "PSI-26"
                        else "test matrix 'calibrated initial O2 % by vol' / 100"),
        "pressure_atm": round(p, 4) if p is not None else None,
        "co2_mmhg": round(rec["co2_mmhg"], 2) if isinstance(rec["co2_mmhg"], (int, float)) else None,
        "flow_restrictors": nres, "flow_restrictors_source": nres_src,
        "ignition_setting": st[0][0] if st else None,
        "u_cm_s_at_ignition": round(st[0][1], 3) if st else None,
        "prior_burns_of_sample": prior,
        "fan_curve": curve, "fan_curve_source": curve_src,
    }


# ----------------------------------------------------------------------------------------------- outcome rules
NOT_A_TEST = re.compile(r"igniter (broke|seems to have broken|failed) (before|prior to) ignition|"
                        r"igniter broke after (third|3rd|\w+) attempt|no go\b")
NOT_SUSTAINED = re.compile(
    r"no ignition|did not ignite|not ignite|would not sustain|no sustained flame|brief ignition|ignited briefly|"
    r"only brief|died when igniter|went out when igniter|died immediately upon igniter|while the igniter was still on|"
    r"brief flame a few times|burned for only|no joy(?![^.]*n2)")
BURNING = re.compile(r"ignit(ed|ion)|burn|flame|spread|soot|blue|bubble|extingu|extinct|quench|vapor jetting|"
                     r"\bout\b|\bext\b|blow|blew")
BLOWOFF = re.compile(r"blow\s?off|blew (out|off)|blown (out|off)")
NO_BLOWOFF = re.compile(r"did not blow ?(off|out)|didn't blow|not blow ?off")
N2 = re.compile(r"\bg?n2 (on|max|to max)|gn2 on|used gn2|turned on n2|n2 and air off|turned up gn2|n2 off")
EXT = re.compile(r"extingui\w*|extinction|quench\w*|went out|goes out|go out|flame out|flashed out|\bout\b|\bext\b")
OFF = re.compile(r"(fan|flow|air)( was)? (turned )?off|turned off (the )?(air ?)?(flow|fan)|shut off flow|"
                 r"turning off flow|fan goes to 38|flow turned off")
COMPLETE = re.compile(r"burned (to completion|entire sample|out (the )?(whole )?sample)|burned out (the )?(whole )?sample|"
                      r"fuel ran out|out of wax")
SELF = re.compile(r"by itself|self-extinguish|on its own")
KEY = re.compile(r"\bat\b|\bto\b|when|reached|\bof\b|\bpot\b|\bfan\b|~|change")


def _value(v, unit, ctx, fc, curve):
    if unit == "cm/s" or "air reading" in ctx or "cm/s" in ctx:
        return 1.0066 * v, "air reading %.3g cm/s" % v
    if v <= 12:
        return fan_to_u(pot_to_fan(v, fc), curve), "pot %.3g" % v
    if 36 <= v <= 125:
        return fan_to_u(v, curve), "fan %.4g" % v
    return None


def extinction_value(text, fc, curve, cell=False):
    """(U cm/s, words) of the setting at which the text says the flame went out, or None.

    After an extinction word: the first number within 40 characters, with no sentence break, clock time or other
    number before it, and (in the observations) a word such as 'at', 'to', 'when', 'pot' or 'fan' between; a
    range 'a-b' gives b. Before it (fan cells: '0.36 extinction'; observations: 'down to 40.5ish flame
    extinguished'). Negated words ('did not extinguish') are skipped, 'not go out until ...' is read."""
    t = text.lower().replace("pots", "pot")
    for m in EXT.finditer(t):
        word = m.group(0)
        if not cell and word in ("out", "ext") and not re.match(r"\s*(at|when)\b", t[m.end():m.end() + 6]):
            continue
        head = t[max(0, m.start() - 14):m.start()]
        tail = t[m.end():m.end() + 40]
        negated = re.search(r"(not|n't)\s+(\w+\s+)?$", head) and not tail.strip().startswith("until")
        if negated:
            continue
        mm = re.match(r"([^\d:;.]{0,34}?)(\d*\.\d+|\d+)(?!\.?\d)(?:\s*-\s*(\d*\.\d+|\d+)(?!\.?\d))?(?!\s*(?::|min|s\b|sec|ohm))\s*(cm/s|pot)?",
                      tail)
        if mm and (cell or KEY.search(mm.group(1))) and not re.search(r"increas|\bup\b|after \w+ \w+ at$", mm.group(1)):
            v = float(mm.group(3) or mm.group(2))
            got = _value(v, mm.group(4), mm.group(1), fc, curve)
            if got and mm.group(3):  # 'on last flow change pot 0.35-0.32': held a, out at b
                held = _value(float(mm.group(2)), mm.group(4), mm.group(1), fc, curve)
                return got[0], got[1] + " (from %s)" % held[1], held[0]
            if got:
                return got
        hb = t[max(0, m.start() - 32):m.start()]
        mh = (re.search(r"(?<![\d.:])(\d*\.\d+|\d+)\s*,?\s*$", hb) if cell else
              re.search(r"(?:down to|to)\s*~?(\d*\.\d+|\d+)\s*(cm/s)?\s*(?:ish)?\s*\(?\s*(?:flame\s+)?$", hb))
        if mh:
            got = _value(float(mh.group(1)), (mh.group(2) if not cell and mh.lastindex and mh.lastindex >= 2 else None), "", fc, curve)
            if got:
                return got
    return None


def outcome_of(rec, inp, fc):
    """Code the two outcomes from the words. Returns a dict with the codes and the rule that fired."""
    text = " ".join((rec["obs"], "| fan:", rec["fan"], "| air:", rec["air"])).lower()
    obs = rec["obs"].lower()
    curve = inp["fan_curve"]
    o = {"case_id": rec["case_id"], "day_group": rec["date"], "sustain": None, "sustain_rule": None,
         "quench_class": None, "quench_u_lo_cm_s": None, "quench_u_hi_cm_s": None, "quench_rule": None,
         "n2_applied": bool(N2.search(obs)) and "vitiat" not in obs}
    if NOT_A_TEST.search(obs) and not re.search(r"ignited (at|on|after|within|quickly|nicely|very)", obs):
        o["sustain_rule"] = "not a test: igniter failed before any flame"
        return o
    if NOT_SUSTAINED.search(obs):
        o.update(sustain=0, sustain_rule="words: no sustained flame", quench_class="not_sustained")
        return o
    if not BURNING.search(text):
        o["sustain_rule"] = "no words about the flame"
        return o
    o["sustain"], o["sustain_rule"] = 1, "words: burning"
    if BLOWOFF.search(text) and not NO_BLOWOFF.search(text):
        o["quench_class"], o["quench_rule"] = "blowoff", "high-flow blowoff: not the low-speed branch"
        return o
    ev = extinction_value(obs, fc, curve) or extinction_value(rec["fan"], fc, curve, cell=True) or \
        extinction_value(rec["air"], fc, curve, cell=True)
    steps = steps_of(rec, fc, curve, prefer_air=bool(ev and ev[1].startswith("air")))
    if "didn't complete" in obs or "did not complete" in obs:
        steps = steps[:1]  # the change of flow was not finished before the burn ended
    us = [u for _, u in steps]
    off = OFF.search(obs) and re.search(r"extingu|out|flame", obs)
    if ev is not None:
        lo, why = ev[0], ev[1]
        if len(ev) > 2:
            us = [u for u in us if u > ev[2] * 1.0001] + [ev[2]]  # the range names the setting held last
        above = [u for u in us if u > lo * 1.0001 + 1e-9]
        if not above:
            o["quench_class"], o["quench_rule"] = "out_no_upper", "out at %s; no higher setting held before" % why
            return o
        hi, history = min(above), "monotone"
        if us and lo > min(us) + 1e-9:
            # a lower setting was held earlier (during ignition, or before the flow was raised): the flame held the
            # setting just before the one it went out at
            at = [k for k, u in enumerate(us) if abs(u - lo) < 1e-6]
            prev = [u for u in us[:at[-1]] if u > lo * 1.0001 + 1e-9] if at else above
            hi, history = (prev[-1] if prev else min(above)), "a lower setting was held earlier"
        o.update(quench_class="out_at_finite_flow", quench_u_lo_cm_s=round(lo, 3), quench_u_hi_cm_s=round(hi, 3),
                 quench_history=history, quench_rule="out at %s; held %.3g cm/s before" % (why, hi))
    elif off:
        held = [u for u in us if u > 0]
        if not held:
            o["quench_class"], o["quench_rule"] = "out_at_fan_off_no_step", "out when the fan was stopped; no step read"
            return o
        o.update(quench_class="out_at_fan_off", quench_u_lo_cm_s=0.0, quench_u_hi_cm_s=round(min(held), 3),
                 quench_rule="burned at every setting down to %.3g cm/s, out when the fan was stopped" % min(held))
    elif COMPLETE.search(obs):
        held = [u for u in us if u > 0]
        if not held:
            o["quench_class"], o["quench_rule"] = "burned_out_no_step", "burned to completion; no step read"
            return o
        o.update(quench_class="burned_to_completion", quench_u_lo_cm_s=0.0, quench_u_hi_cm_s=round(min(held), 3),
                 quench_rule="burned to completion; lowest setting %.3g cm/s" % min(held))
    elif SELF.search(obs):
        o["quench_class"], o["quench_rule"] = "self_extinguished", "went out by itself at a held flow; no value read"
    else:
        o["quench_class"], o["quench_rule"] = "end_not_stated", "the words do not say how the flame ended"
    return o


def quench_scored(o):
    """Cases that enter the quench score: an interval exists and nitrogen was not used during the burn."""
    return o.get("quench_u_hi_cm_s") is not None and not o.get("n2_applied")


# ----------------------------------------------------------------------------------------------- build
def load_all():
    fc = calibrations()
    recs = read_psi26() + read_psi25()
    prior = Counter()
    cases = []
    for rec in recs:
        if rec["partition"] in ("test", "held"):
            cases.append((rec, None, None))
            continue
        key = (rec["investigation"], rec["sample"])
        inp = inputs_of(rec, fc, prior[key])
        prior[key] += 1
        cases.append((rec, inp, outcome_of(rec, inp, fc)))
    return fc, cases


def write_jsonl(path, rows):
    with path.open("w", encoding="utf-8", newline="\n") as f:
        for r in rows:
            f.write(json.dumps(r, sort_keys=True) + "\n")


def cmd_build(_):
    OUT.mkdir(parents=True, exist_ok=True)
    fc, cases = load_all()
    (OUT / "fan_curve.json").write_text(json.dumps(fc, indent=1) + "\n", encoding="utf-8", newline="\n")
    train = [dict(inp, outcome=o) for rec, inp, o in cases if rec["partition"] == "train"]
    val_in = [inp for rec, inp, o in cases if rec["partition"] == "validation"]
    val_out = [o for rec, inp, o in cases if rec["partition"] == "validation"]
    write_jsonl(OUT / "cases_train.jsonl", train)
    write_jsonl(OUT / "cases_validation_inputs.jsonl", val_in)
    write_jsonl(OUT / "cases_validation_outcomes.jsonl", val_out)
    counts = defaultdict(Counter)
    for rec, inp, o in cases:
        stratum = rec["investigation"] + (" 2012" if (rec["date"] or "").startswith("2012") else
                                          " 2013" if rec["investigation"] == "PSI-26" else " solids")
        counts[stratum]["rows " + rec["partition"]] += 1
        if inp is not None:
            counts[stratum]["burns " + rec["partition"]] += 1
            counts[stratum]["sustain coded " + rec["partition"]] += o["sustain"] is not None
            counts[stratum]["quench scored " + rec["partition"]] += quench_scored(o)
            counts[stratum]["has x_o2 " + rec["partition"]] += inp["x_o2_initial"] is not None
    print(json.dumps({k: dict(sorted(v.items())) for k, v in sorted(counts.items())}, indent=1))
    print("train classes:", dict(Counter(c["outcome"]["quench_class"] for c in train)))
    print("written to", OUT.relative_to(REPO))
    return 0


def load_train():
    return [json.loads(line) for line in (OUT / "cases_train.jsonl").open(encoding="utf-8")]


# ----------------------------------------------------------------------------------------------- scores
def interval_pinball(q, tau, lo, hi):
    if q < lo:
        return tau * (lo - q)
    if q > hi:
        return (1 - tau) * (q - hi)
    return 0.0


def crps_interval(qs, lo, hi):
    """2 x mean pinball loss over the quantile levels, with the outcome an interval (0 inside it)."""
    return 2.0 * sum(interval_pinball(q, t, lo, hi) for t, q in zip(QUANTILES, qs)) / len(QUANTILES)


def score(forecasts, outcomes, target, day_of=None):
    """Score forecasts against outcomes.

    forecasts: {case_id: [q05, q25, q50, q75, q95] in cm/s} for target 'quench', {case_id: p} for 'sustain'.
    outcomes:  list of outcome dicts (the 'outcome' of a train case, or a row of cases_validation_outcomes).
    day_of:    {case_id: day group}; defaults to the date inside the case id.
    Returns the case mean, the mean over days of the day means (the primary aggregate), per-day scores,
    and the number of scored cases without a forecast (refusals)."""
    per_day, refused, n = defaultdict(list), 0, 0
    for o in outcomes:
        cid = o["case_id"]
        if target == "quench":
            if not quench_scored(o):
                continue
            y = (o["quench_u_lo_cm_s"], o["quench_u_hi_cm_s"])
        else:
            if o["sustain"] is None:
                continue
            y = o["sustain"]
        f = forecasts.get(cid)
        if f is None:
            refused += 1
            continue
        s = crps_interval(sorted(f), *y) if target == "quench" else (float(f) - y) ** 2
        day = (day_of or {}).get(cid) or o.get("day_group") or day_from_id(cid)
        per_day[day].append(s)
        n += 1
    days = {d: sum(v) / len(v) for d, v in sorted(per_day.items())}
    return {"target": target, "n_cases": n, "n_days": len(days), "refused": refused,
            "case_mean": sum(sum(v) for v in per_day.values()) / n if n else None,
            "day_mean": sum(days.values()) / len(days) if days else None, "per_day": days}


def day_from_id(cid):
    m = re.match(r"PSI-26:(\d{4}-\d{2}-\d{2})", cid)
    return m.group(1) if m else "?"


# ----------------------------------------------------------------------------------------------- baselines
def v_crit_bhattacharjee(x_o2, p_atm=None):
    """Critical opposed-flow velocity for radiative extinction, Bhattacharjee et al., Combust. Flame 163 (2016)
    472-477, as restated in NASA/TM-20210011385 section 2.2.7, eq. 2.5:
    V_g,cr = eps sigma (Tv^4 - Tinf^4) / (rho_g(Tv) c_g (Tf - Tv)), Tv = 673 K, Tf = 2199 K at 21 % O2 (the
    report's linearized flame temperature). Not given on the pages read and set here: eps = 1, Tinf = 300 K,
    c_g = 1070 J/kg/K (air at Tv), Tf - Tinf proportional to the oxygen fraction, rho_g = p / (287 Tv)."""
    eps, sigma, tv, tinf, cg = 1.0, 5.670e-8, 673.0, 300.0, 1070.0
    tf = tinf + (2199.0 - tinf) * (x_o2 / 0.21)
    rho = 101325.0 * (p_atm or 1.0) / (287.0 * tv)
    if tf <= tv + 50:
        return None
    return 100.0 * eps * sigma * (tv ** 4 - tinf ** 4) / (rho * cg * (tf - tv))


def fit_constant_quantiles(outs):
    grid = [i * 0.02 for i in range(0, 1001)]  # 0 .. 20 cm/s
    qs = []
    for t in QUANTILES:
        best = min(grid, key=lambda q: sum(interval_pinball(q, t, o["quench_u_lo_cm_s"], o["quench_u_hi_cm_s"]) for o in outs))
        qs.append(round(best, 2))
    return qs


def lognormal_quantiles(median, s):
    z = {0.05: -1.6449, 0.25: -0.6745, 0.5: 0.0, 0.75: 0.6745, 0.95: 1.6449}
    return [median * math.exp(s * z[t]) for t in QUANTILES]


def cmd_baselines(_):
    train = load_train()
    outs = [c["outcome"] for c in train]
    qouts = [o for o in outs if quench_scored(o)]
    res = {}
    # quench, constant baseline: the five quantiles fitted on train (interval pinball)
    q_const = fit_constant_quantiles(qouts)
    res["quench_constant"] = {"quantiles_cm_s": q_const,
                              **score({c["case_id"]: q_const for c in train}, outs, "quench")}
    # quench, classical correlation at its published constants; its band width is the one fitted constant
    med = {c["case_id"]: v_crit_bhattacharjee(c["x_o2_initial"], c["pressure_atm"])
           for c in train if c["x_o2_initial"] is not None}
    med = {k: v for k, v in med.items() if v is not None}
    best = None
    for s in [0.05 * i for i in range(1, 41)]:
        sc = score({k: lognormal_quantiles(v, s) for k, v in med.items()}, qouts, "quench")
        if best is None or sc["day_mean"] < best[1]["day_mean"]:
            best = (s, sc)
    res["quench_bhattacharjee_2016"] = {"log_width_fitted": round(best[0], 2), **best[1],
                                        "median_at_x_o2": {x: round(v_crit_bhattacharjee(x), 3) for x in (0.15, 0.17, 0.19, 0.21, 0.222)}}
    # sustain, incidence baseline
    sus = [o for o in outs if o["sustain"] is not None]
    p = sum(o["sustain"] for o in sus) / len(sus)
    res["sustain_incidence"] = {"p": round(p, 4), **score({o["case_id"]: p for o in sus}, outs, "sustain")}
    # sustain, the measurer-style constant 'always lit' (the carry of D1 in section 12.9 says every sample lights)
    res["sustain_always"] = score({o["case_id"]: 1.0 for o in sus}, outs, "sustain")
    for k, v in res.items():
        v["per_day"] = {d: round(x, 4) for d, x in v["per_day"].items()}
        for key in ("case_mean", "day_mean"):
            v[key] = round(v[key], 4) if v[key] is not None else None
    (OUT / "baselines_train.json").write_text(json.dumps(res, indent=1) + "\n", encoding="utf-8", newline="\n")
    print(json.dumps(res, indent=1))
    return 0


def cmd_show_train(_):
    fc, cases = load_all()
    for rec, inp, o in cases:
        if rec["partition"] != "train":
            continue
        print("#####", rec["case_id"], "|", inp["material_class"], inp["configuration"], "| O2", inp["x_o2_initial"],
              "| r", inp["flow_restrictors"], "| ign", inp["ignition_setting"], inp["u_cm_s_at_ignition"])
        print("   fan:", rec["fan"], "| air:", rec["air"])
        print("   obs:", rec["obs"][:400])
        print("   ->", o["sustain"], o["quench_class"], o["quench_u_lo_cm_s"], o["quench_u_hi_cm_s"],
              "n2" if o["n2_applied"] else "", "|", o["quench_rule"])
    return 0


def cmd_score(args):
    """Score a forecast file ({case_id: [q05, q25, q50, q75, q95]} or {case_id: p}) on train or validation.
    For validation only the aggregate and the per-day means are printed, never a case."""
    fc = json.loads(Path(args.forecasts).read_text(encoding="utf-8"))
    if args.split == "train":
        outs = [c["outcome"] for c in load_train()]
    else:
        outs = [json.loads(line) for line in (OUT / "cases_validation_outcomes.jsonl").open(encoding="utf-8")]
    res = score(fc, outs, args.target)
    print(json.dumps({k: (round(v, 4) if isinstance(v, float) else v) for k, v in res.items() if k != "per_day"}))
    print(json.dumps({d: round(x, 4) for d, x in res["per_day"].items()}))
    return 0


def cmd_selftest(_):
    assert abs(crps_interval([1, 1, 1, 1, 1], 0, 2)) < 1e-12
    assert abs(crps_interval([3, 3, 3, 3, 3], 0, 2) - 2 * sum((1 - t) for t in QUANTILES) / 5) < 1e-12
    # a point outcome reduces to the usual quantile score
    assert abs(crps_interval([0, 1, 2, 3, 4], 2, 2) - 2 * (0.05 * 2 + 0.25 * 1 + 0 + 0.25 * 1 + 0.05 * 2) / 5) < 1e-12
    v21 = v_crit_bhattacharjee(0.21)
    assert 1.0 < v21 < 1.6, v21
    assert v_crit_bhattacharjee(0.17) > v21
    print("selftest ok; V_cr(21 %) =", round(v21, 3), "cm/s")
    return 0


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("cmd", choices=["build", "baselines", "show-train", "score", "selftest"])
    ap.add_argument("--target", choices=["quench", "sustain"], default="quench")
    ap.add_argument("--forecasts", help="JSON file of forecasts keyed by case_id (score)")
    ap.add_argument("--split", choices=["train", "validation"], default="train")
    args = ap.parse_args()
    return {"build": cmd_build, "baselines": cmd_baselines, "show-train": cmd_show_train, "score": cmd_score,
            "selftest": cmd_selftest}[args.cmd](args)


if __name__ == "__main__":
    sys.exit(main())
