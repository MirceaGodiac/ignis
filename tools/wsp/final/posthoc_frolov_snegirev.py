"""Two more published comparators for Flame Design (PSI-10): Frolov et al. (Fire 2023) and Snegirev (CNF 2023).

Post-hoc, 15 November 2026, after the final run of PROTOCOL 12.11. Nothing is adopted or refused, no IGNIS weight is
read, fitted or tuned, no holdout/ file or evaluator vault is opened, nothing is revealed: the test-day endpoints
and sealed forecasts are the final run's own local caches (as in posthoc_irace_testdays.py), the train/validation
forecasts are the development caches already on disk. Same recipe as RESULTS_PUBLISHED_ACME_BASS.md section 1.

1. Frolov, Ivanov, Frolov, Semenov, Fire 6 (2023) 285. Their transient 1-D detailed-chemistry model puts some of
   their 37 flames out; per flame the extinction time window is read from their Fig. 6 (10 s brackets) and, for two
   flames, the time from their full histories (Figs. 4, 5). Transcription with provenance:
   docs/wsp/research/frolov2023_flames.json. Comparator = step forecast "extinction at their modelled time"
   (window midpoint, or the exact time where they plot one; window ends as sensitivity), the planned stop first on
   the stop path. It exists only for flames their model puts out within 50 s ("windowed"); a second subset adds the
   flames their model never puts out (step forecast with no extinction). Flames with a model extinction after 50 s
   (time unpublished) are left out.
2. Snegirev, Combust. Flame 248 (2023) 112499. Closed-form pre-extinction radius Eq. 20 (with Eqs. 3, 14/22, 17/21,
   35-38), steady radius Eq. 10, steady-burning test R_ss > R_cr -> self-extinguishes, published constants
   (T_cr 1100 K ethylene, c_p 1700 J/kg/K, k 0.07 W/m/K, T_inf 300 K, dh_c 47.17 MJ/kg, sigma_O2 3.42, HITEMP2010
   Planck means at T_cr). Assumptions (ours, flagged): R_s = 3.2 mm (sphere radius; the paper sets R_s = R_b/2 only
   for BRE), kappa_f per test from Eq. 35 (variant: his BRE average 4.1 1/m); inverse flames by swapping the burner
   and ambient reactants (the paper treats burner fuel only). Scored like "Irace R only" (goes out at the first
   horizon iff R_ss > R_cr) and paired with the erfc growth clock of posthoc_irace_comparator.py (OURS, not his).

Scoring: transfer_acme `score` (IPCW integrated Brier; Flame Design endpoints are exact events), contributions per
case computed over the whole block, so a comparator scored on a subset of cases uses the same weights as F2/F1/
surrogate on exactly those cases. Paired differences: transfer_acme `boot` over days (sessions).

    python tools/wsp/final/posthoc_frolov_snegirev.py
"""
import csv
import json
import math
import pathlib
import statistics as st
import sys

HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import run_final as RF  # noqa: E402  (puts tools/wsp/ext on the path)
import posthoc_irace_comparator as IC  # noqa: E402

ROOT = HERE.parents[2]
LOCAL = ROOT / "runs/final/20261115/local/final/flame-design"
REF_TEST = ROOT / "runs/final/20261115/flame-design.json"
WORK = ROOT / "runs/dev/data_all/work/transfer_acme"
F2_VAL = WORK / "fd_f2"
TRAIN_CASES = WORK / "fd_fuel/train_days_cases.jsonl"
LIT = ROOT / "docs/wsp/research/frolov2023_flames.json"
SPLIT = ROOT / "docs/wsp/atlas/splits/PSI-10.json"
OUT = ROOT / "runs/final/20261115/posthoc_frolov_snegirev.json"
ATM = 101325.0
METRICS = ("extinction", "operator_termination", "sum")

tf2, ta, g6, tf, fdm = RF.fd_modules()
import acme_script2  # noqa: E402


# ============================================================================================ Snegirev (2023)
SN = dict(T_cr=1100.0, T_inf=300.0, cp=1700.0, k=0.07, dh_c=47.17e6, sigma=3.42, kappa_avg=4.1, R_s=3.2e-3,
          SB=5.67e-8, kP_CO2_1100=19.7, kP_H2O_1100=4.18)


def kappa_eq35(x_fuel_stream, x_o2_stream, p_atm, n_c=2, n_h=4):
    """Eqs. 35-38: Planck-mean absorption of the stoichiometric products, HITEMP2010 at T_cr = 1100 K (Table 2)."""
    nu_co2, nu_h2o = x_fuel_stream * n_c, x_fuel_stream * n_h / 2
    nu_o2 = x_fuel_stream * (n_c + n_h / 4)
    nu_n2 = (1 - x_fuel_stream) + nu_o2 * (1 / x_o2_stream - 1)
    tot = nu_co2 + nu_h2o + nu_n2
    return p_atm * (nu_co2 / tot * SN["kP_CO2_1100"] + nu_h2o / tot * SN["kP_H2O_1100"])


def snegirev(kind, m_burner, x_burner, x_amb, p_pa, kappa="eq35", cp="const", tad=None, R_s=None):
    """kind normal: burner = fuel stream (X_C2H4), ambient = O2 (X_O2), as published. kind inverse (ours): burner =
    O2 stream, ambient = fuel; the same energy balance with the roles of the two streams swapped.
    m_burner: total burner mass flow, kg/s. Returns radii in mm."""
    R_s = SN["R_s"] if R_s is None else R_s
    if kind == "normal":
        y_f, y_o = IC.y_of(x_burner, IC.W_C2H4), IC.y_of(x_amb, IC.W_O2)
        dh = y_f * SN["dh_c"]                                   # Eq. 3, J per kg of burner stream
        amb_per_burner = SN["sigma"] * y_f / y_o                # sigma_O2,dil / Y_O2,inf (Eq. 5)
        xf_stream, xo_stream = x_burner, x_amb
    else:
        y_o, y_f = IC.y_of(x_burner, IC.W_O2), IC.y_of(x_amb, IC.W_C2H4)
        dh = y_o * SN["dh_c"] / SN["sigma"]
        amb_per_burner = (y_o / SN["sigma"]) / y_f
        xf_stream, xo_stream = x_amb, x_burner
    c_p = SN["cp"] if cp == "const" else dh / ((tad - SN["T_inf"]) * (1 + amb_per_burner))     # Eq. 34 (App. A1)
    kap = SN["kappa_avg"] if kappa == "avg" else kappa_eq35(xf_stream, xo_stream, p_pa / ATM)
    r_ss = m_burner / (4 * math.pi * SN["k"] / c_p * math.log(1 + 1 / amb_per_burner))     # Eq. 10
    f0 = 1 - c_p * (SN["T_cr"] - SN["T_inf"]) * (1 + amb_per_burner) / dh                    # Eq. 22
    out = dict(R_ss_mm=1e3 * r_ss, f_r_cr0=f0, kappa_f=kap, c_p=c_p, q_ch_W=m_burner * dh)
    if f0 <= 0:
        return dict(out, R_cr0_mm=0.0, R_cr_mm=0.0, R=math.inf, disc=None, t_ext_erfc_s=0.0)
    r0 = (3 / (4 * math.pi) * f0 * m_burner * dh / (4 * kap * SN["SB"] * (SN["T_cr"] ** 4 - SN["T_inf"] ** 4))) ** (1 / 3)  # Eq. 21
    disc = 1 - 4 * R_s ** 3 / (f0 * r0 ** 3)
    r_cr = r0 * ((1 + math.sqrt(disc)) / 2) ** (1 / 3) if disc >= 0 else 0.0                # Eq. 20
    R = r_ss / r_cr if r_cr > 0 else math.inf
    D = IC.D0 * ATM / p_pa
    t = (r_cr ** 2 / (4 * D * IC.erfcinv(r_cr / r_ss) ** 2) if r_cr > 0 else 0.0) if R > 1 else math.inf  # OUR clock
    return dict(out, R_cr0_mm=1e3 * r0, R_cr_mm=1e3 * r_cr, R=R, disc=disc, t_ext_erfc_s=t)


def eq31_qcr(x_o2, kappa=None, cp=None):
    """Eq. 31, exact (left) and approximate (Y_O2/sigma << 1, right) critical nominal heat release, W, normal flames."""
    kap = SN["kappa_avg"] if kappa is None else kappa
    c_p = SN["cp"] if cp is None else cp
    y_o = IC.y_of(x_o2, IC.W_O2)
    dT4 = SN["T_cr"] ** 4 - SN["T_inf"] ** 4
    out = {}
    # exact: R_ss(m) = R_cr0(m) for pure fuel (Y_fuel = 1; the dilution cancels only in the approximate form)
    amb = SN["sigma"] / y_o
    f0 = 1 - c_p * (SN["T_cr"] - SN["T_inf"]) * (1 + amb) / SN["dh_c"]
    a = 4 * math.pi * SN["k"] / c_p * math.log(1 + 1 / amb)          # m = a R_ss
    b = (3 / (4 * math.pi) * f0 * SN["dh_c"] / (4 * kap * SN["SB"] * dT4))  # R_cr0^3 = b m
    m = (a ** 3 * b) ** 0.5
    out["exact_W"] = m * SN["dh_c"]
    out["approx_W"] = (4 * math.pi * SN["k"] / c_p * (3 / (4 * math.pi) * f0 / (4 * kap * SN["SB"] * dT4)) ** (1 / 3)
                       * SN["dh_c"] / SN["sigma"] * y_o) ** 1.5
    return out


def snegirev_literature():
    """Every number the paper publishes that can be recomputed without its unpublished per-flame data."""
    out = {}
    # Table 1 (App. A1): effective c_p by Eq. 34 for X_fuel = 1, X_O2 = 0.21 from the tabulated T_ad
    rows = []
    for fuel, dh, sig, tad, cp_pub in (("propane", 46.35e6, 3.63, 2267, 1422), ("ethylene", 47.17e6, 3.42, 2370, 1528)):
        y_o = IC.y_of(0.21, IC.W_O2)
        cp = dh / ((tad - 300) * (1 + sig / y_o))
        tad_for_pub = 300 + dh / (cp_pub * (1 + sig / y_o))
        rows.append(dict(fuel=fuel, T_ad_table_K=tad, cp_eq34=round(cp, 1), cp_published=cp_pub,
                         T_ad_that_gives_published_cp_K=round(tad_for_pub, 1)))
    out["table1_cp_eq34"] = rows
    # Eq. 25: C = f_r,cr,0^(1/3) with f = 0.48 (propane) -> "C ~ 0.78"
    out["eq25_C_from_fr_0p48"] = dict(ours=round(0.48 ** (1 / 3), 3), published=0.78)
    # Eq. 35 range over 1 atm air and 0.38 O2, pure ethylene, against his BRE average 4.1 1/m
    out["kappa_eq35_1atm"] = {f"x_o2={x}": round(kappa_eq35(1.0, x, 1.0), 3) for x in (0.21, 0.30, 0.38)}
    out["kappa_eq35_1atm"]["published_BRE_average"] = 4.1
    # Eq. 31 boundary (Fig. 7, BRE 25 mm, constants as printed): no per-point numbers are published
    out["eq31_q_ch_cr_W"] = {f"x_o2={x}": {k: round(v, 2) for k, v in eq31_qcr(x).items()} for x in (0.21, 0.25, 0.30, 0.35, 0.40)}
    out["eq31_note"] = ("Fig. 7 gives the boundary only as a plotted line (q_ch from about 10 to 200 W); exact and approximate "
                        "forms agree within a few percent, and the exact form equals R_ss(Eq. 10) = R_cr,0(Eq. 21) by construction")
    return out


# ============================================================================================ data access
def splits():
    sp = json.loads(SPLIT.read_text(encoding="utf-8"))
    part = {}
    for name, stratum in sp["strata"].items():
        fam = "acme_normal" if name.startswith("normal") else "acme_inverse"
        for p, gs in stratum["groups"].items():
            for g in gs:
                part[(fam, g.split(":")[-1])] = p
    return part


def conditions():
    with (LOCAL / "flame_design/conditions.csv").open(encoding="utf-8", newline="") as f:
        return {r["test"]: r for r in csv.DictReader(f)}


def fnum(s):
    try:
        return float(s)
    except (TypeError, ValueError):
        return math.nan


def frolov_table():
    lit = json.loads(LIT.read_text(encoding="utf-8"))
    flames = {}
    for kind, key in (("normal", "table1_normal"), ("inverse", "table2_inverse")):
        cols = lit[key]["columns"]
        for row in lit[key]["rows"]:
            r = dict(zip(cols, row))
            r["kind"] = kind
            flames[r["flame"]] = r
    mt = lit["model_extinction_time"]
    for row in mt["rows"]:
        r = dict(zip(mt["columns"], row))
        flames[r["flame"]].update(status="window", lo=r["model_out_after_s"], hi=r["model_out_by_s"], exact=r["exact_s"])
    for fl in mt["after_50_s"]["flames"].values():
        flames[fl].update(status="after_50s", lo=50.0, hi=math.inf, exact=None)
    for fl in mt["never"]["flames"].values():
        flames[fl].update(status="never", lo=math.inf, hi=math.inf, exact=None)
    for fl, r in flames.items():
        if "status" not in r:
            assert fl.startswith("F"), fl
            r.update(status="not_iss", lo=math.inf, hi=math.inf, exact=None)
        if r["status"] == "window":
            r["mid"] = r["exact"] if r["exact"] is not None else (r["lo"] + r["hi"]) / 2
        else:
            r["mid"] = r["lo"]
    return lit, flames


def frolov_t(fl, variant):
    if fl["status"] == "window":
        return {"mid": fl["mid"], "lo": fl["lo"], "hi": fl["hi"]}[variant]
    if fl["status"] == "never":
        return math.inf
    return None


# ============================================================================================ checks on the transcription
def check_conditions(lit, flames, cond, part):
    rows = []
    math_rows = lit["mathematics_2023_companion_table"]["rows_that_differ_from_fire_2023"]
    for name, f in flames.items():
        if name.startswith("F"):
            continue
        c = cond.get(name)
        fam = "acme_normal" if f["kind"] == "normal" else "acme_inverse"
        r = dict(flame=name, no=f["no"], kind=f["kind"], partition=part.get((fam, name[:5])), model=f["status"])
        if f["kind"] == "normal":
            pub = dict(x_o2=f["x_o2_chamber"], x_fuel=f["x_c2h4_burner"], G=f["G_in_mg_s"], p=f["p_atm"], z=f["Z_st"])
        else:
            pub = dict(x_o2=f["x_o2_burner"], x_fuel=f["x_c2h4_chamber"], G=f["G_in_mg_s"], p=f["p_atm"], z=f["Z_st"])
        if c is None or not math.isfinite(fnum(c["injected_mass_flow_kg_s"])):
            r.update(ours=None, note=f"no numeric condition row ({c['table_note'] if c else 'absent'}); X_O2 init {c['x_o2_role'] if c else '-'}")
        else:
            m = fnum(c["injected_mass_flow_kg_s"])
            xo, xf = fnum(c["x_o2_role"]), fnum(c["x_fuel_role"])
            g = m / (IC.y_of(xf, IC.W_C2H4) if f["kind"] == "normal" else IC.y_of(xo, IC.W_O2)) * 1e6
            ours = dict(x_o2=xo, x_fuel=xf, G=g, p=fnum(c["pressure_pa"]) / ATM, z=fnum(c["zst_initial"]))
            r["ours"] = {k: round(v, 4) for k, v in ours.items()}
            r["fire_minus_ours"] = dict(x_o2=round(pub["x_o2"] - xo, 4), x_fuel=round(pub["x_fuel"] - xf, 4),
                                        G_pct=round(100 * (pub["G"] / g - 1), 2), p_atm=round(pub["p"] - ours["p"], 3),
                                        z=round(pub["z"] - ours["z"], 4))
            if name in math_rows:
                mr = math_rows[name]
                r["mathematics_minus_ours"] = dict(x_o2=round(mr["x_o2"] - xo, 4), x_fuel=round(mr["x_c2h4"] - xf, 4),
                                                   G_pct=round(100 * (mr["G_in_mg_s"] / g - 1), 2))
            r["in_transfer"] = c["include_transfer"] == "1"
        r["fire"] = pub
        rows.append(r)
    ok = [r for r in rows if r.get("ours")]
    summary = dict(matched_with_numeric_rows=len(ok),
                   max_abs_G_pct=max(abs(r["fire_minus_ours"]["G_pct"]) for r in ok),
                   max_abs_x_o2=max(abs(r["fire_minus_ours"]["x_o2"]) for r in ok),
                   max_abs_x_fuel=max(abs(r["fire_minus_ours"]["x_fuel"]) for r in ok),
                   max_abs_p_atm=max(abs(r["fire_minus_ours"]["p_atm"]) for r in ok),
                   mathematics_rows={r["flame"]: r["mathematics_minus_ours"] for r in ok if "mathematics_minus_ours" in r})
    return dict(summary=summary, rows=rows)


def check_figures(flames, ends):
    """Fig. 7: R_fe,calc / G_in of the 13 model-extinguished flames against the printed fits, and the window sets."""
    fits = {"normal": lambda z: 1.51 + 22.3 * math.exp(-9.35 * z), "inverse": lambda z: 2.11 + 0.00015 * math.exp(12.99 * z)}
    rows = []
    for name, f in flames.items():
        if f["R_fe_calc_mm"] is None:
            continue
        ratio = f["R_fe_calc_mm"] / f["G_in_mg_s"]
        rows.append(dict(flame=name, model=f["status"], R_over_G=round(ratio, 3), fit=round(fits[f["kind"]](f["Z_st"]), 3),
                         rel=round(ratio / fits[f["kind"]](f["Z_st"]) - 1, 3)))
    win = [r for r in rows if r["model"] == "window"]
    calc_set = {n for n, f in flames.items() if f["R_fe_calc_mm"] is not None}
    win_set = {n for n, f in flames.items() if f["status"] == "window"}
    hist = {}
    for name in ("19189K1", "22018G3"):
        f = flames[name]
        e = ends.get(name)
        hist[name] = dict(model_extinction_s=f["exact"], authors_last_measured_point_s={"19189K1": 40.5, "22018G3": 17.7}[name],
                          published_reading={"19189K1": "model ~41 s vs last measured point ~40.5 s", "22018G3": "model ~28.5 s vs last measured point ~17.7 s"}[name],
                          ignis_endpoint=e)
    return dict(fig7_points_vs_printed_fit=dict(windowed=win, after_50s=[r for r in rows if r["model"] != "window"],
                                                windowed_max_abs_rel=max(abs(r["rel"]) for r in win),
                                                windowed_mean_abs_rel=round(st.mean(abs(r["rel"]) for r in win), 3)),
                windowed_flames_all_have_calc_radius=win_set <= calc_set,
                calc_radius_but_not_by_50s=sorted(calc_set - win_set), full_histories=hist)


def snegirev_vs_frolov_exp(flames, cond):
    """Eq. 20 against the measured extinction radii listed by Frolov et al. (normal ISS flames), our conditions."""
    rows = []
    for name, f in flames.items():
        if f["kind"] != "normal" or f["R_fe_exp_mm"] is None or name.startswith("F"):
            continue
        c = cond.get(name)
        if c is None or not math.isfinite(fnum(c["injected_mass_flow_kg_s"])):
            continue
        xo, xf = fnum(c["x_o2_role"]), fnum(c["x_fuel_role"])
        m = fnum(c["injected_mass_flow_kg_s"]) / IC.y_of(xf, IC.W_C2H4)
        s = snegirev("normal", m, xf, xo, fnum(c["pressure_pa"]))
        s34 = snegirev("normal", m, xf, xo, fnum(c["pressure_pa"]), cp="eq34", tad=fnum(c["tad_k_table"]))
        rows.append(dict(flame=name, R_fe_exp_mm=f["R_fe_exp_mm"], R_fe_calc_frolov_mm=f["R_fe_calc_mm"],
                         snegirev_R_cr_mm=round(s["R_cr_mm"], 2), snegirev_R_cr_cp_eq34_mm=round(s34["R_cr_mm"], 2),
                         f_r_cr0=round(s["f_r_cr0"], 3), f_r_cr0_cp_eq34=round(s34["f_r_cr0"], 3),
                         R_ss_over_R_cr=round(s["R"], 2), q_ch_W=round(s["q_ch_W"], 1)))
    err = lambda k: dict(mean_pct=round(st.mean(100 * (r[k] / r["R_fe_exp_mm"] - 1) for r in rows), 1),  # noqa: E731
                         mean_abs_pct=round(st.mean(abs(100 * (r[k] / r["R_fe_exp_mm"] - 1)) for r in rows), 1))
    return dict(rows=rows, n=len(rows), snegirev_vs_measured=err("snegirev_R_cr_mm"), snegirev_cp_eq34_vs_measured=err("snegirev_R_cr_cp_eq34_mm"),
                frolov_1d_vs_measured=err("R_fe_calc_frolov_mm"),
                f_r_cr0_range=[min(r["f_r_cr0"] for r in rows), max(r["f_r_cr0"] for r in rows)],
                f_r_cr0_range_cp_eq34=[min(r["f_r_cr0_cp_eq34"] for r in rows), max(r["f_r_cr0_cp_eq34"] for r in rows)],
                published_f_r="0.59-0.61 for the 19 self-extinguished Flame Design flames (text, Fig. 2b); which flames is not published")


# ============================================================================================ cases
def kc_value(kc, k):
    v = kc[k]
    return v.get("value_si") if v.get("value_si") is not None else v.get("raw_value")


def cond_from_kc(kc):
    return dict(m_r=kc_value(kc, "injected_mass_flow"), x_o2=kc_value(kc, "x_o2_role"), x_f=kc_value(kc, "x_fuel_role"),
                p=kc_value(kc, "pressure"))


def test_day_blocks():
    """The final run's test-day blocks (exactly posthoc_irace_testdays.py)."""
    tests = RF.fd_test_day_tests(ta, fdm, LOCAL / "flame_design/conditions.csv", LOCAL / "flame_design/endpoints.csv")
    blocks = {}
    for fam in ("acme_normal", "acme_inverse"):
        for stop in (False, True):
            tag = "stop" if stop else "nostop"
            cases = []
            for t in (t for t in tests if t["family"] == fam):
                for c in ta.CUTOFFS:
                    if not ta.at_risk(t["endpoint"], c):
                        continue
                    name = lambda role: f"{t['test']}_c{c:g}_{role}_{tag}.json"  # noqa: E731
                    art = {r: json.loads((LOCAL / "forward/artifact" / fam / r / name(r)).read_text(encoding="utf-8")) for r in ("f2", "f1", "surrogate")}
                    if any(not ta.admitted(a) for a in art.values()):
                        continue
                    pre = json.loads((LOCAL / "forward/prefix" / fam / name("f2")).read_text(encoding="utf-8"))
                    st_ = None
                    if stop and "programmed_stop" in pre["known_conditions"]:
                        v = kc_value(pre["known_conditions"], "programmed_stop")
                        st_ = v if v and v > 0 else None
                    cases.append(dict(test=t["test"], session=t["session"], cutoff=c, outcome=ta.outcome_of(t["endpoint"]),
                                      cif={r: ta.cif_of(a) for r, a in art.items()}, cond=cond_from_kc(pre["known_conditions"]),
                                      stop=st_, art_c1={r: a for r, a in art.items()} if c == 1.0 else None,
                                      end=dict(cause=(t["endpoint"]["allowed_causes"] or ["right"])[0], t=t["endpoint"]["last_confirmed_active_s"])))
            blocks[f"{fam}/{tag}"] = cases
    return blocks


def validation_blocks():
    """fd_f2 validation-day forwards (PROTOCOL 12.10 b), read from disk; endpoints from the fd_fuel layout."""
    tests, _, _, _ = ta.fd_tests()
    tests = [t for t in tests if t["part"] == "validation_days"]
    blocks = {}
    for fam in ("acme_normal", "acme_inverse"):
        for stop in (False, True):
            tag = "stop" if stop else "nostop"
            cases = []
            for t in (t for t in tests if t["family"] == fam):
                for c in ta.CUTOFFS:
                    if not ta.at_risk(t["endpoint"], c):
                        continue
                    name = lambda role: f"{t['test']}_c{c:g}_{role}_{tag}.json"  # noqa: E731
                    art = {r: json.loads((F2_VAL / "artifact" / fam / r / name(r)).read_text(encoding="utf-8")) for r in ("f2", "f1", "surrogate")}
                    if any(not ta.admitted(a) for a in art.values()):
                        continue
                    pre = json.loads((F2_VAL / "prefix" / fam / name("f2")).read_text(encoding="utf-8"))
                    st_ = None
                    if stop and "programmed_stop" in pre["known_conditions"]:
                        v = kc_value(pre["known_conditions"], "programmed_stop")
                        st_ = v if v and v > 0 else None
                    cases.append(dict(test=t["test"], session=t["session"], cutoff=c, outcome=ta.outcome_of(t["endpoint"]),
                                      cif={r: ta.cif_of(a) for r, a in art.items()}, cond=cond_from_kc(pre["known_conditions"]),
                                      stop=st_, art_c1=art if c == 1.0 else None, fuel=t["fuel_species"],
                                      end=dict(cause=(t["endpoint"]["allowed_causes"] or ["right"])[0], t=t["endpoint"]["last_confirmed_active_s"])))
            blocks[f"{fam}/{tag}"] = cases
    return blocks


def train_blocks():
    """Train days: F1 and the surrogate as stored by fd_fuel (no F2 forecast of a train day is on disk)."""
    rows = [json.loads(l) for l in TRAIN_CASES.read_text(encoding="utf-8").splitlines() if l.strip()]
    assert all(r["days"] == "train" for r in rows)
    blocks = {}
    for fam in ("acme_normal", "acme_inverse"):
        for tag in ("nostop", "stop"):
            sel = [r for r in rows if r["family"] == fam and r["path"] == tag and not r["f1_refused"]
                   and not r["surrogate_refused"] and "ibs_contrib" in r]
            cases = []
            for r in sel:
                f1 = r["f1_forecast"]
                # the surrogate as scored by fd_fuel (its sealed artifact; the *_dense copy in the jsonl is another grid)
                sp = WORK / "fd_fuel/artifact" / fam / "surrogate" / f"{r['test']}_c{r['cutoff_s']:g}_ethane_{tag}.json"
                sa = json.loads(sp.read_text(encoding="utf-8"))
                cif = {"f1": {c: ta.interp_row(f1["times_s"], f1["event_cdf"][f1["event_categories"].index(c)], ta.HORIZONS) for c in ta.CAUSES},
                       "surrogate": ta.cif_of(sa)}
                st_ = None
                if tag == "stop":
                    v = acme_script2.for_test("PSI-10", r["test"]).get("planned_stop_s")
                    st_ = v if v is not None and v > 0 else None
                t = r["endpoint"]["time_s"]
                c = r["conditions"]
                cases.append(dict(test=r["test"], session=r["session"], cutoff=r["cutoff_s"],
                                  outcome=dict(censoring="event", t_lower=t, t_upper=t, causes=[r["endpoint"]["cause"]]),
                                  cif=cif, cond=dict(m_r=c["injected_mass_flow"], x_o2=c["x_o2_role"], x_f=c["x_fuel_role"], p=c["pressure"]),
                                  stop=st_, stored=r["ibs_contrib"], f1_c1=f1 if r["cutoff_s"] == 1.0 else None,
                                  end=dict(cause=r["endpoint"]["cause"], t=t)))
            blocks[f"{fam}/{tag}"] = cases
    return blocks


# ============================================================================================ comparators
def m_tot(fam, cnd):
    return cnd["m_r"] / (IC.y_of(cnd["x_f"], IC.W_C2H4) if fam == "acme_normal" else IC.y_of(cnd["x_o2"], IC.W_O2))


def sneg_case(fam, cnd, **kw):
    kind = "normal" if fam == "acme_normal" else "inverse"
    xb, xa = (cnd["x_f"], cnd["x_o2"]) if kind == "normal" else (cnd["x_o2"], cnd["x_f"])
    return snegirev(kind, m_tot(fam, cnd), xb, xa, cnd["p"], **kw)


def irace_case(fam, cnd):
    kind = "normal" if fam == "acme_normal" else "inverse"
    z = IC.zst(cnd["x_f"], cnd["x_o2"])
    return IC.irace(kind, cnd["m_r"], m_tot(fam, cnd), z, cnd["p"])


def comparator_cifs(fam, k, flames):
    """Step forecasts per comparator for one case; None where the comparator does not exist for the case."""
    h0 = ta.HORIZONS[0]
    out = {}
    fl = flames.get(k["test"])
    for v in ("mid", "lo", "hi"):
        t = frolov_t(fl, v) if fl else None
        out[f"frolov_{v}"] = IC.step_cif(t, k["stop"], ta.HORIZONS) if t is not None else None
    s = sneg_case(fam, k["cond"])
    sa = sneg_case(fam, k["cond"], kappa="avg")
    s0 = sneg_case(fam, k["cond"], R_s=0.0)
    out["snegirev_R_only"] = IC.step_cif(h0 if s["R"] > 1 else math.inf, k["stop"], ta.HORIZONS)
    out["snegirev_R_only_kappa4.1"] = IC.step_cif(h0 if sa["R"] > 1 else math.inf, k["stop"], ta.HORIZONS)
    out["snegirev_R_only_Rs0"] = IC.step_cif(h0 if s0["R"] > 1 else math.inf, k["stop"], ta.HORIZONS)
    out["snegirev_plus_erfc_clock"] = IC.step_cif(s["t_ext_erfc_s"], k["stop"], ta.HORIZONS)
    q = irace_case(fam, k["cond"])
    out["irace_plus_erfc_clock"] = IC.step_cif(q["t_ext_erfc_s"], k["stop"], ta.HORIZONS)
    out["irace_R_only"] = IC.step_cif(h0 if q["R"] > 1 else math.inf, k["stop"], ta.HORIZONS)
    k["sneg"], k["sneg_k41"], k["irace"] = s, sa, q
    return out


def contributions(cases, roles, fill_role):
    """Per-case contributions over the whole block; a missing comparator is filled with `fill_role` (its value
    is never used: only the comparator's own cases are averaged)."""
    outcomes = [k["outcome"] for k in cases]
    res = {}
    for r in roles:
        cif = {cz: [(k["comp"].get(r) or k["cif"][fill_role])[cz] if r not in k["cif"] else k["cif"][r][cz] for k in cases] for cz in ta.CAUSES}
        s = ta.score(outcomes, cif)
        res[r] = {cz: s[cz]["contrib"] for cz in ta.CAUSES}
        res[r]["sum"] = [a + b for a, b in zip(res[r]["extinction"], res[r]["operator_termination"])]
    return res


def summarise(cases, contrib, idx, models, comps):
    ss = [cases[i]["session"] for i in idx]
    out = dict(cases=len(idx), tests=len({cases[i]["test"] for i in idx}), days=sorted(set(ss)),
               endpoints={e: sum(cases[i]["end"]["cause"] == e for i in idx) for e in ("extinction", "operator_termination", "right")},
               score={}, paired={})
    if not idx:
        return out
    for r in models + comps:
        out["score"][r] = {m: math.fsum(contrib[r][m][i] for i in idx) / len(idx) for m in METRICS}
    for o in comps:
        for base in models[:1]:
            out["paired"][f"{base}_minus_{o}"] = {m: ta.boot([contrib[base][m][i] - contrib[o][m][i] for i in idx], ss) for m in METRICS}
    return out


def median_ext(times, cdf):
    """Time where the extinction incidence reaches 0.5 (linear between forecast times); None if it never does."""
    prev_t, prev_v = 0.0, 0.0
    for t, v in zip(times, cdf):
        if v >= 0.5:
            return prev_t + (0.5 - prev_v) * (t - prev_t) / (v - prev_v) if v > prev_v else t
        prev_t, prev_v = t, v
    return None


# ============================================================================================ main
def main():
    lit, flames = frolov_table()
    part = splits()
    cond = conditions()
    res = dict(what=__doc__.strip().splitlines()[0], sources=dict(frolov=str(LIT.relative_to(ROOT)), snegirev="doi:10.1016/j.combustflame.2022.112499"),
               rules="post hoc; no IGNIS weight read or touched; no holdout/, no evaluator vault, no --final-test, no reveal")

    # ---- Frolov: matching, conditions, figure consistency
    res["frolov_conditions"] = check_conditions(lit, flames, cond, part)
    tb, vb, trb = test_day_blocks(), validation_blocks(), train_blocks()
    ends = {}
    for blocks in (tb, vb, trb):
        for cases in blocks.values():
            for k in cases:
                ends[k["test"]] = k["end"]
    res["frolov_figures"] = check_figures(flames, ends)
    res["snegirev_literature"] = snegirev_literature()
    res["snegirev_vs_frolov_measured_radius"] = snegirev_vs_frolov_exp(flames, cond)

    # ---- scoring
    res["blocks"] = {}
    worst = {}
    ref_test = json.loads(REF_TEST.read_text(encoding="utf-8"))
    ref_val = json.loads((F2_VAL / "report.json").read_text(encoding="utf-8"))
    comps_f = ["frolov_mid", "frolov_lo", "frolov_hi", "irace_plus_erfc_clock"]
    comps_s = ["snegirev_R_only", "snegirev_R_only_kappa4.1", "snegirev_R_only_Rs0", "snegirev_plus_erfc_clock",
               "irace_R_only", "irace_plus_erfc_clock"]
    for part_name, blocks, models in (("test_days", tb, ["f2", "f1", "surrogate"]), ("validation_days", vb, ["f2", "f1", "surrogate"]),
                                      ("train_days", trb, ["f1", "surrogate"])):
        for key, cases in blocks.items():
            fam = key.split("/")[0]
            if not cases:
                continue
            for k in cases:
                k["comp"] = comparator_cifs(fam, k, flames)
            roles = models + sorted({r for k in cases for r in k["comp"]})
            contrib = contributions(cases, roles, models[0])
            # reproduction of the stored scores
            chk = {}
            if part_name == "test_days":
                rb = ref_test["blocks"][key]["all_answer"]
                chk = {f"{r}/{m}": abs(math.fsum(contrib[r][m]) / len(cases) - rb["score"][r][m]["case_mean"]) for r in models for m in METRICS} \
                    if rb["cases"] == len(cases) else {"cases_differ": [rb["cases"], len(cases)]}
            elif part_name == "validation_days":
                rb = ref_val["blocks"][key]["all_answer"]
                chk = {f"{r}/{m}": abs(math.fsum(contrib[r][m]) / len(cases) - rb["score"][r][m]["case_mean"]) for r in models for m in METRICS} \
                    if rb["cases"] == len(cases) else {"cases_differ": [rb["cases"], len(cases)]}
            else:
                chk = {f"{r}/{m}": max(abs(contrib[r][m][i] - cases[i]["stored"][r][m]) for i in range(len(cases))) for r in models for m in METRICS}
            worst[f"{part_name}:{key}"] = max(v for v in chk.values() if isinstance(v, float)) if all(isinstance(v, float) for v in chk.values()) else chk
            win = [i for i, k in enumerate(cases) if flames.get(k["test"], {}).get("status") == "window"]
            wn = [i for i, k in enumerate(cases) if flames.get(k["test"], {}).get("status") in ("window", "never")]
            b = dict(cases_in_block=len(cases), sessions_in_block=len({k["session"] for k in cases}), reproduce_abs_diff_max=worst[f"{part_name}:{key}"])
            b["frolov_windowed"] = summarise(cases, contrib, win, models, comps_f)
            b["frolov_windowed_plus_never"] = summarise(cases, contrib, wn, models, ["frolov_mid", "irace_plus_erfc_clock"])
            if part_name == "test_days":
                allidx = list(range(len(cases)))
                b["snegirev_all_cases"] = summarise(cases, contrib, allidx, models, comps_s)
            res["blocks"][f"{part_name}:{key}"] = b

    # ---- per flame: Frolov time vs observed, F2 (or F1 on train) median, Irace/Snegirev clocks
    per = []
    for part_name, blocks, role in (("test_days", tb, "f2"), ("validation_days", vb, "f2"), ("train_days", trb, "f1")):
        for key, cases in blocks.items():
            if not key.endswith("nostop"):
                continue
            fam = key.split("/")[0]
            for k in cases:
                if k["cutoff"] != 1.0 or k["test"] not in flames:
                    continue
                fl = flames[k["test"]]
                if part_name == "train_days":
                    f = k["f1_c1"]
                    med = median_ext(f["times_s"], f["event_cdf"][f["event_categories"].index("extinction")])
                else:
                    a = k["art_c1"][role]
                    med = median_ext(a["times_s"], a["event_cdf"][a["event_categories"].index("extinction")])
                per.append(dict(flame=k["test"], no=fl["no"], partition=part_name, family=fam, day=k["session"],
                                observed=k["end"], frolov_model=fl["status"], frolov_t_mid=fl.get("mid"), frolov_window=[fl.get("lo"), fl.get("hi")],
                                ignis_role=role, ignis_median_ext_s=med,
                                irace_R=round(k["irace"]["R"], 3), irace_erfc_s=k["irace"]["t_ext_erfc_s"],
                                snegirev_R=round(k["sneg"]["R"], 3), snegirev_R_kappa4p1=round(k["sneg_k41"]["R"], 3),
                                snegirev_R_cr_mm=round(k["sneg"]["R_cr_mm"], 2), snegirev_R_ss_mm=round(k["sneg"]["R_ss_mm"], 2),
                                snegirev_erfc_s=k["sneg"]["t_ext_erfc_s"]))
    res["per_flame"] = per

    def err(rows, key):
        e = [abs(r[key] - r["observed"]["t"]) for r in rows if r[key] is not None and math.isfinite(r[key])]
        ln = [math.log(r[key] / r["observed"]["t"]) for r in rows if r[key] is not None and math.isfinite(r[key]) and r[key] > 0]
        return dict(n=len(e), median_abs_s=round(st.median(e), 2) if e else None, mean_abs_s=round(st.mean(e), 2) if e else None,
                    median_ln_pred_over_obs=round(st.median(ln), 3) if ln else None)
    et = {}
    for part_name in ("test_days", "validation_days", "train_days"):
        rows = [r for r in per if r["partition"] == part_name and r["observed"]["cause"] == "extinction" and r["frolov_model"] == "window"]
        et[part_name] = dict(flames=[r["flame"] for r in rows], frolov_mid=err(rows, "frolov_t_mid"),
                             ignis=err([dict(r, ignis_median_ext_s=r["ignis_median_ext_s"]) for r in rows], "ignis_median_ext_s"),
                             ignis_role=rows[0]["ignis_role"] if rows else None,
                             ignis_no_median=[r["flame"] for r in rows if r["ignis_median_ext_s"] is None])
    res["event_time_error_extinguished_windowed"] = et
    # Snegirev / Irace as classifiers on the test days (per test, cutoff-free)
    cls = {}
    for fam in ("acme_normal", "acme_inverse"):
        rows = {}
        for k in tb[f"{fam}/nostop"]:
            rows[k["test"]] = (k["sneg"]["R"], k["sneg_k41"]["R"], k["irace"]["R"], k["end"]["cause"])
        ext = [v for v in rows.values() if v[3] == "extinction"]
        oth = [v for v in rows.values() if v[3] != "extinction"]
        cls[fam] = dict(tests=len(rows), extinguished=len(ext), stopped_or_right=len(oth),
                        snegirev_R_gt_1=dict(extinguished=sum(v[0] > 1 for v in ext), other=sum(v[0] > 1 for v in oth)),
                        snegirev_kappa4p1_R_gt_1=dict(extinguished=sum(v[1] > 1 for v in ext), other=sum(v[1] > 1 for v in oth)),
                        irace_R_gt_1=dict(extinguished=sum(v[2] > 1 for v in ext), other=sum(v[2] > 1 for v in oth)),
                        snegirev_R_range=[round(min(v[0] for v in rows.values()), 3), round(max(v[0] for v in rows.values()), 3)])
    res["test_day_classification"] = cls
    res["reproduce_abs_diff_max"] = worst
    res["reproduced"] = all(isinstance(v, float) and v <= 1e-9 for v in worst.values())
    def clean(o):
        if isinstance(o, float):
            return o if math.isfinite(o) else ("inf" if o > 0 else "-inf") if not math.isnan(o) else None
        if isinstance(o, dict):
            return {str(k): clean(v) for k, v in o.items()}
        if isinstance(o, (list, tuple)):
            return [clean(v) for v in o]
        return o
    OUT.write_text(json.dumps(clean(res), indent=1, default=str), encoding="utf-8")
    print("wrote", OUT.relative_to(ROOT).as_posix(), "reproduced:", res["reproduced"])
    print(json.dumps(worst, indent=0))


if __name__ == "__main__":
    main()
