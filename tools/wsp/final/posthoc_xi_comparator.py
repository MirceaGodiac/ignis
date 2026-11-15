"""Head-to-head with the droplet forecaster of Xi, Torero & Jahn (2021), rebuilt from their equations.

Post-hoc, 15 November 2026, after the final run of PROTOCOL 12.11. Nothing is adopted or refused.

Their droplet-size forecast (sections 2.1, 3 and 5 of the deposited version, UCL Discovery 10106831):
  R(t)^2 = R0^2 - K_hat * t                     (Eqs. 2 and 8)
R0 is the droplet's initial radius; K_hat is the one droplet-phase parameter, corrected from the observed
radius by minimising the Euclidean distance between prediction and observation (they use a bisection on
that distance; for this one-parameter quadratic the least-squares minimum is the same point, computed in
closed form here). Observations during thermal swelling are left out ("valid observations start at
0.9 s for n-heptane"); here the valid part starts at the largest diameter seen in the prefix, a rule
stated before any score was computed. Their CFD gas phase and second parameter (B) set the flame radius
only and do not change the droplet forecast, so they are not needed.

Inputs: R0 from the experiment's initial diameter (test sheet); prefix observations (time <= cutoff)
from data/work/records/flex/observations.jsonl, the information every release was given at the cutoff.
Outcomes: only the continuations already revealed and cached by posthoc_point_metrics.py; this script
never calls reveal. A case with fewer than 3 valid prefix points gets no forecast and is counted.

Windows: "xi" = their task (cutoff 5 s, scored on 5 < t <= 10 s); "ignis" = the evaluator's window
(cutoffs 1, 3, 5 s, scored on cutoff < t <= cutoff + 10 s). Same cases and observation times for every
model. Paired differences: 95 % bootstrap over sessions, as posthoc_point_metrics.

Usage: python tools/wsp/final/posthoc_xi_comparator.py
"""
import json
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import posthoc_point_metrics as P  # noqa: E402

ROOT = P.ROOT
REC = ROOT / "data/work/records/flex"
OUT = ROOT / "runs/final/20261115/posthoc_xi_comparator.json"
LOCAL = P.LOCAL / "xi_comparator"
FAMILIES = ["flex_heptane", "flex_methanol"]


def load_prefixes(run_ids):
    obs = {r: [] for r in run_ids}
    with open(REC / "observations.jsonl", encoding="utf8") as f:
        for line in f:
            o = json.loads(line)
            r = o.get("run_id")
            if r in obs and o.get("quantity") == "liquid_diameter" and o.get("valid", True) \
                    and o.get("value_si") is not None and o.get("time_s") is not None:
                obs[r].append((o["time_s"], o["value_si"]))
    for r in obs:
        obs[r].sort()
    d0 = {}
    with open(REC / "experiments.jsonl", encoding="utf8") as f:
        for line in f:
            e = json.loads(line)
            if e["id"] in obs:
                c = e["conditions"].get("initial_diameter") or {}
                d0[e["id"]] = c.get("value_si")
    return obs, d0


def xi_fit(prefix, d0, cutoff):
    """K_hat by least squares on R^2 = R0^2 - K t over the valid prefix; None when not possible."""
    pts = [(t, d) for t, d in prefix if 0 <= t <= cutoff]
    if not pts or not d0:
        return None
    i_peak = max(range(len(pts)), key=lambda i: pts[i][1])
    valid = pts[i_peak:]
    if len(valid) < 3:
        return None
    r0 = d0 / 2
    # minimise sum (sqrt(r0^2 - K t) - r)^2 ~ linearised on r^2 for the closed form, then one Newton refinement
    num = sum(t * (r0 * r0 - (d / 2) ** 2) for t, d in valid)
    den = sum(t * t for t, _ in valid)
    if den <= 0:
        return None
    k = num / den
    for _ in range(20):  # Newton on the radius residuals (their objective)
        g = h = 0.0
        for t, d in valid:
            s = r0 * r0 - k * t
            if s <= 0:
                continue
            rm = s ** 0.5
            dr = -t / (2 * rm)
            g += (rm - d / 2) * dr
            h += dr * dr
        if h == 0:
            break
        step = g / h
        k -= step
        if abs(step) < 1e-16:
            break
    return {"r0": r0, "k": k, "n_valid": len(valid), "t_valid_from": valid[0][0]}


def xi_metrics(fit, cont, t_hi):
    t, y = [], []
    for o in cont.get("observations", []):
        if not o.get("valid", True) or o.get("value_si") is None or o.get("time_s") is None:
            continue
        if o.get("quantity") and o["quantity"] != "liquid_diameter":
            continue
        if cont["cutoff_s"] < o["time_s"] <= t_hi:
            t.append(o["time_s"])
            y.append(o["value_si"])
    if not t:
        return None
    p = [2 * max(fit["r0"] ** 2 - fit["k"] * ti, 0.0) ** 0.5 for ti in t]
    e = [a - b for a, b in zip(p, y)]
    n = len(e)
    rmse = (sum(v * v for v in e) / n) ** 0.5
    return {"n_points": n, "mae_um": sum(abs(v) for v in e) / n * 1e6, "rmse_um": rmse * 1e6,
            "nrmse_mean_pct": 100 * rmse / (sum(y) / n),
            "mape_pct": 100 * sum(abs(v) / yi for v, yi in zip(e, y) if yi > 0) / n}


def window(cont, t_hi):
    c = dict(cont)
    c["observations"] = [o for o in cont["observations"] if o.get("time_s") is not None and o["time_s"] <= t_hi]
    return c


def main():
    LOCAL.mkdir(parents=True, exist_ok=True)
    summary = {"what": __doc__.strip().splitlines()[0], "families": {}}
    for fam in FAMILIES:
        spec = P.LIST["families"][fam]
        adopted = spec["release"]
        labels = {adopted: spec["label"], **{(v if isinstance(v, str) else v["id"]): k
                                            for k, v in spec["baselines"].items()}}
        listings = {}
        for rid in labels:
            fc = P.SEALED / fam / f"forecast_shared_{P.short(rid)}.json"
            if fc.exists():
                listings[rid] = {(c["run_id"], c["cutoff_s"]): c for c in
                                 json.loads(fc.read_text(encoding="utf8"))["cases"] if c.get("status") == "artifact"}
        cases = sorted(listings[adopted])
        prefixes, d0 = load_prefixes({r for r, _ in cases})
        fam_out = {}
        for wname, sel, hi in [("xi", lambda c: c == 5, lambda c: 10.0), ("ignis", lambda c: True, lambda c: c + 10.0)]:
            rows = {"Xi et al. 2021 (rebuilt)": []}
            no_fc = []
            for run, cut in cases:
                if not sel(cut):
                    continue
                cont = json.loads((P.CONT / f"{run.replace(':', '_')}__{cut:g}.json").read_text(encoding="utf8"))["continuation"]
                session = run.split(":")[0] + ":" + run.split(":")[-1]
                fit = xi_fit(prefixes.get(run, []), d0.get(run), cut)
                if fit is None:
                    no_fc.append([run, cut])
                    continue
                m = xi_metrics(fit, cont, hi(cut))
                if m is None:
                    continue
                rows["Xi et al. 2021 (rebuilt)"].append({"run": run, "cutoff": cut, "session": session, **fit, **m})
                for rid, lab in labels.items():
                    c = listings.get(rid, {}).get((run, cut))
                    if c is None:
                        continue
                    art = json.loads((P.ART / f"{c['artifact_id'].split(':')[1]}.json").read_text(encoding="utf8"))
                    mm = P.case_metrics(art, window(cont, hi(cut)))
                    if mm:
                        rows.setdefault(lab, []).append({"run": run, "cutoff": cut, "session": session, **mm})
            (LOCAL / f"{fam}_{wname}.json").write_text(json.dumps(rows, indent=1), encoding="utf8")
            xi_rows = rows["Xi et al. 2021 (rebuilt)"]
            keys = {(r["run"], r["cutoff"]) for r in xi_rows}
            res = {"no_forecast_xi": no_fc, "models": {}, "paired_adopted_minus": {}}
            for lab, rr in rows.items():
                rr = [r for r in rr if (r["run"], r["cutoff"]) in keys]
                vals = sorted(r["nrmse_mean_pct"] for r in rr)
                res["models"][lab] = {"n_cases": len(rr), "n_sessions": len({r["session"] for r in rr}),
                                      "nrmse_median": vals[len(vals) // 2] if len(vals) % 2 else
                                      (vals[len(vals) // 2 - 1] + vals[len(vals) // 2]) / 2,
                                      **{k: P.agg(rr, k) for k in ["mae_um", "nrmse_mean_pct"]}}
            ad = [r for r in rows.get(spec["label"], []) if (r["run"], r["cutoff"]) in keys]
            for lab, rr in rows.items():
                if lab != spec["label"]:
                    res["paired_adopted_minus"][lab] = {k: P.paired(ad, rr, k) for k in ["mae_um", "nrmse_mean_pct"]}
            if wname == "ignis":
                res["by_cutoff"] = {}
                for cut in (1.0, 3.0, 5.0):
                    sub = {lab: [r for r in rr if r["cutoff"] == cut and (r["run"], r["cutoff"]) in keys]
                           for lab, rr in rows.items()}
                    res["by_cutoff"][f"{cut:g}"] = {lab: P.agg(v, "mae_um") | {"n": len(v)} for lab, v in sub.items() if v}
            fam_out[wname] = res
        summary["families"][fam] = fam_out
    OUT.write_text(json.dumps(summary, indent=1), encoding="utf8")
    json.dump(summary, sys.stdout, indent=1)


if __name__ == "__main__":
    main()
