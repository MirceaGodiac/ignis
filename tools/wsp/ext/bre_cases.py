#!/usr/bin/env python3
"""ACME BRE, the Burning Rate Emulator (PSI-20): forecast cases and their scorer. Preparation only:
this tool changes no model, and nothing in it is fitted except the constants of the named baselines,
on TRAIN days (`baselines`). Every threshold below is typed, with its reason.

Partitions are the frozen day groups of docs/wsp/atlas/splits/PSI-20.json. TRAIN days: every file may
be read. VALIDATION days: inputs are built; their outcomes are measured by this script into a separate
file and never printed. TEST days: no file of a test day is opened; they are counted from the file
catalogue (runs/dev/data_all/work/atlas/files.jsonl) only. The two Chamber Calculator workbooks span
all days: their rows are read and every row whose test point is not on a train or validation day is
dropped unread (only the TPID cell is looked at).

Two targets (docs/wsp/feedback/bre_feasibility.md):

  T1  flame end and cause: seconds from the measured flame onset to the flame end, with the cause
      `extinction` (the flame went out while the fuel still flowed) or `operator_termination` (the
      scripted stop of the flow ended it). Cases are (test, cutoff) at 1, 3, 5 s after onset with the
      flame still burning. Score: IPCW Brier per cause at HORIZONS, integrated over them (ported
      unchanged from tools/wsp/ext/transfer_acme.py, which ports crates/ignis-lab/src/eval).
  T2  heat flux to the burner surface: the centre Medtherm gauge of the BRE burner, mean over 8-12 s
      after onset, kW/m2, for flames still lit at 12 s. Score: CRPS from the quantiles at 5, 25, 50,
      75, 95 % (`crps_quantiles`, as crates/ignis-lab/src/eval/scores.rs).

Files written under runs/dev/data_all/work/transfer_bre/:
  cases_train.jsonl               one row per TRAIN test: inputs, prefix (first 3 s), outcome, flags
  cases_validation_inputs.jsonl   one row per VALIDATION test: inputs and prefix only
  cases_validation_outcomes.jsonl VALIDATION outcomes (never printed; read only by `score` in a registered look)
  counts.json                     tests by partition with inputs and outcome (test days: catalogue only)
  baselines_train.json            the baseline constants (train) and their train scores

    python tools/wsp/ext/bre_cases.py build
    python tools/wsp/ext/bre_cases.py baselines
    python tools/wsp/ext/bre_cases.py score --target t1 --forecasts F.jsonl [--partition train]
    python tools/wsp/ext/bre_cases.py selftest

Forecast file formats (one JSON object per line):
  t1: {"test": "19039C1", "cutoff_s": 3.0, "cif": {"extinction": [p at HORIZONS], "operator_termination": [...]}}
  t2: {"test": "19039C1", "quantiles": [q05, q25, q50, q75, q95]}
"""
import argparse
import csv
import json
import math
import os
import re
import sys
from collections import Counter, defaultdict
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[2]
sys.path.insert(0, str(HERE))
import flame_design as fd          # noqa: E402  readers and signal helpers, used unchanged
import acme_script2 as s2          # noqa: E402  the planned stop from the script file, used unchanged
from transfer_acme import ipcw_brier, ipcw_terms, integrated_brier  # noqa: E402  the scorer, unchanged

INV = "PSI-20"
RAW = REPO / "runs/dev/data_all/raw" / INV / "Raw Data"
PRE = f"{INV}_Raw Data_"
CATALOGUE_FILES = REPO / "runs/dev/data_all/work/atlas/files.jsonl"
CATALOGUE_TESTS = REPO / "runs/dev/data_all/work/atlas/tests.jsonl"
SPLIT = REPO / "docs/wsp/atlas/splits" / f"{INV}.json"
OUT = REPO / "runs/dev/data_all/work/transfer_bre"
CALCULATORS = ["Final ACME Chamber Calculator - 2019-01-31 to 2019-04-09.xlsx",
               "Final ACME Chamber Calculator - 2020-07-22 to 2021-02-03.xlsx"]

# --- scoring grid -------------------------------------------------------------------------------
# CFI-G horizons, extended to 210 s because BRE scripts hold constant flow mostly up to 180 s (planned stops
# 56-202 s after onset; five train scripts plan 300-364 s, scored on extinction up to 210 s).
HORIZONS = [6.0, 8.0, 10.0, 12.0, 15.0, 20.0, 25.0, 30.0, 40.0, 50.0, 60.0, 80.0, 100.0, 120.0, 150.0, 180.0, 210.0]
CUTOFFS = [1.0, 3.0, 5.0]
CAUSES = ["extinction", "operator_termination"]
LEVELS = [0.05, 0.25, 0.5, 0.75, 0.95]
T2_WINDOW = (8.0, 12.0)       # s after onset: igniter retracted (3-4 s) and its transient gone
PREFIX_WINDOW = (2.0, 3.0)    # s after onset: the sensor prefix a T2 forecast may use (cutoff 3 s)

# --- typed measurement thresholds -----------------------------------------------------------------
ONSET_PMT_V = 0.15        # PMT sum above its pre-ignition median: lit (the CFI-G radiometer rise size)
ONSET_HOLD_S = 0.3
PMT_END_V = 0.05          # PMT sum back within 0.05 V of the armed dark level (as flame_design.py)
PMT_HOLD_S = 0.5
LABEL_MARGIN_S = 0.5      # flame end this much before the flow change: extinction (as flame_design.py)
STAMP_TO_STAGE_S = 3.5    # ignition stamp to stabilization #1 when unstamped (igniterDelayBeforeRetract, typical)
# --- units -----------------------------------------------------------------------------------------
BTU_FT2_S_KW_M2 = 11.356526682  # 1 Btu/(ft2 s) in kW/m2; gauge scale "1 mV = 1 Btu/ft2" (ACME non-imaging
                                # data file description 2017-02-17, Table 2, AIN-20/21), per second assumed
PSIA_PA = 6894.757293
N_STP = 101325.0 / (8.314462618 * 273.15)   # mol/m3 at the MFC standard (0 degC, 760 Torr)
GCF = {"ethylene": 0.6062, "methane": 0.7786766523371256, "nitrogen": 1.0}  # Hastings, calculator SourceTable
MW = {"ethylene": 28.0532, "methane": 16.0425, "nitrogen": 28.0134}           # calculator Property Table
LHV_MJ_KG = {"ethylene": 47.16, "methane": 50.03}  # textbook lower heating values (not from the archive)
BURNER_CODE_M = {"6": 0.025, "7": 0.050}          # data-file header `Burner:`; matches the day table on every day read
NAN = float("nan")


# ---------------------------------------------------------------------------------------------
# partitions and catalogue
# ---------------------------------------------------------------------------------------------
def partitions():
    g = json.loads(SPLIT.read_text(encoding="utf-8"))["strata"]["all"]["groups"]
    return {gid.split(":")[-1]: part for part, ids in g.items() for gid in ids}


def catalogue_tests():
    """Per test id of PSI-20: the kinds of its raw files, from the file catalogue (no file opened)."""
    out = defaultdict(Counter)
    with CATALOGUE_FILES.open(encoding="utf-8") as f:
        for line in f:
            r = json.loads(line)
            if r["investigation"] != INV or not r["latest_version"] or r["repeat_of_row"] or not r["test_id"]:
                continue
            if not re.fullmatch(r"\d{5}[A-Z]\d+", r["test_id"]):
                continue
            n = r["file_name"]
            k = ("data" if re.search(r"_data1?\.csv$", n) else "setpoints" if n.endswith("_mfcSetpoints.csv")
                 else "tp" if n.endswith("_TP.txt") else "script" if r["kind"] == "script or settings" and n.endswith(".txt")
                 else "image_video" if r["kind"] in ("image archive", "video") else "other")
            out[r["test_id"]][k] += 1
            out[r["test_id"]]["bytes_" + k] += r["size_bytes"]
    return out


def day_conditions():
    """Fuel bottle and burner size per day, from the test catalogue (conditions only)."""
    out = {}
    with CATALOGUE_TESTS.open(encoding="utf-8") as f:
        for line in f:
            r = json.loads(line)
            if r["investigation_id"] != INV:
                continue
            sizes = {s["dimension"]: s["value"]["value_si"] for s in r["geometry"].get("sizes", [])}
            out[r["test_id"]] = dict(fuel=r["fuel"]["name"]["value_si"],
                                     x_fuel_bottle=r["injection"]["x_fuel_injected"]["value_si"],
                                     burner_diameter_m=sizes.get("burner_diameter"))
    return out


def files_of(test):
    def first(*names):
        for n in names:
            if (RAW / n).exists():
                return RAW / n
        return None
    return dict(data=first(f"{PRE}{test}_data.csv", f"{PRE}{test}_data1.csv"),
                setpoints=first(f"{PRE}{test}_mfcSetpoints.csv"), tp=first(f"{PRE}{test}_TP.txt"),
                script=first(f"{PRE}as run{test}.txt", f"{PRE}{test}.txt", f"{PRE}{test}_BRE.txt"))


# ---------------------------------------------------------------------------------------------
# inputs
# ---------------------------------------------------------------------------------------------
def calculator_rows(keep_days):
    """TP rows of the Chamber Calculator EventData sheets, for the given days only. Only pre-test columns
    are kept: start pressure, start mixture, start chamber temperatures, fuel type, igniter settings.
    Never kept: Ignite (the operators' yes/no), the MFC volumes of the test, Pend, the end temperatures,
    Burn Fraction, the event delta and the finish mixture (all after the test)."""
    import openpyxl
    out, dup = {}, Counter()
    species = ["O2", "He", "CO2", "N2", "H2O", "ethylene", "methane", "propane", "butane", "ethane", "H2"]
    for name in CALCULATORS:
        wb = openpyxl.load_workbook(RAW / f"{PRE}{name}", data_only=True, read_only=True)
        for r in list(wb["EventData"].iter_rows(values_only=True))[4:]:
            tp = str(r[2]).strip() if r[0] == "TP" and r[2] is not None else None
            if not tp or tp[:5] not in keep_days:
                continue                                   # a test-day row is dropped here, unread
            mix = [float(x or 0.0) for x in r[39:50]]     # AN..AX: Start Mixture, standard litres
            tot = sum(mix)
            row = dict(calc_file=name, tstart_gmt=str(r[12]), pressure_start_psia=r[18], fuel_type_calc=r[23],
                       igniter_wattage=r[24], igniter_on_time_ms=r[25],
                       chamber_t_start_c=[r[i] for i in range(28, 32)],
                       start_mixture_l=tot, **{f"x_{s}": (m / tot if tot else None) for s, m in zip(species, mix)})
            dup[tp] += 1
            out.setdefault(tp, row)       # a repeated TPID keeps its first row and is flagged
    return out, {k for k, v in dup.items() if v > 1}


def gcf_mix(x_fuel, fuel):
    return 1.0 / (x_fuel / GCF[fuel] + (1.0 - x_fuel) / GCF["nitrogen"])


def header_of(path):
    with open(path, encoding="utf-8", errors="replace") as f:
        first = f.readline()
    if not re.match(r"\d{4}-\d\d-\d\d \d{3} ", first):
        return None
    return fd.read_header(first)


def inputs_of(test, fls, day, calc, calc_dup):
    flags = []
    s = s2.parse(fls["script"]) if fls["script"] else {}
    if not s:
        flags.append("no_script")
    elif s.get("testpointName") not in (None, test):
        flags.append(f"script_names_{s.get('testpointName')}")
    hdr = header_of(fls["data"]) if fls["data"] else None
    num = lambda k: s2._num(s, k)  # noqa: E731
    fuel, xb = day["fuel"], day["x_fuel_bottle"]
    d_m = day["burner_diameter_m"]
    if hdr:
        code_d = BURNER_CODE_M.get(hdr.get("Burner"))
        if code_d is None or (d_m is not None and abs(code_d - d_m) > 1e-9):
            flags.append(f"burner_code_{hdr.get('Burner')}_vs_table_{d_m}")
        if hdr.get("Test Point") not in (None, test):
            flags.append(f"data_names_{hdr.get('Test Point')}")
    elif fls["data"]:
        flags.append("data_without_header")
    c = calc.get(test, {})
    if not c:
        flags.append("no_calculator_row")
    if test in calc_dup:
        flags.append("calculator_tpid_repeated")
    ft = (c.get("fuel_type_calc") or "").lower()
    if c and not ((fuel == "ethylene" and "ethylene" in ft and (("50/50" in ft) == (xb == 0.5)))
                  or (fuel == "methane" and "methane" in ft)):
        flags.append(f"fuel_calc_{ft}_vs_table_{fuel}_{xb}")
    q_test, q_ign, q_n2 = num("fuelBottleTestFlowRateN2Slpm"), num("fuelBottleIgnitionFlowRateN2Slpm"), num("nitrogenTestFlowRateN2Slpm")
    area = math.pi * d_m ** 2 / 4.0 if d_m else None
    mdot = None
    if q_test is not None and fuel in GCF and xb is not None:
        bottle_slpm = q_test * gcf_mix(xb, fuel)                            # actual standard l/min of the bottle gas
        mdot = bottle_slpm * xb / 60.0 / 1000.0 * N_STP * MW[fuel] / 1000.0  # kg/s of fuel
        fuel_slpm, n2_slpm = bottle_slpm * xb, bottle_slpm * (1 - xb) + (q_n2 or 0.0)
    plan = s2.planned(s) if s else {}
    p_psia = c.get("pressure_start_psia")
    return dict(
        test=test, day=test[:5], era="2019" if test.startswith("19") else "2020-21",
        fuel_species=fuel, x_fuel_bottle=xb, burner_diameter_m=d_m,
        fuel_flow_test_n2eq_slpm=q_test, fuel_flow_ignition_n2eq_slpm=q_ign, nitrogen_test_flow_slpm=q_n2,
        fuel_mass_flow_kg_s=mdot,
        fuel_mass_flux_kg_m2_s=(mdot / area if mdot is not None and area else None),
        x_fuel_burner_stream=(fuel_slpm / (fuel_slpm + n2_slpm) if mdot is not None else None),
        burner_exit_velocity_stp_m_s=((fuel_slpm + n2_slpm) / 60.0 / 1000.0 / area if mdot is not None and area else None),
        heat_release_rate_w=(mdot * LHV_MJ_KG[fuel] * 1e6 if mdot is not None and fuel in LHV_MJ_KG else None),
        pressure_pa=(p_psia * PSIA_PA if isinstance(p_psia, (int, float)) else None),
        x_o2_ambient=c.get("x_O2"), x_n2_ambient=c.get("x_N2"), x_co2_ambient=c.get("x_CO2"),
        x_he_ambient=c.get("x_He"), x_fuel_residual_ambient=(c.get("x_ethylene") or 0.0) + (c.get("x_methane") or 0.0) if c else None,
        chamber_temperature_start_c=c.get("chamber_t_start_c"),
        igniter_power_v=num("igniterIgnitionPowerV"), igniter_duration_ms=num("igniterIgnitionDurationMs"),
        igniter_preheat_ms=num("igniterPreheatTimeMs"),
        fuel_flow_to_ignition_s=(num("fuelFlowIgnitionDeltaMs") / 1000.0 if num("fuelFlowIgnitionDeltaMs") is not None else None),
        stabilization_s=(num("stabilizationTime1Sec") or 0.0) + (num("stabilizationTime2Sec") or 0.0),
        ignition_to_test_ramp_s=num("ignitionToTestRampTimeSec"), constant_flow_time_s=num("constantFlowTimeSec"),
        planned_stop_s=plan.get("planned_stop_s"), planned_stop_branch=plan.get("branch"),
        flow_timeout_s=num("flowTimeoutSec"),
        extinction_detection_stops_test=s.get("extinctionDetectionStopsTest"),
        burner_temperature_limit_c=num("temperatureLimitC"), burner_temperature_limit_critical_c=num("temperatureLimitCriticalC"),
        flags=flags)


# ---------------------------------------------------------------------------------------------
# outcomes: flame onset, flame end, cause, heat flux
# ---------------------------------------------------------------------------------------------
def first_sustained(t, mask, start, hold, stop=math.inf):
    v = fd.first_sustained(t, mask, start, hold)
    return v if v == v and v <= stop else NAN


def measure(test, fls, inp):
    """Outcome and the 3-s prefix of one test. Times in s after the measured onset unless *_clock."""
    import numpy as np
    row = dict(status="ok", flags=[])
    if not fls["data"]:
        row["status"] = "no_data_file"
        return row
    if header_of(fls["data"]) is None:
        row["status"] = "data_without_header"
        return row
    hdr, t, d = fd.read_data(fls["data"])
    if hdr.get("Test Point") not in (None, test):
        row["status"] = "data_file_names_other_test"
        return row
    cols = list(d)
    pmts = ["PMT1", "PMT2", "PMT3"]
    hf_c = "BREHeatFluxCenter" if "BREHeatFluxCenter" in d else "Ch20" if "Ch20" in d else None
    hf_o = "BREHeatFluxOffset" if "BREHeatFluxOffset" in d else "Ch21" if "Ch21" in d else None
    if len(t) < 500 or any(c not in d for c in pmts + ["MFCA3FuelFlow"]):
        row["status"] = "short_or_unknown_columns"
        return row
    on_clock = lambda s: s + 86400 if s < t[0] - 40000 else s  # noqa: E731
    stamps, tp_name = {}, None
    if fls["tp"]:
        stamps, tp_name, _ = fd.read_tp(fls["tp"])
        if tp_name and tp_name != test:
            row["flags"].append(f"tp_names_{tp_name}")
            stamps = {}                           # a TP file of another test is not used
    else:
        row["flags"].append("no_tp")
    # ignition reference
    if "Ignition and retraction" in stamps:
        ign = on_clock(stamps["Ignition and retraction"])
    else:
        # no stamp: fuel readback on, plus the script's own fuel-to-ignition delay (fuelFlowIgnitionDeltaMs)
        on = fd.smooth(t, d["MFCA3FuelFlow"], 0.2) > 0.15
        e = first_sustained(t, on, t[0], 2.0)
        delay = inp.get("fuel_flow_to_ignition_s")
        ign = e + (delay if delay is not None else 5.0) if e == e else NAN
        row["flags"].append("ignition_reference_from_flow")
    if ign != ign:
        row["status"] = "no_ignition_reference"
        return row
    # onset: PMT sum above its pre-ignition level
    near = (t > ign - 3.0) & (t < ign + 5.0)
    if "Ignition and retraction" in stamps and (t[0] > ign - 1.0 or near.sum() < 0.8 * 8.0 * 125.0):
        row["status"] = "record_gap_at_ignition"            # the onset is not in the file
        return row
    # The PMTs idle at a 4.5 V rail until they are armed, a few seconds before ignition: the baseline is
    # the median of the armed (sum < 1 V) samples before the ignition reference, the search starts after arming.
    ps = sum(d[c] for c in pmts)
    dark = [fd.dark_level(t, d[c]) for c in pmts]
    win = (t > ign - 15) & (t < ign - 0.3)
    armed = np.where(win & (ps < 1.0))[0]
    if len(armed) >= 25:
        base, t_arm = float(np.median(ps[armed])), float(t[armed[0]])
    else:
        later = np.where((t > ign - 15) & (ps < 1.0))[0]
        base, t_arm = float(sum(dark)), (float(t[later[0]]) if len(later) else ign - 3.0)
        row["flags"].append("pmt_not_armed_before_ignition_reference")
    pre = (t > max(t_arm, ign - 15) + 0.3) & (t < ign - 0.3)   # sensor offsets: armed, before ignition
    if pre.sum() < 25:
        pre = (t > t_arm + 0.3) & (t < t_arm + 2.3)
    lo = max(ign - 3.0, t_arm + 0.3)
    hi = ign + 25.0 if "Ignition and retraction" in stamps else t[-1]
    t0 = first_sustained(t, fd.smooth(t, ps, 0.1) > base + ONSET_PMT_V, lo, ONSET_HOLD_S, hi)
    rad = [c for c in cols if c.startswith("Radiometer") and not c.endswith("CJ")]
    t0_rad = NAN
    if rad:
        rs = sum(d[c] for c in rad)
        rb = float(np.median(rs[pre]))
        t0_rad = first_sustained(t, fd.smooth(t, rs, 0.1) > rb + 0.15, lo, ONSET_HOLD_S, hi)
    if t0 != t0:
        row.update(status="ok", lit=0, onset_minus_stamp_s=NAN)
        row["flags"].append("no_flame_seen")
        return row
    gaps = np.where((np.diff(t) > 1.0) & (t[:-1] > t0))[0]
    if len(gaps):
        row["flags"].append(f"record_gap_after_onset_{float(t[gaps[0]] - t0):.0f}s")
    row.update(lit=1, onset_clock=t0, onset_minus_stamp_s=t0 - ign, onset_radiometer_minus_pmt_s=t0_rad - t0,
               file_end_s=float(t[-1]) - t0)
    # flame end: PMT sum back at its armed dark level
    ex = fd.smooth(t, sum(d[c] - k for c, k in zip(pmts, dark)), 0.2)
    pmt_end = first_sustained(t, ex < PMT_END_V, t0 + 0.5, PMT_HOLD_S)
    # flow change: first commanded departure from the test flow after the constant stage began
    pre_stage = (inp.get("stabilization_s") or 0.0) + (inp.get("ignition_to_test_ramp_s") or 0.0)
    stage0 = on_clock(stamps["Flame stabilization #1"]) if "Flame stabilization #1" in stamps else ign + STAMP_TO_STAGE_S
    t_ref = stage0 + pre_stage + 0.5
    fc = NAN
    if fls["setpoints"]:
        ts, vs = fd.read_setpoints(fls["setpoints"], t[0])
        before = np.where(ts <= t_ref)[0]
        if len(before):
            level = vs[before[-1]]
            tol = np.maximum(0.02 * np.abs(level), 0.005)
            later = np.where((ts > t_ref) & np.any(np.abs(vs - level) > tol, axis=1))[0]
            if len(later):
                fc = float(ts[later[0]])
    if fc != fc:
        row["flags"].append("no_flow_change_in_command_log")
        on = fd.smooth(t, d["MFCA3FuelFlow"], 0.2)
        lvl = float(np.median(on[(t > t_ref) & (t < t_ref + 2)])) if ((t > t_ref) & (t < t_ref + 2)).any() else NAN
        if lvl == lvl and lvl > 0.1:
            fc = first_sustained(t, np.abs(on - lvl) > 0.05 * lvl, t_ref + 2, 0.3)
    # label from the sensors: the flame went dark before the flow was cut, or the cut ended it
    if pmt_end == pmt_end and (fc != fc or pmt_end < fc - LABEL_MARGIN_S):
        s_cause = "extinction"
    elif fc == fc:
        s_cause = "operator_termination"
    else:
        s_cause = None
    ext = on_clock(stamps["Extinction detected"]) if "Extinction detected" in stamps else NAN
    off = on_clock(stamps["MFCs off"]) if "MFCs off" in stamps else NAN
    tp_class = ("extinction_detected" if ext == ext else "scripted_end" if off == off else "no_tp_stamps")
    agree = int((s_cause == "extinction") == (tp_class == "extinction_detected")) if tp_class != "no_tp_stamps" and s_cause else None
    # cause: the script's own account when it has one (its detector averages radiometers/PMTs against
    # limits, ACME-DOC-010 3.1.7.13, and its action is the flow cut); the sensors otherwise and for the time.
    # A detection while the PMTs were still lit is an extinction ended at the cut (flagged).
    if tp_class == "extinction_detected":
        cause = "extinction"
        if s_cause != "extinction":
            row["flags"].append("extinction_detected_while_pmt_lit")
    elif tp_class == "scripted_end":
        cause = s_cause
        if s_cause == "extinction":
            row["flags"].append("sensor_extinction_before_scripted_end")
    else:
        cause = s_cause
    if cause == "extinction":
        end, cens = (pmt_end if pmt_end == pmt_end and (fc != fc or pmt_end < fc) else fc), "event"
    elif cause == "operator_termination":
        end, cens = fc, "event"
    else:
        end, cens = float(t[-1]), "right"
        row["flags"].append("burning_at_file_end")
    if cause == "extinction" and end < t_ref:
        row["flags"].append("ended_before_constant_stage")
    if pmt_end == pmt_end and fc == fc and 0.0 <= fc - pmt_end < 2 * LABEL_MARGIN_S:
        row["flags"].append("end_just_before_flow_change")
    planned = inp.get("planned_stop_s")
    row.update(end_s=end - t0, cause=cause, censoring=cens, pmt_end_s=pmt_end - t0, flow_change_s=fc - t0,
               tp_class=tp_class, tp_extinction_detected_s=ext - t0, tp_mfcs_off_s=off - t0, labels_agree=agree,
               label_sensor=s_cause,
               stop_minus_planned_s=(fc - ign - planned if fc == fc and planned is not None and "Ignition and retraction" in stamps else NAN))
    # heat flux (centre and offset gauges), pre-ignition offset removed
    if hf_c:
        def q(c, a, b):
            w = (t >= t0 + a) & (t < t0 + b)
            return float((np.mean(d[c][w]) - np.median(d[c][pre])) * 1000.0 * BTU_FT2_S_KW_M2) if w.sum() > 10 else NAN
        lit_through = (end - t0) > T2_WINDOW[1]
        row.update(q_center_8_12s_kw_m2=q(hf_c, *T2_WINDOW) if lit_through else NAN,
                   q_offset_8_12s_kw_m2=q(hf_o, *T2_WINDOW) if (lit_through and hf_o) else NAN,
                   prefix_q_center_2_3s_kw_m2=q(hf_c, *PREFIX_WINDOW), prefix_q_offset_2_3s_kw_m2=q(hf_o, *PREFIX_WINDOW) if hf_o else NAN,
                   heat_flux_channel=hf_c)
        if not lit_through:
            row["flags"].append("t2_flame_ended_before_window")
    else:
        row["flags"].append("no_heat_flux_channel")
    return row


def clean(v):
    if isinstance(v, float):
        return None if v != v else float(f"{v:.6g}")
    if isinstance(v, list):
        return [clean(x) for x in v]
    return v


OUTCOME_KEYS = ["status", "lit", "end_s", "cause", "censoring", "pmt_end_s", "flow_change_s", "tp_class",
                "tp_extinction_detected_s", "tp_mfcs_off_s", "labels_agree", "label_sensor", "stop_minus_planned_s",
                "onset_minus_stamp_s", "onset_radiometer_minus_pmt_s", "file_end_s",
                "q_center_8_12s_kw_m2", "q_offset_8_12s_kw_m2"]
PREFIX_KEYS = ["prefix_q_center_2_3s_kw_m2", "prefix_q_offset_2_3s_kw_m2", "heat_flux_channel"]


def cmd_build(_args):
    part = partitions()
    keep = {d for d, p in part.items() if p in ("train", "validation")}
    cat = catalogue_tests()
    days = day_conditions()
    calc, calc_dup = calculator_rows(keep)
    OUT.mkdir(parents=True, exist_ok=True)
    rows = {"train": [], "validation": []}
    counts = {p: Counter() for p in ("train", "validation", "test")}
    for test in sorted(cat):
        p = part.get(test[:5], "held")
        c = counts.setdefault(p, Counter())
        c["tests_in_catalogue"] += 1
        has = cat[test]
        text_ok = has["data"] and has["tp"] and has["script"]
        c["with_data_tp_script_in_catalogue"] += int(bool(text_ok))
        c["with_data_script_in_catalogue"] += int(bool(has["data"] and has["script"]))
        c["image_video_bytes"] += has["bytes_image_video"]
        if p not in ("train", "validation"):
            continue                                   # test day: nothing opened
        fls = files_of(test)
        inp = inputs_of(test, fls, days[test[:5]], calc, calc_dup)
        out = measure(test, fls, inp)
        prefix = {k: clean(out.get(k)) for k in PREFIX_KEYS}
        outcome = {k: clean(out.get(k)) for k in OUTCOME_KEYS}
        outcome["measurement_flags"] = out.pop("flags")   # kept with the outcome: some flags reveal it
        req = ["fuel_mass_flow_kg_s", "pressure_pa", "x_o2_ambient", "burner_diameter_m", "planned_stop_s"]
        inputs_ok = all(inp.get(k) is not None for k in req)
        c["inputs_complete"] += int(inputs_ok)
        t1_ok = outcome["status"] == "ok" and outcome["lit"] == 1 and outcome["censoring"] in ("event", "right")
        c["t1_outcome"] += int(t1_ok)
        c["t1_inputs_and_outcome"] += int(t1_ok and inputs_ok)
        t2_ok = outcome.get("q_center_8_12s_kw_m2") is not None
        c["t2_outcome"] += int(t2_ok)
        c["t2_inputs_and_outcome"] += int(t2_ok and inputs_ok)
        c["measured_status_" + outcome["status"]] += 1
        c["with_heat_flux_channel"] += int(prefix.get("heat_flux_channel") is not None)
        rows[p].append(dict(test=test, day=test[:5], partition=p, inputs_complete=inputs_ok,
                            inputs={k: clean(v) for k, v in inp.items() if k not in ("test", "day", "flags")},
                            prefix_3s=prefix, flags=inp["flags"], outcome=outcome))
    def dump(name, rr):
        with (OUT / name).open("w", encoding="utf-8") as f:
            for r in rr:
                f.write(json.dumps(r, sort_keys=True) + "\n")
    dump("cases_train.jsonl", rows["train"])
    dump("cases_validation_inputs.jsonl", [{k: v for k, v in r.items() if k != "outcome"} for r in rows["validation"]])
    dump("cases_validation_outcomes.jsonl", [dict(test=r["test"], day=r["day"], outcome=r["outcome"]) for r in rows["validation"]])
    counts = {p: dict(c) for p, c in counts.items()}
    for p in ("validation",):  # keep validation counts to availability only (no outcome value is summarised)
        counts[p] = {k: v for k, v in counts[p].items() if not k.startswith(("measured_status_", "t2_"))}  # T2 needs survival to 12 s
    (OUT / "counts.json").write_text(json.dumps(counts, indent=1, sort_keys=True), encoding="utf-8")
    tr = rows["train"]
    print(json.dumps(dict(
        wrote=str(OUT.relative_to(REPO)), counts=counts,
        train_input_flags=dict(Counter(re.sub(r"_names_.*|_vs_table.*|_calc_.*", "_x", f) for r in tr for f in r["flags"])),
        train_measurement_flags=dict(Counter(re.sub(r"_names_.*|_onset_.*", "_x", f) for r in tr for f in r["outcome"]["measurement_flags"])),
    ), indent=1, sort_keys=True))
    return 0


# ---------------------------------------------------------------------------------------------
# scoring
# ---------------------------------------------------------------------------------------------
def pinball(q, y, tau):
    return (tau - (1.0 if y < q else 0.0)) * (y - q)


def crps_quantiles(levels, q, y):
    """`scores::crps_quantiles`: 2 * sum_k w_k pinball, w_k = (tau_{k+1} - tau_{k-1}) / 2."""
    if not levels or not math.isfinite(y) or any(not math.isfinite(v) for v in q):
        return NAN
    o = sorted(range(len(levels)), key=lambda i: levels[i])
    t, qq = [levels[i] for i in o], [q[i] for i in o]
    ext = [0.0] + t + [1.0]
    return 2.0 * sum((ext[k + 2] - ext[k]) / 2.0 * pinball(qq[k], y, t[k]) for k in range(len(t)))


def t1_cases(rows):
    """(test, cutoff) cases at risk: a lit test whose endpoint is later than the cutoff."""
    out = []
    for r in rows:
        o = r["outcome"]
        if o["status"] != "ok" or o.get("lit") != 1 or o["censoring"] not in ("event", "right") or o["end_s"] is None:
            continue
        for c in CUTOFFS:
            if o["end_s"] > c:
                ev = o["censoring"] == "event"
                out.append(dict(test=r["test"], day=r["day"], cutoff_s=c,
                                outcome=dict(censoring=o["censoring"], t_lower=o["end_s"],
                                             t_upper=o["end_s"] if ev else None,
                                             causes=[o["cause"]] if ev else [])))
    return out


def t2_cases(rows):
    return [dict(test=r["test"], day=r["day"], y=r["outcome"]["q_center_8_12s_kw_m2"]) for r in rows
            if r["outcome"].get("q_center_8_12s_kw_m2") is not None]


def _by_day(items):
    g = defaultdict(list)
    for day, v in items:
        if math.isfinite(v):
            g[day].append(v)
    per_day = {d: sum(v) / len(v) for d, v in sorted(g.items())}
    return per_day, (sum(per_day.values()) / len(per_day) if per_day else NAN)


def score(target, forecasts, rows):
    """Aggregate and per-day scores of `forecasts` on the cases built from `rows` (case-file rows with
    an outcome). T1 times and horizons are seconds after the measured onset, as for CFI-G. Returns
    {aggregate, session_mean, per_day, n, missing}. For T1, per cause: the IPCW integrated Brier over
    all cases (aggregate), and per day the mean of the per-case integrated contributions."""
    if target == "t1":
        cases = t1_cases(rows)
        fmap = {(f["test"], float(f["cutoff_s"])): f for f in forecasts}
        outcomes = [k["outcome"] for k in cases]
        missing = sum(1 for k in cases if (k["test"], k["cutoff_s"]) not in fmap)
        res = dict(n_cases=len(cases), missing_forecasts=missing, horizons_s=HORIZONS, causes={})
        for cause in CAUSES:
            cif = [[(fmap[(k["test"], k["cutoff_s"])]["cif"][cause][j] if (k["test"], k["cutoff_s"]) in fmap else NAN)
                    for j in range(len(HORIZONS))] for k in cases]
            per_h, contrib = [], [[NAN] * len(HORIZONS) for _ in cases]
            for j, h in enumerate(HORIZONS):
                col = [row[j] for row in cif]
                per_h.append(ipcw_brier(col, outcomes, cause, h)["brier"])
                elig, w, ind, _, _ = ipcw_terms(outcomes, cause, h)
                for i in range(len(cases)):
                    if elig[i] and math.isfinite(col[i]):
                        contrib[i][j] = w[i] * (ind[i] - col[i]) ** 2
            per_case = [integrated_brier(HORIZONS, row) for row in contrib]
            per_day, smean = _by_day([(k["day"], v) for k, v in zip(cases, per_case)])
            res["causes"][cause] = dict(integrated_brier=integrated_brier(HORIZONS, per_h), session_mean=smean,
                                        brier_per_horizon=per_h, per_day=per_day)
        res["integrated_brier_mean_of_causes"] = sum(res["causes"][c]["integrated_brier"] for c in CAUSES) / len(CAUSES)
        return res
    if target == "t2":
        cases = t2_cases(rows)
        fmap = {f["test"]: f for f in forecasts}
        vals = [(k["day"], crps_quantiles(LEVELS, fmap[k["test"]]["quantiles"], k["y"]) if k["test"] in fmap else NAN)
                for k in cases]
        fin = [v for _, v in vals if math.isfinite(v)]
        per_day, smean = _by_day(vals)
        return dict(n_cases=len(cases), missing_forecasts=len(cases) - len(fin), unit="kW/m2",
                    crps=sum(fin) / len(fin) if fin else NAN, session_mean=smean, per_day=per_day)
    raise ValueError(target)


def read_jsonl(path):
    with open(path, encoding="utf-8") as f:
        return [json.loads(x) for x in f if x.strip()]


# ---------------------------------------------------------------------------------------------
# baselines (constants from TRAIN only)
# ---------------------------------------------------------------------------------------------
def quantile(xs, q):
    xs = sorted(xs)
    if not xs:
        return NAN
    p = q * (len(xs) - 1)
    i = int(math.floor(p))
    return xs[i] if i + 1 >= len(xs) else xs[i] + (xs[i + 1] - xs[i]) * (p - i)


def cmd_baselines(_args):
    rows = read_jsonl(OUT / "cases_train.jsonl")
    cases = t1_cases(rows)
    inputs = {r["test"]: r["inputs"] for r in rows}
    # B1 incidence: per cutoff, the train fraction of cases at risk with an event of the cause by h
    inc = {}
    for c in CUTOFFS:
        sel = [k["outcome"] for k in cases if k["cutoff_s"] == c]
        inc[c] = {cause: [sum(1 for o in sel if o["censoring"] == "event" and o["causes"][0] == cause
                              and o["t_upper"] <= h) / len(sel) for h in HORIZONS] for cause in CAUSES}
    f_inc = [dict(test=k["test"], cutoff_s=k["cutoff_s"], cif=inc[k["cutoff_s"]]) for k in cases]
    # B2 planned stop: extinction incidence of B1, cut at the script's planned stop; the rest of the mass is
    # an operator termination at the planned stop plus the train median offset of measured stops from it
    offs = [r["outcome"]["stop_minus_planned_s"] for r in rows if r["outcome"].get("cause") == "operator_termination"
            and r["outcome"].get("stop_minus_planned_s") is not None]
    delta = quantile(offs, 0.5)
    onset_off = quantile([r["outcome"]["onset_minus_stamp_s"] for r in rows if r["outcome"].get("onset_minus_stamp_s") is not None], 0.5)
    f_plan, n_noplan = [], 0
    for k in cases:
        ps = inputs[k["test"]].get("planned_stop_s")
        if ps is None:
            n_noplan += 1
            f_plan.append(dict(test=k["test"], cutoff_s=k["cutoff_s"], cif=inc[k["cutoff_s"]]))
            continue
        stop = ps + delta - onset_off                       # on the onset clock
        fe = inc[k["cutoff_s"]]["extinction"]
        fe_stop = fe[max([j for j, h in enumerate(HORIZONS) if h <= stop], default=0)] if stop >= HORIZONS[0] else 0.0
        ext = [fe[j] if h < stop else fe_stop for j, h in enumerate(HORIZONS)]
        op = [(1.0 - fe_stop) if h >= stop else 0.0 for h in HORIZONS]
        f_plan.append(dict(test=k["test"], cutoff_s=k["cutoff_s"], cif=dict(extinction=ext, operator_termination=op)))
    s_inc, s_plan = score("t1", f_inc, rows), score("t1", f_plan, rows)
    # T2 baselines
    t2 = t2_cases(rows)
    ys = [k["y"] for k in t2]
    qc = [quantile(ys, q) for q in LEVELS]
    f_const = [dict(test=k["test"], quantiles=qc) for k in t2]
    pre = {r["test"]: r["prefix_3s"]["prefix_q_center_2_3s_kw_m2"] for r in rows}
    ratios = [k["y"] / pre[k["test"]] for k in t2 if pre.get(k["test"]) and pre[k["test"]] > 0]
    rq = [quantile(ratios, q) for q in LEVELS]
    f_pers = [dict(test=k["test"], quantiles=[pre[k["test"]] * r for r in rq]) if pre.get(k["test"]) and pre[k["test"]] > 0
              else dict(test=k["test"], quantiles=qc) for k in t2]
    s_const, s_pers = score("t2", f_const, rows), score("t2", f_pers, rows)
    res = dict(
        note="constants computed on TRAIN days only; scored in-sample on TRAIN (no validation outcome read)",
        t1=dict(cases=len(cases), cases_by_cutoff=dict(Counter(k["cutoff_s"] for k in cases)),
                incidence=dict(constants={str(c): inc[c] for c in CUTOFFS}, score=s_inc),
                planned_stop=dict(constants=dict(stop_minus_planned_median_s=delta, onset_minus_stamp_median_s=onset_off,
                                                 cases_without_planned_stop=n_noplan), score=s_plan)),
        t2=dict(cases=len(t2), constant=dict(quantiles_kw_m2=qc, score=s_const),
                persistence=dict(ratio_quantiles=rq, score=s_pers)))
    (OUT / "baselines_train.json").write_text(json.dumps(res, indent=1, default=lambda x: None), encoding="utf-8")
    short = lambda s: {c: dict(ibs=round(s["causes"][c]["integrated_brier"], 4), session_mean=round(s["causes"][c]["session_mean"], 4)) for c in CAUSES}  # noqa: E731
    print(json.dumps(dict(t1_cases=len(cases), t1_incidence=short(s_inc), t1_planned_stop=short(s_plan),
                          planned_stop_constants=res["t1"]["planned_stop"]["constants"],
                          t2_cases=len(t2), t2_constant=dict(crps=round(s_const["crps"], 3), session_mean=round(s_const["session_mean"], 3), q=[round(x, 2) for x in qc]),
                          t2_persistence=dict(crps=round(s_pers["crps"], 3), session_mean=round(s_pers["session_mean"], 3), ratio_q=[round(x, 3) for x in rq])),
                     indent=1))
    return 0


def cmd_score(args):
    if args.partition == "validation":
        if not args.look:
            sys.exit("validation outcomes are scored only in a look registered in docs/wsp/PROTOCOL.md: pass --look <section>")
        outs = {r["test"]: r["outcome"] for r in read_jsonl(OUT / "cases_validation_outcomes.jsonl")}
        rows = [dict(r, outcome=outs[r["test"]]) for r in read_jsonl(OUT / "cases_validation_inputs.jsonl")]
    else:
        rows = read_jsonl(OUT / "cases_train.jsonl")
    res = score(args.target, read_jsonl(args.forecasts), rows)
    if args.partition == "validation":   # aggregate only
        res = {k: v for k, v in res.items() if k not in ("per_day",)}
        for c in res.get("causes", {}).values():
            c.pop("per_day", None)
    print(json.dumps(res, indent=1, default=lambda x: None))
    return 0


def cmd_selftest(_args):
    rows = [dict(test="T1", day="A", outcome=dict(status="ok", lit=1, censoring="event", end_s=10.0, cause="extinction",
                                                  q_center_8_12s_kw_m2=5.0)),
            dict(test="T2", day="B", outcome=dict(status="ok", lit=1, censoring="event", end_s=100.0, cause="operator_termination",
                                                  q_center_8_12s_kw_m2=3.0))]
    perfect = []
    for k in t1_cases(rows):
        te, cause = k["outcome"]["t_upper"], k["outcome"]["causes"][0]
        perfect.append(dict(test=k["test"], cutoff_s=k["cutoff_s"],
                            cif={c: [1.0 if (c == cause and h >= te) else 0.0 for h in HORIZONS] for c in CAUSES}))
    s = score("t1", perfect, rows)
    assert all(abs(s["causes"][c]["integrated_brier"]) < 1e-12 for c in CAUSES), s
    assert s["n_cases"] == 6
    s2_ = score("t2", [dict(test="T1", quantiles=[5.0] * 5), dict(test="T2", quantiles=[3.0] * 5)], rows)
    assert abs(s2_["crps"]) < 1e-12
    assert abs(crps_quantiles([0.5], [0.0], 1.0) - 0.5) < 1e-12  # 2 * w(0.5) * pinball = 2 * 0.5 * 0.5
    print("selftest ok")
    return 0


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    sub = ap.add_subparsers(dest="cmd", required=True)
    sub.add_parser("build").set_defaults(fn=cmd_build)
    sub.add_parser("baselines").set_defaults(fn=cmd_baselines)
    sub.add_parser("selftest").set_defaults(fn=cmd_selftest)
    p = sub.add_parser("score")
    p.add_argument("--target", choices=["t1", "t2"], required=True)
    p.add_argument("--forecasts", required=True)
    p.add_argument("--partition", choices=["train", "validation"], default="train")
    p.add_argument("--look", help="the PROTOCOL.md section that registers this validation look")
    p.set_defaults(fn=cmd_score)
    a = ap.parse_args()
    return a.fn(a)


if __name__ == "__main__":
    sys.exit(main())
