"""F3 stage 1: neutrality (carry vs F2) on stored F2 prefixes, and F3 against F2 on s-Flame train days.
No validation outcome is opened: validation-day prefixes are built from inputs only and compared as
forecasts; only train cases are scored."""
import copy
import json
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

MAIN = Path("D:/dev/SpaceApps-Ardealu")
WT = MAIN / ".claude/worktrees/f3"
sys.path.insert(0, str(MAIN / "tools/wsp/ext"))
import sflame_cases as sc  # noqa: E402  (main checkout copy: REPO = main; reads inputs, writes nothing here)
import transfer_acme as ta  # noqa: E402

EXE = str(WT / "target/release/ignis.exe")
ENV = dict(__import__("os").environ, IGNIS_ARTIFACTS=str(MAIN / "artifacts"))
OUT = MAIN / "runs/dev/gen7/f3"
F2 = "sha256:55fcf4cbbf500ca57a4e4207426826d5c3d3b21d84fb279479e29b7e5a65971d"
CARRY = "sha256:f2e28a3ec1c2a6accf4bc521c8644ce47846ac7e7e65b4409327bc2d70282ba3"
F3 = "sha256:d1660331c1f96091dad7f6f148c08f50db0cd03317b6c1dec458bd92696a45aa"
SETTINGS = json.dumps(ta.SETTINGS, separators=(",", ":"))


def forward(release, pp, op):
    if not op.exists():
        r = subprocess.run([EXE, "wsp", "forward", "--release", release, "--prefix", str(pp), "--settings", SETTINGS,
                            "--out", str(op)], capture_output=True, text=True, env=ENV)
        if r.returncode != 0:
            raise RuntimeError(r.stderr + r.stdout)
    return json.loads(op.read_text(encoding="utf-8"))


def with_limit(p, value):
    p = copy.deepcopy(p)
    m = copy.deepcopy(p["known_conditions"]["pressure"])
    m["raw_value"] = value
    m["value_si"] = value
    m["raw_unit"] = m["unit_si"] = "degC"
    m["derivation"] = dict(m.get("derivation") or {}, method_id="wsp.ext.acme_script2.burner_limit")
    p["known_conditions"]["burner_limit_c"] = m
    return ta.finish_prefix(p)


def run_set(name, prefixes, jobs_of):
    """prefixes: {key: prefix dict}; jobs_of(key, prefix) -> [(tag, release, prefix)]"""
    d = OUT / name
    (d / "prefix").mkdir(parents=True, exist_ok=True)
    (d / "artifact").mkdir(parents=True, exist_ok=True)
    jobs = []
    for key, p in prefixes.items():
        for tag, rel, q in jobs_of(key, p):
            pp = d / "prefix" / f"{key}_{tag}.json"
            pp.write_text(json.dumps(q), encoding="utf-8")
            jobs.append(((key, tag), rel, pp, d / "artifact" / f"{key}_{tag}.json"))
    with ThreadPoolExecutor(12) as ex:
        res = list(ex.map(lambda j: (j[0], forward(j[1], j[2], j[3])), jobs))
    return dict(res)


def neutrality(name, prefixes):
    arts = run_set(name, prefixes, lambda k, p: [("f2", F2, p), ("carry", CARRY, with_limit(p, 500.0))])
    n = same = admitted = 0
    diffs = []
    for k in prefixes:
        a, b = arts[(k, "f2")], arts[(k, "carry")]
        n += 1
        if a["support"]["admitted"] != b["support"]["admitted"]:
            diffs.append((k, "admission", a["support"].get("reasons"), b["support"].get("reasons")))
            continue
        if a["support"]["admitted"]:
            admitted += 1
            if a["event_cdf"] == b["event_cdf"] and a["times_s"] == b["times_s"]:
                same += 1
            else:
                diffs.append((k, "table"))
        else:
            same += 1
    return dict(set=name, prefixes=n, admitted_by_f2=admitted, identical=same, differences=diffs[:10], n_diff=len(diffs))


def main():
    rep = {}
    # s-Flame train: the stored F2 prefixes
    tp = {p.stem: json.loads(p.read_text(encoding="utf-8")) for p in sorted((MAIN / "runs/dev/data_all/work/transfer_sflame/f2_train/prefix").glob("*.json"))}
    rep["sflame_train"] = neutrality("neutral_sflame_train", tp)
    # s-Flame validation INPUTS (no outcome): every in-target test with an oxygen fraction, cutoffs 1, 3, 5
    src = "runs/dev/data_all/work/transfer_sflame/cases_validation_inputs.jsonl"
    rows = [json.loads(l) for l in (MAIN / src).read_text(encoding="utf-8").splitlines() if l.strip()]
    vp = {}
    for i, c in enumerate(rows):
        if not (c["in_target_set"] and c["inputs"].get("x_o2_role") is not None):
            continue
        for k in (1.0, 3.0, 5.0):
            for stop in (False, True):
                vp[f"{c['test']}_c{k:g}_{'stop' if stop else 'nostop'}"] = sc.f2_prefix(c, k, stop, "inputs-only", i + 1, src=src)
    rep["sflame_validation_inputs"] = neutrality("neutral_sflame_validation_inputs", vp)
    # Flame Design normal, every stored F2-era prefix of the recorded fuel (train and validation days; forecasts only)
    fdp = {p.stem: json.loads(p.read_text(encoding="utf-8")) for p in
           sorted((MAIN / "runs/dev/data_all/work/transfer_acme/fd_fuel/prefix/acme_normal").glob("*_recorded_*.json"))}
    rep["flame_design_normal"] = neutrality("neutral_flame_design", fdp)
    (OUT / "neutrality.json").write_text(json.dumps(rep, indent=1), encoding="utf-8")
    for v in rep.values():
        print(v)

    # F3 on the s-Flame train days, with the script's limit (an input): scored against F2 and the baselines
    allc = sc.read_jsonl(sc.OUT / "cases_train.jsonl")
    cases = [c for c in allc if c["in_target_set"]]
    byt = {c["test"]: c for c in cases}
    arts = run_set("f3_sflame_train", tp, lambda k, p: [
        ("f2", F2, p),
        ("f3", F3, with_limit(p, byt[k.split("_")[0]]["burner_limit_c"]) if byt.get(k.split("_")[0], {}).get("burner_limit_c") is not None else p)])
    cfig = json.loads(sc.CFIG_BASELINE.read_text(encoding="utf-8"))["families"]["acme_normal"]["cutoffs"]
    out = {}
    for path in ("nostop", "stop"):
        fc = {}
        for tag in ("f2", "f3"):
            fc[tag] = {}
            for k in tp:
                t, c, pth = k.split("_")
                if pth != path:
                    continue
                a = arts[(k, tag)]
                if a["support"]["admitted"]:
                    fc[tag][(t, float(c[1:]))] = ta.cif_of(a)
        common = set(fc["f2"]) & set(fc["f3"])
        res = {tag: sc.public(sc.score({k: v for k, v in fc[tag].items() if k in common}, cases)) for tag in fc}
        res["cfig_incidence"] = sc.public(sc.score({k: {cc: cfig[f"{k[1]:g}"][cc] for cc in ta.CAUSES} for k in common}, cases))
        inc = sc.incidence_from(cases)
        res["sflame_train_incidence"] = sc.public(sc.score({k: {cc: inc[k[1]][cc] for cc in ta.CAUSES} for k in common}, cases))
        res["answered"] = {tag: len(fc[tag]) for tag in fc}
        out[path] = res
        print(path, {t: (r["integrated_brier"] if isinstance(r, dict) and "integrated_brier" in r else r) for t, r in res.items()})
    (OUT / "f3_sflame_train.json").write_text(json.dumps(out, indent=1, default=str), encoding="utf-8")


main()
