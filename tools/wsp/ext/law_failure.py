#!/usr/bin/env python3
"""PROTOCOL 13.4 failure analysis (development, in-sample): where the adopted law releases lose to their best
baseline, on train and validation cases together (validation is study data since 13.4; test days stay unread).
Reads the per-case study files of law_score.py (train and `_dev_validation`). Prints markdown tables to stdout."""
import json
import math
import statistics
import sys
from collections import defaultdict
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
W = REPO / "runs/dev/data_all/work"


def jl(p):
    return [json.loads(l) for l in Path(p).read_text(encoding="utf-8").splitlines() if l.strip()]


def m(v):
    v = [x for x in v if x is not None and x == x]
    return sum(v) / len(v) if v else None


def f(x, n=3):
    return "-" if x is None else (f"{x:.{n}f}" if abs(x) >= 1e-3 or x == 0 else f"{x:.1e}")


def load(slug, rid, base):
    out = []
    for part, suf in (("train", ""), ("validation", "_dev_validation")):
        r = jl(W / f"transfer_{slug}/law_study_{rid[7:23]}{suf}.jsonl")
        b = {x["case_id"]: x for x in jl(W / f"transfer_{slug}/law_study_baseline_{base}{suf}.jsonl")}
        for x in r:
            x["part"] = part
            x["base"] = b.get(x["case_id"])
        out += r
    return out


def table(title, rows, strata, loss, bloss, obs, fc, obs_label="observed", fc_label="forecast"):
    print(f"\n#### {title}\n")
    for sn, sf in strata.items():
        g = defaultdict(list)
        for r in rows:
            g[sf(r)].append(r)
        print(f"By {sn}:\n\n| stratum | cases | {obs_label} | {fc_label} | law score | best baseline score | law minus baseline |\n|---|---|---|---|---|---|---|")
        for k in sorted(g, key=lambda k: -sum((loss(r) or 0) - (bloss(r) or 0) for r in g[k])):
            v = g[k]
            a, b = m([loss(r) for r in v]), m([bloss(r) for r in v])
            print(f"| {k} | {len(v)} | {f(m([obs(r) for r in v]))} | {f(m([fc(r) for r in v]))} | {f(a)} | {f(b)} | {f(None if a is None or b is None else a - b)} |")
        print()


def bins(x, e):
    if x is None:
        return "missing"
    p = None
    for v in e:
        if x < v:
            return f"< {v:g}" if p is None else f"{p:g}-{v:g}"
        p = v
    return f">= {e[-1]:g}"


def rid(slug, c):
    return json.loads((REPO / f"configs/models/law/{slug}/{c}/release.json").read_text(encoding="utf-8"))["id"]


def cld():
    rows = [r for r in load("cld", rid("cld", "d1-fable"), "loss_by_dilution_ramp") if r["forecast"]["loss"] and "loss" in (r.get("targets") or [])]
    def lb(r):
        return m(list((r["loss_contribution"].get("loss_brier_by_horizon") or {}).values()))
    def blb(r):
        return m(list(((r["base"] or {}).get("loss_contribution") or {}).get("loss_brier_by_horizon", {}).values()))
    sched = lambda r: [s["stage"] for s in (r["inputs"].get("schedule_from_ignition_command") or [])]  # noqa: E731
    st = {"partition": lambda r: r["part"],
          "procedure: dilution ramp in the script": lambda r: "dilutes" if r["inputs"]["x_fuel_final"] < r["inputs"]["x_fuel_initial"] else "no dilution",
          "procedure: velocity ramps (ramp_1..3)": lambda r: "velocity ramps" if any(s.startswith("ramp_") for s in sched(r)) else "none",
          "procedure: coflow matching ramp": lambda r: "coflow ramp" if "coflow_matching_ramp" in sched(r) else "none",
          "fuel": lambda r: r["inputs"]["fuel"],
          "final fuel fraction": lambda r: bins(r["inputs"]["x_fuel_final"], [0.15, 0.25, 0.4, 0.7]),
          "round": lambda r: r["inputs"].get("round")}
    table("cld loss: per case, mean Brier over the horizons at risk; law = D1, best baseline = incidence by dilution ramp",
          rows, st, lb, blb, lambda r: 1.0 if r["outcome"].get("loss") else 0.0, lambda r: r["forecast"]["loss"]["p_loss_before_fuel_off"],
          "observed loss rate", "forecast P(lost before fuel-off)")
    print("Coverage: dilution flag (x_fuel_final < x_fuel_initial) on", sum(1 for r in rows if r["inputs"].get("x_fuel_initial") is not None), "of", len(rows),
          "cases; schedule stages on", sum(1 for r in rows if sched(r)), "of", len(rows))


def bass1():
    rows = [r for r in load("bass1", rid("bass1", "d1-fable"), "Bhattacharjee_et_al._2016") if r.get("scored") and r["forecast"]]
    lo = lambda r: r["outcome"]["quench_u_lo_cm_s"]  # noqa: E731
    st = {"partition": lambda r: r["part"], "sample": lambda r: f"{r['inputs']['material_class']} {r['inputs']['configuration']}",
          "thickness, mm": lambda r: bins((r["inputs"].get("thickness_m") or 0) * 1000 if r["inputs"].get("thickness_m") else None, [0.5, 2.5, 4]),
          "procedure: speed at ignition, cm/s": lambda r: bins(r["inputs"].get("u_cm_s_at_ignition"), [5, 10, 20]),
          "procedure: earlier burns of the sample": lambda r: r["inputs"].get("prior_burns_of_sample"),
          "investigation": lambda r: r["inputs"]["investigation"], "O2": lambda r: bins(r["inputs"].get("x_o2_initial"), [0.16, 0.18, 0.2])}
    table("bass1 quench_u: interval CRPS cm/s; law = D1, best baseline = Bhattacharjee 2016",
          rows, st, lambda r: r["loss_contribution"]["interval_crps_cm_s"],
          lambda r: ((r["base"] or {}).get("loss_contribution") or {}).get("interval_crps_cm_s"),
          lambda r: 0.5 * (r["outcome"]["quench_u_lo_cm_s"] + r["outcome"]["quench_u_hi_cm_s"]), lambda r: r["forecast"][2],
          "observed interval midpoint, cm/s", "forecast median, cm/s")


def bre():
    rows = load("bre", rid("bre", "w0-fable"), "planned_stop")
    def l(r, c="extinction"):
        return ((r["loss_contribution"] or {}).get("ipcw_integrated_brier") or {}).get(c)
    def bl(r, c="extinction"):
        return (((r["base"] or {}).get("loss_contribution") or {}).get("ipcw_integrated_brier") or {}).get(c)
    rows = [r for r in rows if r["forecast"] and l(r) is not None and l(r) == l(r)]
    i = lambda r, k: r["inputs"].get(k)  # noqa: E731
    st = {"partition": lambda r: r["part"],
          "procedure: planned stop, s": lambda r: bins(i(r, "planned_stop_s"), [100, 150, 250]),
          "procedure: extinction detection stops the test": lambda r: i(r, "extinction_detection_stops_test"),
          "procedure: era (script version)": lambda r: i(r, "era"),
          "pressure, kPa": lambda r: bins((i(r, "pressure_pa") or 0) / 1000, [65, 85]),
          "ambient O2": lambda r: bins(i(r, "x_o2_ambient"), [0.25, 0.3, 0.36]),
          "fuel and bottle": lambda r: f"{i(r, 'fuel_species')} {i(r, 'x_fuel_bottle')}",
          "burner, m": lambda r: i(r, "burner_diameter_m")}
    H = [6, 8, 10, 12, 15, 20, 25, 30, 40, 50, 60, 80, 100, 120, 150, 180, 210]
    j = H.index(60)
    table("bre T1 extinction: per case IPCW integrated Brier; law = W0, best baseline = planned stop; observed and forecast = extinction by 60 s",
          rows, st, l, bl, lambda r: 1.0 if (r["outcome"]["censoring"] == "event" and r["outcome"]["causes"][0] == "extinction" and r["outcome"]["t_upper"] <= 60) else 0.0,
          lambda r: r["forecast"]["cif"]["extinction"][j], "observed ext. by 60 s", "forecast ext. by 60 s")


def slice_():
    rows = load("slice", rid("slice", "w0-fable"), "incidence_pooled")
    rows = [r for r in rows if r["loss_contribution"]["lifted"] is not None]
    # the step from the previous line of the same flame (conditions of earlier lines only)
    allr = sorted(rows, key=lambda r: (r["part"], r["day"], str(r.get("test")), r.get("step_in_test") or 0))
    prev = {}
    for a, b in zip(allr, allr[1:]):
        if (a["part"], a["day"], a.get("test")) == (b["part"], b["day"], b.get("test")):
            prev[b["case_id"]] = a
    def step(r):
        p = prev.get(r["case_id"])
        if not p:
            return "first line of the flame"
        rc = r["inputs"]["u_coflow_cm_s"] / p["inputs"]["u_coflow_cm_s"] if p["inputs"]["u_coflow_cm_s"] else None
        rj = r["inputs"]["u_jet_m_s"] / p["inputs"]["u_jet_m_s"] if p["inputs"]["u_jet_m_s"] else None
        if rc and rc > 1.05:
            return "coflow stepped up"
        if rj and rj > 1.05:
            return "jet stepped up"
        if (rc and rc < 0.95) or (rj and rj < 0.95):
            return "stepped down"
        return "repeated setting"
    st = {"partition": lambda r: r["part"], "procedure: position in the sequence (step_in_test)": lambda r: bins(r.get("step_in_test"), [2, 4, 7]),
          "procedure: step from the previous line": step, "tube, mm": lambda r: r["inputs"]["burner_nominal_mm"],
          "fuel": lambda r: f"{r['inputs']['fuel_hydrocarbon']} {r['inputs']['x_fuel']}", "day": lambda r: f"{r['part'][:1]} {r['day']}"}
    table("slice lifted: Brier per line; law = W0, best baseline = pooled incidence", rows, st, lambda r: r["loss_contribution"]["lifted"],
          lambda r: ((r["base"] or {}).get("loss_contribution") or {}).get("lifted"), lambda r: r["outcome"]["lifted"], lambda r: r["forecast"]["lifted"])
    st2 = dict(st)
    table("slice blow_off: Brier per line; law = W0, best baseline = pooled incidence", [r for r in rows if r["loss_contribution"]["blow_off"] is not None],
          st2, lambda r: r["loss_contribution"]["blow_off"], lambda r: ((r["base"] or {}).get("loss_contribution") or {}).get("blow_off"),
          lambda r: r["outcome"]["blow_off"], lambda r: r["forecast"]["blow_off"])


def shift():
    print("\n#### Conditions by partition\n")
    for slug, keys in (("cld", ["fuel", "x_fuel_final", "x_fuel_initial", "v_coflow_final_m_s", "round"]),
                       ("bre", ["fuel_species", "x_fuel_bottle", "x_o2_ambient", "pressure_pa", "burner_diameter_m", "planned_stop_s"])):
        tr = jl(W / f"transfer_{slug}/cases_train.jsonl")
        va = jl(W / f"transfer_{slug}/cases_validation_inputs.jsonl")
        print(f"{slug}:\n\n| condition | train | validation |\n|---|---|---|")
        for k in keys:
            def d(rows):
                v = [r["inputs"].get(k) for r in rows if r["inputs"].get(k) is not None]
                if v and isinstance(v[0], (int, float)):
                    return f"n {len(v)}, median {statistics.median(v):.3g}, range {min(v):.3g}-{max(v):.3g}"
                c = defaultdict(int)
                for x in v:
                    c[x] += 1
                return ", ".join(f"{a}: {b}" for a, b in sorted(c.items(), key=lambda z: -z[1]))
            print(f"| {k} | {d(tr)} | {d(va)} |")
        if slug == "cld":
            for rows, nm in ((tr, "train"), (va, "validation")):
                n = sum(1 for r in rows if r["inputs"].get("x_fuel_final") is not None and r["inputs"]["x_fuel_final"] < r["inputs"]["x_fuel_initial"])
                print(f"| tests that dilute ({nm}) | {n} of {len(rows)} | |")
        print()
    tr = jl(W / "transfer_samer/cases_train.jsonl")
    va = jl(W / "transfer_samer/cases_validation_inputs.jsonl")
    for k in ("material", "inlet_velocity", "measurement"):
        c = [defaultdict(int), defaultdict(int)]
        for i, rows in enumerate((tr, va)):
            for r in rows:
                c[i][r["inputs"][k]["value"]] += 1
        print(f"samer {k}: train {dict(c[0])}; validation {dict(c[1])}")


if __name__ == "__main__":
    {"cld": cld, "bass1": bass1, "bre": bre, "slice": slice_, "shift": shift}[sys.argv[1]]()
