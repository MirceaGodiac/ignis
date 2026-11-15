#!/usr/bin/env python3
"""Part (b) of the look registered in docs/wsp/PROTOCOL.md 12.6: the sixth-generation normal-flame
releases (`acme_normal.wsp7`: carry of P1, P2, P3, the baseline set) on the Flame Design (PSI-10)
transfer set of 11.1 as amended, without and with the planned stop. Normal flames only.

A thin wrapper. It computes nothing of its own: the endpoint measurement is
`tools/wsp/ext/flame_design.py` (thresholds and code unchanged) and the prefixes, forwards, IPCW Brier
scores and session bootstrap are the functions of `tools/wsp/ext/transfer_acme.py` (unchanged). What
this file sets:

  * where the data are read: the re-downloaded layout `runs/dev/data_all/raw/PSI-10/Raw Data/` and
    `.../Experimental table/` (file names unchanged), instead of `raw/samples/PSI-10/`;
  * where results are written: `runs/dev/data_all/work/transfer_acme/fd_gen6/` only;
  * the releases (four `acme_normal.wsp7` releases instead of the three of the fifth generation), and
    the comparisons (each candidate minus the carry, for extinction, operator termination and their
    sum, the forecast's proper score in 12.6);
  * the locator paths inside the prefixes, which name the files actually read (they do not reach the
    forward; the prefix hash changes, nothing it computes does);
  * the ignis binary: target/release (the fifth-generation copy in runs/dev/bin predates wsp7).

The date of a test, which committee2.rs reads from the prefix: `transfer_acme.fd_prefix` writes
run_id = `PSI-10:acme_normal:<test>` and session_id = `PSI-10:session:<day>`. The session id holds no
YYYY-MM-DD, so committee2 falls back on the run id: the first five characters of `<test>` are the
NASA day stamp YYDDD (19115A1 = day 115 of 2019). `score` checks, per forward, that the date the
release reports (`diagnostics.test_date`) is the date in the header of the test's sensor file.

    python tools/wsp/ext/transfer_gen6.py chain          # step 1: CFI-G train vs wsp-dev feedback
    python tools/wsp/ext/transfer_gen6.py conditions     # flame_design.py conditions, new layout
    python tools/wsp/ext/transfer_gen6.py tests-file     # normal-flame tests outside the test days
    python tools/wsp/ext/transfer_gen6.py endpoints      # flame_design.py endpoints on those tests
    python tools/wsp/ext/transfer_gen6.py score          # arms A and B, both paths
    python tools/wsp/ext/transfer_gen6.py tables         # aggregates as text
"""
import argparse
import csv
import hashlib
import json
import math
import re
import sys
from collections import Counter
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import flame_design as fdm  # noqa: E402
import transfer_acme as ta  # noqa: E402

REPO = ta.REPO
DATA = REPO / "runs/dev/data_all"
RAW = DATA / "raw/PSI-10"
GEN6 = DATA / "work/transfer_acme/fd_gen6"
FD_WORK = GEN6 / "flame_design"

# --- layout: only where things are read and written --------------------------------------------
fdm.FILES = RAW / "Raw Data"
fdm.TABLES = {fam: str(Path("..") / "Experimental table" / name) for fam, name in fdm.TABLES.items()}
fdm.OUT = FD_WORK
ta.OUT = GEN6
ta.FD_WORK = FD_WORK
ta.BASELINE = GEN6 / "incidence_baseline_cfig_train.json"

FAMILY = "acme_normal"
ROLES = {
    "carry": "sha256:bf18aff473e32413d25255fc61764f3ecd8f3fd052387b49487e993cbd0cfacb",
    "p2": "sha256:ceffbb0dd674fe4b073b728cc30b50c53ca4d8baaea8a5fbc4837742de01b85c",
    "p3": "sha256:51f4bbc01faf7ccf2c9070b8f62c213aaab442c195113a70919023bc0cc83de9",
    "baseline_set": "sha256:dd05f4f8b6157cdbbccc0f9b0168284f6425df77c01e1a16db5c79cae406a513",
}
ta.RELEASES = {FAMILY: ROLES}
# PROTOCOL 12.6, train, shared records (no planned stop), extinction / operator termination
REGISTERED = {"carry": (0.0518, 0.0274), "p2": (0.0526, 0.0281), "p3": (0.0522, 0.0240)}
# PROTOCOL 12.6, train, with the planned stop (supplementary check of the stop path)
REGISTERED_STOP = {"carry": (0.0536, 0.0215), "p2": (0.0542, 0.0166), "p3": (0.0538, 0.0140)}
TOL4 = 0.00005  # the registered numbers are printed to four decimals
CAUSES = ta.CAUSES
METRICS = ("extinction", "operator_termination", "sum")
# 11.3 (fifth generation, P1 = acme_normal.wsp5 P1), for the carry's comparison
P1_14NOV = {"B/nostop": (0.1203, 0.3166), "B/stop": (0.0823, 0.0887),
            "A_propane/nostop": (0.0522, 0.5007), "A_propane/stop": (0.0081, 0.0980)}


def write_json(path, obj):
    ta.write_json(path, obj)


# ---------------------------------------------------------------------------------------------
# step 1: the chain on CFI-G train
# ---------------------------------------------------------------------------------------------
def cmd_chain(args):
    ta.selftest()
    binary = ta.use_binary(args)
    cases, not_at_risk, runs, with_ctrl = ta.cfig_cases(FAMILY)
    pdir, adir = GEN6 / "cfig/prefix", GEN6 / "cfig/artifact"
    jobs = []
    for k in cases:
        name = f"{k['test']}_c{k['cutoff']:g}.json"
        k["prefix_path"] = pdir / name
        write_json(k["prefix_path"], k["prefix"])
        for role, rel in ROLES.items():
            jobs.append(((k["run_id"], k["cutoff"], role), rel, k["prefix_path"], adir / role / name))
    arts = ta.forward_many(jobs, args.workers)
    ref_path = GEN6 / "cfig/wspdev_fb_train_normal.json"
    ref = json.loads(ref_path.read_text(encoding="utf-8"))
    ok = [ref["runs"], len(ref["releases"][0]["cases"]), ref["not_at_risk_cases"]] == [runs, len(cases), not_at_risk]
    rep = dict(purpose="PROTOCOL 12.6 (b), step 1: the scoring chain on CFI-G train, no planned stop", ignis=binary,
               wsp_dev_report=ref_path.relative_to(REPO).as_posix(), runs=runs, cases=len(cases),
               not_at_risk=not_at_risk, runs_with_control_history=with_ctrl, releases={})
    outcomes = [k["outcome"] for k in cases]
    for role, rel in ROLES.items():
        a = [arts[(k["run_id"], k["cutoff"], role)] for k in cases]
        assert all(ta.admitted(x) for x in a), f"{role}: a train case was refused"
        s = ta.score(outcomes, {c: [ta.cif_of(x)[c] for x in a] for c in CAUSES})
        rr = next(r for r in ref["releases"] if r["release_id"] == rel)
        ev = rr["aggregate"]["events"]["causes"]
        dev = {(c["run_id"], c["cutoff_s"]): c for c in rr["cases"]}
        d_end, d_ep = 0.0, 0
        for k, x in zip(cases, a):
            dc = dev[(k["run_id"], k["cutoff"])]
            d_end = max(d_end, max(abs(p - q[-1]) for p, q in zip(dc["events_at_end"], x["event_cdf"])))
            e = dc["endpoint"]
            d_ep += not (e["censoring"] == k["outcome"]["censoring"] and e["t_lower_s"] == k["outcome"]["t_lower"]
                         and e["t_upper_s"] == k["outcome"]["t_upper"] and e["allowed_causes"] == k["outcome"]["causes"])
        row = dict(release=rel, chain={c: s[c]["integrated_brier"] for c in CAUSES},
                   wsp_dev={c: ev[c]["integrated_brier"] for c in CAUSES},
                   abs_diff={c: abs(ev[c]["integrated_brier"] - s[c]["integrated_brier"]) for c in CAUSES},
                   max_abs_diff_brier_per_horizon={c: max(abs(p["brier"] - q["brier"]) for p, q in zip(ev[c]["brier"], s[c]["brier"]))
                                                   for c in CAUSES},
                   per_case_max_abs_diff_events_at_end=d_end, per_case_endpoint_mismatches=d_ep)
        ok &= all(v <= 1e-12 for v in row["abs_diff"].values()) and d_ep == 0 and d_end <= 1e-12
        if role in REGISTERED:
            row["registered"] = dict(zip(CAUSES, REGISTERED[role]))
            row["abs_diff_to_registered"] = {c: abs(s[c]["integrated_brier"] - r) for c, r in zip(CAUSES, REGISTERED[role])}
            ok &= all(v <= TOL4 for v in row["abs_diff_to_registered"].values())
        rep["releases"][role] = row
    rep["reproduced"] = bool(ok)
    base = ta.baseline_from({FAMILY: cases})
    if ta.BASELINE.exists():
        rep["baseline"] = dict(state="already frozen; not rewritten",
                               recomputed_equal=json.loads(ta.BASELINE.read_text(encoding="utf-8")) == base)
    elif ok:
        write_json(ta.BASELINE, base)
        rep["baseline"] = dict(state="frozen now")
    if ta.BASELINE.exists():
        rep["baseline"].update(file=ta.BASELINE.relative_to(REPO).as_posix(),
                               sha256=hashlib.sha256(ta.BASELINE.read_bytes()).hexdigest())
    # supplementary: the planned-stop path on train, the stage sum of the 'as run' file as programmed_stop
    if args.with_stop:
        import acme_script
        files = DATA / "raw/PSI-159/Raw Data"
        sjobs, sk = [], []
        for k in cases:
            f = files / f"PSI-159_Raw Data_as run{k['test']}.txt"
            v = acme_script.parse(f) if f.exists() else None
            p = json.loads(json.dumps(k["prefix"]))
            if v is not None and v > 0:
                p["known_conditions"]["programmed_stop"] = ta.measured(
                    float(v), "s", "stage durations", None, hashlib.sha256(f.read_bytes()).hexdigest(),
                    "wsp.ext.acme.cond.programmed_stop", "sum of the script's stage durations (acme_script.parse)",
                    path=f.name)
            p = ta.finish_prefix(p)
            name = f"{k['test']}_c{k['cutoff']:g}_stop.json"
            write_json(pdir / name, p)
            sk.append(dict(k, carries_stop="programmed_stop" in p["known_conditions"]))
            for role, rel in ROLES.items():
                sjobs.append(((k["run_id"], k["cutoff"], role), rel, pdir / name, adir / role / name))
        sarts = ta.forward_many(sjobs, args.workers)
        st = dict(cases=len(sk), carrying_stop=sum(k["carries_stop"] for k in sk), releases={})
        for role in ROLES:
            a = [sarts[(k["run_id"], k["cutoff"], role)] for k in sk]
            s = ta.score(outcomes, {c: [ta.cif_of(x)[c] for x in a] for c in CAUSES})
            st["releases"][role] = dict(chain={c: s[c]["integrated_brier"] for c in CAUSES},
                                        registered=dict(zip(CAUSES, REGISTERED_STOP[role])) if role in REGISTERED_STOP else None)
        rep["supplementary_with_planned_stop"] = st
    write_json(GEN6 / "cfig/chain_check.json", rep)
    print(json.dumps(rep, indent=1))
    return 0 if ok else 2


# ---------------------------------------------------------------------------------------------
# step 2: endpoints by flame_design.py (unchanged), new layout
# ---------------------------------------------------------------------------------------------
def cmd_conditions(_args):
    return fdm.cmd_conditions(None)


def check_days():
    """ta.FD_DAYS must be the frozen split file."""
    s = json.loads((REPO / "docs/wsp/atlas/splits/PSI-10.json").read_text(encoding="utf-8"))
    g = s["strata"]["normal_flames"]["groups"]
    days = {f"{p}_days": {x.split(":")[-1] for x in g[p]} for p in ("train", "validation", "test")}
    assert days == ta.FD_DAYS[FAMILY], (days, ta.FD_DAYS[FAMILY])


def cmd_tests_file(_args):
    check_days()
    with (FD_WORK / "conditions.csv").open(encoding="utf-8", newline="") as f:
        rows = [r for r in csv.DictReader(f) if r["include_transfer"] == "1" and r["family"] == FAMILY]
    keep = [r["test"] for r in rows if ta.fd_part(FAMILY, r["test"]) != "test_days"]
    path = GEN6 / "tests_remaining_normal.txt"
    path.write_text("\n".join(keep) + "\n", encoding="utf-8", newline="\n")
    print(json.dumps(dict(wrote=path.relative_to(REPO).as_posix(), included_normal=len(rows),
                          on_test_days_dropped=len(rows) - len(keep), remaining=len(keep),
                          sessions_remaining=len({t[:5] for t in keep}))))
    return 0


def cmd_endpoints(_args):
    a = argparse.Namespace(tests=None, tests_file=str(GEN6 / "tests_remaining_normal.txt"), included=False,
                           out="endpoints.csv")
    return fdm.cmd_endpoints(a)


# ---------------------------------------------------------------------------------------------
# step 3 and 4: forwards and scores
# ---------------------------------------------------------------------------------------------
def prefix(t, cutoff, fuel, stop, sha_c, sha_e):
    """transfer_acme.fd_prefix, with the locator paths naming the files of this run."""
    p = ta.fd_prefix(t, cutoff, fuel, stop, sha_c, sha_e)
    for key, m in p["known_conditions"].items():
        m["locator"]["path"] = (FD_WORK / ("endpoints.csv" if key == "programmed_stop" else "conditions.csv")
                                ).relative_to(REPO).as_posix()
    return ta.finish_prefix(p)


def run_block(tag, tests, fuel, stop, sha_c, sha_e, workers, header_dates):
    pdir = GEN6 / "prefix"
    cases, jobs = [], []
    for t in tests:
        for c in ta.CUTOFFS:
            if not ta.at_risk(t["endpoint"], c):
                continue
            pname = f"{t['test']}_c{c:g}_{fuel}_{'stop' if stop else 'nostop'}.json"
            p = prefix(t, c, fuel, stop, sha_c, sha_e)
            write_json(pdir / pname, p)
            cases.append(dict(test=t["test"], session=t["session"], part=t["part"], cutoff=c,
                              outcome=ta.outcome_of(t["endpoint"]), carries_stop="programmed_stop" in p["known_conditions"],
                              block=tag))
            for role, rel in ROLES.items():
                jobs.append(((t["test"], c, role), rel, pdir / pname, GEN6 / "artifact" / role / pname))
    arts = ta.forward_many(jobs, workers)
    dates = Counter()
    for k in cases:
        a = {role: arts[(k["test"], k["cutoff"], role)] for role in ROLES}
        k["refusal"] = {role: f"{x['support'].get('gate')}: {'; '.join(x['support'].get('reasons', []))}"
                        for role, x in a.items() if not ta.admitted(x)}
        if not k["refusal"]:
            k["cif"] = {role: ta.cif_of(x) for role, x in a.items()}
        for role, x in a.items():
            d = (x.get("diagnostics") or {})
            got, src = d.get("test_date"), d.get("test_date_source")
            want = header_dates.get(k["test"])
            dates[f"{src}:{'equal' if got == want else ('no date' if got is None else 'different')}"] += 1
            k.setdefault("p_late", {})[role] = d.get("p_late")
    return cases, dict(dates)


def score_set(sel, baseline, cases_file=None):
    out = dict(cases_at_risk=len(sel), tests_with_a_case=len({k["test"] for k in sel}),
               sessions_with_a_case=len({k["session"] for k in sel}),
               cases_by_cutoff={f"{c:g}": sum(1 for k in sel if k["cutoff"] == c) for c in ta.CUTOFFS},
               cases_carrying_programmed_stop=sum(k["carries_stop"] for k in sel), refused={})
    for role in ROLES:
        reasons = Counter(re.sub(r"-?\d+(\.\d+)?", "#", k["refusal"][role]) for k in sel if role in k["refusal"])
        out["refused"][role] = dict(cases=sum(reasons.values()), tests=len({k["test"] for k in sel if role in k["refusal"]}),
                                    reasons=dict(reasons))
    ans = [k for k in sel if not k["refusal"]]
    out.update(cases_scored=len(ans), tests_scored=len({k["test"] for k in ans}), sessions_scored=len({k["session"] for k in ans}))
    if not ans:
        return out
    outcomes = [k["outcome"] for k in ans]
    out["endpoints_of_scored_cases"] = dict(Counter(o["causes"][0] if o["causes"] else "right_censored" for o in outcomes))
    cifs = {role: {c: [k["cif"][role][c] for k in ans] for c in CAUSES} for role in ROLES}
    cifs["incidence_baseline"] = {c: [baseline[f"{k['cutoff']:g}"][c] for k in ans] for c in CAUSES}
    sc = {m: ta.score(outcomes, cifs[m]) for m in cifs}
    contrib = {m: {c: sc[m][c]["contrib"] for c in CAUSES} for m in sc}
    for m in contrib:
        contrib[m]["sum"] = [a + b for a, b in zip(contrib[m]["extinction"], contrib[m]["operator_termination"])]
    sessions = [k["session"] for k in ans]
    out["integrated_brier"] = {m: dict({c: sc[m][c]["integrated_brier"] for c in CAUSES},
                                       sum=sc[m]["extinction"]["integrated_brier"] + sc[m]["operator_termination"]["integrated_brier"])
                               for m in sc}
    def means(v):
        by = {}
        for x, s in zip(v, sessions):
            by.setdefault(s, []).append(x)
        return dict(case_mean=math.fsum(v) / len(v), session_equal_mean=math.fsum(math.fsum(b) / len(b) for b in by.values()) / len(by),
                    n_cases=len(v), n_sessions=len(by))
    out["score"] = {m: {x: means(contrib[m][x]) for x in METRICS} for m in sc}
    out["paired"] = {f"{m}_minus_carry": {x: ta.boot([a - b for a, b in zip(contrib[m][x], contrib["carry"][x])], sessions)
                                          for x in METRICS} for m in ("p2", "p3", "baseline_set", "incidence_baseline")}
    out["p_late_by_role"] = {role: dict(Counter(str(k["p_late"][role]) for k in ans)) for role in ROLES}
    if cases_file:  # PER-TEST OUTCOMES: local only, never quoted
        cases_file.parent.mkdir(parents=True, exist_ok=True)
        with cases_file.open("w", encoding="utf-8", newline="") as f:
            w = csv.writer(f)
            w.writerow(["test", "session", "days", "block", "cutoff_s", "end_s", "cause", "carries_programmed_stop"]
                       + [f"ibs_contrib[{x}]:{m}" for m in contrib for x in METRICS])
            for i, k in enumerate(ans):
                o = k["outcome"]
                w.writerow([k["test"], k["session"], k["part"], k["block"], f"{k['cutoff']:g}", ta._or(o["t_upper"], o["t_lower"]),
                            o["causes"][0] if o["causes"] else "", int(k["carries_stop"])]
                           + [f"{contrib[m][x][i]:.6f}" for m in contrib for x in METRICS])
        out["cases_file"] = cases_file.relative_to(REPO).as_posix()
    return out


def with_days(cases, baseline, cases_file):
    b = score_set(cases, baseline, cases_file)
    b["by_days"] = {part: score_set([k for k in cases if k["part"] == part], baseline) for part in ("train_days", "validation_days")}
    return b


def cmd_score(args):
    ta.selftest()
    check_days()
    binary = ta.use_binary(args)
    chain = json.loads((GEN6 / "cfig/chain_check.json").read_text(encoding="utf-8"))
    if not (chain.get("reproduced") and chain["ignis"]["sha256"] == binary["sha256"]):
        sys.exit("the chain check did not reproduce with this binary: nothing is run on Flame Design")
    baseline = json.loads(ta.BASELINE.read_text(encoding="utf-8"))["families"][FAMILY]["cutoffs"]
    tests, sha_c, sha_e, dropped = ta.fd_tests()
    tests = [t for t in tests if t["family"] == FAMILY]
    assert not any(t["part"] == "test_days" for t in tests)
    with (FD_WORK / "endpoints.csv").open(encoding="utf-8", newline="") as f:
        header_dates = {r["test"]: r["date"] for r in csv.DictReader(f)}
    # the date rule of committee2 against the sensor-file header, per test (counts only)
    def ydoy(test):
        import datetime
        d = datetime.date(2000 + int(test[:2]), 1, 1) + datetime.timedelta(days=int(test[2:5]) - 1)
        return d.isoformat()
    run_id_vs_header = Counter("equal" if ydoy(t["test"]) == header_dates.get(t["test"]) else "different" for t in tests)
    rep = dict(purpose="PROTOCOL 12.6 part (b): Flame Design transfer set of 11.1 as amended, normal flames",
               ignis=binary, chain_check=chain["releases"], releases=ROLES, horizons_s=ta.HORIZONS, cutoffs_s=ta.CUTOFFS,
               baseline=dict(file=ta.BASELINE.relative_to(REPO).as_posix(), sha256=hashlib.sha256(ta.BASELINE.read_bytes()).hexdigest()),
               inputs=dict(conditions_csv_sha256=sha_c, endpoints_csv_sha256=sha_e),
               bootstrap=dict(resamples=ta.N_BOOT, seed=ta.BOOT_SEED, unit="session", level=0.95),
               dropped_test_days=dropped[FAMILY], run_id_date_vs_sensor_header=dict(run_id_vs_header),
               tests=dict(included=len(tests), sessions=len({t["session"] for t in tests}),
                          by_fuel=dict(Counter(t["fuel_species"] for t in tests)),
                          with_endpoint=sum(t["endpoint"] is not None for t in tests),
                          without_endpoint=dict(Counter(t["why"] for t in tests if t["endpoint"] is None)),
                          with_programmed_stop=sum(1 for t in tests if t["programmed_stop"] == t["programmed_stop"] and t["programmed_stop"] > 0),
                          by_days={p: dict(tests=sum(1 for t in tests if t["part"] == p),
                                           sessions=len({t["session"] for t in tests if t["part"] == p}))
                                   for p in ("train_days", "validation_days")}),
               blocks={}, date_diagnostics={})
    prop = [t for t in tests if t["include_unchanged"] == "1"]
    eth = [t for t in tests if t["fuel_species"] == "ethylene"]
    for stop in (False, True):
        path = "stop" if stop else "nostop"
        ca, da = run_block("A_propane", prop, "recorded", stop, sha_c, sha_e, args.workers, header_dates)
        ce, de = run_block("A_ethylene", eth, "recorded", stop, sha_c, sha_e, args.workers, header_dates)
        cb, db = run_block("B", eth, "ethane", stop, sha_c, sha_e, args.workers, header_dates)
        rep["date_diagnostics"][path] = dict(A_propane=da, A_ethylene=de, B=db)
        rep["blocks"][f"A_propane/{path}"] = with_days(ca, baseline, GEN6 / "cases" / f"A_propane_{path}.csv")
        rep["blocks"][f"A_ethylene_refusals/{path}"] = with_days(ce, baseline, None)
        rep["blocks"][f"B/{path}"] = with_days(cb, baseline, GEN6 / "cases" / f"B_{path}.csv")
        rep["blocks"][f"decision_set/{path}"] = with_days(cb + ca, baseline, GEN6 / "cases" / f"decision_{path}.csv")
    write_json(GEN6 / "report.json", rep)
    print(f"wrote {(GEN6 / 'report.json').relative_to(REPO).as_posix()}")
    return 0


def cmd_tables(_args):
    rep = json.loads((GEN6 / "report.json").read_text(encoding="utf-8"))
    f4 = lambda x: "n/a" if x is None or x != x else f"{x:.4f}"  # noqa: E731
    ci = lambda c: "(one session)" if c is None else f"[{c[0]:+.4f}, {c[1]:+.4f}]"  # noqa: E731
    print("tests:", json.dumps(rep["tests"]))
    print("dropped:", rep["dropped_test_days"], "| run-id date vs header:", rep["run_id_date_vs_sensor_header"])
    print("date diagnostics:", json.dumps(rep["date_diagnostics"]))

    def part(label, b):
        print(f"  -- {label}: cases at risk {b['cases_at_risk']} ({b['cases_by_cutoff']}), scored {b['cases_scored']} of "
              f"{b['tests_scored']} tests in {b['sessions_scored']} sessions; carrying stop {b['cases_carrying_programmed_stop']}")
        for role, r in b["refused"].items():
            if r["cases"]:
                print(f"     refused by {role}: {r['cases']} cases / {r['tests']} tests: {r['reasons']}")
        if "score" not in b:
            return
        print(f"     endpoints {b['endpoints_of_scored_cases']}; p_late of P2/P3 {b['p_late_by_role']['p2']} / {b['p_late_by_role']['p3']}")
        print("     model               ext(case)  term(case)  sum(case) | ext(sess)  term(sess)  sum(sess)")
        for m, v in b["score"].items():
            print(f"     {m:<19} {f4(v['extinction']['case_mean'])}     {f4(v['operator_termination']['case_mean'])}      "
                  f"{f4(v['sum']['case_mean'])}    | {f4(v['extinction']['session_equal_mean'])}     "
                  f"{f4(v['operator_termination']['session_equal_mean'])}      {f4(v['sum']['session_equal_mean'])}")
            ib = b["integrated_brier"][m]
            assert all(abs(ib[x] - v[x]["case_mean"]) < 1e-12 for x in METRICS), (m, ib, v)
        for name, v in b["paired"].items():
            for x in METRICS:
                p = v[x]
                print(f"     {name} [{x}]: {p['case_mean']:+.4f} {ci(p['ci95_case_mean'])}; session-equal "
                      f"{p['session_equal_mean']:+.4f} {ci(p['ci95_session_equal_mean'])}")
    for key, b in rep["blocks"].items():
        print(f"\n== {key}")
        part("all remaining", b)
        for label, v in b["by_days"].items():
            part(label, v)
    return 0


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    sub = ap.add_subparsers(dest="cmd", required=True)
    p = sub.add_parser("chain")
    p.add_argument("--with-stop", action="store_true", help="also the supplementary planned-stop path on train")
    p.set_defaults(fn=cmd_chain)
    for name, fn in (("conditions", cmd_conditions), ("tests-file", cmd_tests_file), ("endpoints", cmd_endpoints),
                     ("score", cmd_score), ("tables", cmd_tables)):
        sub.add_parser(name).set_defaults(fn=fn)
    for q in sub.choices.values():
        q.add_argument("--workers", type=int, default=12)
        q.add_argument("--ignis", help="ignis binary relative to the repository (default target/release/ignis.exe)")
    args = ap.parse_args()
    return args.fn(args)


if __name__ == "__main__":
    sys.exit(main())
