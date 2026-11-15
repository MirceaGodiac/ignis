#!/usr/bin/env python3
"""PROTOCOL 12.10 part (b): F2 (`acme_*.wsp12`) on the Flame Design VALIDATION days, once, against F1 as
scored in 12.8 and ethylene as ethane (the 11.1 surrogate), with the incidence baseline of 11.1.

Inputs as registered:
  * without the planned stop every release reads the same prefix (ethylene as itself for F2 and F1;
    ethane's indicators for the surrogate; the propane session as propane for all);
  * with the planned stop, F2 reads the NEW planned stop (tools/wsp/ext/acme_script2.py; the stage sum
    where the script file is incomplete), F1 and the surrogate the OLD stage sum, as in 12.8.
Endpoints: those of the 12.8 run (fd_fuel/flame_design/endpoints.csv, flame_design.py unchanged).
The test days are not read. Chain functions are those of transfer_acme.py / transfer_gen6.py /
transfer_fuel.py, unchanged. Writes under runs/dev/data_all/work/transfer_acme/fd_f2/.

    python tools/wsp/ext/transfer_f2.py chain --ignis target-merged/release/ignis.exe
    python tools/wsp/ext/transfer_f2.py score --ignis target-merged/release/ignis.exe
    python tools/wsp/ext/transfer_f2.py tables
"""
import argparse
import json
import math
import sys
from collections import Counter
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import transfer_fuel as tf  # noqa: E402  (fd_fuel layout: conditions + merged endpoints of 12.8)
import transfer_gen6 as g6  # noqa: E402
import transfer_acme as ta  # noqa: E402
import acme_script2  # noqa: E402

REPO = ta.REPO
WORK = REPO / "runs/dev/data_all/work/transfer_acme/fd_f2"
ROLES = {
    "acme_normal": dict(f2="sha256:2ae770d8d159bca359e2125ef348ce13724abce49e91af4a3d62c5c66af6bc52",
                        f1=tf.ROLES["acme_normal"]["f1"], surrogate=tf.ROLES["acme_normal"]["surrogate"]),
    "acme_inverse": dict(f2="sha256:7404f3ad888f8c01d38ce4267588f5ea4532343dd23d491eb21097ae304aa7f7",
                         f1=tf.ROLES["acme_inverse"]["f1"], surrogate=tf.ROLES["acme_inverse"]["surrogate"]),
}
REGISTERED = {"acme_normal": (0.052162, 0.023997, 210), "acme_inverse": (0.027091, 0.060816, 204)}
METRICS = ("extinction", "operator_termination", "sum")


def cmd_chain(args):
    ta.selftest()
    binary = ta.use_binary(args)
    rep = dict(purpose="PROTOCOL 12.10 chain check: F2 on CFI-G train, no planned stop", ignis=binary, families={})
    ok = True
    for fam, roles in ROLES.items():
        cases, nar, runs, _ = ta.cfig_cases(fam)
        jobs = []
        for k in cases:
            name = f"{k['test']}_c{k['cutoff']:g}.json"
            pp = WORK / "cfig/prefix" / fam / name
            ta.write_json(pp, k["prefix"])
            for role in ("f2", "f1"):
                jobs.append(((k["run_id"], k["cutoff"], role), roles[role], pp, WORK / "cfig/artifact" / fam / role / name))
        arts = ta.forward_many(jobs, args.workers)
        out = dict(runs=runs, cases=len(cases), not_at_risk=nar, releases={})
        for role in ("f2", "f1"):
            a = [arts[(k["run_id"], k["cutoff"], role)] for k in cases]
            assert all(ta.admitted(x) for x in a)
            s = ta.score([k["outcome"] for k in cases], {c: [ta.cif_of(x)[c] for x in a] for c in ta.CAUSES})
            out["releases"][role] = {c: s[c]["integrated_brier"] for c in ta.CAUSES}
        ref = json.loads((WORK / f"cfig/wspdev_fb_train_{fam.split('_')[1]}.json").read_text(encoding="utf-8"))
        rr = next(r for r in ref["releases"] if r["release_id"] == roles["f2"])
        ev = rr["aggregate"]["events"]["causes"]
        out["wsp_dev_f2"] = {c: ev[c]["integrated_brier"] for c in ta.CAUSES}
        out["wsp_dev_counts"] = [ref["runs"], len(rr["cases"]), ref["not_at_risk_cases"]]
        reg = REGISTERED[fam]
        out["f2_equals_registered"] = len(cases) == reg[2] and all(abs(out["releases"]["f2"][c] - r) <= 5e-7 for c, r in zip(ta.CAUSES, reg))
        out["chain_equals_wsp_dev"] = all(abs(out["releases"]["f2"][c] - out["wsp_dev_f2"][c]) <= 1e-12 for c in ta.CAUSES)
        ok &= out["f2_equals_registered"] and out["chain_equals_wsp_dev"] and out["wsp_dev_counts"] == [runs, len(cases), nar]
        rep["families"][fam] = out
    rep["reproduced"] = bool(ok)
    ta.write_json(WORK / "cfig/chain_check.json", rep)
    print(json.dumps(rep, indent=1))
    return 0 if ok else 2


def new_stop(t):
    r = acme_script2.for_test("PSI-10", t["test"])
    v = r.get("planned_stop_s")
    if v is not None and v > 0:
        return v, r["branch"]
    s = t["programmed_stop"]
    return (s, f"{r['branch']} -> stage sum (registered fallback)") if s == s and s > 0 else (float("nan"), f"{r['branch']}, no stage sum")


def build(fam, tests, stop, sha_c, sha_e, workers):
    cases, jobs, branches = [], [], Counter()
    for t in tests:
        ethylene = t["fuel_species"] == "ethylene"
        t_new = dict(t)
        if stop:
            v, br = new_stop(t)
            t_new["programmed_stop"] = v
            branches[br] += 1
        for c in ta.CUTOFFS:
            if not ta.at_risk(t["endpoint"], c):
                continue
            k = dict(test=t["test"], session=t["session"], part=t["part"], cutoff=c, outcome=ta.outcome_of(t["endpoint"]),
                     fuel=t["fuel_species"], family=fam, carries={})
            for role, rel in ROLES[fam].items():
                fuel = "ethane" if (role == "surrogate" and ethylene) else "recorded"
                src = t_new if (stop and role == "f2") else t
                p = g6.prefix(src, c, fuel, stop, sha_c, sha_e)
                if stop and role == "f2" and "programmed_stop" in p["known_conditions"]:
                    m = p["known_conditions"]["programmed_stop"]
                    m["derivation"].update(method_id=acme_script2.METHOD_ID, method_version=acme_script2.METHOD_VERSION,
                                           notes="planned flow stop on the forecast clock from the script parameter file "
                                                 "(tools/wsp/ext/acme_script2.py; the stage sum where the file is incomplete)")
                    p = ta.finish_prefix(p)
                if role == "f3":
                    # PROTOCOL 13.5 (b): F3 reads F2's prefix and the script's burner limit of the test (an input)
                    p = g6.prefix(t_new if stop else t, c, fuel, stop, sha_c, sha_e)
                    if stop and "programmed_stop" in p["known_conditions"]:
                        p["known_conditions"]["programmed_stop"]["derivation"].update(
                            method_id=acme_script2.METHOD_ID, method_version=acme_script2.METHOD_VERSION)
                    lim = acme_script2.burner_limit_for_test("PSI-10", t["test"]).get("burner_limit_c")
                    if lim is not None:
                        m = dict(p["known_conditions"]["pressure"])
                        m.update(raw_value=float(lim), value_si=float(lim), raw_unit="degC", unit_si="degC")
                        p["known_conditions"]["burner_limit_c"] = m
                    p = ta.finish_prefix(p)
                    k.setdefault("limit", lim)
                if role in ("f2", "f1", "f3"):
                    assert "fuel_is_ethane" not in p["known_conditions"]
                k["carries"][role] = "programmed_stop" in p["known_conditions"]
                name = f"{t['test']}_c{c:g}_{role}_{'stop' if stop else 'nostop'}.json"
                pp = WORK / "prefix" / fam / name
                ta.write_json(pp, p)
                jobs.append(((t["test"], c, role), rel, pp, WORK / "artifact" / fam / role / name))
            cases.append(k)
    arts = ta.forward_many(jobs, workers)
    for k in cases:
        k["art"] = {role: arts[(k["test"], k["cutoff"], role)] for role in ROLES[fam]}
        k["refusal"] = {role: "; ".join(a["support"].get("reasons", [])) for role, a in k["art"].items() if not ta.admitted(a)}
    return cases, dict(branches)


def contrib(sel, roles, baseline):
    outcomes = [k["outcome"] for k in sel]
    out = {}
    for r in roles:
        if r == "incidence_baseline":
            cif = {c: [baseline[k["family"]][f"{k['cutoff']:g}"][c] for k in sel] for c in ta.CAUSES}
        else:
            cif = {c: [ta.cif_of(k["art"][r])[c] for k in sel] for c in ta.CAUSES}
        s = ta.score(outcomes, cif)
        out[r] = {c: s[c]["contrib"] for c in ta.CAUSES}
        out[r]["sum"] = [a + b for a, b in zip(out[r]["extinction"], out[r]["operator_termination"])]
    return out


def summarise(cases, baseline):
    roles = ("f2", "f1", "surrogate")
    out = dict(cases=len(cases), tests=len({k["test"] for k in cases}), sessions=len({k["session"] for k in cases}),
               endpoints=dict(Counter(k["outcome"]["causes"][0] for k in cases)),
               carrying_stop={r: sum(k["carries"][r] for k in cases) for r in roles},
               refused={r: dict(cases=sum(r in k["refusal"] for k in cases), tests=len({k["test"] for k in cases if r in k["refusal"]}),
                                reasons=dict(Counter(k["refusal"][r] for k in cases if r in k["refusal"]))) for r in roles},
               only_one_of_f2_f1=dict(f2_only=sum("f2" not in k["refusal"] and "f1" in k["refusal"] for k in cases),
                                      f1_only=sum("f1" not in k["refusal"] and "f2" in k["refusal"] for k in cases)))
    ans = [k for k in cases if not k["refusal"]]
    out["all_answer"] = dict(cases=len(ans), tests=len({k["test"] for k in ans}), sessions=len({k["session"] for k in ans}))
    if ans:
        c = contrib(ans, roles + ("incidence_baseline",), baseline)
        ss = [k["session"] for k in ans]
        out["all_answer"]["score"] = {m: {x: tf.means(c[m][x], ss) for x in METRICS} for m in c}
        out["all_answer"]["paired"] = {f"f2_minus_{o}": {x: ta.boot([a - b for a, b in zip(c["f2"][x], c[o][x])], ss) for x in METRICS}
                                       for o in ("f1", "surrogate")}
    both = [k for k in cases if "f2" not in k["refusal"] and "f1" not in k["refusal"]]
    if both:
        c = contrib(both, ("f2", "f1"), baseline)
        out["f2_f1_both_answer"] = dict(cases=len(both), tests=len({k["test"] for k in both}),
                                        score={m: {x: math.fsum(c[m][x]) / len(both) for x in METRICS} for m in c})
    return out


def cmd_score(args):
    ta.selftest()
    binary = ta.use_binary(args)
    chain = json.loads((WORK / "cfig/chain_check.json").read_text(encoding="utf-8"))
    if not (chain["reproduced"] and chain["ignis"]["sha256"] == binary["sha256"]):
        sys.exit("chain check not reproduced with this binary")
    g6.check_days()
    baseline = {fam: v["cutoffs"] for fam, v in json.loads(tf.ta.BASELINE.read_text(encoding="utf-8"))["families"].items()}
    tests, sha_c, sha_e, dropped = ta.fd_tests()
    tests = [t for t in tests if t["part"] == "validation_days"]
    assert all(t["test"][:5] in ta.FD_DAYS[t["family"]]["validation_days"] for t in tests)
    rep = dict(purpose="PROTOCOL 12.10 (b): F2 on the Flame Design validation days", ignis=binary, releases=ROLES,
               inputs=dict(conditions_csv_sha256=sha_c, endpoints_csv_sha256=sha_e), baseline_file=str(tf.ta.BASELINE),
               bootstrap=dict(resamples=ta.N_BOOT, seed=ta.BOOT_SEED, unit="session"), dropped_test_days=dropped,
               tests={}, blocks={})
    for fam in ROLES:
        tt = [t for t in tests if t["family"] == fam]
        rep["tests"][fam] = dict(tests=len(tt), sessions=sorted({t["session"] for t in tt}), by_fuel=dict(Counter(t["fuel_species"] for t in tt)),
                                 without_endpoint=sum(t["endpoint"] is None for t in tt))
        for stop in (False, True):
            cases, br = build(fam, tt, stop, sha_c, sha_e, args.workers)
            path = "stop" if stop else "nostop"
            b = summarise(cases, baseline)
            b["f2_new_stop_branches"] = br
            if fam == "acme_normal":
                b["by_fuel"] = {f: summarise([k for k in cases if k["fuel"] == f], baseline) for f in ("ethylene", "propane")}
            rep["blocks"][f"{fam}/{path}"] = b
    ta.write_json(WORK / "report.json", rep)
    print("wrote", (WORK / "report.json").relative_to(REPO).as_posix())
    return 0


def cmd_score_f3(args):
    """PROTOCOL 13.5 (b): F3 beside F2 on the Flame Design validation days, normal flames, once. Writes
    fd_f2/report_f3.json (aggregates); F2's forwards are those of 12.10 (cached)."""
    ta.selftest()
    binary = ta.use_binary(args)
    out = WORK / "report_f3.json"
    if out.exists():
        sys.exit(f"{out} exists: run once")
    g6.check_days()
    baseline = {fam: v["cutoffs"] for fam, v in json.loads(tf.ta.BASELINE.read_text(encoding="utf-8"))["families"].items()}
    tests, sha_c, sha_e, dropped = ta.fd_tests()
    tests = [t for t in tests if t["part"] == "validation_days" and t["family"] == "acme_normal"]
    ROLES["acme_normal"] = dict(f2=ROLES["acme_normal"]["f2"], f3=args.release)
    rep = dict(purpose="PROTOCOL 13.5 (b): F3 beside F2 on the Flame Design validation days", ignis=binary,
               releases=ROLES["acme_normal"], blocks={})
    for stop in (False, True):
        cases, _br = build("acme_normal", tests, stop, sha_c, sha_e, args.workers)
        ans = [k for k in cases if not k["refusal"]]
        c = contrib(ans, ("f2", "f3", "incidence_baseline"), baseline)
        ss = [k["session"] for k in ans]
        rep["blocks"]["stop" if stop else "nostop"] = dict(
            cases=len(cases), answered_by_both=len(ans), sessions=len(set(ss)),
            refused={r: sum(r in k["refusal"] for k in cases) for r in ("f2", "f3")},
            limits=dict(Counter(str(k.get("limit")) for k in cases)),
            score={r: {x: tf.means(c[r][x], ss) for x in METRICS} for r in c})
    ta.write_json(out, rep)
    print("wrote", out.relative_to(REPO).as_posix())
    return 0


def cmd_tables(_args):
    rep = json.loads((WORK / "report.json").read_text(encoding="utf-8"))
    ci = lambda c: "(one session)" if c is None else f"[{c[0]:+.4f}, {c[1]:+.4f}]"  # noqa: E731
    print("tests", json.dumps(rep["tests"]))

    def show(label, b):
        print(f"  -- {label}: cases {b['cases']} ({b['tests']} tests, {b['sessions']} sessions) {b['endpoints']}; carrying stop {b['carrying_stop']}")
        for r, v in b["refused"].items():
            if v["cases"]:
                print(f"     refused by {r}: {v['cases']} cases / {v['tests']} tests: {v['reasons']}")
        print(f"     only one of F2/F1 answers: {b['only_one_of_f2_f1']}; all answer {b['all_answer']['cases']} cases")
        a = b["all_answer"]
        if "score" in a:
            for m, v in a["score"].items():
                print(f"     {m:<19} " + "  ".join(f"{v[x]['case_mean']:.4f} ({v[x]['session_equal_mean']:.4f})" for x in METRICS))
            for n, v in a["paired"].items():
                for x in METRICS:
                    p = v[x]
                    print(f"     {n} [{x}]: {p['case_mean']:+.4f} {ci(p['ci95_case_mean'])}; session-equal {p['session_equal_mean']:+.4f} "
                          f"{ci(p['ci95_session_equal_mean'])} ({p['n_sessions']} sessions)")
        if "f2_f1_both_answer" in b:
            v = b["f2_f1_both_answer"]
            print(f"     F2/F1 both answer {v['cases']} cases: F2 " + " ".join(f"{v['score']['f2'][x]:.4f}" for x in METRICS)
                  + " | F1 " + " ".join(f"{v['score']['f1'][x]:.4f}" for x in METRICS))
    for key, b in rep["blocks"].items():
        print(f"\n== {key}; F2 new-stop branches {b['f2_new_stop_branches']}")
        show("all validation-day tests", b)
        for f, v in b.get("by_fuel", {}).items():
            show(f, v)
    return 0


def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    for n, fn in (("chain", cmd_chain), ("score", cmd_score), ("tables", cmd_tables), ("score-f3", cmd_score_f3)):
        p = sub.add_parser(n)
        p.set_defaults(fn=fn)
        p.add_argument("--workers", type=int, default=12)
        p.add_argument("--ignis")
        p.add_argument("--release", help="score-f3: the release beside F2")
    a = ap.parse_args()
    return a.fn(a)


if __name__ == "__main__":
    sys.exit(main())
