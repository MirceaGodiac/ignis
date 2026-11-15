#!/usr/bin/env python3
"""PROTOCOL 12.3 step 3 (attribution, TRAIN only): the planned stop of acme_script2.py (v2 rule) against
the stage sum, with the diagnostic committees zs1 (runs/dev/gen6/fuel/diag/stop_variants.json).

Sets, all TRAIN:
  cfig_normal_stop    P3 on data_ext (old input)   vs zs1 normal on data_ext2 (new input)
  cfig_inverse_stop   P1 on data_ext               vs zs1 inverse on data_ext2
  cfig_normal_nostop  P3 vs zs1 normal on the shared records (data), no planned stop: the era days alone
  fd_normal_stop      F1 normal, old stop (stage sum) vs zs1 normal, new stop (acme_script2), Flame Design train days
  fd_inverse_stop     F1 inverse, old stop            vs zs1 inverse, new stop

CFI-G prefixes are built as `wsp-dev feedback` builds them (transfer_acme.cfig_prefix) from the records of
the named data root, TRAIN ids only, and each set is checked against a `wsp-dev feedback` report of the same
root. Flame Design reads the train-day copy of fd_fuel/attribution (only train-day rows). Nothing fitted.

    python tools/wsp/ext/stop_v2_attribution.py run --ignis target-merged/release/ignis.exe
"""
import argparse
import json
import math
import re
import statistics as st
import sys
from collections import Counter, defaultdict
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
sys.path.insert(0, str(HERE.parents[1] / "wsp/analysis"))
import fd_fuel_attribution as fa  # noqa: E402  (train-day copy of Flame Design; layout)
import transfer_fuel as tf  # noqa: E402
import transfer_gen6 as g6  # noqa: E402
import transfer_acme as ta  # noqa: E402
import acme_script2  # noqa: E402
from fd_train_anatomy import qt  # noqa: E402

REPO = ta.REPO
OUT = REPO / "runs/dev/data_all/work/transfer_acme/stop_v2"
ZS1 = json.loads((REPO / "runs/dev/gen6/fuel/diag/stop_variants.json").read_text(encoding="utf-8"))
P3, P1 = tf.ROLES["acme_normal"]["surrogate"], tf.ROLES["acme_inverse"]["surrogate"]
F1N, F1I = tf.ROLES["acme_normal"]["f1"], tf.ROLES["acme_inverse"]["f1"]
SEVEN = {"21349D2", "21349D3", "21349M5", "21349M6", "19171J1", "19171J3", "19189D3"}
METRICS = ("extinction", "operator_termination", "sum")


def train_rows(root, family, file, key, ids):
    pat = re.compile(r'"%s"\s*:\s*"([^"]+)"' % key)
    out = {}
    p = root / "work/records" / family / file
    if not p.exists():
        return out
    for line in p.open(encoding="utf-8"):
        if not any(x in ids for x in pat.findall(line)):
            continue
        r = json.loads(line)
        if r.get(key) in ids:
            out[r[key]] = r
    return out


def cfig_cases(family, root):
    import lib
    ids = set(lib.partition_ids(family, "train"))
    exps = train_rows(root, family, "experiments.jsonl", "id", ids)
    assert set(exps) == ids
    ends = train_rows(root, family, "endpoints.jsonl", "run_id", ids)
    ctrl = {}
    for name in ("control_histories.jsonl", "control_history.jsonl", "extra_control_history.jsonl"):
        ctrl.update(train_rows(root, family, name, "run_id", ids))
    cases = []
    for rid in sorted(ids):
        for c in ta.CUTOFFS:
            if ta.at_risk(ends.get(rid), c):
                cases.append(dict(run_id=rid, test=rid.split(":")[-1], cutoff=c, session=exps[rid].get("session_id"),
                                  outcome=ta.outcome_of(ends[rid]), prefix=ta.cfig_prefix(exps[rid], ctrl.get(rid), c),
                                  carries_stop="programmed_stop" in exps[rid]["conditions"]))
    return cases


def forward(cases, release, tag, workers, dense=False):
    jobs = []
    for k in cases:
        name = f"{k['test']}_c{k['cutoff']:g}.json"
        pp = OUT / "prefix" / tag / name
        ta.write_json(pp, k["prefix"])
        jobs.append(((k["test"], k["cutoff"]), release, pp, OUT / ("artifact_dense" if dense else "artifact") / tag / release[7:19] / name))
    old = ta.SETTINGS
    ta.SETTINGS = dict(old, times_s=tf.DENSE) if dense else old
    try:
        return ta.forward_many(jobs, workers)
    finally:
        ta.SETTINGS = old


def t_end(o):
    lo, hi = o["t_lower"], o["t_upper"]
    return (lo + hi) / 2 if lo is not None and hi is not None else (hi if hi is not None else lo)


def side(cases, release, tag, workers):
    a = forward(cases, release, tag, workers)
    d = forward(cases, release, tag, workers, dense=True)
    refused = [k for k in cases if not ta.admitted(a[(k["test"], k["cutoff"])])]
    assert not refused, (tag, len(refused), a[(refused[0]["test"], refused[0]["cutoff"])]["support"])
    s = ta.score([k["outcome"] for k in cases], {c: [ta.cif_of(a[(k["test"], k["cutoff"])])[c] for k in cases] for c in ta.CAUSES})
    con = {c: s[c]["contrib"] for c in ta.CAUSES}
    con["sum"] = [x + y for x, y in zip(con["extinction"], con["operator_termination"])]
    return dict(ib={c: s[c]["integrated_brier"] for c in ta.CAUSES}, contrib=con, art=a, dense=d)


def centre(cases, dense):
    """Tests that ended by operator termination, one row per test (earliest cutoff at risk)."""
    first = {}
    for k in cases:
        if k["outcome"]["causes"] == ["operator_termination"] and (k["test"] not in first or k["cutoff"] < first[k["test"]]["cutoff"]):
            first[k["test"]] = k
    lr, w, inside = [], [], 0
    for k in first.values():
        a = dense[(k["test"], k["cutoff"])]
        cats = a["event_categories"]
        tot = [x + y for x, y in zip(a["event_cdf"][cats.index("extinction")], a["event_cdf"][cats.index("operator_termination")])]
        t10, t50, t90 = (qt(a["times_s"], tot, q) for q in (0.1, 0.5, 0.9))
        te = t_end(k["outcome"])
        if t50 is None or not te:
            continue
        lr.append(math.log(t50 / te))
        if t10 and t90:
            w.append(math.log(t90 / t10))
            inside += t10 <= te <= t90
    q = st.quantiles(lr, n=4) if len(lr) >= 4 else [float("nan")] * 3
    return dict(tests=len(first), median=st.median(lr), q25=q[0], q75=q[2], width=st.median(w), inside=inside)


def compare(name, before_cases, after_cases, before, after, workers, tags):
    """before/after: release ids; the two case lists are the same cases (same order) with different prefixes."""
    assert [(k["test"], k["cutoff"]) for k in before_cases] == [(k["test"], k["cutoff"]) for k in after_cases]
    assert [k["outcome"] for k in before_cases] == [k["outcome"] for k in after_cases]
    b, a = side(before_cases, before, tags[0], workers), side(after_cases, after, tags[1], workers)
    n = len(before_cases)
    sess = [k["session"] for k in before_cases]
    res = dict(name=name, before=before, after=after, cases=n, tests=len({k["test"] for k in before_cases}),
               sessions=len(set(sess)),
               carrying_stop_before=sum(k.get("carries_stop", False) for k in before_cases),
               carrying_stop_after=sum(k.get("carries_stop", False) for k in after_cases),
               score_before={x: math.fsum(b["contrib"][x]) / n for x in METRICS},
               score_after={x: math.fsum(a["contrib"][x]) / n for x in METRICS},
               integrated_brier_before=b["ib"], integrated_brier_after=a["ib"])
    per = {}
    for x in METRICS:
        row = {}
        for s in sorted(set(sess)):
            idx = [i for i, t in enumerate(sess) if t == s]
            row[s] = (math.fsum(a["contrib"][x][i] for i in idx) - math.fsum(b["contrib"][x][i] for i in idx)) / len(idx)
        per[x] = dict(by_session=row, improve=sum(v < -1e-12 for v in row.values()), worsen=sum(v > 1e-12 for v in row.values()),
                      same=sum(abs(v) <= 1e-12 for v in row.values()))
    res["per_session_after_minus_before"] = per
    res["termination_centre_before"] = centre(before_cases, b["dense"])
    res["termination_centre_after"] = centre(after_cases, a["dense"])
    # the seven tests whose measured flow change falls inside an ignition-to-test ramp
    idx = [i for i, k in enumerate(before_cases) if k["test"] in SEVEN]
    if idx:
        res["seven_ramp_tests"] = dict(
            tests=sorted({before_cases[i]["test"] for i in idx}), cases=len(idx),
            termination_loss_share_before=math.fsum(b["contrib"]["operator_termination"][i] for i in idx) / math.fsum(b["contrib"]["operator_termination"]),
            termination_loss_share_after=math.fsum(a["contrib"]["operator_termination"][i] for i in idx) / math.fsum(a["contrib"]["operator_termination"]),
            termination_case_mean_before=math.fsum(b["contrib"]["operator_termination"][i] for i in idx) / len(idx),
            termination_case_mean_after=math.fsum(a["contrib"]["operator_termination"][i] for i in idx) / len(idx),
            termination_mean_other_before=math.fsum(v for i, v in enumerate(b["contrib"]["operator_termination"]) if i not in idx) / (n - len(idx)),
            termination_mean_other_after=math.fsum(v for i, v in enumerate(a["contrib"]["operator_termination"]) if i not in idx) / (n - len(idx)))
    # era of each test as the releases read it (p_late), cutoff-free
    pl = Counter()
    for k in before_cases:
        x, y = (side_art["art"][(k["test"], k["cutoff"])].get("diagnostics") or {} for side_art in (b, a))
        pl[f"{x.get('p_late')} -> {y.get('p_late')}"] += 1
    res["p_late_before_to_after_cases"] = dict(pl)
    res["p_late_tests_changed"] = len({k["test"] for k in before_cases
                                       if (b["art"][(k["test"], k["cutoff"])].get("diagnostics") or {}).get("p_late")
                                       != (a["art"][(k["test"], k["cutoff"])].get("diagnostics") or {}).get("p_late")})
    return res


def wsp_dev_check(res, fb_before, fb_after):
    out = {}
    for side_name, fb, rel in (("before", fb_before, res["before"]), ("after", fb_after, res["after"])):
        d = json.loads((OUT / "cfig" / fb).read_text(encoding="utf-8"))
        r = next(x for x in d["releases"] if x["release_id"] == rel)
        ev = r["aggregate"]["events"]["causes"]
        out[side_name] = dict(cases=len(r["cases"]),
                              max_abs_diff=max(abs(ev[c]["integrated_brier"] - res[f"integrated_brier_{side_name}"][c]) for c in ta.CAUSES))
    res["wsp_dev_check"] = out


def fd_cases(family, stop_rule):
    """Flame Design train-day ethylene cases of one family; stop_rule 'old' (stage sum) or 'new' (acme_script2)."""
    tests, sha_c, sha_e = fa.train_tests() if family == "acme_normal" else (None, None, None)
    if family == "acme_inverse":
        fa.use_train_copy()
        import csv
        import hashlib
        cpath, epath = fa.FDW / "conditions.csv", fa.FDW / "endpoints.csv"
        sha_c, sha_e = (hashlib.sha256(p.read_bytes()).hexdigest() for p in (cpath, epath))
        cond = [dict(r, row=i + 2) for i, r in enumerate(csv.DictReader(cpath.open(encoding="utf-8", newline="")))
                if r["include_transfer"] == "1" and r["family"] == family]
        ends = {r["test"]: dict(r, erow=i + 2) for i, r in enumerate(csv.DictReader(epath.open(encoding="utf-8", newline="")))}
        tests = []
        for r in cond:
            assert r["test"][:5] in fa.TRAIN[family]
            e = ends[r["test"]]
            end = float(e["end_s"])
            tests.append(dict(r, part="train_days", erow=e["erow"], programmed_stop=float(e["script_programmed_stop_s"] or "nan"),
                              endpoint=dict(censoring="event", last_confirmed_active_s=end, first_terminal_s=end,
                                            allowed_causes=[e["label_sensor"]])))
    branches = Counter()
    cases = []
    for t in tests:
        t = dict(t)
        if family == "acme_normal" and float(t["tad_k_table"]) < 1900.0:
            branches["skipped: table T_ad below F1's 1900 K bound (F1 and zs1 refuse)"] += 1 if stop_rule == "new" else 0
            continue
        if stop_rule == "new":
            r = acme_script2.for_test("PSI-10", t["test"])
            v = r.get("planned_stop_s")
            branches[r["branch"]] += 1
            t["programmed_stop"] = v if v is not None and v > 0 else float("nan")
        for c in ta.CUTOFFS:
            if not ta.at_risk(t["endpoint"], c):
                continue
            p = g6.prefix(t, c, "recorded", True, sha_c, sha_e)
            if stop_rule == "new" and "programmed_stop" in p["known_conditions"]:
                m = p["known_conditions"]["programmed_stop"]
                m["derivation"].update(method_id=acme_script2.METHOD_ID, method_version=acme_script2.METHOD_VERSION,
                                       notes="planned flow stop on the forecast clock from the script parameter file alone "
                                             "(tools/wsp/ext/acme_script2.py, D + S + F - G)")
                m["locator"].update(column="script parameters", path=acme_script2.script_path("PSI-10", t["test"]).name)
                p = ta.finish_prefix(p)
            cases.append(dict(test=t["test"], cutoff=c, session=t["session"], outcome=ta.outcome_of(t["endpoint"]), prefix=p,
                              carries_stop="programmed_stop" in p["known_conditions"]))
    return cases, dict(branches)


def cmd_run(args):
    binary = ta.use_binary(args)
    rep = dict(purpose="12.3 step 3: planned stop v2 against the stage sum, TRAIN only", ignis=binary, zs1=ZS1, sets={})
    ext, ext2, data = REPO / "runs/dev/data_ext", REPO / "runs/dev/data_ext2", REPO / "data"
    for fam, before, fb in (("acme_normal", P3, "normal"), ("acme_inverse", P1, "inverse")):
        cb, ca = cfig_cases(fam, ext), cfig_cases(fam, ext2)
        r = compare(f"CFI-G train {fb}, with planned stop", cb, ca, before, ZS1[fb], args.workers, (f"cfig_{fb}_old", f"cfig_{fb}_new"))
        wsp_dev_check(r, f"fb_{fb}_stop_old.json", f"fb_{fb}_stop_new.json")
        rep["sets"][f"cfig_{fb}_stop"] = r
    cn = cfig_cases("acme_normal", data)
    r = compare("CFI-G train normal, no planned stop (era days only)", cn, cn, P3, ZS1["normal"], args.workers, ("cfig_normal_nostop", "cfig_normal_nostop"))
    wsp_dev_check(r, "fb_normal_nostop.json", "fb_normal_nostop.json")
    rep["sets"]["cfig_normal_nostop"] = r
    for fam, f1, fb in (("acme_normal", F1N, "normal"), ("acme_inverse", F1I, "inverse")):
        cb, _ = fd_cases(fam, "old")
        ca, br = fd_cases(fam, "new")
        r = compare(f"Flame Design train days {fb}, ethylene as itself, with planned stop", cb, ca, f1, ZS1[fb], args.workers,
                    (f"fd_{fb}_old", f"fd_{fb}_new"))
        r["new_stop_branches_by_test"] = br
        rep["sets"][f"fd_{fb}_stop"] = r
    ta.write_json(OUT / "report.json", rep)
    for k, r in rep["sets"].items():
        print(f"\n== {k}: {r['cases']} cases, {r['tests']} tests, {r['sessions']} sessions; stop carried {r['carrying_stop_before']} -> {r['carrying_stop_after']}; "
              f"wsp-dev check {r.get('wsp_dev_check')}; branches {r.get('new_stop_branches_by_test')}")
        print("   before", {x: round(v, 4) for x, v in r["score_before"].items()})
        print("   after ", {x: round(v, 4) for x, v in r["score_after"].items()})
        for x, p in r["per_session_after_minus_before"].items():
            print(f"   per session [{x}]: improve {p['improve']} worsen {p['worsen']} same {p['same']}: "
                  + ", ".join(f"{s[-10:]} {v:+.4f}" for s, v in p["by_session"].items()))
        for w in ("before", "after"):
            c = r[f"termination_centre_{w}"]
            print(f"   termination centre {w}: tests {c['tests']} median {c['median']:+.2f} [{c['q25']:+.2f}, {c['q75']:+.2f}] width {c['width']:.2f} inside {c['inside']}")
        print("   p_late", r["p_late_before_to_after_cases"], "tests changed", r["p_late_tests_changed"])
        if "seven_ramp_tests" in r:
            print("   seven", json.dumps(r["seven_ramp_tests"]))
    return 0


def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    p = sub.add_parser("run")
    p.set_defaults(fn=cmd_run)
    p.add_argument("--workers", type=int, default=12)
    p.add_argument("--ignis")
    a = ap.parse_args()
    return a.fn(a)


if __name__ == "__main__":
    sys.exit(main())
