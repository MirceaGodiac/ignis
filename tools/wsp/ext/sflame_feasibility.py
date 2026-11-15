#!/usr/bin/env python3
"""ACME s-Flame (PSI-23): can the burner-flame models be run on it? Feasibility only, NO outcomes.

Reads, for the tests of the days frozen as train or validation in docs/wsp/atlas/splits/PSI-23.json:
the file names, the script parameter files (`as run<T>.txt`, else `<T>.txt`: settings fixed before
ignition), the two header lines of `<T>_data.csv` (date, MFC gas codes and full scales, channel
names) and its FIRST data row (chamber pressures PTA1-3 at the start of the record, before the
flame). It never opens a TP file (stamps include "Extinction detected"), a Comments file, a log, a
setpoint log, an image file, or anything of a test day or of a day the split file does not list.
It prints counts and ranges of conditions only and writes
runs/dev/data_all/work/transfer_acme/sflame/feasibility.json.

    python tools/wsp/ext/sflame_feasibility.py
"""
import collections
import json
import os
import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
RAW = REPO / "runs/dev/data_all/raw/PSI-23/Raw Data"
FD_RAW = REPO / "runs/dev/data_all/raw/PSI-10/Raw Data"
OUT = REPO / "runs/dev/data_all/work/transfer_acme/sflame"
PRE = "PSI-23_Raw Data_"
STAGE_KEYS = ["stabilizationTime1Sec", "stabilizationTime2Sec", "ignitionToTestRampTimeSec", "constantFlowTimeSec",
              "constantFlowTime1Sec", "testFlowRamp1To2TimeSec", "constantFlowTime2Sec", "testFlowRamp2To3TimeSec",
              "constantFlowTime3Sec"]


def split():
    s = json.loads((REPO / "docs/wsp/atlas/splits/PSI-23.json").read_text(encoding="utf-8"))
    g = s["strata"]["all"]["groups"]
    return {d.split(":")[-1]: p for p in g for d in g[p]}


def script(path):
    out = {}
    for line in path.read_text(encoding="utf-8", errors="replace").replace("\r", "\n").split("\n"):
        k, sep, v = line.strip().rstrip(",").partition(":")
        if sep:
            out[k.strip()] = v.strip()
    return out


def header(path):
    with open(path, encoding="utf-8", errors="replace") as f:
        h, cols, first = f.readline(), f.readline(), f.readline()
    parts = [p.strip() for p in h.split(",")]
    meta = {}
    m = re.match(r"(\d{4}-\d\d-\d\d) (\d{3})", parts[0])
    if m:
        meta["date"] = m.group(1)
    for p in parts[1:]:
        if ":" in p:
            k, v = p.split(":", 1)
            meta[k.strip()] = v.strip()
    cols = [c.strip() for c in cols.split(",")]
    vals = first.strip().split(",")
    row = dict(zip(cols, vals)) if len(vals) == len(cols) else {}
    return meta, cols, row


def rng(v):
    v = [x for x in v if x == x]
    return [min(v), max(v)] if v else None


def main():
    part_of = split()
    names = os.listdir(RAW)
    tests = collections.defaultdict(set)
    for n in names:
        m = re.match(re.escape(PRE) + r"(?:as run)?(\d{5}[A-Z]+\d+)(_data\.csv|_mfcSetpoints\.csv|_TP\.txt|_Comments\.txt|\.txt)$", n)
        if m:
            tests[m.group(1)].add(("as run" if "as run" in n else "") + m.group(2))
    by_part = collections.Counter(part_of.get(t[:5], "not listed (held)") for t in tests)
    rows, keys_union = [], collections.Counter()
    for t in sorted(tests):
        part = part_of.get(t[:5])
        if part not in ("train", "validation"):
            continue  # test days and unlisted days: names counted above, nothing opened
        f = tests[t]
        sp = RAW / f"{PRE}as run{t}.txt" if "as run.txt" in f else (RAW / f"{PRE}{t}.txt" if ".txt" in f else None)
        s = script(sp) if sp else {}
        keys_union.update(s.keys())
        r = dict(test=t, day=t[:5], part=part, files=sorted(f), script_source="as run" if "as run.txt" in f else (".txt" if sp else "none"))
        num = lambda k: float(s[k]) if k in s and re.fullmatch(r"-?[\d.]+(e-?\d+)?", s[k]) else float("nan")  # noqa: E731
        r.update(fuel_test_n2slpm=num("fuelBottleTestFlowRateN2Slpm"), n2_test_n2slpm=num("nitrogenTestFlowRateN2Slpm"),
                 fuel_ign_n2slpm=num("fuelBottleIgnitionFlowRateN2Slpm"), n2_ign_n2slpm=num("nitrogenIgnitionFlowRateN2Slpm"),
                 ramp_holds=num("numberOfRampHolds"), ramp_fuel=num("rampSizeFuelSlpm"), ramp_n2=num("rampSizeN2Slpm"),
                 ign_to_test_ramp_s=num("ignitionToTestRampTimeSec"), burner_limit_c=num("burnerTemperatureLimitC"),
                 flow_timeout_s=num("flowTimeoutSec"), ext_stops=s.get("extinctionDetectionStopsTest", ""),
                 stage_sum_s=sum(num(k) for k in STAGE_KEYS if k in s) if s else float("nan"),
                 constant_key=[k for k in ("constantFlowTimeSec", "constantFlowTime1Sec") if k in s])
        if "_data.csv" in f:
            meta, cols, first = header(RAW / f"{PRE}{t}_data.csv")
            r.update(date=meta.get("date"), mfc_gas="/".join(meta.get(f"MFCA{i} Gas", "") for i in (1, 2, 3)),
                     mfc_fs="/".join(meta.get(f"MFCA{i} Full Scale", "") for i in (1, 2, 3)), cols=cols,
                     pta_first=[float(first[c]) if c in first else float("nan") for c in ("PTA1", "PTA2", "PTA3")])
        rows.append(r)
    # the same first-row pressure readings on a few Flame Design files, against its table pinit (a scale check)
    fd_check = []
    import csv
    tab = {r["Test"]: r for r in csv.DictReader(open(FD_RAW.parent / "Experimental table/PSI-10_Experimental table_Flame Design_Normal Flame.csv",
                                                         encoding="utf-8-sig"))}
    for t in ("19115A1", "19137A1", "19150A1", "19182N1", "21301A1"):  # train / validation days of Flame Design
        p = FD_RAW / f"PSI-10_Raw Data_{t}_data.csv"
        if p.exists() and t in tab:
            _, _, first = header(p)
            fd_check.append(dict(test=t, pinit_atm=tab[t]["pinit (atm)"], pta_first=[first.get(c) for c in ("PTA1", "PTA2", "PTA3")]))
    FD_COLS = header(FD_RAW / "PSI-10_Raw Data_19115A1_data.csv")[1]
    need = ["PMT1", "PMT2", "PMT3", "MFCA1StaticMixerFlow", "MFCA2BypassFlow", "MFCA3FuelFlow"]
    rep = dict(
        tests_by_name_by_part=dict(by_part),
        opened=dict(tests=len(rows), days=sorted({r["day"] for r in rows})),
        files_per_test=dict(collections.Counter(x for r in rows for x in r["files"])),
        script_source=dict(collections.Counter(r["script_source"] for r in rows)),
        script_keys=dict(keys_union),
        by_day={}, fd_pressure_scale_check=fd_check,
        channels=dict(distinct_column_sets=len({tuple(r["cols"]) for r in rows if "cols" in r}),
                      missing_vs_flame_design_19115A1=sorted({c for r in rows if "cols" in r for c in FD_COLS} - {c for r in rows if "cols" in r for c in r["cols"]}),
                      endpoint_channels_present=sum(1 for r in rows if "cols" in r and all(c in r["cols"] for c in need)),
                      radiometer_named=dict(collections.Counter(",".join(c for c in r["cols"] if c.startswith("Radiometer") and not c.endswith("CJ"))
                                                                for r in rows if "cols" in r)),
                      with_data=sum(1 for r in rows if "cols" in r)))
    for d in sorted({r["day"] for r in rows}):
        rr = [r for r in rows if r["day"] == d]
        rep["by_day"][d] = dict(
            part=rr[0]["part"], tests=len(rr), with_data=sum("cols" in r for r in rr),
            dates=sorted({r.get("date") for r in rr if r.get("date")}),
            mfc_gas=dict(collections.Counter(r.get("mfc_gas") for r in rr)), mfc_fs=dict(collections.Counter(r.get("mfc_fs") for r in rr)),
            fuel_test_n2slpm=rng([r["fuel_test_n2slpm"] for r in rr]), n2_test_n2slpm=rng([r["n2_test_n2slpm"] for r in rr]),
            ignition_equals_test_flow=sum(r["fuel_ign_n2slpm"] == r["fuel_test_n2slpm"] and r["n2_ign_n2slpm"] == r["n2_test_n2slpm"] for r in rr),
            ramped=sum((r["ramp_holds"] or 0) > 0 or (r["ramp_fuel"] or 0) != 0 or (r["ramp_n2"] or 0) != 0 for r in rr),
            ign_to_test_ramp_s=rng([r["ign_to_test_ramp_s"] for r in rr]), stage_sum_s=rng([r["stage_sum_s"] for r in rr]),
            burner_limit_c=dict(collections.Counter(r["burner_limit_c"] for r in rr)),
            extinction_stops_test=dict(collections.Counter(r["ext_stops"] for r in rr)),
            constant_key=dict(collections.Counter("|".join(r["constant_key"]) for r in rr)),
            pta_first=[rng([r["pta_first"][i] for r in rr if "pta_first" in r]) for i in range(3)])
    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / "feasibility.json").write_text(json.dumps(dict(rep, rows=rows), indent=1), encoding="utf-8")
    print(json.dumps(rep, indent=1))
    return 0


if __name__ == "__main__":
    sys.exit(main())
