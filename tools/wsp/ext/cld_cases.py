#!/usr/bin/env python3
"""ACME CLD Flame (PSI-21, coflow laminar diffusion flame): forecast cases and their score.

Preparation for a blind, zero-shot law. This tool fits nothing except the constants of the named baselines on
train days (subcommand `baselines`), changes no model and opens no file of a test day.

Partitions are the frozen day groups of docs/wsp/atlas/splits/PSI-21.json.
  train       inputs and outcomes are measured and written to cases_train.jsonl
  validation  inputs -> cases_validation_inputs.jsonl; outcomes -> cases_validation_outcomes.jsonl, written
              by script and never printed (the console reports availability counts only)
  test        no file of a test day is opened; counts come from the file catalogue
              (runs/dev/data_all/work/atlas/files.jsonl)

Sources per test point T (runs/dev/data_all/raw/PSI-21/Raw Data/):
  `T.txt` or `as runT.txt`   script parameters, written before the test (inputs)
  `T_data.csv`               125 Hz channels: three MFC readbacks, PMT1 (OH*, 310 nm), PMT2 (broadband),
                             PMT3 (CH*, 430 nm), five radiometers, thermocouples, pressures (outcomes; the
                             pre-ignition seconds also give the chamber pressure reading, an input)
  `T_mfcSetpoints.csv`       command log of the three MFC setpoints (V); gives the ignition command time
  `T_TP.txt`                 stage stamps at 1 s; used for cross-checks only
  the experiment table       fuel bottle of the test day (inputs)

Targets (definitions in docs/wsp/feedback/cld_feasibility.md):
  loss        flame lost while fuel still flows: the OH* photomultiplier returns to its dark level more than
              LABEL_MARGIN_S before the fuel flow is commanded off. Event time on the clock since flame onset.
              Forecast: P(loss by h) at HORIZONS_S. Score: integrated Brier over HORIZONS_S on the cases at
              risk (a case leaves when its fuel is commanded off with the flame burning), trapezoid as in
              tools/wsp/ext/transfer_acme.py; and the Brier of the yes/no (loss before the fuel is off).
  radiometer  log10 of the radiometer sum above its pre-ignition level (V), median over the first
              constant-flow stage, for flames that burn through it. Forecast: quantiles at 5, 25, 50, 75,
              95 %. Score: CRPS approximated by the mean pinball loss over the five levels (times 2).

    python tools/wsp/ext/cld_cases.py build            # train + validation case files
    python tools/wsp/ext/cld_cases.py counts           # counts by partition (test days: catalogue only)
    python tools/wsp/ext/cld_cases.py baselines        # baselines on train, through score()
"""
import argparse
import json
import math
import re
import sys
from collections import Counter, defaultdict
from pathlib import Path

import numpy as np

REPO = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(Path(__file__).resolve().parent))
import flame_design as fd  # noqa: E402  (readers: read_data, read_setpoints, read_tp, smooth, first_sustained)

INV = "PSI-21"
RAW = REPO / "runs/dev/data_all/raw" / INV / "Raw Data"
TABLE = REPO / "runs/dev/data_all/raw" / INV / "Experimental table" / "PSI-21_Experimental table_CLD Flame.csv"
SPLIT = REPO / "docs/wsp/atlas/splits/PSI-21.json"
CATALOGUE = REPO / "runs/dev/data_all/work/atlas/files.jsonl"
OUT = REPO / "runs/dev/data_all/work/transfer_cld"
NAN = float("nan")

# ---------------------------------------------------------------------------------------------
# Typed thresholds of the measurement. Set on train tests before any validation outcome existed.
# ---------------------------------------------------------------------------------------------
PMT_LIT_V = 0.05          # PMT1 (OH*) above its armed dark level: flame present
PMT_END_V = 0.02          # ... back below this excess: dark
PMT_HOLD_LIT_S = 0.2      # onset: lit for this long
PMT_HOLD_END_S = 0.5      # end: dark for this long
LABEL_MARGIN_S = 0.5      # dark this much before the fuel-off command: loss under flow
ONSET_WINDOW_S = (-1.0, 12.0)   # onset searched in this window around the ignition command
RAD_PRE_S = (-8.0, -1.0)  # radiometer baseline: these seconds around the ignition command
RAD_SKIP_S = 2.0          # radiometer level: skip the first seconds of the constant-flow stage
RAD_MIN_S = 3.0           # ... and need at least this much burning stage after the skip
RAD_FLOOR_V = 0.005       # excess below this is taken at the floor before the log
MATCH_RAMP_OVERHEAD_S = 1.87  # script behaviour, not a parameter: with a coflow matching ramp the fuel-off
#                               command comes 1.87 s later than the stage sum (train tests without a loss and
#                               without velocity ramps, any ramp length; read from the command logs)
SCHEDULE_KEYS = ["igniterDelayBeforeRetractMs", "flowTimeoutSec", "primingAndPreIgnitionDurationSec",
                 "stabilizationTime1Sec", "dilutionTimeSec", "stabilizationTime2Sec", "matchingCoflowRampTimeSec",
                 "stabilizationTime3Sec", "fuelAndCoflowRampTimeSec", "constantFlowTimeSec",
                 "velocityAirIgnitionMPerSec", "velocityAirFinalMPerSec", "velocityMixIgnitionMPerSec",
                 "velocityMixFinalMPerSec", "concentrationInitial", "concentrationFinal", "rampCount"]
HORIZONS_S = [1.0, 2.0, 3.0, 5.0, 7.0, 10.0, 15.0, 20.0, 30.0, 45.0, 60.0, 90.0, 120.0]
QUANTILES = [0.05, 0.25, 0.5, 0.75, 0.95]
PTA_PSIA_PER_V, PTA_ZERO_V = 36.0, 0.508   # two-point reading of the ACME chamber transducer
#                                            (docs/wsp/feedback/acme_raw_signals_train.md); not a calibration
PSIA_PA = 6894.757

FUELS = {"CH4": "methane", "C2H4": "ethylene"}
MW = {"methane": 16.043, "ethylene": 28.054, "o2": 31.999, "n2": 28.013}
NU_O2 = {"methane": 2.0, "ethylene": 3.0}
X_O2_COFLOW = 0.21        # synthetic air coflow, 21/79 O2/N2 by volume (data interpretation document, sec. 3)
FUEL_TUBE_ID_M = 2.18e-3  # inner diameter of the fuel tube; final report gives 2.1 (design) and 2.2 mm
COFLOW_ID_M = 25.0e-3     # inner diameter of the coflow annulus (final report)
LFL = {"methane": 0.050, "ethylene": 0.027}   # lower flammability limits in air (Coward and Jones 1952, USBM 503)
ROPER_CONST = 1330.0      # s/m^2 (Roper's circular-port correlation, SI)

SCRIPT_NUM = ["preIgnitionDataTimeSec", "postFlameDataTimeSec", "igniterIgnitionPowerV", "igniterIgnitionDurationMs",
              "igniterDelayBeforeRetractMs", "flowTimeoutSec", "nitrogenPrimingDurationSec",
              "primingAndPreIgnitionDurationSec", "stabilizationTime1Sec", "dilutionTimeSec", "stabilizationTime2Sec",
              "matchingCoflowRampTimeSec", "stabilizationTime3Sec", "fuelAndCoflowRampTimeSec", "constantFlowTimeSec",
              "velocityAirIgnitionMPerSec", "velocityAirFinalMPerSec", "velocityMixIgnitionMPerSec",
              "velocityMixFinalMPerSec", "concentrationInitial", "concentrationFinal", "rampCount", "rampSizeMPerSec",
              "rampTimeSec", "rampConstantFlowTimeSec", "fiberIgnitionPosition", "fiberTestPosition",
              "fiberSegmentDistance", "fiberVelocity", "accelerationCoflowMPerSec2"]
SCRIPT_BOOL = ["useIgnitionDetection", "extinctionDetectionStopsTest", "fiberMoveDuringTest", "useFoma"]


def num(x):
    try:
        return float(x)
    except (TypeError, ValueError):
        return NAN


def finite(x):
    return isinstance(x, (int, float)) and x == x and abs(x) != math.inf


def clean(v):
    """JSON-safe: NaN -> None."""
    if isinstance(v, float):
        return v if finite(v) else None
    if isinstance(v, dict):
        return {k: clean(x) for k, x in v.items()}
    if isinstance(v, (list, tuple)):
        return [clean(x) for x in v]
    if isinstance(v, np.generic):
        return clean(v.item())
    return v


# ---------------------------------------------------------------------------------------------
# partitions, table, file names
# ---------------------------------------------------------------------------------------------
def partitions():
    g = json.loads(SPLIT.read_text(encoding="utf-8"))["strata"]["all"]["groups"]
    return {gid.split(":")[-1]: p for p, gids in g.items() for gid in gids}


def day_table():
    """Fuel bottle per test day (rule FILL of docs/wsp/atlas/INVESTIGATIONS.md for the fill-series cells).
    The 'Flames ignited' column is an outcome and is not read here."""
    import csv
    rows = list(csv.DictReader(TABLE.open(encoding="utf-8-sig")))
    out, prev = {}, None
    for r in rows:
        m = re.match(r"\s*(\d+)%\s*(C2H\d+|CH\d+)", r["Fuel Bottle"])
        pct, sp = float(m.group(1)), m.group(2)
        species = "ethylene" if sp.startswith("C2H") else "methane"
        # a fill-series artefact continues the run of the cell above (same percentage, same carbon count)
        if prev and prev[0] == pct and prev[1] == species and sp not in ("CH4", "C2H4"):
            bottle = prev
        else:
            bottle = (pct, species)
        prev = bottle
        out[r["Test Day"].strip()] = dict(test_type=r["Test Type"].strip(), bottle_fraction=bottle[0] / 100.0,
                                          fuel=bottle[1], fuel_mfc_slpm=num(r["Fuel MFC"]),
                                          n2_mfc_slpm=num(r["N2 MFC"]))
    return out


TEST_RE = re.compile(r"^(?:as run)?(\d{5}[A-Z]\d+(?:_1)?)(?![A-Za-z0-9])")  # one letter + digits: test points; RADn / MFCA runs are auxiliary


def test_files(names):
    """test id -> dict of the files this tool uses (None when absent)."""
    ids = set()
    for n in names:
        m = TEST_RE.match(n)
        if m and (n.endswith("_data.csv") or n.endswith(".txt")) and "CAM3" not in n:
            ids.add(m.group(1))
    out = {}
    for t in sorted(ids):
        base = t.replace("_1", "")
        suffix = "_1" if t.endswith("_1") else ""
        def first(*c):
            return next((x for x in c if x in names), None)
        out[t] = dict(
            data=first(f"{t}_data.csv"),
            setpoints=first(f"{t}_mfcSetpoints.csv"),
            tp=first(f"{t}_TP.txt", f"{base}-1_TP.txt" if suffix else "\0"),
            script=first(f"as run{t}.txt", f"{t}.txt"),
        )
    return out


def read_script(path):
    s = {}
    for line in path.read_text(encoding="utf-8", errors="replace").replace("\r", "\n").split("\n"):
        k, sep, v = line.strip().rstrip(",").partition(":")
        if sep:
            s[k.strip()] = v.strip()
    return s


# ---------------------------------------------------------------------------------------------
# inputs: the commanded schedule from the script (CoflowTest.js, CLD Flame Experimental Procedure)
# ---------------------------------------------------------------------------------------------
def zst(species, x_fuel):
    """Stoichiometric mixture fraction of a fuel/N2 stream (fuel mole fraction x_fuel) against air."""
    if not (finite(x_fuel) and x_fuel > 0):
        return NAN
    yf = x_fuel * MW[species] / (x_fuel * MW[species] + (1 - x_fuel) * MW["n2"])
    yo = X_O2_COFLOW * MW["o2"] / (X_O2_COFLOW * MW["o2"] + (1 - X_O2_COFLOW) * MW["n2"])
    s = NU_O2[species] * MW["o2"] / MW[species]
    return 1.0 / (1.0 + s * yf / yo)


def schedule(p):
    """Commanded flow history on the clock of the ignition command (igniter pulse), as segments
    [t0, t1, mix velocity start/end (m/s), coflow velocity start/end (m/s), fuel mole fraction start/end, name].
    Follows the script's stage order; a ramp of zero length is a step. The flow timeout (counted from the
    start of fuel priming) caps the schedule."""
    g = lambda k: p.get(k, NAN) if finite(p.get(k, NAN)) else 0.0  # noqa: E731
    D = g("igniterDelayBeforeRetractMs") / 1000.0
    cI, cF = p.get("concentrationInitial", NAN), p.get("concentrationFinal", NAN)
    mI, mF = p.get("velocityMixIgnitionMPerSec", NAN), p.get("velocityMixFinalMPerSec", NAN)
    aI, aF = p.get("velocityAirIgnitionMPerSec", NAN), p.get("velocityAirFinalMPerSec", NAN)
    segs, t = [], 0.0

    def add(name, dur, m0, m1, a0, a1, c0, c1):
        nonlocal t
        segs.append([t, t + dur, m0, m1, a0, a1, c0, c1, name])
        t += dur
    add("ignition", D, mI, mI, aI, aI, cI, cI)
    add("stabilization_1", g("stabilizationTime1Sec"), mI, mI, aI, aI, cI, cI)
    add("dilution", g("dilutionTimeSec"), mI, mI, aI, aI, cI, cF)
    add("stabilization_2", g("stabilizationTime2Sec"), mI, mI, aI, aI, cF, cF)
    a_match = mI if g("matchingCoflowRampTimeSec") > 0 else aI
    add("coflow_matching_ramp", g("matchingCoflowRampTimeSec"), mI, mI, aI, a_match, cF, cF)
    if g("matchingCoflowRampTimeSec") > 0:
        add("coflow_matching_settle", MATCH_RAMP_OVERHEAD_S, mI, mI, a_match, a_match, cF, cF)
    add("stabilization_3", g("stabilizationTime3Sec"), mI, mI, a_match, a_match, cF, cF)
    add("ramp_to_final", g("fuelAndCoflowRampTimeSec"), mI, mF, a_match, aF, cF, cF)
    add("constant_flow", g("constantFlowTimeSec"), mF, mF, aF, aF, cF, cF)
    n, dv = int(g("rampCount")), g("rampSizeMPerSec")
    m, a = mF, aF
    for i in range(n):
        add(f"ramp_{i + 1}", g("rampTimeSec"), m, m + dv, a, a + dv, cF, cF)
        m, a = m + dv, a + dv
        add(f"ramp_hold_{i + 1}", g("rampConstantFlowTimeSec"), m, m, a, a, cF, cF)
    planned_end = t
    cap = g("flowTimeoutSec") - g("primingAndPreIgnitionDurationSec") if g("flowTimeoutSec") > 0 else math.inf
    capped = cap < planned_end
    if capped:
        segs = [s for s in segs if s[0] < cap]
        segs[-1][1] = min(segs[-1][1], cap)
    return segs, min(planned_end, cap), capped


def inputs_of(test, fn, day_info):
    """Everything known before ignition. Returns (inputs dict, script dict)."""
    s = read_script(RAW / fn["script"]) if fn["script"] else {}
    p = {k: num(s[k]) for k in SCRIPT_NUM if k in s}
    b = {k: s[k] == "true" for k in SCRIPT_BOOL if k in s}
    fuel = day_info["fuel"]
    inp = dict(
        fuel=fuel, bottle_fuel_fraction=day_info["bottle_fraction"], test_type=day_info["test_type"],
        round=1 if test[:2] == "17" or test[:5] < "18100" else (2 if test[:2] == "18" else 3),
        fuel_mfc_full_scale_slpm=day_info["fuel_mfc_slpm"], n2_mfc_full_scale_slpm=day_info["n2_mfc_slpm"],
        script_file=fn["script"], script_name_matches=(s.get("testpointName") == test.replace("_1", "")) if s else None,
        script=dict(p, **b),
    )
    if not s:
        return inp, s
    missing = [k for k in SCHEDULE_KEYS if not finite(p.get(k, NAN))]
    inp["script_missing_keys"] = missing
    if missing:
        return inp, s
    segs, end, capped = schedule(p)
    inp.update(
        x_fuel_initial=p.get("concentrationInitial"), x_fuel_final=p.get("concentrationFinal"),
        v_fuel_ignition_m_s=p.get("velocityMixIgnitionMPerSec"), v_fuel_final_m_s=p.get("velocityMixFinalMPerSec"),
        v_coflow_ignition_m_s=p.get("velocityAirIgnitionMPerSec"), v_coflow_final_m_s=p.get("velocityAirFinalMPerSec"),
        zst_initial=zst(fuel, p.get("concentrationInitial", NAN)), zst_final=zst(fuel, p.get("concentrationFinal", NAN)),
        schedule_from_ignition_command=[dict(t0_s=a[0], t1_s=a[1], v_fuel_m_s=[a[2], a[3]], v_coflow_m_s=[a[4], a[5]],
                                             x_fuel=[a[6], a[7]], stage=a[8]) for a in segs if a[1] > a[0]],
        planned_fuel_off_from_ignition_command_s=end, planned_end_capped_by_flow_timeout=capped,
        planned_constant_flow_from_ignition_command_s=next(([a[0], a[1]] for a in segs if a[8] == "constant_flow"), None),
        fuel_tube_inner_diameter_m=FUEL_TUBE_ID_M, coflow_inner_diameter_m=COFLOW_ID_M, x_o2_coflow=X_O2_COFLOW,
        x_fuel_at_stoichiometric_surface_final=x_fuel_stoich(fuel, p.get("concentrationFinal", NAN)),
        lower_flammability_limit_fuel_in_air=LFL[fuel],
        roper_length_final_m=roper_length(fuel, p.get("concentrationFinal", NAN), p.get("velocityMixFinalMPerSec", NAN)),
    )
    return inp, s


def x_fuel_stoich(species, x_fuel):
    """Fuel mole fraction of the stoichiometric mixture of the fuel stream with air (complete combustion)."""
    if not (finite(x_fuel) and x_fuel > 0):
        return NAN
    S = x_fuel * NU_O2[species] / X_O2_COFLOW      # moles of air per mole of fuel stream
    return x_fuel / (1.0 + S)


def roper_length(species, x_fuel, v_fuel):
    """Roper's experimental correlation for a circular port (Roper 1977; Roper, Smith and Cunningham 1977, as in
    Turns, An Introduction to Combustion, eq. 9.59): L_f = 1330 s/m2 * Q_F / ln(1 + 1/S), Q_F the volumetric
    fuel-stream flow (the commanded mean exit velocity times the tube area, fuel at ambient temperature), S the
    moles of ambient (air) per mole of nozzle fluid at stoichiometry. Published constant, nothing fitted."""
    if not (finite(x_fuel) and x_fuel > 0 and finite(v_fuel) and v_fuel > 0):
        return NAN
    S = x_fuel * NU_O2[species] / X_O2_COFLOW
    q = v_fuel * math.pi * FUEL_TUBE_ID_M ** 2 / 4.0
    return ROPER_CONST * q / math.log(1.0 + 1.0 / S)


# ---------------------------------------------------------------------------------------------
# outcomes: measured from the channels
# ---------------------------------------------------------------------------------------------
def dark_of(t, y, t_from, t_to):
    w = (t >= t_from) & (t < t_to)
    return float(np.median(y[w])) if w.sum() >= 10 else NAN


def measure(test, fn, inp):
    """Returns (pre-ignition inputs from the record, outcome dict, audit dict). Times in s."""
    out = dict(status="ok", flags=[])
    if not fn["data"]:
        out["status"] = "no_data_file"
        return {}, out, {}
    hdr, t, d = fd.read_data(RAW / fn["data"])
    need = ["PMT1", "PMT2", "PMT3", "MFCA1StaticMixerFlow", "MFCA2BypassFlow", "MFCA3FuelFlow", "PTA3"]
    if len(t) < 500 or any(c not in d for c in need):
        out["status"] = "short_or_unknown_columns"
        return {}, out, {}
    sc = inp.get("script", {})
    # --- ignition command: fuel setpoint on (start of priming) + priming duration -----------------
    t_ign, src = NAN, "none"
    ts = vs = None
    if fn["setpoints"]:
        ts, vs = fd.read_setpoints(RAW / fn["setpoints"], t[0])
        on = np.where(vs[:, 2] > 0.001)[0]
        if len(on) and finite(sc.get("primingAndPreIgnitionDurationSec", NAN)):
            t_ign, src = float(ts[on[0]]) + sc["primingAndPreIgnitionDurationSec"], "setpoints+priming"
    stamps = {}
    if fn["tp"]:
        stamps, _, _ = fd.read_tp(RAW / fn["tp"])
    st_ign = stamps.get("Ignition and retraction")
    if st_ign is not None and st_ign < t[0] - 40000:
        st_ign += 86400
    if not finite(t_ign) and st_ign is not None:
        t_ign, src = st_ign + 0.5, "tp_stamp+0.5"
    if not finite(t_ign):
        out["status"] = "no_ignition_reference"
        return {}, out, {}
    audit = dict(ignition_source=src, ignition_minus_tp_stamp_s=(t_ign - st_ign) if st_ign is not None else NAN)
    # --- pre-ignition record (inputs) ---------------------------------------------------------------
    pre = (t >= t_ign + RAD_PRE_S[0]) & (t < t_ign + RAD_PRE_S[1])
    rad = [c for c in ("Radiometer+60", "Radiometer+30", "RadiometerCenter", "Radiometer-30", "Radiometer-60") if c in d]
    rs = sum(d[c] for c in rad) if rad else None
    pre_inputs = dict(
        chamber_pressure_reading_pa=(PTA_PSIA_PER_V * (float(np.median(d["PTA3"][pre])) - PTA_ZERO_V) * PSIA_PA)
        if pre.sum() >= 10 else NAN,
        pta3_pre_ignition_v=float(np.median(d["PTA3"][pre])) if pre.sum() >= 10 else NAN,
        burner_tc_pre_ignition_v=float(np.median(d["TCBurner"][pre])) if pre.sum() >= 10 and "TCBurner" in d else NAN,
        radiometer_sum_pre_ignition_v=float(np.median(rs[pre])) if rs is not None and pre.sum() >= 10 else NAN,
        pmt_gains=[num(hdr.get(f"PMT {i} Gain")) for i in (1, 2, 3)],
        mfc_full_scale_slpm=[num(hdr.get(f"MFCA{i} Full Scale")) for i in (1, 2, 3)],
    )
    # --- PMT1 (OH*): onset and end -------------------------------------------------------------------
    p1 = d["PMT1"]
    dark = dark_of(t, p1, t_ign - 6.0, t_ign - 0.5)       # armed (MFCs on), before the igniter is powered
    if not finite(dark) or dark > 1.0:
        out["flags"].append("pmt1_not_armed_before_ignition")
        dark = float(np.percentile(p1[t > t_ign], 2)) if (t > t_ign).any() else NAN
    ex = fd.smooth(t, p1 - dark, 0.1)
    onset = fd.first_sustained(t, (ex > PMT_LIT_V) & (t < t_ign + ONSET_WINDOW_S[1]), t_ign + ONSET_WINDOW_S[0],
                               PMT_HOLD_LIT_S)
    out.update(t_ignition_command_clock=t_ign, pmt1_dark_v=dark)
    # fuel off: first command to zero fuel after the ignition command; readback as fallback
    fuel_off, fsrc = NAN, "none"
    if ts is not None:
        z = np.where((ts > t_ign) & (vs[:, 2] <= 0.001))[0]
        if len(z):
            fuel_off, fsrc = float(ts[z[0]]), "setpoints"
    if not finite(fuel_off):
        fv = fd.smooth(t, d["MFCA3FuelFlow"], 0.2)
        fuel_off = fd.first_sustained(t, fv < 0.05, t_ign, 1.0)
        fsrc = "readback" if finite(fuel_off) else "none"
        out["flags"].append("fuel_off_from_readback")
    out.update(fuel_off_source=fsrc)
    if not finite(onset):
        out.update(ignited=False, onset_clock=NAN)
        out["fuel_off_from_ignition_command_s"] = fuel_off - t_ign
        return pre_inputs, out, audit
    end = fd.first_sustained(t, ex < PMT_END_V, onset + PMT_HOLD_LIT_S, PMT_HOLD_END_S)
    if not finite(end):
        end = NAN
        out["flags"].append("pmt1_lit_at_file_end")
    loss = bool(finite(end) and (not finite(fuel_off) or end < fuel_off - LABEL_MARGIN_S))
    out.update(
        ignited=True, onset_clock=onset, onset_from_ignition_command_s=onset - t_ign,
        flame_end_from_onset_s=end - onset, fuel_off_from_onset_s=fuel_off - onset,
        planned_fuel_off_from_onset_s=inp["planned_fuel_off_from_ignition_command_s"] - (onset - t_ign),
        fuel_off_from_ignition_command_s=fuel_off - t_ign,
        loss=loss, loss_time_from_onset_s=(end - onset) if loss else None,
        pmt1_peak_excess_v=float(np.max(ex[(t >= onset) & (t < (end if finite(end) else t[-1]))]))
        if ((t >= onset) & (t < (end if finite(end) else t[-1]))).any() else NAN,
    )
    # stage at loss, from the commanded schedule
    if loss and inp.get("schedule_from_ignition_command"):
        tl = end - t_ign
        st = [s["stage"] for s in inp["schedule_from_ignition_command"] if s["t0_s"] <= tl < s["t1_s"]]
        out["loss_stage"] = st[0] if st else ("after_planned_end" if tl >= inp["planned_fuel_off_from_ignition_command_s"] else "before_schedule")
    # --- radiometer level over the first constant-flow stage ---------------------------------------
    cf = inp.get("planned_constant_flow_from_ignition_command_s")
    if rs is not None and cf and cf[1] > cf[0] and finite(pre_inputs["radiometer_sum_pre_ignition_v"]):
        a = t_ign + cf[0] + RAD_SKIP_S
        b = t_ign + cf[1]
        stop = min(x for x in (b, end, fuel_off) if finite(x))
        w = (t >= a) & (t < stop)
        if stop - a >= RAD_MIN_S and w.sum() >= 50:
            exr = float(np.median(rs[w])) - pre_inputs["radiometer_sum_pre_ignition_v"]
            out.update(radiometer_excess_v=exr, radiometer_log10=math.log10(max(exr, RAD_FLOOR_V)),
                       radiometer_window_from_onset_s=[a - onset, stop - onset],
                       radiometer_window_full=bool(stop >= b - 0.01))
    # cross-checks with the script's own stamps
    if stamps:
        audit["tp_extinction_detected"] = "Extinction detected" in stamps
        audit["tp_has_constant_flow"] = "Constant flow" in stamps
    if inp.get("planned_fuel_off_from_ignition_command_s") is not None and finite(fuel_off):
        audit["fuel_off_minus_planned_s"] = (fuel_off - t_ign) - inp["planned_fuel_off_from_ignition_command_s"]
    return pre_inputs, out, audit


# ---------------------------------------------------------------------------------------------
# build
# ---------------------------------------------------------------------------------------------
def build_cases(which=("train", "validation")):
    part = partitions()
    days = day_table()
    names = {p.name for p in RAW.iterdir()}
    files = test_files(names)
    rows = []
    for t, fn in files.items():
        p = part.get(t[:5])
        if p not in which:           # test days and unknown days: never opened
            continue
        inp, s = inputs_of(t, fn, days[t[:5]])
        if not s:
            pre, out, audit = {}, dict(status="no_script", flags=[]), {}
        elif inp.get("script_missing_keys"):
            pre, out, audit = {}, dict(status="script_incomplete", flags=[]), {}
        else:
            pre, out, audit = measure(t, fn, inp)
        inp.update(pre)
        rows.append(dict(case_id=f"{INV}:{t}", test=t, day=t[:5], partition=p, inputs=inp, outcome=out, audit=audit))
    return rows


def case_has(r, target):
    o, i = r["outcome"], r["inputs"]
    base = o.get("status") == "ok" and i.get("schedule_from_ignition_command")
    if target == "loss":
        return bool(base and o.get("ignited") and finite(o.get("fuel_off_from_onset_s", NAN))
                    and o.get("fuel_off_from_onset_s") > 0)
    if target == "radiometer":
        return bool(base and o.get("ignited") and o.get("radiometer_log10") is not None)
    raise ValueError(target)


def at_cutoff(o):
    """What a forecast made at flame onset may know from the record: that the flame lit, and when."""
    return dict(ignited=o.get("ignited"), onset_from_ignition_command_s=o.get("onset_from_ignition_command_s"),
                planned_fuel_off_from_onset_s=o.get("planned_fuel_off_from_onset_s"))


def cmd_build(_args):
    OUT.mkdir(parents=True, exist_ok=True)
    rows = build_cases()
    tr = [r for r in rows if r["partition"] == "train"]
    va = [r for r in rows if r["partition"] == "validation"]
    for r in tr:
        r["targets"] = [k for k in ("loss", "radiometer") if case_has(r, k)]
        r["at_cutoff"] = at_cutoff(r["outcome"])
    with (OUT / "cases_train.jsonl").open("w", encoding="utf-8") as f:
        for r in tr:
            f.write(json.dumps(clean(r)) + "\n")
    n_vi = 0
    with (OUT / "cases_validation_inputs.jsonl").open("w", encoding="utf-8") as f, \
            (OUT / "cases_validation_outcomes.jsonl").open("w", encoding="utf-8") as g:
        for r in va:
            targets = [k for k in ("loss", "radiometer") if case_has(r, k)]
            usable = r["outcome"].get("status") == "ok"
            vi = dict(case_id=r["case_id"], test=r["test"], day=r["day"], partition="validation",
                      usable=usable, inputs=r["inputs"], at_cutoff=at_cutoff(r["outcome"]) if usable else None)
            n_vi += usable
            f.write(json.dumps(clean(vi)) + "\n")
            g.write(json.dumps(clean(dict(case_id=r["case_id"], day=r["day"], targets=targets, outcome=r["outcome"],
                                          audit=r["audit"]))) + "\n")
    ntr = Counter(k for r in tr for k in r["targets"])
    print(f"train: {len(tr)} test points, loss cases {ntr['loss']}, radiometer cases {ntr['radiometer']}")
    print(f"validation: {len(va)} test points, {n_vi} with a complete script and a data file; "
          f"outcomes in a separate file (not summarised)")
    print(f"-> {OUT}")


# ---------------------------------------------------------------------------------------------
# score
# ---------------------------------------------------------------------------------------------
def integrated(horizons, values):
    """Trapezoid over the horizons divided by their span (as tools/wsp/ext/transfer_acme.py)."""
    a = 0.0
    for k in range(1, len(horizons)):
        a += 0.5 * (values[k] + values[k - 1]) * (horizons[k] - horizons[k - 1])
    return a / (horizons[-1] - horizons[0])


def pinball(q, tau, y):
    return (tau - (1.0 if y < q else 0.0)) * (y - q)


def crps_quantiles(qs, y):
    """CRPS approximated from quantiles at QUANTILES: 2 x mean pinball loss."""
    return 2.0 * sum(pinball(q, tau, y) for q, tau in zip(qs, QUANTILES)) / len(QUANTILES)


def window_end(o):
    """Last time (s since onset) at which a case is scored: the planned fuel-off time from the script (an input),
    or the actual fuel-off time when the fuel was cut earlier with the flame burning (an unplanned stop
    censors the case there). The window does not depend on whether the flame was lost."""
    planned = o.get("planned_fuel_off_from_onset_s")
    if planned is None:
        planned = o.get("fuel_off_from_onset_s")
    if not o.get("loss") and o.get("fuel_off_from_onset_s") is not None:
        return min(planned, o["fuel_off_from_onset_s"])
    return planned


def loss_terms(fc, o):
    """Per-horizon squared errors of one loss forecast (None outside the case's window, see window_end), and
    the squared error of the yes/no."""
    p = [float(x) for x in fc["p_loss_by"]]
    if len(p) != len(HORIZONS_S) or any(not (0.0 <= x <= 1.0) for x in p) or \
            any(p[k] > p[k + 1] + 1e-12 for k in range(len(p) - 1)):
        raise ValueError(f"p_loss_by must hold {len(HORIZONS_S)} non-decreasing probabilities")
    pa = float(fc.get("p_loss_before_fuel_off", p[-1]))
    lost = bool(o.get("loss"))
    tl = o.get("loss_time_from_onset_s") if lost else None
    w = window_end(o)
    terms = []
    for h, ph in zip(HORIZONS_S, p):
        if h > w:
            terms.append(None)
        else:
            terms.append((ph - (1.0 if (lost and tl <= h) else 0.0)) ** 2)
    return terms, (pa - (1.0 if lost else 0.0)) ** 2


def _loss_block(cases):
    """Aggregate of a list of (terms, brier_any): per-horizon mean over the cases at risk, integrated over the
    horizons that hold at least one case (a horizon nobody reaches drops out of the integral)."""
    hs, means, ns = [], [], []
    for k, h in enumerate(HORIZONS_S):
        v = [c[0][k] for c in cases if c[0][k] is not None]
        if v:
            hs.append(h)
            means.append(float(np.mean(v)))
            ns.append(len(v))
    ibs = integrated(hs, means) if len(hs) >= 2 else (means[0] if means else NAN)
    return dict(ibs=ibs, brier_any=float(np.mean([c[1] for c in cases])) if cases else NAN, n=len(cases),
                brier_by_horizon={f"{h:g}": m for h, m in zip(hs, means)},
                at_risk_by_horizon={f"{h:g}": n for h, n in zip(hs, ns)})


def score(forecasts, outcomes, target):
    """Score forecasts against outcomes.
    forecasts: {case_id: forecast}; outcomes: {case_id: {"day": ..., "outcome": {...}}}.
    target 'loss': forecast = {"p_loss_by": [P(flame lost by h) for h in HORIZONS_S], non-decreasing in [0, 1],
                               "p_loss_before_fuel_off": P(loss at any time before the fuel is commanded off)
                               (optional; defaults to the last value of p_loss_by)}.
       Primary: integrated Brier over HORIZONS_S on the cases at risk at each horizon (a case leaves the risk
       set when its fuel is commanded off with the flame still burning). Secondary: Brier of the yes/no.
    target 'radiometer': forecast = {"quantiles": [5 non-decreasing values of log10(V) at QUANTILES]}.
       CRPS approximated by 2 x mean pinball loss.
    Returns the aggregate over all cases, the mean over days of the day scores, and the per-day scores.
    Cases without a forecast are listed in 'missing_forecasts' and not scored."""
    per_case, missing = {}, []
    for cid, row in outcomes.items():
        if cid not in forecasts:
            missing.append(cid)
            continue
        o = row["outcome"]
        if target == "loss":
            per_case[cid] = (row["day"], loss_terms(forecasts[cid], o))
        elif target == "radiometer":
            q = [float(x) for x in forecasts[cid]["quantiles"]]
            if len(q) != len(QUANTILES) or any(q[k] > q[k + 1] for k in range(len(q) - 1)):
                raise ValueError(f"{cid}: quantiles must be {len(QUANTILES)} non-decreasing values")
            per_case[cid] = (row["day"], crps_quantiles(q, o["radiometer_log10"]))
        else:
            raise ValueError(target)
    by_day = defaultdict(list)
    for dday, v in per_case.values():
        by_day[dday].append(v)
    if target == "loss":
        per_day = {dday: _loss_block(v) for dday, v in sorted(by_day.items())}
        agg = _loss_block([v for _, v in per_case.values()]) if per_case else {}
        keys = ["ibs", "brier_any"]
    else:
        per_day = {dday: dict(crps=float(np.mean(v)), n=len(v)) for dday, v in sorted(by_day.items())}
        agg = dict(crps=float(np.mean([v for _, v in per_case.values()])), n=len(per_case)) if per_case else {}
        keys = ["crps"]
    agg_days = {k: float(np.mean([d[k] for d in per_day.values()])) for k in keys} if per_day else {}
    return dict(target=target, n_cases=len(per_case), n_days=len(per_day), aggregate=agg,
                aggregate_day_mean=agg_days, per_day=per_day, missing_forecasts=missing)


# ---------------------------------------------------------------------------------------------
# baselines (constants from train days only)
# ---------------------------------------------------------------------------------------------
def load(path):
    return [json.loads(x) for x in path.open(encoding="utf-8")]


def km_curve(rows):
    """Kaplan-Meier estimate of P(loss by h) at HORIZONS_S; a case is censored at the end of its window
    (window_end)."""
    ev = []
    for r in rows:
        o, w = r["outcome"], window_end(r["outcome"])
        ev.append((o["loss_time_from_onset_s"], 1) if o.get("loss") and o["loss_time_from_onset_s"] <= w else (w, 0))
    ev.sort()
    out, S, k, n = [], 1.0, 0, len(ev)
    at_risk = n
    for h in HORIZONS_S:
        while k < n and ev[k][0] <= h:
            t = ev[k][0]
            d = c = 0
            while k < n and ev[k][0] == t:
                d += ev[k][1]
                c += 1 - ev[k][1]
                k += 1
            if at_risk > 0:
                S *= 1.0 - d / at_risk
            at_risk -= d + c
        out.append(1.0 - S)
    return out


def previous_test(rows):
    """case_id -> the previous test point of the same day among `rows` (order of the ignition command)."""
    out = {}
    by_day = defaultdict(list)
    for r in rows:
        by_day[r["day"]].append(r)
    for rs in by_day.values():
        rs = sorted(rs, key=lambda r: r["outcome"].get("t_ignition_command_clock") or 0.0)
        for a, b in zip(rs, rs[1:]):
            out[b["case_id"]] = a
    return out


def classical_lfl(r):
    """Coward and Jones lower flammability limit applied at the stoichiometric surface: a fuel stream whose
    stoichiometric mixture with air is below the limit cannot hold a diffusion flame. Published constants; the
    criterion is a yes/no, written as P = 1 or 0 at every horizon."""
    x = r["inputs"]["x_fuel_at_stoichiometric_surface_final"]
    return 1.0 if (x is not None and x < r["inputs"]["lower_flammability_limit_fuel_in_air"]) else 0.0


def _with_end(curve):
    return dict(p_loss_by=list(curve), p_loss_before_fuel_off=curve[-1])


def cmd_baselines(_args):
    tr = load(OUT / "cases_train.jsonl")
    res = {}
    # ---------------- loss ----------------
    L = [r for r in tr if "loss" in r["targets"]]
    outc = {r["case_id"]: r for r in L}
    fc = {}
    km = km_curve(L)
    fc["incidence"] = {r["case_id"]: _with_end(km) for r in L}
    byf = {f: km_curve([r for r in L if r["inputs"]["fuel"] == f]) for f in ("methane", "ethylene")}
    fc["incidence_by_fuel"] = {r["case_id"]: _with_end(byf[r["inputs"]["fuel"]]) for r in L}

    def diluted(r):    # the script dilutes the fuel stream during the test
        i = r["inputs"]
        return i["x_fuel_final"] < i["x_fuel_initial"]
    byd = {k: km_curve([r for r in L if diluted(r) == k]) for k in (True, False)}
    fc["incidence_by_dilution_ramp"] = {r["case_id"]: _with_end(byd[diluted(r)]) for r in L}
    prev = previous_test([r for r in tr if r["outcome"].get("ignited") and r["outcome"].get("status") == "ok"])
    pl = {k: km_curve([r for r in L if r["case_id"] in prev and bool(prev[r["case_id"]]["outcome"].get("loss")) == k])
          for k in (True, False)}
    fc["persistence_previous_test_of_day"] = {
        r["case_id"]: _with_end(pl[bool(prev[r["case_id"]]["outcome"].get("loss")) if r["case_id"] in prev else False])
        for r in L}
    fc["classical_lfl_at_stoichiometric_surface"] = {
        r["case_id"]: dict(p_loss_by=[classical_lfl(r)] * len(HORIZONS_S), p_loss_before_fuel_off=classical_lfl(r))
        for r in L}
    for name, f in fc.items():
        res[f"loss/{name}"] = score(f, outc, "loss")
    res["loss/_constants"] = dict(horizons_s=HORIZONS_S, incidence=km, by_fuel=byf,
                                  by_dilution_ramp={str(k): v for k, v in byd.items()},
                                  persistence={str(k): v for k, v in pl.items()})
    # ---------------- radiometer ----------------
    R = [r for r in tr if "radiometer" in r["targets"]]
    outr = {r["case_id"]: r for r in R}
    y = np.array([r["outcome"]["radiometer_log10"] for r in R])
    qall = [float(np.quantile(y, q)) for q in QUANTILES]
    rb = {"constant": {r["case_id"]: dict(quantiles=qall) for r in R}}
    qf = {f: [float(np.quantile([r["outcome"]["radiometer_log10"] for r in R if r["inputs"]["fuel"] == f], q))
              for q in QUANTILES] for f in ("methane", "ethylene")}
    rb["constant_by_fuel"] = {r["case_id"]: dict(quantiles=qf[r["inputs"]["fuel"]]) for r in R}
    # persistence: the previous test of the day with a radiometer level, plus the quantiles of the train
    # differences between consecutive tests (the first test of a day takes the by-fuel constant)
    prevr = previous_test(R)
    diffs = [r["outcome"]["radiometer_log10"] - prevr[r["case_id"]]["outcome"]["radiometer_log10"]
             for r in R if r["case_id"] in prevr]
    dq = [float(np.quantile(diffs, q)) for q in QUANTILES]
    rb["persistence_previous_test_of_day"] = {
        r["case_id"]: dict(quantiles=[prevr[r["case_id"]]["outcome"]["radiometer_log10"] + d for d in dq])
        if r["case_id"] in prevr else dict(quantiles=qf[r["inputs"]["fuel"]]) for r in R}
    # classical: Roper flame length (published constant) with one offset per fuel and the residual quantiles
    # (constants fitted on train)
    rr = defaultdict(list)
    for r in R:
        rr[r["inputs"]["fuel"]].append(r["outcome"]["radiometer_log10"] - math.log10(r["inputs"]["roper_length_final_m"]))
    rq = {f: [float(np.quantile(v, q)) for q in QUANTILES] for f, v in rr.items()}
    rb["roper_length_plus_fuel_offset"] = {
        r["case_id"]: dict(quantiles=[math.log10(r["inputs"]["roper_length_final_m"]) + d
                                      for d in rq[r["inputs"]["fuel"]]]) for r in R}
    for name, f in rb.items():
        res[f"radiometer/{name}"] = score(f, outr, "radiometer")
    res["radiometer/_constants"] = dict(quantiles=QUANTILES, constant=qall, by_fuel=qf, persistence_diff=dq,
                                        roper_offset=rq)
    for k, v in res.items():
        if k.endswith("_constants"):
            continue
        a = v["aggregate"]
        print(f"{k:50s} cases {v['n_cases']:3d} days {v['n_days']:2d}  "
              + "  ".join(f"{m} {a[m]:.4f} (day-mean {v['aggregate_day_mean'][m]:.4f})"
                          for m in ("ibs", "brier_any", "crps") if m in a))
    (OUT / "baselines_train.json").write_text(json.dumps(clean(res), indent=1), encoding="utf-8")
    print(f"-> {OUT / 'baselines_train.json'}")


# ---------------------------------------------------------------------------------------------
# counts
# ---------------------------------------------------------------------------------------------
def cmd_counts(_args):
    part = partitions()
    # test days: catalogue only
    cat = defaultdict(lambda: defaultdict(set))
    sizes = defaultdict(lambda: Counter())
    for line in CATALOGUE.open(encoding="utf-8"):
        r = json.loads(line)
        if r.get("investigation") != INV or r.get("repeat_of_row") is not None:
            continue
        day = r.get("test_day")
        p = part.get(day or "", "no_day")
        sizes[p][r["medium"]] += r["size_bytes"] or 0
        if r.get("test_id_scheme") == "acme_test_point" and r.get("test_id"):
            cat[p][r["kind"]].add(r["test_id"])
    for p in ("train", "validation", "test", "no_day"):
        k = cat[p]
        both = k["sensor time series"] & k["script or settings"]
        print(p, {kk: len(v) for kk, v in k.items()}, "data+script:", len(both),
              {m: f"{b / 1e9:.1f} GB" for m, b in sizes[p].items()})


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    sub = ap.add_subparsers(dest="cmd", required=True)
    sub.add_parser("build").set_defaults(fn=cmd_build)
    sub.add_parser("counts").set_defaults(fn=cmd_counts)
    sub.add_parser("baselines").set_defaults(fn=cmd_baselines)
    a = ap.parse_args()
    a.fn(a)


if __name__ == "__main__":
    main()
