#!/usr/bin/env python3
"""ACME E-FIELD Flames (PSI-22): forecast cases, outcomes measured from the raw text files, baselines and
the score. Preparation for a blind forecast; it changes no model and fits nothing except the constants of
the named baselines, on train days only.

Two targets (docs/wsp/feedback/efield_feasibility.md):

  T1  ion current at a commanded mesh voltage. A case is (test, planned level): the levels are read from
      the script parameter file `<T>_VoltageSweep.txt` before ignition (the initial voltage, then every step
      of a stepped sweep, or the ramp's start, every multiple of 500 V inside it and its end), first pass
      only. Outcome: the median ion current (uA, in the direction the field drives it, so positive for
      both polarities) over the second half of the step's plateau (steps), or over +-0.25 s around the
      ramp's crossing of the level (ramps). Scored only where the flame was burning through the window
      (the forecast is conditional on a burning flame; T2 is the event that removes it). CRPS from the
      quantiles 5, 25, 50, 75, 95 %.
  T2  the flame ends under the field. Clock: seconds since the field is switched on (the first change of
      the commanded E-field output after ignition). Window: the planned field programme
      (`window_s` = efieldInitializeTimeSec + constantFlowTimeSec + the sweep's planned length), ended
      earlier by the command itself returning to rest (field off), an unplanned flow change or the end of
      the record. Event: the photomultipliers go dark inside the window (the flame_design.py rule: PMT sum
      back within 0.05 V of its dark level for 0.5 s; a flame that goes dark within 0.5 s after a flow
      command is not an event). Scored as an integrated Brier score over the horizons 2..150 s that were
      observed (pooled per horizon, trapezoid over horizons with at least 5 tests), and as a Brier score
      for "ends before the field programme ends" on tests not censored by a flow change or the record.

  A test whose flame is not lit when the field comes on (PMT rule) has status flame_not_lit or
  flame_out_before_field and no T1/T2 case. Step levels: start + k x step, k = 0 .. floor(span / |step|);
  a level shorter than half its dwell, or never commanded, is `not_reached` and not scored; a window whose
  raw ion-current reading touches 1.45 V (the channel range is +-1.5 V) is `adc_clipped` and not scored.

Conversions are NASA's (ACME-DOC-010 Rev D, section 10.2): IonCurrent uA = 1000 V / 162; EfieldVoltage
V = 2486.1795 v + 141.9733; EfieldSetpointOut V = C s^2 + B s + A with (A, B, C) = (-10886.5, 1048.203,
2.7365) for the negative supply and (-8.79, 1074.01, -1.594) for the positive supply; MFC slpm = v / 5 x
full scale (header). Header codes from the same document (SystemDefineSetup): Burner 3 = coflow, 5 = gas
jet 2.1 mm; EFieldPol 1 = negative, 2 = positive; EField 1 = mesh at 35 mm, 2 = 40 mm; MFCA3 gas 0 =
methane, 1 = 30/70 and 2 = 40/60 methane/N2, 4 = ethylene, 5 = 30/70 ethylene/N2.

Partitions: docs/wsp/atlas/splits/PSI-22.json. Train days: everything is read. Validation days: the same
files are read by this script, but their outcomes go only to cases_validation_outcomes.jsonl and nothing
about them is printed. Test days and days not in the split file: no file is opened (counts come from the
file catalogue in `counts`).

    python -I tools/wsp/ext/efield_cases.py build        # writes the three case files (prints train counts only)
    python -I tools/wsp/ext/efield_cases.py counts       # inputs/outcome availability by partition (catalogue for test days)
    python -I tools/wsp/ext/efield_cases.py baselines    # baselines fitted and scored on train days only
    python -I tools/wsp/ext/efield_cases.py score --forecasts F.jsonl [--partition train]

Forecast file (one JSON object per line):
    {"test": "18080A1", "t1": {"<level_id>": [q05, q25, q50, q75, q95], ...},
     "t2": {"cif": {"<horizon_s>": p, ...}, "p_end_in_window": p}}
"""
import argparse
import json
import math
import re
import sys
from collections import Counter, defaultdict
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(Path(__file__).resolve().parent))
import flame_design as fd  # noqa: E402  (readers and the PMT rule of the sibling tool; nothing of PSI-10 is read)

INV = "PSI-22"
RAW = REPO / "runs/dev/data_all/raw" / INV / "Raw Data"
TABLE = REPO / "runs/dev/data_all/raw" / INV / "Experimental table" / "PSI-22_Experimental table_ACME E-FIELD Flames.csv"
CATALOGUE = REPO / "runs/dev/data_all/work/atlas/files.jsonl"
SPLIT = REPO / "docs/wsp/atlas/splits/PSI-22.json"
OUT = REPO / "runs/dev/data_all/work/transfer_efield"
PRE = f"{INV}_Raw Data_"

# --- NASA conversions (ACME-DOC-010 Rev D, 10.2) ------------------------------------------------------
ION_UA_PER_V = 1000.0 / 162.0
EV_A, EV_B = 2486.1795, 141.9733
SETPOINT = {"negative": (-10886.5, 1048.203, 2.7365), "positive": (-8.79, 1074.01, -1.594)}
ION_ADC_LIMIT_V = 1.45          # the channel's range is +-1.5 V; above this the reading may be clipped
POL = {"1": "negative", "2": "positive"}
BURNER = {"3": "coflow", "5": "gas_jet_2.1mm", "4": "gas_jet_1.3mm"}
MESH = {"0": "removed", "1": "35mm", "2": "40mm"}
FUEL_GAS = {"0": ("methane", 1.0), "1": ("methane", 0.30), "2": ("methane", 0.40), "3": ("methane", 0.45),
            "4": ("ethylene", 1.0), "5": ("ethylene", 0.30), "6": ("ethylene", 0.50)}
CARBON = {"methane": 1, "ethylene": 2}
# Burner tube: inner diameter 0.085 in, 3.66e-6 m2 (data interpretation document p. 8; the gas-jet burner
# uses the same tubing). Coflow annulus 4.86e-4 m2 (same page).
FUEL_TUBE_AREA_M2 = 3.66e-6
MOLAR_VOLUME_STP_M3 = 0.022414   # MFC standard conditions 0 C, 760 torr (data interpretation document p. 8)
E_CHARGE = 1.602176634e-19
AVOGADRO = 6.02214076e23

# --- typed measurement rules ---------------------------------------------------------------------------
SETPOINT_CHANGE_V = 0.01        # the commanded output departs from its pre-field value: field on
RAMP_GRID_V = 500.0             # ramp levels: start, every multiple of 500 V inside, end
RAMP_HALF_WINDOW_S = 0.25
RAMP_LEVEL_TOL_V = 20.0         # a ramp level counts as crossed when the command comes within 20 V of it
LIT_BEFORE_FIELD_S = 1.0        # the flame must be lit (PMT rule) through the second before the field is on
FIELD_OFF_HOLD_S = 0.5          # the command back at rest this long: the field programme has ended
STEP_TOL_FRAC, STEP_TOL_MIN_V = 0.3, 40.0
MIN_PLATEAU_FRAC = 0.5          # a step counts as reached when its plateau lasts half its planned dwell
PRE_FIELD_S = 2.0               # the sensor block a forecast may see: the 2 s before the field is switched on
HORIZONS_S = [2, 5, 10, 15, 20, 30, 45, 60, 90, 120, 150]
# "ends in the window" is scored where the field programme ran to its end: no censoring, or a programme that
# the command itself ended a little before the planned length (a command, not an outcome)
CLOSED = (None, "field_off_early")
LEVELS_Q = [0.05, 0.25, 0.5, 0.75, 0.95]
NAN = float("nan")


# =====================================================================================================
# partitions and files
# =====================================================================================================
def partitions():
    g = json.loads(SPLIT.read_text(encoding="utf-8"))["strata"]["all"]["groups"]
    return {x.split(":")[-1]: p for p, xs in g.items() for x in xs}


def catalogue_tests():
    """Test ids and their raw text-file kinds, from the file catalogue (names only; nothing opened)."""
    tests = defaultdict(set)
    for line in CATALOGUE.open(encoding="utf-8"):
        r = json.loads(line)
        if r.get("investigation") != INV or r.get("category") != "Raw Data":
            continue
        m = re.match(rf"{re.escape(PRE)}(\d{{5}}[A-Z]\d+)_(data\.csv|VoltageSweep\.txt|TP\.txt|mfcSetpoints\.csv)$",
                     r["file_name"])
        if m:
            tests[m.group(1)].add(m.group(2))
    return tests


def day_table():
    """Condition columns of the experiment table (Flames Ignited and Notes are not read)."""
    import csv
    out = {}
    with TABLE.open(encoding="utf-8-sig", newline="") as f:
        for r in csv.DictReader(f):
            d = (r.get("Test Day") or "").strip()
            if d and d not in out:
                out[d] = dict(test_type=r["Test Type"].strip(), mesh_polarity=r["Mesh Polarity"].strip(),
                              fuel_bottle=r["Fuel Bottle"].strip())
    return out


def guard(test, part):
    if part.get(test[:5]) not in ("train", "validation"):
        raise SystemExit(f"{test}: day {test[:5]} is not a train or validation day; not opened")


# =====================================================================================================
# readers
# =====================================================================================================
def num(x):
    try:
        return float(x)
    except (TypeError, ValueError):
        return NAN


def read_script(path):
    return fd.read_script(path)


def read_header(path):
    with open(path, encoding="utf-8", errors="replace") as f:
        return fd.read_header(f.readline())


def setpoint_to_volts(s, polarity):
    a, b, c = SETPOINT[polarity]
    return c * s * s + b * s + a


# =====================================================================================================
# the plan (script only: known before ignition)
# =====================================================================================================
def plan(script, polarity):
    """Planned levels of the first pass and the planned length of the field programme."""
    g = {k: num(script.get(k)) for k in ("efieldInitializeVolts", "efieldInitializeTimeSec", "efieldMinVoltage",
                                         "efieldMaxVoltage", "efieldStepVoltage", "efieldStepDurationSecs",
                                         "efieldRampRatePerSec", "efieldRampDurationSecs", "efieldHoldDurationSecs",
                                         "efieldCycles")}
    if script.get("efieldCommandManually", "false") != "false" or any(v != v for v in g.values()):
        return dict(kind="unreadable", levels=[], window_s=NAN, params=g)
    lo, hi = g["efieldMinVoltage"], g["efieldMaxVoltage"]
    lo, hi = min(lo, hi), max(lo, hi)
    cyc = max(1, int(g["efieldCycles"]))
    # The initial voltage is held through the constant-flow stage: the sweep starts efieldInitializeTimeSec
    # + constantFlowTimeSec after the field is switched on (within 0.7 s on every train test read).
    cf = num(script.get("constantFlowTimeSec"))
    t0 = g["efieldInitializeTimeSec"] + (cf if cf == cf else 0.0)
    levels = [dict(level_id="init", kind="init", v_cmd=g["efieldInitializeVolts"], dwell_s=t0, t_planned_s=0.0)]
    step, rate = g["efieldStepVoltage"], g["efieldRampRatePerSec"]
    if rate != 0:
        kind = "ramp"
        start, sgn = (lo, 1.0) if rate > 0 else (hi, -1.0)
        end = start + rate * g["efieldRampDurationSecs"]
        end = min(max(end, lo), hi)
        vs = [start]
        k = math.floor(min(start, end) / RAMP_GRID_V) + 1
        while k * RAMP_GRID_V < max(start, end):
            vs.append(k * RAMP_GRID_V)
            k += 1
        vs = sorted(set(vs[1:]), reverse=sgn < 0)
        vs = [start] + [v for v in vs if v != start] + ([end] if end not in vs and end != start else [])
        for i, v in enumerate(vs):
            levels.append(dict(level_id=f"r{i:02d}", kind="ramp", v_cmd=v, dwell_s=0.0,
                               t_planned_s=t0 + abs(v - start) / abs(rate)))
        one_pass = abs(end - start) / abs(rate) + g["efieldHoldDurationSecs"]
        window = t0 + cyc * one_pass
    elif step != 0:
        kind = "step"
        start, sgn = (lo, 1.0) if step > 0 else (hi, -1.0)
        # The staircase runs for (span / |step|) dwells per pass (read from the train records: the sweep
        # stops there, and a step cut to less than half its dwell is not counted as reached), so its
        # levels are start + k step for k = 0 .. floor(span / |step|).
        span = hi - lo
        n = int(math.floor(span / abs(step) + 1e-9))
        vs = [start + k * step for k in range(n + 1)]
        d = g["efieldStepDurationSecs"]
        for i, v in enumerate(vs):
            levels.append(dict(level_id=f"s{i:02d}", kind="step", v_cmd=v, dwell_s=d,
                               t_planned_s=t0 + i * d))
        window = t0 + cyc * span / abs(step) * d
    else:
        kind, window = "hold", t0
    return dict(kind=kind, levels=levels, window_s=window, params=g, cycles=cyc)


def planned_v_max(script):
    """Largest |voltage| the script's field programme commands, from `<T>_VoltageSweep.txt` alone (written
    before ignition; works on any test day). Independent of `plan()`'s level list on purpose, so the two can
    be checked against each other. Returns (v_max_planned, v_bound_script):
      v_max_planned   max(|efieldInitializeVolts|, |start|, |end|) where, for a stepped sweep, start is the
                      minimum (step > 0) or maximum (step < 0) bound and end = start + floor(span/|step|) step;
                      for a ramp, start likewise by the sign of the rate and end = start + rate x
                      efieldRampDurationSecs clipped to the bounds; with neither, the initial voltage only.
                      Cycles go back and forth between start and end, so they add no larger magnitude.
      v_bound_script  max(|efieldMinVoltage|, |efieldMaxVoltage|), the programme's bounds as typed.
    """
    g = {k: num(script.get(k)) for k in ("efieldInitializeVolts", "efieldMinVoltage", "efieldMaxVoltage",
                                         "efieldStepVoltage", "efieldRampRatePerSec", "efieldRampDurationSecs")}
    if any(v != v for v in g.values()):
        return NAN, NAN
    lo, hi = sorted((g["efieldMinVoltage"], g["efieldMaxVoltage"]))
    step, rate = g["efieldStepVoltage"], g["efieldRampRatePerSec"]
    ends = [g["efieldInitializeVolts"]]
    if rate != 0:
        start = lo if rate > 0 else hi
        ends += [start, min(max(start + rate * g["efieldRampDurationSecs"], lo), hi)]
    elif step != 0:
        start = lo if step > 0 else hi
        ends += [start, start + math.floor((hi - lo) / abs(step) + 1e-9) * step]
    return max(abs(v) for v in ends), max(abs(lo), abs(hi))


def held_v_max(script):
    """Largest |voltage| the programme holds for at least half a dwell, by the duration rule read from the train
    records (a stepped pass lasts span/|step| dwells, so the last level held for half a dwell or more is
    k = ceil(span/|step| - 0.5) - 1). Script only, known before the test. Ramps and the initial voltage as in
    `planned_v_max`. Within one step of the highest level actually reached on train (see
    docs/wsp/feedback/efield_planned_fields.md)."""
    g = {k: num(script.get(k)) for k in ("efieldInitializeVolts", "efieldMinVoltage", "efieldMaxVoltage",
                                         "efieldStepVoltage", "efieldRampRatePerSec")}
    if any(v != v for v in g.values()):
        return NAN
    if g["efieldStepVoltage"] == 0 or g["efieldRampRatePerSec"] != 0:
        return planned_v_max(script)[0]
    lo, hi = sorted((g["efieldMinVoltage"], g["efieldMaxVoltage"]))
    step = g["efieldStepVoltage"]
    start = lo if step > 0 else hi
    k = max(0, math.ceil((hi - lo) / abs(step) - 0.5 - 1e-9) - 1)
    return max(abs(g["efieldInitializeVolts"]), abs(start), abs(start + k * step))


# =====================================================================================================
# measurement (train and validation days only)
# =====================================================================================================
def measure(test):
    """Inputs and outcomes of one test. Returns (inputs, outcomes, status)."""
    import numpy as np
    paths = {k: RAW / f"{PRE}{test}_{k}" for k in ("data.csv", "VoltageSweep.txt", "TP.txt", "mfcSetpoints.csv")}
    if not paths["data.csv"].exists() or not paths["VoltageSweep.txt"].exists():
        return None, None, "no_data_or_script"
    script = read_script(paths["VoltageSweep.txt"])
    hdr, t, d = fd.read_data(paths["data.csv"])
    need = ["IonCurrent", "EfieldVoltage", "EfieldSetpointOut", "PMT1", "PMT2", "PMT3", "MFCA3FuelFlow"]
    if len(t) < 500 or any(c not in d for c in need):
        return None, None, "short_or_unknown_columns"
    polarity = POL.get(hdr.get("EFieldPol", ""), "unknown")
    if polarity == "unknown":
        return None, None, "unknown_polarity"
    gas = FUEL_GAS.get(hdr.get("MFCA3 Gas", ""))
    burner = BURNER.get(hdr.get("Burner", ""), "unknown")
    pl = plan(script, polarity)

    # ---- inputs from the script and header (known before ignition) ---------------------------------
    u_fuel = num(script.get("velocityMixFinalMPerSec"))
    u_air = num(script.get("velocityAirFinalMPerSec"))
    x_fuel = num(script.get("concentrationFinal"))
    species = gas[0] if gas else "unknown"
    n_c = CARBON.get(species, 0)
    fuel_mol_s = u_fuel * FUEL_TUBE_AREA_M2 * x_fuel / MOLAR_VOLUME_STP_M3 if gas else NAN
    inputs = dict(
        test=test, day=test[:5], burner=burner, polarity=polarity, mesh_position=MESH.get(hdr.get("EField", ""), "unknown"),
        fuel=species, fuel_bottle_fraction=gas[1] if gas else NAN, x_fuel=x_fuel,
        u_fuel_m_s=u_fuel, u_coflow_m_s=u_air, u_fuel_ignition_m_s=num(script.get("velocityMixIgnitionMPerSec")),
        x_fuel_ignition=num(script.get("concentrationInitial")),
        fuel_flow_mol_s=fuel_mol_s, carbon_flow_atoms_s=fuel_mol_s * n_c * AVOGADRO if gas else NAN,
        constant_flow_s=num(script.get("constantFlowTimeSec")), flow_timeout_s=num(script.get("flowTimeoutSec")),
        extinction_stops_test=script.get("extinctionDetectionStopsTest", ""),
        sweep_kind=pl["kind"], sweep=dict(pl["params"]), cycles=pl.get("cycles"), window_s=pl["window_s"],
        sweep_v_max_planned=planned_v_max(script)[0], sweep_v_bound_script=planned_v_max(script)[1],
        sweep_v_max_held=held_v_max(script),
        levels=[{k: v for k, v in lv.items()} for lv in pl["levels"]],
        mfc_full_scale_slpm=[num(hdr.get(f"MFCA{i} Full Scale")) for i in (1, 2, 3)],
        pmt_gains=[num(hdr.get(f"PMT {i} Gain")) for i in (1, 2, 3)],
    )

    # ---- clocks --------------------------------------------------------------------------------------
    stamps, tp_name, reason = fd.read_tp(paths["TP.txt"]) if paths["TP.txt"].exists() else ({}, None, None)

    def on_clock(s):
        return s + 86400 if s < t[0] - 40000 else s
    ign = on_clock(stamps["Ignition and retraction"]) if "Ignition and retraction" in stamps else NAN
    sp = d["EfieldSetpointOut"]
    v_cmd = setpoint_to_volts(sp, polarity)
    v_meas = EV_A * d["EfieldVoltage"] + EV_B
    sgn = -1.0 if polarity == "negative" else 1.0
    ion = sgn * ION_UA_PER_V * d["IonCurrent"]
    ion_raw = d["IonCurrent"]

    # flame: the PMT rule of flame_design.py
    dark = [fd.dark_level(t, d[c]) for c in ("PMT1", "PMT2", "PMT3")]
    ex = fd.smooth(t, sum(d[c] - k for c, k in zip(("PMT1", "PMT2", "PMT3"), dark)), 0.2)
    armed = np.where(ex < fd.PMT_LIT_V)[0]
    t_arm = float(t[armed[0]]) if len(armed) else NAN
    start = ign - 3 if ign == ign else (t_arm if t_arm == t_arm else t[0])
    onset = fd.first_sustained(t, ex > fd.PMT_LIT_V, max(start, t_arm) if t_arm == t_arm else start, 0.5)
    if onset != onset:
        return inputs, dict(status="flame_not_lit"), "flame_not_lit"
    # field on: the commanded output leaves its value at the flame onset
    i0 = int(np.searchsorted(t, onset))
    s0 = float(np.median(sp[i0:i0 + 25]))
    moved = np.where((t > onset) & (np.abs(sp - s0) > SETPOINT_CHANGE_V))[0]
    if not len(moved):
        return inputs, dict(status="field_never_on"), "field_never_on"
    t_on = float(t[moved[0]])
    flame_end = fd.first_sustained(t, ex < fd.PMT_END_V, t_on - LIT_BEFORE_FIELD_S, fd.PMT_HOLD_S)
    wl = (t >= t_on - LIT_BEFORE_FIELD_S) & (t <= t_on)
    lit_at_on = bool(wl.any() and np.all(ex[wl] > fd.PMT_END_V)) and not (flame_end == flame_end and flame_end <= t_on)

    # pre-field block (a forecast may see it: the record up to the field-on cutoff)
    w = (t >= t_on - PRE_FIELD_S) & (t < t_on)
    rad = fd.radiometer_channels(d)
    fs = inputs["mfc_full_scale_slpm"]
    inputs["pre_field"] = dict(
        seconds_since_flame_onset=t_on - onset,
        pmt_sum_above_dark_v=float(np.median(ex[w])) if w.any() else NAN,
        radiometer_sum_v=float(np.median(sum(d[c] for c in rad)[w])) if (w.any() and rad) else NAN,
        ion_current_ua=float(np.median(ion[w])) if w.any() else NAN,
        flows_slpm=[float(np.median(d[c][w]) / 5.0 * f) if w.any() else NAN
                    for c, f in zip(("MFCA1StaticMixerFlow", "MFCA2BypassFlow", "MFCA3FuelFlow"), fs)],
        chamber_pressure_atm=float((np.median(d["PTA3"][w]) - 0.5) * 14.5038 * 10 / 4 / 14.69595) if (w.any() and "PTA3" in d) else NAN,
        burner_tc_v=float(np.median(d["TCBurner"][w])) if (w.any() and "TCBurner" in d) else NAN,
    )
    # carbon flow from the measured fuel-bottle flow (MFCA3, slpm at 0 C and 760 torr) x the bottle's fuel fraction
    f3 = inputs["pre_field"]["flows_slpm"][2]
    inputs["pre_field"]["carbon_flow_atoms_s_measured"] = (f3 / 60.0 / 1000.0 / MOLAR_VOLUME_STP_M3 * gas[1] * n_c * AVOGADRO
                                                           if (gas and f3 == f3) else NAN)
    if not lit_at_on:
        return inputs, dict(status="flame_out_before_field"), "flame_out_before_field"

    # ---- censoring: an unplanned flow change, the end of the record -------------------------------
    flow_change = NAN
    if paths["mfcSetpoints.csv"].exists():
        ts, vs = fd.read_setpoints(paths["mfcSetpoints.csv"], t[0])
        before = np.where(ts <= t_on)[0]
        if len(before):
            lvl = vs[before[-1]]
            tol = np.maximum(0.02 * np.abs(lvl), 0.005)
            later = np.where((ts > t_on) & np.any(np.abs(vs - lvl) > tol, axis=1))[0]
            if len(later):
                flow_change = float(ts[later[0]])
    if pl["kind"] == "unreadable":
        return inputs, dict(status="no_plan"), "no_plan"
    # the field programme ends when the command is back at rest (its pre-field value, or near 0 V)
    sweep_start = t_on + pl["levels"][0]["dwell_s"]
    rest = (np.abs(sp - s0) < 0.005) | (np.abs(v_cmd) < 60.0) | ((v_cmd > -160.0) if polarity == "negative" else False)
    field_off = fd.first_sustained(t, rest, sweep_start + 0.5, FIELD_OFF_HOLD_S)
    window_end = t_on + pl["window_s"]
    observed_end = min(x for x in (window_end, float(t[-1]), field_off) if x == x)
    fc_before_end = flow_change == flow_change and flow_change < observed_end
    ended = flame_end == flame_end and flame_end <= observed_end and not (fc_before_end and flame_end > flow_change)
    if ended and fc_before_end and flame_end > flow_change - fd.LABEL_MARGIN_S:
        ended = flame_end < flow_change - fd.LABEL_MARGIN_S   # dark just after a flow command: not the field's event
    censor = min(observed_end, flow_change) if fc_before_end else observed_end
    t2 = dict(event=int(bool(ended)), time_s=(flame_end - t_on) if ended else None,
              observed_s=censor - t_on, window_planned_s=pl["window_s"],
              field_off_s=(field_off - t_on) if field_off == field_off else None,
              censored_by=("flow_change" if fc_before_end and not ended else
                           "record_end" if (not ended and t[-1] < min(window_end, field_off if field_off == field_off else math.inf)) else
                           "field_off_early" if (not ended and field_off == field_off and field_off < window_end - 1.0) else None),
              v_cmd_at_end=float(v_cmd[np.searchsorted(t, flame_end) - 1]) if ended else None,
              tp_extinction_detected_s=(on_clock(stamps["Extinction detected"]) - t_on) if "Extinction detected" in stamps else None,
              tp_abort_reason=reason)

    # ---- T1: the current at each planned level --------------------------------------------------------
    t1 = {}
    cursor = t_on
    alive_until = flame_end if flame_end == flame_end else math.inf
    hard_end = min(flow_change if fc_before_end else math.inf, float(t[-1]),
                   (field_off + 0.2) if field_off == field_off else math.inf, window_end + 5.0)
    for lv in pl["levels"]:
        vk = lv["v_cmd"]
        res = dict(status="not_reached")
        if lv["kind"] in ("init", "step"):
            tol = max(STEP_TOL_MIN_V, STEP_TOL_FRAC * abs(num(inputs["sweep"]["efieldStepVoltage"]) or 0.0)) \
                if lv["kind"] == "step" else max(STEP_TOL_MIN_V, 0.3 * abs(vk))
            inn = (np.abs(v_cmd - vk) < tol) & (t >= cursor) & (t < hard_end)
            idx = np.where(inn)[0]
            if len(idx):
                a = idx[0]
                gap = np.where(~inn[a:])[0]
                b = a + (gap[0] if len(gap) else len(inn) - a) - 1
                ta, tb = float(t[a]), float(t[b])
                if tb - ta >= MIN_PLATEAU_FRAC * lv["dwell_s"] and tb > ta:
                    wa, wb = ta + 0.5 * (tb - ta), tb
                    res = window_result(t, ion, ion_raw, v_meas, v_cmd, wa, wb, t_on, alive_until)
                    cursor = tb
        else:  # ramp: first crossing of the level after the cursor
            prev = v_cmd[:-1] - vk
            nxt = v_cmd[1:] - vk
            near = np.abs(nxt) < RAMP_LEVEL_TOL_V
            cross = np.where((((prev <= 0) & (nxt >= 0)) | ((prev >= 0) & (nxt <= 0)) | near)
                             & (t[1:] >= cursor) & (t[1:] < hard_end))[0]
            if len(cross):
                tc = float(t[cross[0] + 1])
                res = window_result(t, ion, ion_raw, v_meas, v_cmd, tc - RAMP_HALF_WINDOW_S, tc + RAMP_HALF_WINDOW_S,
                                    t_on, alive_until)
                cursor = tc
        t1[lv["level_id"]] = res

    # ---- leak check: the duplicate sweep after the flame (quality flag, never an input) --------------
    post = (t > (flame_end if flame_end == flame_end else t[-1]) + 5.0) & (np.abs(v_cmd) > 1000)
    leak = float(np.median(np.abs(ion[post]))) if post.sum() > 50 else None
    outcomes = dict(status="ok", t1=t1, t2=t2, leak_check_median_abs_ua=leak,
                    peak_current_ua=max([r["ion_current_ua"] for r in t1.values() if r["status"] == "ok"], default=None))
    return inputs, outcomes, "ok"


def window_result(t, ion, ion_raw, v_meas, v_cmd, wa, wb, t_on, alive_until):
    import numpy as np
    w = (t >= wa) & (t <= wb)
    if w.sum() < 3:
        return dict(status="window_too_short")
    if wb > alive_until:
        return dict(status="flame_out", t_from_field_on_s=[wa - t_on, wb - t_on])
    r = dict(status="ok", ion_current_ua=float(np.median(ion[w])), ion_current_iqr_ua=float(np.subtract(*np.percentile(ion[w], [75, 25]))),
             v_applied=float(np.median(v_meas[w])), v_cmd_measured=float(np.median(v_cmd[w])),
             t_from_field_on_s=[wa - t_on, wb - t_on], n=int(w.sum()))
    if float(np.max(np.abs(ion_raw[w]))) >= ION_ADC_LIMIT_V:
        r["status"] = "adc_clipped"
    return r


# =====================================================================================================
# build
# =====================================================================================================
def clean(x):
    if isinstance(x, float):
        return None if x != x or x in (math.inf, -math.inf) else x
    if isinstance(x, dict):
        return {k: clean(v) for k, v in x.items()}
    if isinstance(x, list):
        return [clean(v) for v in x]
    return x


def cmd_build(_args):
    part = partitions()
    table = day_table()
    cat = catalogue_tests()
    OUT.mkdir(parents=True, exist_ok=True)
    files = {k: (OUT / f"cases_{k}.jsonl").open("w", encoding="utf-8", newline="\n")
             for k in ("train", "validation_inputs", "validation_outcomes")}
    status = {"train": Counter(), "validation": Counter()}
    for test in sorted(cat):
        p = part.get(test[:5])
        if p not in ("train", "validation"):
            continue
        guard(test, part)
        try:
            inputs, outcomes, st = measure(test)
        except Exception as e:  # one unreadable file must not hide the others
            inputs, outcomes, st = None, None, f"error:{type(e).__name__}"
        status[p][st] += 1
        if inputs is None:
            continue
        inputs.update(partition=p, group=f"{INV}:day:{test[:5]}", table=table.get(test[:5], {}))
        if p == "train":
            files["train"].write(json.dumps(clean(dict(inputs=inputs, outcomes=outcomes)), ensure_ascii=False) + "\n")
        else:
            files["validation_inputs"].write(json.dumps(clean(dict(inputs=inputs)), ensure_ascii=False) + "\n")
            files["validation_outcomes"].write(json.dumps(clean(dict(test=test, outcomes=outcomes)), ensure_ascii=False) + "\n")
    for f in files.values():
        f.close()
    # validation: how many tests were written, nothing about their outcomes
    print(json.dumps(dict(wrote=str(OUT.relative_to(REPO)), train_status=dict(status["train"]),
                          validation_tests_written=sum(v for k, v in status["validation"].items()
                                                       if k in ("ok", "flame_not_lit", "flame_out_before_field", "field_never_on")),
                          validation_unreadable=sum(v for k, v in status["validation"].items()
                                                    if k not in ("ok", "flame_not_lit", "flame_out_before_field", "field_never_on"))),
                     indent=1))
    return 0


def load(name):
    p = OUT / f"cases_{name}.jsonl"
    return [json.loads(x) for x in p.read_text(encoding="utf-8").splitlines() if x.strip()]


# =====================================================================================================
# scoring
# =====================================================================================================
def pinball(q, y, tau):
    return (tau - (1.0 if y < q else 0.0)) * (y - q)


def crps_quantiles(levels, q, y):
    """As tools/wsp/ext/transfer_droplet.py (the evaluator's formula from five quantiles)."""
    if not math.isfinite(y) or any(v is None or not math.isfinite(v) for v in q):
        return NAN
    o = sorted(range(len(levels)), key=lambda i: levels[i])
    tl, qq = [levels[i] for i in o], [q[i] for i in o]
    ext = [0.0] + tl + [1.0]
    return 2.0 * sum((ext[k + 2] - ext[k]) / 2.0 * pinball(qq[k], y, tl[k]) for k in range(len(tl)))


def integrated(hs, bs):
    hb = [(h, b) for h, b in zip(hs, bs) if math.isfinite(b)]
    if len(hb) < 2:
        return hb[0][1] if hb else NAN
    s = math.fsum((hb[i][0] - hb[i - 1][0]) * (hb[i][1] + hb[i - 1][1]) / 2.0 for i in range(1, len(hb)))
    return s / (hb[-1][0] - hb[0][0])


def t2_eligible(o, h):
    """Status of horizon h for one test: 1/0 indicator, or None when h is outside what was observed."""
    t2 = o["t2"]
    if h > (t2["window_planned_s"] or 0) + 1e-9:
        return None
    if t2["event"]:
        return 1.0 if t2["time_s"] <= h else 0.0
    return 0.0 if t2["observed_s"] >= h else None


def score(forecasts, cases):
    """forecasts: {test: forecast dict}; cases: rows with inputs and outcomes. Returns aggregate and per-day
    scores. T1: mean CRPS (uA) over scored (test, level) cases, per day and overall (overall = mean of all
    cases; also the mean of day means). T2: Brier at each horizon pooled over tests, integrated over the
    horizons with at least 5 eligible tests; Brier of p_end_in_window."""
    t1_case, t1_day = [], defaultdict(list)
    t2_h = defaultdict(list)
    t2_hd = defaultdict(lambda: defaultdict(list))
    b_end, b_end_day = [], defaultdict(list)
    missing = Counter()
    for c in cases:
        i, o = c["inputs"], c["outcomes"]
        if o.get("status") != "ok":
            continue
        f = forecasts.get(i["test"])
        if f is None:
            missing["test"] += 1
            continue
        day = i["day"]
        for lid, r in o["t1"].items():
            if r["status"] != "ok":
                continue
            q = (f.get("t1") or {}).get(lid)
            if q is None:
                missing["t1_level"] += 1
                continue
            s = crps_quantiles(LEVELS_Q, q, r["ion_current_ua"])
            if math.isfinite(s):
                t1_case.append(s)
                t1_day[day].append(s)
        t2f = f.get("t2") or {}
        cif = {float(k): v for k, v in (t2f.get("cif") or {}).items()}
        for h in HORIZONS_S:
            y = t2_eligible(o, h)
            if y is None:
                continue
            p = cif.get(float(h))
            if p is None:
                missing["t2_horizon"] += 1
                continue
            t2_h[h].append((p - y) ** 2)
            t2_hd[day][h].append((p - y) ** 2)
        if o["t2"]["censored_by"] in CLOSED and t2f.get("p_end_in_window") is not None:
            y = float(o["t2"]["event"])
            e = (t2f["p_end_in_window"] - y) ** 2
            b_end.append(e)
            b_end_day[day].append(e)

    def ib(hmap):
        hs = [h for h in HORIZONS_S if len(hmap.get(h, [])) >= 5]
        return integrated(hs, [sum(hmap[h]) / len(hmap[h]) for h in hs]), hs

    mean = lambda xs: sum(xs) / len(xs) if xs else NAN  # noqa: E731
    agg_ib, hs = ib(t2_h)
    per_day = {}
    for day in sorted(set(t1_day) | set(t2_hd) | set(b_end_day)):
        hsd = [h for h in HORIZONS_S if len(t2_hd[day].get(h, [])) >= 1]
        per_day[day] = dict(t1_crps_ua=mean(t1_day[day]), t1_cases=len(t1_day[day]),
                            t2_integrated_brier=integrated(hsd, [mean(t2_hd[day][h]) for h in hsd]),
                            t2_end_in_window_brier=mean(b_end_day[day]), t2_tests=len(b_end_day[day]))
    return dict(
        t1_crps_ua=mean(t1_case), t1_cases=len(t1_case), t1_crps_ua_mean_of_days=mean([mean(v) for v in t1_day.values()]),
        t2_integrated_brier=agg_ib, t2_horizons_used=hs, t2_brier_by_horizon={h: mean(t2_h[h]) for h in hs},
        t2_n_by_horizon={h: len(t2_h[h]) for h in hs},
        t2_end_in_window_brier=mean(b_end), t2_end_in_window_tests=len(b_end),
        days=len(per_day), per_day=per_day, missing=dict(missing))


# =====================================================================================================
# baselines (train days only)
# =====================================================================================================
ETA_DIFFUSION = 5.0e-7     # electrons per fuel carbon atom at saturation, hydrocarbon diffusion flames
ETA_FID = 0.25 / 96485.0   # 0.25 C per g-atom of carbon (aliphatic compounds, flame-ionization literature)


def quantiles_sorted(xs, qs):
    xs = sorted(xs)
    out = []
    for q in qs:
        k = q * (len(xs) - 1)
        lo, hi = math.floor(k), math.ceil(k)
        out.append(xs[lo] + (xs[hi] - xs[lo]) * (k - lo))
    return out


def lognormal_q(median, sigma):
    z = {0.05: -1.6449, 0.25: -0.6745, 0.5: 0.0, 0.75: 0.6745, 0.95: 1.6449}
    return [median * math.exp(sigma * z[q]) for q in LEVELS_Q]


# Thomson's one-dimensional space-charge-limited current between plane electrodes, J = (9/8) eps0 mu V^2 / d^3
# (the classical sub-saturation law of flame ionisation, Lawton and Weinberg, Electrical Aspects of Combustion,
# 1969), with the standard small-ion mobilities of room-temperature air (positive 1.4e-4, negative 1.9e-4
# m2/(V s)): a negative mesh collects positive ions, a positive mesh negative carriers. Gap: the mesh position
# of the header (35 or 40 mm); area: the 8.4 cm mesh disc (final report p. 13).
EPS0 = 8.8541878128e-12
MOBILITY = {"negative": 1.4e-4, "positive": 1.9e-4}
GAP_M = {"35mm": 0.035, "40mm": 0.040}
MESH_AREA_M2 = math.pi * 0.042 ** 2


def i_scl_ua(inputs, v):
    d = GAP_M.get(inputs.get("mesh_position"))
    if d is None or v != v:
        return NAN
    return 9.0 / 8.0 * EPS0 * MOBILITY[inputs["polarity"]] * v * v / d ** 3 * MESH_AREA_M2 * 1e6


def level_v(c, lid):
    for lv in c["inputs"]["levels"]:
        if lv["level_id"] == lid:
            return abs(lv["v_cmd"])
    return NAN


def i_sat_ua(inputs, eta):
    n = inputs.get("carbon_flow_atoms_s")
    return eta * n * E_CHARGE * 1e6 if n else NAN


def cmd_baselines(_args):
    cases = [c for c in load("train") if c["outcomes"].get("status") == "ok"]
    ys = [(c, lid, r) for c in cases for lid, r in c["outcomes"]["t1"].items() if r["status"] == "ok"]
    allv = [r["ion_current_ua"] for _, _, r in ys]
    q_const = quantiles_sorted(allv, LEVELS_Q)
    # classical saturation law, with one log-width per law fitted on train (its only constant besides eta)
    out = {}

    def sigma_of(pred):
        res = [math.log(max(r["ion_current_ua"], 1e-3) / p) for c, lid, r in ys
               for p in [pred(c, lid)] if p == p and p > 0]
        m = sum(res) / len(res)
        return (sum((x - m) ** 2 for x in res) / len(res)) ** 0.5, m
    # T2 incidence: Kaplan-Meier on train tests (event = flame end inside the window)
    t2s = [c["outcomes"]["t2"] for c in cases]
    times = sorted({t["time_s"] for t in t2s if t["event"]})
    surv, s = {}, 1.0
    for h in HORIZONS_S:
        for tt in [x for x in times if (surv.get("_last", -1) < x <= h)]:
            at_risk = sum(1 for t in t2s if (t["time_s"] if t["event"] else t["observed_s"]) >= tt)
            d_ = sum(1 for t in t2s if t["event"] and t["time_s"] == tt)
            s *= 1 - d_ / at_risk if at_risk else 1
        surv["_last"] = h
        surv[h] = 1 - s
    cif = {str(h): surv[h] for h in HORIZONS_S}
    closed = [t for t in t2s if t["censored_by"] in CLOSED]
    p_end = sum(t["event"] for t in closed) / len(closed)

    def forecasts(t1fn):
        return {c["inputs"]["test"]: dict(t1={lid: t1fn(c, lid) for lid in c["outcomes"]["t1"]},
                                          t2=dict(cif=cif, p_end_in_window=p_end)) for c in cases}
    out["constant (train pool quantiles of the current; T2: Kaplan-Meier incidence)"] = score(
        forecasts(lambda c, lid: q_const), cases)
    for name, eta in (("saturation law, eta = 5e-7 per C (diffusion flames)", ETA_DIFFUSION),
                      ("saturation law, eta = 2.6e-6 per C (FID, 0.25 C/g-atom C)", ETA_FID)):
        pred = lambda c, lid, eta=eta: i_sat_ua(c["inputs"], eta)  # noqa: E731
        sg, bias = sigma_of(pred)
        out[name] = score(forecasts(lambda c, lid, pred=pred, sg=sg: lognormal_q(pred(c, lid), sg)
                                    if pred(c, lid) == pred(c, lid) else [None] * 5), cases)
        out[name]["fitted_log_sigma"] = sg
        out[name]["mean_log_ratio_observed_over_law"] = bias
    pred = lambda c, lid: min(i_sat_ua(c["inputs"], ETA_DIFFUSION), i_scl_ua(c["inputs"], level_v(c, lid)))  # noqa: E731
    sg, bias = sigma_of(pred)
    name = "space-charge law up to saturation, min(Thomson planar, eta = 5e-7)"
    out[name] = score(forecasts(lambda c, lid: lognormal_q(pred(c, lid), sg) if pred(c, lid) == pred(c, lid) else [None] * 5), cases)
    out[name]["fitted_log_sigma"] = sg
    out[name]["mean_log_ratio_observed_over_law"] = bias
    # T2: incidence by stratum (polarity x coflow on/off), Kaplan-Meier within each, same T1 as the constant
    def km(sub):
        tt = sorted({x["time_s"] for x in sub if x["event"]})
        cif_, s_ = {}, 1.0
        for h in HORIZONS_S:
            for x in [y for y in tt if (cif_.get("_l", -1) < y <= h)]:
                n_ = sum(1 for z in sub if (z["time_s"] if z["event"] else z["observed_s"]) >= x)
                s_ *= 1 - sum(1 for z in sub if z["event"] and z["time_s"] == x) / n_ if n_ else 1
            cif_["_l"] = h
            cif_[h] = 1 - s_
        return {str(h): cif_[h] for h in HORIZONS_S}
    strat = lambda c: (c["inputs"]["polarity"], (c["inputs"]["u_coflow_m_s"] or 0) > 0, c["inputs"]["burner"])  # noqa: E731
    groups = defaultdict(list)
    for c in cases:
        groups[strat(c)].append(c)
    st_cif = {k: km([c["outcomes"]["t2"] for c in v]) for k, v in groups.items()}
    st_p = {}
    for k, v in groups.items():
        cl = [c["outcomes"]["t2"] for c in v if c["outcomes"]["t2"]["censored_by"] in CLOSED]
        st_p[k] = sum(x["event"] for x in cl) / len(cl) if cl else p_end
    fs = {c["inputs"]["test"]: dict(t1={lid: q_const for lid in c["outcomes"]["t1"]},
                                    t2=dict(cif=st_cif[strat(c)], p_end_in_window=st_p[strat(c)])) for c in cases}
    out["stratified incidence (polarity x coflow x burner; T1 as the constant)"] = score(fs, cases)
    out["stratified incidence (polarity x coflow x burner; T1 as the constant)"]["strata"] = {
        "|".join(map(str, k)): dict(tests=len(groups[k]), events=sum(c["outcomes"]["t2"]["event"] for c in groups[k]),
                                    p_end_in_window=st_p[k]) for k in groups}
    # persistence (a cutoff variant: needs the measured current of the previous level)

    def prev_level(c, lid):
        ids = list(c["outcomes"]["t1"])
        k = ids.index(lid)
        for j in range(k - 1, -1, -1):
            r = c["outcomes"]["t1"][ids[j]]
            if r["status"] == "ok":
                return r["ion_current_ua"]
        return NAN
    res = [r["ion_current_ua"] - prev_level(c, lid) for c, lid, r in ys if prev_level(c, lid) == prev_level(c, lid)]
    qd = quantiles_sorted(res, LEVELS_Q)
    out["persistence (previous level's measured current + train quantiles of the step change)"] = score(
        forecasts(lambda c, lid: [prev_level(c, lid) + x for x in qd] if prev_level(c, lid) == prev_level(c, lid) else [None] * 5),
        cases)
    # the constant baseline's forecasts, as a file, so that `score` can be shown to reproduce it
    with (OUT / "forecasts_train_constant.jsonl").open("w", encoding="utf-8", newline="\n") as f:
        for test, fc in forecasts(lambda c, lid: q_const).items():
            f.write(json.dumps(clean(dict(test=test, **fc))) + "\n")
    out["_constants"] = dict(t1_constant_quantiles_ua=q_const, t1_persistence_step_change_quantiles_ua=qd,
                             t2_cif=cif, t2_p_end_in_window=p_end, train_tests_ok=len(cases), t1_train_cases=len(ys))
    (OUT / "baselines_train.json").write_text(json.dumps(clean(out), indent=1), encoding="utf-8")
    brief = {k: (v if k == "_constants" else {kk: v[kk] for kk in ("t1_crps_ua", "t1_cases", "t1_crps_ua_mean_of_days",
                                                                   "t2_integrated_brier", "t2_end_in_window_brier",
                                                                   "t2_end_in_window_tests", "days", "missing") if kk in v}
                 | {kk: v[kk] for kk in ("fitted_log_sigma", "mean_log_ratio_observed_over_law") if kk in v})
             for k, v in out.items()}
    print(json.dumps(clean(brief), indent=1))
    return 0


def cmd_score(args):
    cases = load(args.partition)
    fc = {}
    for line in Path(args.forecasts).read_text(encoding="utf-8").splitlines():
        if line.strip():
            r = json.loads(line)
            fc[r["test"]] = r
    if args.partition != "train":
        sys.exit("validation outcomes are scored only in a registered look; this command scores train only")
    print(json.dumps(clean(score(fc, cases)), indent=1))
    return 0


# =====================================================================================================
# counts
# =====================================================================================================
def cmd_counts(_args):
    part = partitions()
    cat = catalogue_tests()
    by = defaultdict(Counter)
    for test, kinds in cat.items():
        p = part.get(test[:5], "held")
        by[p]["tests_named"] += 1
        by[p]["days"] = 0
        if {"data.csv", "VoltageSweep.txt"} <= kinds:
            by[p]["data_and_script"] += 1
        if {"data.csv", "VoltageSweep.txt", "TP.txt", "mfcSetpoints.csv"} <= kinds:
            by[p]["all_four_files"] += 1
    for p in by:
        by[p]["days"] = len({t[:5] for t in cat if part.get(t[:5], "held") == p})
    res = {p: dict(v) for p, v in by.items()}
    for p in ("train", "validation"):
        f = OUT / f"cases_{'train' if p == 'train' else 'validation_inputs'}.jsonl"
        if f.exists():
            rows = [json.loads(x) for x in f.read_text(encoding="utf-8").splitlines() if x.strip()]
            res[p]["written"] = len(rows)
            res[p]["with_planned_levels"] = sum(1 for r in rows if len(r["inputs"]["levels"]) > 1)
            res[p]["planned_t1_levels"] = sum(len(r["inputs"]["levels"]) for r in rows)
            if p == "train":
                ok = [r for r in rows if r["outcomes"].get("status") == "ok"]
                res[p]["t1_t2_outcome_measured"] = len(ok)
                res[p]["t1_levels_scored"] = sum(1 for r in ok for x in r["outcomes"]["t1"].values() if x["status"] == "ok")
                res[p]["t1_level_status"] = dict(Counter(x["status"] for r in ok for x in r["outcomes"]["t1"].values()))
                res[p]["t2_status"] = dict(Counter(("event" if r["outcomes"]["t2"]["event"] else
                                                    "censored_" + str(r["outcomes"]["t2"]["censored_by"]))
                                                   for r in ok))
                res[p]["not_ok"] = dict(Counter(r["outcomes"].get("status") for r in rows if r["outcomes"].get("status") != "ok"))
    print(json.dumps(res, indent=1))
    return 0


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    sub = ap.add_subparsers(dest="cmd", required=True)
    sub.add_parser("build").set_defaults(fn=cmd_build)
    sub.add_parser("counts").set_defaults(fn=cmd_counts)
    sub.add_parser("baselines").set_defaults(fn=cmd_baselines)
    p = sub.add_parser("score")
    p.add_argument("--forecasts", required=True)
    p.add_argument("--partition", default="train")
    p.set_defaults(fn=cmd_score)
    a = ap.parse_args()
    return a.fn(a)


if __name__ == "__main__":
    sys.exit(main())
