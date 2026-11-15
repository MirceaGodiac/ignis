#!/usr/bin/env python3
"""A Python copy of the generic law forward (crates/ignis-wsp/src/law.rs), for train-only diagnostics: finite-
difference sensitivity of a train score to one typed number. Diagnostic only: it never writes a weight into a
model, never assembles a release, never runs on validation, held or test cases. Its output is checked against the
Rust forward (`check`) before any use; a sensitivity is a question about the model, not a proposed value.

    python -I tools/wsp/ext/law_pyforward.py check --slug bre --release sha256:...   (Python against law_run.py on train)

`params(slug, release)` reads every number of a release (`ignis wsp inspect --edits`) into full tensors (shapes
from the slug's spec); `forward(decl, params, rows, targets, times)` returns what tools/wsp/law_run.py returns:
{case_id: {target: {kind, admitted, reasons, quantiles, probability, times_s, cdf}}}.
"""
import argparse
import json
import math
import subprocess
import sys
from pathlib import Path
from statistics import NormalDist

import numpy as np

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[2]
sys.path.insert(0, str(HERE))
from law_transcription_check import elements, statements  # noqa: E402

LEVELS = [0.05, 0.25, 0.5, 0.75, 0.95]
ND = NormalDist()
H = 8
BISECTION = 200   # as law.rs; the sensitivity runs set 64 (interval below 1e-18 of the bracket)


def decl_of(slug):
    return json.loads((REPO / f"configs/models/law/{slug}/decl.json").read_text(encoding="utf-8"))


def params(slug, release):
    spec = json.loads((REPO / f"configs/models/law/{slug}/spec.json").read_text(encoding="utf-8"))
    shapes = {t["name"]: tuple(t["shape"]) for t in spec["tensors"]}
    r = subprocess.run([str(REPO / "target-merged/release/ignis.exe"), "wsp", "inspect", "--release", release, "--edits"],
                       cwd=REPO, capture_output=True, text=True, check=True)
    el = elements(statements(r.stdout))
    P = {n: np.zeros(s) if s else np.zeros(()) for n, s in shapes.items()}
    for (name, idx), v in el.items():
        P[name][idx if idx else ()] = v
    if "num.substeps" in P and float(P["num.substeps"]) == 0.0:
        P["num.substeps"] = np.array(512.0)
    return P


def lookup(case, name):
    src = case["inputs"] if isinstance(case.get("inputs"), dict) else case
    if name not in src:
        return None, None, False
    v = src[name]
    if isinstance(v, dict):
        return v.get("value"), v.get("unit"), v.get("known", True) is not False
    return v, None, True


def net(P, prefix, z):
    b0 = float(P[f"{prefix}.b0"])
    if f"{prefix}.skip" not in P:
        return b0
    h = P[f"{prefix}.b1"] + P[f"{prefix}.W1"] @ z
    return b0 + float(P[f"{prefix}.skip"] @ z) + float(P[f"{prefix}.w2"] @ np.maximum(h, 0.0))


def features(decl, P, case):
    z, reasons = [], []
    for f in decl["features"]:
        n = f["name"]
        c, s, lo, hi = (float(P[f"x.{n}.{k}"]) for k in ("centre", "scale", "lo", "hi"))
        v, unit, known = lookup(case, n)
        ok = known and isinstance(v, (int, float)) and not isinstance(v, bool) and math.isfinite(float(v)) and (unit is None or unit == f["unit"])
        if not ok:
            reasons.append(f"feature {n}: condition \"{n}\" missing from the prefix")
            z.append(math.nan)
            continue
        x = float(v)
        if not (lo <= x <= hi):
            reasons.append(f"feature {n} = {x} {f['unit']} is outside its declared domain [{lo}, {hi}]")
            z.append(math.nan)
            continue
        t = f["transform"]
        z.append((x - c) / s if t == "identity" else math.log(x / c) / s if t == "ln" else (1.0 / x - 1.0 / c) / s)
    return np.array(z), reasons


def mixture_ln_quantile(p, sd, w, k):
    zq = ND.inv_cdf(p)
    if w == 0.0 or k == 1.0:
        return sd * zq
    if w == 1.0:
        return k * sd * zq
    a, b = sd * zq, k * sd * zq
    lo, hi = min(a, b), max(a, b)
    r2 = math.sqrt(2.0)
    for _ in range(BISECTION):
        mid = 0.5 * (lo + hi)
        if (1 - w) * 0.5 * math.erfc(-mid / sd / r2) + w * 0.5 * math.erfc(-mid / (k * sd) / r2) - p < 0:
            lo = mid
        else:
            hi = mid
    return 0.5 * (lo + hi)


def softplus(x):
    return np.where(x > 30.0, x, np.log1p(np.exp(np.minimum(x, 30.0))))


def event_table(clocks, tau, stop, times, substeps):
    """law.rs event_table, vectorised over the substeps of each interval."""
    nc = len(clocks)
    lt = np.array([c[0] for c in clocks])
    k = np.array([c[1] for c in clocks])

    def ln_s(t):
        t = np.asarray(t, dtype=float)
        out = np.zeros(t.shape)
        m = t > 0
        out[m] = -softplus(k[:, None] * (np.log(t[m])[None, :] - lt[:, None])).sum(axis=0)
        return out
    ln_tau = float(ln_s(np.array([tau]))[0])
    rows = np.zeros((nc + 2, len(times)))
    f = np.zeros(nc)
    t, s = tau, 1.0
    for j, tj in enumerate(times):
        target = tj if stop is None else min(tj, stop)
        if target > t:
            u = t + (target - t) * np.arange(substeps + 1) / substeps
            u[-1] = target
            surv = np.minimum(np.exp(ln_s(u[1:]) - ln_tau), 1.0)
            surv = np.minimum.accumulate(np.minimum(surv, s))
            prev = np.concatenate([[s], surv[:-1]])
            lost = prev - surv
            mid = 0.5 * (u[:-1] + u[1:])
            hz = (k[:, None] / mid[None, :]) / (1.0 + np.exp(-k[:, None] * (np.log(mid)[None, :] - lt[:, None])))
            tot = hz.sum(axis=0)
            share = np.where(tot > 0, hz / np.where(tot > 0, tot, 1.0), 1.0 / nc)
            f += (share * np.where(lost > 0, lost, 0.0)[None, :]).sum(axis=1)
            s = float(surv[-1])
            t = target
        stopped = stop is not None and tj >= stop
        rows[0, j] = 0.0 if stopped else s
        rows[1:1 + nc, j] = f
        rows[nc + 1, j] = s if stopped else 0.0
    return rows


def forward(decl, P, rows, targets, times=None, levels=LEVELS):
    out = {}
    tdecl = {t["name"]: t for t in decl["targets"]}
    for n, case in enumerate(rows, 1):
        cid = str(case.get("case_id") or case.get("test") or f"line{n}")
        z, reasons = features(decl, P, case)
        res = {}
        for tn in targets:
            t = tdecl[tn]
            if reasons:
                res[tn] = dict(kind=t["kind"], admitted=False, reasons=list(reasons))
                continue
            if t["kind"] == "positive":
                m, sd = net(P, f"{tn}.median", z), net(P, f"{tn}.spread", z)
                if not (sd > 0):
                    res[tn] = dict(kind="positive", admitted=False, reasons=[f"target {tn}: spread not positive"])
                    continue
                w, kk = float(P[f"{tn}.tail_weight"]), float(P[f"{tn}.tail_width"])
                res[tn] = dict(kind="positive", admitted=True, reasons=[],
                               quantiles={str(l): math.exp(m + mixture_ln_quantile(l, sd, w, kk)) for l in levels}, ln_median=m, ln_sd=sd)
            elif t["kind"] == "binary":
                lg = net(P, f"{tn}.logit", z)
                res[tn] = dict(kind="binary", admitted=True, reasons=[], probability=1.0 / (1.0 + math.exp(-lg)) if lg > -700 else 0.0)
            else:
                clocks = [(net(P, f"{tn}.{c}.median", z), float(P[f"{tn}.{c}.shape"])) for c in t["causes"]]
                tau = float(case.get("cutoff_s", 0.0) or 0.0)
                tt = sorted(set(float(x) for x in (case.get("times_s") or times or []) if float(x) >= tau))
                passes = []
                sdc = (t.get("scheduled_stop") or {}).get("condition")
                if sdc:
                    v, unit, known = lookup(case, sdc)
                    ow, os_ = P[f"{tn}.stop.offset_w"], P[f"{tn}.stop.offset_s"]
                    tot = float(ow[ow > 0].sum())
                    if known and isinstance(v, (int, float)) and v > 0 and tot > 0:
                        passes = [(float(ow[i]) / tot, float(v) + float(os_[i])) for i in range(len(ow)) if ow[i] > 0]
                if not passes:
                    passes = [(1.0, None)]
                if any(st is not None and st <= tau for _, st in passes):
                    res[tn] = dict(kind="event", admitted=False, reasons=["stop at or before the cutoff"])
                    continue
                table = np.zeros((len(clocks) + 2, len(tt)))
                for wgt, st in passes:
                    table += wgt * event_table(clocks, tau, st, tt, int(P["num.substeps"]))
                cats = ["active"] + list(t["causes"]) + ["scheduled_stop"]
                res[tn] = dict(kind="event", admitted=True, reasons=[], times_s=tt, cdf={c: table[i].tolist() for i, c in enumerate(cats)})
        out[cid] = res
    return out


def cmd_check(a):
    """The Python forward against law_run.py on the slug's train law rows (every target, every case)."""
    import law_inputs as LI
    rows = [json.loads(l) for l in (REPO / f"runs/dev/data_all/work/transfer_{a.slug}/law_train.jsonl").read_text(encoding="utf-8").splitlines() if l.strip()]
    if a.limit:
        rows = rows[:a.limit]
    d = decl_of(a.slug)
    P = params(a.slug, a.release)
    targets = [t["name"] for t in d["targets"]]
    times = [float(x) for x in a.times.split(",")] if a.times else None
    py = forward(d, P, rows, targets, times)
    import law_score
    rs = law_score.run_law(a.slug, a.release, rows, ",".join(targets), times=times, tag="pycheck")
    worst = {}
    for cid, tr in rs.items():
        for tn, s in tr.items():
            p = py[cid][tn]
            if s["admitted"] != p["admitted"]:
                worst[tn] = max(worst.get(tn, 0.0), 1.0)
                continue
            if not s["admitted"]:
                continue
            if s["kind"] == "positive":
                e = max(abs(math.log(p["quantiles"][k] / s["quantiles"][k])) for k in s["quantiles"])
            elif s["kind"] == "binary":
                e = abs(p["probability"] - s["probability"])
            else:
                e = max(abs(a_ - b_) for c in s["cdf"] for a_, b_ in zip(p["cdf"][c], s["cdf"][c]))
            worst[tn] = max(worst.get(tn, 0.0), e)
    print(json.dumps(dict(slug=a.slug, cases=len(rows), largest_difference=worst), indent=1))
    return 0


def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    c = sub.add_parser("check")
    c.add_argument("--slug", required=True)
    c.add_argument("--release", required=True)
    c.add_argument("--times", default="")
    c.add_argument("--limit", type=int, default=0)
    c.set_defaults(fn=cmd_check)
    a = ap.parse_args()
    return a.fn(a)


if __name__ == "__main__":
    sys.exit(main())
