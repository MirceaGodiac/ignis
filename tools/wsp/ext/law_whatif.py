#!/usr/bin/env python3
"""The logged what-if tool of PROTOCOL 13.6: check a typed candidate of a law on the development cases.

    python -I tools/wsp/ext/law_whatif.py run --slug cld --base configs/models/law/cld/d1-fable --edits my_step.wsp --note "methane loses sooner; ethylene unchanged"
    python -I tools/wsp/ext/law_whatif.py describe --slug cld --base configs/models/law/cld/d1-fable
    python -I tools/wsp/ext/law_whatif.py cases --slug cld --base configs/models/law/cld/d1-fable --out runs/dev/gen7/cases_cld.csv

`run` builds the candidate IN MEMORY: every number of the base release (`ignis wsp inspect --edits`) with the changed
lines of `--edits` applied, under the compiler's rules (a tensor the slug's spec does not have, an index outside its
shape, an element written twice in the file, or a value outside the tensor's bounds is refused and nothing runs).
It runs base and candidate on the development cases (train and validation days; PROTOCOL 13.4: in-sample) with the
Python copy of the forward (tools/wsp/ext/law_pyforward.py; checked against the Rust forward to 1e-14) and the slug's
scoring-time rules, through the data lane's own score(), and prints for each scored target: pooled, by partition
and by day (days lower over days whose forecasts differ), beside every baseline; by the main strata of the failure
analysis; and calibration. Every run appends one line to runs/dev/gen7/whatif_log.jsonl. It never reads a test day
and never writes a release, a patch or a file under configs/.

Slugs and their scoring rules: bre (`--rule flame_end`, the registered mapping, or `--rule lasts`, phys_bre2.md
section 2), cld (base d1), cld2, cld3, slice, slice2 (the p_lifted rule of phys_slice2.md), bass2, bass3, bass4, efield, efield3, spice, samer.
"""
import argparse
import copy
import csv
import datetime
import hashlib
import json
import math
import subprocess
import sys
from collections import defaultdict
from pathlib import Path

import numpy as np

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[2]
sys.path.insert(0, str(HERE))
import law_anatomy as AN  # noqa: E402
import law_inputs as LI  # noqa: E402
import law_pyforward as PF  # noqa: E402
import law_score as LS  # noqa: E402
from law_transcription_check import elements, statements  # noqa: E402

LOG = REPO / "runs/dev/gen7/whatif_log.jsonl"
SLUGS = {"bre": "bre", "slice": "slice", "cld": "cld", "cld2": "cld2", "slice2": "slice2", "bass2": "bass2", "bass3": "bass3", "bass4": "bass4", "bass5": "bass5", "efield": "efield", "efield3": "efield3", "cld3": "cld3", "spice": "spice", "samer": "samer"}
ANATOMY = {"cld2": "cld", "cld3": "cld", "efield3": "efield", "slice2": "slice", "bass2": "bass1", "bass3": "bass1", "bass4": "bass1", "bass5": "bass1"}   # strata definitions of the failure analysis
PARTS = ("train", "validation")


# ------------------------------------------------------------------------------------------- the candidate
def spec_of(slug):
    s = json.loads((REPO / f"configs/models/law/{slug}/spec.json").read_text(encoding="utf-8"))
    return {t["name"]: t for t in s["tensors"]}


def apply_edits(slug, P0, text):
    """The changed lines on top of the base, under the compiler's rules; raises SystemExit naming the first fault."""
    spec = spec_of(slug)
    seen = set()
    P = {k: v.copy() for k, v in P0.items()}
    stm = statements(text)
    for st in stm:
        el = elements([st])
        for (name, idx), v in el.items():
            if name not in spec:
                sys.exit(f"refused: tensor {name} is not in the spec of {slug}")
            shape = tuple(spec[name]["shape"])
            if len(idx) != len(shape) or any(not (0 <= i < n) for i, n in zip(idx, shape)):
                sys.exit(f"refused: {name}{list(idx)} is outside the shape {list(shape)}")
            if (name, idx) in seen:
                sys.exit(f"refused: element {name}{list(idx)} written twice")
            seen.add((name, idx))
            b = spec[name].get("bounds") or {}
            if (b.get("min") is not None and v < b["min"]) or (b.get("max") is not None and v > b["max"]):
                sys.exit(f"refused: {name}{list(idx)} = {v} is outside the bounds [{b.get('min')}, {b.get('max')}]")
            if not math.isfinite(v):
                sys.exit(f"refused: {name}{list(idx)} is not finite")
            if idx:
                P[name][idx] = v
            else:
                P[name] = np.array(v)
    return P, len(seen)


def base_release(base):
    return json.loads((REPO / base / "release.json").read_text(encoding="utf-8"))["id"]


# ------------------------------------------------------------------------------------------- running
class Runner:
    def __init__(self, slug, rule="flame_end"):
        self.slug, self.rule = slug, rule
        self.decl = PF.decl_of(slug)
        self.S = LS.SLUGS[SLUGS[slug]]()
        self.ctx = {p: self.S.load(p) for p in PARTS}

    def forecasts(self, P):
        LS.run_law = lambda slug, rid, rows, targets, times=None, tag="": PF.forward(self.decl, P, rows, targets.split(","), times)
        out = {}
        for p in PARTS:
            c = copy.copy(self.ctx[p])
            out[p] = self.bre_lasts(c, P) if (self.slug == "bre" and self.rule == "lasts") else self.S.release(c, "in-memory")
        return out

    def bre_lasts(self, ctx, P):
        """phys_bre2.md section 2: extinction by h = 1 - lasts(min(h, stop)) / lasts(c); termination = lasts(stop) / lasts(c)."""
        import bre_cases as L
        b = json.loads((REPO / "runs/dev/data_all/work/transfer_bre/baselines_train.json").read_text(encoding="utf-8"))["t1"]["planned_stop"]["constants"]
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
        res = PF.forward(self.decl, P, rows, ["lasts"])
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

    @staticmethod
    def merge(a, b):
        if isinstance(a, list):
            return a + b
        if isinstance(a, dict):
            out = dict(a)
            for k, v in b.items():
                out[k] = Runner.merge(a[k], v) if (k in a and isinstance(v, (dict, list)) and type(a[k]) is type(v)) else v
            return out
        return a

    def pooled_ctx(self):
        out = self.ctx[PARTS[0]]
        for p in PARTS[1:]:
            out = self.merge(out, self.ctx[p])
        return out

    @classmethod
    def pooled_fc(cls, fc):
        out = {}
        for t in fc[PARTS[0]]:
            x = fc[PARTS[0]][t]
            for p in PARTS[1:]:
                x = cls.merge(x, fc[p][t])
            out[t] = x
        return out

    def scores(self, fc):
        """{target: {metric label: {pooled, train, validation}}} through the lane's score()."""
        out = defaultdict(dict)
        res = {p: self.S.score(self.ctx[p], fc[p]) for p in PARTS}
        res["pooled"] = self.S.score(self.pooled_ctx(), self.pooled_fc(fc))
        for t, mets in self.S.metrics.items():
            for lab, path in mets:
                out[t][lab] = {p: LS.get(res[p].get(t), path) for p in res}
        return out, res

    def restrict(self, fc_a, fc_b):
        return {p: self.S.restrict(fc_a[p], fc_b[p]) for p in PARTS}

    def study(self, fc):
        rows = []
        for p in PARTS:
            if self.S.study:
                for r in self.S.study(self.ctx[p], fc[p]):
                    r["partition"] = p
                    rows.append(r)
        return rows


def sections(slug, rows):
    try:
        return AN.slug_sections(ANATOMY.get(slug, slug), rows)
    except Exception as e:   # a study row format the anatomy does not know: report, do not fail the run
        return [(f"(strata not available: {e})", "none", [])]


def by_day(cases):
    g = defaultdict(list)
    for c in cases:
        d = c["strata"].get("day")
        if d is not None:
            g[d].append(c["loss"])
    return {d: sum(v) / len(v) for d, v in g.items()}


def fmt(x):
    return AN.fmt(x)


# ------------------------------------------------------------------------------------------- commands
def cmd_run(a):
    slug = a.slug
    rid = base_release(a.base)
    P0 = PF.params(slug, rid)
    text = Path(a.edits).read_text(encoding="utf-8")
    P1, n_el = apply_edits(slug, P0, text)
    R = Runner(slug, a.rule)
    fb, fn = R.forecasts(P0), R.forecasts(P1)
    both_b, both_n = R.restrict(fb, fn), R.restrict(fn, fb)
    sb, _ = R.scores(both_b)
    sn, _ = R.scores(both_n)
    bl = {n: {p: R.S.restrict(f, both_n[p]) for p, f in ((p, R.S.baselines(R.ctx[p])[n]) for p in PARTS)} for n in R.S.baselines(R.ctx["train"])}
    sbl = {}
    for n, f in bl.items():
        try:
            sbl[n] = R.scores(f)[0]
        except Exception as e:   # a baseline that does not carry every target of the slug
            sbl[n] = None
    print(f"# what-if: {slug}, base {a.base} ({rid}), rule {a.rule}, cases {a.cases}; {n_el} elements changed; in-sample, partitions {','.join(PARTS)}" + ("; test days of plastics read after the final run (PROTOCOL 13.8): no held-out score" if "test" in PARTS else ""))
    print(f"note: {a.note}\n")
    pooled = {}
    for t in sn:
        print(f"## target {t} (on the cases base and candidate both answer; lower is better)\n")
        print("| metric | partition | base | candidate | " + " | ".join(n for n in sbl) + " |")
        print("|---" * (4 + len(sbl)) + "|")
        for lab in sn[t]:
            for p in ("pooled",) + PARTS:
                bv = [fmt((sbl[n] or {}).get(t, {}).get(lab, {}).get(p)) if sbl[n] else "-" for n in sbl]
                print(f"| {lab} | {p} | {fmt(sb[t][lab][p])} | {fmt(sn[t][lab][p])} | " + " | ".join(bv) + " |")
            pooled[f"{t}: {lab}"] = dict(base=sb[t][lab]["pooled"], candidate=sn[t][lab]["pooled"])
        print()
    # strata, by day, calibration from the per-case study rows
    rb, rn = R.study(both_b), R.study(both_n)
    days_summary = {}
    for (title, kind, cb), (_, _, cn) in zip(sections(slug, rb), sections(slug, rn)):
        if kind == "none":
            print(title)
            continue
        print(f"## {title}\n\nBase:")
        print("\n".join((AN.table_quantile(cb) if kind == "quantile" else AN.table_binary(cb) + AN.calibration(cb))))
        print("\nCandidate:")
        print("\n".join((AN.table_quantile(cn) if kind == "quantile" else AN.table_binary(cn) + AN.calibration(cn))))
        db, dn = by_day(cb), by_day(cn)
        diff = [d for d in db if d in dn and abs(db[d] - dn[d]) > 1e-12]
        low = sum(1 for d in diff if dn[d] < db[d])
        days_summary[title] = dict(days=len(db), differ=len(diff), lower=low)
        print(f"\nBy day (mean per-case loss): candidate lower on **{low} of {len(diff)}** days whose forecasts differ ({len(db)} days).\n")
        print("| day | base | candidate |\n|---|---|---|")
        for d in sorted(db):
            print(f"| {d} | {fmt(db[d])} | {fmt(dn.get(d))} |")
        print()
    LOG.parent.mkdir(parents=True, exist_ok=True)
    with LOG.open("a", encoding="utf-8", newline="\n") as f:
        f.write(json.dumps(dict(time=datetime.datetime.now(datetime.timezone.utc).isoformat(timespec="seconds"), slug=slug, rule=a.rule, coding=a.cases, partitions=list(PARTS),
                                base=a.base, base_release=rid, edits=str(a.edits), edits_sha256=hashlib.sha256(text.encode("utf-8")).hexdigest(),
                                elements_changed=n_el, note=a.note, pooled=pooled, days=days_summary)) + "\n")
    print(f"logged -> {LOG.relative_to(REPO)}")
    return 0


def cmd_describe(a):
    rid = base_release(a.base)
    d = PF.decl_of(a.slug)
    print(f"# {a.slug}: base {a.base} ({rid})\n\nFeatures (column order of every W1 and skip):\n")
    for i, f in enumerate(d["features"]):
        print(f"- {i} `{f['name']}` [{f['unit']}], {f['transform']}: {f.get('description', '')}")
    print("\nTargets:\n")
    for t in d["targets"]:
        print(f"- `{t['name']}` ({t['kind']}{', causes ' + ', '.join(t['causes']) if t.get('causes') else ''}"
              f"{', stop condition ' + t['scheduled_stop']['condition'] if t.get('scheduled_stop') else ''}): {t.get('description', '')}")
    out = subprocess.run([str(REPO / "target-merged/release/ignis.exe"), "wsp", "inspect", "--release", rid, "--edits"], cwd=REPO,
                         capture_output=True, text=True, check=True).stdout
    print("\nEvery nonzero number of the base (current values; elements not listed are zero):\n\n```text\n" + out.strip() + "\n```")
    return 0


def flat(prefix, v, out):
    if isinstance(v, dict):
        for k, x in v.items():
            flat(f"{prefix}{k}." if prefix or True else k, x, out) if isinstance(x, dict) else out.__setitem__(f"{prefix}{k}", x if not isinstance(x, list) else json.dumps(x))
    else:
        out[prefix.rstrip(".")] = v


def cmd_cases(a):
    rid = base_release(a.base)
    P0 = PF.params(a.slug, rid)
    R = Runner(a.slug, a.rule)
    fb = R.forecasts(P0)
    rows = R.study(fb)
    flatrows = []
    for r in rows:
        o = {"partition": r.get("partition"), "day": r.get("day") or r.get("group"), "case_id": r.get("case_id")}
        for k in ("inputs", "outcome", "forecast", "loss_contribution"):
            v = r.get(k)
            if isinstance(v, dict):
                for kk, x in v.items():
                    if isinstance(x, dict):
                        for k3, y in x.items():
                            o[f"{k}.{kk}.{k3}"] = json.dumps(y) if isinstance(y, (list, dict)) else y
                    else:
                        o[f"{k}.{kk}"] = json.dumps(x) if isinstance(x, list) else x
            else:
                o[k] = json.dumps(v) if isinstance(v, list) else v
        for k in ("coverage", "sustain_coverage", "forecast_sustain", "sustain_brier", "target", "level", "cutoff_s", "test", "step_in_test"):
            if k in r:
                o[k] = json.dumps(r[k]) if isinstance(r[k], (list, dict)) else r[k]
        flatrows.append(o)
    cols = []
    for o in flatrows:
        for k in o:
            if k not in cols:
                cols.append(k)
    out = Path(a.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    with out.open("w", encoding="utf-8", newline="") as f:
        w = csv.DictWriter(f, fieldnames=cols)
        w.writeheader()
        for o in flatrows:
            w.writerow(o)
    print(f"{len(flatrows)} rows (partitions {','.join(PARTS)}; in-sample) -> {out}")
    return 0


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    sub = ap.add_subparsers(dest="cmd", required=True)
    for name, fn in (("run", cmd_run), ("describe", cmd_describe), ("cases", cmd_cases)):
        s = sub.add_parser(name)
        s.add_argument("--slug", required=True, choices=list(SLUGS))
        s.add_argument("--base", required=True, help="candidate directory, e.g. configs/models/law/cld/d1-fable")
        s.add_argument("--rule", default="flame_end", choices=["flame_end", "lasts"], help="bre only: the scoring rule")
        s.add_argument("--partitions", default="train,validation",
                       help="partitions to run, comma-separated; `test` exists only for the plastics on coding v2 (PROTOCOL 13.8: the "
                            "test days of plastics are study data after the final run; nothing scored on them is held out)")
        s.add_argument("--cases", "--coding", dest="cases", default="v1", choices=["v1", "v2", "v3"],
                       help="case version: slice and slice2 (v2: the tube from the picture table, tools/wsp/ext/slice_cases_v2.py) and "
                            "the plastics bass1/bass2/bass3 (v2: the outcome coding of bass1_cases_v2.py); v1, the registered one, elsewhere")
        if name == "run":
            s.add_argument("--edits", required=True)
            s.add_argument("--note", required=True, help="the effect expected, written before the run")
        if name == "cases":
            s.add_argument("--out", required=True)
        s.set_defaults(fn=fn)
    a = ap.parse_args()
    if a.cases != "v1" and not (a.slug.startswith("slice") or a.slug.startswith("bass")):
        sys.exit(f"refused: {a.slug} has one case version only")
    if a.cases == "v3" and not a.slug.startswith("bass"):
        sys.exit("refused: version 3 exists only for the plastics coding")
    LS.SLICE_CASES = a.cases if a.slug.startswith("slice") else "v1"
    global PARTS
    PARTS = tuple(x.strip() for x in a.partitions.split(",") if x.strip())
    if "test" in PARTS and not (a.slug.startswith("bass") and a.cases in ("v2", "v3")):
        sys.exit("refused: the test partition is study data only for the plastics on coding v2 (PROTOCOL 13.8)")
    LS.BASS_CODING = a.cases if a.slug.startswith("bass") else "v1"
    if any(x in str(getattr(a, "edits", "") or "") + str(getattr(a, "out", "") or "") for x in ("holdout",)):
        sys.exit("refused: a path under holdout/")
    return a.fn(a)


if __name__ == "__main__":
    sys.exit(main())
