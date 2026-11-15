#!/usr/bin/env python3
"""The s-Flame (PSI-23) step of the final run of the seventh generation (PROTOCOL 12.5, 13.5, 13.6): the burner-flame
releases scored once on the unread TEST days 20142, 20170, 20176, on a frozen release list, aggregates only.

    python tools/wsp/final/run_final_sflame.py inputs-check            # test days, INPUTS only: counts and admission
    python tools/wsp/final/run_final_sflame.py rehearse                # the identical path on the VALIDATION days
    python tools/wsp/final/run_final_sflame.py final --releases tools/wsp/final/releases_final_sflame.json \\
        --sha256 <sha256 of the list> --execute

Day 19245 is not a test day: it is not in the experiment table nor in the split file, and its eight files (two
session logs and six gas-chromatograph module files, dated 2019-09-11, the date of day 19254) are mis-named files of
train test 19254K1 (docs/wsp/atlas/FILES.md); it holds no script file and no data file, so it gives no case and does
not belong to the final run.

Code path (rehearsal and final alike):
  1. the lane's case builder (`tools/wsp/ext/sflame_cases.py`: inputs, endpoints, build), unchanged, with its output
     directory set to a local root that is never printed (`runs/final/gen7/<mode>/transfer_sflame/`) and its
     partition map relabelled: rehearsal = the frozen map; final = the test days labelled "validation" and the real
     validation days left out (train days rebuilt as they are). Inputs-check runs the inputs step only.
  2. every listed release on every constant-flow case of the scored days with its registered inputs (F2: the prefixes
     of 13.1, a test with helium dilution refused, as 13.1 registered; the wsp13 releases: `sflame_cases.f3_prefix`,
     the script's burner limit, the diluent, the two oxygen candidates; blends refused by the lane's adapter), cutoffs
     1, 3, 5 s, without and with the planned stop; the CFI-G train incidence and the s-Flame train incidence on the
     cases each release answers. Each stratum (hydrocarbons in nitrogen, hydrocarbons in helium, hydrogen, blends) is
     scored as its own set, as the registered looks were. Endpoints: the final run is scored on version 2, the data
     lane's general rule (`sflame_endpoints_v2.md`, `sflame_cases.endpoints_v2_for`); the s-Flame train incidence
     stays the registered one (train days on version 1, as in 13.1 and 13.5); the rehearsal scores version 1 (against the registered numbers) and version 2.
  3. aggregates only: rehearsal compares with the registered numbers (13.1: F2 0.2619 / 0.3698; 13.5 (a): F3 stage 1
     0.2048 / 0.2051; 13.5 stage 2: F3 stage 2 on the helium tests 0.9990; and the F4 development tables); final writes
     docs/wsp/RESULTS_FINAL_SFLAME.md and runs/final/gen7/final/sflame.json, appends one line per scored forecast to
     runs/evaluation/reveal_log.jsonl, and creates runs/final/gen7/markers/sflame.started before reading anything, so
     that it cannot run twice.

`final` refuses unless the list's sha256 equals --sha256, the list says "frozen": true, and --execute is given.
Nothing in this file opens a test-day file outside `inputs-check` (inputs only: the script file and the first 0.5 s
of the data file, before the flow is on) and `final`.
"""
import argparse
import contextlib
import datetime
import hashlib
import io
import json
import math
import os
import sys
from collections import Counter, defaultdict
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(REPO / "tools/wsp/ext"))
import sflame_cases as sc  # noqa: E402
import transfer_acme as ta  # noqa: E402

ROOT = REPO / "runs/final/gen7"
MARKER = ROOT / "markers/sflame.started"
REVEAL_LOG = REPO / "runs/evaluation/reveal_log.jsonl"
RESULTS_MD = REPO / "docs/wsp/RESULTS_FINAL_SFLAME.md"
LIST = REPO / "tools/wsp/final/releases_final_sflame.json"
TEST_DAYS = ("20142", "20170", "20176")
STRATA = ("hydrocarbons in nitrogen", "hydrocarbons in helium", "hydrogen", "blends")
REGISTERED = {  # rehearsal: (release label, stratum, path) -> sum, cases
    ("F2", "hydrocarbons in nitrogen", "nostop"): (0.2619, 46), ("F2", "hydrocarbons in nitrogen", "stop"): (0.3698, 46),
    ("F3 stage 1", "hydrocarbons in nitrogen", "nostop"): (0.2048, 46),
    ("F3 stage 1", "hydrocarbons in nitrogen", "stop"): (0.2051, 46),
    ("F3 stage 2", "hydrocarbons in helium", "nostop"): (0.9990, 35),
    # the development tables of F4 (docs/wsp/feedback/sflame_f4_result.md, coded endpoints)
    ("F4 stage 1, methane", "hydrocarbons in nitrogen", "nostop"): (0.0815, 46),
    ("F4 stage 1, methane", "hydrocarbons in nitrogen", "stop"): (0.0811, 46),
    ("F4 stage 2, helium", "hydrocarbons in helium", "nostop"): (0.1105, 35),
    ("F4 stage 2, helium", "hydrocarbons in helium", "stop"): (0.1099, 35),
    ("F4 stage 3, hydrogen", "hydrocarbons in helium", "nostop"): (0.1105, 35),
}
WSP13 = "configs/models/acme/wsp13/normal"



def default_binary(name):
    """The merged build (target-merged/release, which carries every forward of the branch); this checkout's own
    target/release when there is none (a worktree)."""
    merged = REPO / "target-merged/release" / name
    return str((merged if merged.exists() else REPO / "target/release" / name).relative_to(REPO))

def rid(path):
    return json.loads((REPO / path / "release.json").read_text(encoding="utf-8"))["id"]


def template():
    rel = lambda label, path, inputs: dict(label=label, release=rid(path), path=path, inputs=inputs)
    return {
        "schema": "ignis.wsp.final-release-list.sflame/1",
        "frozen": False,
        "status": "TEMPLATE; the coordinator freezes it (frozen true, committed) before the final run",
        "investigation": "PSI-23 ACME s-Flame",
        "test_days": list(TEST_DAYS),
        "model": rel("F3 stage 1", f"{WSP13}/f3-s1", "f3"),
        "comparators": [rel("F2", "configs/models/acme/wsp12/normal/f2-fable", "f2"),
                        rel("F3 stage 2", f"{WSP13}/f3-s2", "f3"),
                        rel("F4 stage 1, methane", f"{WSP13}/f4-methane", "f3"),
                        rel("F4 stage 2, helium", f"{WSP13}/f4-helium", "f3"),
                        rel("F4 stage 3, hydrogen", f"{WSP13}/f4-hydrogen", "f3")],
        "baselines": "the CFI-G train incidence (frozen, nothing from s-Flame) and the s-Flame train incidence, on the cases each release answers",
        "endpoints": "version 2: the data lane's general rule (docs/wsp/feedback/sflame_endpoints_v2.md), by the lane's builder",
    }


@contextlib.contextmanager
def lane_root(out, parts):
    """The lane's builder pointed at `out` with the partition map `parts`; its prints are captured."""
    old_out, old_split = sc.OUT, sc.split
    sc.OUT = out
    sc.split = lambda: dict(parts)
    out.mkdir(parents=True, exist_ok=True)
    buf = io.StringIO()
    try:
        with contextlib.redirect_stdout(buf):
            yield
    finally:
        sc.OUT, sc.split = old_out, old_split


def frozen_parts():
    s = json.loads(sc.SPLIT.read_text(encoding="utf-8"))["strata"]["all"]["groups"]
    return {d.split(":")[-1]: p for p in s for d in s[p]}


def parts_for(mode):
    p = frozen_parts()
    if mode == "final":
        return {d: ("validation" if part == "test" else "held" if part == "validation" else part) for d, part in p.items()}
    if mode == "inputs-check":
        return {d: ("validation" if part == "test" else "held") for d, part in p.items()}
    return p


def build(mode):
    out = ROOT / mode / "transfer_sflame"
    with lane_root(out, parts_for(mode)):
        sc.cmd_inputs(None)
        if mode != "inputs-check":
            sc.cmd_endpoints(None)
            sc.cmd_build(None)
            # the second endpoint version (the data lane's general rule, sflame_endpoints_v2.md), by the same code
            rows = sc.endpoints_v2_for(sc.read_jsonl(out / "inputs.jsonl"), sc.list_tests())
            sc.write_jsonl(out / "endpoints_v2.jsonl", rows)
    return out


def load(out, version):
    """Train cases (version 1: they form the registered s-Flame train incidence) and scored-day cases (the days
    labelled "validation" by the mode) with the endpoints of `version`: "v1" (flame_design.py measure, as coded) or
    "v2" (sflame_endpoints_v2.md)."""
    v2 = {r["test"]: r["endpoint"] for r in sc.read_jsonl(out / "endpoints_v2.jsonl")} if version == "v2" else {}
    train = [c for c in sc.read_jsonl(out / "cases_train.jsonl") if c["in_target_set"]]
    vout = {r["test"]: r["endpoint"] for r in sc.read_jsonl(out / "cases_validation_outcomes.jsonl")}
    scored = [c for c in sc.read_jsonl(out / "cases_validation_inputs.jsonl") if c["in_target_set"]]
    for c in scored:
        c["outcome"] = dict(endpoint=vout[c["test"]])
    # the train cases only form the s-Flame train incidence, which stays the registered one (version 1) in both
    for c in scored:
        if version == "v2":
            c["outcome"] = dict(c["outcome"], endpoint=v2[c["test"]])
    return train, scored


def stratum(c):
    fuel = c["inputs"]["fuel_species"] or ""
    return "blends" if "+" in fuel else "hydrogen" if fuel == "hydrogen" else (
        "hydrocarbons in helium" if sc.is_helium(c) else "hydrocarbons in nitrogen")


def prefixes(cases, src_rel, kind, stop, cutoffs, out, tag):
    src = REPO / src_rel
    src_sha = sc.sha(src)
    row_of = {json.loads(l)["test"]: i + 1 for i, l in enumerate(src.read_text(encoding="utf-8").splitlines()) if l.strip()}
    byt = {c["test"]: c for c in cases}
    pp = {}
    for t, k in cutoffs:
        c = byt[t]
        p = (sc.f2_prefix(c, k, stop, src_sha, row_of[t], src=src_rel) if kind == "f2"
             else sc.f3_prefix(c, k, stop, src_sha, row_of[t], src_rel))
        path = out / "prefix" / tag / f"{t}_c{k:g}_{'stop' if stop else 'nostop'}.json"
        ta.write_json(path, p)
        pp[(t, k)] = path
    return pp


def run_release(rel, cases, src_rel, out, stop, cutoffs, workers):
    tag = rel["release"][7:19]
    pp = prefixes(cases, src_rel, rel["inputs"], stop, cutoffs, out, tag)
    jobs = [(key, rel["release"], p, out / "artifact" / tag / p.name) for key, p in pp.items()]
    arts = sc.forward_cases(jobs, workers)
    if rel["inputs"] == "f2":  # 13.1: a helium-diluted test is a refusal of the registration (F2 has no diluent input)
        byt = {c["test"]: c for c in cases}
        arts = {k: (a if not sc.is_helium(byt[k[0]]) else dict(a, support=dict(admitted=False, gate="condition",
                    reasons=["registration (13.1): helium dilution; F2 has no diluent input"]))) for k, a in arts.items()}
    return arts


def score_block(releases, cases, train_cases, src_rel, out, workers, with_outcomes=True):
    """Per release, stratum and path: answered and refused counts and, with outcomes, the sum of the release and of
    both incidence baselines on the cases it answers."""
    res = {}
    for stop in (False, True):
        path = "stop" if stop else "nostop"
        keys_all = [(k["test"], k["cutoff"]) for k in sc.at_risk_cases(cases)] if with_outcomes else \
                   [(c["test"], k) for c in cases for k in sc.CUTOFFS]
        for rel in releases:
            arts = run_release(rel, cases, src_rel, out, stop, keys_all, workers)
            for st in STRATA:
                sub = [c for c in cases if stratum(c) == st]
                tests = {c["test"] for c in sub}
                ks = [k for k in keys_all if k[0] in tests]
                if not ks:
                    continue
                fc = {k: ta.cif_of(arts[k]) for k in ks if ta.admitted(arts[k])}
                refused = Counter(" ".join(arts[k]["support"].get("reasons", ["?"]))[:120] for k in ks if not ta.admitted(arts[k]))
                row = dict(cases=len(ks), tests=len(tests), answered_cases=len(fc), answered_tests=len({k[0] for k in fc}),
                           refused_cases=sum(refused.values()), refusal_reasons=dict(refused))
                if with_outcomes and fc:
                    keys = sorted(fc)
                    b_cfig, b_inc = sc.baselines_for(keys, train_cases)
                    for n, f in ((rel["label"], fc), ("CFI-G train incidence", b_cfig), ("s-Flame train incidence", b_inc)):
                        con, kk = sc.contrib_on(keys, f, sub)
                        m = sc.means_by(con, kk, lambda k: True)
                        row[n] = dict(sum_case_mean=m["sum"]["case_mean"], sum_day_mean=m["sum"]["day_mean"],
                                      extinction=m["extinction"]["case_mean"], termination=m["operator_termination"]["case_mean"])
                res.setdefault(rel["label"], {}).setdefault(st, {})[path] = row
    return res


def releases_of(lst):
    return [lst["model"]] + lst["comparators"]


def cmd_template(_a):
    LIST.write_text(json.dumps(template(), indent=1) + "\n", encoding="utf-8")
    print(f"wrote {LIST.relative_to(REPO).as_posix()}")
    return 0


def cmd_inputs_check(a):
    lst = json.loads(Path(a.releases).read_text(encoding="utf-8"))
    out = build("inputs-check")
    rows = [r for r in sc.read_jsonl(out / "inputs.jsonl") if r["part"] == "validation"]
    assert all(r["day"] in TEST_DAYS for r in rows), "inputs-check reads the test days only"
    cases = [sc.case_row(r) for r in rows]
    target = [c for c in cases if c["in_target_set"]]
    by = Counter((c["inputs"]["fuel_species"], sc.diluent_text(c),
                  "O2 known" if c["inputs"].get("x_o2_role") is not None else
                  "two O2 candidates" if len(c.get("inputs_by_o2_candidate") or {}) == 2 else "no O2") for c in target)
    print(f"test days {', '.join(TEST_DAYS)}: {len(cases)} tests with inputs, {len(target)} constant-flow (target set)")
    for k, n in sorted(by.items()):
        print(f"  {k[0]} in {k[1]}, {k[2]}: {n} tests")
    sc.write_jsonl(out / "cases_test_inputs.jsonl", target)
    src_rel = (out / "cases_test_inputs.jsonl").relative_to(REPO).as_posix()
    res = score_block(releases_of(lst), target, [], src_rel, out, a.workers, with_outcomes=False)
    rep = dict(purpose="final run, s-Flame: test-day INPUTS only (no endpoint measured, no outcome read)",
               tests=len(cases), target_tests=len(target), by_fuel_diluent={" | ".join(k): n for k, n in by.items()},
               admission={lab: {st: {p: dict(answered_tests=r["answered_tests"], tests=r["tests"], refusal_reasons=r["refusal_reasons"])
                                     for p, r in d.items()} for st, d in v.items()} for lab, v in res.items()})
    ta.write_json(ROOT / "inputs-check" / "sflame_admission.json", rep)
    for lab, v in res.items():
        print(f"{lab}: " + "; ".join(f"{st} {d['nostop']['answered_tests']}/{d['nostop']['tests']}" for st, d in v.items()))
    return 0


def cmd_rehearse(a):
    lst = json.loads(Path(a.releases).read_text(encoding="utf-8"))
    out = build("rehearse")
    src_rel = (out / "cases_validation_inputs.jsonl").relative_to(REPO).as_posix()
    res = {}
    for version in ("v1", "v2"):
        train, val = load(out, version)
        res[version] = score_block(releases_of(lst), val, train, src_rel, out, a.workers)
    bad = 0
    for (lab, st, path), (want, n) in REGISTERED.items():
        row = res["v1"].get(lab, {}).get(st, {}).get(path, {})
        got = round(row.get(lab, {}).get("sum_case_mean", float("nan")), 4)
        ok = got == want and row.get("answered_cases") == n
        bad += not ok
        print(f"{'ok ' if ok else 'DIFFERS'} v1 {lab}, {st}, {path}: {got} on {row.get('answered_cases')} cases (registered {want} on {n})")
    print("version 2 (reported, nothing registered on it):")
    for lab, v in res["v2"].items():
        for st, d in v.items():
            for path, r in d.items():
                if lab in r:
                    print(f"   v2 {lab}, {st}, {path}: {r[lab]['sum_case_mean']:.4f} on {r['answered_cases']} cases "
                          f"(CFI-G inc {r['CFI-G train incidence']['sum_case_mean']:.4f}, s-Flame inc {r['s-Flame train incidence']['sum_case_mean']:.4f})")
    ta.write_json(ROOT / "rehearse" / "sflame_rehearsal.json", dict(purpose="rehearsal on the VALIDATION days", scores=res,
                                                                     reproduced_v1=bad == 0))
    print("rehearsal reproduces the registered numbers on version 1" if bad == 0 else f"{bad} registered numbers not reproduced")
    return 1 if bad else 0


def reveal_line(artifact_id, run_id, purpose):
    e = {"artifact_id": artifact_id, "at": datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%S.%fZ"),
         "final_test": True, "granted": True, "host": os.environ.get("COMPUTERNAME") or os.environ.get("HOSTNAME") or "unknown",
         "partition": "test", "purpose": purpose, "run_id": run_id, "split_id": "docs/wsp/atlas/splits/PSI-23.json",
         "user": os.environ.get("USERNAME") or os.environ.get("USER") or "unknown"}
    return json.dumps(dict(sorted(e.items())), separators=(",", ":"))


def cmd_final(a):
    raw = Path(a.releases).read_bytes()
    lst = json.loads(raw)
    if hashlib.sha256(raw).hexdigest() != a.sha256:
        sys.exit("the release list's sha256 does not equal --sha256")
    if lst.get("frozen") is not True:
        sys.exit("the release list is not frozen")
    if not a.execute:
        sys.exit("refused: the final run needs --execute")
    if MARKER.exists():
        sys.exit(f"{MARKER} exists: the s-Flame step of the final run has been started once")
    MARKER.parent.mkdir(parents=True, exist_ok=True)
    MARKER.write_text(json.dumps(dict(started=datetime.datetime.now(datetime.timezone.utc).isoformat(), list_sha256=a.sha256)) + "\n")
    out = build("final")
    train, test = load(out, "v2")
    assert all(c["day"] in TEST_DAYS for c in test)
    src_rel = (out / "cases_validation_inputs.jsonl").relative_to(REPO).as_posix()
    res = score_block(releases_of(lst), test, train, src_rel, out, a.workers)
    with REVEAL_LOG.open("a", encoding="utf-8") as f:
        for p in sorted((out / "artifact").rglob("*.json")):
            art = json.loads(p.read_text(encoding="utf-8"))
            if ta.admitted(art):
                f.write(reveal_line(art.get("id") or p.stem, f"PSI-23:acme_normal:{p.stem.split('_')[0]}",
                                    "final run gen7, s-Flame (PROTOCOL 12.5)") + "\n")
    ta.write_json(ROOT / "final" / "sflame.json", dict(list_sha256=a.sha256, endpoints="v2 (sflame_endpoints_v2.md)", scores=res))
    L = ["# Final run, s-Flame (PSI-23) test days", "", f"Release list sha256 `{a.sha256}`; days {', '.join(TEST_DAYS)}; "
         "endpoints version 2 (docs/wsp/feedback/sflame_endpoints_v2.md; the s-Flame train incidence as registered, version 1); "
         "aggregates only; sum of the "
         "integrated Brier scores, case means.", "",
         "| release | stratum | path | cases answered / at risk | release | CFI-G train inc. | s-Flame train inc. |", "|---|---|---|---|---|---|---|"]
    for lab, v in res.items():
        for st, d in v.items():
            for path, r in d.items():
                g = lambda n: f"{r[n]['sum_case_mean']:.4f}" if n in r else "–"
                L.append(f"| {lab} | {st} | {path} | {r['answered_cases']} / {r['cases']} | {g(lab)} | {g('CFI-G train incidence')} | {g('s-Flame train incidence')} |")
    RESULTS_MD.write_text("\n".join(L) + "\n", encoding="utf-8")
    print(f"wrote {RESULTS_MD.relative_to(REPO).as_posix()}")
    return 0


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    sub = ap.add_subparsers(dest="cmd", required=True)
    for n, fn in (("template", cmd_template), ("inputs-check", cmd_inputs_check), ("rehearse", cmd_rehearse), ("final", cmd_final)):
        p = sub.add_parser(n)
        p.set_defaults(fn=fn)
        p.add_argument("--releases", default=str(LIST))
        p.add_argument("--sha256")
        p.add_argument("--execute", action="store_true")
        p.add_argument("--workers", type=int, default=12)
        p.add_argument("--ignis", default=default_binary("ignis.exe"))
    a = ap.parse_args()
    ta.EXE = (REPO / a.ignis).resolve()
    return a.fn(a)


if __name__ == "__main__":
    sys.exit(main())
