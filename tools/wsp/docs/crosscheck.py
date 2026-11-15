#!/usr/bin/env python3
"""Document lane: cross-check NASA's condition tables against NASA's own sensor and settings files.

Conditions only. For the 19 PSI flight combustion investigations this script compares every condition
that a NASA table gives (flows, composition, pressure and chamber atmosphere, test ids and dates) with
NASA's own sensor or settings text files for the same test, checks each table against itself, and
decides the cause of each difference with five ordered checks. It writes one record per discrepancy
to `runs/dev/data_docs/discrepancies/discrepancies.jsonl` (schema `ignis.docs.discrepancy/1`) and
prints aggregates only. No language model reads data; nothing is fitted except the documented
straight-line reading of one uncalibrated pressure transducer, which is declared as our method.

    python tools/wsp/docs/crosscheck.py

Boundaries (absolute):
  * The table side is `runs/dev/data_all/work/atlas/tests.jsonl` (conditions, no outcome). Raw tables
    are not opened, so no outcome column is ever parsed.
  * NASA files are opened only through `NasaFiles.open`, which refuses the protected paths, checks the
    SHA-256 of the upload log and, for the project families (PSI-25, PSI-69, PSI-159), only serves
    files of tests that `tests.jsonl` lists (train and validation; the test partition is never read).
  * Only flow, setpoint, readback, pressure, temperature-before-ignition and composition channels are
    parsed, and only up to ignition where the file marks it. Photomultiplier, radiometer, camera and
    flame-measurement channels are never parsed. Script files are read for flow settings and the test
    name only; timeline files for the ignition stamp and the test name only.
  * Output: aggregates on the console; the discrepancy records, which hold conditions only.
"""
import csv
import datetime as dt
import hashlib
import json
import math
import re
import statistics as st
import sys
from collections import Counter, defaultdict
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
ATLAS = REPO / "runs/dev/data_all/work/atlas/tests.jsonl"
UPLOAD = Path(r"D:\Codex\Worktrees\ignis-science-transfer\SpaceApps-Ardealu\runs\dev\data_astra\logs"
              r"\upload_verified.jsonl")
OUT = REPO / "runs/dev/data_docs/discrepancies/discrepancies.jsonl"
SUMMARY = REPO / "runs/dev/data_docs/discrepancies/summary.json"
SCHEMA = "ignis.docs.discrepancy/1"

PROTECTED = [
    "holdout/", "runs/dev/holdout_ext/", "runs/dev/bass_front/val_sealed/",
    "runs/dev/data_all/work/atlas/tests_outcomes.jsonl", "runs/dev/data_all/work/flame_design/",
    "runs/dev/data_all/work/transfer_acme/", "runs/dev/data_astra/work/labels/",
    "runs/dev/data_astra/work/discrepancies/", "runs/dev/data_docs/cards/", "runs/dev/data_docs/final/",
    "runs/dev/data_docs/protect/", "runs/dev/data_docs/gold/", "runs/dev/data_docs/visual/",
    "runs/dev/data_ext/",
]
PROJECT = {"PSI-25", "PSI-69", "PSI-159"}

# --- physical constants and documented conversions ----------------------------------------------
# ACME mass flow controllers: Hastings HFC-D-302, calibrated for nitrogen, standard 0 C and 760 torr
# (ACME data interpretation documents of E-FIELD Flames, p. 8, and s-Flame, section 4.A.1). The
# readback is 0-5 V over the full scale in the file header. Component gas conversion factors (GCF)
# and molecular weights: the chamber calculator workbooks (`SourceTable` row 3, `Property Table`);
# the same factors are in `ACMEFlowCalculator.xlsx` (PSI-22), whose mixture rule is 1/sum(x_i/GCF_i)
# and whose planning temperature is 22 C (factor 273.15/295.15).
GCF = {"o2": 0.9779, "he": 1.4005, "co2": 0.7526179096810801, "n2": 1.0, "h2o": 0.7992,
       "ethylene": 0.6062, "methane": 0.7786766523371256, "propane": 0.3499, "butane": 0.2622,
       "ethane": 0.4998, "h2": 1.0037583476195506}
MW = {"o2": 31.9988, "he": 4.002602, "co2": 44.0095, "n2": 28.0134, "h2o": 18.0153, "ethylene": 28.0532,
      "methane": 16.0425, "propane": 44.0956, "butane": 58.122, "ethane": 30.069, "h2": 2.01588}
VM_0C = 22.41397  # L/mol, ideal gas at 0 C and 101.325 kPa
PLAN_T_FACTOR = 273.15 / (273.15 + 22.0)  # ACMEFlowCalculator.xlsx, Tables!B31:B32
PSI_PA = 6894.757293168
ATM_PA = 101325.0
MMHG_PA = 133.322387415
CALC_SPECIES = ["o2", "he", "co2", "n2", "h2o", "ethylene", "methane", "propane", "butane", "ethane", "h2"]

# MFC gas codes of the data-file header, from `ACMEFlowCalculator.xlsx` sheet HeaderTables/Tables
# (PSI-22). Codes 19 and above are not in that table; they are read from the chamber calculator's
# bottle (Source-ID) of the same test point.
CODES_MFCA1 = {0: {"o2": .21, "n2": .79}, 1: {"o2": .30, "n2": .70}, 2: {"o2": .40, "n2": .60},
               3: {"o2": .85, "n2": .15}, 4: {"o2": .30, "he": .70}}
CODES_MFCA2 = {0: {"n2": 1.0}, 1: {"he": 1.0}}
CODES_MFCA3 = {0: {"methane": 1.0}, 1: {"methane": .3, "n2": .7}, 2: {"methane": .4, "n2": .6},
               3: {"methane": .45, "n2": .55}, 4: {"ethylene": 1.0}, 5: {"ethylene": .3, "n2": .7},
               6: {"ethylene": .5, "n2": .5}, 7: {"h2": 1.0}, 8: {"h2": .444, "methane": .556},
               9: {"h2": .556, "methane": .444}, 10: {"h2": .667, "methane": .333},
               11: {"h2": .2, "methane": .25, "he": .55}, 12: {"h2": .25, "methane": .2, "he": .55},
               13: {"h2": .3, "methane": .15, "he": .55}, 14: {"h2": .45, "he": .55},
               15: {"methane": .45, "he": .55}, 16: {"ethylene": .2, "he": .8},
               17: {"ethylene": .25, "he": .75}, 18: {"ethylene": .3, "he": .7}}
BURNER_CODES = {0: "spherical_1/4in", 1: "spherical_3/8in", 2: "spherical_1/2in", 3: "coflow",
                4: "gas_jet_1.3mm", 5: "gas_jet_2.1mm", 6: "bre_25mm", 7: "bre_50mm"}
EXPERIMENT_CODES = {0: "Flame Design", 1: "Coflow", 2: "S-Flame", 3: "E-Field", 4: "BRE"}

FUEL_WORDS = {"ch4": "methane", "methane": "methane", "c2h4": "ethylene", "ethylene": "ethylene",
              "c3h8": "propane", "propane": "propane", "c4h10": "butane", "butane": "butane",
              "n-butane": "butane", "c2h6": "ethane", "ethane": "ethane", "h2": "h2", "hydrogen": "h2",
              "he": "he", "helium": "he", "n2": "n2", "nitrogen": "n2", "o2": "o2", "oxygen": "o2"}

NAN = float("nan")


def isnum(x):
    return isinstance(x, (int, float)) and not isinstance(x, bool) and x == x and math.isfinite(x)


def r6(x):
    return None if not isnum(x) else float(f"{x:.6g}")


def med(v):
    v = [x for x in v if isnum(x)]
    return st.median(v) if v else NAN


def robust_sigma(v):
    v = [x for x in v if isnum(x)]
    if len(v) < 3:
        return NAN
    m = st.median(v)
    return 1.4826 * st.median([abs(x - m) for x in v])


def pct(v, q):
    v = sorted(x for x in v if isnum(x))
    if not v:
        return NAN
    k = (len(v) - 1) * q
    lo, hi = math.floor(k), math.ceil(k)
    return v[lo] + (v[hi] - v[lo]) * (k - lo)


# =================================================================================================
# NASA files: guarded, hash-checked access
# =================================================================================================
class Refused(Exception):
    pass


class NasaFiles:
    def __init__(self):
        self.by_inv = defaultdict(dict)
        for line in UPLOAD.open(encoding="utf-8"):
            r = json.loads(line)
            self.by_inv[r["investigation"]][r["file_name"]] = r
        self.allowed_tests = {}     # project investigation -> set of test tokens a file name may carry
        self.investigation_wide = {"PSI-159": set(CALCULATORS["PSI-159"])}
        self.opened = Counter()
        self.refused = Counter()
        self.hash_failures = []
        self._verified = {}

    def has(self, inv, name):
        return name in self.by_inv.get(inv, {})

    def names(self, inv):
        return self.by_inv.get(inv, {}).keys()

    def _guard(self, inv, name, path):
        p = str(path).replace("\\", "/")
        for bad in PROTECTED:
            if bad in p:
                raise Refused(f"protected path: {bad}")
        if inv in PROJECT and name in self.investigation_wide.get(inv, set()):
            return   # one file for the whole campaign; its reader keeps only listed tests' rows
        if inv in PROJECT:
            ok = self.allowed_tests.get(inv, set())
            toks = set(re.findall(r"[A-Za-z0-9]+(?:_\d+)?", name)) | set(re.split(r"[ _.\-]", name))
            toks |= set(re.findall(r"FLEX-\d+", name))
            toks |= set(re.findall(r"\d{5}[A-Z]{1,2}\d+(?:_\d+)?", name))
            toks |= set(re.findall(r"\d{5}[A-Z]{1,2}\d+(?=_[A-Za-z]|\.)", name))
            if not (toks & ok):
                raise Refused("project investigation: file of a test not listed in tests.jsonl")

    def path(self, inv, name):
        r = self.by_inv[inv][name]
        path = Path(r["source_path"])
        try:
            self._guard(inv, name, path)
        except Refused as e:
            self.refused[f"{inv}: {e}"] += 1
            raise
        if path not in self._verified:
            h = hashlib.sha256()
            with path.open("rb") as f:
                for chunk in iter(lambda: f.read(1 << 22), b""):
                    h.update(chunk)
            ok = h.hexdigest() == r["sha256"]
            self._verified[path] = ok
            if not ok:
                self.hash_failures.append(name)
        if not self._verified[path]:
            raise Refused("sha256 differs from the upload log")
        self.opened[inv] += 1
        return path

    def sha(self, inv, name):
        return self.by_inv[inv][name]["sha256"]

    def text(self, inv, name):
        with self.path(inv, name).open("rb") as f:
            return f.read().decode("utf-8", "replace").replace("\r\n", "\n").replace("\r", "\n")


# =================================================================================================
# Atlas (table side)
# =================================================================================================
def load_atlas():
    rows = defaultdict(list)
    with ATLAS.open(encoding="utf-8") as f:
        for line in f:
            r = json.loads(line)
            rows[r["investigation_id"]].append(r)
    return rows


def val(obj, path):
    v = obj
    for part in path.split("."):
        if isinstance(v, dict):
            v = v.get(part)
        else:
            return None
    return v


def resolved(v):
    return isinstance(v, dict) and v.get("status") == "resolved" and v.get("value_si") is not None


def table_side(v, field_path):
    s = (v or {}).get("source") or {}
    return {"field_path": field_path, "value_si": v.get("value_si") if v else None,
            "unit_si": v.get("unit_si") if v else None, "raw_value": v.get("raw_value") if v else None,
            "raw_unit": v.get("raw_unit") if v else None, "status": v.get("status") if v else None,
            "file": s.get("file"), "column": s.get("column"), "row": s.get("row"), "sheet": s.get("sheet"),
            "field": s.get("field"), "basis": v.get("basis") if v else None}


def alias(row, scheme):
    for a in row.get("aliases") or []:
        if a.get("scheme") == scheme:
            return a.get("name")
    return None


def set_allowed(nasa, atlas):
    """The project investigations' files may be opened only for tests that tests.jsonl lists."""
    for inv in PROJECT:
        ok = set()
        for row in atlas.get(inv, []):
            tid = row["test_id"].split(":")[-1]
            ok.add(tid.replace(" ", ""))
            for a in row.get("aliases") or []:
                n = str(a.get("name") or "").replace(" ", "")
                if a.get("scheme") in ("table_test", "file_flex_number", "testsheet_header", "cir_test_point",
                                       "listing", "file_test", "proof_sheet_test", "test"):
                    ok.add(n)
        ok.discard("")
        nasa.allowed_tests[inv] = ok


# =================================================================================================
# Records
# =================================================================================================
RECORDS = []
FIXED_READINGS = []      # our_reading_wrong: the reading was fixed, the record went
COMPARED = Counter()     # (investigation, field) -> comparisons made
AGREE = Counter()
LIMITS = []              # what could not be compared and why


def record(inv, row, field, table, sensor, diff, status, checks, verdict, decided_by, note="",
           others=None, test_id=None, group_id=None):
    rid = hashlib.sha256(json.dumps([inv, test_id or (row or {}).get("test_id"), field,
                                     table.get("column"), sensor.get("file"), sensor.get("channel")],
                                    sort_keys=True, default=str).encode()).hexdigest()[:16]
    rec = {
        "schema": SCHEMA, "id": rid, "investigation": inv,
        "acronym": (row or {}).get("acronym"),
        "test_id": test_id or (row or {}).get("test_id"), "atlas_id": (row or {}).get("atlas_id"),
        "group_id": group_id or (row or {}).get("group_id"),
        "partition": (row or {}).get("project_partition"), "field": field,
        "table": table, "sensor": sensor, "other_sources": others or [],
        "difference": diff, "status": status, "cause_checks": checks,
        "decided_by_check": decided_by, "verdict": verdict, "note": note,
        "nasa_value_replaced": False,
    }
    RECORDS.append(rec)
    return rec


def chk(n, name, result, decisive=False):
    return {"check": n, "name": name, "result": result, "decisive": decisive}


CHECK_NAMES = {1: "ours: same method on agreeing tests", 2: "third NASA source",
               3: "physics that cannot be otherwise", 4: "shape of the difference",
               5: "same quantity, two definitions"}


def shape_of(t, s, prev_row_value=None, planned=None, rel=1e-4):
    """Check 4: name the shape of a ratio table/sensor."""
    if not (isnum(t) and isnum(s)) or s == 0 or t == 0:
        return None
    r = t / s
    for k, name in ((10.0, "factor 10"), (0.1, "factor 1/10"), (2.0, "factor 2"), (0.5, "factor 1/2"),
                    (100.0, "factor 100"), (0.01, "factor 1/100")):
        if abs(r / k - 1) < 0.06:
            return name
    if isnum(prev_row_value) and abs(t - prev_row_value) <= rel * abs(t):
        return "equal to the row above"
    if isnum(planned) and abs(t - planned) <= 0.02 * abs(t):
        return "equal to the planned value"
    return None


# =================================================================================================
# ACME readers (PSI-10, 20, 21, 22, 23, 159)
# =================================================================================================
def clock(s):
    h, m, sec, ms = s.strip().split(":")
    return int(h) * 3600 + int(m) * 60 + int(sec) + int(ms) / 1000.0


def acme_header(line):
    parts = [p.strip() for p in line.split(",")]
    h = {}
    m = re.match(r"(\d{4}-\d\d-\d\d) (\d{3}) (\S+)", parts[0])
    if m:
        h.update(date=m.group(1), doy=int(m.group(2)), clock=clock(m.group(3)))
    for p in parts[1:]:
        if ":" in p:
            k, v = p.split(":", 1)
            h[k.strip()] = v.strip()
    return h


def unwrap_to(t, ref):
    return t + 86400 if t < ref - 40000 else t


TP_KEYS = {"Ignition and retraction", "MFCs on for priming"}


def acme_tp(text):
    """Only the ignition stamp, the priming stamp and the test name; nothing else is parsed."""
    stamps, name = {}, None
    for line in text.split("\n"):
        m = re.match(r"\s*([A-Za-z][^:]*?):\s*(\d{4}-\d\d-\d\d) (\d\d):(\d\d):(\d\d)", line)
        if m and m.group(1).strip() in TP_KEYS:
            stamps[m.group(1).strip()] = int(m.group(3)) * 3600 + int(m.group(4)) * 60 + int(m.group(5))
            continue
        m = re.match(r"\s*Test Point:\s*(\S+)", line)
        if m and name is None:
            name = m.group(1)
    return stamps, name


def acme_script(text):
    """Flow settings and the test name only (no stage durations, no post-extinction settings)."""
    out = {}
    for line in text.split("\n"):
        key, sep, value = line.strip().rstrip(",").partition(":")
        key = key.strip()
        if not sep:
            continue
        if key == "testpointName" or (re.search(r"Slpm$|MPerSec$", key) and "xtinction" not in key) or \
                re.fullmatch(r"constantFlowTime\d?Sec", key):
            out[key] = value.strip()
    return out


def acme_setpoints(text, t_end):
    lines = text.split("\n")
    hdr = acme_header(lines[0]) if lines else {}
    ref = hdr.get("clock", 0.0)
    rows = []
    for line in lines[2:]:
        sp = line.strip().split(",")
        try:
            t = unwrap_to(clock(sp[0]), ref)
            v = [float(x) for x in sp[1:4]]
        except (ValueError, IndexError):
            continue
        if t > t_end:
            break
        rows.append((t, v))
    return hdr, rows


DATA_COLS = ["MFCA1StaticMixerFlow", "MFCA2BypassFlow", "MFCA3FuelFlow", "PTA2", "PTA3"]
ZERO_READ_V = 0.005        # a readback this close to its zero offset reads no flow (zero noise is ~0.0002 V)
ZERO_READ_HOLD_S = 3.0     # ... and means no flow only after the command has been held this long
# Tolerance of a readback confirming a command: 1 % of full scale (0.05 V) plus 5 % of the command. Our
# assumption for a Hastings controller; how often agreeing tests meet it is reported (check 1).
RB_TOL_V, RB_TOL_REL = 0.05, 0.05


def acme_data(path, t_end):
    """Flow readbacks and chamber pressure voltages up to t_end. Other columns are not parsed."""
    with open(path, encoding="utf-8", errors="replace") as f:
        hdr = acme_header(f.readline())
        cols = [c.strip() for c in f.readline().split(",")]
        idx = {c: cols.index(c) for c in DATA_COLS if c in cols}
        ref = hdr.get("clock", 0.0)
        t, vals = [], {c: [] for c in idx}
        for line in f:
            sp = line.split(",", max(idx.values()) + 1 if idx else 1)
            try:
                tt = unwrap_to(clock(sp[0]), ref)
            except (ValueError, IndexError):
                continue
            if tt > t_end:
                break
            try:
                row = {c: float(sp[i]) for c, i in idx.items()}
            except (ValueError, IndexError):
                continue
            t.append(tt)
            for c in idx:
                vals[c].append(row[c])
    return hdr, t, vals


def acme_file_names(inv, test, nasa):
    pre = "" if inv == "PSI-21" else f"{inv}_Raw Data_"
    alias_tp = test[:5] + "J" + test[5:]

    def first(*c):
        for n in c:
            if nasa.has(inv, n):
                return n
        return None
    return dict(setpoints=first(f"{pre}{test}_mfcSetpoints.csv"), data=first(f"{pre}{test}_data.csv"),
                tp=first(f"{pre}{test}_TP.txt", f"{pre}{alias_tp}_TP.txt"),
                script=first(f"{pre}as run{test}.txt", f"{pre}{test}.txt"))


def mix_gcf(comp):
    s = sum(x / GCF[k] for k, x in comp.items() if x > 0)
    return 1.0 / s if s > 0 else NAN


def molar(slpm_n2eq, comp):
    """mol/s of a bottle stream from its nitrogen-equivalent slpm reading (0 C, 760 torr)."""
    if not (isnum(slpm_n2eq) and comp):
        return NAN
    return slpm_n2eq * mix_gcf(comp) / VM_0C / 60.0


def stream(fuel_slpm, fuel_comp, n2_slpm, n2_comp, ox_slpm, ox_comp):
    """Species molar and mass flows of the burner stream: MFCA3 (fuel bottle), MFCA2 (diluent),
    MFCA1 (oxidiser bottle). Mass flows in mg/s on the 0 C standard of the controllers."""
    n = defaultdict(float)
    for slpm, comp in ((fuel_slpm, fuel_comp), (n2_slpm, n2_comp), (ox_slpm, ox_comp)):
        if isnum(slpm) and slpm > 0 and comp:
            tot = molar(slpm, comp)
            for k, x in comp.items():
                n[k] += x * tot
    total = sum(n.values())
    mass = {k: v * MW[k] * 1000.0 for k, v in n.items()}
    fuels = [k for k in n if k in ("methane", "ethylene", "propane", "butane", "ethane", "h2")]
    return {
        "n": dict(n), "mass_mg_s": mass, "total_mg_s": sum(mass.values()),
        "fuel_mg_s": sum(mass[k] for k in fuels), "o2_mg_s": mass.get("o2", 0.0),
        "inert_mg_s": sum(mass[k] for k in mass if k not in fuels and k != "o2"),
        "n2_mg_s": mass.get("n2", 0.0),
        "x_fuel": (sum(n[k] for k in fuels) / total) if total > 0 else NAN,
        "x_o2": (n.get("o2", 0.0) / total) if total > 0 else NAN,
    }


class AcmeTest:
    """Everything this script reads for one ACME test point, all before ignition."""
    __slots__ = ("inv", "test", "files", "hdr", "script", "tp_name", "t_ign", "t_end", "cmd", "cmd_t",
                 "cmd_hold", "rb", "rb_settled", "rb_offset", "pta3", "pta2", "status", "shas", "first_cmd",
                 "rb_state", "ign_ref")

    def __init__(self, inv, test):
        self.inv, self.test = inv, test
        self.status, self.shas = "ok", {}
        self.hdr, self.script, self.tp_name = {}, {}, None
        self.cmd = [NAN] * 3
        self.cmd_t = [NAN] * 3
        self.cmd_hold = [NAN] * 3
        self.rb = [NAN] * 3
        self.rb_settled = [False] * 3
        self.rb_state = [None] * 3
        self.rb_offset = [0.0] * 3
        self.pta2 = self.pta3 = NAN
        self.t_ign = self.t_end = NAN
        self.first_cmd = [NAN] * 3
        self.ign_ref = None
        self.files = {}


def first_reactant_command(sp_text, light_k):
    """Without a timeline file: the time of the first command of the stream that lights the flame.
    No flame exists before its reactant flows, so nothing read up to this command follows ignition."""
    lines = sp_text.split("\n")
    ref = acme_header(lines[0]).get("clock", 0.0) if lines else 0.0
    for line in lines[2:]:
        sp = line.strip().split(",")
        try:
            t = unwrap_to(clock(sp[0]), ref)
            v = [float(x) for x in sp[1:4]]
        except (ValueError, IndexError):
            continue
        if v[light_k] > 0:
            return t
    return NAN


def read_acme(inv, test, nasa, light_k=2):
    a = AcmeTest(inv, test)
    fn = acme_file_names(inv, test, nasa)
    a.files = fn
    if not fn["setpoints"]:
        a.status = "no_setpoints"
        return a
    try:
        sp_text = nasa.text(inv, fn["setpoints"])
        a.shas["setpoints"] = nasa.sha(inv, fn["setpoints"])
        stamps = {}
        if fn["tp"]:
            stamps, a.tp_name = acme_tp(nasa.text(inv, fn["tp"]))
            a.shas["tp"] = nasa.sha(inv, fn["tp"])
    except Refused as e:
        a.status = f"refused:{e}"
        return a
    hdr0 = acme_header(sp_text.split("\n", 1)[0])
    a.hdr = hdr0
    ref = hdr0.get("clock", 0.0)
    if "Ignition and retraction" in stamps:
        a.t_ign = unwrap_to(stamps["Ignition and retraction"], ref)
        a.t_end = a.t_ign + 0.999           # the stamp has one-second resolution
        a.ign_ref = "timeline stamp 'Ignition and retraction'"
    else:
        a.t_ign = a.t_end = first_reactant_command(sp_text, light_k)
        a.ign_ref = "first command of the lighting stream (no ignition stamp)"
        if not isnum(a.t_end):
            a.status = "no_ignition_reference"
            return a
    _, rows = acme_setpoints(sp_text, a.t_end)
    if not rows:
        a.status = "no_command_before_ignition"
        return a
    last = rows[-1][1]
    for k in range(3):
        a.cmd[k] = last[k]
        # the time this level was set, and how long it was held before the window end
        tset = rows[-1][0]
        for t, v in reversed(rows):
            if abs(v[k] - last[k]) > 1e-9:
                break
            tset = t
        a.cmd_t[k] = tset
        a.cmd_hold[k] = a.t_end - tset
        nz = [t for t, v in rows if v[k] > 0]
        a.first_cmd[k] = nz[0] if nz else NAN
    if fn["script"]:
        try:
            a.script = acme_script(nasa.text(inv, fn["script"]))
            a.shas["script"] = nasa.sha(inv, fn["script"])
        except Refused:
            pass
    if fn["data"]:
        try:
            path = nasa.path(inv, fn["data"])
            a.shas["data"] = nasa.sha(inv, fn["data"])
            hdr, t, vals = acme_data(path, a.t_end)
            a.hdr = hdr or hdr0
            chans = DATA_COLS[:3]
            for k, c in enumerate(chans):
                if c not in vals or not t:
                    continue
                y = vals[c]
                # zero offset: samples held at a zero command for at least 1 s, before the flow began
                t_zero_end = (a.first_cmd[k] - 0.3) if isnum(a.first_cmd[k]) else a.t_end
                z = [yy for tt, yy in zip(t, y) if tt < t_zero_end and tt > t_zero_end - 10]
                a.rb_offset[k] = st.median(z) if len(z) > 50 else 0.0
                if a.cmd[k] > 0 and a.cmd_hold[k] >= 0.8:
                    lo = max(a.cmd_t[k] + 0.6, a.t_end - 1.0)
                    w = [(tt, yy) for tt, yy in zip(t, y) if lo <= tt <= a.t_end]
                    if len(w) >= 40:
                        half = len(w) // 2
                        m1 = st.median([yy for _, yy in w[:half]])
                        m2 = st.median([yy for _, yy in w[half:]])
                        a.rb[k] = m2 - a.rb_offset[k]
                        flat = abs(m2 - m1) <= 0.02 * abs(m2) + 0.003
                        if flat and abs(a.rb[k]) <= ZERO_READ_V:
                            # a flat zero: the controller has not answered yet, or does not flow
                            a.rb_state[k] = "zero_read" if a.cmd_hold[k] >= ZERO_READ_HOLD_S else None
                        elif flat and a.rb[k] >= 0.1 * a.cmd[k]:
                            a.rb_state[k] = "settled"
                        a.rb_settled[k] = a.rb_state[k] == "settled"
                elif a.cmd[k] == 0:
                    w = [yy for tt, yy in zip(t, y) if a.t_end - 1.0 <= tt <= a.t_end]
                    if len(w) >= 40:
                        a.rb[k] = st.median(w) - a.rb_offset[k]
                        a.rb_settled[k] = True
                        a.rb_state[k] = "zero_command"
            for c, attr in (("PTA3", "pta3"), ("PTA2", "pta2")):
                if c in vals and t:
                    w = [yy for tt, yy in zip(t, vals[c]) if a.t_ign - 3.0 <= tt <= a.t_ign]
                    if len(w) >= 40:
                        setattr(a, attr, st.median(w))
        except Refused:
            pass
    return a


def fs(a, k):
    try:
        return float(a.hdr.get(f"MFCA{k + 1} Full Scale", "nan"))
    except ValueError:
        return NAN


def code(a, k):
    try:
        return int(a.hdr.get(f"MFCA{k + 1} Gas", ""))
    except ValueError:
        return None


def v2slpm(a, k, v):
    return v / 5.0 * fs(a, k) if isnum(v) else NAN


RELEVANT = {"normal": (1, 2), "inverse": (0, 1)}   # MFCA indices that carry a stream


def script_flows(a, orient):
    """Flows of the script in nitrogen-equivalent slpm for MFCA1/2/3: the ignition flow and each
    constant-flow stage with its planned duration. The main stage is the longest one."""
    s = a.script
    if not s:
        return None

    def g(*keys):
        for k in keys:
            if k in s:
                try:
                    return float(s[k])
                except ValueError:
                    return NAN
        return NAN
    rel = RELEVANT[orient]
    ign = [g("oxidizerIgnitionFlowSlpm"), g("nitrogenIgnitionFlowSlpm", "nitrogenIgnitionFlowRateN2Slpm"),
           g("fuelIgnitionFlowSlpm", "fuelBottleIgnitionFlowRateN2Slpm")]
    stages = []
    for i in (1, 2, 3):
        fl = [g(f"oxidizerTestFlow{i}Slpm"), g(f"nitrogenTestFlow{i}Slpm"), g(f"fuelTestFlow{i}Slpm")]
        if i == 1:
            fl = [fl[0], g("nitrogenTestFlow1Slpm", "nitrogenTestFlowRateN2Slpm"),
                  g("fuelTestFlow1Slpm", "fuelBottleTestFlowRateN2Slpm")]
        dur = g(f"constantFlowTime{i}Sec") if i > 1 else g("constantFlowTime1Sec", "constantFlowTimeSec")
        stages.append({"stage": i, "flows": fl, "duration_s": dur})
    for k in range(3):
        if k not in rel:          # a stream that this orientation does not use carries no flow
            ign[k] = 0.0 if not isnum(ign[k]) else ign[k]
            for sg in stages:
                sg["flows"][k] = 0.0 if not isnum(sg["flows"][k]) else sg["flows"][k]
    for sg in stages[1:]:        # a later stage the script does not name, or names with no time, did not run
        if not isnum(sg["duration_s"]) or sg["duration_s"] <= 0:
            sg["flows"] = [NAN] * 3
    for k in rel:                # a later stage that names no flow for a stream keeps it at zero
        for sg in stages[1:]:
            if isnum(sg["duration_s"]) and sg["duration_s"] > 0 and not isnum(sg["flows"][k]):
                sg["flows"][k] = 0.0
    runs = [sg for sg in stages if all(isnum(x) for x in sg["flows"])]
    main = None
    if runs:
        timed = [sg for sg in runs if isnum(sg["duration_s"])]
        main = max(timed, key=lambda sg: sg["duration_s"]) if timed else runs[0]
    multi = len([sg for sg in runs if (sg["duration_s"] or 0) > 0]) > 1
    return {"ign": ign, "test": main["flows"] if main else [NAN] * 3, "main_stage": main["stage"] if main else None,
            "stages": runs, "multi_stage": multi}


# =================================================================================================
# Chamber calculator (EventData): pressure, bottle and start mixture before each test point
# =================================================================================================
CALCULATORS = {
    "PSI-10": ["PSI-10_Raw Data_Final ACME Chamber Calculator - 2019-04-09 to 2019-08-13.xlsx",
               "PSI-10_Raw Data_Final ACME Chamber Calculator - 2021-10-28 to 2022-02-04.xlsx"],
    "PSI-20": ["PSI-20_Raw Data_Final ACME Chamber Calculator - 2019-01-31 to 2019-04-09.xlsx",
               "PSI-20_Raw Data_Final ACME Chamber Calculator - 2020-07-22 to 2021-02-03.xlsx"],
    "PSI-21": ["Final ACME Chamber Calculator - 2018-05-22 to 2018-09-19.xlsx",
               "Final ACME Chamber Calculator - 2021-02-03 to 2021-03-09.xlsx"],
    "PSI-159": ["PSI-159_Raw Data_Final ACME Chamber Calculator - 2021-03-09 to 2021-10-21.xlsx"],
}
# EventData columns read (0-based). Nothing after the test point is read: not the Ignite decision,
# the litres delivered, the end pressure, the burn fraction or the finish mixture.
EV = dict(op=0, tpid=2, tstart=12, pstart=18, sid=21, type=22, fueltype=23, start_mix=range(39, 50))


def load_calculators(nasa):
    import openpyxl
    out = defaultdict(list)    # (inv, tpid) -> list of event dicts
    sources = {}
    for inv, names in CALCULATORS.items():
        for name in names:
            if not nasa.has(inv, name):
                LIMITS.append(f"{inv}: chamber calculator {name} not on disk")
                continue
            path = nasa.path(inv, name)
            wb = openpyxl.load_workbook(path, data_only=True, read_only=True)
            ws = wb["SourceTable"]
            for row in ws.iter_rows(min_row=4, values_only=True):
                if row and row[2] and isinstance(row[2], str) and any(isnum(x) for x in row[3:14]):
                    comp = {sp: float(x) for sp, x in zip(CALC_SPECIES, row[3:14]) if isnum(x) and x > 0}
                    sources[str(row[2]).strip()] = comp
            ws = wb["EventData"]
            for i, row in enumerate(ws.iter_rows(min_row=5, values_only=True), start=5):
                if not row or row[EV["tpid"]] in (None, ""):
                    continue
                op = str(row[EV["op"]] or "").strip()
                if op not in ("TP", "INV_TP"):
                    continue
                tpid = str(row[EV["tpid"]]).strip().replace(" ", "")
                if inv in PROJECT and tpid not in nasa.allowed_tests.get(inv, set()):
                    continue      # a test point the atlas does not list (test partition): not read
                mix = [row[j] if j < len(row) else None for j in EV["start_mix"]]
                tot = sum(x for x in mix if isnum(x))
                frac = {sp: (x / tot) for sp, x in zip(CALC_SPECIES, mix) if isnum(x) and tot > 0}
                out[(inv, tpid)].append({
                    "file": name, "row": i, "op": op, "tstart": row[EV["tstart"]],
                    "pstart_psia": row[EV["pstart"]] if isnum(row[EV["pstart"]]) else NAN,
                    "sid": str(row[EV["sid"]] or "").strip(), "type": row[EV["type"]],
                    "fuel_type": row[EV["fueltype"]], "start_frac": frac, "start_total_l": tot,
                    "sha256": nasa.sha(inv, name)})
            wb.close()
    return out, sources


def calc_event(calc, inv, test, a=None):
    ev = calc.get((inv, test))
    if not ev:
        return None
    if len(ev) == 1 or a is None or not isnum(a.hdr.get("clock", NAN)):
        return ev[0]
    # several events with one TPID: the one whose start is closest to the data file's start
    def gap(e):
        t = e["tstart"]
        if isinstance(t, dt.datetime):
            s = t.hour * 3600 + t.minute * 60 + t.second
            return abs(s - a.hdr["clock"])
        return 1e9
    return min(ev, key=gap)


# =================================================================================================
# Comparison helpers
# =================================================================================================
class Family:
    """One comparison family: every (table, reading) pair of one investigation, field and method.
    The family's own distribution is check 1 (is it ours?)."""

    def __init__(self, inv, field, method, kind="ratio", floor=0.03, abs_floor=0.0):
        self.inv, self.field, self.method, self.kind = inv, field, method, kind
        self.floor, self.abs_floor = floor, abs_floor
        self.items = []

    def add(self, row, t, s, ctx):
        if isnum(t) and isnum(s):
            self.items.append((row, t, s, ctx))

    def stat(self, x):
        t, s = x[1], x[2]
        if self.kind == "ratio":
            return math.log(t / s) if t > 0 and s > 0 else NAN
        return t - s

    def centre(self):
        v = [self.stat(x) for x in self.items]
        return med(v), robust_sigma(v), v

    def summary(self):
        c, sg, v = self.centre()
        v = [x for x in v if isnum(x)]
        if self.kind == "ratio":
            return {"n": len(self.items), "median_ratio_table_over_reading": r6(math.exp(c)) if isnum(c) else None,
                    "p5": r6(math.exp(pct(v, .05))) if v else None, "p95": r6(math.exp(pct(v, .95))) if v else None,
                    "robust_sigma_log": r6(sg)}
        return {"n": len(self.items), "median_difference": r6(c), "p5": r6(pct(v, .05)),
                "p95": r6(pct(v, .95)), "robust_sigma": r6(sg)}


# =================================================================================================
# ACME per-test tables: PSI-159 CFI-G and PSI-10 Flame Design
# =================================================================================================
def acme_bottles(a, calc_ev, sources, table_fuel_comp, orientation):
    """Composition of the MFCA3 fuel bottle and the MFCA1 oxidiser bottle, with their sources."""
    c3, c1, c2 = code(a, 2), code(a, 0), code(a, 1)
    fuel = {"from_code": CODES_MFCA3.get(c3) if c3 is not None else None, "from_calc": None,
            "from_table": table_fuel_comp}
    if calc_ev and calc_ev.get("sid") in sources and orientation == "normal":
        comp = sources[calc_ev["sid"]]
        if any(k in comp for k in ("methane", "ethylene", "propane", "butane", "ethane", "h2")):
            fuel["from_calc"] = comp
    ox = CODES_MFCA1.get(c1) if c1 is not None else None
    dil = CODES_MFCA2.get(c2) if c2 is not None else {"n2": 1.0}
    return fuel, ox, dil


def comp_from_name(raw):
    """'30% propane', '100% C2H4', 'ethane', '30% n-butane' -> {species: fraction, n2: rest}."""
    if not isinstance(raw, str):
        return None
    s = raw.strip().lower()
    m = re.match(r"^\s*(\d+(?:\.\d+)?)\s*%\s*([a-z0-9\-]+)", s)
    if m:
        sp = FUEL_WORDS.get(m.group(2))
        if not sp:
            return None
        x = float(m.group(1)) / 100.0
        rest = 1.0 - x
        dil = "he" if re.search(r"\bHe\b|helium", raw, re.I) else "n2"
        return {sp: x, dil: rest} if rest > 1e-9 else {sp: 1.0}
    sp = FUEL_WORDS.get(s)
    return {sp: 1.0} if sp else None


def same_comp(c1, c2, tol=0.011):
    if not c1 or not c2:
        return None
    keys = set(c1) | set(c2)
    return all(abs(c1.get(k, 0) - c2.get(k, 0)) <= tol for k in keys)


def comp_text(c):
    return None if not c else {k: r6(v) for k, v in sorted(c.items())}


def acme_per_test(nasa, atlas, calc, sources):
    """Flows, composition, pressure and ambient of the per-test tables against the files."""
    fams = {}
    tests = []
    for inv in ("PSI-159", "PSI-10"):
        for row in atlas.get(inv, []):
            if row["level"] != "test":
                continue
            name = (alias(row, "table_test") or row["test_id"]).replace(" ", "")
            if ":" in name:
                name = name.split(":")[-1]
            file_name = alias(row, "listing") or name
            orient = val(row, "geometry.flame_orientation.value_si")
            tests.append((inv, row, name, file_name, orient))
    read = {}
    for inv, row, name, file_name, orient in tests:
        read[(inv, name)] = read_acme(inv, file_name, nasa, light_k=2 if orient == "normal" else 0)
    status = Counter((inv, read[(inv, name)].status) for inv, row, name, fname, o in tests)

    def fam(inv, field, method, **kw):
        key = (inv, field, method)
        if key not in fams:
            fams[key] = Family(inv, field, method, **kw)
        return fams[key]

    per = []
    for inv, row, name, file_name, orient in tests:
        a = read[(inv, name)]
        if a.status != "ok":
            continue
        ev = calc_event(calc, inv, file_name, a)
        tf_raw = val(row, "fuel.name.raw_value")
        table_comp = comp_from_name(tf_raw) if inv == "PSI-159" else None
        if inv == "PSI-10" and orient == "normal":
            table_comp = {FUEL_WORDS.get(str(tf_raw).lower(), "ethylene"): 1.0}
        fuel_b, ox_b, dil_b = acme_bottles(a, ev, sources, table_comp, orient)
        fuel_comp = fuel_b["from_calc"] or fuel_b["from_code"] or table_comp
        sf = script_flows(a, orient)
        cmd_slpm = [v2slpm(a, k, a.cmd[k]) for k in range(3)]
        rb_slpm = [v2slpm(a, k, a.rb[k]) if a.rb_settled[k] else NAN for k in range(3)]
        ctx = dict(a=a, ev=ev, fuel_b=fuel_b, ox_b=ox_b, dil_b=dil_b, fuel_comp=fuel_comp, sf=sf,
                   cmd=cmd_slpm, rb=rb_slpm, orient=orient, name=name, file_name=file_name)
        per.append((inv, row, ctx))
        # streams: plan(test), plan(ignition), command at ignition, readback at ignition
        if orient == "normal":
            mk = lambda f: stream(f[2], fuel_comp, f[1], dil_b, 0.0, None)
        else:
            mk = lambda f: stream(0.0, None, f[1], dil_b, f[0], ox_b)
        rel = RELEVANT[orient]
        plan_ok = bool(sf) and all(isnum(sf["test"][k]) for k in rel)
        ctx["plan_test"] = mk(sf["test"]) if plan_ok else None
        ctx["plan_ign"] = mk(sf["ign"]) if sf and all(isnum(sf["ign"][k]) for k in rel) else None
        ctx["plan_stages"] = [(sg["stage"], mk(sg["flows"])) for sg in sf["stages"]] if sf else []
        # the execution is observable before ignition only if every stream the script lights the
        # flame with is already commanded at the ignition stamp (and no other stream is)
        if sf and all(isnum(sf["ign"][k]) for k in rel):
            observable = all((sf["ign"][k] > 0) == (cmd_slpm[k] > 0) for k in rel)
        else:
            observable = all(cmd_slpm[k] > 0 for k in rel if k != 1) and any(cmd_slpm[k] > 0 for k in rel)
        ctx["observable"] = observable
        ctx["cmd_ign"] = mk(cmd_slpm) if observable else None
        # the reading in which every stream whose readback stays at zero is taken as not flowing
        zero_read = [k for k in rel if a.rb_state[k] == "zero_read"]
        mixed = [0.0 if k in zero_read else cmd_slpm[k] for k in range(3)]
        ctx["zero_read"] = zero_read
        ctx["mixed_ign"] = mk(mixed) if observable and zero_read else None
        ctx["rb_ign"] = mk(rb_slpm) if observable and all(isnum(rb_slpm[k]) for k in rel) else None
        ctx["multi_stage"] = bool(sf) and sf["multi_stage"]
        ctx["ign_is_test"] = plan_ok and ctx["plan_ign"] is not None and not ctx["multi_stage"] and all(
            abs(sf["ign"][k] - sf["test"][k]) <= 0.005 * max(sf["test"][k], 1e-9) + 1e-6 for k in rel)
        if not sf:   # no script: the command at ignition is the only reading of the test flow
            ctx["ign_is_test"] = observable
        # families for check 1. PSI-10 tabulates the masses of each stream; its mole fraction follows
        # from them (checked inside the row, check 3), so the masses are compared and not the fraction.
        if orient == "normal":
            pairs = [("injection.fuel_mass_flow", "fuel_mg_s", 1e6, "ratio")]
            pairs += ([("injection.inert_mass_flow", "inert_mg_s", 1e6, "ratio")] if inv == "PSI-10" else
                      [("injection.x_fuel_injected", "x_fuel", 1.0, "diff")])
        else:
            pairs = [("injection.oxidizer_mass_flow", "o2_mg_s", 1e6, "ratio")]
            pairs += ([("injection.inert_mass_flow", "inert_mg_s", 1e6, "ratio")] if inv == "PSI-10" else
                      [("oxidizer_stream.x_o2", "x_o2", 1.0, "diff")])
        ctx["pairs"] = pairs
        for fpath, key, scale, kind in pairs:
            tv = val(row, fpath)
            if not resolved(tv):
                continue
            t = tv["value_si"] * scale
            if kind == "ratio" and t <= 0:
                continue
            if ctx["plan_test"]:
                fam(inv, fpath, f"plan_test:{orient}", kind=kind,
                    floor=0.03, abs_floor=0.004).add(row, t, ctx["plan_test"][key], ctx)
            if ctx["ign_is_test"] and ctx["cmd_ign"]:
                fam(inv, fpath, f"command_at_ignition:{orient}", kind=kind,
                    floor=0.03, abs_floor=0.004).add(row, t, ctx["cmd_ign"][key], ctx)
    return per, fams, status, read


def judge_family(f, items_ok=None):
    """Return (centre, sigma, threshold) for a family; ratio families in log space."""
    c, sg, v = f.centre()
    if f.kind == "ratio":
        thr = max(math.log(1 + f.floor), 5 * sg if isnum(sg) else 0)
    else:
        thr = max(f.abs_floor, 5 * sg if isnum(sg) else 0)
    return c, sg, thr


def fmt_stream_value(s, key):
    return None if not s else r6(s.get(key))


def acme_flow_records(per, fams):
    """Discrepancy records of the per-test ACME tables (flows and injected composition)."""
    summaries, centres = {}, {}
    for key, f in fams.items():
        c, sg, thr = judge_family(f)
        centres[key] = (c, sg, thr)
        summaries["|".join(key)] = f.summary() | {"threshold": r6(math.exp(thr) if f.kind == "ratio" else thr)}
    for inv, row, ctx in per:
        a, orient = ctx["a"], ctx["orient"]
        for fpath, key, scale, kind in ctx["pairs"]:
            tv = val(row, fpath)
            if not resolved(tv):
                continue
            t = tv["value_si"] * scale
            if kind == "ratio" and t <= 0:
                continue
            fk_e = (inv, fpath, f"command_at_ignition:{orient}")
            fk_p = (inv, fpath, f"plan_test:{orient}")
            use_e = bool(ctx["ign_is_test"] and ctx["cmd_ign"] and fk_e in centres)
            if not use_e and not (ctx["plan_test"] and fk_p in centres):
                continue
            COMPARED[(inv, fpath)] += 1
            fk_ref = fk_e if use_e else fk_p
            c, sg, thr = centres[fk_ref]
            fac = math.exp(c) if kind == "ratio" else c

            def dev(s):
                if not isnum(s):
                    return NAN
                if kind == "ratio":
                    if s <= 0:
                        return math.inf
                    return math.log(t / s) - c
                return (t - s) - c

            def basis(s):
                return (s * fac if kind == "ratio" else s + fac) if isnum(s) else NAN

            def bad(d):
                return d == math.inf or (isnum(d) and abs(d) > thr)
            E = ctx["cmd_ign"][key] if use_e else NAN
            P = ctx["plan_test"][key] if ctx["plan_test"] else NAN
            M = ctx["mixed_ign"][key] if (use_e and ctx["mixed_ign"]) else NAN
            dE, dP, dM = dev(E), dev(P), dev(M)
            bE, bP, bM = bad(dE), bad(dP), bad(dM)
            if not (bE or bP or bM):
                AGREE[(inv, fpath)] += 1
                continue
            if use_e and not bE and not bM and bP:
                AGREE[(inv, fpath)] += 1      # the table follows the execution; only the script differs
                continue
            devs = "; ".join(f"{nm} {r6(dv)}" for nm, dv in (("command", dE), ("script", dP), ("readback-informed", dM))
                             if isnum(dv) or dv == math.inf)
            checks = [chk(1, CHECK_NAMES[1],
                          f"the same method on all {len(fams[fk_ref].items)} tests of the family gives a common "
                          f"table/reading {'ratio' if kind == 'ratio' else 'difference'} of {r6(fac)} "
                          f"(robust sigma {r6(sg)}); this test's deviations from it: {devs}; threshold {r6(thr)} "
                          f"({'log ratio' if kind == 'ratio' else 'absolute'}): not the common factor")]
            verdict, decided, status, note = "unresolved", None, "seen_once", ""
            reading, which = (E, "command") if use_e else (P, "script")
            # --- check 2: a third NASA source ------------------------------------------------------
            if use_e:
                plan_eq_table = isnum(dP) and not bP
                plan_eq_exec = isnum(P) and isnum(E) and (abs(math.log(P / E)) < 0.01 if kind == "ratio" and P > 0 and E > 0
                                                          else abs(P - E) < 0.002)
                if bE and plan_eq_table:
                    checks.append(chk(2, CHECK_NAMES[2], "the script (as-run parameter file) gives the table's value;"
                                      " the command log at the ignition stamp differs from both", True))
                    verdict, decided = "table_records_plan_not_execution", 2
                elif bE and plan_eq_exec:
                    checks.append(chk(2, CHECK_NAMES[2], "the script and the command log agree with each other and"
                                      " not with the table", True))
                    verdict, decided = "table_value_wrong", 2
                elif not bE and bM:
                    reading, which = M, "readback"
                    checks.append(chk(2, CHECK_NAMES[2], "script and command agree with the table; the settled"
                                      " readback before ignition of at least one stream does not"))
                    status = "unsettled"
                else:
                    checks.append(chk(2, CHECK_NAMES[2], "script, command and table do not settle it"))
            else:
                checks.append(chk(2, CHECK_NAMES[2], "only the script is readable for the test flow before ignition"
                                  " (the flame is lit with another flow, or the command comes after the stamp)"))
                # does the table hold another stage of the script?
                for stg, sv in ctx["plan_stages"]:
                    if stg != (ctx["sf"] or {}).get("main_stage") and sv and not bad(dev(sv[key])):
                        checks.append(chk(5, CHECK_NAMES[5], f"the table value is the script's stage {stg} flow, not the"
                                          f" longest stage {ctx['sf']['main_stage']}: the same quantity at another"
                                          " stage of the plan", True))
                        verdict, decided = "two_definitions", 5
                        reading, which = sv[key], f"script stage {stg}"
                        break
            # --- status from an independent reading -------------------------------------------------
            if use_e and status != "unsettled":
                ind = independent_reading(ctx, key, kind, E)
                if ind is True:
                    status = "confirmed"
                elif ind is False:
                    status = "unsettled"
            # --- check 3: physics ------------------------------------------------------------------------
            phys = flow_physics(inv, row, orient, ctx, key, t)
            if phys:
                dec = decided is None and phys["decides"]
                checks.append(chk(3, CHECK_NAMES[3], phys["text"], dec))
                if dec:
                    verdict, decided = phys["verdict"], 3
            # --- check 4: shape --------------------------------------------------------------------------
            prev_t, prev_reading = previous_row(per, inv, row, fpath, scale, key)
            # "equal to the row above" is a copy only when the row above's own reading is that value
            prev = prev_t if (isnum(prev_reading) and isnum(prev_t) and
                              abs(prev_reading * (fac if kind == "ratio" else 1) - prev_t) <= thr * max(abs(prev_t), 1)) else None
            shp = shape_of(t, basis(reading), prev, basis(P) if (use_e and which != "readback") else None)
            if shp:
                # with readings that disagree among themselves the shape is described, not used
                dec = decided is None and shp != "equal to the planned value" and status != "unsettled"
                checks.append(chk(4, CHECK_NAMES[4], shp + ("" if shp != "equal to the row above" else
                                  ": the row above is read at that value and this row is not"), dec))
                if dec:
                    verdict, decided = "table_value_wrong", 4
            else:
                checks.append(chk(4, CHECK_NAMES[4], "no factor of 2 or 10, not a copy of the row above, not the plan"))
            if decided is None and not any(x["check"] == 5 for x in checks):
                checks.append(chk(5, CHECK_NAMES[5], two_def_text(fpath, ctx)))
            sensor = {"value": r6(basis(reading)), "unit": tv.get("raw_unit"),
                      "file": a.files["setpoints"] if which == "command" else (a.files["data"] if which == "readback" else a.files["script"]),
                      "channel": {"command": ["MFCA1Setpoint(V)", "MFCA2Setpoint(V)", "MFCA3Setpoint(V)"],
                                  "readback": ["MFCA1StaticMixerFlow", "MFCA2BypassFlow", "MFCA3FuelFlow"]}.get(
                                      which, [f"{which}: oxidizer/nitrogen/fuel TestFlow (slpm, nitrogen equivalent)"]),
                      "method": METHOD_TEXT[which if which in METHOD_TEXT else "script"],
                      "value_controller_basis": r6(reading),
                      ("family_factor_table_over_reading" if kind == "ratio" else "family_difference_table_minus_reading"): r6(fac),
                      "sha256": a.shas.get({"command": "setpoints", "readback": "data"}.get(which, "script"))}
            others = []
            if which != "script" and isnum(P):
                others.append({"source": "script (as run), longest constant-flow stage", "file": a.files["script"],
                               "value": r6(basis(P)), "stage": (ctx["sf"] or {}).get("main_stage")})
            if which == "readback" and isnum(E):
                others.append({"source": "command log at the ignition stamp", "file": a.files["setpoints"],
                               "value": r6(basis(E))})
            if which == "command" and isnum(M) and abs(M - E) > 1e-9:
                others.append({"source": "settled readbacks before ignition (command where not settled)",
                               "file": a.files["data"], "value": r6(basis(M))})
            others.append({"source": "bottles", "fuel_from_chamber_calculator": comp_text(ctx["fuel_b"]["from_calc"]),
                           "fuel_from_header_gas_code": comp_text(ctx["fuel_b"]["from_code"]),
                           "fuel_from_table_name": comp_text(ctx["fuel_b"]["from_table"]),
                           "oxidiser_from_header_gas_code": comp_text(ctx["ox_b"]),
                           "calculator_event": ({"file": ctx["ev"]["file"], "row": ctx["ev"]["row"], "source_id": ctx["ev"]["sid"]}
                                                if ctx["ev"] else None)})
            ref = basis(reading)
            diff = {"kind": kind,
                    "table_over_reading" if kind == "ratio" else "table_minus_reading":
                        r6((t / ref) if kind == "ratio" and isnum(ref) and ref else (t - ref) if isnum(ref) else NAN),
                    "deviation": r6(dev(reading)), "method_error": r6(sg), "threshold": r6(thr),
                    "n_sigma": r6(abs(dev(reading)) / sg) if isnum(sg) and sg > 1e-9 and isnum(dev(reading)) else None,
                    "space": "log ratio" if kind == "ratio" else "difference"}
            record(inv, row, fpath, table_side(tv, fpath) | {"value": r6(t), "unit": tv.get("raw_unit")},
                   sensor, diff, status, checks, verdict, decided, note, others, test_id=row["test_id"])
    return summaries


METHOD_TEXT = {
    "command": "last command of the setpoint log at or before the ignition stamp (+0.999 s, the stamp's resolution);"
               " nitrogen-equivalent slpm = V/5 x header full scale; molar flow with the Hastings gas factors of the"
               " bottle (mixture rule 1/sum(x/GCF)) at 0 C and 760 torr; put on the table's basis with the family factor",
    "readback": "settled readbacks before the ignition stamp (zero offset removed; command where a stream is not settled);"
                " same conversion and family factor",
    "script": "the script's (as-run file's) constant-flow stage; same conversion; family factor of the script comparison",
}


def independent_reading(ctx, key, kind, E):
    """True if the settled readbacks before ignition confirm the command for this quantity, False if they
    contradict it, None if no stream that carries the quantity is settled."""
    a = ctx["a"]
    rel = RELEVANT[ctx["orient"]]
    # the streams that carry the quantity
    carry = {"fuel_mg_s": (2,), "o2_mg_s": (0,), "inert_mg_s": (1,), "n2_mg_s": (1,)}.get(key, rel)
    if key in ("x_fuel", "x_o2") and ctx["cmd"][1] == 0 and a.rb_settled[1] and isnum(ctx["rb"][1]):
        # no diluent: the fraction is the bottle's whatever the bottle flow; a zero diluent readback
        # is the independent reading
        return abs(ctx["rb"][1]) <= 0.003 / 5.0 * fs(a, 1) + 1e-9
    settled = [k for k in carry if a.rb_settled[k] and isnum(ctx["rb"][k])]
    if not settled:
        return None
    if any(a.rb_state[k] == "zero_read" for k in carry):
        return False
    for k in settled:
        c, r = ctx["cmd"][k], ctx["rb"][k]        # slpm
        tol = RB_TOL_REL * abs(c) + RB_TOL_V / 5.0 * fs(a, k)
        if abs(r - c) > tol:
            return None                           # within the controller's offset band we cannot say
    return True if len(settled) == len(carry) else None


def flow_physics(inv, row, orient, ctx, key, t):
    """Check 3 for flows: no diluent flowing means the injected fraction is the bottle's; a PSI-10 row's
    components must add to its total and give its mole fraction."""
    a = ctx["a"]
    if key in ("x_fuel", "x_o2") and ctx.get("cmd_ign") is not None:
        k_dil = 1
        dil_off = ctx["cmd"][k_dil] == 0 and (not a.rb_settled[k_dil] or abs(ctx["rb"][k_dil]) < 0.003 / 5 * fs(a, k_dil) + 1e-9)
        bottle = ctx["fuel_comp"] if key == "x_fuel" else ctx["ox_b"]
        if dil_off and bottle:
            xb = sum(v for k, v in bottle.items() if (k == "o2") == (key == "x_o2") and k not in ("n2", "he", "co2"))
            rb_txt = "and its readback reads zero" if a.rb_settled[k_dil] else "(its readback is not settled)"
            return {"text": f"the diluent controller is commanded to zero before ignition {rb_txt}; with no diluent the"
                            f" injected fraction must be the bottle's, {r6(xb)}; the table gives {r6(t)}",
                    "decides": abs(t - xb) > 0.01, "verdict": "table_value_wrong"}
    pr = table_physics(inv, row, orient)
    if pr:
        return {"text": pr["text"], "decides": False, "verdict": None}
    return None


def table_physics(inv, row, orient):
    """PSI-10 tables give components and totals; check them against each other (check 3)."""
    if inv != "PSI-10":
        return None
    inj = row.get("injection", {})
    mt = inj.get("total_mass_flow")
    mi = inj.get("inert_mass_flow")
    if orient == "normal":
        mf, xf = inj.get("fuel_mass_flow"), inj.get("x_fuel_injected")
        sp = FUEL_WORDS.get(str(val(row, "fuel.name.raw_value")).lower(), "ethylene")
    else:
        mf, xf = inj.get("oxidizer_mass_flow"), row.get("oxidizer_stream", {}).get("x_o2")
        sp = "o2"
    if not all(resolved(x) for x in (mt, mi, mf, xf)):
        return None
    xfv = xf["value_si"]
    mfv, miv, mtv = [x["value_si"] * 1e6 for x in (mf, mi, mt)]
    sum_ok = abs(mfv + miv - mtv) <= 0.01 * max(mtv, 1e-9)
    nf, ni = mfv / MW[sp], miv / MW["n2"]
    x_calc = nf / (nf + ni) if (nf + ni) > 0 else NAN
    x_ok = isnum(x_calc) and abs(x_calc - xfv) <= 0.01
    text = (f"row: components {r6(mfv)} + {r6(miv)} vs total {r6(mtv)} mg/s ({'consistent' if sum_ok else 'inconsistent'});"
            f" mole fraction from the row's masses {r6(x_calc)} vs tabulated {r6(xfv)} "
            f"({'consistent' if x_ok else 'inconsistent'})")
    return {"text": text, "decides": False, "sum_ok": sum_ok, "x_ok": x_ok}


def previous_row(per, inv, row, fpath, scale, key):
    """The table value of the row above (same table file) and that row's own reading."""
    rows = [(r, c) for i, r, c in per if i == inv and r["source_rows"] and row["source_rows"] and
            r["source_rows"][0]["file"] == row["source_rows"][0]["file"]]
    rows.sort(key=lambda rc: rc[0]["source_rows"][0]["row"] or 0)
    for i, (r, c) in enumerate(rows):
        if r is row and i > 0:
            pr, pc = rows[i - 1]
            if pr["source_rows"][0]["row"] != row["source_rows"][0]["row"] - 1:
                return None, None
            v = val(pr, fpath)
            pt = v["value_si"] * scale if resolved(v) else None
            s = pc["cmd_ign"] if (pc["ign_is_test"] and pc["cmd_ign"]) else pc["plan_test"]
            return pt, (s[key] if s else None)
    return None, None


def two_def_text(fpath, ctx):
    if "x_" in fpath:
        return ("a mole fraction of the injected stream on both sides; the table could be by volume at a different"
                " standard, which changes a ratio of gases by the ratio of their gas factors only")
    return "a mass flow on both sides; the standard temperature is absorbed in the family factor"


def readback_offsets(per):
    """Check 1 for the readbacks: settled readback minus command (V) per investigation and controller,
    over tests whose command was held at least ZERO_READ_HOLD_S before ignition."""
    out = defaultdict(list)
    for inv, row, ctx in per:
        a = ctx["a"]
        for k in range(3):
            if a.cmd[k] > 0 and a.rb_state[k] == "settled" and a.cmd_hold[k] >= ZERO_READ_HOLD_S:
                out[(inv, k)].append(a.rb[k] - a.cmd[k])
    return {key: {"n": len(v), "median_V": r6(med(v)), "p10_V": r6(pct(v, .1)), "p90_V": r6(pct(v, .9))}
            for key, v in out.items()}


def annotate_unsettled(per, offsets):
    """Unsettled flow records: say how large the controller's usual offset is next to its command."""
    by_test = {(inv, row["test_id"]): ctx for inv, row, ctx in per}
    for rec in RECORDS:
        if rec["status"] != "unsettled" or rec["investigation"] not in ("PSI-10", "PSI-159"):
            continue
        ctx = by_test.get((rec["investigation"], rec["test_id"]))
        if not ctx:
            continue
        a = ctx["a"]
        parts = []
        for k in ctx.get("zero_read", []):
            o = offsets.get((rec["investigation"], k), {})
            parts.append(f"MFCA{k + 1} is commanded {r6(a.cmd[k])} V ({r6(100 * a.cmd[k] / 5)} % of full scale) for "
                         f"{r6(a.cmd_hold[k])} s before ignition and reads {r6(a.rb[k])} V; the same controller's settled "
                         f"readback minus command over the investigation is {o.get('median_V')} V (p10 {o.get('p10_V')},"
                         f" p90 {o.get('p90_V')}, n {o.get('n')})")
        if parts:
            small = all(a.cmd[k] <= abs(offsets.get((rec["investigation"], k), {}).get("p10_V") or 0) * 1.5
                        for k in ctx["zero_read"])
            rec["note"] = ("; ".join(parts) + (". The command is within the controller's own offset band, so the "
                           "readback cannot tell flow from no flow." if small else
                           ". The command is well above the offset band; the readback before ignition says no flow,"
                           " the command and the script say flow. What flowed after ignition is not read."))


def factor_records(fams):
    """A common factor between table and reading on every test is a unit or a definition (check 1). One
    record per investigation and quantity where it exceeds the method's spread."""
    seen = set()
    for (inv, fpath, method), f in sorted(fams.items()):
        if f.kind != "ratio" or not method.startswith("command_at_ignition") or len(f.items) < 10:
            continue
        c, sg, v = f.centre()
        if not (isnum(c) and isnum(sg)) or abs(c) < math.log(1.02) or abs(c) < 5 * sg:
            continue
        if (inv, fpath) in seen:
            continue
        seen.add((inv, fpath))
        fac = math.exp(c)
        vv = [x for x in v if isnum(x)]
        t_implied = 273.15 / fac - 273.15     # the standard temperature the table's mass flows imply
        big = fac < 0.95
        checks = [chk(1, CHECK_NAMES[1], f"the table/reading ratio is {r6(fac)} on all {len(vv)} tests (p5 "
                      f"{r6(math.exp(pct(vv, .05)))}, p95 {r6(math.exp(pct(vv, .95)))}): one factor, not a defect of rows", True)]
        if big:
            checks.append(chk(5, CHECK_NAMES[5],
                              f"the controllers' slpm is at 0 C and 760 torr (ACME data interpretation documents); a ratio of"
                              f" {r6(fac)} is what a standard litre at {r6(t_implied)} C gives (25 C gives"
                              f" {r6(273.15 / 298.15)}): the table converts the same command with another standard temperature", True))
        else:
            checks.append(chk(5, CHECK_NAMES[5],
                              f"a common {r6(100 * (fac - 1))} % offset, within a controller's accuracy; with the documented gas"
                              " factors the table's own conversion is not identified (for ethylene a factor of 0.60 instead of"
                              f" 0.6062 would give {r6(0.6 / 0.6062)})"))
        ts = table_side(val(f.items[0][0], fpath), fpath)
        record(inv, None, fpath, {"field_path": fpath, "column": ts["column"], "file": ts["file"], "value": None,
                                  "unit": "mg/s", "scope": "every test of the family"},
               {"value": None, "file": "<test>_mfcSetpoints.csv (every test)", "channel": "MFCA setpoints",
                "method": METHOD_TEXT["command"].replace("; put on the table's basis with the family factor", ""),
                "family": method},
               {"kind": "ratio", "table_over_reading": r6(fac), "reading_over_table": r6(1 / fac),
                "method_error": r6(sg), "p5": r6(math.exp(pct(vv, .05))), "p95": r6(math.exp(pct(vv, .95))),
                "n_tests": len(vv), "space": "log ratio"},
               "confirmed", checks, "two_definitions" if big else "unresolved", 5 if big else None,
               "common factor of the family; the per-test comparisons are made after removing it",
               test_id=f"{inv}:all tests ({method.split(':')[1]} flames)", group_id=f"{inv}:all")
        RECORDS[-1]["acronym"] = f.items[0][0].get("acronym")


def acme_conditions_records(per):
    """Pressure and chamber atmosphere before ignition against the chamber calculator (CIR transducers and
    the as-run fill record) and the ACME transducer; test ids and dates against file contents and names."""
    fam_p = defaultdict(list)
    fam_x = defaultdict(list)
    pta = defaultdict(list)
    for inv, row, ctx in per:
        a, ev = ctx["a"], ctx["ev"]
        if ev is None:
            continue
        tp = val(row, "atmosphere.pressure")
        if resolved(tp) and isnum(ev["pstart_psia"]):
            fam_p[inv].append((row, ctx, tp["value_si"], ev["pstart_psia"] * PSI_PA))
        if isnum(a.pta3) and isnum(ev["pstart_psia"]):
            pta[inv].append((a.pta3, ev["pstart_psia"]))
        for fpath, species in (("atmosphere.x_o2", ("o2",)),
                               ("atmosphere.x_fuel_ambient", ("ethylene", "methane", "propane", "butane", "ethane", "h2"))):
            tv = val(row, fpath)
            if resolved(tv) and ev["start_frac"]:
                fam_x[(inv, fpath)].append((row, ctx, tv["value_si"], sum(ev["start_frac"].get(s, 0.0) for s in species)))
    # PTA3: a straight line through the calculator's pressure, per investigation (our reading of an
    # uncalibrated transducer; ACME s-Flame interpretation document, section 4.A.1)
    lines = {}
    for inv, v in pta.items():
        if len(v) >= 10:
            xs, ys = [x[0] for x in v], [x[1] for x in v]
            mx, my = st.mean(xs), st.mean(ys)
            sxx = sum((x - mx) ** 2 for x in xs)
            b = sum((x - mx) * (y - my) for x, y in zip(xs, ys)) / sxx if sxx > 0 else NAN
            lines[inv] = (my - b * mx, b)
    PTA_LINES.update(lines)
    out = {"pta3_line_psia": {k: {"a": r6(v[0]), "b_per_V": r6(v[1])} for k, v in lines.items()}}
    for inv, items in fam_p.items():
        logs = [math.log(t / c) for _, _, t, c in items if t > 0 and c > 0]
        c0, sg = med(logs), robust_sigma(logs)
        thr = max(math.log(1.03), 5 * sg if isnum(sg) else 0)
        out[f"{inv}|pressure"] = {"n": len(logs), "median_ratio": r6(math.exp(c0)), "robust_sigma_log": r6(sg),
                                  "threshold": r6(math.exp(thr))}
        for row, ctx, t, cpa in items:
            COMPARED[(inv, "atmosphere.pressure")] += 1
            d = math.log(t / cpa) - c0
            if abs(d) <= thr:
                AGREE[(inv, "atmosphere.pressure")] += 1
                continue
            a, ev = ctx["a"], ctx["ev"]
            pta_pa = (lines[inv][0] + lines[inv][1] * a.pta3) * PSI_PA if inv in lines and isnum(a.pta3) else NAN
            agrees_calc = isnum(pta_pa) and abs(math.log(pta_pa / cpa)) <= thr
            agrees_table = isnum(pta_pa) and abs(math.log(pta_pa / t)) <= thr
            status = "confirmed" if agrees_calc else ("unsettled" if agrees_table else "seen_once")
            checks = [chk(1, CHECK_NAMES[1], f"table/calculator pressure ratio over {len(logs)} tests is {r6(math.exp(c0))}"
                          f" (robust sigma {r6(sg)}); this test is {r6(d)} from it (threshold {r6(thr)})")]
            prev = previous_value(per, inv, row, "atmosphere.pressure")
            verdict, dec = "unresolved", None
            if agrees_calc:
                checks.append(chk(2, CHECK_NAMES[2], f"the ACME transducer PTA3, read with our line, gives {r6(pta_pa)} Pa:"
                                  " it agrees with the calculator and not with the table", True))
                verdict, dec = "table_value_wrong", 2
            elif agrees_table:
                checks.append(chk(2, CHECK_NAMES[2], f"PTA3 gives {r6(pta_pa)} Pa, with the table: the calculator row may not"
                                  " belong to this test point (a repeated TPID or another fill)"))
            else:
                checks.append(chk(2, CHECK_NAMES[2], "PTA3 not readable or agrees with neither"))
            shp = shape_of(t, cpa, prev)
            use = bool(shp) and dec is None and status != "unsettled"
            checks.append(chk(4, CHECK_NAMES[4], shp or "no simple shape", use))
            if use:
                verdict, dec = "table_value_wrong", 4
            tv = val(row, "atmosphere.pressure")
            record(inv, row, "atmosphere.pressure",
                   table_side(tv, "atmosphere.pressure") | {"value": tv["raw_value"], "unit": tv["raw_unit"]},
                   {"value": r6(ev["pstart_psia"]), "unit": "psia", "value_si": r6(cpa), "file": ev["file"],
                    "channel": "EventData Pstart (psia), CIR transducers", "row": ev["row"],
                    "method": "chamber calculator row of this test point (TPID), pressure at the start of the event",
                    "sha256": ev["sha256"]},
                   {"kind": "ratio", "table_over_reading": r6(t / cpa), "deviation": r6(d), "method_error": r6(sg),
                    "threshold": r6(thr), "space": "log ratio"},
                   status, checks, verdict, dec, "",
                   [{"source": "ACME transducer PTA3 before ignition, straight line through the calculator",
                     "file": a.files.get("data"), "value_pa": r6(pta_pa), "volts": r6(a.pta3)}],
                   test_id=row["test_id"])
    # the table against the ACME transducer itself (independent of the calculator, from which a table
    # may have been copied)
    for inv, items in fam_p.items():
        if inv not in lines:
            continue
        a0, b0 = lines[inv]
        trip = [(row, ctx, t, (a0 + b0 * ctx["a"].pta3) * PSI_PA) for row, ctx, t, c in items if isnum(ctx["a"].pta3)]
        logs = [math.log(t / p) for _, _, t, p in trip if t > 0 and p > 0]
        c0, sg = med(logs), robust_sigma(logs)
        thr = max(math.log(1.03), 5 * sg if isnum(sg) else 0)
        out[f"{inv}|pressure_vs_pta3"] = {"n": len(logs), "median_ratio": r6(math.exp(c0)), "robust_sigma_log": r6(sg),
                                          "threshold": r6(math.exp(thr))}
        for row, ctx, t, p in trip:
            COMPARED[(inv, "atmosphere.pressure:PTA3")] += 1
            d = math.log(t / p) - c0
            if abs(d) <= thr:
                AGREE[(inv, "atmosphere.pressure:PTA3")] += 1
                continue
            if any(r["test_id"] == row["test_id"] and r["field"] == "atmosphere.pressure" for r in RECORDS):
                continue          # already a record against the calculator, with PTA3 in it
            a, ev = ctx["a"], ctx["ev"]
            tv = val(row, "atmosphere.pressure")
            cal_eq_table = abs(math.log(t / (ev["pstart_psia"] * PSI_PA))) < 0.003
            checks = [chk(1, CHECK_NAMES[1], f"table/PTA3 over {len(logs)} tests is {r6(math.exp(c0))} (robust sigma {r6(sg)});"
                          f" this test is {r6(d)} from it (threshold {r6(thr)})"),
                      chk(2, CHECK_NAMES[2], "the chamber calculator gives the table's value" +
                          (" digit for digit: it is the table's source and counts with the table" if cal_eq_table else ""))]
            record(inv, row, "atmosphere.pressure",
                   table_side(tv, "atmosphere.pressure") | {"value": tv["raw_value"], "unit": tv["raw_unit"]},
                   {"value": r6(p / PSI_PA), "unit": "psia", "value_si": r6(p), "file": a.files.get("data"),
                    "channel": "PTA3 (V), median of the 3 s before the ignition reference",
                    "method": f"straight line psia = {r6(a0)} + {r6(b0)} V through the calculator's start pressures of"
                              " the investigation (our reading of an uncalibrated transducer)", "volts": r6(a.pta3),
                    "sha256": a.shas.get("data")},
                   {"kind": "ratio", "table_over_reading": r6(t / p), "deviation": r6(d), "method_error": r6(sg),
                    "threshold": r6(thr), "space": "log ratio"},
                   "seen_once", checks, "unresolved", None,
                   "the ACME transducer has not been calibrated in years (s-Flame data interpretation, 4.A.1)",
                   [{"source": "chamber calculator Pstart", "file": ev["file"], "row": ev["row"],
                     "value_psia": r6(ev["pstart_psia"])}], test_id=row["test_id"])
    for (inv, fpath), items in fam_x.items():
        ds = [t - c for _, _, t, c in items]
        c0, sg = med(ds), robust_sigma(ds)
        thr = max(0.005, 5 * sg if isnum(sg) else 0)
        out[f"{inv}|{fpath}"] = {"n": len(ds), "median_difference": r6(c0), "robust_sigma": r6(sg), "threshold": r6(thr)}
        for row, ctx, t, cx in items:
            COMPARED[(inv, fpath)] += 1
            d = (t - cx) - c0
            if abs(d) <= thr:
                AGREE[(inv, fpath)] += 1
                continue
            ev = ctx["ev"]
            # a round value shared by the rows of the day while the calculator moves: the day's nominal value
            same_day = [tt for rr, cc, tt, xx in items if rr["group_id"] == row["group_id"]]
            nominal = (abs(t * 100 - round(t * 100)) < 1e-9 and sum(1 for tt in same_day if tt == t) >= 3)
            prev = previous_value(per, inv, row, fpath)
            shp = "the day's round nominal value, repeated on its rows" if nominal else shape_of(t, cx, prev)
            checks = [chk(1, CHECK_NAMES[1], f"table minus calculator over {len(ds)} tests is {r6(c0)} (robust sigma "
                          f"{r6(sg)}); this test is {r6(d)} from it (threshold {r6(thr)})"),
                      chk(2, CHECK_NAMES[2], "no third source of the chamber composition before ignition is read (the gas"
                          " chromatograph files are not tied to test points)"),
                      chk(4, CHECK_NAMES[4], shp or "no simple shape", bool(shp))]
            if nominal:
                verdict, dec = "table_records_plan_not_execution", 4
            else:
                verdict, dec = ("table_value_wrong", 4) if shp else ("unresolved", None)
            if not shp:
                checks.append(chk(5, CHECK_NAMES[5], "the calculator's fraction is of the whole start mixture, water and"
                                  " carbon dioxide included; a table on a dry or another basis would differ"))
            tv = val(row, fpath)
            record(inv, row, fpath, table_side(tv, fpath) | {"value": r6(t), "unit": "1"},
                   {"value": r6(cx), "unit": "1", "file": ev["file"], "row": ev["row"],
                    "channel": "EventData Start Mixture (standard litres per species)",
                    "method": "species litres over the sum of the start mixture of this test point's event",
                    "sha256": ev["sha256"]},
                   {"kind": "difference", "table_minus_reading": r6(t - cx), "deviation": r6(d), "method_error": r6(sg),
                    "threshold": r6(thr), "space": "difference"},
                   "seen_once", checks, verdict, dec, "", [], test_id=row["test_id"])
    for inv, row, ctx in per:
        acme_id_checks(inv, row, ctx["file_name"], ctx["a"])
    return out


PTA_LINES = {}


def previous_value(per, inv, row, fpath):
    rows = [r for i, r, c in per if i == inv and r["source_rows"] and row["source_rows"] and
            r["source_rows"][0]["file"] == row["source_rows"][0]["file"]]
    rows.sort(key=lambda r: r["source_rows"][0]["row"] or 0)
    for i, r in enumerate(rows):
        if r is row and i > 0 and rows[i - 1]["source_rows"][0]["row"] == row["source_rows"][0]["row"] - 1:
            v = val(rows[i - 1], fpath)
            return v["value_si"] if resolved(v) else None
    return None


def yyddd_date(code5):
    try:
        y, d = 2000 + int(code5[:2]), int(code5[2:5])
        return dt.date(y, 1, 1) + dt.timedelta(days=d - 1)
    except ValueError:
        return None


def acme_id_checks(inv, row, name, a):
    """The test name against the names written inside the files, and the dates."""
    COMPARED[(inv, "test_id")] += 1
    inner = [("setpoints header 'Test Point'", a.files.get("setpoints"), a.hdr.get("Test Point")),
             ("timeline 'Test Point'", a.files.get("tp"), a.tp_name),
             ("script 'testpointName'", a.files.get("script"), a.script.get("testpointName"))]
    bad = [(w, f, v) for w, f, v in inner if v and v.replace(" ", "") != name]
    if bad:
        for w, f, v in bad:
            others = [{"source": ww, "file": ff, "value": vv} for ww, ff, vv in inner if (ww, ff, vv) != (w, f, v) and vv]
            agree_others = [o for o in others if o["value"].replace(" ", "") == name]
            checks = [chk(1, CHECK_NAMES[1], "names are compared as text, spaces removed"),
                      chk(2, CHECK_NAMES[2], f"{len(agree_others)} other file(s) of the test carry the file's name")]
            record(inv, row, "test_id", {"field_path": "test_id", "value": name, "file": "file names of the test",
                                         "column": None},
                   {"value": v, "file": f, "channel": w, "method": "name written inside the file"},
                   {"kind": "text", "table": name, "reading": v}, "confirmed" if agree_others else "seen_once", checks,
                   "unresolved", None,
                   "the name inside one file differs from the file's own name; the table follows the file names",
                   others, test_id=row["test_id"])
    else:
        AGREE[(inv, "test_id")] += 1
    COMPARED[(inv, "date")] += 1
    d_name = yyddd_date(name[:5])
    try:
        d_hdr = dt.date.fromisoformat(a.hdr.get("date", ""))
    except ValueError:
        d_hdr = None
    tdate = val(row, "date")
    d_tab = None
    if resolved(tdate):
        try:
            d_tab = dt.date.fromisoformat(str(tdate["value_si"])[:10])
        except ValueError:
            d_tab = None
    probs = []
    if d_hdr and d_name and (d_hdr - d_name).days not in (0, 1):
        probs.append(("the setpoint file header's date against the day code of the test name", d_hdr.isoformat()))
    if d_hdr and a.hdr.get("doy") and (d_hdr - dt.date(d_hdr.year, 1, 1)).days + 1 != a.hdr["doy"]:
        probs.append(("the header's day of year against its own date", f"{d_hdr.isoformat()} / {a.hdr['doy']}"))
    if d_tab and d_hdr and (d_hdr - d_tab).days not in (0, 1):
        probs.append(("the setpoint file header's date against the table's date", d_hdr.isoformat()))
    if not probs:
        AGREE[(inv, "date")] += 1
        return
    for what, v in probs:
        record(inv, row, "date", table_side(tdate, "date") | {"value": d_tab.isoformat() if d_tab else None},
               {"value": v, "file": a.files.get("setpoints"), "channel": "header line", "method": what},
               {"kind": "days", "table_minus_reading": (d_tab - d_hdr).days if (d_tab and d_hdr) else None},
               "seen_once",
               [chk(1, CHECK_NAMES[1], "ACME day codes do not change after midnight (E-FIELD data interpretation,"
                    " section 4.A.1): a one-day difference is allowed")],
               "unresolved", None, "", [], test_id=row["test_id"])


# =================================================================================================
# ACME day tables: PSI-20 BRE, PSI-21 CLD Flame, PSI-22 E-FIELD Flames, PSI-23 s-Flame
# =================================================================================================
DAY_TABLES = ("PSI-20", "PSI-21", "PSI-22", "PSI-23")
EXPERIMENT_OF = {"PSI-10": 0, "PSI-21": 1, "PSI-23": 2, "PSI-22": 3, "PSI-20": 4}


def acme_day_tests(nasa, inv):
    pre = "" if inv == "PSI-21" else f"{inv}_Raw Data_"
    pat = re.compile(rf"^{re.escape(pre)}(\d{{5}}[A-Z]\d+)_mfcSetpoints\.csv$")
    return sorted({m.group(1) for n in nasa.names(inv) for m in [pat.match(n)] if m})


def components_of(row):
    comps = (row.get("fuel") or {}).get("components") or []
    out, ok = {}, True
    for c in comps:
        f = c.get("fraction") or {}
        if f.get("status") == "resolved" and isnum(f.get("value_si")):
            out[c["species"]] = float(f["value_si"])
        else:
            ok = False
    return out if (out and ok) else None


SPECIES_TOKEN = {"methane": "methane", "ethylene": "ethylene", "hydrogen": "h2", "helium": "he", "nitrogen": "n2",
                 "propane": "propane", "butane": "butane", "ethane": "ethane", "oxygen": "o2"}


def canon(comp):
    return {SPECIES_TOKEN.get(k, k): v for k, v in comp.items() if v > 0} if comp else None


def bottle_of(row):
    """The fuel bottle of a day table: the percent and species of the Fuel Bottle label (the species as the
    atlas reads it, which undoes the fill-series artefacts), balance nitrogen unless the label says helium."""
    nm = val(row, "fuel.name")
    if not resolved(nm):
        return None
    raw = str(nm.get("raw_value") or "")
    sp = SPECIES_TOKEN.get(str(nm["value_si"]), str(nm["value_si"]))
    m = re.match(r"\s*(\d+(?:\.\d+)?)\s*%", raw)
    if not m or "+" in str(nm["value_si"]):
        # blends: the atlas components, when the column is the bottle's
        comps = (row.get("fuel") or {}).get("components") or []
        if comps and all((((c.get("fraction") or {}).get("source") or {}).get("column") or "") == "Fuel Bottle" for c in comps):
            return canon(components_of(row))
        return {sp: 1.0} if not m else None
    x = float(m.group(1)) / 100.0
    if x >= 0.999:
        return {sp: 1.0}
    dil = "he" if re.search(r"\bHe\b|helium", raw, re.I) else "n2"
    return {sp: x, dil: 1.0 - x}


def injected_fraction(a, bottle):
    """Fuel mole fraction of the burner stream (MFCA3 bottle + MFCA2 diluent) from the commands."""
    if not bottle:
        return NAN
    f3, f2 = v2slpm(a, 2, a.cmd[2]), v2slpm(a, 1, a.cmd[1])
    dil = CODES_MFCA2.get(code(a, 1), {"n2": 1.0})
    s = stream(f3, bottle, f2, dil, 0.0, None)
    return s["x_fuel"]


def acme_day_records(nasa, atlas, calc, pta_lines):
    out = {}
    for inv in DAY_TABLES:
        rows = {r["test_id"]: r for r in atlas.get(inv, []) if r["level"] == "test_day"}
        tests = acme_day_tests(nasa, inv)
        by_day = defaultdict(list)
        for t in tests:
            by_day[t[:5]].append(t)
        reads = {}
        for t in tests:
            a = read_acme(inv, t, nasa, light_k=2)
            reads[t] = a
        st_counts = Counter(a.status for a in reads.values())
        out[f"{inv}|read_status"] = dict(st_counts)
        # --- dates: table days against the days that have test points in the files ----------------
        file_days = set(by_day)
        for day, row in rows.items():
            COMPARED[(inv, "date")] += 1
            code_day = alias(row, "test_day_yyddd") or day
            if code_day in file_days:
                AGREE[(inv, "date")] += 1
                continue
            near = sorted(d for d in file_days - set(rows) if abs(int(d) - int(code_day)) <= 40 or
                          sorted(d) == sorted(code_day))
            swapped = [d for d in file_days - set(rows) if sorted(d[2:]) == sorted(code_day[2:]) and d[:2] == code_day[:2]]
            checks = [chk(1, CHECK_NAMES[1], f"{len(file_days & set(rows))} of {len(rows)} table days have test points"
                          " in the files under the same day code")]
            verdict, dec = "unresolved", None
            if swapped:
                checks.append(chk(4, CHECK_NAMES[4], f"the files have test points on day {swapped[0]}, which no table row"
                                  " names and whose digits are this day's in another order", True))
                verdict, dec = "table_value_wrong", 4
            record(inv, row, "date", table_side(val(row, "date"), "date") | {"value": code_day},
                   {"value": None, "file": f"{inv} raw-data listing", "channel": "test point file names",
                    "method": "day code (first five digits) of every <YYDDD><letter><n>_mfcSetpoints.csv",
                    "nearest_file_days_not_in_table": near[:6]},
                   {"kind": "presence", "test_points_of_the_day": 0}, "seen_once", checks, verdict, dec,
                   "no test point of the listing carries this day code", [], test_id=day)
        for d in sorted(file_days - set(rows) - {alias(r, "test_day_yyddd") for r in rows.values()}):
            COMPARED[(inv, "date")] += 1
            record(inv, None, "date", {"field_path": "date", "value": None, "file": (next(iter(rows.values()))["source_rows"][0]["file"] if rows else None),
                                       "column": "(no row)"},
                   {"value": d, "file": f"{inv} raw-data listing", "channel": "test point file names",
                    "method": f"{len(by_day[d])} test points carry this day code", "examples": by_day[d][:3]},
                   {"kind": "presence", "test_points_of_the_day": len(by_day[d])}, "seen_once",
                   [chk(1, CHECK_NAMES[1], "a day of test points that the table does not list")],
                   "unresolved", None, "the table has no row for a day the files have test points on",
                   [], test_id=f"{inv}:day:{d}", group_id=f"{inv}:day:{d}")
        # --- per day: bottle, burner, polarity, controller full scales, experiment, fraction ----------
        for day, row in rows.items():
            code_day = alias(row, "test_day_yyddd") or day
            ts = [reads[t] for t in by_day.get(code_day, []) if reads[t].hdr]
            if not ts:
                continue
            # experiment code of the header
            exp_codes = Counter(a.hdr.get("Experiment") for a in ts)
            want = EXPERIMENT_OF.get(inv)
            COMPARED[(inv, "experiment_code")] += 1
            other = {k: v for k, v in exp_codes.items() if k != str(want)}
            if other:
                record(inv, row, "extra_conditions.test_type", {"field_path": "investigation", "value": EXPERIMENT_CODES.get(want),
                                                                "file": row["source_rows"][0]["file"], "column": None},
                       {"value": {EXPERIMENT_CODES.get(int(k), k) if k and k.isdigit() else k: v for k, v in exp_codes.items()},
                        "file": "<test>_mfcSetpoints.csv headers of the day", "channel": "Experiment",
                        "method": "header code, ACMEFlowCalculator.xlsx HeaderTables"},
                       {"kind": "count", "tests_of_another_experiment": sum(other.values()), "tests": len(ts)},
                       "seen_once", [chk(1, CHECK_NAMES[1], "header experiment code against the investigation of the table")],
                       "unresolved", None, "test points of this day are headed as another ACME experiment",
                       [{"tests": [a.test for a in ts if a.hdr.get("Experiment") != str(want)][:8]}], test_id=day)
            else:
                AGREE[(inv, "experiment_code")] += 1
            # fuel bottle
            tab = bottle_of(row)
            if tab:
                COMPARED[(inv, "fuel.components")] += 1
                codes = Counter(code(a, 2) for a in ts)
                mism = {c: n for c, n in codes.items() if c in CODES_MFCA3 and not same_comp(CODES_MFCA3[c], tab)}
                if mism:
                    match_any = any(same_comp(CODES_MFCA3[c], tab) for c in codes if c in CODES_MFCA3)
                    record(inv, row, "fuel.components", table_side(val(row, "fuel.name"), "fuel.name") | {"value": comp_text(tab)},
                           {"value": {str(c): comp_text(CODES_MFCA3[c]) for c in mism}, "file": "<test>_mfcSetpoints.csv headers",
                            "channel": "MFCA3 Gas", "method": "header gas code, ACMEFlowCalculator.xlsx HeaderTables/Tables",
                            "tests_per_code": {str(c): n for c, n in codes.items()}},
                           {"kind": "composition", "tests_with_another_bottle": sum(mism.values()), "tests": len(ts)},
                           "seen_once",
                           [chk(1, CHECK_NAMES[1], "the same code table reads every other day of the investigation"),
                            chk(2, CHECK_NAMES[2], ("the day also has tests with the table's bottle: a second bottle was"
                                                    " used that day and the table names one") if match_any else
                                "no test of the day carries the table's bottle")],
                           "unresolved" if match_any else "unresolved", None,
                           "", [{"tests": [a.test for a in ts if code(a, 2) in mism][:10]}], test_id=day)
                else:
                    AGREE[(inv, "fuel.components")] += 1
            # burner
            bd = None
            for s in (val(row, "geometry.sizes") or []):
                if s.get("dimension") == "burner_diameter" and resolved(s.get("value")):
                    bd = s["value"]["value_si"]
            cfg = val(row, "geometry.configuration.value_si")
            bcodes = Counter(a.hdr.get("Burner") for a in ts)
            exp_burner = None
            if bd is not None and inv == "PSI-20":
                exp_burner = {0.025: {"6"}, 0.05: {"7"}}.get(round(bd, 3))
            elif cfg == "coflow_burner":
                exp_burner = {"3"}
            elif cfg == "gas_jet_burner":
                exp_burner = {"4", "5"}
            if exp_burner:
                COMPARED[(inv, "geometry.burner")] += 1
                bad = {k: v for k, v in bcodes.items() if k not in exp_burner}
                if bad:
                    tv = val(row, "geometry.configuration") if inv != "PSI-20" else next(
                        s["value"] for s in val(row, "geometry.sizes") if s.get("dimension") == "burner_diameter")
                    record(inv, row, "geometry.burner", table_side(tv, "geometry") | {"value": tv.get("raw_value")},
                           {"value": {BURNER_CODES.get(int(k), k) if k and k.isdigit() else k: v for k, v in bcodes.items()},
                            "file": "<test>_mfcSetpoints.csv headers", "channel": "Burner",
                            "method": "header burner code, ACMEFlowCalculator.xlsx HeaderTables"},
                           {"kind": "count", "tests_with_another_burner": sum(bad.values()), "tests": len(ts)},
                           "seen_once", [chk(1, CHECK_NAMES[1], "header burner code against the table's burner")],
                           "unresolved", None, "", [{"tests": [a.test for a in ts if a.hdr.get("Burner") in bad][:10]}],
                           test_id=day)
                else:
                    AGREE[(inv, "geometry.burner")] += 1
            # mesh polarity
            pol = val(row, "extra_conditions.mesh_polarity.value_si")
            if pol in ("negative", "positive"):
                COMPARED[(inv, "extra_conditions.mesh_polarity")] += 1
                want_p = "1" if pol == "negative" else "2"
                pc = Counter(a.hdr.get("EFieldPol") for a in ts)
                bad = {k: v for k, v in pc.items() if k != want_p}
                if bad:
                    tv = val(row, "extra_conditions.mesh_polarity")
                    record(inv, row, "extra_conditions.mesh_polarity", table_side(tv, "extra_conditions.mesh_polarity") | {"value": pol},
                           {"value": {({"0": "not installed", "1": "negative", "2": "positive"}.get(k, k)): v for k, v in pc.items()},
                            "file": "<test>_mfcSetpoints.csv headers", "channel": "EFieldPol",
                            "method": "header polarity code, ACMEFlowCalculator.xlsx HeaderTables"},
                           {"kind": "count", "tests_with_another_polarity": sum(bad.values()), "tests": len(ts)},
                           "seen_once", [chk(1, CHECK_NAMES[1], "header polarity code against the table's polarity")],
                           "unresolved", None, "", [{"tests": [a.test for a in ts if a.hdr.get("EFieldPol") in bad][:10]}],
                           test_id=day)
                else:
                    AGREE[(inv, "extra_conditions.mesh_polarity")] += 1
            # controller full scales (CLD)
            for fpath, k in (("extra_conditions.fuel_mfc_full_scale", 2), ("extra_conditions.n2_mfc_full_scale", 1)):
                tv = val(row, fpath)
                if not resolved(tv):
                    continue
                COMPARED[(inv, fpath)] += 1
                want_fs = float(tv["raw_value"])
                fsc = Counter(fs(a, k) for a in ts)
                bad = {f: n for f, n in fsc.items() if not (isnum(f) and abs(f - want_fs) <= 1e-6)}
                if bad:
                    record(inv, row, fpath, table_side(tv, fpath) | {"value": want_fs, "unit": tv["raw_unit"]},
                           {"value": {str(f): n for f, n in fsc.items()}, "unit": "slpm (nitrogen)",
                            "file": "<test>_mfcSetpoints.csv headers", "channel": f"MFCA{k + 1} Full Scale",
                            "method": "header full scale"},
                           {"kind": "count", "tests_with_another_full_scale": sum(bad.values()), "tests": len(ts)},
                           "seen_once", [chk(1, CHECK_NAMES[1], "header full scale against the table, every day the same way")],
                           "unresolved", None, "", [{"tests": [a.test for a in ts if fs(a, k) in bad][:10]}], test_id=day)
                else:
                    AGREE[(inv, fpath)] += 1
            # injected fuel fraction (CLD range, E-FIELD list, s-Flame bottle) against the commands
            tv = val(row, "injection.x_fuel_injected")
            bottle = None
            if tab:
                bottle = tab
            col = ((tv or {}).get("source") or {}).get("column") or ""
            if tv and tv.get("status") in ("resolved", "multiple") and bottle and "bottle" in col.lower():
                # the atlas fills the injected fraction from the bottle's label: compare it only to say so
                n_dil = sum(1 for a in ts if a.status == "ok" and a.cmd[2] > 0 and a.cmd[1] > 0)
                if n_dil:
                    FIXED_READINGS.append({"investigation": inv, "test_id": day, "field": "injection.x_fuel_injected",
                                           "reading": "the atlas copies the Fuel Bottle label into the injected fraction;"
                                           f" {n_dil} of {len(ts)} test points of the day add nitrogen to the bottle before"
                                           " ignition, so the bottle fraction is not the injected one",
                                           "verdict": "our_reading_wrong"})
                continue
            if tv and tv.get("status") in ("resolved", "multiple") and bottle:
                xs = []
                for a in ts:
                    if a.status != "ok" or a.cmd[2] <= 0:
                        continue
                    b = CODES_MFCA3.get(code(a, 2)) or bottle
                    xs.append((a.test, injected_fraction(a, b), a))
                vals_t = (tv["set"]["values"] if tv.get("status") == "multiple" else [tv["value_si"]])
                kind_t = (tv["set"]["kind"] if tv.get("status") == "multiple" else "list")
                cond = "conditioned_on_ignition" in (tv.get("flags") or [])
                out.setdefault(f"{inv}|x_fuel_pairs", []).append(
                    {"day": day, "table": vals_t, "kind": kind_t, "commanded": sorted({r6(x) for _, x, _ in xs if isnum(x)})})
                if not xs:
                    continue
                COMPARED[(inv, "injection.x_fuel_injected")] += 1
                FRACTION_DAYS.append((inv, row, day, vals_t, kind_t, cond, xs, tv))
        out[f"{inv}|days"] = len(rows)
        out[f"{inv}|test_points_in_files"] = len(tests)
        DAY_READS[inv] = (rows, by_day, reads)
    fraction_records()
    oxygen_list_records(calc)
    sflame_pressure_records(pta_lines)
    return out


FRACTION_DAYS = []
DAY_READS = {}


def fraction_records():
    """Check 1 over every day: how far each commanded fraction lies from the nearest tabulated value."""
    dist = []
    for inv, row, day, vals_t, kind_t, cond, xs, tv in FRACTION_DAYS:
        for t, x, a in xs:
            if not isnum(x):
                continue
            if kind_t == "range":
                lo, hi = min(vals_t), max(vals_t)
                d = 0.0 if lo <= x <= hi else (x - hi if x > hi else x - lo)
            else:
                d = min((x - v for v in vals_t), key=abs)
            dist.append(abs(d))
    sg = robust_sigma(dist)
    thr = max(0.02, 5 * sg if isnum(sg) else 0)
    SUMMARY_EXTRA["day_fraction_distance"] = {"n": len(dist), "median": r6(med(dist)), "robust_sigma": r6(sg),
                                              "threshold": r6(thr)}
    for inv, row, day, vals_t, kind_t, cond, xs, tv in FRACTION_DAYS:
        good = [x for _, x, _ in xs if isnum(x)]
        problems = []
        if kind_t == "range":
            lo, hi = min(vals_t), max(vals_t)
            out_r = [(t, x) for t, x, _ in xs if isnum(x) and (x < lo - thr or x > hi + thr)]
            if out_r and not cond:
                problems.append(("commanded fractions outside the tabulated range", out_r))
        # a tabulated value no test of the day was commanded at (outcome-free in both list kinds)
        missing = [v for v in vals_t if kind_t == "list" and not any(abs(x - v) <= thr for x in good)]
        if missing:
            problems.append(("tabulated values no test of the day was commanded at", missing))
        if kind_t == "list" and not cond:
            extra = [(t, x) for t, x, _ in xs if isnum(x) and not any(abs(x - v) <= thr for v in vals_t)]
            if extra:
                problems.append(("commanded fractions that the table does not list", extra))
        if not problems:
            AGREE[(inv, "injection.x_fuel_injected")] += 1
            continue
        for what, items in problems:
            checks = [chk(1, CHECK_NAMES[1], f"over every day of the four day tables the distance of a commanded fraction"
                          f" to the nearest tabulated value has robust sigma {r6(sg)}; threshold {r6(thr)}")]
            if cond:
                checks.append(chk(5, CHECK_NAMES[5], "the table lists only the concentrations of flames that ignited;"
                                  " only a listed value with no commanded test can be judged without an outcome"))
            dil_tests = [t for t, x, a in xs if a.cmd[1] > 0]
            checks.append(chk(2, CHECK_NAMES[2], "the commands are read at the ignition reference; the scripts give"
                              " velocities, not fractions, for CLD Flame"))
            record(inv, row, "injection.x_fuel_injected",
                   table_side(tv, "injection.x_fuel_injected") | {"value": vals_t, "unit": "1", "set_kind": kind_t},
                   {"value": sorted({r6(x) for x in good}), "unit": "1", "file": "<test>_mfcSetpoints.csv of the day",
                    "channel": ["MFCA2Setpoint(V)", "MFCA3Setpoint(V)"],
                    "method": "fuel mole fraction of MFCA3 bottle (header gas code) plus MFCA2 diluent, commands at the"
                              " ignition reference, Hastings gas factors"},
                   {"kind": "set", "what": what, "items": [(i if not isinstance(i, tuple) else [i[0], r6(i[1])]) for i in items][:20],
                    "threshold": r6(thr), "method_error": r6(sg)},
                   "seen_once", checks, "unresolved", None,
                   f"{len(dil_tests)} of {len(xs)} tests of the day add diluent before ignition", [], test_id=day)


SUMMARY_EXTRA = {}


def oxygen_list_records(calc):
    """BRE lists the oxygen of the flames that ignited per day. Outcome-free direction: a listed value
    that no test point of the day started at (chamber calculator start mixture)."""
    inv = "PSI-20"
    if inv not in DAY_READS:
        return
    rows, by_day, reads = DAY_READS[inv]
    for day, row in rows.items():
        tv = val(row, "atmosphere.x_o2")
        if not tv or tv.get("status") not in ("resolved", "multiple"):
            continue
        vals_t = tv["set"]["values"] if tv.get("status") == "multiple" else [tv["value_si"]]
        code_day = alias(row, "test_day_yyddd") or day
        evs = [calc_event(calc, inv, t, reads.get(t)) for t in by_day.get(code_day, [])]
        xs = [e["start_frac"].get("o2") for e in evs if e and e["start_frac"]]
        if not xs:
            LIMITS.append(f"PSI-20 day {day}: no chamber calculator event for its test points")
            continue
        COMPARED[(inv, "atmosphere.x_o2")] += 1
        tol = 0.015   # the table says the concentrations are approximate; one and a half points of percent
        missing = [v for v in vals_t if not any(abs(x - v) <= tol for x in xs)]
        if not missing:
            AGREE[(inv, "atmosphere.x_o2")] += 1
            continue
        record(inv, row, "atmosphere.x_o2", table_side(tv, "atmosphere.x_o2") | {"value": vals_t, "unit": "1"},
               {"value": sorted({r6(x) for x in xs}), "unit": "1", "file": next((e["file"] for e in evs if e), None),
                "channel": "EventData Start Mixture", "method": "oxygen fraction of the start mixture of each test point of the day"},
               {"kind": "set", "listed_values_without_a_test": missing, "tolerance": tol},
               "seen_once",
               [chk(1, CHECK_NAMES[1], "a tolerance of 0.015 for the table's approximate values"),
                chk(5, CHECK_NAMES[5], "the table lists only flames that ignited; a listed value must still be one some"
                    " test point started at")],
               "unresolved", None, "", [], test_id=day)


def sflame_pressure_records(pta_lines):
    """s-Flame gives about 1 atm for every day; the ACME transducer before ignition, read with the line of
    the calculator campaigns, gives the start pressure of each test point."""
    inv = "PSI-23"
    if inv not in DAY_READS or not pta_lines:
        return
    a0 = med([v[0] for v in pta_lines.values()])
    b0 = med([v[1] for v in pta_lines.values()])
    rows, by_day, reads = DAY_READS[inv]
    allp = []
    for day, row in rows.items():
        code_day = alias(row, "test_day_yyddd") or day
        ps = [(t, (a0 + b0 * reads[t].pta3) * PSI_PA) for t in by_day.get(code_day, []) if isnum(reads[t].pta3)]
        allp += [p for _, p in ps]
        tv = val(row, "atmosphere.pressure")
        if not resolved(tv) or not ps:
            continue
        COMPARED[(inv, "atmosphere.pressure")] += 1
        far = [(t, p) for t, p in ps if abs(math.log(p / tv["value_si"])) > math.log(1.10)]
        if not far:
            AGREE[(inv, "atmosphere.pressure")] += 1
            continue
        record(inv, row, "atmosphere.pressure", table_side(tv, "atmosphere.pressure") | {"value": tv["raw_value"], "unit": "atm"},
               {"value": [[t, r6(p / ATM_PA)] for t, p in far][:20], "unit": "atm", "file": "<test>_data.csv of the day",
                "channel": "PTA3", "method": f"median of the 3 s before ignition, psia = {r6(a0)} + {r6(b0)} V (line of the"
                                             " PSI-10 and PSI-159 calculator campaigns)"},
               {"kind": "set", "tests_beyond_10_percent": len(far), "tests": len(ps)},
               "seen_once",
               [chk(1, CHECK_NAMES[1], "the same line reads PSI-10 and PSI-159 within their spread"),
                chk(5, CHECK_NAMES[5], "the table gives an approximate value for every day and says pressure rises with"
                    " vitiation: a nominal value against a reading", True)],
               "two_definitions", 5, "", [], test_id=day)
    SUMMARY_EXTRA["sflame_pta3_atm"] = {"n": len(allp), "median": r6(med(allp) / ATM_PA) if allp else None,
                                        "p5": r6(pct(allp, .05) / ATM_PA) if allp else None,
                                        "p95": r6(pct(allp, .95) / ATM_PA) if allp else None}


# =================================================================================================
# MDCA droplet investigations: PSI-39 CFI, PSI-69 FLEX, PSI-68 FLEX-2
# =================================================================================================
def mdca_log(text):
    """Header keys of a CIR test point log, up to its 'Note' line; the event lines after it are not parsed."""
    out = {}
    for line in text.split("\n"):
        if line.strip().startswith("Note"):
            break
        k, sep, v = line.partition(":")
        if sep:
            out[k.strip()] = v.strip()
    return out


def parse_ts(s):
    s = (s or "").strip().strip('"')
    for fmt_ in ("%Y-%m-%d %H:%M:%S.%f", "%Y-%m-%d %H:%M:%S"):
        try:
            return dt.datetime.strptime(s, fmt_)
        except ValueError:
            continue
    return None


def chamber_before(text, t_ign, window_s=10.0):
    """Chamber pressure (mean of PT16 and PT17, psia) and thermistor temperature (mean of TM1-4, C),
    medians over the window before ignition. Rows after ignition are not parsed."""
    lines = text.split("\n")
    hdr = [h.strip().strip('"') for h in lines[0].split(",")] if lines else []
    try:
        i16, i17 = hdr.index("FCU_MP_PT16(PSIA)"), hdr.index("FCU_MP_PT17(PSIA)")
        itm = [hdr.index(f"FCU_MP_TM{k}(Deg C)") for k in (1, 2, 3, 4)]
    except ValueError:
        return NAN, NAN, 0
    p, tm = [], []
    for line in lines[1:]:
        sp = line.split(",", max(itm) + 1)
        t = parse_ts(sp[0]) if sp and sp[0] else None
        if t is None:
            continue
        if t > t_ign:
            break
        if (t_ign - t).total_seconds() <= window_s:
            try:
                p.append((float(sp[i16]) + float(sp[i17])) / 2.0)
                tm.append(st.mean(float(sp[i]) for i in itm))
            except (ValueError, IndexError):
                continue
    return med(p), med(tm), len(p)


def mdca_read(nasa, inv, cir, pre):
    """Log header and the chamber before ignition of one CIR test point."""
    log_name = f"{pre}{cir}.txt"
    ch_name = f"{pre}{cir}_Chamber_Readings.txt"
    out = {"log_file": log_name if nasa.has(inv, log_name) else None,
           "chamber_file": ch_name if nasa.has(inv, ch_name) else None}
    if not out["log_file"]:
        return out
    try:
        log = mdca_log(nasa.text(inv, log_name))
    except Refused:
        return out
    out["log"] = log
    ign = parse_ts(log.get("AS Burn to Completion/Igniter On"))
    out["t_ign"] = ign
    if ign and out["chamber_file"]:
        try:
            p, tm, n = chamber_before(nasa.text(inv, ch_name), ign)
            out.update(p_psia=p, t_c=tm, n_rows=n, chamber_sha=nasa.sha(inv, ch_name))
        except Refused:
            pass
    out["log_sha"] = nasa.sha(inv, log_name)
    return out


MIN_FAMILY = 5   # below this many tests no common factor is removed


def ratio_family_records(inv, field, items, floor, method_text, sensor_desc, nominal=False, kind="ratio",
                         unit=None, verdict_if_nominal="two_definitions"):
    """items: (row, table_si, reading_si, sensor dict, others). Check 1 by the family's own spread."""
    if not items:
        return {}
    if kind == "ratio":
        vals_ = [math.log(t / s) for _, t, s, _, _ in items if t > 0 and s > 0]
    else:
        vals_ = [t - s for _, t, s, _, _ in items]
    c0, sg = med(vals_), robust_sigma(vals_)
    if len(vals_) < MIN_FAMILY:
        # too few tests for a common factor to mean anything: compare directly
        c0, sg = 0.0, NAN
    thr = max(floor, 5 * sg if isnum(sg) else 0)
    summ = {"n": len(vals_), "centre": r6(math.exp(c0) if kind == "ratio" else c0), "robust_sigma": r6(sg),
            "threshold": r6(thr), "kind": kind, "common_factor_used": len(vals_) >= MIN_FAMILY}

    def key(r):
        s0 = (r.get("source_rows") or [{}])[0]
        return (s0.get("file") or "", s0.get("sheet") or "", s0.get("row") or 0)
    order = sorted(((key(r), r, s, t) for r, t, s, _, _ in items), key=lambda x: x[0])
    for row, t, s, sensor, others in items:
        COMPARED[(inv, field)] += 1
        if kind == "ratio" and not (t > 0 and s > 0):
            continue
        d = (math.log(t / s) if kind == "ratio" else t - s) - c0
        if abs(d) <= thr:
            AGREE[(inv, field)] += 1
            continue
        checks = [chk(1, CHECK_NAMES[1], f"the same reading on all {len(vals_)} tests gives a common "
                      f"{'ratio' if kind == 'ratio' else 'difference'} table/reading of "
                      f"{r6(math.exp(c0) if kind == 'ratio' else c0)} (robust sigma {r6(sg)}); this test is {r6(d)} from it,"
                      f" threshold {r6(thr)}")]
        verdict, dec = "unresolved", None
        third = [o for o in others if o.get("agrees_with") in ("table", "reading")]
        for o in third:
            checks.append(chk(2, CHECK_NAMES[2], f"{o['source']} agrees with the {o['agrees_with']}", True))
            verdict, dec = ("table_value_wrong" if o["agrees_with"] == "reading" else "unresolved"), 2
            if o["agrees_with"] == "table":
                dec = None
        if dec is None:
            shp = shape_of(t, s, None)
            # the reading of the neighbouring rows of the same table: a value shifted by one row
            for nb_row, nb_s, nb_t in neighbours(row, order):
                dn = (math.log(t / nb_s) if kind == "ratio" and t > 0 and nb_s > 0 else t - nb_s) - c0
                same_cell = abs(nb_t - t) <= 1e-9 * max(abs(t), 1)
                if abs(dn) <= min(thr, abs(d) / 3) and not same_cell:
                    shp = f"equal to the reading of the neighbouring row (test {nb_row['test_id']}): shifted by one row"
                    break
            checks.append(chk(4, CHECK_NAMES[4], shp or "no factor of 2 or 10, not a neighbour's reading", bool(shp)))
            if shp:
                verdict, dec = "table_value_wrong", 4
        if dec is None and nominal:
            checks.append(chk(5, CHECK_NAMES[5], "the table gives a nominal round value and the file a reading of this"
                              " test; beyond the spread of the other tests, that does not explain it"))
        tv = val(row, field)
        status = "confirmed" if any(o.get("agrees_with") == "reading" for o in others) else (
            "unsettled" if any(o.get("agrees_with") == "table" for o in others) else "seen_once")
        record(inv, row, field, table_side(tv, field) | {"value": tv.get("raw_value"), "unit": tv.get("raw_unit")},
               sensor | {"method": method_text}, {"kind": kind, "table_over_reading" if kind == "ratio" else
                                                  "table_minus_reading": r6(t / s if kind == "ratio" else t - s),
                                                  "deviation": r6(d), "method_error": r6(sg), "threshold": r6(thr),
                                                  "unit_si": unit},
               status, checks, verdict, dec, "", others, test_id=row["test_id"])
    return summ


def neighbours(row, order):
    """The rows just above and below in the same table (consecutive source rows), with their readings."""
    for i, (k, r, s, t) in enumerate(order):
        if r is row:
            out = []
            for j in (i - 1, i + 1):
                if 0 <= j < len(order):
                    kj, rj, sj, tj = order[j]
                    if kj[0] == k[0] and kj[1] == k[1] and abs((kj[2] or 0) - (k[2] or 0)) == 1:
                        out.append((rj, sj, tj))
            return out
    return []


def cfi_records(nasa, atlas):
    inv, pre = "PSI-39", "PSI-39_Raw Data_"
    press, pw, ig_t, times = [], [], [], []
    n_read = 0
    for row in atlas.get(inv, []):
        cir = alias(row, "cir_test_point_upper")
        if not cir:
            continue
        m = mdca_read(nasa, inv, cir, pre)
        if not m.get("log"):
            continue
        n_read += 1
        log = m["log"]
        sens = {"file": m["log_file"], "sha256": m.get("log_sha")}
        tp = val(row, "atmosphere.pressure")
        if resolved(tp) and isnum(m.get("p_psia")):
            press.append((row, tp["value_si"], m["p_psia"] * PSI_PA,
                          {"value": r6(m["p_psia"]), "unit": "psia", "file": m["chamber_file"],
                           "channel": "mean of FCU_MP_PT16 and PT17", "sha256": m.get("chamber_sha")}, []))
        ipw = val(row, "extra_conditions.igniter_power")
        try:
            w = float(log.get("Igniter Wattage", "").replace(",", ""))
        except ValueError:
            w = NAN
        if resolved(ipw) and isnum(w) and w > 0:
            pw.append((row, ipw["value_si"], w, sens | {"value": w, "unit": "W", "channel": "Igniter Wattage"}, []))
        itm = val(row, "extra_conditions.igniter_time")
        try:
            ms = float(log.get("Igniter On Time (milliseconds)", "").replace(",", ""))
        except ValueError:
            ms = NAN
        if resolved(itm) and isnum(ms) and ms > 0:
            ig_t.append((row, itm["value_si"], ms / 1000.0, sens | {"value": ms, "unit": "ms",
                                                                    "channel": "Igniter On Time (milliseconds)"}, []))
        # date: the table's (local) date, the log's GMT start, the day of year in the CIR name
        td = val(row, "date")
        started = parse_ts(log.get("AS Started"))
        if resolved(td) and started:
            COMPARED[(inv, "date")] += 1
            d_tab = dt.date.fromisoformat(str(td["value_si"]))
            d_cir = dt.date(d_tab.year, 1, 1) + dt.timedelta(days=int(cir[1:4]) - 1)
            gap = (started.date() - d_tab).days
            if gap in (0, 1):
                AGREE[(inv, "date")] += 1
            else:
                checks = [chk(1, CHECK_NAMES[1], "a local (eastern) date may be one day before the GMT date of the log;"
                              " any other difference is counted"),
                          chk(2, CHECK_NAMES[2], f"the day of year in the CIR test-point name gives {d_cir.isoformat()}, "
                              + ("with the log" if d_cir == started.date() else "not with the log"), d_cir == started.date())]
                verdict, dec = ("table_value_wrong", 2) if d_cir == started.date() else ("unresolved", None)
                record(inv, row, "date", table_side(td, "date") | {"value": str(td["value_si"])},
                       sens | {"value": started.date().isoformat(), "channel": "AS Started (GMT)",
                               "method": "date of the test point log's start stamp"},
                       {"kind": "days", "table_minus_reading": -gap}, "confirmed" if dec else "seen_once",
                       checks, verdict, dec, "", [{"source": "CIR test point name", "value": cir,
                                                   "date": d_cir.isoformat()}], test_id=row["test_id"])
        # start time: local time against the log's GMT start (an offset of 4 or 5 hours)
        tt = val(row, "start_time")
        if resolved(tt) and started and str(tt.get("raw_unit") or "").lower().startswith("e"):
            try:
                h, mi, s = (int(x) for x in str(tt["value_si"]).split(":"))
                loc = h * 3600 + mi * 60 + s
                gmt = started.hour * 3600 + started.minute * 60 + started.second
                off = ((gmt - loc) % 86400) / 3600.0
                times.append((row, off, started, sens))
            except ValueError:
                pass
    summ = {"read": n_read}
    summ["pressure"] = ratio_family_records(
        inv, "atmosphere.pressure", press, math.log(1.05),
        "median over the 10 s before 'AS Burn to Completion/Igniter On' of the mean of PT16 and PT17", {},
        nominal=True, unit="Pa")
    summ["igniter_power"] = ratio_family_records(inv, "extra_conditions.igniter_power", pw, math.log(1.03),
                                                 "test point log header", {}, unit="W")
    summ["igniter_time"] = ratio_family_records(inv, "extra_conditions.igniter_time", ig_t, math.log(1.02),
                                                "test point log header", {}, unit="s")
    # start time: the offset must be a whole number of hours (4 or 5) to within a few minutes
    for row, off, started, sens in times:
        COMPARED[(inv, "start_time")] += 1
        near = min((4.0, 5.0), key=lambda h: abs(off - h))
        if abs(off - near) * 60 <= 10:
            AGREE[(inv, "start_time")] += 1
            continue
        tt = val(row, "start_time")
        shp = None
        for h in (4.0, 5.0):     # am/pm inverted: twelve hours away
            if abs(((off + 12) % 24) - h) * 60 <= 10:
                shp = "twelve hours away: the am/pm label is inverted"
        record(inv, row, "start_time", table_side(tt, "start_time") | {"value": tt.get("raw_value")},
               sens | {"value": started.strftime("%H:%M:%S"), "unit": "GMT", "channel": "AS Started",
                       "method": "start stamp of the CIR test point log"},
               {"kind": "hours", "gmt_minus_local": r6(off), "expected": "4 (EDT) or 5 (EST) hours"},
               "seen_once",
               [chk(1, CHECK_NAMES[1], "the other tests are 4 or 5 hours from the log to within 10 minutes"),
                chk(4, CHECK_NAMES[4], shp or "no simple shape", bool(shp))],
               "table_value_wrong" if shp else "unresolved", 4 if shp else None, "", [], test_id=row["test_id"])
    return summ


def flex_testsheet(text):
    """TEST PARAMETERS and AMBIENT ENVIRONMENT of a FLEX test sheet; reading stops at TEST CONDITIONS,
    so the fitted rate, the extinction diameter and the narrative are never parsed."""
    out = {}
    for line in text.split("\n"):
        if line.strip().upper().startswith("TEST CONDITIONS"):
            break
        k, sep, v = line.partition(":")
        if sep and k.strip() in ("Test Date", "Test Time", "CIR Test Number"):
            out[k.strip()] = v.strip()
            continue
        m = re.match(r"\s*(Pressure \(atm\)|O2 Mole Fraction|N2 Mole Fraction|CO2 Mole Fraction|He Mole Fraction|"
                     r"Temperature \(K\))\s*=\s*([-\d.]+)", line)
        if m:
            out[m.group(1)] = float(m.group(2))
    return out


def flex_records(nasa, atlas):
    inv, pre = "PSI-69", "PSI-69_Raw Data_"
    press, temp, sheet_items = [], [], defaultdict(list)
    n = 0
    for row in atlas.get(inv, []):
        fx = alias(row, "file_flex_number")
        if not fx:
            continue
        name = f"{pre}{fx}_TestSheet.txt"
        if not nasa.has(inv, name):
            continue
        try:
            ts = flex_testsheet(nasa.text(inv, name))
        except Refused:
            continue
        n += 1
        sens_ts = {"file": name, "sha256": nasa.sha(inv, name), "channel": "AMBIENT ENVIRONMENT block"}
        for field, key, scale in (("atmosphere.pressure", "Pressure (atm)", ATM_PA), ("atmosphere.x_o2", "O2 Mole Fraction", 1.0)):
            tv = val(row, field)
            if resolved(tv) and isnum(ts.get(key)):
                sheet_items[field].append((row, tv["value_si"], ts[key] * scale, sens_ts | {"value": ts[key], "unit": key}))
        for sp, key in (("nitrogen", "N2 Mole Fraction"), ("carbon_dioxide", "CO2 Mole Fraction"), ("helium", "He Mole Fraction")):
            d = (val(row, "atmosphere.diluents") or {}).get("items") or []
            for it in d:
                if it.get("species") in (sp, {"carbon_dioxide": "co2"}.get(sp)) and resolved(it.get("fraction")) and isnum(ts.get(key)):
                    sheet_items[f"atmosphere.diluents.{sp}"].append((row, it["fraction"]["value_si"], ts[key],
                                                                     sens_ts | {"value": ts[key], "unit": key}))
        cir = (ts.get("CIR Test Number") or "").replace(" ", "")
        if cir:
            nasa.allowed_tests[inv].add(cir)   # the CIR name that an allowed test's own sheet gives
            m = mdca_read(nasa, inv, cir, pre)
            tp = val(row, "atmosphere.pressure")
            if resolved(tp) and isnum(m.get("p_psia")):
                others = [{"source": "test sheet", "file": name, "value_atm": ts.get("Pressure (atm)"),
                           "agrees_with": None}]
                press.append((row, tp["value_si"], m["p_psia"] * PSI_PA,
                              {"value": r6(m["p_psia"]), "unit": "psia", "file": m["chamber_file"],
                               "channel": "mean of FCU_MP_PT16 and PT17", "sha256": m.get("chamber_sha")}, others))
            tt = val(row, "atmosphere.temperature")
            if resolved(tt) and isnum(m.get("t_c")):
                temp.append((row, tt["value_si"], m["t_c"] + 273.15,
                             {"value": r6(m["t_c"]), "unit": "C", "file": m["chamber_file"],
                              "channel": "mean of FCU_MP_TM1-TM4", "sha256": m.get("chamber_sha")}, []))
    summ = {"testsheets_read": n}
    # table against the test sheet: the sheet prints two decimals
    for field, items in sheet_items.items():
        for row, t, s, sens in items:
            COMPARED[(inv, field + ":testsheet")] += 1
            # both documents print rounded numbers: half a unit of each one's last digit
            tvv = val(row, field) if not field.startswith("atmosphere.diluents.") else None
            raw_t = tvv.get("raw_value") if tvv else None
            if field == "atmosphere.pressure":
                tol = (half_unit(raw_t) * MMHG_PA if raw_t is not None else 0.0) + 0.005 * ATM_PA + 1e-9
            else:
                tol = (half_unit(raw_t) if raw_t is not None else 0.005) + 0.005 + 1e-9
            if abs(t - s) <= tol:
                AGREE[(inv, field + ":testsheet")] += 1
                continue
            tv = val(row, field) if not field.startswith("atmosphere.diluents.") else None
            record(inv, row, field, (table_side(tv, field) if tv else {"field_path": field}) |
                   {"value": r6(t / ATM_PA) if field == "atmosphere.pressure" else r6(t),
                    "unit": "atm" if field == "atmosphere.pressure" else "1"},
                   sens | {"method": "the test sheet's ambient environment, two decimals"},
                   {"kind": "difference", "table_minus_reading": r6((t - s) / (ATM_PA if field == "atmosphere.pressure" else 1)),
                    "threshold": r6(tol / (ATM_PA if field == "atmosphere.pressure" else 1)), "unit": "atm" if field == "atmosphere.pressure" else "1"},
                   "seen_once",
                   [chk(1, CHECK_NAMES[1], "the sheet's last printed digit is the tolerance"),
                    chk(2, CHECK_NAMES[2], "the chamber readings are compared separately for pressure")],
                   "unresolved", None, "the test sheet is a NASA document of the same test; the project record also "
                   "flags this pair", [], test_id=row["test_id"])
    summ["pressure"] = ratio_family_records(inv, "atmosphere.pressure", press, math.log(1.02),
                                            "median over the 10 s before 'AS Burn to Completion/Igniter On' of the mean"
                                            " of PT16 and PT17 (the CIR test number from the test sheet)", {}, unit="Pa")
    summ["temperature"] = ratio_family_records(inv, "atmosphere.temperature", temp, 2.0,
                                               "median over the 10 s before ignition of the mean of the four chamber"
                                               " thermistors", {}, kind="diff", unit="K")
    return summ


def flex2_records(nasa, atlas):
    """FLEX-2 tabulates a planned matrix; every test point's chamber before ignition is compared with the
    matrix's sets."""
    inv = "PSI-68"
    rows = [r for r in atlas.get(inv, []) if r["level"] == "matrix_row"]
    if not rows:
        return {}
    row = rows[0]
    pset = (val(row, "atmosphere.pressure") or {}).get("set", {}).get("values") or []
    oset = (val(row, "atmosphere.x_o2") or {}).get("set", {}).get("values") or []
    tnom = val(row, "atmosphere.temperature")
    names = [n for n in nasa.names(inv) if re.fullmatch(r"(?:PSI-68_Raw Data_)?[A-Z]\d{3}D\d{2}_Chamber_Readings\.txt", n)]
    mixtures = []
    for n in nasa.names(inv):
        if re.fullmatch(r"(?:PSI-68_Raw Data_)?[A-Z]\d{3}R\d{2}_Mixture\.txt", n):
            try:
                lines = nasa.text(inv, n).split("\n")
                hdr = [h.strip() for h in lines[0].split(",")]
                for line in lines[1:]:
                    sp = [x.strip() for x in line.split(",")]
                    if len(sp) == len(hdr):
                        t = dt.datetime.strptime(sp[0], "%m/%d/%Y %H:%M")
                        mixtures.append((t, float(sp[hdr.index("O2")].rstrip("%")) / 100.0, n))
            except (Refused, ValueError):
                continue
    mixtures.sort()
    out_p, out_o, out_t, n_ok = [], [], [], 0
    for ch in sorted(names):
        cir = re.search(r"([A-Z]\d{3}D\d{2})_Chamber", ch).group(1)
        pre = ch[: ch.index(cir)]
        m = mdca_read(nasa, inv, cir, pre)
        if not m.get("t_ign") or not isnum(m.get("p_psia")):
            continue
        n_ok += 1
        p_atm = m["p_psia"] * PSI_PA / ATM_PA
        if pset and min(abs(math.log(p_atm * ATM_PA / v)) for v in pset) > math.log(1.05):
            out_p.append((cir, r6(p_atm)))
        earlier = [x for x in mixtures if x[0] <= m["t_ign"] and (m["t_ign"] - x[0]).days < 2]
        if earlier and oset:
            o2 = earlier[-1][1]
            if min(abs(o2 - v) for v in oset) > 0.01:
                out_o.append((cir, r6(o2)))
        if resolved(tnom) and isnum(m.get("t_c")) and abs(m["t_c"] + 273.15 - tnom["value_si"]) > 3.0:
            out_t.append((cir, r6(m["t_c"])))
    summ = {"test_points_read": n_ok, "mixture_readings": len(mixtures)}
    for field, bad, what, tol in (("atmosphere.pressure", out_p, "chamber pressure before ignition (atm)", "5 %"),
                                  ("atmosphere.x_o2", out_o, "oxygen of the latest chamber gas reading before the test"
                                   " (within two days)", "0.01"),
                                  ("atmosphere.temperature", out_t, "chamber thermistors before ignition (C)", "3 K")):
        COMPARED[(inv, field)] += 1
        summ[field] = {"outside": len(bad)}
        if not bad:
            AGREE[(inv, field)] += 1
            continue
        tv = val(row, field)
        record(inv, row, field, table_side(tv, field) | {"value": tv.get("raw_value"), "unit": tv.get("raw_unit"),
                                                         "scope": "all 8 matrix rows (identical condition cells)"},
               {"value": bad[:25], "file": "<test point>_Chamber_Readings.txt / <reading>_Mixture.txt",
                "channel": what, "method": f"test points whose value is farther than {tol} from every value of the set"},
               {"kind": "count", "test_points_outside_the_set": len(bad), "test_points": n_ok},
               "confirmed" if len(bad) >= 2 else "seen_once",
               [chk(1, CHECK_NAMES[1], f"{n_ok - len(bad)} of {n_ok} test points fall inside the set with the same reading"),
                chk(2, CHECK_NAMES[2], "the table is the planned matrix (one row per fuel, identical condition sets);"
                    " the files record the executed test points", True)],
               "table_records_plan_not_execution", 2, "", [], test_id="matrix (all rows)", group_id=row["group_id"])
    return summ


# =================================================================================================
# Saffire (PSI-98, 99, 100): oxygen and air flow before ignition, from the sensor workbooks
# =================================================================================================
SAFFIRE_BOOKS = {"PSI-98": "PSI-98_Raw Data_Saffire-I Sample {n} Sensor Data Complete TM graphs.xlsx",
                 "PSI-99": "PSI-99_Raw Data_Saffire Sensor Data Sample {n}.xlsx",
                 "PSI-100": "PSI-100_Raw Data_Saffire III Sample {n} Sensors.xlsx"}


def _saffire2_readme():
    """Commanded air flow per Saffire-II sample from the hand transcription of the Readme's Table 1."""
    p = REPO / "tools/wsp/atlas/transcribed/psi99_saffire2_readme_table1.csv"
    out = {}
    if p.exists():
        lines = [ln for ln in p.read_text(encoding="utf-8").splitlines() if ln and not ln.startswith("#")]
        for r in csv.DictReader(lines):
            try:
                out[r["sample"].lstrip("S")] = float(r["air_flow_cm_s"])
            except (KeyError, ValueError):
                continue
    return out


SAFFIRE2_README = _saffire2_readme()


def saffire_before(path):
    """O2 (%), anemometers (cm/s) and pressure (psia) over the 60 s before the igniter power is first
    enabled; the rows after that are not read. Radiometer and temperature columns are not read."""
    import openpyxl
    wb = openpyxl.load_workbook(path, read_only=True, data_only=True)
    ws = wb["Calibrated Data"] if "Calibrated Data" in wb.sheetnames else wb.worksheets[0]
    it = ws.iter_rows(values_only=True)
    hdr = [str(h).strip() if h is not None else "" for h in next(it)]
    next(it)
    col = {k: hdr.index(k) for k in ("Elapsed Time (s)", "O2", "Anemometer 1", "Anemometer 2", "Ambient Pressure",
                                     "Igniter Power Enable", "Average Fan Speed") if k in hdr}
    buf = []
    t_ign = NAN
    for r in it:
        try:
            t = float(r[col["Elapsed Time (s)"]])
            ip = float(r[col["Igniter Power Enable"]] or 0)
        except (TypeError, ValueError, KeyError, IndexError):
            continue
        if ip > 0:
            t_ign = t
            break
        buf.append((t, r[col["O2"]] if "O2" in col else None,
                    r[col["Anemometer 1"]] if "Anemometer 1" in col else None,
                    r[col["Anemometer 2"]] if "Anemometer 2" in col else None,
                    r[col["Ambient Pressure"]] if "Ambient Pressure" in col else None,
                    r[col["Average Fan Speed"]] if "Average Fan Speed" in col else None))
    wb.close()
    if not isnum(t_ign):
        return None
    w = [b for b in buf if t_ign - 60 <= b[0] < t_ign]
    num = lambda xs: [float(x) for x in xs if isinstance(x, (int, float))]
    an = num([b[2] for b in w]) + num([b[3] for b in w])
    return {"t_ign_s": t_ign, "o2_pct": med(num([b[1] for b in w])), "anemometer_cm_s": med(an),
            "p_psia": med(num([b[4] for b in w])), "fan_rpm": med(num([b[5] for b in w])), "n": len(w)}


def saffire_records(nasa, atlas):
    summ = {}
    o2_items, fl_items = [], []
    for inv, pattern in SAFFIRE_BOOKS.items():
        for row in atlas.get(inv, []):
            m = re.search(r"(\d+)$", row["test_id"])
            if not m:
                continue
            name = pattern.format(n=m.group(1))
            if not nasa.has(inv, name):
                LIMITS.append(f"{inv} {row['test_id']}: no sensor workbook {name}")
                continue
            try:
                r = saffire_before(nasa.path(inv, name))
            except Refused:
                continue
            if not r:
                LIMITS.append(f"{inv} {row['test_id']}: the workbook never enables the igniter; nothing read")
                continue
            summ[f"{inv}:{row['test_id']}"] = {"n_rows": r["n"]}
            sens = {"file": name, "sha256": nasa.sha(inv, name)}
            tv = val(row, "atmosphere.x_o2")
            if resolved(tv) and isnum(r["o2_pct"]):
                o2_items.append((inv, row, tv["value_si"], r["o2_pct"] / 100.0,
                                 sens | {"value": r6(r["o2_pct"]), "unit": "%", "channel": "O2"}, []))
            fv = val(row, "flow.speed")
            if resolved(fv) and isnum(r["anemometer_cm_s"]):
                others = []
                rd = SAFFIRE2_README.get(m.group(1)) if inv == "PSI-99" else None
                if isnum(rd):
                    a_ = r["anemometer_cm_s"]
                    agrees = ("reading" if abs(math.log(rd / a_)) <= math.log(1.15) else
                              "table" if abs(rd - fv["value_si"] * 100) < 1e-6 else None)
                    others.append({"source": "Readme Table 1 (hand transcription, tools/wsp/atlas/transcribed/"
                                   "psi99_saffire2_readme_table1.csv)", "value_cm_s": rd, "agrees_with": agrees})
                fl_items.append((inv, row, fv["value_si"], r["anemometer_cm_s"] / 100.0,
                                 sens | {"value": r6(r["anemometer_cm_s"]), "unit": "cm/s",
                                         "channel": "Anemometer 1 and 2", "fan_rpm": r6(r["fan_rpm"])}, others))
    # the fan speed is the commanded side of the flow: a sample whose fan runs like the other samples of
    # the same tabulated flow was run at that flow (the anemometer is then the odd one), and one whose fan
    # runs faster or slower was not
    by_flow = defaultdict(list)
    for inv, row, t, s_, sens, others in fl_items:
        by_flow[(inv, round(t, 4))].append(sens.get("fan_rpm"))
    for inv, row, t, s_, sens, others in fl_items:
        mates = [f for f in by_flow[(inv, round(t, 4))] if isnum(f)]
        f0 = sens.get("fan_rpm")
        if isnum(f0) and len(mates) >= 3:
            rest = list(mates)
            rest.remove(f0)
            m0 = med(rest)
            same = abs(f0 / m0 - 1) <= 0.03
            # a faster fan with a faster anemometer (or slower with slower): the flow was not the table's
            with_reading = (not same) and ((f0 > m0) == (s_ > t))
            others.append({"source": f"fan speed before ignition ({r6(f0)} rpm) against the other samples tabulated at the"
                           f" same flow ({r6(m0)} rpm)",
                           "agrees_with": "table" if same else ("reading" if with_reading else None)})
    for field, items, kind, floor, what in (
            ("atmosphere.x_o2", o2_items, "diff", 0.005, "median O2 over the 60 s before the igniter is first enabled"),
            ("flow.speed", fl_items, "ratio", math.log(1.15), "median of both anemometers over the same 60 s")):
        by_inv = defaultdict(list)
        for inv, row, t, s, sens, others in items:
            by_inv[inv].append((row, t, s, sens, others))
        for inv, its in by_inv.items():
            nominal = field == "flow.speed"
            summ[f"{inv}|{field}"] = ratio_family_records(inv, field, its, floor, what, {}, nominal=nominal, kind=kind)
    return summ


# =================================================================================================
# Smaller tables: SAME-R (PSI-101), SLICE (PSI-106), DAFT (PSI-47), SPICE (PSI-107), BASS-II (PSI-25)
# =================================================================================================
def same_r_records(nasa, atlas):
    """SAME-R: the table's date and resistance ratio against the housekeeping files. The detector,
    particle and smoke channels are never parsed."""
    inv = "PSI-101"
    tests = defaultdict(lambda: {"days": Counter(), "r": [], "heat": []})
    for n in sorted(nasa.names(inv)):
        if not re.match(r"PSI-101_Raw Data_HKPData_gmt\d+_\d+.*\.csv$", n):
            continue
        try:
            lines = nasa.text(inv, n).split("\n")
        except Refused:
            continue
        hdr = lines[0].split(",")
        try:
            i_ts, i_tn = hdr.index("TimeString"), hdr.index("TestNumber")
            i_h, i_r = hdr.index("HeatingInProgress"), hdr.index("SampleResistance(Ohms)")
        except ValueError:
            continue
        for line in lines[1:]:
            sp = line.split(",", i_r + 1)
            if len(sp) <= i_r:
                continue
            try:
                tn = int(float(sp[i_tn]))
                y, doy = sp[i_ts].split("/")[0:2]
                heat = int(float(sp[i_h]))
                res = float(sp[i_r])
            except (ValueError, IndexError):
                continue
            tests[tn]["days"][(int(y), int(doy))] += 1
            tests[tn]["r"].append((heat, res))
            tests[tn]["file"] = n
            if "first_hour" not in tests[tn]:
                try:
                    tests[tn]["first_hour"] = int(sp[i_ts].split("/")[2].split(":")[0])
                except (ValueError, IndexError):
                    pass
    summ = {"test_numbers_in_housekeeping": len(tests)}
    ratio_items = []
    for row in atlas.get(inv, []):
        if row["level"] != "test":
            continue
        try:
            tn = int(re.match(r"(\d+)", row["test_id"]).group(1))
        except (AttributeError, ValueError):
            continue
        if tn not in tests or "#" in row["test_id"]:
            continue
        h = tests[tn]
        td = val(row, "date")
        (y, doy), _ = h["days"].most_common(1)[0]
        d_hkp = dt.date(y, 1, 1) + dt.timedelta(days=doy - 1)
        if resolved(td):
            COMPARED[(inv, "date")] += 1
            d_tab = dt.date.fromisoformat(str(td["value_si"]))
            gap = (d_tab - d_hkp).days
            if gap == 0:
                AGREE[(inv, "date")] += 1
            else:
                hour = h.get("first_hour")
                local = gap == -1 and isinstance(hour, int) and hour < 8
                record(inv, row, "date", table_side(td, "date") | {"value": str(td["value_si"])},
                       {"value": d_hkp.isoformat(), "file": h.get("file"), "channel": "TimeString (year/day of year/time)",
                        "method": "GMT day on which the housekeeping file records this test number most often",
                        "gmt_hour_of_first_record": hour},
                       {"kind": "days", "table_minus_reading": gap}, "seen_once",
                       [chk(1, CHECK_NAMES[1], "the same test number read in every housekeeping file"),
                        chk(5, CHECK_NAMES[5], (f"the run starts at {hour} h GMT, the evening before in United States time:"
                                                " the table's date is the local date and the file's the GMT date")
                            if local else "the difference is not explained by the time zone", local)],
                       "two_definitions" if local else "unresolved", 5 if local else None, "", [],
                       test_id=row["test_id"])
        # resistance ratio: during heating against the samples just before heating
        rr = h["r"]
        first = next((i for i, (ht, _) in enumerate(rr) if ht == 1), None)
        tv = val(row, "extra_conditions.heater_resistance_ratio")
        if first and first >= 5 and resolved(tv):
            r0 = med([r for ht, r in rr[max(0, first - 10):first] if ht == 0])
            on = [r for ht, r in rr[first:] if ht == 1]
            rt = med(on[len(on) // 4:]) if len(on) >= 8 else NAN
            if isnum(r0) and r0 > 0 and isnum(rt):
                ratio_items.append((row, tv["value_si"], rt / r0,
                                    {"value": r6(rt / r0), "unit": "1", "file": h.get("file"),
                                     "channel": "SampleResistance(Ohms) with HeatingInProgress"}, []))
    summ["resistance_ratio"] = ratio_family_records(
        inv, "extra_conditions.heater_resistance_ratio", ratio_items, math.log(1.03),
        "median resistance over the last three quarters of the heating, over the median of the ten samples before"
        " heating", {}, unit="1")
    return summ


def names_days(nasa, inv, patterns):
    days = defaultdict(set)
    for n in nasa.names(inv):
        for p in patterns:
            m = re.search(p, n)
            if m:
                days[int(m.group("d"))].add(n)
    return days


def presence_records(inv, rows, file_days, what, test_key=lambda r: r):
    """Table days against the days that file names carry."""
    tab_days = {}
    for row in rows:
        td = val(row, "date")
        if resolved(td):
            d = dt.date.fromisoformat(str(td["value_si"]))
            tab_days[(d - dt.date(d.year, 1, 1)).days + 1] = row
    for doy, row in tab_days.items():
        COMPARED[(inv, "date")] += 1
        if doy in file_days:
            AGREE[(inv, "date")] += 1
            continue
        near = sorted(file_days, key=lambda d: abs(d - doy))[:3]
        record(inv, row, "date", table_side(val(row, "date"), "date") | {"value": str(val(row, "date")["value_si"])},
               {"value": None, "file": f"{inv} listing", "channel": what, "method": "day of year in the file names",
                "nearest_days_in_names": near},
               {"kind": "presence", "files_of_the_day": 0}, "seen_once",
               [chk(1, CHECK_NAMES[1], f"{len(set(tab_days) & set(file_days))} of {len(tab_days)} table days are named"
                    " in the files")], "unresolved", None, "no file name carries the table's day", [],
               test_id=row["test_id"])
    extra = sorted(set(file_days) - set(tab_days))
    if extra:
        COMPARED[(inv, "date")] += 1
        record(inv, None, "date", {"field_path": "date", "value": None, "column": "(no row)",
                                   "file": rows[0]["source_rows"][0]["file"] if rows else None},
               {"value": extra, "file": f"{inv} listing", "channel": what,
                "method": "days of year in file names that no table row gives",
                "examples": {d: sorted(file_days[d])[:2] for d in extra[:6]}},
               {"kind": "presence", "days_without_a_row": len(extra)}, "seen_once",
               [chk(1, CHECK_NAMES[1], "file names of every kind that carry a day of the year")],
               "unresolved", None, "days of files that the table does not list (setup, calibration or test days)",
               [], test_id=f"{inv}:file days", group_id=f"{inv}:file days")


def slice_records(nasa, atlas):
    inv = "PSI-106"
    rows = [r for r in atlas.get(inv, []) if r["level"] == "test_day"]
    days = names_days(nasa, inv, [r"GMT2012_(?P<d>\d{3})_", r"pic_info_(?P<d>\d{3})", r"^\d+C\d?H\d+_(?:[\d.]+_)?(?P<d>\d{3})[_.]",
                                  r"^C\d?H\d+_(?P<d>\d{3})[_.]"])
    presence_records(inv, rows, days, "GMT2012_<day>, pic_info_<day>, <fuel>_<day> names")
    # fuel named in the file names of a day against the table's fuel of that day
    fuel_names = defaultdict(set)
    for n in nasa.names(inv):
        m = re.match(r"^(?P<pct>\d+)?(?P<f>C\d?H\d+)_(?:[\d.]+_)?(?P<d>\d{3})[_.]", n)
        if m:
            fuel_names[int(m.group("d"))].add(((m.group("pct") or "100"), m.group("f")))
    words = {"CH4": "methane", "C2H4": "ethylene", "C3H8": "propane"}
    for row in rows:
        td = val(row, "date")
        if not resolved(td):
            continue
        d = dt.date.fromisoformat(str(td["value_si"]))
        doy = (d - dt.date(d.year, 1, 1)).days + 1
        if doy not in fuel_names:
            continue
        COMPARED[(inv, "fuel.name")] += 1
        tab = str(val(row, "fuel.name.value_si") or "")
        named = {words.get(f, f) for _, f in fuel_names[doy]}
        if any(w in tab for w in named):
            AGREE[(inv, "fuel.name")] += 1
            continue
        record(inv, row, "fuel.name", table_side(val(row, "fuel.name"), "fuel.name") | {"value": tab},
               {"value": sorted(named), "file": f"{inv} listing", "channel": "fuel prefix of file names of the day",
                "method": "<pct><formula>_<day> names"},
               {"kind": "text"}, "seen_once", [chk(1, CHECK_NAMES[1], "file-name fuels of the other days")],
               "unresolved", None, "", [], test_id=row["test_id"])


def daft_records(nasa, atlas):
    inv = "PSI-47"
    rows = [r for r in atlas.get(inv, []) if r["level"] == "test_day"]
    dates = defaultdict(set)
    for n in nasa.names(inv):
        if re.match(r"PSI-47_Raw Data_[DP]Trk\d+\.txt$", n):
            try:
                text = nasa.text(inv, n)
            except Refused:
                continue
            m = re.search(r"GMT:\s*(\d{2})([A-Za-z]{3})(\d{2})", text)
            if m:
                d = dt.datetime.strptime(m.group(1) + m.group(2) + m.group(3), "%d%b%y").date()
                dates[(d - dt.date(d.year, 1, 1)).days + 1].add(n)
    presence_records(inv, rows, dates, "GMT line of the DustTrak and P-Trak files")


def spice_records(nasa, atlas):
    inv = "PSI-107"
    rows_by_day = {}
    for r in atlas.get(inv, []):
        td = val(r, "date")
        if resolved(td):
            rows_by_day.setdefault(str(td["value_si"]), r)
    days = names_days(nasa, inv, [r"GMT(?P<d>\d{3})(?!\d)", r"GMT2009_(?P<d>\d{3})", r"_Raw Data_(?P<d>\d{3})_SPICE",
                                  r"GMT (?P<d>\d{3}) "])
    presence_records(inv, list(rows_by_day.values()), days, "GMT<day> in picture books, contact sheets and lists")


def bass_records(nasa, atlas):
    inv = "PSI-25"
    for row in atlas.get(inv, []):
        tn, sn = alias(row, "test_number"), alias(row, "sample_number")
        if not (tn and sn):
            continue
        pat = re.compile(rf"_GMT(\d{{4}})_(\d{{3}})_BASS_S{re.escape(sn)}_TP{re.escape(tn)}[a-z]?_ProofSheet\.pdf$")
        hits = [(int(m.group(1)), int(m.group(2)), n) for n in nasa.names(inv) for m in [pat.search(n)] if m]
        td = val(row, "date")
        if not hits or not resolved(td):
            continue
        COMPARED[(inv, "date")] += 1
        d_tab = dt.date.fromisoformat(str(td["value_si"]))
        y, doy, n = hits[0]
        d_file = dt.date(y, 1, 1) + dt.timedelta(days=doy - 1)
        gap = (d_tab - d_file).days
        if gap == 0:
            AGREE[(inv, "date")] += 1
            continue
        src = (td.get("source") or {}).get("file") or ""
        if "records" in src and (d_tab.month, d_tab.day) == (d_file.month, d_file.day):
            # the atlas date of BASS-II is the project record's session id, not a NASA table cell
            FIXED_READINGS.append({"investigation": inv, "test_id": row["test_id"], "field": "date",
                                   "reading": f"the project record's session is dated {d_tab.isoformat()}; the proof sheet"
                                   f" name gives {d_file.isoformat()}: same day and month, the year differs. The"
                                   " record's session id is ours to fix, not a NASA table value",
                                   "verdict": "our_reading_wrong", "file": n})
            continue
        record(inv, row, "date", table_side(td, "date") | {"value": str(td["value_si"])},
               {"value": d_file.isoformat(), "file": n, "channel": "GMT<year>_<day> of the proof sheet name",
                "method": "file name only; the proof sheet is not opened"},
               {"kind": "days", "table_minus_reading": gap}, "seen_once",
               [chk(1, CHECK_NAMES[1], "the other tests' proof sheets carry the record's day"),
                chk(5, CHECK_NAMES[5], "the record's date is its session's day; a session past midnight GMT gives the next"
                    " day in the name" if abs(gap) == 1 else "more than a day", abs(gap) == 1)],
               "two_definitions" if abs(gap) == 1 else "unresolved", 5 if abs(gap) == 1 else None, "", [],
               test_id=row["test_id"])


# =================================================================================================
# Each table against itself
# =================================================================================================
def half_unit(raw):
    """Half a unit of the last printed digit of a number as the source writes it (0.21 -> 0.005). A numeric
    zero from a spreadsheet is an absent component, not a rounded one."""
    if isinstance(raw, (int, float)) and not isinstance(raw, bool) and raw == 0:
        return 0.0
    s = str(raw).strip().rstrip("%").strip()
    m = re.fullmatch(r"[-+]?\d*\.?(\d*)", s)
    if not m or not s:
        return 0.005
    dec = len(m.group(1)) if "." in s else 0
    h = 0.5 * 10 ** (-dec)
    return h / 100.0 if str(raw).strip().endswith("%") else h


NONNEG = ["atmosphere.pressure", "atmosphere.x_o2", "atmosphere.temperature", "flow.speed",
          "injection.fuel_mass_flow", "injection.oxidizer_mass_flow", "injection.inert_mass_flow",
          "injection.total_mass_flow", "injection.x_fuel_injected", "oxidizer_stream.x_o2",
          "atmosphere.x_fuel_ambient", "extra_conditions.fuel_volume_flow_std"]
FRACTIONS = ["atmosphere.x_o2", "injection.x_fuel_injected", "oxidizer_stream.x_o2", "atmosphere.x_fuel_ambient"]


def self_records(atlas):
    for inv, rows in atlas.items():
        # repeated rows: one atlas row built from several rows of the same table file
        for row in rows:
            files = Counter(s["file"] for s in row.get("source_rows") or [] if s.get("row") is not None)
            for f, n in files.items():
                if n < 2 or "TestSheet" in f:
                    continue
                if row["level"] in ("test_day", "matrix_row"):
                    # a day row is built from its continuation rows, one per listed value; more rows
                    # than listed values is a repeat
                    xf = val(row, "injection.x_fuel_injected") or {}
                    n_vals = len((xf.get("set") or {}).get("values") or []) if xf.get("status") == "multiple" else 1
                    if n <= n_vals:
                        continue
                COMPARED[(inv, "row")] += 1
                rr = [s["row"] for s in row["source_rows"] if s["file"] == f]
                # a row with its own test id written twice is a repeat whatever its other cells hold; rows
                # without an id (positional) that agree on their conditions may be two settings of one flame
                named = not re.fullmatch(r"d\d{3}-row\d+", str(row["test_id"]))
                record(inv, row, "row", {"field_path": "row", "value": rr, "file": f, "column": None},
                       {"value": None, "file": f, "channel": "the table itself",
                        "method": "rows that agree in every condition cell and that the atlas emitted once"},
                       {"kind": "count", "rows": n}, "confirmed" if named else "seen_once",
                       [chk(1, CHECK_NAMES[1], "rows compared on their condition cells (atlas source rows); outcome cells"
                            " are not read"),
                        chk(4, CHECK_NAMES[4], "the same test id on rows equal in their conditions: one test written twice"
                            if named else "rows without a test id, equal in their conditions: a repeat, or one setting held"
                            " twice in a flame, which the condition cells cannot tell apart", named)],
                       "table_value_wrong" if named else "unresolved", 4 if named else None,
                       "a repeated row: the test is counted once" if named else "", [], test_id=row["test_id"])
        for row in rows:
            # impossible values
            for fp in NONNEG:
                v = val(row, fp)
                if not resolved(v) or not isnum(v["value_si"]):
                    continue
                bad = v["value_si"] < 0 or (fp in FRACTIONS and v["value_si"] > 1.0 + 1e-9)
                COMPARED[(inv, fp + ":self")] += 1
                if not bad:
                    AGREE[(inv, fp + ":self")] += 1
                    continue
                record(inv, row, fp, table_side(v, fp) | {"value": v["raw_value"], "unit": v["raw_unit"]},
                       {"value": None, "file": table_side(v, fp)["file"], "channel": "the table itself",
                        "method": "a non-negative quantity, a fraction between 0 and 1"},
                       {"kind": "domain"}, "confirmed",
                       [chk(3, CHECK_NAMES[3], "a negative flow or speed, or a fraction above one, cannot be", True)],
                       "table_value_wrong", 3, "", [], test_id=row["test_id"])
            # fractions of the atmosphere
            x = val(row, "atmosphere.x_o2")
            dil = val(row, "atmosphere.diluents") or {}
            items = dil.get("items") or []
            if resolved(x) and items and all(resolved(i.get("fraction")) for i in items):
                tot = x["value_si"] + sum(i["fraction"]["value_si"] for i in items)
                bal = any("balance_gas" in (i["fraction"].get("flags") or []) for i in items)
                if bal:
                    continue
                # only a complete set can be summed: it must name the balance gas of the chamber
                if not any(i.get("species") in ("nitrogen", "helium", "argon") for i in items):
                    continue
                # each printed fraction carries half a unit of its last digit
                tol = sum(half_unit(v.get("raw_value")) for v in [x] + [i["fraction"] for i in items]) + 1e-9
                COMPARED[(inv, "atmosphere.fractions_sum")] += 1
                if abs(tot - 1.0) <= tol:
                    AGREE[(inv, "atmosphere.fractions_sum")] += 1
                    continue
                record(inv, row, "atmosphere.fractions_sum",
                       {"field_path": "atmosphere", "value": r6(tot), "unit": "1",
                        "file": table_side(x, "x")["file"], "column": [table_side(x, "x")["column"]] +
                        [((i["fraction"].get("source") or {}).get("column")) for i in items]},
                       {"value": 1.0, "file": None, "channel": "the table itself", "method": "sum of the mole fractions"},
                       {"kind": "difference", "table_minus_reading": r6(tot - 1.0), "threshold": r6(tol),
                        "threshold_rule": "half a unit of the last printed digit of each fraction"}, "confirmed",
                       [chk(3, CHECK_NAMES[3], "the mole fractions of a gas add to one", True)],
                       "table_value_wrong", 3, "which fraction is wrong is not known", [], test_id=row["test_id"])
        # mixed units within a column
        units = defaultdict(set)
        for row in rows:
            for fp in NONNEG + ["geometry.sizes"]:
                v = val(row, fp)
                if fp == "geometry.sizes":
                    for s in v or []:
                        vv = s.get("value") or {}
                        if vv.get("raw_unit"):
                            src = vv.get("source") or {}
                            units[(src.get("file"), src.get("column"), s.get("dimension"))].add(vv["raw_unit"])
                    continue
                if isinstance(v, dict) and v.get("raw_unit") and v.get("status") in ("resolved", "multiple"):
                    src = v.get("source") or {}
                    units[(src.get("file"), src.get("column"), fp)].add(v["raw_unit"])
        for (f, c, fp), us in units.items():
            COMPARED[(inv, "units")] += 1
            if len(us) <= 1:
                AGREE[(inv, "units")] += 1
                continue
            record(inv, None, fp, {"field_path": fp, "value": sorted(us), "file": f, "column": c},
                   {"value": None, "file": f, "channel": "the table itself", "method": "units the cells of one column give"},
                   {"kind": "units", "units": sorted(us)}, "confirmed",
                   [chk(5, CHECK_NAMES[5], "one column, cells in different units; whether each cell's unit is right is"
                        " not settled by the table itself")],
                   "unresolved", None, "", [], test_id=f"{inv}:{c}", group_id=f"{inv}:column")


# =================================================================================================
# Main
# =================================================================================================
NOT_COMPARED = {
    "PSI-26": "no row in tests.jsonl (the atlas holds none for BASS; its table is not a test table of the atlas)",
    "PSI-102": "a planned matrix whose rows name no test; its run data are legacy .xls workbooks, for which no reader"
               " is installed",
}


def main():
    t0 = dt.datetime.now(dt.timezone.utc)
    nasa = NasaFiles()
    atlas = load_atlas()
    set_allowed(nasa, atlas)
    calc, sources = load_calculators(nasa)
    per, fams, status, read = acme_per_test(nasa, atlas, calc, sources)
    summary = {"acme_read_status": {f"{k[0]}:{k[1]}": v for k, v in status.items()}}
    summary["acme_flow_families"] = acme_flow_records(per, fams)
    offsets = readback_offsets(per)
    summary["readback_minus_command_V"] = {f"{k[0]}:MFCA{k[1] + 1}": v for k, v in offsets.items()}
    annotate_unsettled(per, offsets)
    factor_records(fams)
    summary["acme_conditions"] = acme_conditions_records(per)
    # how often settled readbacks before ignition confirm their command within RB_TOL (check 1 of the readbacks)
    conf = Counter()
    for inv, row, ctx in per:
        a = ctx["a"]
        for k in range(3):
            if a.rb_state[k] == "settled":
                c, r = ctx["cmd"][k], ctx["rb"][k]
                conf[(inv, abs(r - c) <= RB_TOL_REL * abs(c) + RB_TOL_V / 5.0 * fs(a, k))] += 1
    summary["readback_confirms_command"] = {f"{k[0]}:{k[1]}": v for k, v in conf.items()}
    summary["acme_day_tables"] = {k: v for k, v in acme_day_records(nasa, atlas, calc, PTA_LINES).items()
                                  if not k.endswith("x_fuel_pairs")}
    summary["cfi"] = cfi_records(nasa, atlas)
    summary["flex"] = flex_records(nasa, atlas)
    summary["flex2"] = flex2_records(nasa, atlas)
    summary["saffire"] = saffire_records(nasa, atlas)
    summary["same_r"] = same_r_records(nasa, atlas)
    slice_records(nasa, atlas)
    daft_records(nasa, atlas)
    spice_records(nasa, atlas)
    bass_records(nasa, atlas)
    self_records(atlas)
    summary["extra"] = SUMMARY_EXTRA
    # ---- write ------------------------------------------------------------------------------------
    RECORDS.sort(key=lambda r: (int(r["investigation"].split("-")[1]), str(r["test_id"]), r["field"]))
    OUT.parent.mkdir(parents=True, exist_ok=True)
    with OUT.open("w", encoding="utf-8", newline="\n") as f:
        for r in RECORDS:
            f.write(json.dumps(r, ensure_ascii=False, sort_keys=False, default=str) + "\n")
    est = [r for r in RECORDS if r["verdict"] != "unresolved"]
    counts = {
        "schema": SCHEMA, "written_utc": t0.isoformat(timespec="seconds"),
        "n_discrepancies": len(RECORDS), "n_cause_established": len(est),
        "by_verdict": dict(Counter(r["verdict"] for r in RECORDS)),
        "by_status": dict(Counter(r["status"] for r in RECORDS)),
        "by_investigation": dict(sorted(Counter(r["investigation"] for r in RECORDS).items(),
                                        key=lambda kv: int(kv[0].split("-")[1]))),
        "by_field": dict(Counter(r["field"] for r in RECORDS).most_common()),
        "by_investigation_verdict": {f"{k[0]}|{k[1]}": v for k, v in sorted(
            Counter((r["investigation"], r["verdict"]) for r in RECORDS).items(), key=lambda kv: (int(kv[0][0].split("-")[1]), kv[0][1]))},
        "decided_by_check": dict(Counter(str(r["decided_by_check"]) for r in RECORDS)),
        "comparisons": {f"{k[0]}|{k[1]}": v for k, v in sorted(COMPARED.items())},
        "agreements": {f"{k[0]}|{k[1]}": v for k, v in sorted(AGREE.items())},
        "readings_fixed_our_reading_wrong": FIXED_READINGS,
        "files_opened_by_investigation": dict(nasa.opened), "hash_failures": nasa.hash_failures,
        "files_refused_by_the_guard": dict(nasa.refused),
        "limits": LIMITS, "not_compared": NOT_COMPARED,
        "investigations_with_rows": sorted(atlas, key=lambda s: int(s.split("-")[1])),
        "method": summary,
    }
    SUMMARY.write_text(json.dumps(counts, indent=1, default=str), encoding="utf-8")
    # ---- console: aggregates only -------------------------------------------------------------------
    print(json.dumps({k: counts[k] for k in ("n_discrepancies", "n_cause_established", "by_verdict", "by_status",
                                             "by_investigation", "decided_by_check", "hash_failures",
                                             "files_refused_by_the_guard")}, indent=1))
    print(f"wrote {OUT.relative_to(REPO)} and {SUMMARY.relative_to(REPO)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
