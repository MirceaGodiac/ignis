#!/usr/bin/env python3
"""Finite-difference sensitivity of a law release's TRAIN score to each typed number that carries weight.
Diagnostic only: a perturbation is a question about the model, not a proposed value. The perturbed numbers live
in memory and run through the Python copy of the forward (tools/wsp/ext/law_pyforward.py, checked against the Rust
forward to 1e-14 on every train case); nothing is compiled, assembled or registered, and only TRAIN cases are read.

    python -I tools/wsp/ext/law_sensitivity.py --slug cld --release sha256:... [--rel 0.01]

For every nonzero authored element except the domain bounds (x.<f>.lo/hi) and num.substeps: the element is moved by
+rel x |value| (forward difference) and the train score of each primary metric of tools/wsp/ext/law_score.py is
recomputed through the lane's score(). Writes runs/dev/gen7/sensitivity/<slug>_<id16>.json.
"""
import argparse
import copy
import json
import sys
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[2]
sys.path.insert(0, str(HERE))
import law_pyforward as PF  # noqa: E402
import law_score as LS  # noqa: E402

PF.BISECTION = 64


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--slug", required=True)
    ap.add_argument("--release", required=True)
    ap.add_argument("--rel", type=float, default=0.01)
    a = ap.parse_args()
    decl = PF.decl_of(a.slug)
    P0 = PF.params(a.slug, a.release)
    cur = {"P": P0}

    def py_run(slug, rid, rows, targets, times=None, tag="run"):
        return PF.forward(decl, cur["P"], rows, targets.split(","), times)
    LS.run_law = py_run
    S = LS.SLUGS[a.slug]()
    ctx = S.load("train")
    mets = [(t, m[0], m[1]) for t, ms in S.metrics.items() for m in ms[:1]]

    def evaluate():
        c = copy.copy(ctx)
        fc = S.release(c, a.release)
        res = S.score(c, fc)
        return {f"{t}: {lab}": LS.get(res.get(t), path) for t, lab, path in mets}
    t0 = time.time()
    base = evaluate()
    el = []
    for name, arr in P0.items():
        if name.endswith(".lo") or name.endswith(".hi") or name == "num.substeps":
            continue
        for idx in zip(*arr.nonzero()) if arr.ndim else ([()] if float(arr) != 0.0 else []):
            el.append((name, tuple(int(i) for i in idx)))
    out = []
    for name, idx in el:
        v = float(P0[name][idx] if idx else P0[name])
        h = a.rel * abs(v)
        P = {k: (x.copy() if k == name else x) for k, x in P0.items()}
        if idx:
            P[name][idx] = v + h
        else:
            P[name] = P[name] + h
        cur["P"] = P
        try:
            r = evaluate()
        except Exception as e:   # a perturbation the forward refuses (e.g. a spread pushed below zero)
            r = {k: None for k in base}
            r["error"] = str(e)[:200]
        out.append(dict(element=name + (str(list(idx)) if idx else ""), value=v, step=h,
                        delta={k: (r[k] - base[k]) if (r.get(k) is not None and base[k] is not None) else None for k in base}))
    cur["P"] = P0
    res = dict(slug=a.slug, release=a.release, partition="train", rel_step=a.rel, base=base, elements=len(out),
               seconds=round(time.time() - t0, 1), sensitivity=out)
    d = REPO / "runs/dev/gen7/sensitivity"
    d.mkdir(parents=True, exist_ok=True)
    (d / f"{a.slug}_{a.release[7:23]}.json").write_text(json.dumps(res, indent=1, default=lambda x: None), encoding="utf-8")
    print(json.dumps(dict(slug=a.slug, base=base, elements=len(out), seconds=res["seconds"])))
    return 0


if __name__ == "__main__":
    sys.exit(main())
