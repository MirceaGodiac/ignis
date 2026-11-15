#!/usr/bin/env python3
"""Cases files of the smoke experiments SAME-R (PSI-101, split by day) and SAME (PSI-102, held whole), from
NASA's reduced results workbook of both campaigns
(`PSI-101/Analyzed Data/PSI-101_Analyzed Data_SAME, SAME-R_Results.xlsx`, sheet "Combined Rslts Copy this
sheet"; its later copies "Moment Comparisons Jan 14 (2)" and "detection June 14" agree with it on every
train cell compared). One row of that sheet is one measurement of one run: a run number ("61") is the
measurement after the primary aging, the same number with "a" ("61a") the measurement of the smoke kept
for a secondary aging. Column "Flight 1=same 2=sameR" separates the campaigns.

Feasibility and definitions: docs/wsp/feedback/samer_feasibility.md and same_feasibility.md.

Partitions (docs/wsp/atlas/splits/PSI-101.json, PSI-102.json):
  * SAME-R: the day group of the test catalogue (runs/dev/data_all/work/atlas/tests.jsonl). A row whose run is
    on a test day is skipped as soon as its run number has been read: none of its cells is stored or written.
    A run that is not in the catalogue (run 112) has no frozen group and is skipped the same way.
  * SAME: every run is held (group PSI-102:matrix). Inputs go to cases_held_inputs.jsonl, outcomes to
    cases_held_outcomes.jsonl; nothing of the outcomes is printed.

Written (under runs/dev/data_all/work/):
  transfer_samer/cases_train.jsonl                 train days: inputs, outcome, prior (aged rows)
  transfer_samer/cases_validation_inputs.jsonl     validation days: inputs only
  transfer_samer/cases_validation_outcomes.jsonl   validation days: outcome and prior; never printed
  transfer_same/cases_held_inputs.jsonl            SAME: inputs only
  transfer_same/cases_held_outcomes.jsonl          SAME: outcome and prior; never printed

Targets (lane-local; no shared `Quantity` represents them):
  d10   count mean diameter M1/M0 of the smoke at the row's net aging time, m. Score: CRPS of ln(d10 / 1 m),
        from the quantiles 5, 25, 50, 75, 95 % (the project's quantile CRPS, tools/wsp/ext/transfer_droplet.py).
  iss   the ISS smoke detector's forward-scatter channel at full scale (|V| >= 6.0 V) in the reduced value of
        the row. Score: Brier of a probability.

    python -I tools/wsp/ext/samer_cases.py build              # writes the five files, prints counts only
    python -I tools/wsp/ext/samer_cases.py baselines          # train days of SAME-R only: baseline scores
"""
import json
import math
import sys
from collections import Counter, defaultdict
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
BOOK = REPO / "runs/dev/data_all/raw/PSI-101/Analyzed Data/PSI-101_Analyzed Data_SAME, SAME-R_Results.xlsx"
SHEET = "Combined Rslts Copy this sheet"
CATALOGUE = REPO / "runs/dev/data_all/work/atlas/tests.jsonl"
SPLITS = REPO / "docs/wsp/atlas/splits"
WORK = REPO / "runs/dev/data_all/work"
LEVELS = [0.05, 0.25, 0.5, 0.75, 0.95]
ISS_FULL_SCALE_V = 6.0
# Classical coagulation (Smoluchowski, monodisperse, continuum kernel with slip correction), constants as
# published in W. C. Hinds, Aerosol Technology, 2nd ed. (1999): K0 = 4kT/(3 eta) = 3.0e-16 m3/s at 293 K and
# 1 atm (eq. 12.11); K = K0 Cc(d); Cc = 1 + Kn (1.142 + 0.558 exp(-0.999/Kn)), Kn = 2 lambda/d, lambda = 0.066
# um (eqs. 3.18, 3.20). N(t) = N0/(1 + K N0 t); at fixed mass d(t) = d0 (1 + K N0 t)^(1/3) (eq. 12.13).
K0_M3_S = 3.0e-16
MFP_M = 0.066e-6

H_FLIGHT = "Flight 1=same 2=sameR, g=ground"
COL = dict(  # name in the case files -> header of the sheet (row 2)
    material="Sample Type", run="SAME #", description="Description", gmt="GMT", carousel="Carousel",
    sample_id="Sample ID",
    inlet_velocity="Inlet Velocity (cm/sec)", rt_r0="Rt/R0", heating="Duration (sec)",
    primary_aging="Primary Aging Duration (sec)", primary_mixing="Primary Mixing Duration (sec)",
    net_aging="Net Aging  Duration", secondary_volume="Secondary Diagnostics Volume % ",
    secondary_aging="Secondary Aging Duration (sec)", secondary_mixing="Secondary Mixing Duration (sec)",
    use_sts="Use STS Detector", use_ptrak="Use P-Trak", density="Particle density g/cc",
    sample_t="Sample Temperature C", mass_loss="Mass Loss mg",
    # outcomes
    n0="M0 best Diameter", m1="M1  (mm/cm^3)", d10="M1/M0 (micro-m)--AMD", d30="Diameter of Average mass (micro-m)",
    sigma_g="Hatch Choate Sigma g", dg="Hatch Choate Dg", mass="DustTrak B calibratred and corrected for dilution (mg/m3)",
    iss_scatter="ISS Detector Scatter (volts)", sts="STS Detector (volts)", sts_offset="STS Detector offset (volts)",
)


# ---------------------------------------------------------------------------------------------- scoring
def pinball(q, y, tau):
    return (tau - (1.0 if y < q else 0.0)) * (y - q)


def crps_quantiles(levels, q, y):
    """Quantile CRPS with trapezoidal weights, identical to tools/wsp/ext/transfer_droplet.py."""
    o = sorted(range(len(levels)), key=lambda i: levels[i])
    t, qq = [levels[i] for i in o], [q[i] for i in o]
    ext = [0.0] + t + [1.0]
    return 2.0 * sum((ext[k + 2] - ext[k]) / 2.0 * pinball(qq[k], y, t[k]) for k in range(len(t)))


def score(forecasts, outcomes, target="d10", transform=None):
    """forecasts: {case_id: [q05, q25, q50, q75, q95] in the target's SI unit} for target d10, or
    {case_id: probability} for target iss. outcomes: list of outcome rows (from a cases or outcomes file).
    Returns dict(aggregate, day_equal, n, per_day={group: (mean, n)}, missing=[case ids without forecast]).
    A case whose outcome is missing is not scored; a case with an outcome and no forecast is listed."""
    per, missing = defaultdict(list), []
    for r in outcomes:
        o = r["outcome"]
        if not o["scored"][target]:
            continue
        if target == "d10":
            y = o["d10_m"]["value"]
            if y is None:
                continue
            if r["case_id"] not in forecasts:
                missing.append(r["case_id"])
                continue
            tr = transform or (lambda v: math.log(v))
            q = forecasts[r["case_id"]]
            assert len(q) == len(LEVELS) and all(v > 0 for v in q), r["case_id"]
            s = crps_quantiles(LEVELS, [tr(v) for v in q], tr(y))
        elif target == "iss":
            y = o["iss_full_scale"]["value"]
            if y is None:
                continue
            if r["case_id"] not in forecasts:
                missing.append(r["case_id"])
                continue
            p = forecasts[r["case_id"]]
            assert 0.0 <= p <= 1.0
            s = (p - (1.0 if y else 0.0)) ** 2
        else:
            raise ValueError(target)
        per[r["group_id"]].append(s)
    allv = [s for v in per.values() for s in v]
    return dict(aggregate=sum(allv) / len(allv) if allv else None,
                day_equal=sum(sum(v) / len(v) for v in per.values()) / len(per) if per else None,
                n=len(allv), per_day={g: (sum(v) / len(v), len(v)) for g, v in sorted(per.items())},
                missing=missing)


# ---------------------------------------------------------------------------------------------- reading
def num(v):
    if isinstance(v, bool) or v is None:
        return None
    if isinstance(v, (int, float)):
        return float(v) if math.isfinite(v) else None
    return None


def field(value, unit, column, row, known, note=None, status=None):
    st = status or ("resolved" if value is not None else "missing")
    d = dict(value=value, unit=unit, status=st, source=dict(file=BOOK.name, sheet=SHEET, column=column, row=row),
             known=known)
    if note:
        d["note"] = note
    return d


def catalogue_groups():
    split = {}
    for psi in ("PSI-101", "PSI-102"):
        s = json.loads((SPLITS / f"{psi}.json").read_text(encoding="utf-8"))
        for st in s["strata"].values():
            for part, gs in st["groups"].items():
                for g in gs:
                    split[g] = part
    groups = {}
    for line in CATALOGUE.read_text(encoding="utf-8").splitlines():
        t = json.loads(line)
        if t["investigation_id"] == "PSI-101":
            groups[t["test_id"]] = (t["group_id"], split[t["group_id"]])
    return groups


def material_name(v):
    return {"silicone": "silicone", "teflon": "teflon", "kapton": "kapton", "lampwick": "lampwick",
            "pyrell": "pyrell", "dbp": "dibutyl_phthalate"}.get(str(v).strip().lower(), str(v).strip().lower())


def read_rows():
    import openpyxl
    wb = openpyxl.load_workbook(BOOK, read_only=True, data_only=True)
    rows = list(wb[SHEET].iter_rows(values_only=True))
    h = list(rows[1])
    idx = {k: h.index(c) for k, c in COL.items()}
    fi = h.index(H_FLIGHT)
    return rows, idx, fi


def build_case(r, rowno, idx, investigation, group, part, parent):
    """r: the row; parent: the run's primary-aging row (for an aged row) or None."""
    g = lambda k: r[idx[k]]
    run = str(g("run")).strip()
    aged = run.lower().endswith("a")
    src = parent if (aged and parent is not None) else (r, rowno)
    pr, prow = src
    pg = lambda k: pr[idx[k]]

    def from_parent(k, unit, known, conv=lambda x: x, note=None):
        v = num(pg(k))
        return field(conv(v) if v is not None else None, unit, COL[k], prow, known,
                     note=("taken from the run's primary-aging row" if aged else None) if note is None else note)

    vel = num(g("inlet_velocity"))
    net = num(g("net_aging"))
    st = num(pg("sample_t"))
    ml = num(pg("mass_loss"))
    dens = num(g("density"))
    # Secondary aging and volume are written on either the primary or the aged row; the larger is taken.
    sec_a = [num(x[idx["secondary_aging"]]) for x in (r, pr)]
    sec_v = [num(x[idx["secondary_volume"]]) for x in (r, pr)]
    sec_a = max([v for v in sec_a if v is not None], default=None) if aged else None
    sec_v = max([v for v in sec_v if v is not None], default=None) if aged else None
    inputs = dict(
        material=dict(value=material_name(g("material")), unit=None, status="resolved",
                      source=dict(file=BOOK.name, sheet=SHEET, column=COL["material"], row=rowno), known="before_test"),
        measurement=dict(value="aged" if aged else "primary", unit=None, status="resolved",
                         source=dict(file=BOOK.name, sheet=SHEET, column=COL["run"], row=rowno), known="before_test",
                         note="primary: smoke measured after the primary aging; aged: smoke kept for a secondary aging"),
        inlet_velocity=field(vel / 100.0 if vel is not None else None, "m/s", COL["inlet_velocity"], rowno,
                             "before_test", note="nominal duct air speed during heating (set point)"),
        heater_resistance_ratio=from_parent("rt_r0", "1", "before_test"),
        heating_duration=from_parent("heating", "s", "before_test", conv=lambda x: x if x > 0 else None),
        primary_aging_duration=from_parent("primary_aging", "s", "before_test"),
        primary_mixing_duration=from_parent("primary_mixing", "s", "before_test"),
        secondary_aging_duration=field(sec_a, "s", COL["secondary_aging"], rowno, "before_test",
                                       note="aged rows only; the larger of the cells on the primary and aged rows")
        if aged else field(None, "s", COL["secondary_aging"], rowno, "before_test", status="not_applicable"),
        secondary_volume_percent=field(sec_v, "%", COL["secondary_volume"], rowno, "before_test",
                                       note="aged rows only; share of the smoke kept for the secondary aging")
        if aged else field(None, "%", COL["secondary_volume"], rowno, "before_test", status="not_applicable"),
        net_aging_time=field(net, "s", COL["net_aging"], rowno, "horizon",
                             note="age of the smoke when this row was measured: the time the forecast is asked for"),
        sample_temperature=field(st + 273.15 if (st is not None and st > 0) else None, "K", COL["sample_t"], prow,
                                 "during_heating",
                                 note="NASA's sample heating temperature, derived from the logged wire resistance"),
        mass_loss=field(ml * 1e-6 if (ml is not None and ml > 0) else None, "kg", COL["mass_loss"], prow,
                        "after_test_weighing",
                        note="post-flight weighing of the sample; 0 in the sheet is read as not weighed"),
        particle_density=field(dens * 1000.0 if dens is not None else None, "kg/m3", COL["density"], rowno,
                               "before_test", note="NASA's assumed particle density for the material"),
        sts_detector_in_line=dict(value=str(g("use_sts")).strip().lower().startswith("y"), unit=None,
                                  status="resolved", known="before_test",
                                  source=dict(file=BOOK.name, sheet=SHEET, column=COL["use_sts"], row=rowno)),
        gmt_day=dict(value=str(g("gmt")).strip(), unit=None, status="resolved", known="before_test",
                     source=dict(file=BOOK.name, sheet=SHEET, column=COL["gmt"], row=rowno)),
    )
    if inputs["net_aging_time"]["value"] is None:
        inputs["net_aging_time"]["status"] = "missing"
        inputs["net_aging_time"]["raw"] = str(g("net_aging"))

    def out(k, unit, conv):
        v = num(g(k))
        raw = g(k)
        f = field(conv(v) if v is not None else None, unit, COL[k], rowno, "outcome")
        if v is None and raw not in (None, ""):
            f["raw"] = str(raw)
        return f

    d10 = out("d10", "m", lambda v: v * 1e-6 if v > 0 else None)
    iss = num(g("iss_scatter"))
    outcome = dict(
        d10_m=d10,
        d30_m=out("d30", "m", lambda v: v * 1e-6 if v > 0 else None),
        dg_m=out("dg", "m", lambda v: v * 1e-6 if v > 0 else None),
        sigma_g=out("sigma_g", "1", lambda v: v if v > 0 else None),
        number_concentration=out("n0", "1/m3", lambda v: v * 1e6 if v > 0 else None),
        mass_concentration=out("mass", "kg/m3", lambda v: v * 1e-6),
        iss_scatter=out("iss_scatter", "V", lambda v: abs(v)),
        iss_full_scale=dict(value=(abs(iss) >= ISS_FULL_SCALE_V) if iss is not None else None, unit=None,
                            status="resolved" if iss is not None else "missing", known="outcome",
                            rule=f"|ISS Detector Scatter (volts)| >= {ISS_FULL_SCALE_V}",
                            source=dict(file=BOOK.name, sheet=SHEET, column=COL["iss_scatter"], row=rowno)),
        sts_detector=out("sts", "V", lambda v: v),
        sts_detector_offset=out("sts_offset", "V", lambda v: v),
    )
    if not inputs["sts_detector_in_line"]["value"]:
        for k in ("sts_detector", "sts_detector_offset"):
            outcome[k]["status"] = "not_applicable"
    # A case is scored when the forecast's horizon (net aging time) and the target's outcome both exist.
    horizon = inputs["net_aging_time"]["value"] is not None
    outcome["scored"] = dict(d10=horizon and outcome["d10_m"]["value"] is not None,
                             iss=horizon and outcome["iss_full_scale"]["value"] is not None,
                             rule="net_aging_time present and the target's outcome present")
    prefix = "PSI-101" if investigation == "samer" else "PSI-102"
    case = dict(case_id=f"{prefix}:{run}", investigation=prefix, campaign="SAME-R" if investigation == "samer" else "SAME",
                test=run, parent_test=run[:-1] if aged else None, group_id=group, partition=part,
                inputs=inputs, outcome=outcome)
    return case


def prior_of(primary_case):
    """What was measured on the run's primary-aging row: allowed as input to the aged row's forecast."""
    if primary_case is None:
        return None
    o, i = primary_case["outcome"], primary_case["inputs"]
    return dict(test=primary_case["test"], net_aging_time_s=i["net_aging_time"]["value"],
                d10_m=o["d10_m"]["value"], d30_m=o["d30_m"]["value"], sigma_g=o["sigma_g"]["value"],
                number_concentration_per_m3=o["number_concentration"]["value"],
                mass_concentration_kg_m3=o["mass_concentration"]["value"], iss_full_scale=o["iss_full_scale"]["value"])


def build():
    groups = catalogue_groups()
    rows, idx, fi = read_rows()
    cases = {"samer": [], "same": []}
    skipped = Counter()
    primary = {"samer": {}, "same": {}}
    rowpos = {"samer": {}, "same": {}}
    # Pass 1: the keys only, to find each run's primary row; test-day and unlisted rows are never kept.
    keep = []
    for n, r in enumerate(rows[2:], start=3):
        fl = str(r[fi]).strip()
        if fl not in ("1", "2"):
            skipped["not a flight row (ground or empty)"] += 1
            continue
        inv = "samer" if fl == "2" else "same"
        run = str(r[idx["run"]]).strip()
        if inv == "samer":
            if run not in groups:
                skipped["SAME-R run not in the test catalogue (no frozen group)"] += 1
                continue
            g, part = groups[run]
            if part == "test":
                skipped["SAME-R test day: not stored"] += 1
                continue
        else:
            g, part = "PSI-102:matrix", "held"
        keep.append((inv, n, r, g, part, run))
        if not run.lower().endswith("a"):
            rowpos[inv][run] = (r, n)
    for inv, n, r, g, part, run in keep:
        aged = run.lower().endswith("a")
        parent = rowpos[inv].get(run[:-1]) if aged else None
        c = build_case(r, n, idx, inv, g, part, parent)
        if not aged:
            primary[inv][run] = c
        cases[inv].append(c)
    for inv in cases:
        for c in cases[inv]:
            c["prior"] = prior_of(primary[inv].get(c["parent_test"])) if c["parent_test"] else None
    write(cases)
    report(cases, skipped)


def write(cases):
    def dump(path, rows):
        path.parent.mkdir(parents=True, exist_ok=True)
        with path.open("w", encoding="utf-8", newline="\n") as f:
            for r in rows:
                f.write(json.dumps(r, ensure_ascii=False) + "\n")

    def inputs_only(c):
        return {k: v for k, v in c.items() if k not in ("outcome", "prior")}

    def outcomes_only(c):
        return dict(case_id=c["case_id"], group_id=c["group_id"], partition=c["partition"], outcome=c["outcome"],
                    prior=c["prior"])
    sr = cases["samer"]
    dump(WORK / "transfer_samer/cases_train.jsonl", [c for c in sr if c["partition"] == "train"])
    dump(WORK / "transfer_samer/cases_validation_inputs.jsonl", [inputs_only(c) for c in sr if c["partition"] == "validation"])
    dump(WORK / "transfer_samer/cases_validation_outcomes.jsonl", [outcomes_only(c) for c in sr if c["partition"] == "validation"])
    sm = cases["same"]
    dump(WORK / "transfer_same/cases_held_inputs.jsonl", [inputs_only(c) for c in sm])
    dump(WORK / "transfer_same/cases_held_outcomes.jsonl", [outcomes_only(c) for c in sm])


def report(cases, skipped):
    """Counts only. For validation and held rows it counts inputs and the presence of an outcome, never a value."""
    print("skipped rows:", dict(skipped))
    for inv, rs in cases.items():
        by = Counter()
        for c in rs:
            i, o = c["inputs"], c["outcome"]
            core = all(i[k]["value"] is not None for k in
                       ("inlet_velocity", "heater_resistance_ratio", "heating_duration", "net_aging_time"))
            key = (c["partition"], i["measurement"]["value"])
            by[key + ("rows",)] += 1
            by[key + ("core inputs",)] += core
            by[key + ("core inputs + d10",)] += core and o["d10_m"]["value"] is not None
            by[key + ("core inputs + iss",)] += core and o["iss_full_scale"]["value"] is not None
            by[key + ("sample temperature",)] += i["sample_temperature"]["value"] is not None
            by[key + ("mass loss",)] += i["mass_loss"]["value"] is not None
        days = Counter((c["partition"], c["group_id"]) for c in rs)
        print(f"== {inv}: {len(rs)} rows in {len(days)} groups")
        for k in sorted(by):
            print("  ", " / ".join(k), by[k])


# ---------------------------------------------------------------------------------------------- baselines
def load(path):
    return [json.loads(l) for l in path.read_text(encoding="utf-8").splitlines() if l.strip()]


def quantiles(xs, levels=LEVELS):
    """Empirical quantiles, linear interpolation between order statistics (numpy's default rule)."""
    s = sorted(xs)
    out = []
    for p in levels:
        h = (len(s) - 1) * p
        lo = math.floor(h)
        hi = min(lo + 1, len(s) - 1)
        out.append(s[lo] + (h - lo) * (s[hi] - s[lo]))
    return out


def slip(d):
    kn = 2.0 * MFP_M / d
    return 1.0 + kn * (1.142 + 0.558 * math.exp(-0.999 / kn))


def coagulation_point(c):
    """d10 of an aged row from its primary row: d_prior (1 + K N0 dt)^(1/3), K = K0 Cc(d30_prior)."""
    p = c["prior"]
    t = c["inputs"]["net_aging_time"]["value"]
    if not p or None in (p["d10_m"], p["number_concentration_per_m3"], p["net_aging_time_s"], t):
        return None
    dref = p["d30_m"] or p["d10_m"]
    k = K0_M3_S * slip(dref)
    return p["d10_m"] * (1.0 + k * p["number_concentration_per_m3"] * max(t - p["net_aging_time_s"], 0.0)) ** (1 / 3)


def baselines():
    train = load(WORK / "transfer_samer/cases_train.jsonl")
    assert all(c["partition"] == "train" for c in train)
    scored = [c for c in train if c["outcome"]["scored"]["d10"]]
    ln = lambda c: math.log(c["outcome"]["d10_m"]["value"])
    mat = lambda c: c["inputs"]["material"]["value"]
    days = sorted({c["group_id"] for c in train})
    print(f"SAME-R train: {len(train)} rows on {len(days)} days; {len(scored)} with d10")

    def q_exp(xs):
        return [math.exp(v) for v in quantiles(xs)]

    # In-sample constants (the named baselines' constants, from train days only).
    allq = q_exp([ln(c) for c in scored])
    by_mat = defaultdict(list)
    for c in scored:
        by_mat[mat(c)].append(ln(c))
    matq = {m: q_exp(v) for m, v in by_mat.items()}
    by_mk = defaultdict(list)
    for c in scored:
        by_mk[(mat(c), c["inputs"]["measurement"]["value"])].append(ln(c))
    print("constants (d10, um, quantiles 5/25/50/75/95):")
    print("  all     ", [round(v * 1e6, 4) for v in allq])
    for m in sorted(matq):
        print(f"  {m:9s}", [round(v * 1e6, 4) for v in matq[m]], f"n={len(by_mat[m])}")

    def run(name, fc, sel, target="d10"):
        rows = [c for c in train if sel(c)]
        f = {c["case_id"]: fc(c) for c in rows if fc(c) is not None}
        s = score(f, rows, target)
        print(f"  {name:58s} n={s['n']:3d}  aggregate {s['aggregate']:.4f}  day-equal {s['day_equal']:.4f}"
              + (f"  (no forecast: {len(s['missing'])})" if s["missing"] else ""))
        return s

    def lodo(group_fn, sel_pool=lambda c: True):
        """Leave-one-day-out constants: for a case, quantiles from the train cases of the other days."""
        def fc(c):
            pool = [ln(x) for x in scored if x["group_id"] != c["group_id"] and sel_pool(x) and group_fn(x) == group_fn(c)]
            return q_exp(pool) if len(pool) >= 2 else None
        return fc

    every = lambda c: True
    aged = lambda c: c["inputs"]["measurement"]["value"] == "aged"
    print("\nd10, CRPS of ln(d10), all train rows:")
    run("B0 constant, in-sample", lambda c: allq, every)
    run("B0 constant, leave-one-day-out", lodo(lambda c: 0), every)
    run("B1 per material, in-sample", lambda c: matq[mat(c)], every)
    run("B1 per material, leave-one-day-out", lodo(mat), every)
    print("d10, CRPS of ln(d10), aged rows (secondary aging) only:")
    run("B0 constant, in-sample", lambda c: allq, aged)
    run("B1 per material, in-sample", lambda c: matq[mat(c)], aged)
    run("P  persistence: the run's primary-row d10 (point)",
        lambda c: [c["prior"]["d10_m"]] * 5 if c["prior"] and c["prior"]["d10_m"] else None, aged)
    run("C  coagulation, Hinds K0 Cc, from the primary row (point)",
        lambda c: [coagulation_point(c)] * 5 if coagulation_point(c) else None, aged)

    # ISS detector at full scale: incidence baselines.
    iss = [c for c in train if c["outcome"]["scored"]["iss"]]
    y = lambda c: 1.0 if c["outcome"]["iss_full_scale"]["value"] else 0.0
    inc = sum(y(c) for c in iss) / len(iss)
    inc_m = defaultdict(list)
    for c in iss:
        inc_m[mat(c)].append(y(c))
    inc_m = {m: sum(v) / len(v) for m, v in inc_m.items()}
    print(f"\nISS scatter at full scale: incidence on train {inc:.3f} (n={len(iss)}); per material",
          {m: round(v, 3) for m, v in sorted(inc_m.items())})

    def lodo_inc(c, by_material):
        pool = [y(x) for x in iss if x["group_id"] != c["group_id"] and (not by_material or mat(x) == mat(c))]
        return sum(pool) / len(pool) if pool else None
    print("ISS full scale, Brier:")
    run("I0 incidence, in-sample", lambda c: inc, every, "iss")
    run("I0 incidence, leave-one-day-out", lambda c: lodo_inc(c, False), every, "iss")
    run("I1 per-material incidence, in-sample", lambda c: inc_m[mat(c)], every, "iss")
    run("I1 per-material incidence, leave-one-day-out", lambda c: lodo_inc(c, True), every, "iss")
    run("P  persistence (aged rows): primary row's state",
        lambda c: (1.0 if c["prior"]["iss_full_scale"] else 0.0) if c["prior"] and c["prior"]["iss_full_scale"] is not None else None,
        aged, "iss")
    run("I1 per-material incidence, in-sample, aged rows", lambda c: inc_m[mat(c)], aged, "iss")
    return 0


def main(argv):
    cmd = argv[1] if len(argv) > 1 else "build"
    if cmd == "build":
        build()
    elif cmd == "baselines":
        baselines()
    else:
        raise SystemExit(__doc__)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
