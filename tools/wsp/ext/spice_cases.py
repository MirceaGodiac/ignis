#!/usr/bin/env python3
"""SPICE (PSI-107) forecast cases from the investigators' own smoke-point table, and their score.

Source: `Raw Data/PSI-107_Raw Data_SmokePointReview-0g-3.xlsx`, sheet "Smoke Point Datasheet" (the University
of Maryland review of the video record: one row per smoke point, with the flow settings at that moment and the
luminous flame length at the smoke point, already converted to mm by the investigators' own formulas). Feasibility
and definitions: docs/wsp/feedback/spice_feasibility.md.

Partitions come from docs/wsp/atlas/splits/PSI-107.json (group = GMT day of 2009). Rows of test days, of days
that are not in the split (day 062, the KC-135 and STS rows at the foot of the sheet) are dropped as they are
read: they are never stored, counted or written.

Targets (one row = one smoke point = one case):
  T1  smoke-point flame length `L_sp_mm` from the pre-test conditions (fuel, nozzle, coflow); the fuel flow at
      the smoke point is an outcome of the same search and is NOT an input of T1.
  T2  the same length given the fuel flow at the smoke point (a flame-length law tested on sooty flames whose
      length is well defined).  Its validation inputs are a separate file, to be opened only after the T1
      validation forecasts are frozen, because they carry the T1 secondary outcome (the smoke-point flow).

Writes, under runs/dev/data_all/work/transfer_spice/:
  cases_train.jsonl                       train days: inputs (T1 and T2) and outcomes
  cases_validation_inputs.jsonl           validation days: T1 inputs only
  cases_validation_inputs_given_flow.jsonl validation days: T2 inputs (open after T1 forecasts are frozen)
  cases_validation_outcomes.jsonl         validation outcomes, kept apart; this script never prints them
  baselines_train.json                    baseline scores on train days

Forecast form: quantiles q05, q25, q50, q75, q95 of the target in its unit (mm). Score: quantile score averaged
over the five levels, times two (the five-point approximation of CRPS; a point forecast scores its absolute
error). Aggregate = mean over days of the day mean (each day weighs the same); the pooled case mean is also given.

    python -I tools/wsp/ext/spice_cases.py          (needs openpyxl; run without -I if it is a user-site package)
"""
import json
import math
import statistics
import sys
from collections import defaultdict
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
RAW = REPO / "runs/dev/data_all/raw/PSI-107/Raw Data"
SPD = RAW / "PSI-107_Raw Data_SmokePointReview-0g-3.xlsx"
SPLIT = REPO / "docs/wsp/atlas/splits/PSI-107.json"
OUT = REPO / "runs/dev/data_all/work/transfer_spice"
LEVELS = (0.05, 0.25, 0.50, 0.75, 0.95)
QKEYS = ("q05", "q25", "q50", "q75", "q95")
Z = {0.05: -1.6449, 0.25: -0.6745, 0.50: 0.0, 0.75: 0.6745, 0.95: 1.6449}

# Columns of "Smoke Point Datasheet" (0-based; header rows 3-4 of the sheet, data from row 5).
C = dict(day=0, t_ignite=1, t_off=2, t_data=3, no=4, tp=5, fuel=6, d_mm=7, p_pa=8, mw_mix=9, mw_fuel=10, x_fuel=11,
         fuel_display=12, air_display=13, fan_display=14, rad=15, gain=16, px_len=17, L_sp_mm=18, px_w=19, w_mm=20,
         rho_jet_g_m3=21, tad_k=22, mu_air_g_m_s=23, q_total_sccm=25, q_hc_sccm=26, mdot_total_mg_s=27,
         mdot_hc_mg_s=28, u_air_cm_s=29, u_fuel_cm_s=30, mdot_over_zst=38)

# Stoichiometric moles of air per mole of hydrocarbon (O2 / 0.21), for Roper's S.
AIR_PER_HC = {"ethylene": 3.0 / 0.21, "propane": 5.0 / 0.21, "propylene": 4.5 / 0.21, "methane": 2.0 / 0.21}


def split_map():
    s = json.loads(SPLIT.read_text(encoding="utf-8"))["strata"]["all"]["groups"]
    return {g: part for part, gs in s.items() for g in gs}


def hc_name(fuel_text):
    f = fuel_text.lower()
    for k in AIR_PER_HC:
        if k in f:
            return k
    return None


def num(v):
    return float(v) if isinstance(v, (int, float)) and not isinstance(v, bool) and math.isfinite(v) else None


def read_rows():
    import openpyxl

    parts = split_map()
    wb = openpyxl.load_workbook(SPD, read_only=True, data_only=True)
    ws = wb["Smoke Point Datasheet"]
    kept = []
    for i, r in enumerate(ws.iter_rows(min_row=5, values_only=True), start=5):
        try:
            day = int(float(str(r[C["day"]]).strip()))
        except ValueError:
            continue  # blank, heading, or the KC-135 / STS rows at the foot of the sheet
        part = parts.get(f"PSI-107:gmt:2009-{day:03d}")
        if part not in ("train", "validation"):
            continue  # test day, or a day outside the split: dropped unread
        g = {k: r[j] for k, j in C.items()}
        fuel = str(g["fuel"]).strip()
        zst = None
        if num(g["mdot_total_mg_s"]) and num(g["mdot_over_zst"]):
            zst = g["mdot_total_mg_s"] / g["mdot_over_zst"]
        hc = hc_name(fuel)
        x = num(g["x_fuel"])
        case = {
            "case_id": f"PSI-107:{day:03d}:{g['tp']}",
            "investigation": "PSI-107",
            "day": f"{day:03d}",
            "group_id": f"PSI-107:gmt:2009-{day:03d}",
            "partition": part,
            "source": {"file": SPD.name, "sheet": "Smoke Point Datasheet", "row": i},
            "smoke_point_label": g["tp"],
            "inputs": {
                "fuel": fuel,
                "hydrocarbon": hc,
                "x_fuel": x,
                "mw_jet_g_mol": num(g["mw_mix"]),
                "nozzle_id_mm": num(g["d_mm"]),
                "pressure_pa": num(g["p_pa"]),
                "air_display": num(g["air_display"]),
                "u_air_cm_s": num(g["u_air_cm_s"]),
                "fan_display": num(g["fan_display"]),
                "zst_jet": zst,
                "tad_k": num(g["tad_k"]),
                "rho_jet_g_m3": num(g["rho_jet_g_m3"]),
                "mu_air_mean_g_m_s": num(g["mu_air_g_m_s"]),
                "air_per_jet_mol": (x * AIR_PER_HC[hc]) if (hc and x) else None,
            },
            "inputs_given_flow": {
                "mdot_jet_mg_s": num(g["mdot_total_mg_s"]),
                "mdot_hc_mg_s": num(g["mdot_hc_mg_s"]),
                "q_jet_sccm": num(g["q_total_sccm"]),
                "u_jet_cm_s": num(g["u_fuel_cm_s"]),
            },
            "outcome": {
                "L_sp_mm": num(g["L_sp_mm"]),
                "mdot_sp_mg_s": num(g["mdot_total_mg_s"]),
                "flame_width_mm": num(g["w_mm"]),
            },
        }
        kept.append(case)
    return kept


# ---------------------------------------------------------------- scoring

def qscore(q, y):
    """Five-level quantile score x 2 (CRPS approximation). q: dict q05..q95 (non-decreasing)."""
    s = 0.0
    for tau, k in zip(LEVELS, QKEYS):
        e = y - q[k]
        s += (tau - (1.0 if e < 0 else 0.0)) * e
    return 2.0 * s / len(LEVELS)


def score(forecasts, outcomes, target="L_sp_mm"):
    """forecasts: {case_id: {q05..q95}}; outcomes: {case_id: {"day":..., target: y}}.
    Returns aggregate (day-equal mean), pooled mean, per-day means, n, and cases without a forecast."""
    per_day = defaultdict(list)
    missing = []
    for cid, o in outcomes.items():
        y = o.get(target)
        if y is None:
            continue
        f = forecasts.get(cid)
        if f is None:
            missing.append(cid)
            continue
        qs = [f[k] for k in QKEYS]
        if any(b < a for a, b in zip(qs, qs[1:])):
            raise ValueError(f"quantiles not non-decreasing for {cid}")
        per_day[o["day"]].append(qscore(f, y))
    days = {d: statistics.fmean(v) for d, v in sorted(per_day.items())}
    allv = [x for v in per_day.values() for x in v]
    return {"target": target, "aggregate_day_mean": statistics.fmean(days.values()) if days else None,
            "pooled_mean": statistics.fmean(allv) if allv else None, "n": len(allv),
            "per_day": days, "missing": missing}


# ---------------------------------------------------------------- baselines (train only)

def emp_q(vals):
    v = sorted(vals)
    n = len(v)

    def q(t):
        h = (n - 1) * t
        lo = math.floor(h)
        return v[lo] + (h - lo) * (v[min(lo + 1, n - 1)] - v[lo])
    return {k: q(t) for t, k in zip(LEVELS, QKEYS)}


def lognormal_q(median, sigma):
    return {k: median * math.exp(Z[t] * sigma) for t, k in zip(LEVELS, QKEYS)}


def lin_faeth_coflow_mm(c):
    """Spalding / Lin & Faeth nonbuoyant laminar jet in coflow, unity coefficients:
    L/d = Re/(16 Zst), Re = rho_j u_j d / mu_air(T_mean)  =>  L = mdot/(4 pi mu Zst)."""
    m = c["inputs_given_flow"]["mdot_jet_mg_s"]
    mu, z = c["inputs"]["mu_air_mean_g_m_s"], c["inputs"]["zst_jet"]
    if not (m and mu and z):
        return None
    return (m * 1e-3) / (4 * math.pi * mu * z) * 1e3


def spalding_still_mm(c):
    """Still-air similarity form L/d = 3 Re/(32 Zst) (same symbols)."""
    v = lin_faeth_coflow_mm(c)
    return None if v is None else 1.5 * v


def roper_mm(c):
    """Roper (1977) circular port, empirical form as quoted by Turns: L_f = 1330 Q_F (T_inf/T_F) / ln(1 + 1/S),
    L in m, Q in m^3/s, S = stoichiometric moles of air per mole of jet gas; T_inf/T_F = 1 here."""
    q, s = c["inputs_given_flow"]["q_jet_sccm"], c["inputs"]["air_per_jet_mol"]
    if not (q and s):
        return None
    return 1330.0 * (q * 1e-6 / 60.0) / math.log(1.0 + 1.0 / s) * 1e3


def baselines(train):
    res = {}
    out = {c["case_id"]: {"day": c["day"], "L_sp_mm": c["outcome"]["L_sp_mm"]} for c in train}
    ys = [c["outcome"]["L_sp_mm"] for c in train]
    # B0 pooled climatology, in sample and leave-one-day-out
    q0 = emp_q(ys)
    res["T1_B0_pooled_climatology_in_sample"] = score({c["case_id"]: q0 for c in train}, out)
    lodo = {}
    for c in train:
        rest = [d["outcome"]["L_sp_mm"] for d in train if d["day"] != c["day"]]
        lodo[c["case_id"]] = emp_q(rest)
    res["T1_B0_pooled_climatology_leave_one_day_out"] = score(lodo, out)
    # B1 per-fuel climatology (fallback to pooled below 4 train smoke points of the fuel)
    byf = defaultdict(list)
    for c in train:
        byf[c["inputs"]["fuel"]].append(c["outcome"]["L_sp_mm"])
    f1, f1l = {}, {}
    for c in train:
        v = byf[c["inputs"]["fuel"]]
        f1[c["case_id"]] = emp_q(v) if len(v) >= 4 else q0
        vr = [d["outcome"]["L_sp_mm"] for d in train if d["inputs"]["fuel"] == c["inputs"]["fuel"] and d["day"] != c["day"]]
        f1l[c["case_id"]] = emp_q(vr) if len(vr) >= 4 else lodo[c["case_id"]]
    res["T1_B1_per_fuel_climatology_in_sample"] = score(f1, out)
    res["T1_B1_per_fuel_climatology_leave_one_day_out"] = score(f1l, out)
    # T2 classical laws, at their published constants; point form, then with one log-spread from train residuals
    for name, fn in (("lin_faeth_coflow", lin_faeth_coflow_mm), ("spalding_still_air", spalding_still_mm),
                     ("roper_1977", roper_mm)):
        pts = {c["case_id"]: fn(c) for c in train}
        ok = {k: v for k, v in pts.items() if v}
        res[f"T2_{name}_point"] = score({k: {q: v for q in QKEYS} for k, v in ok.items()}, out)
        r = [math.log(out[k]["L_sp_mm"] / v) for k, v in ok.items()]
        bias = statistics.fmean(r)
        sig = statistics.pstdev(r)
        res[f"T2_{name}_point"]["median_log_ratio_obs_over_law"] = statistics.median(r)
        res[f"T2_{name}_point"]["mean_log_ratio_obs_over_law"] = bias
        res[f"T2_{name}_lognormal_train_sigma"] = score({k: lognormal_q(v, sig) for k, v in ok.items()}, out)
        res[f"T2_{name}_lognormal_train_sigma"]["sigma_ln"] = sig
    return res


def write(path, rows):
    with path.open("w", encoding="utf-8", newline="\n") as fh:
        for r in rows:
            fh.write(json.dumps(r, ensure_ascii=False) + "\n")


def main():
    rows = read_rows()
    train = [c for c in rows if c["partition"] == "train" and c["outcome"]["L_sp_mm"] is not None]
    val = [c for c in rows if c["partition"] == "validation"]
    OUT.mkdir(parents=True, exist_ok=True)
    write(OUT / "cases_train.jsonl", train)
    strip = lambda c, keep: {k: v for k, v in c.items() if k in keep}
    base = ("case_id", "investigation", "day", "group_id", "partition", "source", "smoke_point_label", "inputs")
    write(OUT / "cases_validation_inputs.jsonl", [strip(c, base) for c in val])
    write(OUT / "cases_validation_inputs_given_flow.jsonl", [strip(c, base + ("inputs_given_flow",)) for c in val])
    write(OUT / "cases_validation_outcomes.jsonl",
          [{"case_id": c["case_id"], "day": c["day"], **c["outcome"]} for c in val])
    b = baselines(train)
    (OUT / "baselines_train.json").write_text(json.dumps(b, indent=1), encoding="utf-8")
    days = defaultdict(int)
    for c in train:
        days[c["day"]] += 1
    vdays = defaultdict(int)
    for c in val:
        vdays[c["day"]] += 1
    print(f"train smoke points {len(train)} by day {dict(days)}; validation cases {len(val)} by day {dict(vdays)}")
    for k, v in b.items():
        extra = {x: round(v[x], 4) for x in ("sigma_ln", "median_log_ratio_obs_over_law") if x in v}
        print(f"{k:58s} agg {v['aggregate_day_mean']:.3f} mm  pooled {v['pooled_mean']:.3f} mm  n {v['n']} {extra}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
