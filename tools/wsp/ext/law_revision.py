#!/usr/bin/env python3
"""Assemble and score one typed revision of PROTOCOL 13.6 (a candidate checked once with the what-if tool).

    python -I tools/wsp/ext/law_revision.py assemble --slug cld --parent d1-fable --cand c1-fable --label C1 \
        --edits runs/dev/gen7/study/cld_c1.wsp --study docs/wsp/research/phys_cld_study.md
    python -I tools/wsp/ext/law_revision.py dev --slug cld --parent d1-fable --cand c1-fable \
        --other cld:w0-fable --other cld2:s1-fable

`assemble`: the edits file verbatim (two header comment lines added) as a revision of the parent release, compiled
and assembled by `ignis wsp`; then checked element by element: the release equals its parent except exactly the
elements the file writes, which carry the file's numbers. A line that cannot be applied stops.

`dev`: the official development score (PROTOCOL 13.4, 13.6; train and validation days together, in-sample) through
the Rust forward (`tools/wsp/law_run.py` via the lane scorer `law_score.py`, never the Python copy) and the lane's own
score(): for every scored target, on the cases parent and candidate both answer, pooled and by partition, beside the
parent, the other releases named (on the same cases where they answer them) and every baseline; and by day (mean
per-case loss, the strata of the failure analysis), with the days lower over the days whose forecasts change.
Writes runs/dev/gen7/<slug>_<cand>_dev.json (aggregates and per-day means only). Fits nothing, changes no number.
"""
import argparse
import copy
import hashlib
import json
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[2]
sys.path.insert(0, str(HERE))
from law_transcription_check import elements, statements  # noqa: E402

IG = REPO / "target-merged/release/ignis.exe"


def rid(slug, cand):
    return json.loads((REPO / f"configs/models/law/{slug}/{cand}/release.json").read_text(encoding="utf-8"))["id"]


def ig(*a):
    r = subprocess.run([str(IG), "wsp", *a], cwd=REPO, capture_output=True, text=True)
    if r.returncode != 0:
        sys.exit(f"ignis wsp {' '.join(a[:3])} failed: {(r.stderr or r.stdout)[-1500:]}")
    return r.stdout


def short(i):
    return f"sha256:{i[7:15]}…{i[-4:]}"


KIND = {"pmma_rod": "rods", "pmma_sphere": "spheres", "pmma_spherical_section": "sections", "pmma_film": "films", "pmma_slab": "slabs"}
STRATA = {s: (lambda r: KIND.get(r.get("material_class"), "other")) for s in ("bass1", "bass2", "bass3")}


def cmd_assemble(a):
    src = REPO / a.edits
    sha = hashlib.sha256(src.read_bytes()).hexdigest()
    study = REPO / a.study
    ssha = hashlib.sha256(study.read_bytes()).hexdigest()
    block = src.read_text(encoding="utf-8")
    parent = rid(a.slug, a.parent)
    d = REPO / f"configs/models/law/{a.slug}/{a.cand}"
    d.mkdir(parents=True, exist_ok=True)
    hdr = [f"# {a.slug} {a.label}: {a.edits} (sha256 {sha}), verbatim, a revision of",
           f"# {a.slug} {a.parent} {short(parent)} ({a.study}); every other number is the parent's."]
    (d / "edits.wsp").write_text("\n".join(hdr) + "\n" + block.rstrip("\n") + "\n", encoding="utf-8", newline="\n")
    meta = {"patch_id": f"law-{a.slug}-v1-{a.cand}", "parent_release_id": parent,
            "author": {"provider": "anthropic", "model_id": "claude-fable-5-2"},
            "exposure": {"sources": [f"repo:{a.study}(sha256:{ssha};candidate-{a.label})",
                                     f"repo:{a.edits}(sha256:{sha})",
                                     "runs/dev/gen7/whatif_log.jsonl(the-logged-what-if-run-of-" + a.label + ")"],
                         "datasets": [{"dataset_id": f"{a.slug}-train-and-validation-cases", "partitions": ["train", "validation"]}],
                         "development_feedback": ["PROTOCOL-13.2-13.3-13.4-development-scores", f"what-if-run-of-{a.label}"],
                         "prior_candidates": [parent]},
            "rationale": f"{a.slug} {a.label} (PROTOCOL 13.6): data-informed from the development days (train and validation); "
                         f"{a.edits} verbatim as a revision of {a.slug} {a.parent} ({a.study}). "
                         "Transcribed by the assembly engineer (claude-opus-5-5); no number chosen.",
            "origin": "system"}
    (d / "meta.json").write_text(json.dumps(meta, indent=2) + "\n", encoding="utf-8", newline="\n")
    ig("compile", "--family", a.slug, "--edits", str(d / "edits.wsp"), "--meta", str(d / "meta.json"), "--out", str(d / "patch.json"))
    out = json.loads(ig("assemble", "--family", a.slug, "--patch", str(d / "patch.json"), "--binding",
                        str(REPO / f"configs/models/law/{a.slug}/binding.json"), "--out", str(d / "release.json")))
    new = out["release_id"]
    p0 = elements(statements(ig("inspect", "--release", parent, "--edits")))
    p1 = elements(statements(ig("inspect", "--release", new, "--edits")))
    typed = elements(statements(block))
    bad = {}
    for k in set(p0) | set(p1) | set(typed):
        want = typed[k] if k in typed else p0.get(k, 0.0)
        if p1.get(k, 0.0) != want:
            bad[str(k)] = (p0.get(k), p1.get(k), want)
    same_as_parent = sorted(str(k) for k, v in typed.items() if p0.get(k, 0.0) == v)
    differ = sorted(str(k) for k in set(p0) | set(p1) if p0.get(k, 0.0) != p1.get(k, 0.0))
    res = dict(slug=a.slug, candidate=a.cand, label=a.label, edits=a.edits, edits_sha256=sha, study=a.study, study_sha256=ssha,
               parent=parent, release=new, elements_written=len(typed), elements_written_equal_to_parent=same_as_parent,
               elements_differing_from_parent=len(differ), differing_not_written=[k for k in differ if k not in {str(x) for x in typed}],
               mismatches=bad)
    (d / "assembly_check.json").write_text(json.dumps(res, indent=1) + "\n", encoding="utf-8", newline="\n")
    print(json.dumps(res, indent=1))
    return 1 if (bad or res["differing_not_written"]) else 0


# ------------------------------------------------------------------------------------------- development score
def cmd_dev(a):
    import law_score as LS
    import law_whatif as WI
    if a.coding:
        LS.BASS_CODING = a.coding   # the lane's plastics coding switch (v1 registered, v2 the second coding)
    R = WI.Runner(a.slug)          # contexts of the lane (train, validation) and the slug's scoring rules; no forward here

    def forecasts(slug, release):
        S = R.S if slug == a.slug else LS.SLUGS[slug]()
        ctx = R.ctx if slug == a.slug else {p: S.load(p) for p in WI.PARTS}
        return {p: S.release(copy.copy(ctx[p]), release) for p in WI.PARTS}   # LS.run_law: the Rust forward

    ids = {"parent": rid(a.slug, a.parent), "candidate": rid(a.slug, a.cand)}
    fc = {"parent": forecasts(a.slug, ids["parent"]), "candidate": forecasts(a.slug, ids["candidate"])}
    for o in a.other:
        s, c = o.split(":")
        ids[o] = rid(s, c)
        fc[o] = forecasts(s, ids[o])
    both = R.restrict(fc["candidate"], fc["parent"])          # cases both answer (candidate's forecasts)
    on_both = {"parent": R.restrict(fc["parent"], both), "candidate": both}
    for o in a.other:
        on_both[o] = R.restrict(fc[o], both)
    bl_names = list(R.S.baselines(R.ctx["train"]))
    for n in bl_names:
        on_both[n] = {p: R.S.restrict(R.S.baselines(R.ctx[p])[n], both[p]) for p in WI.PARTS}

    def count(f):
        return {t: sum(len(f[p][t]) for p in WI.PARTS) for t in f["train"]}

    res = dict(slug=a.slug, ids=ids, n=dict((k, count(v)) for k, v in on_both.items()), scores={}, days={})
    for k, f in on_both.items():
        try:
            res["scores"][k] = R.scores(f)[0]
        except Exception as e:   # a baseline that does not carry every target
            res["scores"][k] = None
            print(f"note: {k} not scored on every target ({e})", file=sys.stderr)
    # the other releases also on the cases they answer together with the candidate (for counts different from `both`)
    for o in a.other:
        res["scores"][o + " (cases it and the candidate both answer)"] = R.scores(R.restrict(fc["candidate"], fc[o]))[0]
        res["scores"][o + " (own forecasts on those cases)"] = R.scores(R.restrict(fc[o], fc["candidate"]))[0]
        res["n"][o + " (cases it and the candidate both answer)"] = count(R.restrict(fc["candidate"], fc[o]))
    # by stratum of the lane's rows (plastics: the kind of sample), every metric, pooled and by partition
    kind = STRATA.get(a.slug)
    if kind:
        res["strata"] = {}
        ks = {p: {r["case_id"]: kind(r) for r in R.ctx[p]["rows"]} for p in WI.PARTS}
        for kv in sorted({v for p in ks for v in ks[p].values()}):
            sub = lambda f: {p: {t: {c: x for c, x in f[p][t].items() if ks[p].get(c) == kv} for t in f[p]} for p in WI.PARTS}  # noqa: E731
            res["strata"][kv] = {}
            for k, f in on_both.items():
                g = sub(f)
                try:
                    res["strata"][kv][k] = dict(n=count(g), scores=R.scores(g)[0])
                except Exception:
                    res["strata"][kv][k] = None
    study = {k: R.study(on_both[k]) for k in ["parent", "candidate"] + list(a.other)}
    secs = {k: WI.sections(a.slug, v) for k, v in study.items()}
    for i, (title, kind, _) in enumerate(secs["candidate"]):
        if kind == "none":
            continue
        dd = {k: WI.by_day(secs[k][i][2]) for k in secs}
        db, dn = dd["parent"], dd["candidate"]
        diff = [d for d in db if d in dn and abs(db[d] - dn[d]) > 1e-12]
        res["days"][title] = dict(days=len(db), differ=len(diff), lower=sum(1 for d in diff if dn[d] < db[d]),
                                  cases={k: len(secs[k][i][2]) for k in secs},
                                  per_day={str(d): {k: dd[k].get(d) for k in dd} for d in sorted(db)})
    out = REPO / f"runs/dev/gen7/{a.slug}_{a.cand}{'_' + a.coding if a.coding else ''}_dev.json"
    out.write_text(json.dumps(res, indent=1, default=lambda x: None) + "\n", encoding="utf-8", newline="\n")
    # a compact print
    for t, mets in res["scores"]["candidate"].items():
        for lab, v in mets.items():
            row = " | ".join(f"{k}: {WI.fmt(((res['scores'].get(k) or {}).get(t) or {}).get(lab, {}).get('pooled'))}" for k in on_both)
            print(f"{t} / {lab} pooled: {row}")
    for title, v in res["days"].items():
        print(f"{title[:60]}: lower on {v['lower']} of {v['differ']} days that change ({v['days']} days)")
    print("->", out.relative_to(REPO))
    return 0


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    sub = ap.add_subparsers(dest="cmd", required=True)
    s = sub.add_parser("assemble")
    for k in ("slug", "parent", "cand", "label", "edits", "study"):
        s.add_argument("--" + k, required=True)
    s.set_defaults(fn=cmd_assemble)
    s = sub.add_parser("dev")
    for k in ("slug", "parent", "cand"):
        s.add_argument("--" + k, required=True)
    s.add_argument("--coding", choices=["v1", "v2"], help="plastics slugs: the lane's BASS_CODING (v1 registered, v2 second coding)")
    s.add_argument("--other", action="append", default=[], help="slug:candidate of another release to score beside")
    s.set_defaults(fn=cmd_dev)
    a = ap.parse_args()
    if any("holdout" in str(v) for v in vars(a).values()):
        sys.exit("refused: a path under holdout/")
    return a.fn(a)


if __name__ == "__main__":
    sys.exit(main())
