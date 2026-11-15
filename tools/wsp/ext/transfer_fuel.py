#!/usr/bin/env python3
"""PROTOCOL 12.8: the fuel-table releases F1 (`acme_*.wsp12`) on the ethylene tests of Flame Design
(PSI-10), "as itself" (fuel_species = ethylene, no indicators) against "as ethane" (the surrogate of
11.1 with ethane's indicators) on identical cases, without and with the planned stop. Plus the train-day
case file for the study step of 12.3.

A thin wrapper over the unchanged chain: `flame_design.py` (endpoints, thresholds unchanged) and
`transfer_acme.py` (prefixes, forwards, IPCW Brier, session bootstrap), with the data layout of
`transfer_gen6.py`. It writes only under runs/dev/data_all/work/transfer_acme/fd_fuel/.

Reused: the normal-flame conditions and endpoints of fd_gen6 (copied, byte for byte). Measured here,
the same way: the inverse-flame endpoints of the included tests outside the test day 21361.

    python tools/wsp/ext/transfer_fuel.py chain       # CFI-G train, both families, no planned stop
    python tools/wsp/ext/transfer_fuel.py endpoints   # inverse endpoints; merged endpoints.csv
    python tools/wsp/ext/transfer_fuel.py score       # Flame Design ethylene, both families and paths
    python tools/wsp/ext/transfer_fuel.py tables
    python tools/wsp/ext/transfer_fuel.py train-days  # train-day case file (dense forecast grid added)
"""
import argparse
import csv
import hashlib
import json
import math
import shutil
import sys
from collections import Counter
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import transfer_gen6 as g6  # noqa: E402  (sets the read layout of flame_design and transfer_acme)
import flame_design as fdm  # noqa: E402
import transfer_acme as ta  # noqa: E402

REPO = ta.REPO
WORK = REPO / "runs/dev/data_all/work/transfer_acme/fd_fuel"
FD_WORK = WORK / "flame_design"
SRC_G6 = g6.FD_WORK          # the fd_gen6 conditions and normal endpoints, copied
g6.FD_WORK = FD_WORK          # prefix locators name the files of this run
fdm.OUT = FD_WORK
ta.OUT = WORK
ta.FD_WORK = FD_WORK
ta.BASELINE = WORK / "incidence_baseline_cfig_train.json"

ROLES = {
    "acme_normal": {"f1": "sha256:131060f2b1389f70e27c171b2996b584e7f5cfb7e64aabb450b97069e957ffdf",
                    "surrogate": "sha256:96a91f15ab94ff6a0b32e51f34e5c1f71d8377ae618b4cdd331bdb1b20fdaa34"},
    "acme_inverse": {"f1": "sha256:d59251cfba37d46cf264dd706f1df58bb59be8e4ef8964335944dd6c372c3b42",
                     "surrogate": "sha256:d7292e8974eed0f0ade50df68e14e3b9134c5c41ec3b8ddd664e02db7561fcc2"},
}
FUEL = {"f1": "recorded", "surrogate": "ethane"}   # how each role gets the fuel
REGISTERED = {"acme_normal": (0.052162, 0.023997, 210), "acme_inverse": (0.027091, 0.060816, 204)}
CAUSES = ta.CAUSES
METRICS = ("extinction", "operator_termination", "sum")
PARTS = ("train_days", "validation_days")


def wj(p, o):
    ta.write_json(p, o)


# ---------------------------------------------------------------------------------------------
def cmd_chain(args):
    ta.selftest()
    binary = ta.use_binary(args)
    rep = dict(purpose="PROTOCOL 12.8 chain check: CFI-G train, no planned stop", ignis=binary, families={})
    ok, cases_by_family = True, {}
    for fam, roles in ROLES.items():
        cases, nar, runs, _ = ta.cfig_cases(fam)
        cases_by_family[fam] = cases
        jobs = []
        for k in cases:
            name = f"{k['test']}_c{k['cutoff']:g}.json"
            pp = WORK / "cfig/prefix" / fam / name
            wj(pp, k["prefix"])
            for role, rel in roles.items():
                jobs.append(((k["run_id"], k["cutoff"], role), rel, pp, WORK / "cfig/artifact" / fam / role / name))
        arts = ta.forward_many(jobs, args.workers)
        ref = json.loads((WORK / f"cfig/wspdev_fb_train_{fam.split('_')[1]}.json").read_text(encoding="utf-8"))
        outcomes = [k["outcome"] for k in cases]
        fr = dict(runs=runs, cases=len(cases), not_at_risk=nar,
                  wsp_dev_counts=[ref["runs"], len(ref["releases"][0]["cases"]), ref["not_at_risk_cases"]], releases={})
        ok &= fr["wsp_dev_counts"] == [runs, len(cases), nar]
        tables = {}
        for role, rel in roles.items():
            a = [arts[(k["run_id"], k["cutoff"], role)] for k in cases]
            assert all(ta.admitted(x) for x in a), (fam, role)
            tables[role] = [x["event_cdf"] for x in a]
            s = ta.score(outcomes, {c: [ta.cif_of(x)[c] for x in a] for c in CAUSES})
            rr = next(r for r in ref["releases"] if r["release_id"] == rel)
            ev = rr["aggregate"]["events"]["causes"]
            row = dict(release=rel, chain={c: s[c]["integrated_brier"] for c in CAUSES},
                       wsp_dev={c: ev[c]["integrated_brier"] for c in CAUSES},
                       abs_diff={c: abs(ev[c]["integrated_brier"] - s[c]["integrated_brier"]) for c in CAUSES})
            ok &= all(v <= 1e-12 for v in row["abs_diff"].values())
            if role == "f1":
                reg = REGISTERED[fam]
                row["registered"] = reg
                ok &= len(cases) == reg[2] and all(abs(s[c]["integrated_brier"] - r) <= 5e-7 for c, r in zip(CAUSES, reg))
            fr["releases"][role] = row
        fr["f1_tables_equal_surrogate_bit_for_bit"] = sum(a == b for a, b in zip(tables["f1"], tables["surrogate"]))
        rep["families"][fam] = fr
    rep["reproduced"] = bool(ok)
    base = ta.baseline_from(cases_by_family)
    if not ta.BASELINE.exists() and ok:
        wj(ta.BASELINE, base)
    if ta.BASELINE.exists():
        frozen = json.loads(ta.BASELINE.read_text(encoding="utf-8"))
        g6b = json.loads((g6.GEN6 / "incidence_baseline_cfig_train.json").read_text(encoding="utf-8"))
        rep["baseline"] = dict(file=ta.BASELINE.relative_to(REPO).as_posix(), recomputed_equal=frozen == base,
                               normal_equals_gen6=frozen["families"]["acme_normal"] == g6b["families"]["acme_normal"],
                               sha256=hashlib.sha256(ta.BASELINE.read_bytes()).hexdigest())
    wj(WORK / "cfig/chain_check.json", rep)
    print(json.dumps(rep, indent=1))
    return 0 if ok else 2


# ---------------------------------------------------------------------------------------------
def cmd_endpoints(_args):
    g6.check_days()
    FD_WORK.mkdir(parents=True, exist_ok=True)
    src = SRC_G6
    shutil.copyfile(src / "conditions.csv", FD_WORK / "conditions.csv")
    with (FD_WORK / "conditions.csv").open(encoding="utf-8", newline="") as f:
        inv = [r["test"] for r in csv.DictReader(f) if r["include_transfer"] == "1" and r["family"] == "acme_inverse"]
    keep = [t for t in inv if ta.fd_part("acme_inverse", t) != "test_days"]
    (WORK / "tests_remaining_inverse.txt").write_text("\n".join(keep) + "\n", encoding="utf-8", newline="\n")
    print(json.dumps(dict(inverse_included=len(inv), on_test_day_dropped=len(inv) - len(keep), remaining=len(keep))))
    fdm.cmd_endpoints(argparse.Namespace(tests=None, tests_file=str(WORK / "tests_remaining_inverse.txt"),
                                         included=False, out="endpoints_inverse.csv"))
    # merged endpoints.csv: the normal rows of fd_gen6 as they are, then the inverse rows
    with (src / "endpoints.csv").open(encoding="utf-8", newline="") as f:
        rn = list(csv.DictReader(f))
    with (FD_WORK / "endpoints_inverse.csv").open(encoding="utf-8", newline="") as f:
        ri = list(csv.DictReader(f))
    with (FD_WORK / "endpoints.csv").open("w", encoding="utf-8", newline="") as f:
        w = csv.DictWriter(f, fieldnames=fdm.FIELDS)
        w.writeheader()
        w.writerows(rn + ri)
    print(json.dumps(dict(merged_rows=len(rn) + len(ri), normal=len(rn), inverse=len(ri))))
    return 0


# ---------------------------------------------------------------------------------------------
def forwards(fam, tests, stop, sha_c, sha_e, workers, settings=None, tag=""):
    """Cases (identical for both roles) and the artifacts of F1 (fuel as recorded) and the surrogate."""
    cases, jobs = [], []
    for t in tests:
        for c in ta.CUTOFFS:
            if not ta.at_risk(t["endpoint"], c):
                continue
            k = dict(test=t["test"], session=t["session"], part=t["part"], cutoff=c, outcome=ta.outcome_of(t["endpoint"]),
                     family=fam, prefixes={})
            for role, rel in ROLES[fam].items():
                fuel = FUEL[role]
                pname = f"{t['test']}_c{c:g}_{fuel}_{'stop' if stop else 'nostop'}.json"
                p = g6.prefix(t, c, fuel, stop, sha_c, sha_e)
                if role == "f1":
                    assert "fuel_is_ethane" not in p["known_conditions"] and "fuel_is_butane" not in p["known_conditions"]
                pp = WORK / "prefix" / fam / pname
                wj(pp, p)
                k["prefixes"][role] = p
                jobs.append(((t["test"], c, role), rel, pp, WORK / f"artifact{tag}" / fam / role / pname))
            k["carries_stop"] = "programmed_stop" in k["prefixes"]["f1"]["known_conditions"]
            cases.append(k)
    if settings is None:
        arts = ta.forward_many(jobs, workers)
    else:  # another time grid (train-day study only): same prefixes, other settings
        old = ta.SETTINGS
        ta.SETTINGS = settings
        try:
            arts = ta.forward_many(jobs, workers)
        finally:
            ta.SETTINGS = old
    for k in cases:
        k["art"] = {role: arts[(k["test"], k["cutoff"], role)] for role in ROLES[fam]}
        k["refusal"] = {role: f"{x['support'].get('gate')}: {'; '.join(x['support'].get('reasons', []))}"
                        for role, x in k["art"].items() if not ta.admitted(x)}
    return cases


def contributions(cases, baseline):
    """Per-case IBS contributions of F1, surrogate and incidence baseline on the cases both releases answer;
    the surrogate alone on the cases F1 refuses. All endpoints are exact events, so IPCW weights are 1."""
    def contrib(sel, models):
        out = {}
        if not sel:
            return out
        outcomes = [k["outcome"] for k in sel]
        for m in models:
            if m == "incidence_baseline":
                cif = {c: [baseline[k["family"]][f"{k['cutoff']:g}"][c] for k in sel] for c in CAUSES}
            else:
                cif = {c: [ta.cif_of(k["art"][m])[c] for k in sel] for c in CAUSES}
            s = ta.score(outcomes, cif)
            out[m] = {c: s[c]["contrib"] for c in CAUSES}
            out[m]["sum"] = [a + b for a, b in zip(out[m]["extinction"], out[m]["operator_termination"])]
        return out
    answered = [k for k in cases if not k["refusal"]]
    refused_f1 = [k for k in cases if "f1" in k["refusal"] and "surrogate" not in k["refusal"]]
    return answered, contrib(answered, ("f1", "surrogate", "incidence_baseline")), refused_f1, \
        contrib(refused_f1, ("surrogate", "incidence_baseline"))


def means(v, sessions):
    by = {}
    for x, s in zip(v, sessions):
        by.setdefault(s, []).append(x)
    return dict(case_mean=math.fsum(v) / len(v), session_equal_mean=math.fsum(math.fsum(b) / len(b) for b in by.values()) / len(by))


def summarise(cases, baseline):
    out = dict(cases=len(cases), tests=len({k["test"] for k in cases}), sessions=len({k["session"] for k in cases}),
               carrying_stop=sum(k["carries_stop"] for k in cases),
               refused={role: dict(cases=sum(role in k["refusal"] for k in cases),
                                   tests=len({k["test"] for k in cases if role in k["refusal"]}),
                                   reasons=dict(Counter(k["refusal"][role] for k in cases if role in k["refusal"])))
                        for role in ("f1", "surrogate")})
    ans, c, ref, cr = contributions(cases, baseline)
    out["answered"] = dict(cases=len(ans), tests=len({k["test"] for k in ans}), sessions=len({k["session"] for k in ans}))
    if ans:
        ss = [k["session"] for k in ans]
        out["answered"]["endpoints"] = dict(Counter(k["outcome"]["causes"][0] for k in ans))
        out["answered"]["score"] = {m: {x: means(c[m][x], ss) for x in METRICS} for m in c}
        out["answered"]["paired_f1_minus_surrogate"] = {x: ta.boot([a - b for a, b in zip(c["f1"][x], c["surrogate"][x])], ss)
                                                        for x in METRICS}
        out["answered"]["paired_f1_minus_incidence"] = {x: ta.boot([a - b for a, b in zip(c["f1"][x], c["incidence_baseline"][x])], ss)
                                                        for x in METRICS}
    if ref:
        rs = [k["session"] for k in ref]
        out["refused_by_f1"] = dict(cases=len(ref), tests=len({k["test"] for k in ref}), sessions=len(set(rs)),
                                    endpoints=dict(Counter(k["outcome"]["causes"][0] for k in ref)),
                                    score={m: {x: means(cr[m][x], rs) for x in METRICS} for m in cr})
    return out


def load():
    g6.check_days()
    baseline = {fam: v["cutoffs"] for fam, v in json.loads(ta.BASELINE.read_text(encoding="utf-8"))["families"].items()}
    tests, sha_c, sha_e, dropped = ta.fd_tests()
    assert not any(t["part"] == "test_days" for t in tests)
    return baseline, tests, sha_c, sha_e, dropped


def cmd_score(args):
    ta.selftest()
    binary = ta.use_binary(args)
    chain = json.loads((WORK / "cfig/chain_check.json").read_text(encoding="utf-8"))
    if not (chain["reproduced"] and chain["ignis"]["sha256"] == binary["sha256"]):
        sys.exit("chain check not reproduced with this binary")
    baseline, tests, sha_c, sha_e, dropped = load()
    rep = dict(purpose="PROTOCOL 12.8: F1 (fuel as recorded) against the ethane surrogate, Flame Design ethylene",
               ignis=binary, releases=ROLES, inputs=dict(conditions_csv_sha256=sha_c, endpoints_csv_sha256=sha_e),
               baseline=chain["baseline"], bootstrap=dict(resamples=ta.N_BOOT, seed=ta.BOOT_SEED, unit="session"),
               dropped_test_days=dropped, tests={}, blocks={})
    for fam in ROLES:
        eth = [t for t in tests if t["family"] == fam and t["fuel_species"] == "ethylene"]
        rep["tests"][fam] = dict(ethylene=len(eth), sessions=len({t["session"] for t in eth}),
                                 with_endpoint=sum(t["endpoint"] is not None for t in eth),
                                 without=dict(Counter(t["why"] for t in eth if t["endpoint"] is None)),
                                 by_days={p: sum(t["part"] == p for t in eth) for p in PARTS})
        for stop in (False, True):
            cases = forwards(fam, eth, stop, sha_c, sha_e, args.workers)
            b = summarise(cases, baseline)
            b["by_days"] = {p: summarise([k for k in cases if k["part"] == p], baseline) for p in PARTS}
            # F1's refusals: the T_ad of the refused tests against the 1900 K line (a condition, not an outcome)
            tad = {t["test"]: float(t["tad_k_table"]) for t in eth}
            b["f1_refused_tests_tad_k"] = sorted({tad[k["test"]] for k in cases if "f1" in k["refusal"]})
            b["f1_answered_tests_tad_min_k"] = min((tad[k["test"]] for k in cases if not k["refusal"]), default=None)
            rep["blocks"][f"{fam}/{'stop' if stop else 'nostop'}"] = b
    wj(WORK / "report.json", rep)
    print("wrote", (WORK / "report.json").relative_to(REPO).as_posix())
    return 0


def rule(p):
    ci = p["ci95_case_mean"]
    if ci is None:
        return "level (no interval)"
    if p["case_mean"] < 0 and ci[1] < 0:
        return "adds something"
    if p["case_mean"] > 0 and ci[0] > 0:
        return "loses something"
    return "level"


def cmd_tables(_args):
    rep = json.loads((WORK / "report.json").read_text(encoding="utf-8"))
    f4 = lambda x: f"{x:.4f}"  # noqa: E731
    ci = lambda c: "(one session)" if c is None else f"[{c[0]:+.4f}, {c[1]:+.4f}]"  # noqa: E731
    print("tests:", json.dumps(rep["tests"]), "\ndropped:", rep["dropped_test_days"])

    def show(label, b):
        a = b["answered"]
        print(f"  -- {label}: cases {b['cases']} ({b['tests']} tests, {b['sessions']} sessions), carrying stop {b['carrying_stop']}; "
              f"answered by both {a['cases']} ({a['tests']} tests, {a['sessions']} sessions) {a.get('endpoints')}")
        for role, r in b["refused"].items():
            if r["cases"]:
                print(f"     refused by {role}: {r['cases']} cases / {r['tests']} tests: {r['reasons']}")
        if "score" in a:
            for m, v in a["score"].items():
                print(f"     {m:<19} " + "  ".join(f"{f4(v[x]['case_mean'])} ({f4(v[x]['session_equal_mean'])})" for x in METRICS))
            for name in ("paired_f1_minus_surrogate", "paired_f1_minus_incidence"):
                for x in METRICS:
                    p = a[name][x]
                    print(f"     {name} [{x}]: {p['case_mean']:+.4f} {ci(p['ci95_case_mean'])}; session-equal "
                          f"{p['session_equal_mean']:+.4f} {ci(p['ci95_session_equal_mean'])}")
            print(f"     rule (extinction, case mean): {rule(a['paired_f1_minus_surrogate']['extinction'])}")
        if "refused_by_f1" in b:
            r = b["refused_by_f1"]
            print(f"     on the {r['cases']} cases F1 refuses ({r['tests']} tests, {r['sessions']} sessions, {r['endpoints']}):")
            for m, v in r["score"].items():
                print(f"       {m:<17} " + "  ".join(f"{f4(v[x]['case_mean'])} ({f4(v[x]['session_equal_mean'])})" for x in METRICS))
    for key, b in rep["blocks"].items():
        print(f"\n== {key}; F1 refused tests' table Tad (K): {b['f1_refused_tests_tad_k']}; lowest answered Tad {b['f1_answered_tests_tad_min_k']}")
        show("all remaining", b)
        for p, v in b["by_days"].items():
            show(p, v)
    return 0


# ---------------------------------------------------------------------------------------------
DENSE = [round(math.exp(math.log(1.5) + i * (math.log(600.0) - math.log(1.5)) / 79), 4) for i in range(80)]


def quantile_time(times, cdf_total, q):
    """First time at which the total event probability reaches q (linear in ln t); None if never on the grid."""
    prev_t, prev_p = None, None
    for t, p in zip(times, cdf_total):
        if p >= q:
            if prev_t is None or p == prev_p:
                return t
            lt = math.log(prev_t) + (q - prev_p) / (p - prev_p) * (math.log(t) - math.log(prev_t))
            return math.exp(lt)
        prev_t, prev_p = t, p
    return None


def cmd_train_days(args):
    """One row per (family, path, case) of the TRAIN DAYS only. Validation-day tests are not written."""
    binary = ta.use_binary(args)
    baseline, tests, sha_c, sha_e, _ = load()
    out = WORK / "train_days_cases.jsonl"
    n = Counter()
    with out.open("w", encoding="utf-8", newline="\n") as f:
        for fam in ROLES:
            eth = [t for t in tests if t["family"] == fam and t["fuel_species"] == "ethylene" and t["part"] == "train_days"]
            for stop in (False, True):
                path = "stop" if stop else "nostop"
                cases = forwards(fam, eth, stop, sha_c, sha_e, args.workers)
                assert all(k["part"] == "train_days" for k in cases)
                ans, c, ref, cr = contributions(cases, baseline)
                idx = {id(k): i for i, k in enumerate(ans)}
                ridx = {id(k): i for i, k in enumerate(ref)}
                dense_settings = dict(ta.SETTINGS, times_s=DENSE)
                dense = {(k["test"], k["cutoff"]): k for k in forwards(fam, eth, stop, sha_c, sha_e, args.workers, dense_settings, "_dense")}
                for k in cases:
                    p = k["prefixes"]["f1"]
                    cond = {key: (m["value_si"] if m.get("value_si") is not None else m.get("raw_value"))
                            for key, m in p["known_conditions"].items()}
                    a = k["art"]["f1"]
                    d = dense[(k["test"], k["cutoff"])]["art"]
                    row = dict(family=fam, path=path, test=k["test"], session=k["session"], days="train", cutoff_s=k["cutoff"],
                               conditions=cond, endpoint=dict(cause=k["outcome"]["causes"][0], time_s=k["outcome"]["t_upper"]),
                               f1_refused=k["refusal"].get("f1"), surrogate_refused=k["refusal"].get("surrogate"),
                               f1_forecast=dict(times_s=a["times_s"], event_categories=a["event_categories"], event_cdf=a["event_cdf"],
                                                diagnostics=a.get("diagnostics")) if "f1" not in k["refusal"] else None,
                               f1_forecast_dense=dict(times_s=d["f1"]["times_s"], event_categories=d["f1"]["event_categories"],
                                                      event_cdf=d["f1"]["event_cdf"]) if "f1" not in k["refusal"] else None,
                               surrogate_forecast_dense=dict(times_s=d["surrogate"]["times_s"],
                                                             event_categories=d["surrogate"]["event_categories"],
                                                             event_cdf=d["surrogate"]["event_cdf"]) if "surrogate" not in k["refusal"] else None)
                    if id(k) in idx:
                        i = idx[id(k)]
                        row["ibs_contrib"] = {m: {x: c[m][x][i] for x in METRICS} for m in c}
                    elif id(k) in ridx:
                        i = ridx[id(k)]
                        row["ibs_contrib"] = {m: {x: cr[m][x][i] for x in METRICS} for m in cr}
                    f.write(json.dumps(row, ensure_ascii=False) + "\n")
                    n[(fam, path)] += 1
    print(json.dumps(dict(wrote=out.relative_to(REPO).as_posix(), ignis=binary, rows={f"{a}/{b}": v for (a, b), v in n.items()})))
    return 0


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    sub = ap.add_subparsers(dest="cmd", required=True)
    for name, fn in (("chain", cmd_chain), ("endpoints", cmd_endpoints), ("score", cmd_score), ("tables", cmd_tables),
                     ("train-days", cmd_train_days)):
        sub.add_parser(name).set_defaults(fn=fn)
    for q in sub.choices.values():
        q.add_argument("--workers", type=int, default=12)
        q.add_argument("--ignis", help="ignis binary relative to the repository (default target/release/ignis.exe)")
    args = ap.parse_args()
    return args.fn(args)


if __name__ == "__main__":
    sys.exit(main())
