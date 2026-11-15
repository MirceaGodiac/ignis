#!/usr/bin/env python3
"""PROTOCOL 12.3 step 3 (attribution), normal flames: seven DIAGNOSTIC variants of F1's ethylene row
(runs/dev/gen6/fuel/diag/variants.json) scored on the Flame Design TRAIN DAYS ONLY, ethylene as itself,
with the chain of transfer_fuel.py (unchanged functions). Diagnostic only: nothing is fitted or searched.

Train-day isolation: the conditions and endpoints files are first copied to
fd_fuel/attribution/flame_design/ keeping only rows whose test day is a normal-flame TRAIN day (the
test name is the only field read to decide); every later step reads that copy.

Also: flow-stop time minus the script's stage sum for train-day tests (both families) that ran to the
scripted end.

    python tools/wsp/ext/fd_fuel_attribution.py prepare
    python tools/wsp/ext/fd_fuel_attribution.py score
    python tools/wsp/ext/fd_fuel_attribution.py stops
"""
import argparse
import csv
import json
import math
import statistics as st
import sys
from collections import Counter, defaultdict
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import transfer_fuel as tf  # noqa: E402  (layout of fd_fuel; sets g6/ta paths)
import transfer_gen6 as g6  # noqa: E402
import transfer_acme as ta  # noqa: E402
from fd_train_anatomy import qt  # noqa: E402

REPO = ta.REPO
SRC = tf.FD_WORK
ATT = tf.WORK / "attribution"
FDW = ATT / "flame_design"
TRAIN = {"acme_normal": ta.FD_DAYS["acme_normal"]["train_days"], "acme_inverse": ta.FD_DAYS["acme_inverse"]["train_days"]}
VARIANTS = json.loads((REPO / "runs/dev/gen6/fuel/diag/variants.json").read_text(encoding="utf-8"))
FAM = "acme_normal"
F1 = tf.ROLES[FAM]["f1"]
SURR = tf.ROLES[FAM]["surrogate"]
ROLES = dict(f1=F1, surrogate=SURR, **{k: v["release_id"] for k, v in VARIANTS.items()})
FUEL = {r: ("ethane" if r == "surrogate" else "recorded") for r in ROLES}
METRICS = ("extinction", "operator_termination", "sum")


def cmd_prepare(_args):
    FDW.mkdir(parents=True, exist_ok=True)
    keep = TRAIN[FAM] | TRAIN["acme_inverse"]
    n = Counter()
    for name in ("conditions.csv", "endpoints.csv"):
        with (SRC / name).open(encoding="utf-8", newline="") as f:
            rd = csv.DictReader(f)
            fields = rd.fieldnames
            rows = []
            for r in rd:
                if r["test"][:5] in keep:   # the only field read before the filter
                    rows.append(r)
        with (FDW / name).open("w", encoding="utf-8", newline="") as f:
            w = csv.DictWriter(f, fieldnames=fields)
            w.writeheader()
            w.writerows(rows)
        n[name] = len(rows)
    print(json.dumps(dict(wrote=FDW.relative_to(REPO).as_posix(), rows=dict(n))))
    return 0


def use_train_copy():
    ta.FD_WORK = FDW
    g6.FD_WORK = FDW
    tf.FD_WORK = FDW
    ta.OUT = ATT


def train_tests():
    use_train_copy()
    cpath, epath = FDW / "conditions.csv", FDW / "endpoints.csv"
    import hashlib
    sha_c, sha_e = (hashlib.sha256(p.read_bytes()).hexdigest() for p in (cpath, epath))
    with cpath.open(encoding="utf-8", newline="") as f:
        cond = [dict(r, row=i + 2) for i, r in enumerate(csv.DictReader(f)) if r["include_transfer"] == "1" and r["family"] == FAM]
    with epath.open(encoding="utf-8", newline="") as f:
        ends = {r["test"]: dict(r, erow=i + 2) for i, r in enumerate(csv.DictReader(f))}
    tests = []
    for r in cond:
        assert r["test"][:5] in TRAIN[FAM], r["test"]
        if r["fuel_species"] != "ethylene":
            continue
        e = ends[r["test"]]
        num = lambda s: float(s) if s not in ("", None) else float("nan")  # noqa: E731
        end, label = num(e["end_s"]), e["label_sensor"]
        assert e["status"] == "ok" and label in ta.CAUSES
        tests.append(dict(r, part="train_days", erow=e["erow"], programmed_stop=num(e["script_programmed_stop_s"]),
                          endpoint=dict(censoring="event", last_confirmed_active_s=end, first_terminal_s=end, allowed_causes=[label])))
    return tests, sha_c, sha_e


def run(tests, stop, sha_c, sha_e, workers, settings, tag):
    cases, jobs = [], []
    for t in tests:
        for c in ta.CUTOFFS:
            if not ta.at_risk(t["endpoint"], c):
                continue
            k = dict(test=t["test"], session=t["session"], cutoff=c, outcome=ta.outcome_of(t["endpoint"]),
                     o2=float(t["x_o2_role"]), refusal={}, art={})
            pp = {}
            for fuel in ("recorded", "ethane"):
                p = g6.prefix(t, c, fuel, stop, sha_c, sha_e)
                name = f"{t['test']}_c{c:g}_{fuel}_{'stop' if stop else 'nostop'}.json"
                pp[fuel] = ATT / "prefix" / name
                ta.write_json(pp[fuel], p)
            for role, rel in ROLES.items():
                jobs.append(((t["test"], c, role), rel, pp[FUEL[role]], ATT / f"artifact{tag}" / role / pp[FUEL[role]].name))
            cases.append(k)
    old = ta.SETTINGS
    ta.SETTINGS = settings
    try:
        arts = ta.forward_many(jobs, workers)
    finally:
        ta.SETTINGS = old
    for k in cases:
        for role in ROLES:
            a = arts[(k["test"], k["cutoff"], role)]
            k["art"][role] = a
            if not ta.admitted(a):
                k["refusal"][role] = "; ".join(a["support"].get("reasons", []))
    return cases


def contrib(sel, roles):
    outcomes = [k["outcome"] for k in sel]
    out = {}
    for r in roles:
        s = ta.score(outcomes, {c: [ta.cif_of(k["art"][r])[c] for k in sel] for c in ta.CAUSES})
        out[r] = {c: s[c]["contrib"] for c in ta.CAUSES}
        out[r]["sum"] = [a + b for a, b in zip(out[r]["extinction"], out[r]["operator_termination"])]
    return out


def centre(sel_one, dense, role):
    lr, w, inside = [], [], 0
    for k in sel_one:
        a = dense[(k["test"], k["cutoff"])]["art"][role]
        cats = a["event_categories"]
        e, o = a["event_cdf"][cats.index("extinction")], a["event_cdf"][cats.index("operator_termination")]
        tot = [x + y for x, y in zip(e, o)]
        t10, t50, t90 = qt(a["times_s"], tot, 0.1), qt(a["times_s"], tot, 0.5), qt(a["times_s"], tot, 0.9)
        te = k["outcome"]["t_upper"]
        if t50 is None:
            continue
        lr.append(math.log(t50 / te))
        if t10 and t90:
            w.append(math.log(t90 / t10))
            inside += t10 <= te <= t90
    q = st.quantiles(lr, n=4) if len(lr) >= 4 else [float("nan")] * 3
    return dict(n=len(sel_one), median=st.median(lr) if lr else None, q25=q[0], q75=q[2],
                width=st.median(w) if w else None, inside=inside)


def cmd_score(args):
    ta.use_binary(args)
    tests, sha_c, sha_e = train_tests()
    rep = dict(purpose="12.3 step 3, diagnostic variants of F1's ethylene row, Flame Design normal TRAIN days only",
               roles=ROLES, variants=VARIANTS, tests=len(tests), sessions=sorted({t["session"] for t in tests}), paths={})
    dense_settings = dict(ta.SETTINGS, times_s=tf.DENSE)
    for stop in (False, True):
        path = "stop" if stop else "nostop"
        cases = run(tests, stop, sha_c, sha_e, args.workers, ta.SETTINGS, "")
        dense = {(k["test"], k["cutoff"]): k for k in run(tests, stop, sha_c, sha_e, args.workers, dense_settings, "_dense")}
        refused = {r: sum(r in k["refusal"] for k in cases) for r in ROLES}
        ans = [k for k in cases if "f1" not in k["refusal"]]
        assert all(not k["refusal"] for k in ans), Counter(r for k in ans for r in k["refusal"])
        c = contrib(ans, ROLES)
        P = dict(cases=len(cases), answered=len(ans), tests_answered=len({k["test"] for k in ans}), refused=refused)
        P["score"] = {r: {x: math.fsum(c[r][x]) / len(ans) for x in METRICS} for r in ROLES}
        allc = contrib(cases, ("zqg", "surrogate"))
        P["score_all_cases"] = {r: {x: math.fsum(allc[r][x]) / len(cases) for x in METRICS} for r in ("zqg", "surrogate")}
        P["n_all_cases"] = len(cases)
        # per session, extinction, against F1
        ss = sorted({k["session"] for k in ans})
        per = {}
        for r in ROLES:
            row = {}
            for s in ss:
                idx = [i for i, k in enumerate(ans) if k["session"] == s]
                row[s] = (math.fsum(c[r]["extinction"][i] for i in idx) - math.fsum(c["f1"]["extinction"][i] for i in idx)) / len(idx)
            per[r] = dict(by_session=row, improve=sum(v < -1e-12 for v in row.values()), worsen=sum(v > 1e-12 for v in row.values()))
        P["per_session_extinction_minus_f1"] = per
        # centre and spread, cutoff 1 s, F1-answered tests
        one = [k for k in ans if k["cutoff"] == 1.0]
        groups = {
            "extinction, air": [k for k in one if k["outcome"]["causes"][0] == "extinction" and k["o2"] < 0.25],
            "extinction, 36-40 % O2": [k for k in one if k["outcome"]["causes"][0] == "extinction" and k["o2"] >= 0.33],
            "operator termination": [k for k in one if k["outcome"]["causes"][0] == "operator_termination"],
        }
        P["centre"] = {g: {r: centre(sel, dense, r) for r in ROLES} for g, sel in groups.items()}
        # held at 36-40 % O2: mean P(ext <= 120 s), all cutoffs
        held = [k for k in ans if k["o2"] >= 0.33 and k["outcome"]["causes"][0] == "operator_termination"]
        P["held_36_40"] = dict(cases=len(held), tests=len({k["test"] for k in held}),
                               mean_p_ext_120={r: st.fmean(ta.cif_of(k["art"][r])["extinction"][-1] for k in held) for r in ROLES})
        # zqg on the six tests F1 refuses
        ref = [k for k in cases if "f1" in k["refusal"]]
        if ref:
            rc = contrib(ref, ("zqg", "surrogate"))
            P["f1_refused_cases"] = dict(cases=len(ref), tests=len({k["test"] for k in ref}),
                                         score={r: {x: math.fsum(rc[r][x]) / len(ref) for x in METRICS} for r in ("zqg", "surrogate")},
                                         mean_p_ext_120={r: st.fmean(ta.cif_of(k["art"][r])["extinction"][-1] for k in ref) for r in ("zqg", "surrogate")})
        rep["paths"][path] = P
    ta.write_json(ATT / "report.json", rep)
    print(json.dumps(rep, indent=1)[:200], "... wrote", (ATT / "report.json").relative_to(REPO).as_posix())
    return 0


def cmd_stops(_args):
    """Train days, both families: flow-stop time minus the stage sum, tests that ran to the scripted end."""
    use_train_copy()
    rows = list(csv.DictReader((FDW / "endpoints.csv").open(encoding="utf-8", newline="")))
    with (FDW / "conditions.csv").open(encoding="utf-8", newline="") as f:
        fam = {r["test"]: r["family"] for r in csv.DictReader(f)}
    num = lambda s: float(s) if s not in ("", None) else float("nan")  # noqa: E731
    out = defaultdict(list)
    for r in rows:
        assert r["test"][:5] in TRAIN[fam[r["test"]]]
        stage, fc = num(r["script_programmed_stop_s"]), num(r["flow_change_s"])
        if not (r["label_sensor"] == "operator_termination" and r["tp_class"] == "scripted_end" and stage == stage and fc == fc):
            continue
        onset = num(r["onset_minus_stamp_s"])
        out[(fam[r["test"]], r["test"][:5])].append(dict(stage=stage, late=fc - stage, late_from_stamp=fc + onset - stage, onset=onset))
    rep = {}
    allv = []
    for k in sorted(out):
        v = out[k]
        allv += [(k[0], x) for x in v]
        late = [x["late"] for x in v]
        rep[f"{k[0]}/{k[1]}"] = dict(
            n=len(v), stage_sum_s=[min(x["stage"] for x in v), max(x["stage"] for x in v)],
            late_s_median=st.median(late), late_s_range=[min(late), max(late)],
            late_fraction_median=st.median(x["late"] / x["stage"] for x in v),
            late_from_ignition_stamp_s_median=st.median(x["late_from_stamp"] for x in v),
            onset_after_stamp_s_median=st.median(x["onset"] for x in v))
    # does lateness grow with the stage sum? least-squares line and rank correlation, per family (descriptive)
    for famname in ("acme_normal", "acme_inverse"):
        v = [x for f, x in allv if f == famname]
        if len(v) < 3:
            continue
        xs, ys = [x["stage"] for x in v], [x["late"] for x in v]
        mx, my = st.fmean(xs), st.fmean(ys)
        sxx = sum((a - mx) ** 2 for a in xs)
        slope = sum((a - mx) * (b - my) for a, b in zip(xs, ys)) / sxx if sxx else float("nan")
        rep[f"{famname}/all"] = dict(n=len(v), stage_sum_s=[min(xs), max(xs)], late_s_median=st.median(ys),
                                     late_s_quartiles=st.quantiles(ys, n=4), late_fraction_median=st.median(b / a for a, b in zip(xs, ys)),
                                     descriptive_line=dict(slope_s_per_s=slope, intercept_s=my - slope * mx),
                                     by_stage_sum={f"{s:g}": dict(n=sum(1 for a in xs if a == s),
                                                                 late_s_median=st.median([b for a, b in zip(xs, ys) if a == s]))
                                                   for s in sorted(set(xs))})
    ta.write_json(ATT / "stop_timing.json", rep)
    print(json.dumps(rep, indent=1))
    return 0


def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    for n, fn in (("prepare", cmd_prepare), ("score", cmd_score), ("stops", cmd_stops)):
        p = sub.add_parser(n)
        p.set_defaults(fn=fn)
        p.add_argument("--workers", type=int, default=12)
        p.add_argument("--ignis")
    a = ap.parse_args()
    return a.fn(a)


if __name__ == "__main__":
    sys.exit(main())
