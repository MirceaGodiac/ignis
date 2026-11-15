#!/usr/bin/env python3
"""ACME Flame Design (PSI-10): conditions and per-test endpoints for a transfer test of the frozen
CFI-G burner-flame models (`acme_normal.wsp5`, `acme_inverse.wsp5`). Preparation only: this tool
scores nothing, changes no model and fits nothing. Every number below is a typed threshold or a
copy of a constant of the forward; none is estimated from data.

Three subcommands, all of which write only under `runs/dev/data_all/work/flame_design/`:

  conditions   The two experiment tables -> one row per test with the condition keys the models read
               (pressure, injected_mass_flow, x_o2_role, x_fuel_role, tad, zst, fuel_species), the
               model features and whether each is inside the declared domain, the availability of
               the raw text files, and the proposed inclusion flag. Reads no outcome: the tables'
               after-test columns (XO2 final, XC2H4 final, pfinal, pmean) are never read, and the
               stoichiometric mixture fraction is recomputed from the initial composition because
               the table's "mean Zst" is taken at the mean of the initial and final composition.
               -> conditions.csv

  plan         From the manifest listing only (no network): which raw text files the given tests
               need, their bytes, and the data lane's fetch command, in batches. `--execute` runs
               that command (public NASA files, one at a time); without it nothing is downloaded.

  endpoints    Per test, from `<test>_data.csv` (about 125 Hz sensor channels on the script's clock),
               `<test>_mfcSetpoints.csv` (the commanded flows), `<test>_TP.txt` (stage stamps) and
               the script parameter file (`as run<test>.txt`, or `<test>.txt` in early 2019):
                 onset          the radiometers rise above their pre-ignition level (the CFI-G
                                measurement of docs/wsp/feedback/acme_raw_signals_train.md)
                 flame end      the photomultipliers return to their armed dark level
                 flow change    the first commanded departure from the test flow after the
                                constant-flow stage has begun (stop, post-extinction change, ramp)
                 label          extinction when the flame end precedes the flow change, operator
                                termination when the flow changed first; cross-checked against the
                                script's own stamps ("Extinction detected", "MFCs off", "Reason")
               -> endpoints.csv. THIS FILE HOLDS OUTCOMES. The console report is counts and
               statuses only, so a model author can run the tool and stay blind; do not open the
               CSV before the transfer test is registered.

    python tools/wsp/ext/flame_design.py conditions
    python tools/wsp/ext/flame_design.py plan --tests 19115A1,21349A1 [--execute]
    python tools/wsp/ext/flame_design.py endpoints --tests 19115A1,21349A1
    python tools/wsp/ext/flame_design.py endpoints --included        # every test conditions.csv includes
"""
import argparse
import csv
import json
import math
import os
import re
import subprocess
import sys
from collections import Counter
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
DATA_ROOT = REPO / "runs/dev/data_all"
FILES = DATA_ROOT / "raw/samples/PSI-10"
LISTING = DATA_ROOT / "manifests/PSI-10.listing.jsonl"
OUT = DATA_ROOT / "work/flame_design"
PRE = "PSI-10_Raw Data_"
TABLES = {
    "acme_normal": "PSI-10_Experimental table_Flame Design_Normal Flame.csv",
    "acme_inverse": "PSI-10_Experimental table_Flame Design_Inverse Flame.csv",
}

# ---------------------------------------------------------------------------------------------
# Constants copied from the forwards (crates/ignis-wsp/src/flame.rs and flame3.rs, `scales` and the
# `norm.*` constants). The forward is the authority; these copies only pre-compute its domain gate.
# ---------------------------------------------------------------------------------------------
P_REF_PA, TAD_REF_K, TAD_SCALE_K, ZST_SCALE = 101_325.0, 2200.0, 300.0, 0.2
SCALES = {
    "acme_normal": dict(mdot_ref=1.0e-6, o2_ref=0.30, o2_scale=0.10, xf_ref=0.30, zst_ref=0.25,
                        domain=dict(c_logp=(-0.92, 1.25), c_logm=(-2.31, 1.39), c_o2=(-1.5, 1.5),
                                    c_xf=(-2.71, 1.21), c_tad=(-2.67, 2.33), c_zst=(-1.1, 2.5))),
    "acme_inverse": dict(mdot_ref=5.0e-6, o2_ref=0.50, o2_scale=0.25, xf_ref=0.10, zst_ref=0.45,
                         domain=dict(c_logp=(-0.92, 1.25), c_logm=(-1.61, 0.88), c_o2=(-1.6, 1.6),
                                     c_xf=(-0.70, 1.61), c_tad=(-2.0, 3.0), c_zst=(-1.85, 1.5))),
}
# Fuels the forwards accept (`fuel_species`); anything else is refused at the condition gate.
MODEL_FUELS = ("ethane", "propane", "butane")
SPECIES = {"C2H4": "ethylene", "C3H8": "propane"}
MW = {"ethylene": 28.054, "propane": 44.096, "o2": 31.999, "n2": 28.013}
NU_O2 = {"ethylene": 3.0, "propane": 5.0}  # mol O2 per mol fuel, complete combustion
ATM_PA = 101_325.0

# ---------------------------------------------------------------------------------------------
# Typed thresholds of the endpoint measurement (the CFI-G ones unless noted).
# ---------------------------------------------------------------------------------------------
ONSET_RISE_V = 0.15       # radiometer sum above its pre-ignition median (or 8 sd, if larger)
PMT_LIT_V = 0.05          # PMT sum above its armed dark level: lit
PMT_END_V = 0.05          # ... back below: dark. CFI-G used 0.02 V on the raw sum; here one PMT
PMT_END_V_TIGHT = 0.02    #   idles at about 0.13 V, so the level is taken above the dark level
PMT_HOLD_S = 0.5          # the PMT must stay dark this long
PMT_DARK_BAND_V = 0.25    # samples this close to a channel's lowest level define its dark level
RAD_FALL_AFTER_S = 3.0    # the radiometer cross-check ignores the ignition flash
FLOW_ON_V = 0.05          # MFC readback: flowing
FLOW_GAP_S = 1.0          # readback off this long: the fed phase has ended
LABEL_MARGIN_S = 0.5      # flame end this much before the flow change: extinction
STAMP_TO_STAGE_S = 2.5    # "Ignition and retraction" stamp to "Flame stabilization #1" when unstamped
STAGE_KEYS = ["stabilizationTime1Sec", "stabilizationTime2Sec", "ignitionToTestRampTimeSec",
              "constantFlowTimeSec",  # 2019 scripts (one constant-flow stage)
              "constantFlowTime1Sec", "testFlowRamp1To2TimeSec", "constantFlowTime2Sec",
              "testFlowRamp2To3TimeSec", "constantFlowTime3Sec"]
PRE_STAGE_KEYS = ["stabilizationTime1Sec", "stabilizationTime2Sec", "ignitionToTestRampTimeSec"]

NAN = float("nan")


def num(x):
    try:
        return float(x)
    except (TypeError, ValueError):
        return NAN


def fmt(v, nd=3):
    if isinstance(v, float):
        return "" if v != v else f"{v:.{nd}f}"
    return "" if v is None else str(v)


# ---------------------------------------------------------------------------------------------
# conditions
# ---------------------------------------------------------------------------------------------
def mass_fraction(x, w):
    return x * w / (x * w + (1.0 - x) * MW["n2"])


def zst_initial(species, x_fuel_stream, x_o2_stream):
    """Stoichiometric mixture fraction of a fuel/N2 stream against an O2/N2 stream."""
    s = NU_O2[species] * MW["o2"] / MW[species]
    yf, yo = mass_fraction(x_fuel_stream, MW[species]), mass_fraction(x_o2_stream, MW["o2"])
    return 1.0 / (1.0 + s * yf / yo)


def listing():
    out = {}
    with LISTING.open(encoding="utf-8") as f:
        for line in f:
            r = json.loads(line)
            if r.get("category") == "Raw Data":
                out[r["file_name"]] = r["size_bytes"]
    return out


def file_names(test, names):
    """The four text files of a test, by the names the listing actually has (or None)."""
    def first(*cands):
        for c in cands:
            if c in names:
                return c
        return None
    alias = test[:5] + "J" + test[5:]  # day 19189: TP files of K1..K3, P1 are stored as JK1..JK3, JP1
    return dict(
        data=first(f"{PRE}{test}_data.csv"),
        setpoints=first(f"{PRE}{test}_mfcSetpoints.csv"),
        tp=first(f"{PRE}{test}_TP.txt", f"{PRE}{alias}_TP.txt"),
        script=first(f"{PRE}as run{test}.txt", f"{PRE}{test}.txt"),
    )


def table_rows():
    for family, name in TABLES.items():
        with (FILES / name).open(encoding="utf-8-sig", newline="") as f:
            for r in csv.DictReader(f):
                yield family, {k.strip(): (v or "").strip() for k, v in r.items()}


def condition_row(family, r, names):
    sc = SCALES[family]
    test = r["Test"]
    if family == "acme_normal":
        species = SPECIES.get(r["Fuel"], r["Fuel"].lower())
        m, x_o2, x_f = num(r["mfuel (mg/s)"]), num(r["XO2 (init)"]), num(r["Xfuel"])
        note = r["Xfuel"] if x_f != x_f else ""
        zst = zst_initial(species, x_f, x_o2) if species in NU_O2 and x_f == x_f else NAN
    else:
        species = "ethylene"  # the inverse table has no fuel column; its ambient column is XC2H4
        m, x_o2, x_f = num(r["mO2 (mg/s)"]), num(r["XO2"]), num(r["XC2H4 initial"])
        note = r["XO2"] if x_o2 != x_o2 else ""
        zst = zst_initial(species, x_f, x_o2) if x_o2 == x_o2 else NAN
    p, tad, zst_tab = num(r["pinit (atm)"]) * ATM_PA, num(r["Tad (K)"]), num(r["mean Zst"])
    m_kg = m * 1.0e-6
    constant = all(v == v for v in (m, x_o2, x_f, tad, zst))
    feats = dict(c_logp=NAN, c_logm=NAN, c_o2=NAN, c_xf=NAN, c_tad=NAN, c_zst=NAN)
    outside = []
    if constant:
        feats = dict(
            c_logp=math.log(p / P_REF_PA), c_logm=math.log(m_kg / sc["mdot_ref"]),
            c_o2=(x_o2 - sc["o2_ref"]) / sc["o2_scale"], c_xf=math.log(x_f / sc["xf_ref"]),
            c_tad=(tad - TAD_REF_K) / TAD_SCALE_K, c_zst=(zst - sc["zst_ref"]) / ZST_SCALE)
        outside = [k for k, (lo, hi) in sc["domain"].items() if not lo <= feats[k] <= hi]
    fn = file_names(test, names)
    have = {k: v is not None for k, v in fn.items()}
    fuel_ok = species in MODEL_FUELS
    readable = have["data"] and have["tp"]
    return dict(
        test=test, family=family, day=r["Day"], session=test[:5], fuel_species=species,
        pressure_pa=fmt(p, 1), injected_mass_flow_kg_s=f"{m_kg:.6e}" if m == m else "",
        x_o2_role=fmt(x_o2, 6), x_fuel_role=fmt(x_f, 6), tad_k_table=fmt(tad, 1),
        zst_initial=fmt(zst, 6), zst_table_mean=fmt(zst_tab, 6), table_note=note,
        **{k: fmt(v, 4) for k, v in feats.items()},
        constant_flow=int(constant), numeric_in_domain=int(constant and not outside),
        outside="|".join(outside), fuel_in_domain=int(fuel_ok),
        has_data=int(have["data"]), has_setpoints=int(have["setpoints"]), has_tp=int(have["tp"]),
        has_script=int(have["script"]),
        data_bytes=names.get(fn["data"], "") if fn["data"] else "",
        # inclusion, from conditions and file availability only (never from an outcome):
        include_transfer=int(constant and not outside and readable),          # surrogate-fuel arms too
        include_unchanged=int(constant and not outside and readable and fuel_ok),  # the models as they are
    )


def cmd_conditions(_args):
    names = listing()
    rows = [condition_row(fam, r, names) for fam, r in table_rows()]
    OUT.mkdir(parents=True, exist_ok=True)
    path = OUT / "conditions.csv"
    with path.open("w", encoding="utf-8", newline="") as f:
        w = csv.DictWriter(f, fieldnames=list(rows[0].keys()))
        w.writeheader()
        w.writerows(rows)
    rep = {}
    for fam in TABLES:
        rr = [r for r in rows if r["family"] == fam]
        rep[fam] = dict(
            tests=len(rr), constant_flow=sum(r["constant_flow"] for r in rr),
            by_fuel=dict(Counter(r["fuel_species"] for r in rr)),
            numeric_in_domain=sum(r["numeric_in_domain"] for r in rr),
            outside_by_feature=dict(Counter(k for r in rr for k in r["outside"].split("|") if k)),
            has_data=sum(r["has_data"] for r in rr), has_tp=sum(r["has_tp"] for r in rr),
            has_script=sum(r["has_script"] for r in rr),
            include_transfer=sum(r["include_transfer"] for r in rr),
            include_transfer_sessions=len({r["session"] for r in rr if r["include_transfer"]}),
            include_unchanged=sum(r["include_unchanged"] for r in rr),
            include_transfer_data_bytes=sum(int(r["data_bytes"] or 0) for r in rr if r["include_transfer"]),
        )
    print(json.dumps(dict(wrote=str(path.relative_to(REPO)), **rep), indent=1))
    return 0


# ---------------------------------------------------------------------------------------------
# plan / fetch
# ---------------------------------------------------------------------------------------------
def tests_of(args):
    if getattr(args, "included", False):
        path = OUT / "conditions.csv"
        if not path.exists():
            sys.exit("run `conditions` first")
        with path.open(encoding="utf-8", newline="") as f:
            return [r["test"] for r in csv.DictReader(f) if r["include_transfer"] == "1"]
    if args.tests_file:
        text = Path(args.tests_file).read_text(encoding="utf-8")
    else:
        text = args.tests or ""
    tests = [t for t in re.split(r"[\s,]+", text) if t]
    bad = [t for t in tests if not re.fullmatch(r"\d{5}[A-Z]\d+", t)]
    if bad or not tests:
        sys.exit(f"give test names such as 19115A1 (unreadable: {bad})")
    return tests


def cmd_plan(args):
    names = listing()
    tests = tests_of(args)
    need = []
    for t in tests:
        for kind, n in file_names(t, names).items():
            if n and not (FILES / n).exists():
                need.append((t, n, names[n]))
    total = sum(b for _, _, b in need)
    print(json.dumps(dict(tests=len(tests), files_missing_locally=len(need), bytes=total), indent=1))
    if not need:
        return 0
    exe = REPO / "target/release" / ("ignis.exe" if os.name == "nt" else "ignis")
    by_test = {}
    for t, n, _ in need:
        by_test.setdefault(t, []).append(n)
    batch, batches = [], []
    for t in by_test:  # about 25 tests per call keeps the pattern short
        batch.append(t)
        if len(batch) == 25:
            batches.append(batch)
            batch = []
    if batch:
        batches.append(batch)
    for b in batches:
        alt = "|".join(sorted({re.escape(n[len(PRE):]) for t in b for n in by_test[t]}))
        pattern = f"^{re.escape(PRE)}({alt})$"
        cmd = [str(exe), "sources", "fetch", "PSI-10", "--pattern", pattern, "--category", "Raw Data"]
        if args.execute:
            cmd.append("--execute")
        env = dict(os.environ, IGNIS_DATA=str(DATA_ROOT.relative_to(REPO)).replace("\\", "/"))
        r = subprocess.run(cmd, cwd=REPO, env=env, capture_output=True, text=True)
        tail = (r.stdout or r.stderr).strip().splitlines()[-12:]
        print(f"# {len(b)} tests, {'executed' if args.execute else 'dry run'} (exit {r.returncode})")
        print("\n".join(tail))
    return 0


# ---------------------------------------------------------------------------------------------
# endpoints: readers
# ---------------------------------------------------------------------------------------------
def clock(s):
    h, m, sec, ms = s.strip().split(":")
    return int(h) * 3600 + int(m) * 60 + int(sec) + int(ms) / 1000.0


def read_header(line):
    parts = [p.strip() for p in line.split(",")]
    h = {}
    m = re.match(r"(\d{4}-\d\d-\d\d) (\d{3}) (\S+)", parts[0])
    if m:
        h.update(date=m.group(1), doy=m.group(2), script_clock=clock(m.group(3)))
    for p in parts[1:]:
        if ":" in p:
            k, v = p.split(":", 1)
            h[k.strip()] = v.strip()
    return h


def unwrap(t):
    """Seconds of day -> monotonic across midnight."""
    import numpy as np
    if len(t) > 1:
        jumps = np.where(np.diff(t) < -40000)[0]
        for i in jumps:
            t[i + 1:] += 86400
    return t


def read_data(path):
    import numpy as np
    with open(path, encoding="utf-8", errors="replace") as f:
        hdr = read_header(f.readline())
        cols = [c.strip() for c in f.readline().split(",")]
        t, rows = [], []
        for line in f:
            sp = line.rstrip("\r\n").split(",")
            if len(sp) != len(cols):
                continue
            try:
                tc = clock(sp[0])
                vv = [float(x) for x in sp[1:]]
            except ValueError:
                continue
            t.append(tc)
            rows.append(vv)
    t = unwrap(np.array(t, float))
    a = np.array(rows, float) if rows else np.zeros((0, len(cols) - 1))
    return hdr, t, {c: a[:, i] for i, c in enumerate(cols[1:])}


def read_setpoints(path, t_first):
    """The command log: (time, [MFCA1, MFCA2, MFCA3] setpoint in V), as a step function."""
    import numpy as np
    t, v = [], []
    with open(path, encoding="utf-8", errors="replace") as f:
        f.readline()
        f.readline()
        for line in f:
            sp = line.strip().split(",")
            try:
                t.append(clock(sp[0]))
                v.append([float(x) for x in sp[1:4]])
            except (ValueError, IndexError):
                continue
    t = unwrap(np.array(t, float))
    if len(t) and t[0] < t_first - 40000:
        t += 86400
    return t, np.array(v, float).reshape(-1, 3)


def read_tp(path):
    """Stage stamps (seconds of day), the test name written inside the file, and the abort reason."""
    stamps, name, reason = {}, None, None
    for line in open(path, encoding="utf-8", errors="replace").read().replace("\r", "\n").split("\n"):
        m = re.match(r"\s*([A-Za-z][^:]*?):\s*(\d{4}-\d\d-\d\d) (\d\d):(\d\d):(\d\d)", line)
        if m:
            stamps[m.group(1).strip()] = int(m.group(3)) * 3600 + int(m.group(4)) * 60 + int(m.group(5))
            continue
        m = re.match(r"\s*Test Point:\s*(\S+)", line)
        if m:
            name = m.group(1)
        m = re.match(r"\s*Reason:\s*(.+)", line)
        if m:
            reason = m.group(1).strip()
    return stamps, name, reason


def read_script(path):
    out = {}
    for line in open(path, encoding="utf-8", errors="replace").read().replace("\r", "\n").split("\n"):
        key, sep, value = line.strip().rstrip(",").partition(":")
        if sep:
            out[key.strip()] = value.strip()
    return out


# ---------------------------------------------------------------------------------------------
# endpoints: measurement
# ---------------------------------------------------------------------------------------------
def smooth(t, y, w):
    """Centred running mean over w seconds, normalised at the ends of the record."""
    import numpy as np
    dt = float(np.median(np.diff(t))) if len(t) > 2 else 0.008
    n = max(1, int(round(w / dt)))
    k = np.ones(n)
    return np.convolve(y, k, mode="same") / np.convolve(np.ones(len(y)), k, mode="same")


def first_sustained(t, mask, start, hold):
    """First time >= start at which mask is true and stays true for `hold` seconds (or to the end)."""
    import numpy as np
    idx = np.where(mask & (t >= start))[0]
    if not len(idx):
        return NAN
    bad_t = t[~mask]
    for i in idx:
        nxt = bad_t[np.searchsorted(bad_t, t[i])] if np.searchsorted(bad_t, t[i]) < len(bad_t) else math.inf
        if nxt - t[i] >= hold or nxt == math.inf:
            return float(t[i])
    return NAN


def dark_level(t, y, block=0.5, band=PMT_DARK_BAND_V):
    """Armed, unlit level of a PMT channel: the median of every sample within `band` of its lowest
    half-second median. A channel can idle above zero and settle for a second after arming, so the
    lowest block alone would sit below the level the channel returns to after the flame."""
    import numpy as np
    dt = float(np.median(np.diff(t)))
    n = max(1, int(round(block / dt)))
    m = len(y) // n
    if m == 0:
        return float(np.median(y))
    floor = float(np.median(y[: m * n].reshape(m, n), axis=1).min())
    return float(np.median(y[y < floor + band]))


def radiometer_channels(d):
    lo = [c for c in d if c.startswith("Radiometer") and c.endswith("LoGain")]
    if lo:
        return lo
    return [c for c in d if c.startswith("Radiometer") and not c.endswith("CJ")
            and "Offset" not in c and "HiGain" not in c]


def measure(test, fn):
    """One row of endpoints.csv. Times are seconds since the flame onset unless named *_clock."""
    import numpy as np
    row = dict(test=test, status="ok", flags=[])
    if not fn["data"] or not (FILES / fn["data"]).exists():
        row["status"] = "no_data_file"
        return row
    hdr, t, d = read_data(FILES / fn["data"])
    need = ["PMT1", "PMT2", "PMT3", "MFCA1StaticMixerFlow", "MFCA2BypassFlow", "MFCA3FuelFlow"]
    if len(t) < 500 or any(c not in d for c in need):
        row["status"] = "short_or_unknown_columns"
        return row
    row.update(date=hdr.get("date", ""), n_samples=len(t), dt_s=float(np.median(np.diff(t))),
               pmt_gains="/".join(hdr.get(f"PMT {i} Gain", "") for i in (1, 2, 3)),
               mfc_gas="/".join(hdr.get(f"MFCA{i} Gas", "") for i in (1, 2, 3)),
               mfc_full_scale="/".join(hdr.get(f"MFCA{i} Full Scale", "") for i in (1, 2, 3)))

    def on_clock(s):  # a 1 s stamp of the TP file on the data clock (midnight)
        return s + 86400 if s < t[0] - 40000 else s

    # --- script stamps and parameters --------------------------------------------------------
    stamps, tp_name, reason = ({}, None, None)
    if fn["tp"] and (FILES / fn["tp"]).exists():
        stamps, tp_name, reason = read_tp(FILES / fn["tp"])
        if tp_name and tp_name != test:
            row["flags"].append(f"tp_names_{tp_name}")
    else:
        row["flags"].append("no_tp")
    script = read_script(FILES / fn["script"]) if fn["script"] and (FILES / fn["script"]).exists() else {}
    if not script:
        row["flags"].append("no_script")
    elif script.get("testpointName") not in (None, test):
        row["flags"].append(f"script_names_{script.get('testpointName')}")
    pre_stage = sum(num(script.get(k, 0)) for k in PRE_STAGE_KEYS) if script else 0.0
    stage_sum = sum(num(script.get(k, 0)) for k in STAGE_KEYS) if script else NAN
    has_c = script and any(k in script for k in ("constantFlowTimeSec", "constantFlowTime1Sec"))
    row.update(
        script_programmed_stop_s=stage_sum if has_c else NAN,   # from the ignition-stage stamp, as acme_script.py
        script_flow_timeout_s=num(script.get("flowTimeoutSec")),
        script_burner_limit_c=num(script.get("burnerTemperatureLimitC")),
        script_extinction_stops_test=script.get("extinctionDetectionStopsTest", ""),
        script_ramp_to_zero_s=num(script.get("rampFlowToZeroSec")) if script.get("useRampFlowToZero") == "true" else NAN,
    )

    # --- ignition stamp ----------------------------------------------------------------------
    flow_v = {c: smooth(t, d[c], 0.2) for c in need[3:]}
    if "Ignition and retraction" in stamps:
        ign, src = on_clock(stamps["Ignition and retraction"]), "TP"
    elif "Flame stabilization #1" in stamps:
        ign, src = on_clock(stamps["Flame stabilization #1"]) - STAMP_TO_STAGE_S, "stab1-2.5s"
    else:
        any_on = np.max(np.stack(list(flow_v.values())), axis=0) > 0.15
        e = first_sustained(t, any_on, t[0], 5.0)
        ign, src = (e + 1.0 if e == e else NAN), "sustained_flow_on+1s"
    if ign != ign:
        row["status"] = "no_ignition_reference"
        return row
    stage0 = on_clock(stamps["Flame stabilization #1"]) if "Flame stabilization #1" in stamps else ign + STAMP_TO_STAGE_S
    row.update(ignition_stamp_clock=ign, ignition_stamp_source=src)

    # --- flame onset: radiometers ------------------------------------------------------------
    rad = radiometer_channels(d)
    t0, method = NAN, "none"
    if rad:
        rs = sum(d[c] for c in rad)
        pre = (t > ign - 15) & (t < ign - 3)
        if pre.sum() < 10:
            pre = t < ign
        if pre.sum() >= 10:
            base, sd = float(np.median(rs[pre])), float(np.std(rs[pre]))
            hit = np.where((t > ign - 3) & (t < ign + 25) & (smooth(t, rs, 0.1) > base + max(ONSET_RISE_V, 8 * sd)))[0]
            if len(hit):
                t0, method = float(t[hit[0]]), "radiometer"
    # --- photomultipliers ---------------------------------------------------------------------
    dark = [dark_level(t, d[c]) for c in ("PMT1", "PMT2", "PMT3")]
    ex = smooth(t, sum(d[c] - k for c, k in zip(("PMT1", "PMT2", "PMT3"), dark)), 0.2)
    if max(dark) > 1.0:
        row["flags"].append("pmt_never_dark")
    armed = np.where((ex < PMT_LIT_V) & (t < ign + 25))[0]   # the PMTs idle at a rail until armed
    t_arm = float(t[armed[0]]) if len(armed) else NAN
    onset_pmt = first_sustained(t, ex > PMT_LIT_V, max(t_arm, ign - 3) if t_arm == t_arm else ign - 3, 0.2)
    if t0 != t0:
        if onset_pmt == onset_pmt and onset_pmt < ign + 25:
            t0, method = onset_pmt, "pmt"
        else:
            t0, method = ign + 2.5, "stamp+2.5s"
            row["flags"].append("onset_not_seen")
    row.update(onset_clock=t0, onset_method=method, onset_minus_stamp_s=t0 - ign,
               onset_pmt_minus_radiometer_s=onset_pmt - t0 if method == "radiometer" else NAN,
               file_end_s=float(t[-1]) - t0)
    lit = bool(np.any((t > t0 - 0.5) & (t < t0 + 1.5) & (ex > PMT_LIT_V)))
    pmt_end = first_sustained(t, ex < PMT_END_V, t0 + 0.5, PMT_HOLD_S) if lit else NAN
    pmt_end_tight = first_sustained(t, ex < PMT_END_V_TIGHT, t0 + 0.5, PMT_HOLD_S) if lit else NAN
    if not lit:
        row["flags"].append("pmt_not_lit_at_onset")
    elif pmt_end != pmt_end:
        row["flags"].append("pmt_lit_at_file_end")
    elif pmt_end_tight != pmt_end_tight or pmt_end_tight - pmt_end > 1.0:
        row["flags"].append("pmt_end_threshold_sensitive")
    # independent look: the steepest fall of the radiometer sum after onset
    rad_drop = NAN
    if rad:
        rsm = smooth(t, sum(d[c] for c in rad), 0.5)
        n = max(1, int(round(0.5 / row["dt_s"])))
        if len(rsm) > 2 * n:
            slope = rsm[n:] - rsm[:-n]
            ok = t[:-n] > t0 + RAD_FALL_AFTER_S
            if ok.any():
                rad_drop = float(t[:-n][ok][np.argmin(slope[ok])]) + 0.25
    row.update(pmt_lit=int(lit), pmt_end_s=pmt_end - t0, pmt_end_tight_s=pmt_end_tight - t0,
               radiometer_steepest_fall_s=rad_drop - t0)

    # --- flow: the command log, and the readbacks ----------------------------------------------
    t_ref = stage0 + pre_stage + 0.5          # the constant-flow stage has begun
    level, flow_change, src_fc = [NAN] * 3, NAN, "none"
    if fn["setpoints"] and (FILES / fn["setpoints"]).exists():
        ts, vs = read_setpoints(FILES / fn["setpoints"], t[0])
        before = np.where(ts <= t_ref)[0]
        if len(before):
            level = [float(x) for x in vs[before[-1]]]
            tol = np.maximum(0.02 * np.abs(np.array(level)), 0.005)
            later = np.where((ts > t_ref) & np.any(np.abs(vs - np.array(level)) > tol, axis=1))[0]
            src_fc = "setpoints"
            if len(later):
                flow_change = float(ts[later[0]])
    if src_fc == "none":  # no command log: the readbacks leave their level
        row["flags"].append("flow_change_from_readback")
        w = (t > t_ref) & (t < t_ref + 2)
        if w.sum() > 10:
            level = [float(np.median(v[w])) for v in flow_v.values()]
            off = np.zeros(len(t), bool)
            for v, lv in zip(flow_v.values(), level):
                if lv > 0.1:
                    off |= np.abs(v - lv) > 0.05 * lv
            flow_change, src_fc = first_sustained(t, off, t_ref + 2, 0.3), "readback"
    used = [v for v, lv in zip(flow_v.values(), level) if lv == lv and lv > 0.02]
    if not used:
        used = [v for v in flow_v.values() if np.median(v[(t > t0 + 2) & (t < t0 + 8)] if ((t > t0 + 2) & (t < t0 + 8)).any() else v) > 0.1]
        row["flags"].append("no_test_flow_level")
    flow_off = NAN
    if used:
        all_off = np.max(np.stack(used), axis=0) < FLOW_ON_V
        flow_off = first_sustained(t, all_off, t0, FLOW_GAP_S)
    row.update(flow_level_v="/".join(fmt(x, 4) for x in level), flow_change_source=src_fc,
               flow_change_s=flow_change - t0, flow_off_s=flow_off - t0,
               constant_stage_begins_s=t_ref - t0)

    # --- label from the sensors -----------------------------------------------------------------
    fc = flow_change if flow_change == flow_change else flow_off
    if not lit:
        label, end = "unreadable", NAN
    elif pmt_end == pmt_end and (fc != fc or pmt_end < fc - LABEL_MARGIN_S):
        label, end = "extinction", pmt_end
    elif fc == fc:
        label, end = "operator_termination", fc
    else:
        label, end = "burning_at_file_end", float(t[-1])
    if label == "extinction" and pmt_end < t_ref:
        row["flags"].append("ended_before_constant_stage")
    # the flame went dark in the second before the flow command: the label rests on the margin
    if pmt_end == pmt_end and fc == fc and 0.0 <= fc - pmt_end < 2 * LABEL_MARGIN_S:
        row["flags"].append("end_just_before_flow_change")
    row.update(label_sensor=label, end_s=end - t0)

    # --- the script's own account -----------------------------------------------------------------
    ext = on_clock(stamps["Extinction detected"]) if "Extinction detected" in stamps else NAN
    off = on_clock(stamps["MFCs off"]) if "MFCs off" in stamps else NAN
    if reason or "Test Point Failure" in stamps:
        tp_class = "abort"
    elif ext == ext and (off != off or ext <= off + 1.0):
        tp_class = "extinction_detected"
    elif off == off:
        tp_class = "scripted_end"
    else:
        tp_class = "no_stamps"
    row.update(tp_class=tp_class, tp_reason=reason or "", tp_extinction_detected_s=ext - t0, tp_mfcs_off_s=off - t0)
    agree = ""
    if tp_class != "no_stamps" and label in ("extinction", "operator_termination"):
        agree = int((label == "extinction") == (tp_class == "extinction_detected"))
    row["labels_agree"] = agree
    return row


FIELDS = ["test", "status", "date", "n_samples", "dt_s", "ignition_stamp_clock", "ignition_stamp_source",
          "onset_clock", "onset_method", "onset_minus_stamp_s", "onset_pmt_minus_radiometer_s",
          "pmt_lit", "pmt_end_s", "pmt_end_tight_s", "radiometer_steepest_fall_s",
          "constant_stage_begins_s", "flow_change_s", "flow_change_source", "flow_off_s", "file_end_s",
          "label_sensor", "end_s", "tp_class", "tp_reason", "tp_extinction_detected_s", "tp_mfcs_off_s",
          "labels_agree", "script_programmed_stop_s", "script_flow_timeout_s", "script_burner_limit_c",
          "script_extinction_stops_test", "script_ramp_to_zero_s", "flow_level_v", "pmt_gains", "mfc_gas",
          "mfc_full_scale", "flags"]


def cmd_endpoints(args):
    try:
        import numpy  # noqa: F401
    except ImportError:
        sys.exit("numpy is needed for the endpoint measurement")
    names = listing()
    tests = tests_of(args)
    rows = []
    for test in tests:
        try:
            row = measure(test, file_names(test, names))
        except Exception as e:  # one unreadable file must not hide the others
            row = dict(test=test, status=f"error:{type(e).__name__}", flags=[])
        row["flags"] = "|".join(row.get("flags", []))
        rows.append({k: fmt(row.get(k, "")) for k in FIELDS})
    OUT.mkdir(parents=True, exist_ok=True)
    path = OUT / (args.out or "endpoints.csv")
    with path.open("w", encoding="utf-8", newline="") as f:
        w = csv.DictWriter(f, fieldnames=FIELDS)
        w.writeheader()
        w.writerows(rows)
    ok = [r for r in rows if r["status"] == "ok"]
    # counts and statuses only: nothing here says how or when a test ended
    print(json.dumps(dict(
        wrote=str(path.relative_to(REPO)), tests=len(rows), status=dict(Counter(r["status"] for r in rows)),
        onset_method=dict(Counter(r["onset_method"] for r in ok)),
        flow_change_source=dict(Counter(r["flow_change_source"] for r in ok)),
        endpoint_resolved=sum(r["label_sensor"] in ("extinction", "operator_termination") for r in ok),
        endpoint_unresolved=sum(r["label_sensor"] not in ("extinction", "operator_termination") for r in ok),
        script_account_available=sum(r["labels_agree"] != "" for r in ok),
        sensor_and_script_agree=sum(r["labels_agree"] == "1" for r in ok),
        sensor_and_script_disagree=sum(r["labels_agree"] == "0" for r in ok),
        flag_counts=dict(Counter(re.sub(r"_names_.*", "_names_other", f) for r in ok for f in r["flags"].split("|") if f)),
    ), indent=1))
    return 0


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    sub = ap.add_subparsers(dest="cmd", required=True)
    sub.add_parser("conditions").set_defaults(fn=cmd_conditions)
    for name, fn in (("plan", cmd_plan), ("endpoints", cmd_endpoints)):
        p = sub.add_parser(name)
        p.add_argument("--tests", help="comma-separated test names")
        p.add_argument("--tests-file", help="a file of test names")
        p.add_argument("--included", action="store_true", help="every test with include_transfer = 1 in conditions.csv")
        if name == "plan":
            p.add_argument("--execute", action="store_true", help="download (default: dry run)")
        else:
            p.add_argument("--out", help="file name under runs/dev/data_all/work/flame_design/ (default endpoints.csv)")
        p.set_defaults(fn=fn)
    args = ap.parse_args()
    return args.fn(args)


if __name__ == "__main__":
    sys.exit(main())
