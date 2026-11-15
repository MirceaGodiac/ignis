"""Point-error metrics (MAE, RMSE, normalised RMSE, MAPE) of the frozen FLEX releases on the sealed test
partition, for comparison with published droplet studies.

Post-hoc look on 15 November 2026, after the final run of PROTOCOL 12.11. Nothing is adopted or refused.
The forecasts are the sealed artifacts of the final run (runs/evaluation/artifacts); every continuation is
opened through `ignis eval reveal --final-test`, so each opening is in runs/evaluation/reveal_log.jsonl.
Per-case rows go to runs/final/20261115/local/posthoc/ only; stdout and the summary hold aggregates.

Metrics per case, on the continuation observations inside the forecast window (cutoff < t <= last
forecast time), using the forecast median (quantile 0.5) interpolated linearly to observation times:
  mae_um, rmse_um, nrmse_mean_pct (RMSE / mean measured diameter: the metric of Xi, Torero & Jahn 2021),
  mape_pct.
Aggregates: case mean, and session-weighted mean (mean of session means, as the evaluator's estimate).
Paired differences: adopted minus comparator, 95 % bootstrap over sessions (2000 draws, seed 0).
"""
import json
import pathlib
import random
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[3]
SEALED = ROOT / "runs/final/20261115/local/final/sealed"
ART = ROOT / "runs/evaluation/artifacts"
LOCAL = ROOT / "runs/final/20261115/local/posthoc"
CONT = LOCAL / "continuations"
IGNIS = ROOT / "target-merged/release/ignis.exe"
LIST = json.loads((ROOT / "tools/wsp/final/releases_final_20261115.json").read_text(encoding="utf8"))

FAMILIES = ["flex_methanol", "flex_heptane"]


def short(rid):
    return rid.split(":")[1][:16]


def interp(x, xs, ys):
    if x <= xs[0]:
        return ys[0]
    if x >= xs[-1]:
        return ys[-1]
    lo, hi = 0, len(xs) - 1
    while hi - lo > 1:
        mid = (lo + hi) // 2
        if xs[mid] <= x:
            lo = mid
        else:
            hi = mid
    w = (x - xs[lo]) / (xs[hi] - xs[lo])
    return ys[lo] * (1 - w) + ys[hi] * w


def continuation(artifact_id, run_id, cutoff):
    key = f"{run_id.replace(':', '_')}__{cutoff:g}.json"
    p = CONT / key
    if not p.exists():
        r = subprocess.run([str(IGNIS), "eval", "reveal", artifact_id, "--final-test", "--out", str(p)],
                           cwd=ROOT, capture_output=True, text=True)
        if r.returncode != 0 or not p.exists():
            raise SystemExit(f"reveal refused for {run_id} @ {cutoff}: {r.stderr.strip()[:300]}")
    return json.loads(p.read_text(encoding="utf8"))["continuation"]


def case_metrics(art, cont):
    times = art["times_s"]
    levels = art["quantile_levels"]
    med = art["quantiles"][levels.index(0.5)]
    cutoff = cont.get("cutoff_s")
    q = art.get("quantity")
    t, y = [], []
    for o in cont.get("observations", []):
        if not o.get("valid", True) or o.get("value_si") is None or o.get("time_s") is None:
            continue
        if q and o.get("quantity") and o["quantity"] != q:
            continue
        ts = o["time_s"]
        if (cutoff is None or ts > cutoff) and times[0] <= ts <= times[-1]:
            t.append(ts)
            y.append(o["value_si"])
    if not t:
        return None
    p = [interp(ti, times, med) for ti in t]
    e = [pi - yi for pi, yi in zip(p, y)]
    n = len(e)
    mae = sum(abs(v) for v in e) / n
    rmse = (sum(v * v for v in e) / n) ** 0.5
    ybar = sum(y) / n
    mape = sum(abs(v) / yi for v, yi in zip(e, y) if yi > 0) / n
    return {"n_points": n, "mae_um": mae * 1e6, "rmse_um": rmse * 1e6,
            "nrmse_mean_pct": 100 * rmse / ybar, "mape_pct": 100 * mape}


def agg(rows, key):
    vals = [r[key] for r in rows]
    by = {}
    for r in rows:
        by.setdefault(r["session"], []).append(r[key])
    sess = [sum(v) / len(v) for v in by.values()]
    return {"case_mean": sum(vals) / len(vals), "session_weighted": sum(sess) / len(sess)}


def paired(a_rows, b_rows, key, n_boot=2000, seed=0):
    b = {(r["run"], r["cutoff"]): r for r in b_rows}
    pairs = [(r["session"], r[key] - b[(r["run"], r["cutoff"])][key]) for r in a_rows
             if (r["run"], r["cutoff"]) in b]
    by = {}
    for s, d in pairs:
        by.setdefault(s, []).append(d)
    sessions = list(by)
    est = sum(sum(v) / len(v) for v in by.values()) / len(sessions)
    rng = random.Random(seed)
    boots = []
    for _ in range(n_boot):
        draw = [rng.choice(sessions) for _ in sessions]
        boots.append(sum(sum(by[s]) / len(by[s]) for s in draw) / len(draw))
    boots.sort()
    return {"estimate": est, "ci95": [boots[int(0.025 * n_boot)], boots[int(0.975 * n_boot) - 1]],
            "n_cases": len(pairs), "n_sessions": len(sessions)}


def main():
    LOCAL.mkdir(parents=True, exist_ok=True)
    CONT.mkdir(parents=True, exist_ok=True)
    summary = {"what": __doc__.strip().splitlines()[0], "release_list": "tools/wsp/final/releases_final_20261115.json",
               "families": {}}
    for fam in FAMILIES:
        spec = LIST["families"][fam]
        adopted = spec["release"]
        labels = {adopted: spec["label"], **{v if isinstance(v, str) else v["id"]: k
                                            for k, v in spec["baselines"].items()}}
        per_release = {}
        # the adopted release's artifacts open the continuations; the others reuse them
        order = [adopted] + [r for r in labels if r != adopted]
        for rid in order:
            fc = SEALED / fam / f"forecast_shared_{short(rid)}.json"
            if not fc.exists():
                continue
            listing = json.loads(fc.read_text(encoding="utf8"))
            rows = []
            for c in listing["cases"]:
                if c.get("status") != "artifact":
                    continue
                aid = c["artifact_id"]
                art = json.loads((ART / f"{aid.split(':')[1]}.json").read_text(encoding="utf8"))
                cont = continuation(aid, c["run_id"], c["cutoff_s"])
                m = case_metrics(art, cont)
                if m is None:
                    continue
                session = c["run_id"].split(":")[0] + ":" + c["run_id"].split(":")[-1]
                rows.append({"run": c["run_id"], "cutoff": c["cutoff_s"], "session": session, **m})
            per_release[rid] = rows
        (LOCAL / f"{fam}_cases.json").write_text(json.dumps(per_release, indent=1), encoding="utf8")
        fam_out = {"releases": {}, "paired_vs_adopted": {}}
        for rid, rows in per_release.items():
            fam_out["releases"][labels[rid]] = {
                "n_cases": len(rows), "n_sessions": len({r["session"] for r in rows}),
                **{k: agg(rows, k) for k in ["mae_um", "rmse_um", "nrmse_mean_pct", "mape_pct"]}}
        for rid, rows in per_release.items():
            if rid == adopted:
                continue
            fam_out["paired_vs_adopted"][labels[rid]] = {
                k: paired(per_release[adopted], rows, k) for k in ["mae_um", "nrmse_mean_pct"]}
        summary["families"][fam] = fam_out
    out = ROOT / "runs/final/20261115/posthoc_point_metrics.json"
    out.write_text(json.dumps(summary, indent=1), encoding="utf8")
    json.dump(summary, sys.stdout, indent=1)


if __name__ == "__main__":
    main()
