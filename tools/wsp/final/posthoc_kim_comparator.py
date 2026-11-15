"""Head-to-head preparation with the published CFI-G burner-flame models, rebuilt from their equations. DRAFT.

Post-hoc, 15 November 2026, after the final run of PROTOCOL 12.11. Nothing is adopted or refused, no IGNIS
weight is read, fitted or tuned. This draft only runs on train and validation records (dry run).

Sources (full text read; PDFs and hashes in docs/wsp/research/kim2023_flames.json):
  K23  Kim et al., Proc. Combust. Inst. 39 (2023) 1647-1656, NSF PAR 10426596 (accepted manuscript).
  KT   M. Kim, M.S. thesis, Univ. of Maryland, 2022 (DRUM 1903/29540); Table 4.2 holds the hot-flame radii,
       Table E.1 maps the cool-flame tests A-G to test points.
  I21  Irace et al., Combust. Flame 229 (2021) 111373, as Chapter 4 of Irace's 2022 WashU dissertation
       (doi 10.7936/kpc0-th51): Eq. 4.10, rhoD = rho0 D0 (Tch/298 K)^(1/2), rho0 = 1146 g/m3 (N2),
       D0 = 20.4 mm2/s (O2 into N2), Tch = mean of 298 K and the flame temperature.
  I23a Irace et al., PCI 39 (2023) 1905-1913 = dissertation Ch. 5: critical reactant mass flux at radiative
       extinction J = m_R / (4 pi r_ext^2) (Eq. 5.1), r_ext = sqrt(m_R / (4 pi J)) (Eqs. 5.2, 5.4);
       J = 0.2 g/m2 s (ethylene, normal), 0.68 g/m2 s (O2, inverse); ethylene, 1 bar only.
  I23b Irace et al., PCI 39 (2023), "On the existence of steady-state ..." = dissertation Ch. 6:
       R = r_f / r_ext (Eq. 6.9), r_f from Eqs. 6.5 (normal) and 6.7 (inverse) evaluated "as in [I21]" with
       Tch = mean of ambient and 1130 K. R > 1: the flame cannot reach steady state and extinguishes;
       R < 1: it should approach steady state.

The two predictors:
  1. Radius (K23 Eqs. 7-8 normal, Eq. 13 inverse): r_hf = m_tot / (4 pi rhoD L), L = ln(1/(1-Zst)) normal,
     ln(1/Zst) inverse. rhoD = 0.0334 g/m s at 300 K (K23 Sec. 4), carried to the flame by the I21 rule
     rhoD(Tch) = rhoD_300 (Tch/300)^(1/2), Tch = (300 K + 1400 K)/2 for hot flames. K23 states the 1400 K and
     0.0334 but not the carrying rule; the I21 rule reproduces the model values K23 plotted in Fig. 7 (see
     reproduce_kim_radii). The radius is the quasi-steady size; neither paper gives r(t).
  2. Extinction occurrence (I23b): extinguishes iff R > 1. It says WHETHER, not WHEN; no published closed form
     gives the extinction time of an ISS flame. J for alkanes is not published: the primary rule uses the
     published numbers as they stand (fuel-based 0.2 for every normal flame, O2-based 0.68 for every inverse
     flame); the sensitivity rule carries the O2-based 0.68 to normal alkane flames by their own stoichiometry
     (J_F = 0.68 / nu_F, the I23a conversion run backwards). Both are transfers, stated as such.

Inputs: data/work/records/acme_{normal,inverse}/experiments.jsonl conditions (mass flow of the burner
reactant, its mole fraction, Zst of the PSI table) and endpoints.jsonl (SE/FT label, burn time), for runs of
the train and validation partitions only. Test runs are skipped before their conditions or endpoints are
parsed. The figure trace comes from docs/wsp/research/kim2023_flames.json.

Usage: python tools/wsp/final/posthoc_kim_comparator.py            (dry run, train + validation)
"""
import glob
import json
import math
import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parents[3]
REC = ROOT / "data/work/records"
TRACE = ROOT / "docs/wsp/research/kim2023_flames.json"
OUT = ROOT / "runs/final/20261115/posthoc_kim_comparator_dryrun.json"
LOCAL = ROOT / "runs/final/20261115/local/posthoc/kim_comparator"
PARTS = ("train", "validation")  # never "test"

W = {"ethane": 30.069, "propane": 44.096, "butane": 58.122, "N2": 28.0134, "O2": 31.998}
NU = {"ethane": 3.5 * 31.998 / 30.069, "propane": 5 * 31.998 / 44.096, "butane": 6.5 * 31.998 / 58.122}

RHOD_300_K23 = 0.0334e-3          # kg/m s at 300 K, K23 Sec. 4
T_HOT_K23 = 1400.0                # K, K23 Sec. 4 (peak hot-flame temperature from I21)
RHO0D0_I21 = 1146e-3 * 20.4e-6    # kg/m s at 298 K, I21 Eq. 4.10
T_CR_I23 = 1130.0                 # K, I23a
J_F_I23 = 0.2e-3                  # kg/m2 s, ethylene, normal, I23a
J_O2_I23 = 0.68e-3                # kg/m2 s, O2, inverse, I23a


def mass_fraction(x, w_species):
    return x * w_species / (x * w_species + (1 - x) * W["N2"])


def l_term(family, zst):
    return math.log(1 / (1 - zst)) if family == "acme_normal" else math.log(1 / zst)


def total_flow(family, m_r, x_r, fuel):
    w = W[fuel] if family == "acme_normal" else W["O2"]
    return m_r / mass_fraction(x_r, w)


def rhod_kim(t_flame=T_HOT_K23, t_amb=300.0):
    return RHOD_300_K23 * math.sqrt(((t_amb + t_flame) / 2) / 300.0)


def rhod_irace(t_flame=T_CR_I23, t_amb=298.0):
    return RHO0D0_I21 * math.sqrt(((t_amb + t_flame) / 2) / 298.0)


def kim_radius(family, m_r, x_r, zst, fuel, rhod=None):
    """K23 Eq. 8 (normal) / Eq. 13 (inverse), metres."""
    rhod = rhod or rhod_kim()
    return total_flow(family, m_r, x_r, fuel) / (4 * math.pi * rhod * l_term(family, zst))


def r_ext(family, m_r, fuel, rule="published"):
    """I23a Eqs. 5.2 / 5.4 (= I23b Eq. 6.4), metres."""
    if family == "acme_inverse":
        j = J_O2_I23
    elif rule == "published":
        j = J_F_I23
    else:  # "o2_equivalent": carry the O2-based flux to the alkane by its stoichiometry
        j = J_O2_I23 / NU[fuel]
    return math.sqrt(m_r / (4 * math.pi * j))


def irace_R(family, m_r, x_r, zst, fuel, rule="published", rhod="irace"):
    """I23b Eq. 6.9. rhod='irace': Eqs. 6.5/6.7 as published; 'kim': K23's rhoD carried to Tch=(300+1130)/2."""
    rd = rhod_irace() if rhod == "irace" else rhod_kim(T_CR_I23)
    return kim_radius(family, m_r, x_r, zst, fuel, rd) / r_ext(family, m_r, fuel, rule)


def split_of(family):
    sp = json.loads(open(glob.glob(str(ROOT / f"data/splits/{family}-*.json"))[0], encoding="utf8").read())["split"]
    part = {g: k for k in ("train", "validation", "test") for g in sp[k]}
    return {r: part[g] for r, g in sp["run_to_group"].items()}


def load_runs(family):
    """Train and validation runs only; a test run is dropped on its id, before anything else is parsed."""
    split = split_of(family)
    runs = {}
    with open(REC / family / "experiments.jsonl", encoding="utf8") as f:
        for line in f:
            rid = line.split('"id": "', 1)[1].split('"', 1)[0]
            if split.get(rid) not in PARTS:
                continue
            e = json.loads(line)
            assert e["id"] == rid, rid
            c = {k: v.get("value_si") for k, v in e["conditions"].items()}
            c["fuel"] = e["conditions"]["fuel_species"]["raw_value"]
            runs[rid] = {"run": rid, "partition": split[rid], "session": e["session_id"], **c}
    with open(REC / family / "endpoints.jsonl", encoding="utf8") as f:
        for line in f:
            rid = line.split('"run_id": "', 1)[1].split('"', 1)[0]
            if rid not in runs:
                continue
            ep = json.loads(line)
            assert ep["run_id"] == rid, rid
            runs[rid].update(label=ep["original_label"], first_terminal_s=ep["first_terminal_s"],
                             resolution=ep["resolution"])
    return runs


def reproduce_kim_radii(trace, normal):
    """Their model values (Fig. 7, traced) vs the rebuilt Eq. 8, on the 15 hot flames of KT Table 4.2."""
    rows = []
    for t in trace["cool_flame_tests"]:
        run = normal.get(f"PSI-159:acme_normal:{t['test_point']}")
        if run is None:
            rows.append({"test": t["test"], "missing": "not in train/validation"})
            continue
        r_mod = kim_radius("acme_normal", run["injected_mass_flow"], run["x_fuel_injected"], run["zst"], "butane")
        rows.append({"test": t["test"], "run": run["run"], "partition": run["partition"],
                     "m_hc_mg_s": run["injected_mass_flow"] * 1e6, "zst": run["zst"],
                     "r_hf_measured_mm": t["r_hf_mm"], "r_hf_their_model_fig7_mm": t.get("r_hf_model_fig7_mm"),
                     "r_hf_rebuilt_mm": 1e3 * r_mod})
    ok = [r for r in rows if r.get("r_hf_their_model_fig7_mm")]
    rel = [r["r_hf_rebuilt_mm"] / r["r_hf_their_model_fig7_mm"] - 1 for r in ok]
    err = [r["r_hf_rebuilt_mm"] / r["r_hf_measured_mm"] - 1 for r in ok]
    fig_err = [r["r_hf_their_model_fig7_mm"] / r["r_hf_measured_mm"] - 1 for r in ok]
    return {"rows": rows, "n": len(ok),
            "rebuilt_vs_their_model_pct": {"mean": 100 * sum(rel) / len(rel), "max_abs": 100 * max(map(abs, rel))},
            "rebuilt_vs_measured_pct": {"mean": 100 * sum(err) / len(err), "mae": 100 * sum(map(abs, err)) / len(err)},
            "their_model_vs_measured_pct": {"mean": 100 * sum(fig_err) / len(fig_err),
                                            "mae": 100 * sum(map(abs, fig_err)) / len(fig_err)}}


def identify_fig7(trace, normal, tol=0.006):
    """Match the other plotted hot flames to train/validation runs on their model value (it depends on
    m_HC, X_HC and Zst only). K23: n-butane normal, X_O2 0.37-0.40, X_HC 0.15 or 0.3, 0.75-2.0 mg/s, 0.5-2 bar."""
    cand = []
    for r in normal.values():
        if r["fuel"] != "butane" or not r.get("injected_mass_flow") or not r.get("zst"):
            continue
        if not (0.365 <= r["x_o2_ambient"] <= 0.405 and round(r["x_fuel_injected"], 2) in (0.15, 0.3)
                and 0.74e-6 <= r["injected_mass_flow"] <= 2.01e-6):
            continue
        cand.append((1e3 * kim_radius("acme_normal", r["injected_mass_flow"], r["x_fuel_injected"], r["zst"], "butane"), r))
    taken = {t["test_point"] for t in trace["cool_flame_tests"]}
    out = []
    for pt in trace["fig7"]["hot"]:
        if pt.get("assigned_test"):
            continue
        m = [(abs(rm / pt["modeled_mm"] - 1), rm, r) for rm, r in cand if r["run"].split(":")[-1] not in taken]
        m = sorted(x for x in m if x[0] <= tol)
        out.append({"measured_mm": pt["measured_mm"], "modeled_mm": pt["modeled_mm"],
                    "candidates": [{"run": r["run"], "partition": r["partition"], "rebuilt_mm": round(rm, 3),
                                    "p_bar": round(r["pressure"] / 1e5, 2), "label": r.get("label"),
                                    "burn_s": r.get("first_terminal_s")} for _, rm, r in m]})
    return out


def dry_run_occurrence(runs, family):
    rows = []
    for r in runs.values():
        if not r.get("injected_mass_flow") or not r.get("zst") or r.get("label") not in ("SE", "FT"):
            continue
        x_r = r["x_fuel_injected"] if family == "acme_normal" else r["x_o2_injected"]
        row = {"run": r["run"], "partition": r["partition"], "session": r["session"], "fuel": r["fuel"],
               "label": r["label"], "burn_s": r["first_terminal_s"], "p_bar": r["pressure"] / 1e5}
        for name, rule, rd in [("R_published", "published", "irace"), ("R_o2_equivalent", "o2_equivalent", "irace"),
                               ("R_kim_rhoD", "published", "kim")]:
            row[name] = irace_R(family, r["injected_mass_flow"], x_r, r["zst"], r["fuel"], rule, rd)
        row["r_ss_kim_mm"] = 1e3 * kim_radius(family, r["injected_mass_flow"], x_r, r["zst"], r["fuel"])
        rows.append(row)

    def table(sub, key):
        tp = sum(1 for x in sub if x[key] > 1 and x["label"] == "SE")
        fp = sum(1 for x in sub if x[key] > 1 and x["label"] == "FT")
        fn = sum(1 for x in sub if x[key] <= 1 and x["label"] == "SE")
        tn = sum(1 for x in sub if x[key] <= 1 and x["label"] == "FT")
        n = len(sub)
        # AUC of R as a score for SE (rank form), ties counted half
        se = [x[key] for x in sub if x["label"] == "SE"]
        ft = [x[key] for x in sub if x["label"] == "FT"]
        auc = (sum((a > b) + 0.5 * (a == b) for a in se for b in ft) / (len(se) * len(ft))) if se and ft else None
        return {"n": n, "SE": len(se), "FT": len(ft), "pred_SE_and_SE": tp, "pred_SE_but_FT": fp,
                "pred_steady_but_SE": fn, "pred_steady_and_FT": tn,
                "accuracy": (tp + tn) / n if n else None, "brier_hard": (fp + fn) / n if n else None,
                "auc_R": auc, "frac_R_gt_1": (tp + fp) / n if n else None}

    summ = {}
    for part in PARTS + ("both",):
        sub = [x for x in rows if part == "both" or x["partition"] == part]
        summ[part] = {k: table(sub, k) for k in ("R_published", "R_o2_equivalent", "R_kim_rhoD")}
    return rows, summ


def main():
    LOCAL.mkdir(parents=True, exist_ok=True)
    trace = json.loads(TRACE.read_text(encoding="utf8"))
    normal, inverse = load_runs("acme_normal"), load_runs("acme_inverse")
    summary = {"what": __doc__.strip().splitlines()[0], "partitions": list(PARTS),
               "constants": {"rhoD_300_K23_g_per_m_s": 0.0334, "T_hot_K23_K": T_HOT_K23,
                             "rhoD_hot_K23_g_per_m_s": rhod_kim() * 1e3, "rho0D0_I21_g_per_m_s": RHO0D0_I21 * 1e3,
                             "rhoD_I23b_g_per_m_s": rhod_irace() * 1e3, "J_F_g_m2_s": 0.2, "J_O2_g_m2_s": 0.68},
               "radius_reproduction": reproduce_kim_radii(trace, normal),
               "fig7_identification": identify_fig7(trace, normal)}
    for fam, runs in (("acme_normal", normal), ("acme_inverse", inverse)):
        rows, summ = dry_run_occurrence(runs, fam)
        (LOCAL / f"{fam}_occurrence_trainval.json").write_text(json.dumps(rows, indent=1), encoding="utf8")
        summary[fam] = summ
    OUT.write_text(json.dumps(summary, indent=1), encoding="utf8")
    json.dump(summary, sys.stdout, indent=1)


if __name__ == "__main__":
    main()
