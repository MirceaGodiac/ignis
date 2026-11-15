"""Score a published model curve (traced from a paper's figure) and the frozen IGNIS releases on one FLEX run.

Post-hoc, 15 November 2026. Nothing is adopted or refused.

A published simulation runs forward from the initial conditions; it sees no observation of the burn. The
IGNIS releases forecast from a prefix (cutoffs 1, 3, 5 s) up to 10 s ahead. So the table always states both
information budgets, and every model is scored on the same observation times:
  - "published, whole trace": the traced curve against our measurements over the curve's time span
    (what a paper's figure shows);
  - for each cutoff c: every model (the published curve included) on c < t <= c + 10 s.
nRMSE = RMSE / mean measured diameter (Xi et al. 2021), median of the forecast for IGNIS.

The traced curve is JSON: {"source": ..., "figure": ..., "quantity": "d_mm" | "d2_mm2" | "d_over_d0",
"time_shift_s": s, "points": [[t, value], ...]}; the curve's time t maps to our time t + time_shift_s.
Measurements come from data/work/records/flex/observations.jsonl (train and validation runs; a test run's
continuations come from the cache of posthoc_point_metrics.py, without reveal).

Usage: python tools/wsp/final/posthoc_published_curve.py --run PSI-69:... --curve trace.json [--out out.json]
"""
import argparse
import bisect
import json
import pathlib
import subprocess
import sys
import tempfile

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import posthoc_point_metrics as P  # noqa: E402
import posthoc_xi_comparator as X  # noqa: E402

FAMILY_OF = {"heptane": "flex_heptane", "methanol": "flex_methanol"}


def at(series, t):
    ts = [a for a, _ in series]
    i = bisect.bisect_left(ts, t)
    if i == 0 or i >= len(series):
        return None
    (t0, v0), (t1, v1) = series[i - 1], series[i]
    return v0 + (v1 - v0) * (t - t0) / (t1 - t0)


def curve_diameters(curve, d0):
    q = curve.get("quantity", "d_mm")
    out = []
    for t, v in curve["points"]:
        if q == "d_mm":
            d = v * 1e-3
        elif q == "d2_mm2":
            d = (max(v, 0.0) ** 0.5) * 1e-3
        elif q == "d_over_d0":
            d = v * d0
        elif q == "d2_over_d02":
            d = (max(v, 0.0) ** 0.5) * d0
        else:
            raise SystemExit(f"unknown quantity {q}")
        out.append((t + curve.get("time_shift_s", 0.0), d))
    return sorted(out)


def score(pred_at, obs, lo, hi):
    e, y = [], []
    for t, d in obs:
        if lo < t <= hi:
            p = pred_at(t)
            if p is not None:
                e.append(p - d)
                y.append(d)
    if len(e) < 5:
        return None
    rmse = (sum(v * v for v in e) / len(e)) ** 0.5
    return {"n": len(e), "nrmse_pct": 100 * rmse / (sum(y) / len(y)), "mae_um": 1e6 * sum(abs(v) for v in e) / len(e)}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--run", required=True)
    ap.add_argument("--curve", required=True)
    ap.add_argument("--out")
    a = ap.parse_args()
    curve = json.loads(pathlib.Path(a.curve).read_text(encoding="utf8"))
    exp = None
    for line in open(X.REC / "experiments.jsonl", encoding="utf8"):
        e = json.loads(line)
        if e["id"] == a.run:
            exp = e
            break
    if exp is None:
        raise SystemExit("run not found")
    fuel = str(exp["fuel_or_material"]).lower()
    fam = next(v for k, v in FAMILY_OF.items() if k in fuel)
    pre, d0 = X.load_prefixes({a.run})
    obs = pre[a.run]
    d0 = d0[a.run]
    cd = curve_diameters(curve, d0)
    spec = P.LIST["families"][fam]
    releases = {spec["label"]: spec["release"],
                **{k: (v if isinstance(v, str) else v["id"]) for k, v in spec["baselines"].items()}}
    res = {"run": a.run, "family": fam, "curve": {k: curve[k] for k in curve if k != "points"},
           "published_whole_trace": score(lambda t: at(cd, t), obs, cd[0][0] - 1e-9, cd[-1][0]), "cutoffs": {}}
    with tempfile.TemporaryDirectory() as td:
        for cut in (1, 3, 5):
            row = {"published (forward, no prefix)": score(lambda t: at(cd, t), obs, cut, cut + 10)}
            fit = X.xi_fit(obs, d0, cut)
            if fit:
                row["Xi et al. 2021 (rebuilt)"] = score(
                    lambda t: 2 * max(fit["r0"] ** 2 - fit["k"] * t, 0.0) ** 0.5, obs, cut, cut + 10)
            for lab, rid in releases.items():
                f = pathlib.Path(td) / f"{abs(hash(lab))}_{cut}.json"
                r = subprocess.run([str(P.IGNIS), "infer", "forecast", "--release", rid, "--run", a.run,
                                    "--cutoff", str(cut), "--settings", '{"horizon_s": 10}', "--out", str(f)],
                                   cwd=P.ROOT, capture_output=True, text=True)
                if r.returncode != 0 or not f.exists():
                    row[lab] = {"refused": r.stderr.strip()[:200]}
                    continue
                art = json.loads(f.read_text(encoding="utf8"))
                art = art.get("artifact", art)
                lv = art["quantile_levels"]
                med = list(zip(art["times_s"], art["quantiles"][lv.index(0.5)]))
                row[lab] = score(lambda t: at(med, t) if med[0][0] <= t <= med[-1][0] else None, obs, cut, cut + 10)
            res["cutoffs"][str(cut)] = row
    txt = json.dumps(res, indent=1)
    if a.out:
        pathlib.Path(a.out).write_text(txt, encoding="utf8")
    print(txt)


if __name__ == "__main__":
    main()
