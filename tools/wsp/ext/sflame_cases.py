#!/usr/bin/env python3
"""ACME s-Flame (PSI-23): inputs, endpoints and case files for the burner-flame target of the adopted
normal-flame model (`acme_normal.wsp12` F2), and the score of that target.

Target (as for CFI-G and Flame Design, PROTOCOL 11.1 and 12.10): when the flame ends after the measured
onset and why (extinction or operator termination), at cutoffs 1, 3 and 5 s, as event probabilities at
the 14 development horizons 6-120 s, scored by the IPCW integrated Brier score of
`tools/wsp/ext/transfer_acme.py` (a port of crates/ignis-lab/src/eval/scores.rs).

Partitions: docs/wsp/atlas/splits/PSI-23.json. Train days: everything may be read. Validation days:
inputs only (script file, data-file header and its first 0.5 s, before the flow is on); their endpoints
are measured by script and written to `cases_validation_outcomes.jsonl`, never printed. Test days and
unlisted days (19245): no file is opened.

Subcommands (all write under runs/dev/data_all/work/transfer_sflame/):

  tad-check   Reproduces the tabulated adiabatic flame temperature of CFI-G (PSI-159) TRAIN records and
              of the Flame Design (PSI-10) TRAIN-day table rows with the equilibrium of `tad_eq`
              (cantera 3.2, NASA-polynomial species of nasa_gas.yaml). -> tad_check.json
  inputs      Inputs of every test of the train and validation days. -> inputs.jsonl, inputs_report.json
  endpoints   flame_design.py's measurement (code and thresholds unchanged; only its folder is set
              here) on train-day tests -> endpoints_train.csv, endpoints_check.json (train only), and on
              validation-day tests -> cases_validation_outcomes.jsonl (no console output of any kind
              about them except the number of rows written).
  build       cases_train.jsonl, cases_validation_inputs.jsonl (inputs only).
  baseline    Incidence baselines through `score` on train days -> baseline_train.json
  f2-train    The adopted F2 on the TRAIN days (a forecast on days that are study data for the author
              of a law; reported as such), no-stop and planned-stop paths -> f2_train.json

    python tools/wsp/ext/sflame_cases.py tad-check
    python tools/wsp/ext/sflame_cases.py inputs
    python tools/wsp/ext/sflame_cases.py endpoints
    python tools/wsp/ext/sflame_cases.py build
    python tools/wsp/ext/sflame_cases.py baseline
    python tools/wsp/ext/sflame_cases.py f2-train --ignis target-merged/release/ignis.exe

`score(forecasts, cases)` is the scoring function: forecasts map (test, cutoff) to the incidence of
each cause at HORIZONS; it returns the aggregate integrated Brier (extinction, operator termination,
their sum), the per-day scores and the cases that had no forecast.
"""
import argparse
import csv
import hashlib
import json
import math
import os
import re
import sys
from collections import Counter, defaultdict
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import transfer_acme as ta  # noqa: E402  (score, forward, prefix helpers; unchanged)
import flame_design as fdm  # noqa: E402  (endpoint measurement; unchanged)
import acme_script2 as a2  # noqa: E402  (planned stop and burner limit; unchanged)

REPO = ta.REPO
RAW = REPO / "runs/dev/data_all/raw/PSI-23/Raw Data"
TABLE = REPO / "runs/dev/data_all/raw/PSI-23/Experimental table/PSI-23_Experimental table_ACME s-Flame.csv"
FD_TABLES = REPO / "runs/dev/data_all/raw/PSI-10/Experimental table"
OUT = REPO / "runs/dev/data_all/work/transfer_sflame"
PRE = "PSI-23_Raw Data_"
SPLIT = REPO / "docs/wsp/atlas/splits/PSI-23.json"
CFIG_BASELINE = REPO / "runs/dev/data_all/work/transfer_acme/fd_fuel/incidence_baseline_cfig_train.json"
CFIG_BASELINE_SHA = "c526a6f1745277d76c038ea373b9e31e28d744db16d28e2903007f418320908b"
F2 = "sha256:55fcf4cbbf500ca57a4e4207426826d5c3d3b21d84fb279479e29b7e5a65971d"
F2_CHAIN = REPO / "runs/dev/data_all/work/transfer_acme/fd_f2/cfig/chain_check.json"
HORIZONS, CUTOFFS, CAUSES = ta.HORIZONS, ta.CUTOFFS, ta.CAUSES
METRICS = ("extinction", "operator_termination", "sum")
NAN = float("nan")

fdm.FILES = RAW  # flame_design.measure reads FILES / <name>; nothing else of it is changed

# ---------------------------------------------------------------------------------------------
# documented conversions
# ---------------------------------------------------------------------------------------------
# Hastings HFC-D-302, nitrogen-calibrated, standard 0 C and 760 torr (s-Flame data interpretation,
# 20 Sept. 2021, section 4.A.1). Component gas conversion factors from ACMEFlowCalculator.xlsx
# (PSI-22 Raw Data, sheet Tables), mixture rule 1/sum(x_i/GCF_i) as in that workbook. The workbook
# writes 99 ("n/a") for the helium GCF of its H2/CH4/He bottles (codes 11-13), which gives a mixture
# factor near 1.9; here helium is 1.4005 for every bottle.
GCF = {"o2": 0.9779, "he": 1.4005, "n2": 1.0, "ethylene": 0.6062, "methane": 0.7787, "h2": 1.0038}
MW = {"o2": 31.9988, "he": 4.002602, "n2": 28.0134, "ethylene": 28.0532, "methane": 16.0425, "h2": 2.01588}
NU_O2 = {"ethylene": 3.0, "methane": 2.0, "h2": 0.5, "ethane": 3.5, "propane": 5.0, "butane": 6.5}
VM_0C = 22.41397  # L/mol at 0 C, 101.325 kPa
ATM_PA, PSI_PA = 101325.0, 6894.757293168
# MFC gas codes of the data-file header (ACME-DOC-010 Rev D, SystemDefineSetup; ACMEFlowCalculator.xlsx HeaderTables)
CODES_MFCA3 = {0: {"methane": 1.0}, 1: {"methane": .3, "n2": .7}, 2: {"methane": .4, "n2": .6},
               3: {"methane": .45, "n2": .55}, 4: {"ethylene": 1.0}, 5: {"ethylene": .3, "n2": .7},
               6: {"ethylene": .5, "n2": .5}, 7: {"h2": 1.0}, 8: {"h2": .444, "methane": .556},
               9: {"h2": .556, "methane": .444}, 10: {"h2": .667, "methane": .333},
               11: {"h2": .2, "methane": .25, "he": .55}, 12: {"h2": .25, "methane": .2, "he": .55},
               13: {"h2": .3, "methane": .15, "he": .55}, 14: {"h2": .45, "he": .55},
               15: {"methane": .45, "he": .55}, 16: {"ethylene": .2, "he": .8},
               17: {"ethylene": .25, "he": .75}, 18: {"ethylene": .3, "he": .7}}
CODES_MFCA2 = {0: {"n2": 1.0}, 1: {"he": 1.0}}
# Days where the header's MFCA3 code and the experiment table disagree, settled by the third NASA source,
# Table 1 of the s-Flame data interpretation document (20 Sept. 2021, p. 5):
#   20190  header 14 (45 % H2 / He), table "35% H2 with He", Table 1 "45% H2 with He"  -> header
#   20136  header 12 (25/20/55 H2/CH4/He), table and Table 1 "20% H2, 25% CH4, with He" -> table (the code of
#          20132, 25/20/55, appears not to have been changed)
BOTTLE_RESOLVED = {"20190": ("header", "Table 1 of the data interpretation document agrees with the header code"),
                   "20136": ("table", "Table 1 of the data interpretation document agrees with the experiment table; "
                                      "the header keeps the code of 20132")}
FUELS = ("ethylene", "methane", "h2")
SPECIES_TEXT = {"ethylene": "ethylene", "methane": "methane", "h2": "hydrogen"}
# PTA3 (chamber) in V. Documented conversion (ACME non-imaging data file description, AIN-6):
# psia = (V - 0.5) * 14.5038 * 10 / 4. The table pressures of Flame Design and CFI-G (on which the models
# were typed) follow the chamber calculator; the straight line through the calculator's start pressures
# (docs/wsp/measure/DISCREPANCIES.md: psia = -18.30 + 36.03 V) is used for `pressure`, the documented
# one is kept beside it. Neither is fitted here.
PTA_DOC = (lambda v: (v - 0.5) * 14.5038 * 10.0 / 4.0)
PTA_CALC = (lambda v: -18.30 + 36.03 * v)
PRE_WINDOW_S = 0.5      # the first 0.5 s of the record (the record starts about 24 s before ignition)
FLOW_OFF_V = 0.10       # an MFC readback below this in the window: no flow yet (MFCA2 idles near 0.06 V)
O2_BLOCK_DROP_V = 0.0015  # a fall of PTA3 between consecutive tests of a day: the chamber was vented and refilled
TK_UV_PER_K = 41.0      # type K sensitivity near room temperature, for the burner temperature at the start


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def split():
    s = json.loads(SPLIT.read_text(encoding="utf-8"))
    g = s["strata"]["all"]["groups"]
    return {d.split(":")[-1]: p for p in g for d in g[p]}


def part_of(test, parts):
    return parts.get(test[:5], "held")


def write_jsonl(path, rows):
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8", newline="\n") as f:
        for r in rows:
            f.write(json.dumps(r, ensure_ascii=False, allow_nan=False, default=_clean) + "\n")


def _clean(x):
    return None


def nn(x):
    """NaN -> None for JSON."""
    if isinstance(x, float) and not math.isfinite(x):
        return None
    if isinstance(x, dict):
        return {k: nn(v) for k, v in x.items()}
    if isinstance(x, list):
        return [nn(v) for v in x]
    return x


def read_jsonl(path):
    return [json.loads(line) for line in Path(path).read_text(encoding="utf-8").splitlines() if line.strip()]


# ---------------------------------------------------------------------------------------------
# adiabatic flame temperature
# ---------------------------------------------------------------------------------------------
_GAS = None
CT_NAME = {"ethylene": "C2H4", "methane": "CH4", "h2": "H2", "ethane": "C2H6", "propane": "C3H8",
           "butane": "C4H10,n-butane", "o2": "O2", "n2": "N2", "he": "He"}
PRODUCTS = ["CO2", "H2O", "CO", "H2", "O2", "N2", "OH", "H", "O", "NO", "NO2", "N2O", "HO2", "H2O2", "He"]


def gas():
    global _GAS
    if _GAS is None:
        import cantera as ct
        allsp = {s.name: s for s in ct.Species.list_from_file("nasa_gas.yaml")}
        names = list(dict.fromkeys(PRODUCTS + [CT_NAME[k] for k in NU_O2]))
        _GAS = ct.Solution(thermo="ideal-gas", species=[allsp[n] for n in names])
    return _GAS


def tad_eq(fuel_stream, ox_stream, p_pa, t0_k=298.15):
    """Adiabatic flame temperature (K): constant-pressure, constant-enthalpy equilibrium of the
    stoichiometric mixture of a fuel stream and an oxidizer stream (mole fractions by species key of
    CT_NAME), both at t0_k, at p_pa. Stoichiometric: moles of fuel stream per mole of oxidizer stream
    a = X_O2,ox / sum_i X_i,fuel nu_i (nu_i: mol O2 per mol of fuel species i, complete combustion)."""
    demand = sum(x * NU_O2[k] for k, x in fuel_stream.items() if k in NU_O2)
    if not (demand > 0 and ox_stream.get("o2", 0) > 0 and p_pa > 0):
        return NAN
    a = ox_stream["o2"] / demand
    X = defaultdict(float)
    for k, x in fuel_stream.items():
        X[CT_NAME[k]] += a * x
    for k, x in ox_stream.items():
        X[CT_NAME[k]] += x
    g = gas()
    g.TPX = t0_k, p_pa, dict(X)
    g.equilibrate("HP")
    return float(g.T)


def zst_of(fuel_stream, ox_stream):
    """Stoichiometric mixture fraction (mass) of the fuel stream against the oxidizer stream, with the
    molar masses of the gases actually present (helium where helium is the diluent)."""
    mw_f = sum(x * MW[k] for k, x in fuel_stream.items())
    mw_o = sum(x * MW[k] for k, x in ox_stream.items())
    d1 = sum(x * MW[k] / mw_f * NU_O2[k] * MW["o2"] / MW[k] for k, x in fuel_stream.items() if k in NU_O2)
    y_o2 = ox_stream["o2"] * MW["o2"] / mw_o
    return y_o2 / (y_o2 + d1)


def stats(d):
    import numpy as np
    d = np.array([x for x in d if x == x])
    if not len(d):
        return dict(n=0)
    return dict(n=int(len(d)), median=round(float(np.median(d)), 2), mean=round(float(d.mean()), 2),
                p95_abs=round(float(np.percentile(np.abs(d), 95)), 2), max_abs=round(float(np.abs(d).max()), 2),
                within_20K=int((np.abs(d) <= 20).sum()))


def cmd_tad_check(_args):
    import cantera as ct
    sys.path.insert(0, str(REPO / "tools/wsp/analysis"))
    import lib
    rows = []
    # CFI-G train records (train ids only)
    for fam in ("acme_normal", "acme_inverse"):
        ids = lib.partition_ids(fam, "train")
        for rid, r in lib.experiments(fam, ids).items():
            c = r["conditions"]
            g = lambda k: c[k]["value_si"] if k in c else None  # noqa: E731
            fuel = c["fuel_species"]["raw_value"].lower().replace("n-butane", "butane")
            if None in (g("tad"), g("x_o2_role"), g("x_fuel_role"), g("pressure")):
                continue
            xf, xo, p = g("x_fuel_role"), g("x_o2_role"), g("pressure")
            fs, os_ = {fuel: xf, "n2": 1 - xf}, {"o2": xo, "n2": 1 - xo}
            rows.append(dict(source="CFI-G train record", family=fam, test=rid.split(":")[-1], fuel=fuel, table=g("tad"),
                             initial_at_p=tad_eq(fs, os_, p), initial_at_1atm=tad_eq(fs, os_, ATM_PA), p_atm=p / ATM_PA))
    # Flame Design train-day table rows
    fd_train = {"acme_normal": ta.FD_DAYS["acme_normal"]["train_days"], "acme_inverse": ta.FD_DAYS["acme_inverse"]["train_days"]}
    spec = {"acme_normal": ("PSI-10_Experimental table_Flame Design_Normal Flame.csv", "Xfuel", "XO2 (init)", "Xfuel", "XO2 (final)"),
            "acme_inverse": ("PSI-10_Experimental table_Flame Design_Inverse Flame.csv", "XC2H4 initial", "XO2", "XC2H4 final", "XO2")}
    for fam, (name, fx, ox, fx2, ox2) in spec.items():
        with (FD_TABLES / name).open(encoding="utf-8-sig", newline="") as f:
            for r in csv.DictReader(f):
                r = {k.strip(): (v or "").strip() for k, v in r.items()}
                if r["Test"][:5] not in fd_train[fam]:
                    continue
                try:
                    T, xf, xo, pi = (float(r[k]) for k in ("Tad (K)", fx, ox, "pinit (atm)"))
                except ValueError:
                    continue
                fuel = "propane" if r.get("Fuel") == "C3H8" else "ethylene"
                fs, os_ = {fuel: xf, "n2": 1 - xf}, {"o2": xo, "n2": 1 - xo}
                row = dict(source="Flame Design train-day table", family=fam, test=r["Test"], fuel=fuel, table=T,
                           initial_at_p=tad_eq(fs, os_, pi * ATM_PA), initial_at_1atm=tad_eq(fs, os_, ATM_PA), p_atm=pi)
                # diagnostic only (train days): the table's own convention, from its after-test columns
                try:
                    xfm, xom, pm = (float(r[fx]) + float(r[fx2])) / 2, (float(r[ox]) + float(r[ox2])) / 2, float(r["pmean (atm)"])
                    row["mean_composition_at_pmean"] = tad_eq({fuel: xfm, "n2": 1 - xfm}, {"o2": xom, "n2": 1 - xom}, pm * ATM_PA)
                except ValueError:
                    pass
                rows.append(row)
    rep = dict(purpose="reproduce the tabulated Tad of CFI-G train records and Flame Design train-day rows",
               method=dict(code="tools/wsp/ext/sflame_cases.py:tad_eq", cantera=ct.__version__, species="nasa_gas.yaml: "
                           + ", ".join(list(dict.fromkeys(PRODUCTS + [CT_NAME[k] for k in NU_O2]))),
                           mixture="stoichiometric mixture of the fuel stream (x_fuel in N2) and the oxidizer stream (x_O2 in N2)",
                           state="T0 = 298.15 K, HP equilibrium"),
               groups={})
    for key in sorted({(r["source"], r["family"], r["fuel"]) for r in rows}):
        rr = [r for r in rows if (r["source"], r["family"], r["fuel"]) == key]
        g = dict(rows=len(rr), table_range_K=[round(min(r["table"] for r in rr), 1), round(max(r["table"] for r in rr), 1)],
                 p_atm_range=[round(min(r["p_atm"] for r in rr), 3), round(max(r["p_atm"] for r in rr), 3)],
                 initial_composition_at_test_pressure_minus_table=stats([r["initial_at_p"] - r["table"] for r in rr]),
                 initial_composition_at_1_atm_minus_table=stats([r["initial_at_1atm"] - r["table"] for r in rr]))
        if any("mean_composition_at_pmean" in r for r in rr):
            g["diagnostic_mean_of_initial_and_final_composition_at_pmean_minus_table"] = stats(
                [r["mean_composition_at_pmean"] - r["table"] for r in rr if "mean_composition_at_pmean" in r])
        rep["groups"][" | ".join(key)] = g
    ta.write_json(OUT / "tad_check.json", nn(rep))
    write_jsonl(OUT / "tad_check_rows.jsonl", [nn(r) for r in rows])
    print(json.dumps(nn(rep), indent=1))
    return 0


# ---------------------------------------------------------------------------------------------
# inputs
# ---------------------------------------------------------------------------------------------
def table_days():
    out = {}
    with TABLE.open(encoding="utf-8-sig", newline="") as f:
        for i, r in enumerate(csv.DictReader(f)):
            r = {k.strip(): (v or "").strip() for k, v in r.items()}
            atm, bottle = r["Atmosphere*"], r["Fuel Bottle"]
            o2 = sorted({float(x) / 100 for x in re.findall(r"~(\d+)% *(?:and|O2)", atm + " O2")})
            out[r["Test Day*"]] = dict(row=i + 2, atmosphere=atm, fuel_bottle=bottle,
                                       ambient_diluent="he" if "He" in atm else "n2", o2_nominal=o2)
    return out


def bottle_from_text(text):
    """The experiment table's 'Fuel Bottle' cell as mole fractions (for the cross-check with the header code)."""
    t = text.replace(",", " ")
    comp = {}
    for pct, sp in re.findall(r"([\d.]+)% *(H2|CH4|C2H4)", t):
        comp[{"H2": "h2", "CH4": "methane", "C2H4": "ethylene"}[sp]] = float(pct) / 100
    rest = 1 - sum(comp.values())
    if rest > 1e-6:
        m = re.search(r"with (N\d|He|CH4)", t)  # the table writes N3, N4 ... for N2 on some rows
        bal = {"He": "he", "CH4": "methane"}.get(m.group(1), "n2") if m else None
        if bal:
            comp[bal] = comp.get(bal, 0) + rest
    return comp


def list_tests():
    tests = defaultdict(set)
    for n in os.listdir(RAW):
        m = re.match(re.escape(PRE) + r"(?:as run)?(\d{5}[A-Z]+\d+)(_data\.csv|_mfcSetpoints\.csv|_TP\.txt|_Comments\.txt|\.txt)$", n)
        if m:
            tests[m.group(1)].add(("as run" if "as run" in n else "") + m.group(2))
    return tests


def file_names(test, have):
    return dict(data=f"{PRE}{test}_data.csv" if "_data.csv" in have else None,
                setpoints=f"{PRE}{test}_mfcSetpoints.csv" if "_mfcSetpoints.csv" in have else None,
                tp=f"{PRE}{test}_TP.txt" if "_TP.txt" in have else None,
                script=(f"{PRE}as run{test}.txt" if "as run.txt" in have else (f"{PRE}{test}.txt" if ".txt" in have else None)))


def read_start(path):
    """Header and the first PRE_WINDOW_S of the sensor record: the state before the flow is on."""
    import numpy as np
    with open(path, encoding="utf-8", errors="replace") as f:
        hdr = fdm.read_header(f.readline())
        cols = [c.strip() for c in f.readline().split(",")]
        t, rows = [], []
        for line in f:
            sp = line.rstrip("\r\n").split(",")
            if len(sp) != len(cols):
                continue
            try:
                tc, vv = fdm.clock(sp[0]), [float(x) for x in sp[1:]]
            except ValueError:
                continue
            if t and tc - t[0] > PRE_WINDOW_S:
                break
            t.append(tc)
            rows.append(vv)
    a = np.array(rows, float)
    med = {c: float(np.median(a[:, i])) for i, c in enumerate(cols[1:])} if len(rows) else {}
    return hdr, cols, med, len(rows)


def num(s, k):
    v = s.get(k)
    try:
        return float(v)
    except (TypeError, ValueError):
        return NAN


def inputs_of(test, have, parts, days):
    fn = file_names(test, have)
    r = dict(test=test, day=test[:5], part=part_of(test, parts), files=sorted(have), flags=[])
    tday = days.get(test[:5], {})
    r["table"] = dict(row=tday.get("row"), atmosphere=tday.get("atmosphere"), fuel_bottle=tday.get("fuel_bottle"))
    # --- script parameter file (set before ignition) ---
    s = a2.parse(RAW / fn["script"]) if fn["script"] else {}
    r["script"] = dict(file=fn["script"], sha256=sha(RAW / fn["script"]) if fn["script"] else None)
    if not s:
        r["flags"].append("no_script_file")
    flows = {k: num(s, k) for k in ("fuelBottleIgnitionFlowRateN2Slpm", "nitrogenIgnitionFlowRateN2Slpm",
                                    "fuelBottleTestFlowRateN2Slpm", "nitrogenTestFlowRateN2Slpm")}
    ramp = {k: num(s, k) for k in ("numberOfRampHolds", "rampSizeFuelSlpm", "rampSizeN2Slpm", "ignitionToTestRampTimeSec")}
    r["script"].update(flows_n2slpm=flows, ramp=ramp, constant_flow_time_s=num(s, "constantFlowTimeSec"),
                       extinction_detection_stops_test=s.get("extinctionDetectionStopsTest"),
                       flow_timeout_s=num(s, "flowTimeoutSec"))
    has_c = "constantFlowTimeSec" in s or "constantFlowTime1Sec" in s
    ramped = any((ramp[k] or 0) != 0 for k in ("numberOfRampHolds", "rampSizeFuelSlpm", "rampSizeN2Slpm") if ramp[k] == ramp[k])
    r["constant_flow"] = bool(s) and has_c and not ramped and all(v == v for v in flows.values())
    r["ignition_flow_equals_test_flow"] = (flows["fuelBottleIgnitionFlowRateN2Slpm"] == flows["fuelBottleTestFlowRateN2Slpm"]
                                           and flows["nitrogenIgnitionFlowRateN2Slpm"] == flows["nitrogenTestFlowRateN2Slpm"])
    if r["constant_flow"] and not r["ignition_flow_equals_test_flow"]:
        r["flags"].append("ignition_flow_differs_from_test_flow")
    plan = a2.for_test("PSI-23", test)
    stop, branch = plan.get("planned_stop_s"), plan.get("branch")
    if not (stop is not None and stop > 0):
        ss = plan.get("stage_sum_s")
        stop, branch = (ss, f"{branch} -> stage sum (registered fallback)") if ss and ss > 0 else (None, f"{branch}, no stage sum")
    r["planned_stop"] = dict(value_s=stop, branch=branch, acme_script2=dict(planned_stop_s=plan.get("planned_stop_s"),
                             stage_sum_s=plan.get("stage_sum_s"), terms=plan.get("terms"), flags=plan.get("flags")),
                             method=f"{a2.METHOD_ID} v{a2.METHOD_VERSION}")
    r["burner_limit"] = plan.get("burner_limit")
    # --- data-file header and the first 0.5 s ---
    if fn["data"]:
        hdr, cols, med, n = read_start(RAW / fn["data"])
        codes = {i: int(float(hdr[f"MFCA{i} Gas"])) if hdr.get(f"MFCA{i} Gas") not in (None, "") else None for i in (1, 2, 3)}
        r["header"] = dict(date=hdr.get("date"), start_clock_s=hdr.get("script_clock"), mfc_gas_codes=codes,
                           mfc_full_scale_slpm={i: num(hdr, f"MFCA{i} Full Scale") for i in (1, 2, 3)},
                           experiment=hdr.get("Experiment"), burner_code=hdr.get("Burner"), samples_in_window=n)
        pre = dict(pta2_v=med.get("PTA2", NAN), pta3_v=med.get("PTA3", NAN),
                   mfc_readback_v=[med.get(c, NAN) for c in ("MFCA1StaticMixerFlow", "MFCA2BypassFlow", "MFCA3FuelFlow")],
                   tcburner_v=med.get("TCBurner", NAN), tccj_v=med.get("TCCJ", NAN))
        if max(abs(x) for x in pre["mfc_readback_v"] if x == x) > FLOW_OFF_V:
            r["flags"].append("flow_on_in_start_window")
        tcj = pre["tccj_v"] / 0.00295 - 273.15
        pre["burner_t_start_c"] = tcj + pre["tcburner_v"] * 1e6 / TK_UV_PER_K
        pre["cold_junction_c"] = tcj
        r["pre_ignition"] = pre
    else:
        r["header"], r["pre_ignition"] = {}, {}
        r["flags"].append("no_data_file")
    # --- compositions and flows ---
    code3 = r["header"].get("mfc_gas_codes", {}).get(3)
    code2 = r["header"].get("mfc_gas_codes", {}).get(2)
    bottle = CODES_MFCA3.get(code3)
    table_bottle = bottle_from_text(tday.get("fuel_bottle", ""))
    if bottle is None:
        bottle = table_bottle or None
        r["flags"].append("bottle_from_table_text" if bottle else "bottle_unknown")
    elif table_bottle and any(abs(bottle.get(k, 0) - table_bottle.get(k, 0)) > 0.011 for k in set(bottle) | set(table_bottle)):
        r["flags"].append(f"header_gas_code_{code3}_differs_from_table_bottle")
        use, why = BOTTLE_RESOLVED.get(test[:5], ("header", "unresolved: header code used"))
        r["bottle_discrepancy"] = dict(header=bottle, table=table_bottle, used=use, why=why)
        if use == "table":
            bottle = table_bottle
    dil2 = CODES_MFCA2.get(code2, {"n2": 1.0})
    src = f"MFCA3 gas code {code3}" if code3 in CODES_MFCA3 else "experiment table"
    if r.get("bottle_discrepancy", {}).get("used") == "table":
        src = "experiment table (header code disagrees, see bottle_discrepancy)"
    r["bottle"] = dict(composition=bottle, source=src,
                       table_text=tday.get("fuel_bottle"), table_composition=table_bottle)
    q = {}
    if bottle and r["constant_flow"]:
        g_b = 1.0 / sum(x / GCF[k] for k, x in bottle.items())
        g_d = 1.0 / sum(x / GCF[k] for k, x in dil2.items())
        mol = defaultdict(float)  # mol/s
        b_slpm, d_slpm = flows["fuelBottleTestFlowRateN2Slpm"] * g_b, flows["nitrogenTestFlowRateN2Slpm"] * g_d
        for k, x in bottle.items():
            mol[k] += b_slpm * x / VM_0C / 60.0
        for k, x in dil2.items():
            mol[k] += d_slpm * x / VM_0C / 60.0
        tot = sum(mol.values())
        q = dict(gcf_bottle=g_b, gcf_diluent=g_d, bottle_slpm_actual=b_slpm, diluent_slpm_actual=d_slpm,
                 burner_stream={k: v / tot for k, v in mol.items()},
                 fuel_mass_flow_kg_s=sum(mol[k] * MW[k] for k in FUELS) * 1e-3,
                 total_mass_flow_kg_s=sum(v * MW[k] for k, v in mol.items()) * 1e-3,
                 fuel_mass_flow_by_species_kg_s={k: mol[k] * MW[k] * 1e-3 for k in FUELS if mol[k] > 0})
    r["flows"] = q
    fuels = sorted(k for k in (bottle or {}) if k in FUELS)
    r["fuel_species"] = "+".join(SPECIES_TEXT[k] for k in fuels) if fuels else None
    r["diluent"] = sorted({k for k in list((bottle or {}).keys()) + list(dil2.keys()) if k in ("n2", "he") and
                           ((bottle or {}).get(k, 0) > 0 or (k in dil2 and flows["nitrogenTestFlowRateN2Slpm"] > 0))})
    r["ambient"] = dict(diluent=tday.get("ambient_diluent"), o2_nominal=tday.get("o2_nominal"))
    # --- pressure ---
    v3 = r["pre_ignition"].get("pta3_v", NAN)
    r["pressure"] = dict(pa=PTA_CALC(v3) * PSI_PA if v3 == v3 else NAN, pa_doc_formula=PTA_DOC(v3) * PSI_PA if v3 == v3 else NAN,
                         channel="PTA3 median over the first 0.5 s",
                         pa_from_pta2_doc=PTA_DOC(r["pre_ignition"].get("pta2_v", NAN)) * PSI_PA)
    return r


def assign_o2(rows):
    """Per-test O2 fraction. Single-value days: the table's nominal value. Days whose table gives two
    values: unknown per test; tests are grouped into blocks separated by a vent-and-refill, seen as a fall
    of the pre-ignition chamber pressure between consecutive tests of the day (vitiation only raises it)."""
    by_day = defaultdict(list)
    for r in rows:
        by_day[r["day"]].append(r)
    for day, rr in by_day.items():
        rr.sort(key=lambda r: ((r["header"].get("date") or ""), r["header"].get("start_clock_s") or 0.0))
        block, last = 0, None
        for i, r in enumerate(rr):
            v = r["pre_ignition"].get("pta3_v", NAN)
            if last is not None and v == v and last - v > O2_BLOCK_DROP_V:
                block += 1
            if v == v:
                last = v
            r["order_in_day"] = i
            r["pressure_block"] = block
        for r in rr:
            o2 = r["ambient"]["o2_nominal"] or []
            if len(o2) == 1:
                r["x_o2"] = dict(value=o2[0], status="nominal (experiment table, '~')", candidates=o2)
            else:
                r["x_o2"] = dict(value=None, status="unknown per test: the table gives two values for the day",
                                 candidates=o2, pressure_block=r["pressure_block"],
                                 blocks_in_day=max(x["pressure_block"] for x in rr) + 1)
    return rows


def finish_inputs(r):
    """Model inputs in the units and names of the case files, and the adiabatic temperature."""
    q, x_o2 = r["flows"], r["x_o2"]["value"]
    amb_dil = r["ambient"]["diluent"] or "n2"
    inp = dict(pressure=r["pressure"]["pa"], fuel_species=r["fuel_species"])
    alt = {}
    if q:
        fs = q["burner_stream"]
        fuel_x = {k: v for k, v in fs.items() if k in FUELS}
        inp["injected_mass_flow"] = q["fuel_mass_flow_kg_s"]
        inp["x_fuel_role"] = sum(fuel_x.values())
        for xo in (r["x_o2"]["candidates"] or []):
            ox = {"o2": xo, amb_dil: 1 - xo}
            alt[f"{xo:.2f}"] = dict(tad=tad_eq(fs, ox, r["pressure"]["pa"]), tad_1atm=tad_eq(fs, ox, ATM_PA), zst=zst_of(fs, ox))
        if x_o2 is not None:
            a = alt[f"{x_o2:.2f}"]
            inp.update(x_o2_role=x_o2, tad=a["tad"], zst=a["zst"])
            # the zst of flame_design.zst_initial, which takes N2 for every diluent
            fsn = {k: v for k, v in fs.items() if k in FUELS}
            fsn["n2"] = 1 - sum(fsn.values())
            r["zst_n2_convention"] = zst_of(fsn, {"o2": x_o2, "n2": 1 - x_o2})
            r["tad_1atm"] = a["tad_1atm"]
    if r["planned_stop"]["value_s"]:
        inp["programmed_stop"] = r["planned_stop"]["value_s"]
    r["inputs"] = inp
    r["inputs_by_o2_candidate"] = alt
    units = dict(pressure="Pa", injected_mass_flow="kg/s (fuel species only)", x_fuel_role="1 (fuel mole fraction of the burner stream)",
                 x_o2_role="1 (ambient O2 mole fraction, nominal)", tad="K", zst="1", fuel_species="text",
                 programmed_stop="s on the forecast clock (acme_script2.py)")
    r["units"] = {k: units[k] for k in inp}
    return r


def cmd_inputs(_args):
    parts, days = split(), table_days()
    tests = list_tests()
    rows = []
    for t in sorted(tests):
        if part_of(t, parts) not in ("train", "validation"):
            continue  # test days and the unlisted day: names counted, nothing opened
        rows.append(inputs_of(t, tests[t], parts, days))
    assign_o2(rows)
    for r in rows:
        finish_inputs(r)
    rows.sort(key=lambda r: r["test"])
    write_jsonl(OUT / "inputs.jsonl", [nn(r) for r in rows])
    names = Counter(part_of(t, parts) for t in tests)
    rep = dict(test_names_by_partition=dict(names), opened=len(rows))
    rep.update(inputs_report(rows))
    ta.write_json(OUT / "inputs_report.json", nn(rep))
    print(json.dumps(nn(rep), indent=1))
    return 0


def rng(v, nd=4):
    v = [x for x in v if x is not None and x == x]
    return [round(min(v), nd), round(max(v), nd)] if v else None


def inputs_report(rows):
    out = dict(by_day={}, flags=dict(Counter(f for r in rows for f in r["flags"])))
    for d in sorted({r["day"] for r in rows}):
        rr = [r for r in rows if r["day"] == d]
        cf = [r for r in rr if r["constant_flow"]]
        out["by_day"][d] = dict(
            part=rr[0]["part"], tests=len(rr), constant_flow=len(cf), fuel=Counter(r["fuel_species"] for r in rr).most_common(),
            bottle=rr[0]["bottle"]["source"] + " " + json.dumps(rr[0]["bottle"]["composition"]),
            table_bottle=rr[0]["bottle"]["table_text"], ambient=rr[0]["table"]["atmosphere"],
            x_o2=sorted({str(r["x_o2"]["value"]) for r in rr}), pressure_blocks=max(r["pressure_block"] for r in rr) + 1,
            pressure_atm=rng([r["pressure"]["pa"] / ATM_PA for r in rr]),
            pressure_atm_doc_formula=rng([r["pressure"]["pa_doc_formula"] / ATM_PA for r in rr]),
            injected_mass_flow_mg_s=rng([r["inputs"].get("injected_mass_flow", NAN) * 1e6 for r in cf], 3),
            x_fuel_role=rng([r["inputs"].get("x_fuel_role", NAN) for r in cf]),
            tad_K=rng([r["inputs"].get("tad", NAN) for r in cf], 1),
            tad_K_by_o2_candidate={k: rng([r["inputs_by_o2_candidate"].get(k, {}).get("tad", NAN) for r in cf], 1)
                                   for k in sorted({k for r in cf for k in r["inputs_by_o2_candidate"]})},
            tad_minus_tad_1atm_K=rng([r["inputs"].get("tad", NAN) - r.get("tad_1atm", NAN) for r in cf], 2),
            zst=rng([r["inputs"].get("zst", NAN) for r in cf]),
            planned_stop_s=dict(Counter(f"{r['planned_stop']['value_s']} ({r['planned_stop']['branch']})" for r in rr)),
            burner_limit_c=dict(Counter(str(r["burner_limit"].get("burner_limit_c")) for r in rr)),
            burner_t_start_c=rng([r["pre_ignition"].get("burner_t_start_c", NAN) for r in rr], 1))
    return out


# ---------------------------------------------------------------------------------------------
# endpoints
# ---------------------------------------------------------------------------------------------
def tp_class(test, fn):
    if not fn["tp"]:
        return "no_tp", None
    stamps, _, reason = fdm.read_tp(RAW / fn["tp"])
    rs = (reason or "").lower()
    if "extinction" in rs:
        return "script_extinction_detected", reason
    if "burner temperature limit" in rs:
        return "script_burner_limit", reason
    if reason:
        return "script_abort_other", reason
    if "MFCs off" in stamps:
        return "script_scripted_end", None
    return "no_stamps", None


def endpoint_of(m):
    """transfer_acme.fd_tests's rule: an exact event at the measured flame end, cause from the sensor
    label; burning at the file end: right-censored there; anything else: no endpoint."""
    end, label = m.get("end_s", NAN), m.get("label_sensor")
    if m.get("status") != "ok":
        return None, f"status {m.get('status')}"
    if label in CAUSES and end == end:
        return dict(censoring="event", last_confirmed_active_s=end, first_terminal_s=end, allowed_causes=[label]), ""
    if label == "burning_at_file_end" and end == end:
        return dict(censoring="right", last_confirmed_active_s=end, first_terminal_s=None, allowed_causes=[]), ""
    return None, f"label {label or 'none'}"


def measure_test(test, have):
    fn = file_names(test, have)
    try:
        m = fdm.measure(test, fn)
    except Exception as e:  # as flame_design's own loop does
        m = dict(test=test, status=f"error:{type(e).__name__}", flags=[])
    cls, reason = tp_class(test, fn)
    ep, why = endpoint_of(m)
    return m, cls, reason, ep, why


def cmd_endpoints(_args):
    parts = split()
    tests = list_tests()
    inputs = {r["test"]: r for r in read_jsonl(OUT / "inputs.jsonl")}
    train_rows, val_rows = [], []
    for t in sorted(inputs):
        part = inputs[t]["part"]
        m, cls, reason, ep, why = measure_test(t, tests[t])
        row = dict(test=t, day=t[:5], part=part, status=m.get("status"), label_sensor=m.get("label_sensor"),
                   end_s=m.get("end_s"), onset_method=m.get("onset_method"), onset_minus_stamp_s=m.get("onset_minus_stamp_s"),
                   pmt_end_s=m.get("pmt_end_s"), flow_change_s=m.get("flow_change_s"), flow_change_source=m.get("flow_change_source"),
                   flow_off_s=m.get("flow_off_s"), constant_stage_begins_s=m.get("constant_stage_begins_s"),
                   file_end_s=m.get("file_end_s"), ignition_stamp_source=m.get("ignition_stamp_source"),
                   tp_class=cls, tp_reason=reason, tp_extinction_detected_s=m.get("tp_extinction_detected_s"),
                   tp_mfcs_off_s=m.get("tp_mfcs_off_s"), flags="|".join(m.get("flags", [])),
                   endpoint=ep, no_endpoint_reason=why or None,
                   method="tools/wsp/ext/flame_design.py measure (unchanged thresholds), file prefix PSI-23_Raw Data_")
        (train_rows if part == "train" else val_rows).append(nn(row))
    assert all(r["part"] == "validation" for r in val_rows) and parts
    write_jsonl(OUT / "endpoints_train.jsonl", train_rows)
    write_jsonl(OUT / "cases_validation_outcomes.jsonl", val_rows)
    rep = endpoint_check(train_rows, inputs)
    ta.write_json(OUT / "endpoints_check.json", nn(rep))
    print(json.dumps(nn(rep), indent=1))
    print(f"validation days: {len(val_rows)} rows written to cases_validation_outcomes.jsonl (not read here)")
    return 0


def endpoint_check(rows, inputs):
    """Train days only: how the channels and thresholds behave on s-Flame."""
    import numpy as np
    out = dict(train_tests=len(rows), status=dict(Counter(r["status"] for r in rows)),
               onset_method=dict(Counter(r["onset_method"] for r in rows if r["status"] == "ok")),
               onset_minus_ignition_stamp_s=ta.quantiles([r["onset_minus_stamp_s"] for r in rows if r["onset_minus_stamp_s"] is not None]),
               ignition_reference=dict(Counter(r["ignition_stamp_source"] for r in rows if r["status"] == "ok")),
               label_sensor=dict(Counter(r["label_sensor"] for r in rows if r["status"] == "ok")),
               flow_change_source=dict(Counter(r["flow_change_source"] for r in rows if r["status"] == "ok")),
               flags=dict(Counter(f for r in rows for f in (r["flags"] or "").split("|") if f)),
               sensor_label_vs_script_reason=dict(Counter(f"{r['label_sensor']} / {r['tp_class']}" for r in rows if r["status"] == "ok")),
               by_fuel={}, constant_flow_only={})
    agree = {("extinction", "script_extinction_detected"), ("operator_termination", "script_burner_limit"),
             ("operator_termination", "script_scripted_end")}
    for sel_name, sel in (("all", rows), ("constant_flow", [r for r in rows if inputs[r["test"]]["constant_flow"]])):
        ok = [r for r in sel if r["status"] == "ok" and r["tp_class"] not in ("no_tp", "no_stamps")]
        d_ext = [r["end_s"] - r["tp_extinction_detected_s"] for r in ok
                 if r["label_sensor"] == "extinction" and r["tp_extinction_detected_s"] is not None]
        d_off = [r["end_s"] - r["tp_mfcs_off_s"] for r in ok
                 if r["label_sensor"] == "operator_termination" and r["tp_mfcs_off_s"] is not None]
        out[f"agreement_{sel_name}"] = dict(
            with_script_account=len(ok), labels_agree=sum((r["label_sensor"], r["tp_class"]) in agree for r in ok),
            extinction_end_minus_script_extinction_stamp_s=ta.quantiles(d_ext) | dict(n=len(d_ext)),
            termination_end_minus_mfcs_off_stamp_s=ta.quantiles(d_off) | dict(n=len(d_off)),
            early_flow_change_before_2s_after_constant_stage=sum(
                1 for r in ok if r["label_sensor"] == "operator_termination" and r["flow_change_s"] is not None
                and r["constant_stage_begins_s"] is not None and r["flow_change_s"] - r["constant_stage_begins_s"] < 2.0))
    for f in sorted({inputs[r["test"]]["fuel_species"] or "unknown" for r in rows}):
        rr = [r for r in rows if (inputs[r["test"]]["fuel_species"] or "unknown") == f]
        out["by_fuel"][f] = dict(tests=len(rr), status=dict(Counter(r["status"] for r in rr)),
                                 onset=dict(Counter(r["onset_method"] for r in rr)),
                                 label_vs_script=dict(Counter(f"{r['label_sensor']} / {r['tp_class']}" for r in rr)),
                                 flags=dict(Counter(x for r in rr for x in (r["flags"] or "").split("|") if x)))
    # the planned stop of acme_script2 against the script's own stamps, on scripted ends of constant-flow tests
    d = []
    for r in rows:
        i = inputs[r["test"]]
        if r["tp_class"] == "script_scripted_end" and i["constant_flow"] and r["tp_mfcs_off_s"] is not None and i["planned_stop"]["value_s"]:
            d.append(r["tp_mfcs_off_s"] - i["planned_stop"]["value_s"])
    out["mfcs_off_stamp_minus_planned_stop_s_on_scripted_ends"] = ta.quantiles(d) | dict(n=len(d))
    _ = np
    return out


# ---------------------------------------------------------------------------------------------
# cases
# ---------------------------------------------------------------------------------------------
TARGET_RULE = ("constant flow (one constant-flow stage, no ramp keys) and a data file; the endpoint is the "
               "flame end after the measured onset by tools/wsp/ext/flame_design.py")


def case_row(i):
    return dict(test=i["test"], day=i["day"], part=i["part"], session=i["day"], in_target_set=bool(i["constant_flow"] and i["header"]),
                inputs=i["inputs"], units=i["units"], inputs_by_o2_candidate=i["inputs_by_o2_candidate"],
                x_o2=i["x_o2"], diluent=i["diluent"], ambient_diluent=i["ambient"]["diluent"],
                burner_limit_c=i["burner_limit"].get("burner_limit_c"), burner_limit_duration_s=i["burner_limit"].get("burner_limit_duration_s"),
                burner_t_start_c=i["pre_ignition"].get("burner_t_start_c"), planned_stop=i["planned_stop"],
                pressure=i["pressure"], flows=i["flows"], bottle=i["bottle"], date=i["header"].get("date"),
                constant_flow=i["constant_flow"], ignition_flow_equals_test_flow=i["ignition_flow_equals_test_flow"],
                flags=i["flags"], known_before_ignition=True)


def cmd_build(_args):
    inputs = read_jsonl(OUT / "inputs.jsonl")
    ends = {r["test"]: r for r in read_jsonl(OUT / "endpoints_train.jsonl")}
    train, val = [], []
    for i in inputs:
        c = case_row(i)
        if i["part"] == "train":
            e = ends[i["test"]]
            c["outcome"] = dict(endpoint=e["endpoint"], label_sensor=e["label_sensor"], end_s=e["end_s"],
                                no_endpoint_reason=e["no_endpoint_reason"], time_origin=ta.FD_TIME_ORIGIN,
                                tp_class=e["tp_class"])
            train.append(c)
        else:
            val.append(c)
    write_jsonl(OUT / "cases_train.jsonl", train)
    write_jsonl(OUT / "cases_validation_inputs.jsonl", val)
    # counts from inputs (both partitions) and from train outcomes (train only)
    def cnt(rows, **kw):
        return dict(tests=len(rows), in_target_set=sum(r["in_target_set"] for r in rows),
                    target_with_x_o2=sum(r["in_target_set"] and r["inputs"].get("x_o2_role") is not None for r in rows),
                    target_by_fuel=dict(Counter(r["inputs"].get("fuel_species") for r in rows if r["in_target_set"])),
                    target_by_ambient_diluent=dict(Counter(r["ambient_diluent"] for r in rows if r["in_target_set"])), **kw)
    rep = dict(rule=TARGET_RULE,
               train=cnt(train, target_with_endpoint=sum(r["in_target_set"] and r["outcome"]["endpoint"] is not None for r in train),
                         target_at_risk_by_cutoff={f"{c:g}": sum(r["in_target_set"] and ta.at_risk(r["outcome"]["endpoint"], c) for r in train)
                                                   for c in CUTOFFS}),
               validation=cnt(val))
    ta.write_json(OUT / "build_report.json", rep)
    print(json.dumps(rep, indent=1))
    return 0


# ---------------------------------------------------------------------------------------------
# score
# ---------------------------------------------------------------------------------------------
def at_risk_cases(cases, cutoffs=CUTOFFS):
    out = []
    for c in cases:
        e = (c.get("outcome") or {}).get("endpoint")
        for k in cutoffs:
            if ta.at_risk(e, k):
                out.append(dict(test=c["test"], day=c["day"], cutoff=k, outcome=ta.outcome_of(e)))
    return out


def score(forecasts, cases, cutoffs=CUTOFFS):
    """forecasts: {(test, cutoff): {cause: [incidence at each of HORIZONS]}} for the causes in CAUSES.
    cases: rows of cases_train.jsonl (or any rows with test, day and outcome.endpoint).
    A case is (test, cutoff) with the test at risk at the cutoff (transfer_acme.at_risk). The score is the
    IPCW integrated Brier over HORIZONS of transfer_acme.score on the cases that have a forecast (IPCW
    weights from those cases); per day, the mean of the cases' contributions. Cases without a forecast are
    counted, not scored."""
    ks = at_risk_cases(cases, cutoffs)
    have = [k for k in ks if (k["test"], k["cutoff"]) in forecasts]
    res = dict(cases_at_risk=len(ks), cases_scored=len(have), cases_without_forecast=len(ks) - len(have),
               tests_scored=len({k["test"] for k in have}), days_scored=len({k["day"] for k in have}))
    if not have:
        return res
    cif = {c: [forecasts[(k["test"], k["cutoff"])][c] for k in have] for c in CAUSES}
    s = ta.score([k["outcome"] for k in have], cif)
    contrib = {c: s[c]["contrib"] for c in CAUSES}
    contrib["sum"] = [a + b for a, b in zip(contrib["extinction"], contrib["operator_termination"])]
    res["integrated_brier"] = {c: s[c]["integrated_brier"] for c in CAUSES}
    res["integrated_brier"]["sum"] = res["integrated_brier"]["extinction"] + res["integrated_brier"]["operator_termination"]
    res["case_mean_contrib"] = {m: math.fsum(contrib[m]) / len(have) for m in METRICS}
    days = [k["day"] for k in have]
    res["session_equal_mean"] = {m: ta.boot(contrib[m], days, n_boot=2000)["session_equal_mean"] for m in METRICS}
    res["per_day"] = {d: dict(cases=days.count(d), **{m: math.fsum(x for x, dd in zip(contrib[m], days) if dd == d) / days.count(d)
                                                       for m in METRICS}) for d in sorted(set(days))}
    res["endpoints"] = dict(Counter(k["outcome"]["causes"][0] if k["outcome"]["causes"] else "right_censored" for k in have))
    res["_contrib"] = contrib
    res["_keys"] = [(k["test"], k["cutoff"]) for k in have]
    return res


def public(res):
    return {k: v for k, v in res.items() if not k.startswith("_")}


def aalen_johansen(outcomes):
    """Cumulative incidence by cause at HORIZONS (competing causes, right censoring); exact event times."""
    ev = sorted({o["t_upper"] for o in outcomes if o["censoring"] == "event"})
    times = [(o["t_upper"] if o["censoring"] == "event" else o["t_lower"]) for o in outcomes]
    surv, cif, curve = 1.0, {c: 0.0 for c in CAUSES}, []
    for t in ev:
        n = sum(1 for x in times if x >= t)
        d = {c: sum(1 for o in outcomes if o["censoring"] == "event" and o["t_upper"] == t and o["causes"][0] == c) for c in CAUSES}
        for c in CAUSES:
            cif[c] += surv * d[c] / n
        surv *= 1 - sum(d.values()) / n
        curve.append((t, dict(cif)))
    out = {c: [] for c in CAUSES}
    for h in HORIZONS:
        last = {c: 0.0 for c in CAUSES}
        for t, v in curve:
            if t <= h:
                last = v
        for c in CAUSES:
            out[c].append(last[c])
    return out


def incidence_from(cases, cutoffs=CUTOFFS):
    ks = at_risk_cases(cases, cutoffs)
    return {c: aalen_johansen([k["outcome"] for k in ks if k["cutoff"] == c]) for c in cutoffs}


def cmd_baseline(_args):
    cases = [c for c in read_jsonl(OUT / "cases_train.jsonl") if c["in_target_set"]]
    assert ta.hashlib.sha256(CFIG_BASELINE.read_bytes()).hexdigest() == CFIG_BASELINE_SHA
    cfig = json.loads(CFIG_BASELINE.read_text(encoding="utf-8"))["families"]["acme_normal"]["cutoffs"]
    ks = at_risk_cases(cases)
    fc_cfig = {(k["test"], k["cutoff"]): {c: cfig[f"{k['cutoff']:g}"][c] for c in CAUSES} for k in ks}
    inc = incidence_from(cases)
    fc_in = {(k["test"], k["cutoff"]): {c: inc[k["cutoff"]][c] for c in CAUSES} for k in ks}
    fc_lodo = {}
    for d in sorted({c["day"] for c in cases}):
        inc_d = incidence_from([c for c in cases if c["day"] != d])
        for k in ks:
            if k["day"] == d:
                fc_lodo[(k["test"], k["cutoff"])] = {c: inc_d[k["cutoff"]][c] for c in CAUSES}
    rep = dict(purpose="incidence baselines through sflame_cases.score on the s-Flame TRAIN days (target set)",
               target_rule=TARGET_RULE,
               baselines=dict(
                   cfig_train_incidence=dict(source=f"{CFIG_BASELINE.relative_to(REPO).as_posix()} (sha256 {CFIG_BASELINE_SHA}), "
                                                    "family acme_normal: frozen in PROTOCOL 11.1 from CFI-G train; nothing from s-Flame",
                                             score=public(score(fc_cfig, cases))),
                   sflame_train_incidence_in_sample=dict(source="Aalen-Johansen cumulative incidence of the s-Flame train cases at risk, "
                                                                "per cutoff; constants fitted on the same train days (in sample)",
                                                         score=public(score(fc_in, cases)),
                                                         curves={f"{c:g}": inc[c] for c in CUTOFFS}),
                   sflame_train_incidence_leave_one_day_out=dict(source="the same, each train day forecast from the other train days",
                                                                 score=public(score(fc_lodo, cases)))))
    ta.write_json(OUT / "baseline_train.json", nn(rep))
    for name, b in rep["baselines"].items():
        s = b["score"]
        print(f"{name}: {s['cases_scored']} cases / {s['tests_scored']} tests / {s['days_scored']} days; "
              + " / ".join(f"{s['integrated_brier'][m]:.4f}" for m in METRICS) + " (IBS ext / term / sum); case-mean "
              + " / ".join(f"{s['case_mean_contrib'][m]:.4f}" for m in METRICS))
    return 0


# ---------------------------------------------------------------------------------------------
# F2 on the train days
# ---------------------------------------------------------------------------------------------
PREFIX_KEYS = ("pressure", "injected_mass_flow", "x_o2_role", "x_fuel_role", "tad", "zst")


def f2_prefix(c, cutoff, stop, src_sha, row, src="runs/dev/data_all/work/transfer_sflame/cases_train.jsonl"):
    cond, inp = {}, c["inputs"]
    for k in PREFIX_KEYS:
        if inp.get(k) is not None:
            cond[k] = ta.measured(float(inp[k]), {"pressure": "Pa", "injected_mass_flow": "kg/s", "tad": "K"}.get(k, "1"), k, row,
                                  src_sha, f"wsp.ext.sflame.cond.{k}", "tools/wsp/ext/sflame_cases.py inputs (before ignition)", path=src)
    if inp.get("fuel_species"):
        cond["fuel_species"] = ta.measured(inp["fuel_species"], None, "fuel_species", row, src_sha, "wsp.ext.sflame.cond.fuel_species",
                                           "fuel species of the bottle (header MFC gas code)", evidence="original_observation", path=src)
    if stop and inp.get("programmed_stop"):
        m = ta.measured(float(inp["programmed_stop"]), "s", "programmed_stop", row, src_sha, a2.METHOD_ID,
                        "planned flow stop on the forecast clock from the script parameter file (tools/wsp/ext/acme_script2.py; "
                        "the stage sum where the file is incomplete)", path=src)
        m["derivation"]["method_version"] = a2.METHOD_VERSION
        cond["programmed_stop"] = m
    return ta.finish_prefix(dict(
        family="acme_normal", run_id=f"PSI-23:acme_normal:{c['test']}", session_id=f"PSI-23:session:{c['day']}",
        cutoff_s=float(cutoff), time_origin=ta.FD_TIME_ORIGIN, known_conditions=cond, known_controls=[], observations=[],
        contours=[], phase_established_at_s=0.0,
        support_evidence=["activity confirmed at the cutoff by the measured flame end (tools/wsp/ext/flame_design.py)"],
        transform_versions={}))


def cmd_f2_train(args):
    ta.selftest()
    binary = ta.use_binary(args)
    chain = json.loads(F2_CHAIN.read_text(encoding="utf-8"))
    assert chain["reproduced"] and chain["ignis"]["sha256"] == binary["sha256"], "F2 chain check not reproduced with this binary"
    allc = read_jsonl(OUT / "cases_train.jsonl")
    src_sha = sha(OUT / "cases_train.jsonl")
    row_of = {c["test"]: i + 1 for i, c in enumerate(allc)}
    cases = [c for c in allc if c["in_target_set"]]
    assert all(c["part"] == "train" for c in cases)
    cfig = json.loads(CFIG_BASELINE.read_text(encoding="utf-8"))["families"]["acme_normal"]["cutoffs"]
    rep = dict(purpose="F2 (acme_normal.wsp12) on s-Flame TRAIN days. These days are study data for a law's author: "
                       "this is a look at train days, not a held-out score.", release=F2, ignis=binary,
               chain_check=F2_CHAIN.relative_to(REPO).as_posix(), cases_train_sha256=src_sha, paths={})
    ks = at_risk_cases(cases)
    byt = {c["test"]: c for c in cases}
    for stop in (False, True):
        path = "stop" if stop else "nostop"
        jobs = []
        for k in ks:
            name = f"{k['test']}_c{k['cutoff']:g}_{path}.json"
            pp = OUT / "f2_train/prefix" / name
            ta.write_json(pp, f2_prefix(byt[k["test"]], k["cutoff"], stop, src_sha, row_of[k["test"]]))
            jobs.append(((k["test"], k["cutoff"]), F2, pp, OUT / "f2_train/artifact" / name))
        arts = ta.forward_many(jobs, args.workers)
        fc, refused, diag = {}, {}, {}
        for key, a in arts.items():
            if ta.admitted(a):
                fc[key] = ta.cif_of(a)
                diag[key] = a.get("diagnostics") or {}
            else:
                refused[key] = f"{a['support'].get('gate')}: {'; '.join(a['support'].get('reasons', []))}"
        reasons = Counter(re.sub(r"-?\d+(\.\d+)?(e-?\d+)?", "#", v) for v in refused.values())
        ans = [c for c in cases if any((c["test"], k) in fc for k in CUTOFFS)]
        fc_base = {key: {c: cfig[f"{key[1]:g}"][c] for c in CAUSES} for key in fc}
        inc = incidence_from(cases)
        fc_inc = {key: {c: inc[key[1]][c] for c in CAUSES} for key in fc}
        s_f2, s_b, s_i = score(fc, cases), score(fc_base, cases), score(fc_inc, cases)
        paired = {m: ta.boot([a - b for a, b in zip(s_f2["_contrib"][m], s_b["_contrib"][m])], [k[0][:5] for k in s_f2["_keys"]],
                             n_boot=10000) for m in METRICS} if fc else {}
        tests_by_fuel = Counter(byt[t]["inputs"].get("fuel_species") for t in {k[0] for k in fc})
        refused_by_fuel = Counter(byt[t]["inputs"].get("fuel_species") for t in {k[0] for k in refused})
        dates = Counter(d.get("test_date_source") for d in diag.values())
        rep["paths"][path] = dict(
            cases_at_risk=len(ks), answered=len(fc), refused=len(refused), refused_tests=len({k[0] for k in refused}),
            refusal_reasons=dict(reasons), answered_tests_by_fuel=dict(tests_by_fuel), refused_tests_by_fuel=dict(refused_by_fuel),
            test_date_source=dict(dates), p_late=dict(Counter(str(d.get("p_late")) for d in diag.values())),
            f2=public(s_f2), cfig_incidence_on_same_cases=public(s_b), sflame_train_incidence_in_sample_on_same_cases=public(s_i),
            f2_minus_cfig_incidence_paired_session_bootstrap=paired, answered_test_ids=sorted({k[0] for k in fc}))
        # the script's burner limit (250 C) against the model's own burner-heating law (inputs only): the
        # members' centre heating node t0 + dT (1 - exp(-t / tau)); the release itself weighs a 510 C limit
        burner = {}
        for (t, k), d in sorted(diag.items()):
            if t in burner:
                continue
            cl = ((d.get("members") or [{}])[0].get("clocks") or {}).get("clocks") or {}
            rise, t0, tau = cl.get("burner_rise_k"), cl.get("burner_t0_c"), cl.get("burner_heat_tau_s")
            lim = byt[t]["burner_limit_c"]
            if None in (rise, t0, tau):
                continue
            need = (lim - t0) / rise if lim is not None else NAN
            t_lim = -tau * math.log(1 - need) if need == need and 0 < need < 1 else None
            burner[t] = dict(burner_equilibrium_c=t0 + rise, model_limits=cl.get("burner_limits"), script_limit_c=lim,
                             time_to_script_limit_s=t_lim, planned_stop_s=byt[t]["inputs"].get("programmed_stop"))
        rep["paths"][path]["burner_limit_vs_model_heating"] = dict(
            tests=len(burner), script_limit_250=sum(1 for b in burner.values() if b["script_limit_c"] == 250.0),
            model_limit_weighted=sorted({str(b["model_limits"]) for b in burner.values()}),
            equilibrium_above_250=sum(1 for b in burner.values() if b["burner_equilibrium_c"] > 250.0),
            reaches_250_before_120s=sum(1 for b in burner.values() if b["time_to_script_limit_s"] is not None and b["time_to_script_limit_s"] < 120),
            reaches_250_before_planned_stop=sum(1 for b in burner.values() if b["time_to_script_limit_s"] is not None and b["planned_stop_s"]
                                                and b["time_to_script_limit_s"] < b["planned_stop_s"]),
            equilibrium_c_range=rng([b["burner_equilibrium_c"] for b in burner.values()], 1),
            time_to_250_s_range=rng([b["time_to_script_limit_s"] for b in burner.values() if b["time_to_script_limit_s"] is not None], 1),
            per_test=burner)
    ta.write_json(OUT / "f2_train.json", nn(rep))
    for path, v in rep["paths"].items():
        print(f"== {path}: {v['answered']} answered / {v['refused']} refused ({v['refused_tests']} tests); reasons {v['refusal_reasons']}")
        for n in ("f2", "cfig_incidence_on_same_cases", "sflame_train_incidence_in_sample_on_same_cases"):
            s = v[n]
            if "integrated_brier" in s:
                print(f"   {n}: " + " / ".join(f"{s['integrated_brier'][m]:.4f}" for m in METRICS)
                      + f"  ({s['cases_scored']} cases, {s['tests_scored']} tests, {s['days_scored']} days)")
    return 0


# ---------------------------------------------------------------------------------------------
# PROTOCOL 13.1: F2 on the validation days, once; and the train-day study file
# ---------------------------------------------------------------------------------------------
def burner_of(diag, byt):
    """Per test: the first member's burner-heating law and the time to the script's limit (inputs only)."""
    out = {}
    for (t, _k), d in sorted(diag.items()):
        if t in out:
            continue
        cl = ((d.get("members") or [{}])[0].get("clocks") or {}).get("clocks") or {}
        rise, t0, tau = cl.get("burner_rise_k"), cl.get("burner_t0_c"), cl.get("burner_heat_tau_s")
        lim, stop = byt[t]["burner_limit_c"], byt[t]["inputs"].get("programmed_stop")
        if None in (rise, t0, tau):
            continue
        need = (lim - t0) / rise if lim is not None else NAN
        tl = -tau * math.log(1 - need) if need == need and 0 < need < 1 else None
        out[t] = dict(burner_equilibrium_c=t0 + rise, time_to_script_limit_s=tl, planned_stop_s=stop,
                      reaches_limit_before_stop=bool(tl is not None and stop and tl < stop))
    return out


def run_f2(cases, tag, src_rel, workers):
    """F2 on every at-risk case of `cases`, both paths. Returns {path: (forecasts, refusals, diagnostics)}."""
    src = REPO / src_rel
    src_sha = sha(src)
    row_of = {json.loads(line)["test"]: i + 1 for i, line in enumerate(src.read_text(encoding="utf-8").splitlines()) if line.strip()}
    byt = {c["test"]: c for c in cases}
    ks = at_risk_cases(cases)
    out = {}
    for stop in (False, True):
        path = "stop" if stop else "nostop"
        jobs = []
        for k in ks:
            name = f"{k['test']}_c{k['cutoff']:g}_{path}.json"
            pp = OUT / tag / "prefix" / name
            ta.write_json(pp, f2_prefix(byt[k["test"]], k["cutoff"], stop, src_sha, row_of[k["test"]], src=src_rel))
            jobs.append(((k["test"], k["cutoff"]), F2, pp, OUT / tag / "artifact" / name))
        arts = ta.forward_many(jobs, workers)
        fc, refused, diag = {}, {}, {}
        for key, a in arts.items():
            if ta.admitted(a):
                fc[key] = ta.cif_of(a)
                diag[key] = a.get("diagnostics") or {}
            else:
                refused[key] = f"{a['support'].get('gate')}: {'; '.join(a['support'].get('reasons', []))}"
        out[path] = (fc, refused, diag)
    return out, src_sha


def means_by(contrib, keys, sel):
    """Case mean and mean of day means of per-case contributions over the selected cases."""
    idx = [i for i, k in enumerate(keys) if sel(k)]
    if not idx:
        return None
    days = defaultdict(list)
    for i in idx:
        days[keys[i][0][:5]].append(i)
    return dict(cases=len(idx), tests=len({keys[i][0] for i in idx}), days=len(days),
                **{m: dict(case_mean=math.fsum(contrib[m][i] for i in idx) / len(idx),
                           day_mean=math.fsum(math.fsum(contrib[m][i] for i in v) / len(v) for v in days.values()) / len(days))
                   for m in METRICS})


def contrib_on(keys, forecasts, cases):
    """Per-case contributions of a forecast set on `keys` (IPCW weights from those cases)."""
    s = score({k: forecasts[k] for k in keys}, cases)
    return s["_contrib"], s["_keys"]


def baselines_for(keys, train_cases):
    cfig = json.loads(CFIG_BASELINE.read_text(encoding="utf-8"))["families"]["acme_normal"]["cutoffs"]
    inc = incidence_from(train_cases)
    return ({k: {c: cfig[f"{k[1]:g}"][c] for c in CAUSES} for k in keys},
            {k: {c: inc[k[1]][c] for c in CAUSES} for k in keys})


def block_summary(fc, cases, train_cases, byt, burner):
    keys_all = sorted(fc)
    b_cfig, b_inc = baselines_for(keys_all, train_cases)
    con = {n: contrib_on(keys_all, f, cases) for n, f in (("f2", fc), ("cfig_incidence", b_cfig), ("sflame_train_incidence", b_inc))}
    keys = con["f2"][1]
    assert all(con[n][1] == keys for n in con)
    groups = {"all": lambda k: True}
    for fuel in sorted({byt[k[0]]["inputs"]["fuel_species"] for k in keys}):
        groups[f"fuel={fuel}"] = (lambda f: (lambda k: byt[k[0]]["inputs"]["fuel_species"] == f))(fuel)
    groups["reaches 250 C before the planned stop (F2 heating law)"] = lambda k: burner.get(k[0], {}).get("reaches_limit_before_stop") is True
    groups["does not reach 250 C before the planned stop"] = lambda k: burner.get(k[0], {}).get("reaches_limit_before_stop") is False
    out = {}
    for g, sel in groups.items():
        row = {n: means_by(con[n][0], keys, sel) for n in con}
        if row["f2"]:
            idx = [i for i, k in enumerate(keys) if sel(k)]
            days = [keys[i][0][:5] for i in idx]
            row["f2_minus_cfig_incidence_day_bootstrap"] = {
                m: ta.boot([con["f2"][0][m][i] - con["cfig_incidence"][0][m][i] for i in idx], days, n_boot=10000) for m in METRICS}
            ends = Counter()
            for i in idx:
                e = byt[keys[i][0]]["outcome"]["endpoint"]
                ends[e["allowed_causes"][0] if e["allowed_causes"] else "right_censored"] += 1
            row["endpoints"] = dict(ends)
        out[g] = row
    return out


def cmd_validation_f2(args):
    """PROTOCOL 13.1, run once. Aggregates only."""
    ta.selftest()
    binary = ta.use_binary(args)
    chain = json.loads(F2_CHAIN.read_text(encoding="utf-8"))
    bc = json.loads((OUT / "binary_check.json").read_text(encoding="utf-8")) if (OUT / "binary_check.json").exists() else {}
    assert chain["reproduced"] and (chain["ignis"]["sha256"] == binary["sha256"] or (
        bc.get("identical") and bc.get("binary_sha256") == binary["sha256"]
        and bc.get("reference_binary_sha256") == chain["ignis"]["sha256"])), "binary neither chain-checked nor shown identical"
    done = OUT / "validation_13_1.json"
    if done.exists() and not args.again:
        sys.exit(f"{done} exists: 13.1 is run once")
    parts = split()
    vin = read_jsonl(OUT / "cases_validation_inputs.jsonl")
    vout = {r["test"]: r for r in read_jsonl(OUT / "cases_validation_outcomes.jsonl")}
    assert all(r["part"] == "validation" and parts.get(r["day"]) == "validation" for r in vin)
    target = [c for c in vin if c["in_target_set"]]
    for c in target:
        c["outcome"] = dict(endpoint=vout[c["test"]]["endpoint"])
    train_cases = [c for c in read_jsonl(OUT / "cases_train.jsonl") if c["in_target_set"]]
    scope, run = Counter(), []
    for c in target:
        he = c["ambient_diluent"] == "he" or "he" in (c["diluent"] or [])
        if c["inputs"].get("x_o2_role") is None:
            scope["no per-test O2 fraction (outside the set)" + (", helium-diluted" if he else "")] += 1
        elif he:
            scope["helium dilution (refused by 13.1: F2 has no diluent input)"] += 1
        else:
            run.append(c)
    scope_cases = Counter()
    for c in target:
        if c not in run:
            n = len(at_risk_cases([c]))
            scope_cases["no O2" if c["inputs"].get("x_o2_role") is None else "helium, O2 known"] += n
    res, src_sha = run_f2(run, "validation_13_1", "runs/dev/data_all/work/transfer_sflame/cases_validation_inputs.jsonl", args.workers)
    byt = {c["test"]: c for c in run}
    rep = dict(protocol="PROTOCOL.md 13.1", release=F2, ignis=binary, cases_validation_inputs_sha256=src_sha,
               outcome_file_sha256=sha(OUT / "cases_validation_outcomes.jsonl"),
               baselines=dict(cfig=f"{CFIG_BASELINE.relative_to(REPO).as_posix()} {CFIG_BASELINE_SHA}",
                              sflame_train="Aalen-Johansen incidence of the s-Flame train target set, per cutoff"),
               constant_flow_tests=len(target), constant_flow_by_day=dict(Counter(c["day"] for c in target)),
               scope_refusals_tests=dict(scope), scope_refusals_cases_at_risk=dict(scope_cases),
               tests_run=len(run), tests_run_by_day=dict(Counter(c["day"] for c in run)), paths={})
    for path, (fc, refused, diag) in res.items():
        reasons = Counter(re.sub(r"-?\d+(\.\d+)?(e-?\d+)?", "#", v) for v in refused.values())
        burner = burner_of(diag, byt)
        rep["paths"][path] = dict(
            cases_at_risk=len(fc) + len(refused), answered_cases=len(fc), answered_tests=len({k[0] for k in fc}),
            answered_days=len({k[0][:5] for k in fc}), binary_refusal_cases=len(refused),
            binary_refusal_tests=len({k[0] for k in refused}), binary_refusal_reasons=dict(reasons),
            answered_tests_by_fuel=dict(Counter(byt[t]["inputs"]["fuel_species"] for t in {k[0] for k in fc})),
            p_late=dict(Counter(str(d.get("p_late")) for d in diag.values())),
            burner_250C=dict(answered_tests=len(burner), reach_before_planned_stop=sum(b["reaches_limit_before_stop"] for b in burner.values()),
                             equilibrium_above_250=sum(b["burner_equilibrium_c"] > 250 for b in burner.values())),
            scores=block_summary(fc, run, train_cases, byt, burner))
    ta.write_json(done, nn(rep))
    print(f"wrote {done.relative_to(REPO).as_posix()} (aggregates only)")
    return 0


def cmd_train_study(args):
    """Train days only: one row per case with conditions, endpoint, F2's table and the score contributions."""
    ta.selftest()
    ta.use_binary(args)
    allc = read_jsonl(OUT / "cases_train.jsonl")
    assert all(c["part"] == "train" for c in allc)
    cases = [c for c in allc if c["in_target_set"]]
    byt = {c["test"]: c for c in cases}
    res, src_sha = run_f2(cases, "f2_train", "runs/dev/data_all/work/transfer_sflame/cases_train.jsonl", args.workers)
    ks = at_risk_cases(cases)
    keys_all = [(k["test"], k["cutoff"]) for k in ks]
    b_cfig, b_inc = baselines_for(keys_all, cases)
    base_con = {}
    for n, f in (("cfig_incidence", b_cfig), ("sflame_train_incidence", b_inc)):
        c, k = contrib_on(keys_all, f, cases)
        base_con[n] = dict(zip(k, zip(*[c[m] for m in METRICS])))
    f2c, burners = {}, {}
    for path, (fc, refused, diag) in res.items():
        c, k = contrib_on(sorted(fc), fc, cases)
        f2c[path] = dict(zip(k, zip(*[c[m] for m in METRICS])))
        burners[path] = burner_of(diag, byt)
    rows = []
    for k in ks:
        key, c = (k["test"], k["cutoff"]), byt[k["test"]]
        row = dict(test=k["test"], day=k["day"], cutoff_s=k["cutoff"], inputs=c["inputs"], diluent=c["diluent"],
                   ambient_diluent=c["ambient_diluent"], burner_limit_c=c["burner_limit_c"], burner_t_start_c=c["burner_t_start_c"],
                   endpoint=c["outcome"]["endpoint"], script_account=c["outcome"]["tp_class"],
                   hit_250C_limit=c["outcome"]["tp_class"] == "script_burner_limit", horizons_s=HORIZONS,
                   contributions={n: dict(zip(METRICS, v[key])) for n, v in base_con.items()}, f2={})
        for path, (fc, refused, diag) in res.items():
            if key in fc:
                row["f2"][path] = dict(forecast=fc[key], contribution=dict(zip(METRICS, f2c[path][key])),
                                       burner=burners[path].get(k["test"]))
            else:
                row["f2"][path] = dict(refused=refused[key])
        rows.append(nn(row))
    write_jsonl(OUT / "train_days_cases.jsonl", rows)

    def agg(sel):
        rr = [r for r in rows if sel(r)]
        if not rr:
            return None
        out = dict(cases=len(rr), tests=len({r["test"] for r in rr}),
                   endpoints=dict(Counter((r["endpoint"]["allowed_causes"] or ["right_censored"])[0] for r in rr)))
        for n in ("cfig_incidence", "sflame_train_incidence"):
            out[n] = {m: math.fsum(r["contributions"][n][m] for r in rr) / len(rr) for m in METRICS}
        for path in ("nostop", "stop"):
            ans = [r for r in rr if "contribution" in r["f2"][path]]
            if ans:
                out[f"f2_{path}"] = dict(cases=len(ans), total_sum=math.fsum(r["f2"][path]["contribution"]["sum"] for r in ans),
                                         **{m: math.fsum(r["f2"][path]["contribution"][m] for r in ans) / len(ans) for m in METRICS})
                out[f"baselines_on_f2_{path}_cases"] = {n: {m: math.fsum(r["contributions"][n][m] for r in ans) / len(ans) for m in METRICS}
                                                        for n in ("cfig_incidence", "sflame_train_incidence")}
        return out
    groups = {"all": lambda r: True}
    for f in sorted({r["inputs"].get("fuel_species") or "?" for r in rows}):
        groups[f"fuel={f}"] = (lambda f: lambda r: (r["inputs"].get("fuel_species") or "?") == f)(f)
    for d in ("n2", "he"):
        groups[f"ambient={d}"] = (lambda d: lambda r: r["ambient_diluent"] == d)(d)
    groups["hit the 250 C limit (script reason)"] = lambda r: r["hit_250C_limit"]
    groups["did not hit the 250 C limit"] = lambda r: not r["hit_250C_limit"]
    for f in ("ethylene", "methane"):
        for h in (True, False):
            groups[f"fuel={f}, hit 250 C={h}"] = (lambda f, h: lambda r: r["inputs"].get("fuel_species") == f and r["hit_250C_limit"] == h)(f, h)
    for h in (True, False):
        groups[f"F2 heating law reaches 250 C before the stop={h}"] = (
            lambda h: lambda r: (r["f2"]["nostop"].get("burner") or {}).get("reaches_limit_before_stop") is h)(h)
    for cause in ("extinction", "operator_termination"):
        groups[f"endpoint={cause}"] = (lambda cause: lambda r: r["endpoint"]["allowed_causes"] == [cause])(cause)
    for cut in CUTOFFS:
        groups[f"cutoff={cut:g}"] = (lambda cut: lambda r: r["cutoff_s"] == cut)(cut)
    an = {g: agg(sel) for g, sel in groups.items()}
    tot = {p: an["all"][f"f2_{p}"]["total_sum"] for p in ("nostop", "stop")}
    for v in an.values():
        for p in ("nostop", "stop"):
            if v and f"f2_{p}" in v:
                v[f"f2_{p}"]["share_of_f2_total_sum"] = v[f"f2_{p}"]["total_sum"] / tot[p]
    ta.write_json(OUT / "train_study.json", nn(dict(cases_train_sha256=src_sha, anatomy=an)))
    print(f"wrote train_days_cases.jsonl ({len(rows)} rows) and train_study.json")
    return 0


# ---------------------------------------------------------------------------------------------
# The adapter's blend refusal (PROTOCOL 13.6, F4): a blend is not handed to a release without a blend law
# ---------------------------------------------------------------------------------------------
# Releases whose record carries a law for hydrogen/methane blends. None does: phys_sflame_f4.md types no blend law,
# and the forward, once hydrogen is admitted, would answer a blend through its oxygen-share rule, which fails on the
# hydrogen-rich bottle. So the lane's adapter does not send a blend prefix to any release; the case is counted as a
# refusal with this reason. The forward is not changed.
BLEND_LAW_RELEASES = frozenset()
BLEND_REFUSAL = ("adapter: the fuel is a blend and this release carries no blend law (PROTOCOL 13.6, "
                 "phys_sflame_f4.md 6; the lane does not hand blends to it)")


def is_blend_prefix(pp):
    fs = json.loads(Path(pp).read_text(encoding="utf-8"))["known_conditions"].get("fuel_species") or {}
    return "+" in str(fs.get("raw_value") or "")


def forward_cases(jobs, workers):
    """ta.forward_many with the adapter's blend refusal: jobs (key, release, prefix, out) whose prefix names a blend and
    whose release carries no blend law are not run; they come back as a condition-gate refusal artifact."""
    run, refused = [], {}
    for key, rel, pp, op in jobs:
        if rel not in BLEND_LAW_RELEASES and is_blend_prefix(pp):
            refused[key] = dict(status="refused", model_release_id=rel,
                                support=dict(admitted=False, gate="condition", reasons=[BLEND_REFUSAL]))
        else:
            run.append((key, rel, pp, op))
    out = ta.forward_many(run, workers) if run else {}
    out.update(refused)
    return out


# ---------------------------------------------------------------------------------------------
# PROTOCOL 13.5 (a): another release on the validation days of 13.1, once, beside F2
# ---------------------------------------------------------------------------------------------
def with_limit(prefix, c):
    """The prefix with the script's burner temperature limit of the test (an input: the script sets it
    before ignition, tools/wsp/ext/acme_script2.py), when the case inputs carry it."""
    lim = c.get("burner_limit_c")
    if lim is None:
        return prefix
    m = dict(prefix["known_conditions"]["pressure"])
    m.update(raw_value=float(lim), value_si=float(lim), raw_unit="degC", unit_si="degC",
             derivation=dict(m.get("derivation") or {}, method_id=a2.METHOD_ID,
                             notes="burnerTemperatureLimitC of the script parameter file (before ignition)"))
    p = dict(prefix, known_conditions=dict(prefix["known_conditions"], burner_limit_c=m))
    return ta.finish_prefix(p)


def run_release(cases, tag, src_rel, workers, release):
    """`release` on every at-risk case of `cases`, both paths, the prefixes of F2 plus the script's limit."""
    src = REPO / src_rel
    src_sha = sha(src)
    row_of = {json.loads(line)["test"]: i + 1 for i, line in enumerate(src.read_text(encoding="utf-8").splitlines()) if line.strip()}
    byt = {c["test"]: c for c in cases}
    out = {}
    for stop in (False, True):
        path = "stop" if stop else "nostop"
        jobs = []
        for k in at_risk_cases(cases):
            name = f"{k['test']}_c{k['cutoff']:g}_{path}.json"
            pp = OUT / tag / "prefix" / name
            ta.write_json(pp, with_limit(f2_prefix(byt[k["test"]], k["cutoff"], stop, src_sha, row_of[k["test"]], src=src_rel), byt[k["test"]]))
            jobs.append(((k["test"], k["cutoff"]), release, pp, OUT / tag / "artifact" / name))
        arts = forward_cases(jobs, workers)
        fc, refused = {}, {}
        for key, a in arts.items():
            if ta.admitted(a):
                fc[key] = ta.cif_of(a)
            else:
                refused[key] = f"{a['support'].get('gate')}: {'; '.join(a['support'].get('reasons', []))}"
        out[path] = (fc, refused)
    return out


def cmd_validation_release(args):
    """PROTOCOL 13.5 (a), run once: `--release` on the nitrogen tests of the validation days of 13.1, beside F2
    (its 13.1 forecasts, reproduced first) and the two incidence baselines. Aggregates only."""
    ta.selftest()
    binary = ta.use_binary(args)
    assert args.release and args.tag, "--release and --tag are required"
    done = OUT / f"validation_{args.tag}.json"
    if done.exists() and not args.again:
        sys.exit(f"{done} exists: the look is run once")
    vin = read_jsonl(OUT / "cases_validation_inputs.jsonl")
    vout = {r["test"]: r for r in read_jsonl(OUT / "cases_validation_outcomes.jsonl")}
    target = [c for c in vin if c["in_target_set"]]
    for c in target:
        c["outcome"] = dict(endpoint=vout[c["test"]]["endpoint"])
    train_cases = [c for c in read_jsonl(OUT / "cases_train.jsonl") if c["in_target_set"]]
    run = [c for c in target if c["inputs"].get("x_o2_role") is not None
           and not (c["ambient_diluent"] == "he" or "he" in (c["diluent"] or []))]
    byt = {c["test"]: c for c in run}
    src_rel = "runs/dev/data_all/work/transfer_sflame/cases_validation_inputs.jsonl"
    f2, _ = run_f2(run, "validation_13_1", src_rel, args.workers)
    # F2 must reproduce 13.1 before anything of the release is read
    want = {"nostop": 0.2619, "stop": 0.3698}
    for path, (fc, _r, _d) in f2.items():
        c2, k2 = contrib_on(sorted(fc), fc, run)
        m = means_by(c2, k2, lambda k: True)
        got = round(m["sum"]["case_mean"], 4)
        print(f"F2 on {path}: {got} on {m['cases']} cases (13.1: {want[path]} on 46)")
        assert got == want[path] and m["cases"] == 46, f"F2 does not reproduce 13.1 on {path}"
    new = run_release(run, f"validation_{args.tag}", src_rel, args.workers, args.release)
    rep = dict(protocol="PROTOCOL.md 13.5 (a)", release=args.release, reference=F2, ignis=binary,
               f2_reproduces_13_1=want, tests_run=len(run), tests_run_by_day=dict(Counter(c["day"] for c in run)), paths={})
    for path in ("nostop", "stop"):
        fc2, _ref2, diag2 = f2[path]
        fc3, ref3 = new[path]
        burner = burner_of(diag2, byt)
        common = sorted(set(fc2) & set(fc3))
        keys_all = common
        b_cfig, b_inc = baselines_for(keys_all, train_cases)
        con = {n: contrib_on(keys_all, f, run) for n, f in (("release", fc3), ("f2", fc2), ("cfig_incidence", b_cfig),
                                                               ("sflame_train_incidence", b_inc))}
        keys = con["release"][1]
        groups = {"all": lambda k: True}
        for fuel in sorted({byt[k[0]]["inputs"]["fuel_species"] for k in keys}):
            groups[f"fuel={fuel}"] = (lambda f: (lambda k: byt[k[0]]["inputs"]["fuel_species"] == f))(fuel)
        groups["reaches 250 C before the planned stop (F2 heating law)"] = lambda k: burner.get(k[0], {}).get("reaches_limit_before_stop") is True
        groups["does not reach 250 C before the planned stop"] = lambda k: burner.get(k[0], {}).get("reaches_limit_before_stop") is False
        reasons = Counter(re.sub(r"-?\d+(\.\d+)?(e-?\d+)?", "#", v) for v in ref3.values())
        rep["paths"][path] = dict(
            f2_answered_cases=len(fc2), release_answered_cases=len(fc3), common_cases=len(common),
            release_refused_cases=len(ref3), release_refused_tests=len({k[0] for k in ref3}), release_refusal_reasons=dict(reasons),
            answered_tests_by_fuel=dict(Counter(byt[t]["inputs"]["fuel_species"] for t in {k[0] for k in fc3})),
            scores={g: {n: means_by(con[n][0], keys, sel) for n in con} for g, sel in groups.items()})
    ta.write_json(done, nn(rep))
    print(f"wrote {done.relative_to(REPO).as_posix()} (aggregates only)")
    return 0


# ---------------------------------------------------------------------------------------------
# PROTOCOL 13.5, stage 2: the diluent and the second oxygen candidate as inputs (condition fields only)
# ---------------------------------------------------------------------------------------------
DILUENT_NAME = {"n2": "nitrogen", "he": "helium"}


def diluent_text(c):
    """`diluent_species` of the case: the one gas of the burner stream and the ambient, or a text that is not a
    row of the diluent table (both gases in the stream, or a stream and an ambient that differ), which the
    forward refuses."""
    stream = sorted(c.get("diluent") or [])
    amb = c.get("ambient_diluent") or "n2"
    if len(stream) == 1 and stream[0] == amb:
        return DILUENT_NAME.get(amb, amb)
    if len(stream) > 1:
        return "+".join(DILUENT_NAME.get(x, x) for x in stream)
    return f"{'+'.join(DILUENT_NAME.get(x, x) for x in stream) or 'none'} in {DILUENT_NAME.get(amb, amb)}"


def f3_prefix(c, cutoff, stop, src_sha, row, src):
    """F2's prefix, plus the script's burner limit, the diluent, and, for a test whose day names two oxygen
    fractions, the two candidates: x_o2_role, tad, zst the lower one, x_o2_role_b, tad_b, zst_b the higher."""
    p = f2_prefix(c, cutoff, stop, src_sha, row, src=src)
    kc = dict(p["known_conditions"])
    cand = c.get("inputs_by_o2_candidate") or {}
    if c["inputs"].get("x_o2_role") is None and len(cand) == 2:
        (xa, a), (xb, b) = sorted(cand.items(), key=lambda kv: float(kv[0]))
        for key, v, unit in (("x_o2_role", float(xa), "1"), ("tad", a["tad"], "K"), ("zst", a["zst"], "1"),
                             ("x_o2_role_b", float(xb), "1"), ("tad_b", b["tad"], "K"), ("zst_b", b["zst"], "1")):
            kc[key] = ta.measured(float(v), unit, key, row, src_sha, f"wsp.ext.sflame.cond.{key}",
                                  "candidate oxygen fraction of the day (experiment table) and its equilibrium (inputs)", path=src)
    x = blend_x(c)
    if x is not None:
        kc["fuel_blend_x"] = ta.measured(x, "1", "fuel_blend_x", row, src_sha, "wsp.ext.sflame.cond.fuel_blend_x",
                                         "mole fraction of the first-named species within the fuel of the burner stream "
                                         "(flows.burner_stream, from the bottle's gas code and the MFC settings)", path=src)
    kc["diluent_species"] = ta.measured(diluent_text(c), None, "diluent_species", row, src_sha, "wsp.ext.sflame.cond.diluent_species",
                                        "balance gas of the burner stream (MFC gas codes) and of the ambient (experiment table)",
                                        evidence="original_observation", path=src)
    p = ta.finish_prefix(dict(p, known_conditions=kc))
    return with_limit(p, c)


def run_f3(cases, tag, src_rel, workers, release, stops=(False, True)):
    src = REPO / src_rel
    src_sha = sha(src)
    row_of = {json.loads(line)["test"]: i + 1 for i, line in enumerate(src.read_text(encoding="utf-8").splitlines()) if line.strip()}
    byt = {c["test"]: c for c in cases}
    out = {}
    for stop in stops:
        path = "stop" if stop else "nostop"
        jobs = []
        for k in at_risk_cases(cases):
            name = f"{k['test']}_c{k['cutoff']:g}_{path}.json"
            pp = OUT / tag / "prefix" / name
            ta.write_json(pp, f3_prefix(byt[k["test"]], k["cutoff"], stop, src_sha, row_of[k["test"]], src_rel))
            jobs.append(((k["test"], k["cutoff"]), release, pp, OUT / tag / "artifact" / name))
        arts = forward_cases(jobs, workers)
        fc, refused = {}, {}
        for key, a in arts.items():
            if ta.admitted(a):
                fc[key] = ta.cif_of(a)
            else:
                refused[key] = f"{a['support'].get('gate')}: {'; '.join(a['support'].get('reasons', []))}"
        out[path] = (fc, refused)
    return out


def helium_block(fc, refused, cases, train_cases, byt):
    reasons = Counter(re.sub(r"-?\d+(\.\d+)?(e-?\d+)?", "#", v) for v in refused.values())
    blk = dict(answered_cases=len(fc), answered_tests=len({k[0] for k in fc}), refused_cases=len(refused),
               refused_tests=len({k[0] for k in refused}), refusal_reasons=dict(reasons),
               answered_tests_by_fuel=dict(Counter(byt[t]["inputs"]["fuel_species"] for t in {k[0] for k in fc})),
               refused_tests_by_fuel=dict(Counter(byt[t]["inputs"]["fuel_species"] for t in {k[0] for k in refused})))
    if fc:
        keys_all = sorted(fc)
        b_cfig, b_inc = baselines_for(keys_all, train_cases)
        con = {n: contrib_on(keys_all, f, cases) for n, f in (("release", fc), ("cfig_incidence", b_cfig), ("sflame_train_incidence", b_inc))}
        keys = con["release"][1]
        groups = {"all": lambda k: True}
        for fuel in sorted({byt[k[0]]["inputs"]["fuel_species"] for k in keys}):
            groups[f"fuel={fuel}"] = (lambda f: (lambda k: byt[k[0]]["inputs"]["fuel_species"] == f))(fuel)
        groups["two oxygen candidates (mixture)"] = lambda k: byt[k[0]]["inputs"].get("x_o2_role") is None
        groups["one oxygen fraction"] = lambda k: byt[k[0]]["inputs"].get("x_o2_role") is not None
        blk["scores"] = {g: {n: means_by(con[n][0], keys, sel) for n in con} for g, sel in groups.items()}
    return blk


def is_helium(c):
    return c["ambient_diluent"] == "he" or "he" in (c["diluent"] or [])


def cmd_validation_helium(args):
    """PROTOCOL 13.5, stage 2, run once: `--release` on the helium-diluted validation tests (an oxygen fraction,
    or two candidates answered as the mixture), beside the two incidence baselines; and the same release on the
    helium-diluted train tests (study data). Aggregates only."""
    ta.selftest()
    binary = ta.use_binary(args)
    assert args.release and args.tag
    done = OUT / f"validation_{args.tag}.json"
    if done.exists() and not args.again:
        sys.exit(f"{done} exists: the look is run once")
    train_all = [c for c in read_jsonl(OUT / "cases_train.jsonl") if c["in_target_set"]]
    vin = read_jsonl(OUT / "cases_validation_inputs.jsonl")
    vout = {r["test"]: r for r in read_jsonl(OUT / "cases_validation_outcomes.jsonl")}
    vt = [c for c in vin if c["in_target_set"] and is_helium(c)]
    for c in vt:
        c["outcome"] = dict(endpoint=vout[c["test"]]["endpoint"])
    tt = [c for c in train_all if is_helium(c)]
    rep = dict(protocol="PROTOCOL.md 13.5 stage 2", release=args.release, ignis=binary,
               validation_tests=len(vt), validation_tests_by_fuel=dict(Counter(c["inputs"]["fuel_species"] for c in vt)),
               train_tests=len(tt), train_tests_by_fuel=dict(Counter(c["inputs"]["fuel_species"] for c in tt)), paths={})
    v = run_f3(vt, f"validation_{args.tag}", "runs/dev/data_all/work/transfer_sflame/cases_validation_inputs.jsonl", args.workers, args.release)
    t = run_f3(tt, f"train_{args.tag}", "runs/dev/data_all/work/transfer_sflame/cases_train.jsonl", args.workers, args.release)
    for path in ("nostop", "stop"):
        rep["paths"][path] = dict(validation=helium_block(*v[path], vt, train_all, {c["test"]: c for c in vt}),
                                  train_study=helium_block(*t[path], tt, train_all, {c["test"]: c for c in tt}))
    ta.write_json(done, nn(rep))
    print(f"wrote {done.relative_to(REPO).as_posix()} (aggregates only)")
    return 0


SPECIES_KEY = {v: k for k, v in SPECIES_TEXT.items()}


def blend_x(c):
    """For a blend `a+b`: the mole fraction of `a` within the fuel of the burner stream; None for one fuel."""
    sp = (c["inputs"].get("fuel_species") or "").split("+")
    if len(sp) != 2:
        return None
    fs = (c.get("flows") or {}).get("burner_stream") or {}
    a, b = (fs.get(SPECIES_KEY.get(x.strip(), x.strip()), 0.0) for x in sp)
    return a / (a + b) if a + b > 0 else None


def oxygen_shares(c, sigma=(("ethylene", 3.422), ("methane", 3.989), ("hydrogen", 7.936))):
    """The share of the oxygen demand of each species of a blend (the dossier's rule, A3), for reports only:
    the forward forms it itself from fuel_blend_x and its own fuel.sigma."""
    x = blend_x(c)
    if x is None:
        return None
    sig, mw = dict(sigma), {"ethylene": 28.054, "methane": 16.043, "hydrogen": 2.016}
    a, b = [t.strip() for t in c["inputs"]["fuel_species"].split("+")]
    y_a = x * mw[a] / (x * mw[a] + (1 - x) * mw[b])
    w_a = y_a * sig[a] / (y_a * sig[a] + (1 - y_a) * sig[b])
    return {a: w_a, b: 1 - w_a}


def cmd_admission(args):
    """Admission of `--release` on every constant-flow test of the s-Flame TRAIN and VALIDATION days (inputs only:
    the F3 prefixes at cutoff 1 s, no planned stop; no outcome is read). Counts per part, fuel and diluent."""
    ta.selftest()
    ta.use_binary(args)
    assert args.release and args.tag
    rows = []
    for part, f in (("train", "cases_train.jsonl"), ("validation", "cases_validation_inputs.jsonl")):
        src = f"runs/dev/data_all/work/transfer_sflame/{f}"
        for i, c in enumerate(read_jsonl(OUT / f)):
            if not c["in_target_set"]:
                continue
            pp = OUT / f"admission_{args.tag}" / "prefix" / f"{c['test']}.json"
            ta.write_json(pp, f3_prefix(c, 1.0, False, sha(REPO / src), i + 1, src))
            rows.append((part, c, pp))
    jobs = [((part, c["test"]), args.release, pp, OUT / f"admission_{args.tag}" / "artifact" / pp.name) for part, c, pp in rows]
    arts = forward_cases(jobs, args.workers)
    table = defaultdict(lambda: dict(tests=0, answered=0, refused=Counter()))
    for part, c, _ in rows:
        a = arts[(part, c["test"])]
        k = f"{part} | {c['inputs']['fuel_species']} | {diluent_text(c)} | {'O2 known' if c['inputs'].get('x_o2_role') is not None else 'two O2 candidates' if len(c.get('inputs_by_o2_candidate') or {}) == 2 else 'no O2'}"
        table[k]["tests"] += 1
        if ta.admitted(a):
            table[k]["answered"] += 1
        else:
            table[k]["refused"][re.sub(r"-?\d+(\.\d+)?(e-?\d+)?", "#", "; ".join(a["support"].get("reasons", [])))[:140]] += 1
    rep = {k: dict(v, refused=dict(v["refused"])) for k, v in sorted(table.items())}
    ta.write_json(OUT / f"admission_{args.tag}.json", dict(release=args.release, rows=rep))
    for k, v in rep.items():
        print(f"{k}: {v['tests']} tests, {v['answered']} answered; refused {v['refused']}")
    return 0


# ---------------------------------------------------------------------------------------------
# Second endpoint version (v2): the endpoint rule of docs/wsp/feedback/sflame_endpoints_v2.md
# ---------------------------------------------------------------------------------------------
V2_VERSION = "sflame.endpoint.v2"
RAD_ON_V = 0.15          # radiometer rise above its pre-ignition level that counts as "on" (flame_design ONSET_RISE_V)
RAD_HELD_FRAC = 0.85     # ... and not falling: at least this fraction of its maximum over the 2 s before the cut
BURNER_HOT_K = 20.0      # burner thermocouple at least this much above its value at onset ...
BURNER_COOL_K_S = 0.5    # ... and not cooling faster than this over the 2 s before the cut
BURNER_FED_LAG_S = 1.5   # hydrogen-bearing detector stops: burner peak this long after the fuel readback is off = still burning


def burner_temp_c(d):
    """Burner thermocouple (type K) in deg C: cold junction (AD590, V/0.00295 - 273.15) + V_tc / 41 uV/K."""
    return (d["TCCJ"] / 0.00295 - 273.15) + d["TCBurner"] * 1e6 / TK_UV_PER_K


def h2_bearing(fuel_species):
    return bool(fuel_species) and "hydrogen" in fuel_species


def measure_v2(test, have, fuel_species):
    """v1 (flame_design.measure, unchanged) plus the witnesses and the v2 label. Times: s since the v1 onset."""
    import numpy as np
    m, cls, reason, ep1, why1 = measure_test(test, have)
    out = dict(test=test, v1=dict(status=m.get("status"), label=m.get("label_sensor"), end_s=m.get("end_s"),
                                  pmt_end_s=m.get("pmt_end_s"), flow_change_s=m.get("flow_change_s")),
               script_account=cls, script_reason=reason, fuel_species=fuel_species, rule_version=V2_VERSION)
    fn = file_names(test, have)
    if m.get("status") != "ok" or not fn["setpoints"] or m.get("label_sensor") not in ("extinction", "operator_termination"):
        out.update(label=m.get("label_sensor"), end_s=m.get("end_s"), endpoint=ep1, branch="v1 (no endpoint or no command log)")
        return out
    hdr, tt, d = fdm.read_data(RAW / fn["data"])
    t0 = m["onset_clock"]
    x = tt - t0
    # the commanded test flow and the cut: the setpoint in force when the constant-flow stage starts (the TP
    # 'Constant flow' stamp + 0.999 s, its resolution; without a TP file the v1 reference), the first time the
    # command equals it, and the first departure from it after that
    ts, vs = fdm.read_setpoints(RAW / fn["setpoints"], tt[0])
    xs = ts - t0
    st = fdm.read_tp(RAW / fn["tp"])[0] if fn["tp"] else {}
    if "Constant flow" in st:
        cf = st["Constant flow"] + (86400 if st["Constant flow"] < tt[0] - 40000 else 0) - t0 + 0.999
    else:
        cf = m["constant_stage_begins_s"]
    nz = np.where(np.any(vs > 0.01, axis=1))[0]
    zero_after = np.where((np.arange(len(xs)) > (nz[0] if len(nz) else 0)) & np.all(vs <= 0.01, axis=1))[0] if len(nz) else []
    ref = min(cf, float(xs[zero_after[0]]) - 1e-6) if len(zero_after) else cf
    i_lv = np.where(xs <= ref)[0]
    level = vs[i_lv[-1]] if len(i_lv) else vs[-1]
    tol = np.maximum(0.02 * np.abs(level), 0.005)
    same = np.all(np.abs(vs - level) <= tol, axis=1) & np.any(vs > 0.01, axis=1)
    reach = np.where(same)[0]
    t_reach = float(xs[reach[0]]) if len(reach) else float(ref)
    dep = np.where((xs > t_reach) & ~np.all(np.abs(vs - level) <= tol, axis=1))[0]
    t_c = float(xs[dep[0]]) if len(dep) else NAN
    if t_c != t_c:  # no departure in the log: the record ends burning; v1 stands
        out.update(label=m.get("label_sensor"), end_s=m.get("end_s"), endpoint=ep1, branch="v1 (no commanded cut)")
        return out

    def at(y, s):
        return float(y[min(np.searchsorted(x, s), len(x) - 1)])
    # W_rad: the radiometer sum is on and not falling at the cut
    rad = fdm.radiometer_channels(d)
    w_rad, r_cut, r_max2 = False, NAN, NAN
    if rad:
        rs = sum(d[c] for c in rad)
        pre = (tt > m["ignition_stamp_clock"] - 15) & (tt < m["ignition_stamp_clock"] - 3)
        base = float(np.median(rs[pre])) if pre.sum() >= 10 else float(np.median(rs[: max(10, int(0.5 / m["dt_s"]))]))
        rsm = fdm.smooth(tt, rs, 0.3) - base
        seg = rsm[(x >= t_c - 2.0) & (x <= t_c - 0.25)]
        r_max2 = float(seg.max()) if len(seg) else NAN
        r_cut = at(rsm, t_c - 0.25)
        w_rad = r_cut >= RAD_ON_V and r_cut >= RAD_HELD_FRAC * r_max2
    # W_tc: the burner is hot and not cooling at the cut
    tb = fdm.smooth(tt, burner_temp_c(d), 0.3)
    rise = at(tb, t_c) - at(tb, 0.0)
    rate2 = (at(tb, t_c) - at(tb, t_c - 2.0)) / 2.0
    w_tc = rise >= BURNER_HOT_K and rate2 >= -BURNER_COOL_K_S
    pmt_end = m.get("pmt_end_s", NAN)
    # for variant B: the fuel readback falls to 5 % of its level before the cut; when the burner peaks after that
    fuel = fdm.smooth(tt, d["MFCA3FuelFlow"], 0.2)
    lv = fuel[(x > t_c - 1.5) & (x < t_c - 0.2)]
    lv = float(np.median(lv)) if len(lv) else NAN
    off = np.where((x > t_c) & (fuel < 0.05 * lv))[0] if lv == lv and lv > 0.05 else []
    t_off = float(x[off[0]]) if len(off) else NAN
    win = (x > t_c - 2.0) & (x < t_c + 15.0)
    t_peak = float(x[win][np.argmax(tb[win])]) if win.any() else NAN
    out["witnesses"] = dict(t_reach_s=t_reach, t_cut_s=t_c, radiometer_at_cut_v=r_cut, radiometer_max_2s_before_v=r_max2,
                            w_rad=w_rad, burner_rise_since_onset_k=rise, burner_rate_2s_before_k_s=rate2, w_tc=w_tc,
                            pmt_end_s=pmt_end, t_fuel_off_s=t_off, t_burner_peak_s=t_peak,
                            burner_peak_after_fuel_off_s=t_peak - t_off if t_off == t_off else NAN)
    h2 = h2_bearing(fuel_species)
    pmt_first = pmt_end == pmt_end and pmt_end < t_c - fdm.LABEL_MARGIN_S
    ext_end = min(pmt_end, t_c) if pmt_end == pmt_end else t_c
    # variant A (radiometer only), kept beside the rule for comparison
    out["variant_a_label"] = ("operator_termination" if w_rad else "extinction") if cls == "script_extinction_detected" else None
    if cls == "script_extinction_detected":
        lag = out["witnesses"]["burner_peak_after_fuel_off_s"]
        if h2 and w_rad and lag == lag:
            if lag >= BURNER_FED_LAG_S:
                lab, end, br = "operator_termination", t_c, "R2h: detector stop; radiometer on at the cut and the burner kept heating after the fuel was off"
            else:
                lab, end, br = "extinction", ext_end, "R2h: script extinction stands (radiometer on, but the burner peaks with the fuel off)"
        elif w_rad:
            lab, end, br = "operator_termination", t_c, "R2: detector stop of a flame the radiometer still sees at the cut"
        else:
            lab, end, br = "extinction", ext_end, "R2: script extinction stands"
    elif cls in ("script_burner_limit", "script_scripted_end"):
        if h2 and (w_rad or w_tc):
            lab, end, br = "operator_termination", t_c, "R1: hydrogen-bearing flame alive at the scripted cut (radiometer or burner)"
        elif pmt_first and not h2:
            lab, end, br = "extinction", pmt_end, "R1: photomultipliers dark before the scripted cut"
        elif pmt_first and h2:
            lab, end, br = "extinction", pmt_end, "R1: hydrogen-bearing, no witness of a flame at the cut"
        else:
            lab, end, br = "operator_termination", t_c, "R1: scripted cut"
    else:
        if pmt_first and not (h2 and (w_rad or w_tc)):
            lab, end, br = "extinction", pmt_end, "R3: no script account; photomultipliers dark before the cut"
        else:
            lab, end, br = "operator_termination", t_c, "R3: no script account; flame alive at the cut"
    ep = dict(censoring="event", last_confirmed_active_s=end, first_terminal_s=end, allowed_causes=[lab])
    out.update(label=lab, end_s=end, endpoint=ep, branch=br)
    return out


def endpoints_v2_for(rows_inputs, tests):
    out = []
    for i in rows_inputs:
        r = measure_v2(i["test"], tests[i["test"]], i["fuel_species"])
        r.update(day=i["day"], part=i["part"], constant_flow=i["constant_flow"])
        out.append(nn(r))
    return out


ADJUDICATED = {  # docs/wsp/research/phys_sflame_f4.md, section 2.3: (label, time s)
    "20155B1": ("operator_termination", 9.7), "20190G1": ("operator_termination", 9.9), "19352D5": ("operator_termination", 17.4),
    "19352D6": ("operator_termination", 20.9), "20024M1": ("operator_termination", 46.8), "20024N3": ("operator_termination", 47.1),
    "20013C3": ("operator_termination", 47.9), "20013C4": ("operator_termination", 47.8), "20013C5": ("operator_termination", 48.0),
    "20155A2": ("extinction", 2.4), "20190A1": ("extinction", 2.6), "19352D4": ("extinction", 4.9), "20006C1": ("extinction", 26.5),
    "20155H1": ("operator_termination", 7.3), "20024N2": ("operator_termination", 15.0), "19308H4": ("extinction", 8.2)}


def cmd_endpoints_v2(_args):
    """Train and validation days (both are development data since PROTOCOL 13.6). Writes the v2 files beside v1."""
    parts = split()
    tests = list_tests()
    inputs = read_jsonl(OUT / "inputs.jsonl")
    assert all(parts.get(i["day"]) in ("train", "validation") for i in inputs)
    rows = endpoints_v2_for(inputs, tests)
    write_jsonl(OUT / "endpoints_train_v2.jsonl", [r for r in rows if r["part"] == "train"])
    write_jsonl(OUT / "cases_validation_outcomes_v2.jsonl", [r for r in rows if r["part"] == "validation"])
    # cases_*_v2: the v1 case rows with the v2 outcome
    by = {r["test"]: r for r in rows}
    for src, dst in (("cases_train.jsonl", "cases_train_v2.jsonl"), ("cases_validation_inputs.jsonl", "cases_validation_v2.jsonl")):
        cc = read_jsonl(OUT / src)
        for c in cc:
            r = by[c["test"]]
            c["outcome"] = dict(endpoint=r["endpoint"], label=r["label"], end_s=r["end_s"], branch=r["branch"],
                                script_account=r["script_account"], rule_version=V2_VERSION, time_origin=ta.FD_TIME_ORIGIN)
        write_jsonl(OUT / dst, cc)
    v1 = {r["test"]: r for r in read_jsonl(OUT / "endpoints_train.jsonl") + read_jsonl(OUT / "cases_validation_outcomes.jsonl")}
    rep = dict(rule=V2_VERSION, tests=len(rows), by_partition=dict(Counter(r["part"] for r in rows)),
               branches=dict(Counter(r["branch"] for r in rows)), adjudicated={}, changed_beyond_adjudicated=[])
    for t, (lab, tm) in ADJUDICATED.items():
        r = by[t]
        rep["adjudicated"][t] = dict(adjudicated=[lab, tm], v1=[v1[t]["label_sensor"], v1[t]["end_s"]], v2=[r["label"], r["end_s"]],
                                     label_reproduced=r["label"] == lab,
                                     time_diff_s=(r["end_s"] - tm) if r["end_s"] is not None else None, branch=r["branch"],
                                     witnesses=r.get("witnesses"))
    for r in rows:
        a = v1[r["test"]]
        if r["test"] in ADJUDICATED:
            continue
        if r["label"] != a["label_sensor"] or (r["end_s"] is not None and a["end_s"] is not None and abs(r["end_s"] - a["end_s"]) > 0.5):
            rep["changed_beyond_adjudicated"].append(dict(test=r["test"], part=r["part"], fuel=r["fuel_species"],
                                                          constant_flow=r["constant_flow"], script=r["script_account"],
                                                          v1=[a["label_sensor"], a["end_s"]], v2=[r["label"], r["end_s"]],
                                                          branch=r["branch"], witnesses=r.get("witnesses")))
    ta.write_json(OUT / "endpoints_v2_report.json", nn(rep))
    print(json.dumps(nn({k: v for k, v in rep.items() if k != "changed_beyond_adjudicated"}), indent=1)[:6000])
    print("changed beyond the 16:", len(rep["changed_beyond_adjudicated"]))
    return 0


def cmd_final_run_build(args):
    """For the single final run only: inputs and v2 endpoints of the TEST days, by the same code, into their own
    files. Refuses to run without --final-run (this agent has not run it; no test-day file has been opened)."""
    if not args.final_run:
        sys.exit("final-run-build opens the test days: give --final-run, and only in the registered final run")
    parts, days = split(), table_days()
    tests = list_tests()
    rows = [inputs_of(t, tests[t], parts, days) for t in sorted(tests) if part_of(t, parts) == "test"]
    assign_o2(rows)
    for r in rows:
        finish_inputs(r)
    write_jsonl(OUT / "final_run/inputs_test.jsonl", [nn(r) for r in rows])
    cases = [case_row(r) for r in rows]
    write_jsonl(OUT / "final_run/cases_test_inputs.jsonl", cases)
    write_jsonl(OUT / "final_run/cases_test_outcomes_v2.jsonl", endpoints_v2_for([nn(r) for r in rows], tests))
    print(f"wrote final_run/: {len(rows)} test-day tests (inputs, and v2 outcomes kept apart)")
    return 0


# ---------------------------------------------------------------------------------------------
# Case version 3 (PROTOCOL 13.8, after the final run with every partition seen; in-sample): two rules on the CASES,
# both known before ignition; endpoints as version 2; versions 1 and 2 untouched
# ---------------------------------------------------------------------------------------------
V3_VERSION = "sflame.cases.v3"


def ramp_time(inp_row):
    v = ((inp_row.get("script") or {}).get("ramp") or {}).get("ignitionToTestRampTimeSec")
    return v if v is not None and v == v else 0.0


def named_o2(atmosphere):
    """The oxygen fractions of a day in the order the experiment table names them ('~30% and ~21% O2 with He')."""
    return [float(x) / 100 for x in re.findall(r"~(\d+)% *(?:and|O2)", (atmosphere or "") + " O2")]


def o2_by_block(inp_row):
    """Rule 2: on a day whose table names two oxygen fractions, the fraction of the test's pressure block, block i
    taking the i-th named fraction (block 0 before the first vent and refill). Determined only when the day has as many
    pressure blocks as named fractions (otherwise the sequence of atmospheres is not seen whole and the mixture
    stays): returns the fraction, or None."""
    x = inp_row.get("x_o2") or {}
    cands = x.get("candidates") or []
    if len(cands) != 2 or x.get("value") is not None:
        return None
    named = named_o2((inp_row.get("table") or {}).get("atmosphere"))
    blocks, b = x.get("blocks_in_day"), x.get("pressure_block")
    if len(named) != 2 or blocks != len(named) or b is None or not (0 <= b < len(named)):
        return None
    return named[b]


def case_v3(c, inp_row, oxygen=True):
    """A case row of version 3: rule 1 (a non-zero ignition-to-test ramp time is not a constant-flow test) and, with
    `oxygen`, rule 2 (the oxygen fraction of the pressure block). Endpoints untouched (the caller's version)."""
    c = json.loads(json.dumps(c))
    rt = ramp_time(inp_row)
    if rt != 0.0:
        c["in_target_set"] = False
        c.setdefault("v3", {})["excluded"] = f"ignition-to-test ramp of {rt:g} s (rule 1)"
    if oxygen:
        x = o2_by_block(inp_row)
        # a case without the candidates' equilibria (no fuel in the burner stream) has no inputs to replace
        alt = (c.get("inputs_by_o2_candidate") or {}).get(f"{x:.2f}") if x is not None else None
        if alt is not None:
            c["inputs"] = dict(c["inputs"], x_o2_role=x, tad=alt["tad"], zst=alt["zst"])
            c.setdefault("v3", {})["x_o2_by_block"] = dict(value=x, block=inp_row["x_o2"]["pressure_block"])
    c.setdefault("v3", {})["rule_version"] = V3_VERSION + ("" if oxygen else " (rule 1 only)")
    return c


def cases_v3(cases, inputs, oxygen=True):
    by = {r["test"]: r for r in inputs}
    return [case_v3(c, by[c["test"]], oxygen) for c in cases]


def cmd_cases_v3(_args):
    """Writes the version-3 case rows beside versions 1 and 2, for the development days (this directory) and for the
    test days as the final step built them (runs/final/gen7/final/transfer_sflame/, whose "validation" files are the
    test days), and counts what each rule changes per partition."""
    final = REPO / "runs/final/gen7/final/transfer_sflame"
    rep = {}
    for root, sets in ((OUT, (("train", "cases_train_v2.jsonl"), ("validation", "cases_validation_v2.jsonl"))),
                       (final, (("test", "cases_validation_inputs.jsonl"),))):
        if not (root / "inputs.jsonl").exists():
            continue
        inputs = read_jsonl(root / "inputs.jsonl")
        for part, f in sets:
            cases = read_jsonl(root / f)
            v3 = cases_v3(cases, inputs)
            write_jsonl(root / f"cases_{part}_v3.jsonl", v3)
            removed = [c["test"] for c, d in zip(cases, v3) if c["in_target_set"] and not d["in_target_set"]]
            by_block = [d for d in v3 if d.get("v3", {}).get("x_o2_by_block")]
            two = [c for c in cases if c["in_target_set"] and c["inputs"].get("x_o2_role") is None
                   and len(c.get("inputs_by_o2_candidate") or {}) == 2]
            rep[part] = dict(target_tests=sum(c["in_target_set"] for c in cases), removed_by_ramp=len(removed), removed_tests=removed,
                             two_candidate_target_tests=len(two),
                             oxygen_by_block_target_tests=sum(d["in_target_set"] for d in by_block),
                             oxygen_by_block_by_day={d: dict(Counter(str(x["v3"]["x_o2_by_block"]["value"]) for x in by_block
                                                                     if x["day"] == d and x["in_target_set"]))
                                                     for d in sorted({x["day"] for x in by_block})},
                             mixture_stays_target_tests=sum(1 for c in two if not o2_by_block(next(r for r in inputs if r["test"] == c["test"]))))
    ta.write_json(OUT / "cases_v3_report.json", nn(rep))
    print(json.dumps(nn(rep), indent=1))
    return 0


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    sub = ap.add_subparsers(dest="cmd", required=True)
    for n, fn in (("tad-check", cmd_tad_check), ("inputs", cmd_inputs), ("endpoints", cmd_endpoints), ("build", cmd_build),
                  ("baseline", cmd_baseline), ("f2-train", cmd_f2_train),
                  ("validation-f2", cmd_validation_f2), ("train-study", cmd_train_study),
                  ("validation-release", cmd_validation_release), ("validation-helium", cmd_validation_helium),
                  ("admission", cmd_admission),
                  ("endpoints-v2", cmd_endpoints_v2), ("final-run-build", cmd_final_run_build),
                  ("cases-v3", cmd_cases_v3)):
        p = sub.add_parser(n)
        p.set_defaults(fn=fn)
        p.add_argument("--ignis")
        p.add_argument("--workers", type=int, default=12)
        p.add_argument("--again", action="store_true", help="validation-f2: rewrite an existing result (cached forwards)")
        p.add_argument("--final-run", action="store_true", help="final-run-build: the registered final run only")
        p.add_argument("--release", help="validation-release: the release of the look")
        p.add_argument("--tag", help="validation-release: the name of its output (validation_<tag>.json)")
    a = ap.parse_args()
    OUT.mkdir(parents=True, exist_ok=True)
    return a.fn(a)


if __name__ == "__main__":
    sys.exit(main())
