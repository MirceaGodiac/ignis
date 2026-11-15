#!/usr/bin/env python3
"""What-if runs for the physicist of the burner flames: a child of a stored `acme_normal.wsp13` release with a few
changed lines, run on the s-Flame development days and checked for neutrality elsewhere, without a scratch registry
of one's own.

    python tools/wsp/ext/sflame_whatif.py --base configs/models/acme/wsp13/normal/f3-s1 \\
        --edits changed.wsp --note "methane hinge shift 0.35 -> 0.5: low-flow methane shorter"

What it does, in order:

1. Builds the child: the lines of `--edits` (the edit language, committee names, e.g. `b0.fuel.s[4] = -0.5`)
   compiled as a literal revision of the base release and assembled into a private registry under
   `runs/dev/gen7/whatif/<run>/registry` (the base release is copied there; the lane registry `artifacts/` is
   only read).
2. Runs base and child on the s-Flame development days (study data, PROTOCOL 13.6): every constant-flow test of the
   TRAIN and VALIDATION days that has an endpoint (the case files of `tools/wsp/ext/sflame_cases.py`), including hydrogen, blends and helium dilution wherever the release admits them; the
   test sets of each stratum (hydrocarbons in nitrogen, hydrocarbons in helium, hydrogen, blends) are scored as
   separate sets, as the looks of 13.1/13.5 (a) and of stage 2 were; `--compare NAME=ID` adds releases beside base and
   child (`:nodiluent` for one with no diluent input, refused on helium tests as 13.1 registered); `--outcomes
   coded,v2` gives every table on the coded endpoints (version 1) and on the data lane's second rule (version 2,
   docs/wsp/feedback/sflame_endpoints_v2.md), the second beside the first, never replacing it (`adjudicated`, the 16
   endpoints of phys_sflame_f4.md 2.3, stays available); cutoffs
   1, 3, 5 s, without and with the planned stop; and, since PROTOCOL 13.8, the TEST days 20142, 20170, 20176 as the
   final step built them (`runs/final/gen7/final/transfer_sflame/`) as a third set, with the F3 inputs (`sflame_cases.f3_prefix`: the script's burner
   limit, the diluent, the two oxygen candidates, the blend fraction).
3. Prints the integrated Brier sum (extinction + operator termination, 6 to 120 s) of base, child and the two
   incidence baselines, pooled and by fuel and diluent, by day, and by whether the child's burner reaches its
   limit before the planned stop (central heating node). Aggregates only.
4. Neutrality: the child against the base on CFI-G TRAIN (`wsp-dev feedback`, shared records) and on the Flame Design
   TRAIN days (the stored prefixes of 12.8 with the recorded fuel; forecasts only): how many forecast tables
   differ. CFI-G and Flame Design validation are never run.
5. Appends one line to `runs/dev/gen7/whatif_log.jsonl`: time, base id, child id, sha256 of the edits, the note,
   the pooled scores.
"""
import argparse
import datetime as dt
import hashlib
import json
import math
import os
import shutil
import subprocess
import sys
from collections import Counter, defaultdict
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import sflame_cases as sc  # noqa: E402
import transfer_acme as ta  # noqa: E402

REPO = ta.REPO
LOG = REPO / "runs/dev/gen7/whatif_log.jsonl"
FD_PREFIXES = REPO / "runs/dev/data_all/work/transfer_acme/fd_fuel/prefix/acme_normal"



def default_binary(name):
    """The merged build (target-merged/release, which carries every forward of the branch); this checkout's own
    target/release when there is none (a worktree)."""
    merged = REPO / "target-merged/release" / name
    return str((merged if merged.exists() else REPO / "target/release" / name).relative_to(REPO))

def sh(*a, env=None):
    r = subprocess.run([str(x) for x in a], capture_output=True, text=True, env=env, cwd=REPO)
    if r.returncode != 0:
        sys.exit(f"{' '.join(map(str, a))} failed:\n{r.stderr}{r.stdout}")
    return r.stdout


def build_child(base_dir, edits, work, exe):
    base = json.loads((base_dir / "release.json").read_text(encoding="utf-8"))
    assert base["model_name"] == "acme_normal.wsp13", "the base is an acme_normal.wsp13 release"
    reg = work / "registry"
    src = REPO / "artifacts/models" / base["id"][7:]
    dst = reg / "models" / base["id"][7:]
    if not dst.exists():
        shutil.copytree(src, dst)
    meta = dict(patch_id=f"whatif-{work.name}", parent_release_id=base["id"],
                author=dict(provider="anthropic", model_id="whatif"),
                exposure=dict(sources=["repo:tools/wsp/ext/sflame_whatif.py(what-if-child;not-a-candidate)"], datasets=[],
                              development_feedback=[], prior_candidates=[]),
                rationale="What-if child, not a candidate.", origin="system")
    (work / "meta.json").write_text(json.dumps(meta, indent=2), encoding="utf-8")
    shutil.copyfile(edits, work / "edits.wsp")
    env = dict(os.environ, IGNIS_ARTIFACTS=str(reg))
    sh(exe, "wsp", "compile", "--family", "acme_normal13", "--edits", work / "edits.wsp", "--meta", work / "meta.json",
       "--out", work / "patch.json", env=env)
    out = json.loads(sh(exe, "wsp", "assemble", "--family", "acme_normal13", "--patch", work / "patch.json",
                        "--binding", base_dir.parent / "binding.json", "--root", reg, env=env))
    return base["id"], out["release_id"], reg


def reaches_limit(a, stop):
    """Whether the first member's burner reaches a weighted limit (central heating node) before the planned stop
    (or 120 s)."""
    for m in (a.get("diagnostics") or {}).get("members") or []:
        cl = m.get("clocks") or {}
        cl = cl.get("clocks", cl)
        lim = cl.get("burner_limits")
        if lim:
            ts = [t for (_l, w, t) in lim if t is not None and w > 0]
            return bool(ts) and min(ts) < (stop or 120.0)
    return None


def run(cases, src_rel, tag, rel, work, workers, stop):
    src = REPO / src_rel
    src_sha = sc.sha(src)
    row_of = {json.loads(line)["test"]: i + 1 for i, line in enumerate(src.read_text(encoding="utf-8").splitlines()) if line.strip()}
    byt = {c["test"]: c for c in cases}
    jobs = []
    for k in sc.at_risk_cases(cases):
        name = f"{k['test']}_c{k['cutoff']:g}_{'stop' if stop else 'nostop'}.json"
        pp = work / "prefix" / tag / name
        ta.write_json(pp, sc.f3_prefix(byt[k["test"]], k["cutoff"], stop, src_sha, row_of[k["test"]], src_rel))
        jobs.append(((k["test"], k["cutoff"]), rel, pp, work / "artifact" / tag / rel[7:19] / name))
    return sc.forward_cases(jobs, workers)


STRATA = ("hydrocarbons in nitrogen", "hydrocarbons in helium", "hydrogen", "blends")
ADJUDICATED = HERE / "sflame_adjudicated_endpoints.json"


def stratum(c):
    fuel = c["inputs"]["fuel_species"] or ""
    if "+" in fuel:
        return "blends"
    if fuel == "hydrogen":
        return "hydrogen"
    return "hydrocarbons in helium" if sc.is_helium(c) else "hydrocarbons in nitrogen"


FINAL = REPO / "runs/final/gen7/final/transfer_sflame"


def test_days(outcomes):
    """The s-Flame test days 20142, 20170, 20176 as the final step built them (PROTOCOL 13.8: study data since the
    final run): its "validation" files are the test days. Their cases carry version-1 endpoints (its
    cases_validation_outcomes.jsonl); `with_outcomes(..., "v2")` takes its endpoints_v2.jsonl. Nothing is built here."""
    if not (FINAL / "cases_validation_inputs.jsonl").exists():
        return [], None
    vout = {r["test"]: r["endpoint"] for r in sc.read_jsonl(FINAL / "cases_validation_outcomes.jsonl")}
    cases = [c for c in sc.read_jsonl(FINAL / "cases_validation_inputs.jsonl") if c["in_target_set"]]
    assert all(c["day"] in ("20142", "20170", "20176") for c in cases)
    for c in cases:
        c["outcome"] = dict(endpoint=vout[c["test"]])
        c["final_v2"] = True
    return cases, (FINAL / "cases_validation_inputs.jsonl").relative_to(REPO).as_posix()


def with_outcomes(cases, version):
    """The cases with the endpoints of `version`: "coded" (version 1, as the case files hold them), "v2" (the general
    second rule of the data lane, sflame_endpoints_v2.md) or "adjudicated" (the 16 endpoints of phys_sflame_f4.md 2.3
    replaced, every other endpoint as coded)."""
    if version == "coded":
        return cases
    if version == "v2":
        # the data lane's general rule (docs/wsp/feedback/sflame_endpoints_v2.md), files beside version 1
        v2 = {r["test"]: r["endpoint"] for f in ("endpoints_train_v2.jsonl", "cases_validation_outcomes_v2.jsonl")
              for r in sc.read_jsonl(sc.OUT / f)}
        if any(c.get("final_v2") for c in cases):  # the test days: the final step's version-2 endpoints
            v2.update({r["test"]: r["endpoint"] for r in sc.read_jsonl(FINAL / "endpoints_v2.jsonl")})
        return [dict(c, outcome=dict(c["outcome"], endpoint=v2[c["test"]])) if c["test"] in v2 else c for c in cases]
    adj = {r["test"]: r for r in json.loads(ADJUDICATED.read_text(encoding="utf-8"))["endpoints"]}
    out = []
    for c in cases:
        r = adj.get(c["test"])
        if r is not None:
            c = dict(c, outcome=dict(c["outcome"], endpoint=dict(censoring="event", last_confirmed_active_s=r["end_s"],
                                                                 first_terminal_s=r["end_s"], allowed_causes=[r["cause"]])))
        out.append(c)
    return out


def with_version(cases, version):
    """`with_outcomes`, plus the case versions of PROTOCOL 13.8 (after the final run with every partition seen;
    in-sample), on the version-2 endpoints: "v3ramp" (rule 1 of sflame_cases.case_v3 only: a non-zero
    ignition-to-test ramp time is not a constant-flow test) and "v3" (rule 1 and rule 2, the oxygen fraction of the
    pressure block where it is determined). The cases a rule takes out of the constant-flow set are dropped here."""
    if version not in ("v3ramp", "v3"):
        return with_outcomes(cases, version)
    cases = with_outcomes(cases, "v2")
    inputs = {r["test"]: r for root in ((FINAL,) if any(c.get("final_v2") for c in cases) else (sc.OUT,))
              for r in sc.read_jsonl(root / "inputs.jsonl")}
    out = [sc.case_v3(c, inputs[c["test"]], oxygen=version == "v3") for c in cases]
    return [c for c in out if c["in_target_set"]]


def table(arts, child, cases, train_cases):
    """arts: {name: {key: artifact}}; scores on the cases the child answers. A release is shown in a group only when it
    answers every one of the child's cases there (its contributions are then computed on the same cases)."""
    byt = {c["test"]: c for c in cases}
    fc = {n: {k: ta.cif_of(a) for k, a in d.items() if ta.admitted(a)} for n, d in arts.items()}
    keys_child = sorted(fc[child])
    if not keys_child:
        return dict(answered={n: len(v) for n, v in fc.items()}, rows={})
    b_cfig, b_inc = sc.baselines_for(keys_child, train_cases)
    con = {"cfig_incidence": sc.contrib_on(keys_child, b_cfig, cases), "sflame_train_incidence": sc.contrib_on(keys_child, b_inc, cases)}
    for n, f in fc.items():
        if set(keys_child) <= set(f):
            con[n] = sc.contrib_on(keys_child, f, cases)
    keys = con[child][1]
    reach = {k: reaches_limit(arts[child][k], byt[k[0]]["inputs"].get("programmed_stop")) for k in keys}
    groups = {"pooled": lambda k: True}
    for g in sorted({(byt[k[0]]["inputs"]["fuel_species"], sc.diluent_text(byt[k[0]])) for k in keys}):
        groups[f"{g[0]} in {g[1]}"] = (lambda g: (lambda k: (byt[k[0]]["inputs"]["fuel_species"], sc.diluent_text(byt[k[0]])) == g))(g)
    for d in sorted({k[0][:5] for k in keys}):
        groups[f"day {d}"] = (lambda d: (lambda k: k[0][:5] == d))(d)
    groups["burner reaches its limit first (child)"] = lambda k: reach[k] is True
    groups["burner does not reach its limit first"] = lambda k: reach[k] is False
    # releases that answer only some of the child's cases: scored on their own common cases, group by group
    partial = {n: f for n, f in fc.items() if n not in con}
    rows = {}
    for g, sel in groups.items():
        r = {n: sc.means_by(con[n][0], keys, sel) for n in con}
        if not r[child]:
            continue
        row = dict(cases=r[child]["cases"], tests=r[child]["tests"], **{n: round(r[n]["sum"]["case_mean"], 4) for n in con})
        gkeys = [k for k in keys if sel(k)]
        for n, f in partial.items():
            row[n] = None if not all(k in f for k in gkeys) else "partial"
        rows[g] = row
    return dict(answered={n: len(v) for n, v in fc.items()}, rows=rows)


def show(title, t, names):
    print(f"\n== {title}: answered " + ", ".join(f"{n} {v}" for n, v in t["answered"].items()))
    for g, r in (t.get("rows") or {}).items():
        cols = "  ".join(f"{n} {r[n]:.4f}" if isinstance(r.get(n), float) else f"{n} -" for n in names)
        print(f"  {g:<44} {r['cases']:>4} cases {r['tests']:>3} tests | {cols}  CFI-G inc {r['cfig_incidence']:.4f}  "
              f"s-Flame inc {r['sflame_train_incidence']:.4f}")


def neutrality_cfig(base, child, reg, wspdev, work):
    out = work / "cfig_train.json"
    env = dict(os.environ, IGNIS_ARTIFACTS=str(reg))
    if "IGNIS_DATA" not in env and not (REPO / "data/work/records").exists():
        # a worktree: the ingested records live in the checkout that holds runs/ (a link here)
        env["IGNIS_DATA"] = str((REPO / "runs").resolve().parent / "data")
    sh(wspdev, "feedback", "--release", base, "--release", child, "--partition", "train",
       "--times", "6,8,10,12,15,20,25,30,40,50,60,80,100,120", "--n-boot", "2", "--out", out, env=env)
    r = json.loads(out.read_text(encoding="utf-8"))["releases"]
    a = {(c["run_id"], c["cutoff_s"]): c["metrics"] for c in r[0]["cases"]}
    b = {(c["run_id"], c["cutoff_s"]): c["metrics"] for c in r[1]["cases"]}
    ib = lambda x: {k: round(v["integrated_brier"], 4) for k, v in x["aggregate"]["events"]["causes"].items() if k != "unresolved_support"}
    return dict(cases=len(a), cases_moved=sum(a[k] != b.get(k) for k in a), base=ib(r[0]), child=ib(r[1]))


def neutrality_fd(base, child, work, workers):
    train_days = set(ta.FD_DAYS["acme_normal"]["train_days"])
    files = sorted(p for p in FD_PREFIXES.glob("*_recorded_*.json") if p.name[:5] in train_days)
    jobs = []
    for p in files:
        for tag, rel in (("base", base), ("child", child)):
            jobs.append(((p.stem, tag), rel, p, work / "artifact" / "fd" / tag / p.name))
    arts = ta.forward_many(jobs, workers)
    moved = sum(1 for p in files if (arts[(p.stem, "base")].get("event_cdf") != arts[(p.stem, "child")].get("event_cdf")))
    adm = Counter((ta.admitted(arts[(p.stem, "base")]), ta.admitted(arts[(p.stem, "child")])) for p in files)
    return dict(prefixes=len(files), tables_moved=moved, admitted_base_child=dict((f"{a}/{b}", n) for (a, b), n in adm.items()))


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--base", required=True, help="a release directory under configs/models/acme/wsp13/normal")
    ap.add_argument("--edits", required=True, help="the changed lines (edit language, committee names)")
    ap.add_argument("--note", required=True, help="the expected effect, stated before the run")
    ap.add_argument("--ignis", default=default_binary("ignis.exe"))
    ap.add_argument("--wsp-dev", default=default_binary("wsp-dev.exe"))
    ap.add_argument("--workers", type=int, default=12)
    ap.add_argument("--no-neutrality", action="store_true")
    ap.add_argument("--compare", action="append", help="NAME=sha256:...[:nodiluent], a release of the lane registry shown beside")
    ap.add_argument("--outcomes", default="coded", help="comma list of coded (v1), v2 (sflame_endpoints_v2.md), adjudicated (phys_sflame_f4.md 2.3), "
                    "v3ramp and v3 (case version 3 of PROTOCOL 13.8 on the v2 endpoints: ramped tests out; and oxygen by block)")
    a = ap.parse_args()
    exe, wspdev = (REPO / a.ignis).resolve(), (REPO / a.wsp_dev).resolve()
    edits = Path(a.edits).resolve()
    esha = hashlib.sha256(edits.read_bytes()).hexdigest()
    stamp = dt.datetime.now(dt.timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    work = REPO / "runs/dev/gen7/whatif" / f"{stamp}-{esha[:8]}"
    work.mkdir(parents=True, exist_ok=True)
    base, child, reg = build_child((REPO / a.base).resolve(), edits, work, exe)
    print(f"base {base}\nchild {child} (registry {reg.relative_to(REPO).as_posix()})\nnote: {a.note}")
    ta.EXE = exe
    os.environ["IGNIS_ARTIFACTS"] = str(reg)
    train_all = [c for c in sc.read_jsonl(sc.OUT / "cases_train.jsonl") if c["in_target_set"]]
    vin = sc.read_jsonl(sc.OUT / "cases_validation_inputs.jsonl")
    vout = {r["test"]: r for r in sc.read_jsonl(sc.OUT / "cases_validation_outcomes.jsonl")}
    val = [c for c in vin if c["in_target_set"]]
    for c in val:
        c["outcome"] = dict(endpoint=vout[c["test"]]["endpoint"])
    compare = {}
    for spec in a.compare or []:
        n, _, rest = spec.partition("=")
        rid, flag = (rest.rsplit(":", 1) if rest.count(":") == 2 else (rest, ""))
        d = reg / "models" / rid[7:]
        if not d.exists():
            shutil.copytree(REPO / "artifacts/models" / rid[7:], d)
        compare[n] = (rid, flag == "nodiluent")
    names = ["child", "base"] + list(compare)
    res = {}
    parts = [("train", train_all, "runs/dev/data_all/work/transfer_sflame/cases_train.jsonl"),
             ("validation", val, "runs/dev/data_all/work/transfer_sflame/cases_validation_inputs.jsonl")]
    test, test_src = test_days(a.outcomes)
    if test:
        parts.append(("test", test, test_src))
    for part, cases0, src in parts:
        for version in a.outcomes.split(","):
            cases = with_version(cases0, version)
            for stop in (False, True):
                tag = f"{part}_{'stop' if stop else 'nostop'}"
                arts = {"child": run(cases, src, tag, child, work, a.workers, stop),
                        "base": run(cases, src, tag, base, work, a.workers, stop)}
                for n, (rid, nodil) in compare.items():
                    arts[n] = run(cases, src, tag, rid, work, a.workers, stop)
                    if nodil:  # a release with no diluent input: refused on helium-diluted tests, as 13.1 registered
                        byt = {c["test"]: c for c in cases}
                        arts[n] = {k: (v if not sc.is_helium(byt[k[0]]) else dict(v, support=dict(admitted=False, reasons=["no diluent input"])))
                                   for k, v in arts[n].items()}
                # each stratum scored on its own cases (IPCW weights from that set), as the registered looks did
                for st in STRATA:
                    sub = [c for c in cases if stratum(c) == st]
                    if not sub:
                        continue
                    tests = {c["test"] for c in sub}
                    pick = {n: {k: v for k, v in d.items() if k[0] in tests} for n, d in arts.items()}
                    name = f"s-Flame {part} days, {st}, {'with' if stop else 'without'} the planned stop, {version} endpoints"
                    t = table(pick, "child", sub, train_all)
                    show(name, t, names)
                    res[name] = t
    neutral = {}
    if not a.no_neutrality:
        neutral["cfig_train"] = neutrality_cfig(base, child, reg, wspdev, work)
        neutral["flame_design_train_days"] = neutrality_fd(base, child, work, a.workers)
        print("\n== neutrality:", json.dumps(neutral))
    pooled = {k: (v.get("rows") or {}).get("pooled") for k, v in res.items()}
    compare_ids = {n: v[0] for n, v in compare.items()}
    line = dict(time=stamp, base=base, child=child, edits_sha256=esha, note=a.note, compare=compare_ids, pooled=pooled, neutrality=neutral,
                work=work.relative_to(REPO).as_posix())
    LOG.parent.mkdir(parents=True, exist_ok=True)
    with LOG.open("a", encoding="utf-8") as f:
        f.write(json.dumps(line) + "\n")
    (work / "report.json").write_text(json.dumps(dict(line, tables=res), indent=1), encoding="utf-8")
    print(f"\nlogged to {LOG.relative_to(REPO).as_posix()}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
