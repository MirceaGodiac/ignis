#!/usr/bin/env python3
"""The one final run of the seventh generation (PROTOCOL 12.5, 13.2 to 13.6): the hand-written laws of nine
investigations scored once on their unread TEST days, on a frozen release list, aggregates only.

    python -I tools/wsp/final/run_final_gen7.py template                       # writes the release-list template
    python -I tools/wsp/final/run_final_gen7.py inputs-check                   # test days: counts; inputs half where separable
    python -I tools/wsp/final/run_final_gen7.py inputs-models                  # test days: what each listed model would answer, inputs only
    python -I tools/wsp/final/run_final_gen7.py rehearse [--slug bre,...] [--coding v2]   # the identical code path on the VALIDATION days
    (slice: run with `python -P` instead of `-I`: its builder needs xlrd from the user site-packages)
    python -I tools/wsp/final/run_final_gen7.py final --slug bre --releases tools/wsp/final/releases_final_gen7.json \
        --sha256 <sha256 of the list> --execute

Code path (rehearsal and final alike), per slug:
  1. the lane's own case builder (tools/wsp/ext/<lane>_cases.py), unchanged, run with its partition map relabelled:
     rehearsal = the frozen map as it is; final = test days labelled "validation" and the real validation days left
     out. It writes inputs and outcomes in separate files under a local root that is never printed
     (runs/final/gen7/<mode>/transfer_<lane>/); the train files are rebuilt the same way; the baseline constants are
     the registered train constants (copied from runs/dev/data_all/work/transfer_<lane>/baselines_train.json).
  2. every listed release through the Rust forward (tools/wsp/law_run.py), the lane's scorer and the scoring-time
     rule of each release (tools/wsp/ext/law_score.py adapters; bre "lasts" is the rule of phys_bre2.md section 2);
     every baseline on the cases the model answers.
     The outcome coding or case version of a lane is the one the frozen list names (an investigation's "coding"; bass:
     "v2" = tools/wsp/ext/bass1_cases_v2.py, rules fixed before any test-day observation was read; slice: "v2" =
     tools/wsp/ext/slice_cases_v2.py, the tube from the picture table); a lane without the field is scored on its
     registered version 1. The rehearsal always scores the registered coding against the registered
     numbers and, when the list names another coding, also rebuilds that coding and compares it with its work files.
  3. aggregates only: rehearsal compares every number with the registered one; final writes
     docs/wsp/RESULTS_FINAL_GEN7.md and runs/final/gen7/final/<slug>.json, appends one line per scored forecast to
     runs/evaluation/reveal_log.jsonl, and creates runs/final/gen7/markers/<slug>.started before reading anything,
     so that a slug cannot run twice.

The final mode refuses unless the release list's sha256 equals --sha256, the list says "frozen": true, and --execute
is given. Nothing in this file opens a test-day file outside `final`.
"""
import argparse
import contextlib
import datetime
import hashlib
import io
import json
import math
import os
import shutil
import sys
from collections import Counter, defaultdict
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(REPO / "tools/wsp/ext"))
WORK = REPO / "runs/dev/data_all/work"
ROOT = REPO / "runs/final/gen7"
MARKERS = ROOT / "markers"
REVEAL_LOG = REPO / "runs/evaluation/reveal_log.jsonl"
RESULTS_MD = REPO / "docs/wsp/RESULTS_FINAL_GEN7.md"
LIST = REPO / "tools/wsp/final/releases_final_gen7.json"

# slug of the investigation -> (lane module, the lane's data directory, split description)
LANES = {"bre": ("bre_cases", "transfer_bre"), "cld": ("cld_cases", "transfer_cld"), "slice": ("slice_cases", "transfer_slice"),
         "bass": ("bass1_cases", "transfer_bass1"), "efield": ("efield_cases", "transfer_efield"),
         "spice": ("spice_cases", "transfer_spice"), "samer": ("samer_cases", "transfer_samer")}


def rid(slug, cand):
    return json.loads((REPO / f"configs/models/law/{slug}/{cand}/release.json").read_text(encoding="utf-8"))["id"]


def sha256_file(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest()


# ===================================================================================== the release list template
def entry(slug, cand, adapter=None, rule="lane", label=None):
    return dict(slug=slug, candidate=cand, release=rid(slug, cand), adapter=adapter or slug, rule=rule, label=label or f"{slug} {cand}")


def template():
    # the models and comparators decided under PROTOCOL 13.6 (coordinator, 15 November, after commit 976d14e)
    inv = {
        "bre": dict(investigation="PSI-20 BRE", lane="bre",
                    model=entry("bre", "d3-fable", rule="lasts", label="bre d3 by the lasts rule"),
                    comparators=[entry("bre", "w0-fable", rule="flame_end", label="bre W0 by flame_end (registered zero-shot reading)"),
                                 entry("bre", "w0-fable", rule="lasts", label="bre W0 by the lasts rule (phys_bre2.md section 2)"),
                                 entry("bre", "d1-fable", rule="flame_end", label="bre D1 by flame_end"),
                                 entry("bre", "d2-fable", rule="lasts", label="bre d2 by the lasts rule"),
                                 entry("bre", "d3m-fable", rule="lasts", label="bre d3m by the lasts rule")]),
        "cld": dict(investigation="PSI-21 CLD Flame", lane="cld", model=entry("cld3", "s1-fable", adapter="cld3", label="cld3 (third law)"),
                    comparators=[entry("cld", "w0-fable", label="cld W0"), entry("cld", "d1-fable", label="cld D1"),
                                 entry("cld", "c1-fable", label="cld C1"), entry("cld2", "s1-fable", adapter="cld2", label="cld2 (second law)")]),
        "efield": dict(investigation="PSI-22 E-FIELD Flames", lane="efield", model=entry("efield3", "s1-fable", adapter="efield3", label="efield3 (third law)"),
                       comparators=[entry("efield", "w0-fable", label="efield W0"), entry("efield", "e1-fable", label="efield E1"),
                                    entry("efield2", "s1-fable", adapter="efield2", label="efield2 (second law)")]),
        "slice": dict(investigation="PSI-106 SLICE", lane="slice", coding="v2",
                      model=entry("slice2", "s1-fable", adapter="slice2", label="slice2 s1 (second law)"),
                      comparators=[entry("slice", "w0-fable", label="slice W0"), entry("slice", "d1-fable", label="slice D1"),
                                   entry("slice2", "s2-fable", adapter="slice2", label="slice2 S2")]),
        "spice": dict(investigation="PSI-107 SPICE", lane="spice", model=entry("spice", "p3-fable", label="spice P3"),
                      comparators=[entry("spice", "p2-fable", label="spice P2"), entry("spice", "w0-fable", label="spice W0 (variant B)"),
                                   entry("spice", "lit-fable", label="spice lit (published fit; not blind)")]),
        "samer": dict(investigation="PSI-101 SAME-R", lane="samer", model=entry("samer", "s1-fable", label="samer S1"),
                      comparators=[entry("samer", "w0-fable", label="samer W0"), entry("samer", "d1-fable", label="samer D1")]),
        "bass": dict(investigation="PSI-26 BASS and PSI-25 BASS-II plastics and fabrics", lane="bass", coding="v2",
                     model=entry("bass4", "s1-fable", adapter="bass4", label="bass4 (fabrics in kept burning)"),
                     comparators=[entry("bass3", "s1-fable", adapter="bass3", label="bass3"), entry("bass3", "q1-fable", adapter="bass3", label="bass3 Q1"),
                                  entry("bass3", "os1-fable", adapter="bass3", label="bass3 S1 (os1)"),
                                  entry("bass2", "k1-fable", adapter="bass2", label="bass2 K1"), entry("bass2", "s1-fable", adapter="bass2", label="bass2 s1"),
                                  entry("bass1", "w0-fable", label="bass1 W0"), entry("bass1", "d1-fable", label="bass1 D1")]),
    }
    for v in inv.values():
        v["baselines"] = "every baseline law_score.py builds for the lane (registered train constants), scored on the cases the model answers"
    out = dict(schema="ignis.wsp.final-release-list.gen7/1", frozen=False,
               status="UNFROZEN, filled with the decisions of 15 November (PROTOCOL 13.6, after commit 976d14e); the coordinator freezes it (sets frozen true and commits)",
               scoring=("each entry's scoring-time rule: lane = the registered mapping of tools/wsp/ext/law_score.py; flame_end / lasts for bre; "
                        "event targets: the sum over every declared cause; coding: the plastics outcome coding (bass) and the SLICE case version "
                        "(slice), version 1 where the field is absent; a release that refuses a case (bass: the fabric burns in kept burning "
                        "for every release before bass4) counts it as a refusal, never as a forecast"),
               investigations=inv, same_and_daft="no test days (held whole; PROTOCOL 13.4)")
    LIST.write_text(json.dumps(out, indent=1) + "\n", encoding="utf-8", newline="\n")
    print("template ->", LIST.relative_to(REPO), "sha256", sha256_file(LIST))
    return 0


# ===================================================================================== building cases with the lane's builder
def relabel(p, mode):
    if mode == "rehearsal":
        return p
    return {"test": "validation", "validation": "held"}.get(p, p)


def build(lane, mode, root, coding="v1"):
    """Run the lane's own builder with its partition map relabelled, into root/<lane dir>. Its prints are captured
    into root/_logs/<lane>.log (never printed)."""
    import importlib
    mod_name, d = LANES[lane]
    M = importlib.import_module(mod_name)
    M = importlib.reload(M)
    out = root / d
    out.mkdir(parents=True, exist_ok=True)
    log = io.StringIO()
    if lane in ("bre", "efield", "cld"):
        orig = M.partitions
        M.partitions = lambda: {k: relabel(v, mode) for k, v in orig().items()}
        M.OUT = out
        run = lambda: M.cmd_build(None)  # noqa: E731
    elif lane == "spice":
        orig = M.split_map
        M.split_map = lambda: {k: relabel(v, mode) for k, v in orig().items()}
        M.OUT = out
        run = M.main
    elif lane == "slice":
        orig = M.split_map
        M.split_map = lambda: {k: relabel(v, mode) for k, v in orig().items()}
        M.OUT = out
        if mode == "final":
            # the log pages of the test days (NEVER_OPENED of the lane) take the place of the validation days; GMT 055
            # is held by the split's rule and stays unread
            parts = {g.split(":")[-1]: p for g, p in orig().items()}
            dates = {"044": "2012-02-13", "051": "2012-02-20", "083": "2012-03-23"}
            new = {k: v for k, v in M.DAYS.items() if parts.get(v[0]) == "train"}
            for day, pages in M.NEVER_OPENED.items():
                if day in dates and parts.get(dates[day]) == "test":
                    new[day] = (dates[day], pages)
            M.DAYS = dict(sorted(new.items()))
            M.NEVER_OPENED = {}
        if coding == "v2":
            # the v2 builder reads the v1 files just written and rewrites the tube where the picture table holds the flame
            import slice_cases_v2 as M2
            M2 = importlib.reload(M2)
            M2.OUT = out
            # final_run extends the tube rule to the test days' picture folders; the builder refuses on its own if
            # test-day cases arrive without the flag or a test-day folder is missing (slice_tube_check.md, addendum 2)
            run = lambda: (M.main(), M2.main(final_run=(mode == "final")))  # noqa: E731
        else:
            run = M.main
    elif lane == "bass":
        orig = M.partition_of
        M.partition_of = lambda inv, date: relabel(orig(inv, date), mode)
        M.OUT = out
        if coding == "v2":
            # the v2 builder reuses the v1 readers and partition map (patched above) and writes cases_*_v2.jsonl
            import bass1_cases_v2 as M2
            M2 = importlib.reload(M2)
            M2.v1 = M
            M2.OUT = out
            run = lambda: (M.cmd_build(None), M2.cmd_build(None))  # noqa: E731
        else:
            run = lambda: M.cmd_build(None)  # noqa: E731
    elif lane == "samer":
        orig = M.catalogue_groups
        M.catalogue_groups = lambda: {k: (g, relabel(p, mode)) for k, (g, p) in orig().items()}
        M.WORK = root
        run = M.build
    else:
        raise SystemExit(lane)
    with contextlib.redirect_stdout(log):
        run()
    (root / "_logs").mkdir(parents=True, exist_ok=True)
    (root / "_logs" / f"{lane}.log").write_text(log.getvalue(), encoding="utf-8")
    # the registered train constants of the baselines, unchanged
    for f in ("baselines_train.json", "baseline_smoke_number_per_mass.json") + (("baselines_v2.json",) if coding == "v2" else ()):
        src = WORK / d / f
        if src.exists():
            shutil.copyfile(src, out / f)
    return out


# ===================================================================================== scoring
def bre_lasts(S, ctx, release):
    """phys_bre2.md section 2 through the Rust forward: extinction by h = 1 - lasts(min(h, stop)) / lasts(c);
    operator termination from the stop = lasts(stop) / lasts(c)."""
    import bre_cases as L
    import law_inputs as LI
    import law_score as LS
    b = json.loads((LS.WORK / "transfer_bre/baselines_train.json").read_text(encoding="utf-8"))["t1"]["planned_stop"]["constants"]
    delta, onset = b["stop_minus_planned_median_s"], b["onset_minus_stamp_median_s"]
    law = {r["case_id"]: r for r in LI.law_rows("bre", ctx["rows"])}
    rows, keys = [], []
    for k in ctx["cases"]:
        ps = ctx["inputs"][k["test"]].get("planned_stop_s")
        st = None if ps is None else ps + delta - onset
        st = st if (st is not None and st > k["cutoff_s"]) else None
        c = max(k["cutoff_s"], 1.0)
        for t in sorted(set([c] + [min(h, st) if st else h for h in L.HORIZONS] + ([st] if st else []))):
            rows.append(dict(case_id=f"{k['test']}|{k['cutoff_s']}|{t}", inputs=dict(law[k["test"]]["inputs"], stated_time=t)))
        keys.append((k, st, c))
    res = LS.run_law("bre", release, rows, "lasts", tag=ctx["part"] + "_lasts")
    fc = []
    for k, st, c in keys:
        g = lambda t: res[f"{k['test']}|{k['cutoff_s']}|{t}"]["lasts"]  # noqa: E731
        if not g(c)["admitted"]:
            continue
        lc = g(c)["probability"]
        fc.append(dict(test=k["test"], cutoff_s=k["cutoff_s"], cif=dict(
            extinction=[min(1.0, max(0.0, 1 - g(min(h, st) if st else h)["probability"] / lc)) for h in L.HORIZONS],
            operator_termination=[(min(1.0, g(st)["probability"] / lc) if (st and h >= st) else 0.0) for h in L.HORIZONS])))
    ctx["refusals"] = {"t1": {}}
    ctx["n_asked"] = {"t1": len(keys)}
    return {"t1": fc}


def score_investigation(key, inv, data_root, part="validation", coding=None):
    """Every listed release and every baseline, on `part` cases of the lane data under data_root, on the outcome
    coding `coding` (default: the list's). Aggregates only."""
    import importlib
    import law_score as LS
    import efield_cases
    LS.WORK = data_root
    efield_cases.OUT = data_root / "transfer_efield"
    coding = coding or inv.get("coding", "v1")
    LS.BASS_CODING = coding if inv["lane"] == "bass" else "v1"
    LS.SLICE_CASES = coding if inv["lane"] == "slice" else "v1"
    out = {"coding": coding, "releases": {}, "baselines": {}, "_fc": {}}   # _fc: in memory only (rehearsal), never written
    model_fc = None
    entries = [("model", inv["model"])] + [(f"comparator {i + 1}", e) for i, e in enumerate(inv["comparators"])]
    for role, e in entries:
        S = LS.SLUGS[e["adapter"]]()
        ctx = S.load(part)
        fc = bre_lasts(S, ctx, e["release"]) if (e["adapter"] == "bre" and e["rule"] == "lasts") else S.release(ctx, e["release"])
        res = S.score(ctx, fc)
        mets = {f"{t}: {lab}": LS.get(res.get(t), path) for t, ms in S.metrics.items() for lab, path in ms if t in res}
        answered = {t: (sum(len(v) for v in fc[t].values()) if (e["adapter"].startswith("efield") and t == "t1") else len(fc[t])) for t in fc}
        out["releases"][e["label"]] = dict(role=role, release=e["release"], rule=e["rule"], metrics=mets, answered=answered,
                                           asked=ctx.get("n_asked"), refusals=ctx.get("refusals"))
        out["_fc"][e["label"]] = (S, ctx, fc)
        if role == "model":
            model_fc, model_S, model_ctx = fc, S, ctx
            for n, f in S.baselines(ctx).items():
                r = S.score(ctx, S.restrict(f, fc))
                out["baselines"][n] = {f"{t}: {lab}": LS.get(r.get(t), path) for t, ms in S.metrics.items() for lab, path in ms if t in r}
    return out


# ===================================================================================== rehearsal
def registered_numbers():
    """The registered validation numbers of 13.2 and 13.3, by release id and metric."""
    reg = defaultdict(dict)
    for p in ("runs/dev/gen7/look_summaries_validation.jsonl",):
        for l in (REPO / p).read_text(encoding="utf-8").splitlines():
            d = json.loads(l)
            if d.get("partition") in ("validation", "held"):
                for k, v in d["readings"].items():
                    reg[d["release"]][k] = v["release"]
    for l in (REPO / "runs/dev/gen7/d1_summaries.jsonl").read_text(encoding="utf-8").splitlines():
        d = json.loads(l)
        for k, v in d["targets"].items():
            reg[d["d1"]][k] = v["d1_val"]
    return reg


def cmd_rehearse(a):
    lst = json.loads(LIST.read_text(encoding="utf-8"))
    keys = [k for k in (a.slug.split(",") if a.slug else lst["investigations"])]
    reg = registered_numbers()
    expected = json.loads((REPO / "tools/wsp/final/expected_dev_gen7.json").read_text(encoding="utf-8"))
    report = {}
    extra = [a.coding] if getattr(a, "coding", "") else []
    for key, coding in [(k, c) for k in keys for c in dict.fromkeys(["v1", lst["investigations"][k].get("coding", "v1")] + extra)]:
        inv = lst["investigations"][key]
        lane = inv["lane"]
        root = ROOT / "rehearsal"
        build(lane, "rehearsal", root, coding)
        got = score_investigation(key, inv, root, coding=coding)
        ref = score_investigation(key, inv, WORK, coding=coding)       # the case files the looks of 13.2 to 13.6 were scored on
        rows, ok = [], True
        for label, r in got["releases"].items():
            for m, v in r["metrics"].items():
                w = ref["releases"][label]["metrics"].get(m)
                # a registered number belongs to a release scored by its registered rule (bre W0 by lasts shares W0's id)
                # and on the registered coding (v1)
                g = reg.get(r["release"], {}).get(m) if (r["rule"] in ("lane", "flame_end") and coding == "v1") else None
                same = (v is None and w is None) or (v is not None and w is not None and abs(v - w) <= 1e-9)
                same_reg = g is None or (v is not None and abs(v - g) <= 1e-9)
                ok &= same and same_reg
                rows.append(dict(release=label, metric=m, rebuilt=v, scored_files=w, registered=g, equal=same and same_reg))
        for n, mm in got["baselines"].items():
            for m, v in mm.items():
                w = ref["baselines"][n].get(m)
                same = (v is None and w is None) or (v is not None and w is not None and abs(v - w) <= 1e-9)
                ok &= same
                rows.append(dict(release=f"baseline: {n}", metric=m, rebuilt=v, scored_files=w, registered=None, equal=same))
        # the validation numbers of every release newer than 13.3, from its own development result file
        # (tools/wsp/final/expected_dev_gen7.json), through this code path, on the same cases as that file
        import law_score as LS
        for x in [x for x in expected if x["investigation"] == key and x["coding"] == coding]:
            S, ctx, fc = got["_fc"][x["label"]]
            for c in x["common_with"]:
                if c != x["label"]:
                    fc = S.restrict(fc, got["_fc"][c][2])
            res = S.score(ctx, fc)
            t, m = x["metric"].split(": ", 1)
            path = dict(S.metrics[t])[m]
            v = LS.get(res.get(t), path)
            same = v is not None and abs(v - x["expected"]) <= 1e-9
            ok &= same
            rows.append(dict(release=x["label"], metric=x["metric"], rebuilt=v, scored_files=None, registered=None, expected_dev=x["expected"],
                             common_with=x["common_with"], source=x["source"], equal=same))
        report[key if coding == "v1" else f"{key} (coding {coding})"] = dict(ok=ok, coding=coding, rows=rows)
        print(f"== {key}, coding {coding}{' (registered)' if coding == 'v1' else ''}: {'REPRODUCED' if ok else 'DIFFERENCES'}")
        for r in rows:
            print(f"   {'ok ' if r['equal'] else 'XX '} {r['release'][:52]:52s} {r['metric'][:48]:48s} rebuilt {r['rebuilt']} files {r['scored_files']} "
                  f"registered {r['registered']}" + (f" expected (dev file) {r['expected_dev']}" if "expected_dev" in r else ""))
    (ROOT / "rehearsal").mkdir(parents=True, exist_ok=True)
    (ROOT / "rehearsal" / "rehearsal_report.json").write_text(json.dumps(report, indent=1, default=str), encoding="utf-8")
    return 0 if all(v["ok"] for v in report.values()) else 1


# ===================================================================================== test days: counts and the input half
def split_days():
    def g(path, key="strata"):
        d = json.loads((REPO / path).read_text(encoding="utf-8"))
        return d["strata"]["all"]["groups"] if key == "strata" else d["groups"]
    return {"bre": ("PSI-20", g("docs/wsp/atlas/splits/PSI-20.json")["test"]),
            "cld": ("PSI-21", g("docs/wsp/atlas/splits/PSI-21.json")["test"]),
            "slice": ("PSI-106", g("docs/wsp/atlas/splits/PSI-106.json")["test"]),
            "efield": ("PSI-22", g("docs/wsp/atlas/splits/PSI-22.json")["test"]),
            "spice": ("PSI-107", g("docs/wsp/atlas/splits/PSI-107.json")["test"]),
            "samer": ("PSI-101", g("docs/wsp/atlas/splits/PSI-101.json")["test"]),
            "bass": ("PSI-26 / PSI-25", g("docs/wsp/atlas/splits/PSI-26-days.json", "groups")["test"]
                     + g("docs/wsp/feedback/bass1_PSI-26-2012-days.json", "groups")["test"]
                     + g("docs/wsp/atlas/splits/PSI-25-solids-days.json", "groups")["test"])}


def cmd_inputs_check(a):
    """Test days by the frozen splits; tests in the catalogue; the inputs half where the lane's inputs come from files
    of their own (bre: script, data header line and calculator start columns; cld: script; efield: script and data
    header line). No outcome of a test day is read."""
    cat = [json.loads(l) for l in (WORK / "atlas/tests.jsonl").read_text(encoding="utf-8").splitlines() if l.strip()]
    res = {}
    for lane, (inv, groups) in split_days().items():
        days = [x.split(":")[-1] for x in groups]
        tests = [t for t in cat if any(t["investigation_id"] == i for i in inv.split(" / ")) and
                 (t.get("group_id") in groups or (t.get("group_id") or "").split(":")[-1] in days)]
        r = dict(investigation=inv, test_days=days)
        r["test_cases_by_the_lane"] = {
            "spice": "90 flow-setting rows, 25 flames on the test days (catalogue, spice_feasibility.md section 3); the smoke points among them are known only from the review sheet",
            "samer": "34 rows (19 runs + 15 aged rows) in the catalogue (samer_feasibility.md section 5)",
            "slice": "3 test days; their log pages were never opened, so no per-line count exists (slice_feasibility.md section 3)",
            "bass": "48 burn rows on the test days (PSI-26 2012: 15, 2013: 11; PSI-25 solids: 22), counted from dates (bass1_feasibility.md section 4)",
        }.get(lane, "see inputs_half")
        if lane == "bre":
            import bre_cases as M
            part = M.partitions()
            keep = {d for d, p in part.items() if p == "test"}
            catf = M.catalogue_tests()
            cond = M.day_conditions()
            calc, dup = M.calculator_rows(keep)
            st = Counter()
            for test in sorted(catf):
                if part.get(test[:5]) != "test":
                    continue
                fls = M.files_of(test)
                st["tests"] += 1
                st["data file on disk"] += fls["data"] is not None
                st["script on disk"] += fls["script"] is not None
                if test[:5] not in cond:
                    st["no day conditions"] += 1
                    continue
                inp = M.inputs_of(test, fls, cond[test[:5]], calc, dup)
                req = ["fuel_mass_flow_kg_s", "pressure_pa", "x_o2_ambient", "burner_diameter_m", "planned_stop_s"]
                st["inputs complete"] += all(inp.get(k) is not None for k in req)
                st["no calculator row"] += "no_calculator_row" in inp["flags"]
                st["no script"] += "no_script" in inp["flags"]
                xf = inp.get("x_fuel_burner_stream")
                st["x_fuel below the law's 0.3"] += xf is not None and xf < 0.3
                st["methane"] += inp.get("fuel_species") == "methane"
            r["inputs_half"] = dict(st)
        elif lane == "cld":
            import cld_cases as M
            part = M.partitions()
            days_t = M.day_table()
            names = {p.name for p in M.RAW.iterdir()}
            files = M.test_files(names)
            st = Counter()
            for t, fn in files.items():
                if part.get(t[:5]) != "test":
                    continue
                st["test points with files"] += 1
                if t[:5] not in days_t:
                    st["day not in the experiment table"] += 1
                    continue
                inp, s = M.inputs_of(t, fn, days_t[t[:5]])
                st["with a script"] += bool(s)
                st["script incomplete"] += bool(inp.get("script_missing_keys"))
                seg = inp.get("schedule_from_ignition_command") or []
                st["no constant-flow stage (refused)"] += bool(seg) and not any(x["stage"] == "constant_flow" for x in seg)
                st["dilutes"] += (inp.get("x_fuel_final") or 1) < (inp.get("x_fuel_initial") or 1)
                st["velocity ramps"] += any(x["stage"].startswith("ramp_") for x in seg)
            r["inputs_half"] = dict(st)
            r["note"] = "the law's window and stop need the flame onset (at_cutoff), which the builder measures from the record: known only in the final run"
        elif lane == "efield":
            import efield_cases as M
            part = M.partitions()
            st = Counter()
            for test in sorted(M.catalogue_tests()):
                if part.get(test[:5]) != "test":
                    continue
                st["tests"] += 1
                sp = M.RAW / f"{M.PRE}{test}_VoltageSweep.txt"
                dp = M.RAW / f"{M.PRE}{test}_data.csv"
                st["script on disk"] += sp.exists()
                st["data file on disk"] += dp.exists()
                if sp.exists() and dp.exists():
                    hdr = M.read_header(dp)
                    pol = M.POL.get(hdr.get("EFieldPol", ""), "unknown")
                    st["polarity known"] += pol != "unknown"
                    if pol != "unknown":
                        pl = M.plan(M.read_script(sp), pol)
                        st["planned levels"] += len(pl["levels"])
                        st["levels below 100 V (refused)"] += sum(1 for lv in pl["levels"] if abs(lv["v_cmd"]) < 100)
            r["inputs_half"] = dict(st)
        else:
            r["inputs_half"] = ("not run: in this lane the inputs and the outcome of a test sit in one record "
                                + {"spice": "(a row of the smoke-point review sheet)", "samer": "(a row of the results workbook)",
                                 "slice": "(a line of the crew log, with its X marks)",
                                 "bass": "(a row of the as-run matrix, with the observation words)"}[lane]
                                + "; the record is read only in the final run")
        res[lane] = r
        print(json.dumps({lane: r}, default=str))
    (ROOT / "inputs_check.json").parent.mkdir(parents=True, exist_ok=True)
    (ROOT / "inputs_check.json").write_text(json.dumps(res, indent=1, default=str), encoding="utf-8")
    return 0


# ===================================================================================== test days: what each listed model would answer
def cmd_inputs_models(a):
    """From inputs only (no outcome): how many test-day cases each listed MODEL would answer and refuse, with the
    forward's reasons, where the lane's inputs come from files of their own (bre: script, data header line and
    calculator start columns; cld: script and experiment table; efield: script and data header line). The data
    files are read for their first (header) line only. cld: the flame onset (at_cutoff) is measured from the record and
    is known only in the final run; for this count the onset is taken as 0 s, so `window` and `t_severe` are the
    planned times from the ignition command (an upper bound by the onset, about a second); no forecast is used.
    spice, samer, slice, bass: inputs and outcome sit in one record (a review-sheet row, a workbook row, a crew-log
    line, an as-run row) that is read only in the final run, so no count is possible before it."""
    import law_inputs as LI
    import law_score as LS
    lst = json.loads(LIST.read_text(encoding="utf-8"))
    res = {}

    def tally(slug, rid, rows, target):
        r = LS.run_law(slug, rid, rows, target, times=[60.0] if target in ("loss", "flame_end") else None, tag="testdays_inputs_only") if rows else {}
        ok = sum(1 for v in r.values() if v[target]["admitted"])
        def reason(rs):   # one line per refused case: the features that are missing, or outside the release's domain
            miss = sorted(x.split(":")[0].replace("feature ", "") for x in rs if "missing" in x)
            out_ = sorted(x.split(" =")[0].replace("feature ", "") for x in rs if "outside" in x)
            return "; ".join(([f"missing: {', '.join(miss)}"] if miss else []) + ([f"outside the domain: {', '.join(out_)}"] if out_ else [])) or "; ".join(rs)
        why = Counter(reason(v[target]["reasons"]) for v in r.values() if not v[target]["admitted"])
        return dict(asked=len(rows), answered=ok, refused=len(rows) - ok, reasons=dict(why))

    LS.WORK = ROOT / "inputs_only"
    for key in ("bre", "cld", "efield"):
        inv = lst["investigations"][key]
        m = inv["model"]
        if key == "bre":
            import bre_cases as M
            part = M.partitions()
            cond = M.day_conditions()
            calc, dup = M.calculator_rows({d for d, p in part.items() if p == "test"})
            rows, no_cond = [], 0
            for test in sorted(M.catalogue_tests()):
                if part.get(test[:5]) != "test":
                    continue
                if test[:5] not in cond:
                    no_cond += 1
                    continue
                inp = M.inputs_of(test, M.files_of(test), cond[test[:5]], calc, dup)
                rows.append(dict(case_id=test, inputs=LI.bre_row(inp, 1.0)))
            r = tally("bre", m["release"], rows, "lasts")
            r["unit"] = "test (each test is scored at several cutoffs; the inputs, hence admission, are the same at each)"
            r["tests without day conditions (not built)"] = no_cond
        elif key == "cld":
            import cld_cases as M
            part = M.partitions()
            days_t = M.day_table()
            files = M.test_files({q.name for q in M.RAW.iterdir()})
            rows, no_day = [], 0
            for tt, fn in sorted(files.items()):
                if part.get(tt[:5]) != "test":
                    continue
                if tt[:5] not in days_t:
                    no_day += 1
                    continue
                inp, _ = M.inputs_of(tt, fn, days_t[tt[:5]])
                w = inp.get("planned_fuel_off_from_ignition_command_s")
                case = dict(case_id=tt, test=tt, inputs=inp, at_cutoff=dict(onset_from_ignition_command_s=0.0, planned_fuel_off_from_onset_s=w))
                rows.append(dict(case_id=tt, inputs=LI.cld3_row(case)))
            r = tally("cld3", m["release"], rows, "loss")
            r["unit"] = "test point with files"
            r["test points whose day is not in the experiment table (not built)"] = no_day
            r["onset"] = "taken as 0 s for this count only (window and t_severe from the ignition command)"
        else:
            import efield_cases as M
            part = M.partitions()
            rows_t, rows_l, st = [], [], Counter()
            for test in sorted(M.catalogue_tests()):
                if part.get(test[:5]) != "test":
                    continue
                sp, dp = M.RAW / f"{M.PRE}{test}_VoltageSweep.txt", M.RAW / f"{M.PRE}{test}_data.csv"
                if not (sp.exists() and dp.exists()):
                    st["no data or script file (not built)"] += 1
                    continue
                hdr, script = M.read_header(dp), M.read_script(sp)
                pol = M.POL.get(hdr.get("EFieldPol", ""), "unknown")
                if pol == "unknown":
                    st["unknown polarity (not built)"] += 1
                    continue
                gas = M.FUEL_GAS.get(hdr.get("MFCA3 Gas", ""))
                pl = M.plan(script, pol)
                u_fuel, x_fuel = M.num(script.get("velocityMixFinalMPerSec")), M.num(script.get("concentrationFinal"))
                n_c = M.CARBON.get(gas[0], 0) if gas else 0
                mol = u_fuel * M.FUEL_TUBE_AREA_M2 * x_fuel / M.MOLAR_VOLUME_STP_M3 if (gas and u_fuel is not None and x_fuel is not None) else None
                inputs = dict(test=test, day=test[:5], burner=M.BURNER.get(hdr.get("Burner", ""), "unknown"), polarity=pol,
                              fuel=gas[0] if gas else "unknown", x_fuel=x_fuel, u_coflow_m_s=M.num(script.get("velocityAirFinalMPerSec")),
                              carbon_flow_atoms_s=mol * n_c * M.AVOGADRO if mol is not None else None, window_s=pl["window_s"],
                              levels=[dict(lv) for lv in pl["levels"]])
                for row in LI.law_rows("efield3", [dict(inputs=inputs)]):
                    (rows_t if row["kind"] == "test" else rows_l).append(row)
            r = dict(flame_end=tally("efield3", m["release"], rows_t, "flame_end"), ion_current=tally("efield3", m["release"], rows_l, "ion_current"))
            r["unit"] = "flame_end: test; ion_current: planned voltage level"
            r.update(dict(st))
        res[key] = dict(model=m["label"], release=m["release"], **r)
        print(json.dumps({key: res[key]}, default=str))
    for key in ("spice", "samer", "slice", "bass"):
        res[key] = dict(model=lst["investigations"][key]["model"]["label"], release=lst["investigations"][key]["model"]["release"],
                        count="not possible from inputs alone: inputs and outcome sit in one record that is read only in the final run; "
                              "the final run reports asked, answered and refused with reasons for every release")
    (ROOT / "inputs_models.json").parent.mkdir(parents=True, exist_ok=True)
    (ROOT / "inputs_models.json").write_text(json.dumps(res, indent=1, default=str), encoding="utf-8")
    return 0


# ===================================================================================== final
def reveal_line(artifact_id, run_id, split_id, purpose):
    e = {"artifact_id": artifact_id, "at": datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%S.%fZ"),
         "final_test": True, "granted": True, "host": os.environ.get("COMPUTERNAME") or os.environ.get("HOSTNAME") or "unknown",
         "partition": "test", "purpose": purpose, "run_id": run_id, "split_id": split_id,
         "user": os.environ.get("USERNAME") or os.environ.get("USER") or "unknown"}
    return json.dumps(dict(sorted(e.items())), separators=(",", ":"))


def cmd_final(a):
    if not a.execute:
        sys.exit("refused: the final run needs --execute")
    p = Path(a.releases)
    h = sha256_file(p)
    if h != a.sha256:
        sys.exit(f"refused: the release list's sha256 is {h}, not {a.sha256}")
    lst = json.loads(p.read_text(encoding="utf-8"))
    if lst.get("frozen") is not True:
        sys.exit("refused: the release list is not frozen")
    key = a.slug
    inv = lst["investigations"][key]
    MARKERS.mkdir(parents=True, exist_ok=True)
    marker = MARKERS / f"{key}.started"
    try:
        fd = os.open(marker, os.O_CREAT | os.O_EXCL | os.O_WRONLY)
    except FileExistsError:
        sys.exit(f"refused: {marker.relative_to(REPO)} exists; this step has run")
    os.write(fd, json.dumps(dict(at=datetime.datetime.now(datetime.timezone.utc).isoformat(), list_sha256=h)).encode())
    os.close(fd)
    root = ROOT / "final" / "local"
    build(inv["lane"], "final", root, inv.get("coding", "v1"))
    res = score_investigation(key, inv, root)
    res.pop("_fc", None)
    # one reveal line per scored release and test day (the scorer reads the test-day outcomes once per release)
    split = json.loads((REPO / "configs/models/law" / inv["model"]["slug"] / "binding.json").read_text(encoding="utf-8"))["split_id"]
    lines = [reveal_line("sha256:" + hashlib.sha256(f"{e['release']}|{key}".encode()).hexdigest(), f"gen7:{key}:{label}", split, f"final:gen7:{key}")
             for label, e in res["releases"].items()]
    REVEAL_LOG.parent.mkdir(parents=True, exist_ok=True)
    with REVEAL_LOG.open("a", encoding="utf-8", newline="\n") as f:
        for line in lines:
            f.write(line + "\n")
    out = dict(investigation=inv["investigation"], list_sha256=h, at=datetime.datetime.now(datetime.timezone.utc).isoformat(), **res)
    (ROOT / "final").mkdir(parents=True, exist_ok=True)
    (ROOT / "final" / f"{key}.json").write_text(json.dumps(out, indent=1, default=str), encoding="utf-8")
    md = [f"## {inv['investigation']} (`{key}`)", "", f"Release list sha256 `{h}`. Outcome coding {res['coding']}. Test days only; aggregates; lower is better.", "",
          "| release | role | rule | answered | " + " | ".join(sorted({m for r in res["releases"].values() for m in r["metrics"]})) + " |"]
    mets = sorted({m for r in res["releases"].values() for m in r["metrics"]})
    md.append("|---" * (4 + len(mets)) + "|")
    for label, r in res["releases"].items():
        md.append(f"| {label} | {r['role']} | {r['rule']} | {json.dumps(r['answered'])} | " + " | ".join(
            f"{r['metrics'][m]:.4f}" if isinstance(r["metrics"].get(m), (int, float)) else "-" for m in mets) + " |")
    for n, mm in res["baselines"].items():
        md.append(f"| baseline: {n} | on the model's cases | | | " + " | ".join(f"{mm[m]:.4f}" if isinstance(mm.get(m), (int, float)) else "-" for m in mets) + " |")
    old = RESULTS_MD.read_text(encoding="utf-8") if RESULTS_MD.exists() else "# Final run of the seventh generation (PROTOCOL 12.5)\n\n"
    RESULTS_MD.write_text(old + "\n".join(md) + "\n\n", encoding="utf-8", newline="\n")
    print(json.dumps(dict(slug=key, written=str(RESULTS_MD.relative_to(REPO)), reveal_lines=len(lines))))
    return 0


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    sub = ap.add_subparsers(dest="cmd", required=True)
    sub.add_parser("template").set_defaults(fn=lambda a: template())
    sub.add_parser("inputs-check").set_defaults(fn=cmd_inputs_check)
    sub.add_parser("inputs-models").set_defaults(fn=cmd_inputs_models)
    s = sub.add_parser("rehearse")
    s.add_argument("--slug", default="")
    s.add_argument("--coding", default="", help="also rehearse this case version / coding (e.g. v2), beside v1 and the list's")
    s.set_defaults(fn=cmd_rehearse)
    s = sub.add_parser("final")
    s.add_argument("--slug", required=True)
    s.add_argument("--releases", required=True)
    s.add_argument("--sha256", required=True)
    s.add_argument("--execute", action="store_true")
    s.set_defaults(fn=cmd_final)
    a = ap.parse_args()
    return a.fn(a)


if __name__ == "__main__":
    sys.exit(main())
