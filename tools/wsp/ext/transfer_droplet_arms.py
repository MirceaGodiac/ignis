#!/usr/bin/env python3
"""Scoring of a registered droplet transfer arm (PROTOCOL 12.7) with the chain of
`tools/wsp/ext/transfer_droplet.py` (prefixes, `ignis wsp forward`, the evaluator's case metrics). Writes
under `runs/dev/data_all/work/transfer_droplet/<arm>/`: `aggregate.json` (aggregates only),
`cases_all.json` (per case, every partition: for the scoring agent, never handed to an author) and,
with `--train-cases`, the train-day case file of the study step (PROTOCOL 12.3; train days only).

Baselines computed by this file, fixed before any transfer series was scored:
  * persistence: `transfer_droplet.persistence` (last valid diameter carried forward, a point mass).
  * the classical d-squared law: D^2 fitted by ordinary least squares against time over the valid
    liquid diameters of the prefix with 0 <= t <= cutoff; the fitted D^2 at the cutoff decreases at the
    fitted rate K = -slope (K < 0 is set to 0) over the horizon, D = sqrt(max(D^2, 0)); a point mass
    (CRPS = absolute error). An evaluation baseline, not a release.
Aggregates are case means; `days` are the session ids. Paired difference: per case, other minus
reference, on the cases both answered and every baseline is defined; interval: days resampled with
replacement, pooled case mean recomputed, percentile 95 %, 10000 resamples, seed 20261115.

    python tools/wsp/ext/transfer_droplet_arms.py --arm cfi --cases runs/dev/data_all/work/transfer_droplet/cfi/cases.jsonl \
        --release candidate=<id> --release root=<dir> --pair candidate:root --ignis target-merged/release/ignis.exe \
        --train-cases runs/dev/data_all/work/transfer_droplet/cfi/train_days_cases.jsonl
"""
import argparse
import json
import math
import random
import sys
from collections import Counter, defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import transfer_droplet as td  # noqa: E402

KEYS = ("crps", "mae_median", "coverage@0.5", "coverage@0.9")
N_BOOT, SEED = 10000, 20261115


def d2_law(prefix, cont):
    c = prefix["cutoff_s"]
    pts = [(o["time_s"], o["value_si"] ** 2) for o in prefix["observations"]
           if o["quantity"] == "liquid_diameter" and td.measurement_valid(o) and o["value_si"] > 0 and 0.0 <= o["time_s"] <= c]
    if len(pts) < 3:
        return None
    n = len(pts)
    mt = sum(t for t, _ in pts) / n
    mq = sum(q for _, q in pts) / n
    sxx = sum((t - mt) ** 2 for t, _ in pts)
    if sxx <= 0:
        return None
    b = sum((t - mt) * (q - mq) for t, q in pts) / sxx
    k = max(-b, 0.0)
    qc = mq + b * (c - mt)
    e = [abs(math.sqrt(max(qc - k * (o["time_s"] - c), 0.0)) - o["value_si"]) for o in cont["observations"]
         if o["valid"] and o["quantity"] == "liquid_diameter" and o["value_si"] is not None and c < o["time_s"] <= c + td.HORIZON_S]
    return dict(crps=td.mean(e), mae_median=td.mean(e), n_points=len(e), k_m2_s=k) if e else None


def release_id(x):
    p = td.REPO / x / "release.json"
    return json.loads(p.read_text(encoding="utf-8"))["id"] if p.exists() else x


def summarise(rows, roles, sel):
    out = {}
    for role in roles:
        cs = [r for r in rows if sel(r) and r["metrics"][role]["scored"]]
        m = dict(cases=len(cs), tests=len({r["run_id"] for r in cs}), days=len({r["session_id"] for r in cs}),
                 refused=sum(1 for r in rows if sel(r) and not r["metrics"][role]["admitted"]),
                 unscored=sum(1 for r in rows if sel(r) and r["metrics"][role]["admitted"] and not r["metrics"][role]["scored"]))
        for k in KEYS:
            v = [r["metrics"][role]["metrics"][k] for r in cs if k in r["metrics"][role]["metrics"]]
            m[k] = td.mean(v) if v else None
        out[role] = m
    for b in ("persistence", "d2_law"):
        cs = [r for r in rows if sel(r) and r[b]]
        out[b] = dict(cases=len(cs), tests=len({r["run_id"] for r in cs}), days=len({r["session_id"] for r in cs}),
                      crps=td.mean([r[b]["crps"] for r in cs]) if cs else None,
                      mae_median=td.mean([r[b]["mae_median"] for r in cs]) if cs else None)
    return out


def common_block(rows, roles, sel):
    cs = [r for r in rows if sel(r) and all(r["metrics"][x]["scored"] for x in roles) and r["persistence"] and r["d2_law"]]
    out = dict(cases=len(cs), tests=len({r["run_id"] for r in cs}), days=len({r["session_id"] for r in cs}))
    for role in roles:
        out[role] = {k: td.mean([r["metrics"][role]["metrics"][k] for r in cs]) if cs else None for k in KEYS}
    for b in ("persistence", "d2_law"):
        out[b] = dict(crps=td.mean([r[b]["crps"] for r in cs]) if cs else None)
    return out, cs


def crps_of(r, who):
    return r[who]["crps"] if who in ("persistence", "d2_law") else r["metrics"][who]["metrics"]["crps"]


def paired(cs, a, b):
    if not cs:
        return None
    d = [(r["session_id"], crps_of(r, a) - crps_of(r, b)) for r in cs]
    by = defaultdict(list)
    for s, x in d:
        by[s].append(x)
    days = sorted(by)
    rng = random.Random(SEED)
    stats = []
    for _ in range(N_BOOT):
        pick = [days[rng.randrange(len(days))] for _ in days]
        v = [x for s in pick for x in by[s]]
        stats.append(sum(v) / len(v))
    stats.sort()
    return dict(other=a, reference=b, cases=len(d), days=len(days), mean_difference=td.mean([x for _, x in d]),
                ci95_day_bootstrap=[stats[int(0.025 * N_BOOT)], stats[int(0.975 * N_BOOT) - 1]],
                days_where_other_lower=sum(1 for s in days if sum(by[s]) < 0))


def per_day(rows, roles):
    out = {}
    for s in sorted({r["session_id"] for r in rows}):
        rs = [r for r in rows if r["session_id"] == s]
        d = dict(partition=rs[0]["partition"], tests=len({r["run_id"] for r in rs}))
        for role in list(roles) + ["persistence", "d2_law"]:
            v = [crps_of(r, role) for r in rs if (r[role] if role in ("persistence", "d2_law") else r["metrics"][role]["scored"])]
            d[role] = dict(cases=len(v), crps=td.mean(v) if v else None)
        out[s] = d
    return out


def anatomy(rows, roles):
    """Train-day tests only: bias of the median against lead time, centre against spread."""
    out = {}
    for rid in sorted({r["run_id"] for r in rows}):
        rs = [r for r in rows if r["run_id"] == rid]
        t = {}
        for role in roles:
            bins = {"0-2": [], "2-5": [], "5-10": []}
            absd, half90, cov = [], [], []
            for r in rs:
                a = r["artifacts"][role]
                if not r["metrics"][role]["scored"]:
                    continue
                lv = a["quantile_levels"]
                q50, q05, q95 = (a["quantiles"][lv.index(x)] for x in (0.5, 0.05, 0.95))
                c = r["cutoff_s"]
                for o in r["cont"]["observations"]:
                    if not (o["valid"] and o["quantity"] == "liquid_diameter" and o["value_si"] is not None
                            and c < o["time_s"] <= a["times_s"][-1] and o["time_s"] >= a["times_s"][0]):
                        continue
                    m = td.interp(o["time_s"], a["times_s"], q50)
                    lo, hi = td.interp(o["time_s"], a["times_s"], q05), td.interp(o["time_s"], a["times_s"], q95)
                    e = m - o["value_si"]
                    lead = o["time_s"] - c
                    bins["0-2" if lead <= 2 else "2-5" if lead <= 5 else "5-10"].append(e)
                    absd.append(abs(e))
                    half90.append((hi - lo) / 2)
                    cov.append(1.0 if lo <= o["value_si"] <= hi else 0.0)
            t[role] = dict(median_bias_um_by_lead={k: (td.mean(v) * 1e6 if v else None) for k, v in bins.items()},
                           mean_abs_error_um=td.mean(absd) * 1e6 if absd else None,
                           mean_half_width_90_um=td.mean(half90) * 1e6 if half90 else None,
                           coverage_90=td.mean(cov) if cov else None)
        out[rid] = t
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--arm", required=True)
    ap.add_argument("--cases", required=True)
    ap.add_argument("--release", action="append", required=True, help="role=release id or release folder")
    ap.add_argument("--pair", action="append", default=[], help="other:reference (roles, persistence, d2_law)")
    ap.add_argument("--ignis")
    ap.add_argument("--wsp-dev")
    ap.add_argument("--workers", type=int, default=8)
    ap.add_argument("--train-cases")
    a = ap.parse_args()
    binaries = td.use_binaries(a)
    rel = {x.split("=", 1)[0]: release_id(x.split("=", 1)[1]) for x in a.release}
    rows_in = [json.loads(l) for l in Path(a.cases).read_text(encoding="utf-8").splitlines() if l.strip()]
    meta = {r["run_id"]: r for r in rows_in}
    work = td.OUT / a.arm
    rows, nar, pre_ref = td.run_cases([td.ext_case_inputs(r) for r in rows_in], rel, work, a.workers)
    spec = td.score_rows(rows, rel)
    for r in rows:
        r["partition"] = meta[r["run_id"]].get("partition")
        p = json.loads(r["prefix_path"].read_text(encoding="utf-8"))
        r["d2_law"] = d2_law(p, r["cont"])
    roles = list(rel)
    scopes = {"pooled": lambda r: True, "train": lambda r: r["partition"] == "train",
              "validation": lambda r: r["partition"] == "validation"}
    refusals = {role: dict(Counter(r["metrics"][role]["refusal"] for r in rows if not r["metrics"][role]["admitted"]))
                for role in roles}
    unscored = {role: dict(Counter(r["metrics"][role]["reason"] for r in rows
                                   if r["metrics"][role]["admitted"] and not r["metrics"][role]["scored"])) for role in roles}
    agg = dict(arm=a.arm, binaries=binaries, settings=td.SETTINGS, cutoffs_s=td.CUTOFFS, spec=spec, releases=rel,
               tests_in_file=len(rows_in), cases=len(rows), not_at_risk=nar, prefix_refusals=pre_ref,
               refusals=refusals, unscored=unscored, scopes={}, per_day=per_day(rows, roles))
    for name, sel in scopes.items():
        own = summarise(rows, roles, sel)
        com, cs = common_block(rows, roles, sel)
        agg["scopes"][name] = dict(own_cases=own, common_cases=com,
                                   paired=[paired(cs, *p.split(":")) for p in a.pair])
    td.write_json(work / "aggregate.json", agg)
    td.write_json(work / "cases_all.json", dict(note="per case, all partitions: scoring agent only", cases=[
        dict(run_id=r["run_id"], session_id=r["session_id"], partition=r["partition"], cutoff_s=r["cutoff_s"],
             persistence=r["persistence"], d2_law=r["d2_law"], releases=r["metrics"]) for r in rows]))
    if a.train_cases:
        tr = [r for r in rows if r["partition"] == "train"]
        with Path(a.train_cases).open("w", encoding="utf-8", newline="\n") as f:
            for r in tr:
                m = meta[r["run_id"]]
                c = r["cutoff_s"]
                target = [dict(time_s=o["time_s"], value_si=o["value_si"]) for o in r["cont"]["observations"]
                          if o["valid"] and o["quantity"] == "liquid_diameter" and o["value_si"] is not None
                          and c < o["time_s"] <= c + td.HORIZON_S]
                fc = {}
                for role in roles:
                    x = r["artifacts"][role]
                    fc[role] = dict(release=rel[role], admitted=r["metrics"][role]["admitted"], refusal=r["metrics"][role]["refusal"],
                                    times_s=x.get("times_s"), quantile_levels=x.get("quantile_levels"), quantiles=x.get("quantiles"),
                                    metrics=r["metrics"][role]["metrics"], scored=r["metrics"][role]["scored"])
                f.write(json.dumps(dict(run_id=r["run_id"], test=m.get("test"), session_id=r["session_id"], partition="train",
                                        cutoff_s=c, fuel=m.get("fuel"), conditions=m.get("condition_summary"),
                                        target=target, forecasts=fc, persistence=r["persistence"], d2_law=r["d2_law"]),
                                   ensure_ascii=False) + "\n")
        td.write_json(work / "anatomy_train.json", anatomy(tr, roles))
    print(json.dumps({k: v for k, v in agg.items() if k not in ("per_day",)}, indent=1, default=str)[:6000])
    return 0


if __name__ == "__main__":
    sys.exit(main())
