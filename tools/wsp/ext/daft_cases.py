#!/usr/bin/env python3
"""Cases file of DAFT / DAFT-2 (PSI-47), held whole (docs/wsp/atlas/splits/PSI-47.json). Feasibility and
definitions: docs/wsp/feedback/daft_feasibility.md.

DAFT checked a condensation particle counter (modified TSI P-Trak, number concentration) against a light-
scattering photometer (TSI DustTrak, mass concentration) in ISS cabin air, in dust released by separating
Velcro, and (DAFT-2, 2006) in Arizona Road Dust held in Mylar bags. A case is one air sample seen by both
instruments: the forecast is the P-Trak number concentration from what the reference DustTrak read on the
same air, and from the kind of air.

Sources, all read as written:
  * Session D2 (GMT 063, 2005), tests 02 (ER4, Velcro) and 03 (Lab 1 Port 6): instrument logs
    `Raw Data/PSI-47_Raw Data_{PTrk,DTrk}000{2,3}.txt` (1 s per line) and the event table of the analysed
    workbook `PSI-47_Analyzed Data_flight data 063_TIMING.xls`, sheet TIMING ("seconds after log" of each
    instrument, read by NASA from the video). A window is the time between two events in a state that is
    one of: hepa (HEPA filter on the inlet), cabin (sample hose on, no Velcro), velcro (Velcro being
    separated). The first 10 s of every window are dropped (the DustTrak time constant, sheet Headers of
    the Retrieved Data Summary); a window shorter than 20 s after that is not a case. Fixed before any
    P-Trak value was computed.
  * DAFT-2 bags (21 August 2006): the voice-downlinked readings `PSI-47_Analyzed Data_ISS_Daft_Aug21_0611zgy
    time hist.xls`, sheet Data: one case per reading with both instruments.
  * Not used: session D1 (GMT 053), and test 04 of D2: no event table, so the HEPA zero and the sampled air
    cannot be told apart (NASA synchronised them on video, which is not in the archive).

Written (runs/dev/data_all/work/transfer_daft/): cases_held_inputs.jsonl, cases_held_outcomes.jsonl. Nothing of
the outcomes is printed.

Target: P-Trak number concentration N of the window or reading, 1/m3. Score: CRPS of ln(1 + N / 1e6 m^-3)
(N in particles per cm3 plus one, so that a HEPA window near zero is scoreable), from quantiles 5..95 %.

    python -P tools/wsp/ext/daft_cases.py build      # -P, not -I: xlrd is in the user site-packages
"""
import json
import math
import re
import sys
from collections import Counter, defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from samer_cases import LEVELS, crps_quantiles  # noqa: E402  (same quantile CRPS)

REPO = Path(__file__).resolve().parents[3]
RAW = REPO / "runs/dev/data_all/raw/PSI-47"
TIMING = RAW / "Analyzed Data/PSI-47_Analyzed Data_flight data 063_TIMING.xls"
BAGS = RAW / "Analyzed Data/PSI-47_Analyzed Data_ISS_Daft_Aug21_0611zgy time hist.xls"
OUT = REPO / "runs/dev/data_all/work/transfer_daft"
TRIM_S, MIN_S = 10, 20
STATE_AFTER = {"hepa on": "hepa", "hepa off": None, "hose on": "cabin", "velcro start": "velcro",
               "velcro stop": "cabin", "hose off": None, "log on": None, "log off": None}


def read_log(path):
    """Instrument log: header lines, then one value per line at 1 s. Returns (header, [value or None])."""
    # The logs end their lines in CR CR LF: one line per value once CRs are dropped.
    lines = path.read_bytes().replace(b"\r", b"").decode("utf-8", errors="replace").split("\n")
    head, vals, n_points = {}, [], None
    started = False
    for ln in lines:
        if not started:
            if ":" in ln:
                k, v = ln.split(":", 1)
                head[k.strip()] = v.strip()
                if k.strip() == "Number of points":
                    n_points = int(v.strip())
                    started = True
            continue
        s = ln.strip()
        try:
            x = float(s)
            vals.append(x if math.isfinite(x) else None)
        except ValueError:
            vals.append(None)
    while vals and vals[-1] is None:
        vals.pop()
    head["points_read"] = len(vals)
    head["points_declared"] = n_points
    return head, vals


def read_timing():
    import xlrd
    sh = xlrd.open_workbook(str(TIMING)).sheet_by_name("TIMING")
    tests = {}
    for c0, name in ((0, "02"), (7, "03")):
        loc = re.sub(r"^Loc\w*:\s*", "", str(sh.cell_value(3, c0))).strip()
        events = []
        for r in range(6, sh.nrows):
            ev = str(sh.cell_value(r, c0)).strip()
            if not ev:
                continue
            p, d = sh.cell_value(r, c0 + 2), sh.cell_value(r, c0 + 4)
            events.append((ev.lower(), int(p), int(d), r + 1))
        tests[name] = dict(location=loc, events=events)
    return tests


def windows(events):
    out, state = [], None
    for (ev, p, d, row), nxt in zip(events, events[1:] + [None]):
        state = STATE_AFTER.get(ev, "unknown") if ev in STATE_AFTER else "unknown"
        if nxt is None or state in (None, "unknown"):
            continue
        out.append(dict(state=state, ptrak=(p + TRIM_S, nxt[1]), dusttrak=(d + TRIM_S, nxt[2]),
                        events=(ev, nxt[0]), timing_rows=(row, nxt[3])))
    return [w for w in out if w["ptrak"][1] - w["ptrak"][0] >= MIN_S and w["dusttrak"][1] - w["dusttrak"][0] >= MIN_S]


def mean_window(vals, a, b):
    """Mean of samples with time a..b s (sample k of the log is at t = k s, k from 1)."""
    xs = [vals[k - 1] for k in range(a, b + 1) if 1 <= k <= len(vals) and vals[k - 1] is not None]
    return (sum(xs) / len(xs), len(xs)) if xs else (None, 0)


def d2_cases():
    tim = read_timing()
    cases = []
    for t, info in tim.items():
        ph, pv = read_log(RAW / f"Raw Data/PSI-47_Raw Data_PTrk00{t}.txt")
        dh, dv = read_log(RAW / f"Raw Data/PSI-47_Raw Data_DTrk00{t}.txt")
        for k, w in enumerate(windows(info["events"]), start=1):
            dm, dn = mean_window(dv, *w["dusttrak"])
            pm, pn = mean_window(pv, *w["ptrak"])
            cid = f"PSI-47:D2:test{t}:w{k}"
            inputs = dict(
                air=dict(value=w["state"], unit=None, known="before_sample",
                         source=dict(file=TIMING.name, sheet="TIMING", rows=w["timing_rows"], events=w["events"])),
                location=dict(value=info["location"], unit=None, known="before_sample",
                              source=dict(file=TIMING.name, sheet="TIMING", row=4)),
                aerosol=dict(value="iss_cabin_air" if w["state"] != "velcro" else "velcro_release_in_cabin_air",
                             unit=None, known="before_sample"),
                window_duration=dict(value=float(w["ptrak"][1] - w["ptrak"][0]), unit="s", known="before_sample"),
                dusttrak_mass_concentration=dict(
                    value=dm * 1e-6 if dm is not None else None, unit="kg/m3", known="same_air_reference",
                    samples=dn, status="resolved" if dm is not None else "missing",
                    source=dict(file=f"PSI-47_Raw Data_DTrk00{t}.txt", seconds_after_log=w["dusttrak"],
                                note="mean of the 1 s readings; the instrument's resolution is 0.001 mg/m3")),
                dusttrak_inlet=dict(value="no impactor stated for 2005 sessions", unit=None, known="before_sample"),
            )
            outcome = dict(ptrak_number_concentration=dict(
                value=pm * 1e6 if pm is not None else None, unit="1/m3", samples=pn,
                status="resolved" if pm is not None else "missing",
                source=dict(file=f"PSI-47_Raw Data_PTrk00{t}.txt", seconds_after_log=w["ptrak"])))
            cases.append(dict(case_id=cid, investigation="PSI-47", group_id="PSI-47:session:D2", partition="held",
                              test=f"D2-{t}", inputs=inputs, outcome=outcome,
                              logs=dict(ptrak=dict(points_read=ph["points_read"], points_declared=ph["points_declared"]),
                                        dusttrak=dict(points_read=dh["points_read"], points_declared=dh["points_declared"]))))
    return cases


def bag_cases():
    import xlrd
    sh = xlrd.open_workbook(str(BAGS)).sheet_by_name("Data")
    density = sh.cell_value(27, 2)
    cases = []
    for b, c in ((1, 1), (2, 5), (3, 9)):
        for r in range(4, 20):
            t, gmt, p, d = sh.cell_value(r, 0), sh.cell_value(r, c), sh.cell_value(r, c + 1), sh.cell_value(r, c + 2)
            if not isinstance(p, float) or not isinstance(d, float):
                continue
            cid = f"PSI-47:2006-08-21:bag{b}:r{r + 1}"
            inputs = dict(
                air=dict(value="bag", unit=None, known="before_sample"),
                aerosol=dict(value="arizona_road_dust_fine_in_gn2", unit=None, known="before_sample",
                             source=dict(file="PSI-47_Experimental table_DAFT.csv", column="Sample Material")),
                dusttrak_inlet=dict(value="DAFT impactor, removes particles above 2.5 um (final report)", unit=None,
                                    known="before_sample"),
                bag=dict(value=b, unit=None, known="before_sample"),
                elapsed_time=dict(value=float(t), unit="s", known="before_sample",
                                  source=dict(file=BAGS.name, sheet="Data", row=r + 1, column=1),
                                  note="NASA's nominal elapsed time 'applied to all' bags"),
                gmt=dict(value=str(gmt).strip() or None, unit=None, known="before_sample",
                         source=dict(file=BAGS.name, sheet="Data", row=r + 1, column=c + 1)),
                dusttrak_mass_concentration=dict(value=d * 1e-6, unit="kg/m3", known="same_air_reference",
                                                 source=dict(file=BAGS.name, sheet="Data", row=r + 1, column=c + 3)),
                analyst_assumed_density=dict(value=density * 1000.0 if isinstance(density, float) else None,
                                             unit="kg/m3", known="before_sample",
                                             source=dict(file=BAGS.name, sheet="Data", row=28, column=3),
                                             note="the density NASA's sheet assumed to turn mass per particle into a diameter"),
            )
            outcome = dict(ptrak_number_concentration=dict(value=p * 1e6, unit="1/m3", status="resolved",
                                                           source=dict(file=BAGS.name, sheet="Data", row=r + 1, column=c + 2)))
            cases.append(dict(case_id=cid, investigation="PSI-47", group_id="PSI-47:2006-08-21:bags", partition="held",
                              test=f"bag{b}", inputs=inputs, outcome=outcome))
    return cases


def score(forecasts, outcomes):
    """forecasts: {case_id: [q05..q95] of N in 1/m3}. Returns aggregate, group-equal mean, per group, missing."""
    tr = lambda n: math.log(1.0 + max(n, 0.0) / 1e6)
    per, missing = defaultdict(list), []
    for r in outcomes:
        y = r["outcome"]["ptrak_number_concentration"]["value"]
        if y is None:
            continue
        if r["case_id"] not in forecasts:
            missing.append(r["case_id"])
            continue
        q = forecasts[r["case_id"]]
        assert len(q) == len(LEVELS)
        per[r["group_id"]].append(crps_quantiles(LEVELS, [tr(v) for v in q], tr(y)))
    allv = [s for v in per.values() for s in v]
    return dict(aggregate=sum(allv) / len(allv) if allv else None,
                group_equal=sum(sum(v) / len(v) for v in per.values()) / len(per) if per else None,
                n=len(allv), per_group={g: (sum(v) / len(v), len(v)) for g, v in per.items()}, missing=missing)


def smoke_baseline():
    """Reference forecast for the registered held look, constants from SAME-R TRAIN days only: the quantiles
    5..95 % of ln(N/M) (number per mass of smoke, 1/kg) over SAME-R train rows; forecast N = k_q M for each
    quantile. A transfer from smoke to dust, declared as such; DAFT is never used to set it."""
    from samer_cases import WORK, load, quantiles
    train = load(WORK / "transfer_samer/cases_train.jsonl")
    assert all(c["partition"] == "train" for c in train)
    v = []
    for c in train:
        n, m = (c["outcome"][k]["value"] for k in ("number_concentration", "mass_concentration"))
        if n and m and m > 0:
            v.append(math.log(n / m))
    k = [math.exp(x) for x in quantiles(v)]
    out = dict(name="SAME-R train smoke number per mass", levels=LEVELS, number_per_mass_per_kg=k, n_rows=len(v),
               source="runs/dev/data_all/work/transfer_samer/cases_train.jsonl", use="N_q = k_q * dusttrak_mass_concentration")
    (OUT / "baseline_smoke_number_per_mass.json").write_text(json.dumps(out, indent=1), encoding="utf-8")
    print("smoke baseline constants (1/kg):", [f"{x:.3g}" for x in k], "from", len(v), "SAME-R train rows")
    return k


def build():
    cases = d2_cases() + bag_cases()
    OUT.mkdir(parents=True, exist_ok=True)
    with (OUT / "cases_held_inputs.jsonl").open("w", encoding="utf-8", newline="\n") as f:
        for c in cases:
            f.write(json.dumps({k: v for k, v in c.items() if k != "outcome"}, ensure_ascii=False) + "\n")
    with (OUT / "cases_held_outcomes.jsonl").open("w", encoding="utf-8", newline="\n") as f:
        for c in cases:
            f.write(json.dumps(dict(case_id=c["case_id"], group_id=c["group_id"], partition="held",
                                    outcome=c["outcome"]), ensure_ascii=False) + "\n")
    cnt = Counter((c["group_id"], c["inputs"]["air"]["value"]) for c in cases)
    with_both = Counter(c["group_id"] for c in cases
                        if c["inputs"]["dusttrak_mass_concentration"]["value"] is not None
                        and c["outcome"]["ptrak_number_concentration"]["value"] is not None)
    print("cases by group and air:", dict(cnt))
    print("cases with the reference input and an outcome:", dict(with_both))
    for c in cases:
        if "logs" in c:
            lg = c["logs"]
            if lg["ptrak"]["points_read"] != lg["ptrak"]["points_declared"] or lg["dusttrak"]["points_read"] != lg["dusttrak"]["points_declared"]:
                print("  log length differs from its header:", c["test"], lg)
                break
    # Structural check of the scoring path on the held file: prints whether it ran, not a score.
    outs = [json.loads(l) for l in (OUT / "cases_held_outcomes.jsonl").read_text(encoding="utf-8").splitlines()]
    s = score({o["case_id"]: [1e6, 1e7, 1e8, 1e9, 1e10] for o in outs}, outs)
    print("score() ran on", s["n"], "held cases; result not printed")
    smoke_baseline()
    return 0


if __name__ == "__main__":
    sys.exit(build() if (len(sys.argv) < 2 or sys.argv[1] == "build") else print(__doc__))
