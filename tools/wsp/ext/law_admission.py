#!/usr/bin/env python3
"""Admission counts of the seventh-generation W0 law releases on the law input rows of each slug
(tools/wsp/ext/law_inputs.py: conditions only). Runs tools/wsp/law_run.py and counts, per partition and target,
the admitted and refused cases and the refusal reasons. No outcome is read and nothing is scored.

    python -I tools/wsp/ext/law_admission.py [--slug bre,...] [--out runs/dev/gen7/admission_counts.json]
"""
import argparse
import json
import re
import subprocess
import sys
from collections import Counter, defaultdict
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[2]
WORK = REPO / "runs/dev/data_all/work"
OUTDIR = REPO / "runs/dev/gen7/admission"

BRE_H = [6, 8, 10, 12, 15, 20, 25, 30, 40, 50, 60, 80, 100, 120, 150, 180, 210]
EF_H = [2, 5, 10, 15, 20, 30, 45, 60, 90, 120, 150]
CLD_H = [1, 2, 3, 5, 7, 10, 15, 20, 30, 45, 60, 90, 120]

# slug: (candidates, partitions, [(row kind filter or None, targets, times)])
PLAN = {
    "bre": (["w0-fable"], ["train", "validation"], [(None, "flame_end,lasts,height_at_end", BRE_H)]),
    "spice": (["w0-fable", "lit-fable"], ["train", "validation", "validation_given_flow"],
              [(None, "smoke_point_length,smoke_point_fuel_flow,flame_length,emits_soot", None)]),
    "efield": (["w0-fable"], ["train", "validation"],
               [("level", "ion_current", None), ("test", "flame_end,peak_ion_current,peak_voltage", EF_H)]),
    "samer": (["w0-fable"], ["train", "validation"], [(None, "d10,d30,iss_full_scale", None)]),
    "same": (["w0-fable"], ["held"], [(None, "d10,d30,iss_full_scale", None)]),
    "daft": (["w0-fable"], ["held"], [(None, "number_concentration", None)]),
    "bass1": (["w0-fable"], ["train", "validation"],
              [(None, "holds_low_flow,holds_high_flow,oxygen_limit,quench_speed,blowoff_speed,flat_holds_low_flow,"
                      "flat_holds_high_flow,flat_quench_speed", None)]),
    "slice": (["w0-fable"], ["train", "validation"], [(None, "blow_off,lifted,flame_length", None)]),
    "cld": (["w0-fable"], ["train", "validation"], [(None, "loss,radiometer", CLD_H)]),
}


def release(slug, cand):
    return json.loads((REPO / f"configs/models/law/{slug}/{cand}/release.json").read_text(encoding="utf-8"))["id"]


def reason_key(r):
    """A refusal reason with the case's own numbers removed, for counting."""
    return re.sub(r"-?\d+(\.\d+)?(e-?\d+)?", "#", r)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--slug", default="")
    ap.add_argument("--out", default="runs/dev/gen7/admission_counts.json")
    a = ap.parse_args()
    slugs = [s for s in a.slug.split(",") if s] or list(PLAN)
    outp = REPO / a.out
    res = json.loads(outp.read_text(encoding="utf-8")) if outp.exists() else {}
    for slug in slugs:
        cands, parts, jobs = PLAN[slug]
        res[slug] = {}
        for cand in cands:
            rid = release(slug, cand)
            res[slug][cand] = {"release": rid}
            for part in parts:
                rows = [json.loads(l) for l in (WORK / f"transfer_{slug}/law_{part}.jsonl").read_text(encoding="utf-8").splitlines() if l.strip()]
                entry = res[slug][cand].setdefault(part, {"rows": len(rows)})
                for kind, targets, times in jobs:
                    sel = [r for r in rows if kind is None or r.get("kind") == kind]
                    if not sel:
                        continue
                    OUTDIR.mkdir(parents=True, exist_ok=True)
                    cpath = OUTDIR / f"{slug}_{cand}_{part}_{kind or 'all'}_cases.jsonl"
                    fpath = OUTDIR / f"{slug}_{cand}_{part}_{kind or 'all'}_forward.jsonl"
                    cpath.write_text("".join(json.dumps(r) + "\n" for r in sel), encoding="utf-8", newline="\n")
                    cmd = [sys.executable, str(REPO / "tools/wsp/law_run.py"), "--release", rid, "--cases", str(cpath),
                           "--out", str(fpath), "--targets", targets]
                    if times:
                        cmd += ["--times", ",".join(str(t) for t in times)]
                    r = subprocess.run(cmd, cwd=REPO, capture_output=True, text=True)
                    if r.returncode != 0:
                        sys.exit(r.stderr or r.stdout)
                    by_t = defaultdict(lambda: {"cases": 0, "admitted": 0, "refused": 0, "reasons": Counter()})
                    by_cov = defaultdict(Counter)
                    meta = {x["case_id"]: x for x in sel}
                    for line in fpath.read_text(encoding="utf-8").splitlines():
                        row = json.loads(line)
                        for t, s in row["targets"].items():
                            b = by_t[t]
                            b["cases"] += 1
                            if s["admitted"]:
                                b["admitted"] += 1
                            else:
                                b["refused"] += 1
                                for rr in s["reasons"]:
                                    b["reasons"][reason_key(rr)] += 1
                            cov = meta.get(row["case_id"], {}).get("coverage")
                            if cov:
                                by_cov[t][f"{cov}|{'admitted' if s['admitted'] else 'refused'}"] += 1
                    for t, b in by_t.items():
                        b["reasons"] = dict(b["reasons"].most_common())
                        if t in by_cov:
                            b["by_coverage"] = dict(by_cov[t])
                        entry[t] = b
                print(slug, cand, part, {t: (v["admitted"], v["refused"]) for t, v in entry.items() if isinstance(v, dict)})
    outp.write_text(json.dumps(res, indent=1) + "\n", encoding="utf-8", newline="\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
