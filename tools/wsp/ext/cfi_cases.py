#!/usr/bin/env python3
"""The cases file of arm F on CFI (PSI-39), PROTOCOL 12.7: the 7 n-dodecane tests with a processed series
that are free droplets inside the declared domain and off the test days (FLEX-1206, 1239, 1240, 1250, 1264
on train days; FLEX-1221, 1223 on validation days). Writes
`runs/dev/data_all/work/transfer_droplet/cfi/cases.jsonl` in the generic format of
`tools/wsp/ext/transfer_droplet.py`. Reads only the `DropletData.txt` of those 7 tests and their
condition rows of `cfi/conditions.jsonl` (built by `cfi_conditions.py`); refuses to read a test-day test.

Choices fixed before any CFI series was read for scoring:
  * liquid diameter only (`DropletDiameter_mm`, mm -> m), time = `DropletTime_sec` as written, handed over
    under the FLEX time origin (the releases refuse any other string); flags exactly as the FLEX ingest
    sets them (`ignis-data/src/families/flex.rs`: duplicate_timestamp, non_monotonic_time, gap_gt_1s,
    missing, non_finite, non_positive; flagged points kept); the flame-camera series are not handed over.
  * conditions: x_o2, x_he from the campaign spreadsheet's Master Sheet, x_co2 = 0 (no CO2 in CFI),
    pressure = the nominal table pressure x 101325 Pa, fuel text "n-Dodecane". No endpoint (a case is at
    risk while the series holds a valid diameter at or after the cutoff, the rule of a run without one).
  * session_id = the day group of the frozen split; the row carries `partition` (train / validation).

    python -I tools/wsp/ext/cfi_cases.py
"""
import hashlib
import json
import math
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
RAW = REPO / "runs/dev/data_all/raw/PSI-39/Raw Data"
WORK = REPO / "runs/dev/data_all/work/transfer_droplet/cfi"
TESTS = ["1206", "1239", "1240", "1250", "1264", "1221", "1223"]


def series(path, run_id):
    sha = hashlib.sha256(path.read_bytes()).hexdigest()
    lines = path.read_text(encoding="utf-8", errors="replace").splitlines()
    head = lines[0].split("\t")
    ti, vi = head.index("DropletTime_sec"), head.index("DropletDiameter_mm")
    out, seen, prev = [], set(), None
    for line_no, line in enumerate(lines[1:], start=2):
        c = line.split("\t")

        def num(i):
            try:
                x = float(c[i])
            except (ValueError, IndexError):
                return None
            return x
        t, v = num(ti), num(vi)
        if t is None or not math.isfinite(t):
            continue
        flags = []
        if t in seen:
            flags.append("duplicate_timestamp")
        seen.add(t)
        if prev is not None:
            if t < prev:
                flags.append("non_monotonic_time")
            elif t - prev > 1.0:
                flags.append("gap_gt_1s")
        prev = t if prev is None or t > prev else prev
        si = None
        if v is None:
            flags.append("missing")
        elif not math.isfinite(v):
            flags.append("non_finite")
        else:
            if v <= 0.0:
                flags.append("non_positive")
            si = v / 1000.0
        out.append(dict(quantity="liquid_diameter", channel="DropletData", time_s=t, available_at_s=t, value_si=si,
                        raw_value=v if (v is not None and math.isfinite(v)) else None, raw_unit="mm",
                        quality_flags=flags, source=path.name, column="DropletDiameter_mm", row=line_no, sha256=sha,
                        note="time_s is NASA's processed time base for this channel (origin not adjudicated)"))
    return out


def main():
    cond = {r["flex"]: r for r in map(json.loads, (WORK / "conditions.jsonl").read_text(encoding="utf-8").splitlines())}
    rows = []
    for t in TESTS:
        c = cond[t]
        assert c["partition"] in ("train", "validation"), f"FLEX-{t} is on a {c['partition']} day: not read"
        assert c["fuel"] == "n-dodecane" and c["fibre"] == "free" and c["droplet_series"], t
        src = "PSI-39_Science Documents_CFI_DataSummary_Spreadsheet_Jan2021.xlsx (Master Sheet)"
        conditions = dict(
            x_o2=dict(value=c["x_o2"], unit="1", source=src, column="Oxygen"),
            x_co2=dict(value=0.0, unit="1", source=src, column="(no CO2 column; CFI used none)", evidence="ignis_derived"),
            x_he=dict(value=c["x_he"], unit="1", source=src, column="Helium"),
            pressure=dict(value=c["pressure_pa"], unit="Pa", source=src, column="Pressure (nominal atm x 101325)"),
            fuel=dict(value="n-Dodecane", unit=None, source=src, column="Fuel (C12H26)"))
        obs = series(RAW / f"PSI-39_Raw Data_FLEX-{t}_DropletData.txt", f"PSI-39:FLEX-{t}")
        rows.append(dict(run_id=f"PSI-39:FLEX-{t}", session_id=c["group"], investigation="PSI-39", partition=c["partition"],
                         test=f"FLEX-{t}", cir=c["cir"], fuel="n-dodecane",
                         condition_summary=dict(x_o2=c["x_o2"], x_he=c["x_he"], pressure_atm=c["pressure_atm"],
                                                d0_mm_table=c.get("d0_mm_table"), support=c["fibre"]),
                         conditions=conditions, observations=obs, endpoint=None))
    WORK.mkdir(parents=True, exist_ok=True)
    with (WORK / "cases.jsonl").open("w", encoding="utf-8", newline="\n") as f:
        for r in rows:
            f.write(json.dumps(r, ensure_ascii=False) + "\n")
    print(f"wrote {len(rows)} tests: " + ", ".join(f"{r['test']} ({r['partition']})" for r in rows))
    return 0


if __name__ == "__main__":
    sys.exit(main())
