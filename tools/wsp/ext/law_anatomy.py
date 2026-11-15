#!/usr/bin/env python3
"""Train anatomy of a seventh-generation W0 law release, for the revision its physicist will type (PROTOCOL 13.2,
12.3). TRAIN cases only: the per-case study files written by `law_score.py look --partition train`, the train
look's result file, and the sensitivity file of law_sensitivity.py. Writes docs/wsp/feedback/<slug>_train_anatomy.md.

    python -I tools/wsp/ext/law_anatomy.py --slug cld [--candidate w0-fable]
"""
import argparse
import json
import math
import statistics
import sys
from collections import defaultdict
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[2]
WORK = REPO / "runs/dev/data_all/work"
Z = {"0.05": 0, "0.25": 1, "0.5": 2, "0.75": 3, "0.95": 4}


def jl(p):
    return [json.loads(l) for l in Path(p).read_text(encoding="utf-8").splitlines() if l.strip()]


def mean(v):
    v = [x for x in v if x is not None and x == x]
    return sum(v) / len(v) if v else None


def fmt(x, nd=3):
    if x is None:
        return "-"
    if isinstance(x, str):
        return x
    return f"{x:.{nd}f}" if abs(x) >= 1e-3 or x == 0 else f"{x:.2e}"


def bins(x, edges, unit=""):
    if x is None:
        return "missing"
    prev = None
    for e in edges:
        if x < e:
            return f"< {e:g}{unit}" if prev is None else f"{prev:g} to {e:g}{unit}"
        prev = e
    return f">= {edges[-1]:g}{unit}"


# ---------------------------------------------------------------------------------------- per-slug definitions
def q_list(f):
    if f is None:
        return None
    if isinstance(f, dict) and "q05" in f:
        return [f[k] for k in ("q05", "q25", "q50", "q75", "q95")]
    if isinstance(f, dict) and "quantiles" in f:
        return f["quantiles"]
    return f


def quantile_rows(rows, loss_fn, y_fn, q_fn, strata, log=True):
    """rows -> per-case dict(loss, base, err (ln median forecast - ln outcome, or median - outcome), in50, in90, strata)."""
    out = []
    for r in rows:
        y, q = y_fn(r), q_fn(r)
        if y is None or q is None or loss_fn(r) is None:
            continue
        if isinstance(y, tuple):    # interval outcome (lo, hi)
            lo, hi = y
            med = q[2]
            err = 0.0 if lo <= med <= hi else (med - hi if med > hi else med - lo)
            in50 = not (q[3] < lo or q[1] > hi)
            in90 = not (q[4] < lo or q[0] > hi)
        else:
            err = (math.log(q[2]) - math.log(y)) if log else (q[2] - y)
            in50, in90 = q[1] <= y <= q[3], q[0] <= y <= q[4]
        out.append(dict(loss=loss_fn(r), err=err, in50=in50, in90=in90, strata={k: f(r) for k, f in strata.items()}))
    return out


def binary_rows(rows, p_fn, y_fn, loss_fn, strata):
    out = []
    for r in rows:
        p, y = p_fn(r), y_fn(r)
        if p is None or y is None or loss_fn(r) is None:
            continue
        out.append(dict(loss=loss_fn(r), p=p, y=float(y), strata={k: f(r) for k, f in strata.items()}))
    return out


def table_quantile(cases, base_by_id=None):
    lines = []
    for sname in cases[0]["strata"] if cases else []:
        g = defaultdict(list)
        for c in cases:
            g[c["strata"][sname]].append(c)
        lines += ["", f"By {sname}:", "", "| stratum | n | mean loss | share of the loss | median error of the median | in 50 % band | in 90 % band |",
                  "|---|---|---|---|---|---|---|"]
        tot = sum(c["loss"] for c in cases) or 1.0
        for k in sorted(g, key=lambda k: -sum(c["loss"] for c in g[k])):
            v = g[k]
            lines.append(f"| {k} | {len(v)} | {fmt(mean([c['loss'] for c in v]))} | {100 * sum(c['loss'] for c in v) / tot:.0f} % | "
                         f"{fmt(statistics.median([c['err'] for c in v]))} | {sum(c['in50'] for c in v)}/{len(v)} | {sum(c['in90'] for c in v)}/{len(v)} |")
    return lines


def table_binary(cases):
    lines = []
    for sname in cases[0]["strata"] if cases else []:
        g = defaultdict(list)
        for c in cases:
            g[c["strata"][sname]].append(c)
        lines += ["", f"By {sname}:", "", "| stratum | n | Brier | share of the loss | mean forecast | observed rate | forecast - observed |",
                  "|---|---|---|---|---|---|---|"]
        tot = sum(c["loss"] for c in cases) or 1.0
        for k in sorted(g, key=lambda k: -sum(c["loss"] for c in g[k])):
            v = g[k]
            mp, my = mean([c["p"] for c in v]), mean([c["y"] for c in v])
            lines.append(f"| {k} | {len(v)} | {fmt(mean([c['loss'] for c in v]))} | {100 * sum(c['loss'] for c in v) / tot:.0f} % | {fmt(mp)} | {fmt(my)} | {fmt(mp - my)} |")
    return lines


def calibration(cases, edges=(0.05, 0.15, 0.3, 0.5, 0.7, 0.85, 0.95)):
    g = defaultdict(list)
    for c in cases:
        g[bins(c["p"], list(edges))].append(c)
    lines = ["", "Calibration (forecast probability bin):", "", "| bin | n | mean forecast | observed rate |", "|---|---|---|---|"]
    order = [f"< {edges[0]:g}"] + [f"{x:g} to {y:g}" for x, y in zip(edges, edges[1:])] + [f">= {edges[-1]:g}"]
    for k in order:
        if k in g:
            v = g[k]
            lines.append(f"| {k} | {len(v)} | {fmt(mean([c['p'] for c in v]))} | {fmt(mean([c['y'] for c in v]))} |")
    return lines


def inp(r, k):
    v = (r.get("inputs") or {}).get(k)
    return v.get("value") if isinstance(v, dict) else v


def slug_sections(slug, rows):
    """[(title, kind, cases)]."""
    if slug == "cld":
        st = {"fuel": lambda r: inp(r, "fuel"),
              "dilution during the test": lambda r: ("dilutes" if (inp(r, "x_fuel_final") or 1) < (inp(r, "x_fuel_initial") or 1) else "no dilution"),
              "lowest fuel fraction of the schedule": lambda r: bins(min([x for s in (inp(r, "schedule_from_ignition_command") or []) for x in s["x_fuel"]] or [None]) if inp(r, "schedule_from_ignition_command") else None, [0.15, 0.25, 0.4, 0.7]),
              "highest coflow of the schedule, m/s": lambda r: bins(max([x for s in (inp(r, "schedule_from_ignition_command") or []) for x in s["v_coflow_m_s"]] or [None]) if inp(r, "schedule_from_ignition_command") else None, [0.15, 0.3, 0.45]),
              "day": lambda r: r["day"]}
        loss = binary_rows(rows, lambda r: (r["forecast"]["loss"] or {}).get("p_loss_before_fuel_off") if r["forecast"]["loss"] else None,
                           lambda r: (1.0 if r["outcome"].get("loss") else 0.0) if "loss" in (r.get("targets") or []) else None,
                           lambda r: r["loss_contribution"].get("loss_brier_any"), st)
        for c, r in zip(loss, [r for r in rows if r["forecast"]["loss"] and "loss" in (r.get("targets") or [])]):
            c["loss"] = mean([v for v in r["loss_contribution"]["loss_brier_by_horizon"].values()])   # mean over the horizons at risk
        rad = quantile_rows([r for r in rows if "radiometer" in (r.get("targets") or [])], lambda r: r["loss_contribution"].get("radiometer_crps_log10"),
                            lambda r: r["outcome"].get("radiometer_log10"), lambda r: q_list(r["forecast"]["radiometer"]),
                            {k: v for k, v in st.items() if k in ("fuel", "day")} | {"fuel fraction of the stage": lambda r: bins(inp(r, "schedule_from_ignition_command") and next((s["x_fuel"][0] for s in inp(r, "schedule_from_ignition_command") if s["stage"] == "constant_flow"), None), [0.2, 0.4, 0.7, 0.99])}, log=False)
        return [("loss (per case: mean Brier over the horizons at risk; calibration of lost before fuel-off)", "binary", loss),
                ("radiometer (CRPS of log10 V; error = median forecast - outcome, in log10 V)", "quantile", rad)]
    if slug == "bre":
        st = {"fuel": lambda r: inp(r, "fuel_species"), "bottle fuel fraction": lambda r: inp(r, "x_fuel_bottle"),
              "burner diameter, m": lambda r: inp(r, "burner_diameter_m"), "ambient O2": lambda r: bins(inp(r, "x_o2_ambient"), [0.25, 0.3, 0.36]),
              "pressure, Pa": lambda r: bins(inp(r, "pressure_pa"), [65000, 85000]), "day": lambda r: r["day"], "cutoff s": lambda r: r["cutoff_s"]}
        out = []
        H = [6.0, 8.0, 10.0, 12.0, 15.0, 20.0, 25.0, 30.0, 40.0, 50.0, 60.0, 80.0, 100.0, 120.0, 150.0, 180.0, 210.0]
        for cause in ("extinction", "operator_termination"):
            cs = []
            for r in rows:
                f = r["forecast"]
                lc = (r["loss_contribution"]["ipcw_integrated_brier"] or {}).get(cause)
                if not f or lc is None or lc != lc:
                    continue
                o = r["outcome"]
                j = H.index(60.0)
                known = (o["censoring"] == "event") or (o["t_lower"] >= 60.0)
                if not known:
                    continue
                y = 1.0 if (o["censoring"] == "event" and o["causes"][0] == cause and o["t_upper"] <= 60.0) else 0.0
                cs.append(dict(loss=lc, p=f["cif"][cause][j], y=y, strata={k: fn(r) for k, fn in st.items()}))
            out.append((f"T1 {cause} (per case: IPCW integrated Brier; forecast and observed = incidence by 60 s after onset)", "binary", cs))
        return out
    if slug == "slice":
        st = {"fuel": lambda r: f"{inp(r, 'fuel_hydrocarbon')} {inp(r, 'x_fuel')}", "tube, mm": lambda r: inp(r, "burner_nominal_mm"),
              "coflow, cm/s": lambda r: bins(inp(r, "u_coflow_cm_s"), [10, 20, 30, 45]), "jet, m/s": lambda r: bins(inp(r, "u_jet_m_s"), [1, 5, 15]),
              "day": lambda r: r["day"]}
        return [(f"{t} (Brier per line)", "binary", binary_rows(rows, lambda r, t=t: r["forecast"][t], lambda r, t=t: r["outcome"].get(t) if r["outcome"].get("eligible", True) else None,
                                                                  lambda r, t=t: r["loss_contribution"][t], st)) for t in ("blow_off", "lifted")]
    if slug == "efield":
        t1 = [r for r in rows if r["target"] == "t1"]
        t2 = [r for r in rows if r["target"] == "t2"]
        st = {"fuel": lambda r: f"{inp(r, 'fuel')} {inp(r, 'x_fuel')}", "burner and polarity": lambda r: f"{inp(r, 'burner')} {inp(r, 'polarity')}",
              "coflow": lambda r: "coflow on" if (inp(r, "u_coflow_m_s") or 0) > 0 else "no coflow", "day": lambda r: r["day"]}
        a = quantile_rows(t1, lambda r: r["loss_contribution"]["crps_ua"], lambda r: max(r["outcome"]["ion_current_ua"], 1e-3), lambda r: r["forecast"],
                          st | {"level |V|, kV": lambda r: bins(abs(r["level"]["v_cmd"]) / 1000 if r.get("level") else None, [1, 2, 4, 6, 8])})
        b = []
        for r in t2:
            f, o = r["forecast"], r["outcome"]
            if not f:
                continue
            bh = r["loss_contribution"]["brier_by_horizon"]
            known = o["event"] or (o.get("observed_s") or 0) >= 60
            if bh and known and (o.get("window_planned_s") or 0) >= 60:
                b.append(dict(loss=mean(list(bh.values())), p=f["cif"]["60"], y=1.0 if (o["event"] and o["time_s"] <= 60) else 0.0,
                              strata={k: fn(r) for k, fn in st.items()}))
        return [("T1 ion current (CRPS uA per level; error = ln median forecast - ln outcome)", "quantile", a),
                ("T2 flame out under the field (per test: mean Brier over the eligible horizons; forecast and observed = out by 60 s, tests observed to 60 s)", "binary", b)]
    if slug in ("samer",):
        st = {"material": lambda r: inp(r, "material"), "measurement": lambda r: inp(r, "measurement"),
              "inlet velocity, m/s": lambda r: inp(r, "inlet_velocity"), "sample temperature, K": lambda r: bins(inp(r, "sample_temperature"), [600, 700, 800]),
              "day": lambda r: r["group"]}
        a = quantile_rows([r for r in rows if r["outcome"]["scored_d10"]], lambda r: r["loss_contribution"]["d10_crps_ln"], lambda r: r["outcome"]["d10_m"],
                          lambda r: r["forecast"]["d10"], st)
        b = binary_rows([r for r in rows if r["outcome"]["scored_iss"]], lambda r: r["forecast"]["iss"], lambda r: 1.0 if r["outcome"]["iss_full_scale"] else 0.0,
                        lambda r: r["loss_contribution"]["iss_brier"], st)
        return [("d10 (CRPS of ln d10; error = ln median forecast - ln outcome)", "quantile", a), ("ISS detector at full scale (Brier)", "binary", b)]
    if slug == "bass1":
        st = {"sample": lambda r: f"{inp(r, 'material_class')} {inp(r, 'configuration')}", "diameter, m": lambda r: inp(r, "diameter_m"),
              "O2": lambda r: bins(inp(r, "x_o2_initial"), [0.15, 0.17, 0.19, 0.21]), "ignition speed, cm/s": lambda r: bins(inp(r, "u_cm_s_at_ignition"), [3, 6, 10, 20]),
              "investigation": lambda r: inp(r, "investigation"), "day": lambda r: r["day"]}
        a = quantile_rows([r for r in rows if r["scored"]], lambda r: r["loss_contribution"]["interval_crps_cm_s"],
                          lambda r: (r["outcome"]["quench_u_lo_cm_s"], r["outcome"]["quench_u_hi_cm_s"]), lambda r: r["forecast"], st, log=False)
        b = binary_rows(rows, lambda r: r.get("forecast_sustain"), lambda r: r["outcome"].get("sustain"), lambda r: r.get("sustain_brier"), st)
        return [("quench_u (interval CRPS, cm/s; error = median forecast minus the nearer end of the outcome interval, 0 inside it)", "quantile", a),
                ("sustain (Brier)", "binary", b)]
    if slug == "spice":
        st = {"fuel": lambda r: inp(r, "fuel"), "nozzle, mm": lambda r: inp(r, "nozzle_id_mm"), "coflow, cm/s": lambda r: bins(inp(r, "u_air_cm_s"), [10, 20, 40]),
              "day": lambda r: r["day"]}
        out = []
        for t in ("T1", "T2"):
            out.append((f"{t} (quantile score x2, mm; error = ln median forecast - ln outcome)", "quantile",
                        quantile_rows(rows, lambda r, t=t: r["loss_contribution"][t], lambda r: r["outcome"]["L_sp_mm"], lambda r, t=t: q_list(r["forecast"][t]), st)))
        return out
    raise SystemExit(slug)


def score_table(path):
    """The score tables of the train look (release beside every baseline)."""
    t = Path(path).read_text(encoding="utf-8")
    keep = []
    for l in t.splitlines():
        if l.startswith("### target") or l.startswith("| ") or l.startswith("|---") or l.startswith("**Reading") or l.startswith("- Asked"):
            keep.append(l)
        if l.startswith("### target") or l.startswith("**Reading"):
            keep.append("")
    return keep


def sensitivity_lines(slug, rid, top=12):
    p = REPO / f"runs/dev/gen7/sensitivity/{slug}_{rid[7:23]}.json"
    if not p.exists():
        return ["", "(sensitivity not computed)"]
    d = json.loads(p.read_text(encoding="utf-8"))
    lines = ["", f"Each typed number carrying weight moved by +{100 * d['rel_step']:g} % of its value (forward difference, Python copy of "
             f"law.rs checked against the Rust forward); change of the train score (negative = better). {d['elements']} numbers; "
             f"base: {json.dumps({k: round(v, 4) for k, v in d['base'].items() if v is not None})}. Full list: `runs/dev/gen7/sensitivity/{p.name}`.", ""]
    for m in d["base"]:
        s = [x for x in d["sensitivity"] if x["delta"].get(m) is not None]
        s.sort(key=lambda x: -abs(x["delta"][m]))
        lines += [f"Largest for {m}:", "", "| number | value | change of the score for +1 % |", "|---|---|---|"]
        lines += [f"| `{x['element']}` | {x['value']:g} | {x['delta'][m]:+.5f} |" for x in s[:top]]
        lines.append("")
    return lines


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--slug", required=True)
    ap.add_argument("--candidate", default="w0-fable")
    a = ap.parse_args()
    rid = json.loads((REPO / f"configs/models/law/{a.slug}/{a.candidate}/release.json").read_text(encoding="utf-8"))["id"]
    rows = jl(WORK / f"transfer_{a.slug}/law_study_{rid[7:23]}.jsonl")
    sect = slug_sections(a.slug, rows)
    look = REPO / f"runs/dev/gen7/train_looks/{a.slug}_{a.candidate}.md"
    adm = json.loads((REPO / "runs/dev/gen7/admission_counts.json").read_text(encoding="utf-8"))[a.slug].get(a.candidate, {}).get("train", {})
    L = [f"# {a.slug}: train anatomy of the W0 law ({a.candidate}, `{rid}`)", "",
         "TRAIN days only, written by `tools/wsp/ext/law_anatomy.py` after the registered look of PROTOCOL 13.2. For the revision "
         "the physicist will type; no number is changed here. Per-case file: "
         f"`runs/dev/data_all/work/transfer_{a.slug}/law_study_{rid[7:23]}.jsonl` (baselines beside it as `law_study_baseline_*.jsonl`).", "",
         "## 1. Score against the baselines (train, on the cases the release answers)", ""] + score_table(look)
    L += ["## 2. Refusals on train (admission, by reason)", ""]
    for t, b in adm.items():
        if isinstance(b, dict):
            L.append(f"- {t}: {b['admitted']} answered, {b['refused']} refused; reasons: {json.dumps(b['reasons'])}"
                     + (f"; by coverage: {json.dumps(b['by_coverage'])}" if b.get("by_coverage") else ""))
    L += ["", "## 3. Where the loss sits, and the sign of the error", ""]
    for title, kind, cases in sect:
        L += [f"### {title}", "", f"{len(cases)} scored train cases."]
        if not cases:
            continue
        if kind == "quantile":
            L += table_quantile(cases)
            L += ["", f"All cases: median error {fmt(statistics.median([c['err'] for c in cases]))}; in the 50 % band "
                  f"{sum(c['in50'] for c in cases)}/{len(cases)}; in the 90 % band {sum(c['in90'] for c in cases)}/{len(cases)}."]
        else:
            L += table_binary(cases) + calibration(cases)
        L.append("")
    L += ["## 4. Sensitivity of the train score to each typed number"] + sensitivity_lines(a.slug, rid)
    out = REPO / f"docs/wsp/feedback/{a.slug}_train_anatomy.md" if a.candidate == "w0-fable" else REPO / f"docs/wsp/feedback/{a.slug}_{a.candidate}_train_anatomy.md"
    out.write_text("\n".join(L) + "\n", encoding="utf-8", newline="\n")
    print("->", out)
    return 0


if __name__ == "__main__":
    sys.exit(main())
