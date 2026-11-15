"""Head-to-head with the extinction law of Irace et al. (PCI 39, 2023), rebuilt from their equations. DRAFT.

Post-hoc, 15 November 2026, after the final run of PROTOCOL 12.11. Nothing is adopted or refused, no IGNIS weight
is read or fitted, and no test-day or validation-day outcome is read. Nothing here has scored a test case.

Sources (full texts read; the typeset PCI/CNF versions were not downloadable, the authors' manuscripts are the
thesis chapters; transcription with page provenance in docs/wsp/research/irace2023_flames.json):
  Irace PhD 2022 (WashU, doi:10.7936/kpc0-th51): ch. 4 = Combust. Flame 229 (2021) 111373; ch. 5 = PCI 39 (2023)
  1905 ("critical temperature and reactant mass flux"); ch. 6 = PCI 39 (2023) ("existence of steady-state").
  Waddell PhD 2023 (UMD): Table 2.1 (17 normal flames that radiatively extinguished) and Figs. 2.12-2.13.

Their analytical predictor, from test inputs only (all of these are known before ignition):
  r_ext     = sqrt(m_R / (4 pi J_cr))                     Eq. 5.2 (normal, m_R = m_C2H4, J_cr = 0.20 g m-2 s-1)
                                                          Eq. 5.4 (inverse, m_R = m_O2,  J_cr = 0.68 g m-2 s-1)
  r_ss      = m_tot / (4 pi rhoD ln(1/(1 - Z_st)))        Eqs. 4.9 / 6.5 (normal)
            = m_tot / (4 pi rhoD ln(1/Z_st))              Eq. 6.7 (inverse)
  rhoD      = rho0 D0 (T_ch / 298 K)^1/2, rho0 = 1146 g m-3, D0 = 20.4 mm2 s-1     Eq. 4.10
            T_ch = (298 + T_ad)/2 for the quasi-steady size (ch. 4), (298 + 1130)/2 for the R criterion (ch. 6, p. 118)
  R         = r_ss / r_ext                                 Eq. 6.9: R > 1 -> extinguishes radiatively, R < 1 -> steady
  Z_st      = (1 + s Y_F / Y_O)^-1, s = 3 W_O2 / W_C2H4 = 3.42           Eq. 4.1
They publish NO analytical burn time. Their burn times come from the SphDiff transient CFD (57 species, statistical
narrow-band radiation), run with the MEASURED burner-temperature history as inner boundary condition; "on average
within 20 %" (ch. 5.3). Extinction is where the CFD flame radius reaches r_ext (Fig. 6.3a). That CFD is not public
and uses in-test measurements, so it is not a forecast at a cutoff and is not rebuilt here.

The time clock offered here as option `erfc` is NOT theirs: it is the far-field point-source growth
r_f = r_ss erfc(r_f / 2 sqrt(D t)) derived in docs/wsp/research/lit_spherical_flames.md (class D), with their r_ext,
their rhoD (T_ch of ch. 6) for r_ss and their D0 = 20.4 mm2 s-1 at ambient (ch. 6.3.1), D ~ 1/p. Extinction when
r_f reaches r_ext:  t_ext = r_ext^2 / (4 D erfcinv(r_ext / r_ss)^2) for R > 1; never for R <= 1.

Subcommands
  literature   fidelity checks against the authors' own published numbers (no repository data read)
  identify     match their per-flame tables to Flame Design tests; reads ONLY rows of train and validation days
               of the NASA condition tables (the row filter is on the test id, before any other column is read)
  dryrun       step-function forecasts on the TRAIN-day cases (runs/dev/.../fd_fuel/train_days_cases.jsonl), scored
               with the evaluator's IPCW integrated Brier (tools/wsp/ext/transfer_acme.score) next to the F1, ethylene-
               as-ethane and incidence contributions already stored there. F2 is not scored (no train forecasts on disk).

    python tools/wsp/final/posthoc_irace_comparator.py literature
    python tools/wsp/final/posthoc_irace_comparator.py identify
    python tools/wsp/final/posthoc_irace_comparator.py dryrun
"""
import argparse
import csv
import itertools
import json
import math
import pathlib
import statistics as st
import sys
from statistics import NormalDist

ROOT = pathlib.Path(__file__).resolve().parents[3]
LIT = ROOT / "docs/wsp/research/irace2023_flames.json"
SPLIT = ROOT / "docs/wsp/atlas/splits/PSI-10.json"
TABLES = ROOT / "runs/dev/data_all/raw/PSI-10/Experimental table"
TRAIN_CASES = ROOT / "runs/dev/data_all/work/transfer_acme/fd_fuel/train_days_cases.jsonl"
OUT = ROOT / "runs/final/20261115/posthoc_irace"

W_C2H4, W_O2, W_N2 = 28.054, 31.998, 28.013
S_MASS = 3 * W_O2 / W_C2H4                   # 3.42, "sigma" of Eq. 5.4
J_C2H4, J_O2 = 0.20e-3, 0.68e-3              # kg m-2 s-1
RHO0, D0 = 1.146, 20.4e-6                    # kg m-3, m2 s-1 (Eq. 4.10; D0 also the ambient D of Eq. 6.1)
T_CR, T_AMB = 1130.0, 298.0
ATM = 101325.0
_nd = NormalDist()


# --------------------------------------------------------------------------------------------- the law
def y_of(x, w, w_dil=W_N2):
    return x * w / (x * w + (1 - x) * w_dil)


def zst(x_fuel, x_o2):
    """Eq. 4.1 with Y_F of the fuel stream and Y_O of the oxidiser stream (N2 diluent)."""
    yf, yo = y_of(x_fuel, W_C2H4), y_of(x_o2, W_O2)
    return 1.0 / (1.0 + S_MASS * yf / yo)


def rho_d(t_ch):
    return RHO0 * D0 * math.sqrt(t_ch / 298.0)


def r_ext(m_reactant, kind):
    """Eqs. 5.2 / 5.4: m in kg/s -> m."""
    return math.sqrt(m_reactant / (4 * math.pi * (J_C2H4 if kind == "normal" else J_O2)))


def r_ss(m_tot, z, kind, t_ch):
    """Eqs. 4.9 / 6.5 (normal), 6.7 (inverse)."""
    L = math.log(1 / (1 - z)) if kind == "normal" else math.log(1 / z)
    return m_tot / (4 * math.pi * rho_d(t_ch) * L)


def erfcinv(y):
    return -_nd.inv_cdf(0.5 * y) / math.sqrt(2.0)


def irace(kind, m_reactant, m_tot, z, p_pa=ATM):
    """Their analytical quantities plus the (not published) erfc clock."""
    re_ = r_ext(m_reactant, kind)
    rs = r_ss(m_tot, z, kind, (T_AMB + T_CR) / 2)
    R = rs / re_
    D = D0 * ATM / p_pa
    t = re_ ** 2 / (4 * D * erfcinv(re_ / rs) ** 2) if R > 1 else math.inf
    return dict(r_ext_mm=1e3 * re_, r_ss_mm=1e3 * rs, R=R, t_ext_erfc_s=t,
                in_published_domain=abs(p_pa / 1e5 - 1) <= 0.05)


# --------------------------------------------------------------------------------------------- literature
def cmd_literature(_a):
    lit = json.loads(LIT.read_text(encoding="utf-8"))
    tab = lit["tables"]
    out = {}

    # 1. Table 4.2 (CNF 2021): quasi-steady radius from Eqs. 4.9-4.10 with T_ch = (298 + T_ad)/2
    t41 = tab["irace2021_table1"]
    pub = t41["quasi_steady_radius_mm_table2"]
    rows = []
    for fl, xo, xf, mf, mn, tad, z in t41["rows"]:
        r = 1e3 * r_ss((mf + mn) * 1e-6, z, "normal", (T_AMB + tad) / 2)
        rows.append(dict(flame=fl, ours_mm=round(r, 2), published_mm=pub[str(fl)], diff_mm=round(r - pub[str(fl)], 2)))
    out["table4_2_quasi_steady_radius"] = dict(rows=rows, max_abs_diff_mm=max(abs(r["diff_mm"]) for r in rows))

    # 2. Eq. 5.2 against Waddell's 17 measured extinction radii (Fig. 2.13, vector-exact; labels within a fuel-flow
    #    stack are not published, so every within-stack assignment is tried and the range reported)
    t21 = tab["waddell_table2_1"]
    flames = []
    for fl, xo, xf, mt, tad, z in t21["rows"]:
        mf = mt * y_of(xf, W_C2H4)
        flames.append(dict(flame=fl, m_c2h4_mg_s=mf, r_ext_pred_mm=1e3 * r_ext(mf * 1e-6, "normal"),
                           R=irace("normal", mf * 1e-6, mt * 1e-6, z)["R"]))
    meas = [m for m, _ in t21["fig2_13_extinction_radius_mm"]["points"]]
    sim = [s for _, s in t21["fig2_13_extinction_radius_mm"]["points"]]
    stacks = {}
    for f in sorted(flames, key=lambda f: f["m_c2h4_mg_s"]):
        key = min((0.62, 0.82, 1.31, 1.64, 2.53), key=lambda c: abs(c - f["m_c2h4_mg_s"]))
        stacks.setdefault(key, []).append(f)
    order = sorted(range(len(meas)), key=lambda i: meas[i])
    groups, i = [], 0
    for key in sorted(stacks):
        n = len(stacks[key])
        groups.append((stacks[key], [order[i + j] for j in range(n)]))
        i += n
    results = []
    for perms in itertools.product(*[itertools.permutations(idx) for _, idx in groups]):
        pred, obs, mdot = [], [], []
        for (fl, _), perm in zip(groups, perms):
            for f, k in zip(fl, perm):
                pred.append(f["r_ext_pred_mm"]); obs.append(meas[k]); mdot.append(f["m_c2h4_mg_s"])
        J = [1e3 * (m * 1e-6) / (4 * math.pi * (r * 1e-3) ** 2) for m, r in zip(mdot, obs)]   # g m-2 s-1
        mo = st.mean(obs)
        r2 = 1 - sum((o - p) ** 2 for o, p in zip(obs, pred)) / sum((o - mo) ** 2 for o in obs)
        err = [100 * (p - o) / o for o, p in zip(obs, pred)]
        results.append((st.mean(J), st.stdev(J), r2, st.mean(abs(e) for e in err), max(abs(e) for e in err)))
    lo = [min(r[k] for r in results) for k in range(5)]
    hi = [max(r[k] for r in results) for k in range(5)]
    out["eq5_2_vs_waddell_measured_r_ext"] = dict(
        n=len(meas), assignments_tried=len(results),
        J_measured_mean_g_m2_s=[round(lo[0], 4), round(hi[0], 4)], J_measured_sd=[round(lo[1], 4), round(hi[1], 4)],
        R2_eq5_2=[round(lo[2], 3), round(hi[2], 3)], mean_abs_pct_err=[round(lo[3], 2), round(hi[3], 2)],
        max_abs_pct_err=[round(lo[4], 2), round(hi[4], 2)],
        published="J = 0.2 +/- 0.014 g m-2 s-1 and R2 = 0.95 on all 49 flames (irace_diss p. 100)",
        their_cfd_vs_measured_R2=round(1 - sum((s - m) ** 2 for s, m in zip(sim, meas)) / sum((m - st.mean(meas)) ** 2 for m in meas), 3),
        their_cfd_R2_published=0.98,
        stacks={f"{k:g}": [f["flame"] for f in v] for k, v in stacks.items()})
    out["R_criterion_on_waddell_17_extinguished"] = dict(
        R_min=round(min(f["R"] for f in flames), 2), R_max=round(max(f["R"] for f in flames), 2),
        all_R_gt_1=all(f["R"] > 1 for f in flames))

    # 3. extinction temperatures read from Fig. 2.12
    tm = list(t21["fig2_12_extinction_T_K"]["measured"].values())
    ts = list(t21["fig2_12_extinction_T_K"]["simulated"].values())
    out["fig2_12_T_ext"] = dict(measured=[round(st.mean(tm), 1), round(st.stdev(tm), 1)], simulated=[round(st.mean(ts), 1), round(st.stdev(ts), 1)],
                                published="1130 +/- 23 K measured, 1126 +/- 6 K simulated (Waddell p. 38)")

    # 4. R criterion (Eq. 6.9) on Table 6.1 and on Waddell Table 2.2; Irace 2021 flames
    t61 = tab["irace2023b_table1"]
    rows = []
    expect = {"NS1": ">1", "NS2": "<1", "IS1": ">1", "IS2": "<1", "IE1": "<1", "IE2": "<1"}
    for fl, xf, xo, mr, mt in t61["rows"]:
        kind = "normal" if fl.startswith("N") else "inverse"
        z = zst(xf, xo)
        q = irace(kind, mr * 1e-6, mt * 1e-6, z)
        rows.append(dict(flame=fl, Z_st=round(z, 3), R=round(q["R"], 3), expected=expect[fl],
                         agrees=(q["R"] > 1) == (expect[fl] == ">1"), r_ext_mm=round(q["r_ext_mm"], 1), r_ss_mm=round(q["r_ss_mm"], 1)))
    out["table6_1_R"] = rows
    rows = []
    for fl, kind, xo, xf, mt, tad, z, p in tab["waddell_table2_2"]["rows"]:
        if kind == "normal":
            mr = mt * y_of(xf, W_C2H4)
        else:
            mr = mt * y_of(xo, W_O2)
        q = irace(kind, mr * 1e-6, mt * 1e-6, z, p * ATM)
        rows.append(dict(flame=fl, kind=kind, R=round(q["R"], 2), r_ext_mm=round(q["r_ext_mm"], 1), extinguished_published=True,
                         agrees=q["R"] > 1, t_ext_erfc_s=round(q["t_ext_erfc_s"], 1)))
    out["waddell_table2_2_R"] = rows
    rows = []
    for fl, xo, xf, mf, mn, tad, z in t41["rows"]:
        q = irace("normal", mf * 1e-6, (mf + mn) * 1e-6, z)
        rows.append(dict(flame=fl, R=round(q["R"], 2), r_ext_mm=round(q["r_ext_mm"], 1), t_ext_erfc_s=round(q["t_ext_erfc_s"], 1),
                         outcome=t41["outcomes"][str(fl)]["end"]))
    out["irace2021_flames"] = dict(rows=rows, note="flame 3 went out at 337 s (measured), 335 s (their CFD); the erfc clock is not theirs")
    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / "literature.json").write_text(json.dumps(out, indent=1), encoding="utf-8")
    json.dump(out, sys.stdout, indent=1)
    return 0


# --------------------------------------------------------------------------------------------- identify
def allowed_sessions():
    sp = json.loads(SPLIT.read_text(encoding="utf-8"))
    part = {}
    for stratum in sp["strata"].values():
        for p in ("train", "validation"):
            for g in stratum["groups"][p]:
                part[g.split(":")[-1]] = p
    return part


def table_rows(kind, part):
    """Rows of train and validation days only; the test id is read first and other rows are skipped unparsed."""
    path = TABLES / f"PSI-10_Experimental table_Flame Design_{kind.capitalize()} Flame.csv"
    with open(path, newline="", encoding="utf-8-sig") as f:
        header = next(csv.reader([f.readline()]))
        ti = header.index("Test")
        for line in f:
            test = line.split(",")[ti].strip()
            if test[:5] not in part:
                continue
            row = dict(zip(header, next(csv.reader([line]))))
            row["_partition"] = part[test[:5]]
            yield row


def model_conditions(kind, part):
    """The conditions the releases read (normal: ambient O2, burner fuel, fuel mass flow; inverse: burner O2, ambient
    fuel, O2 mass flow), for tests of train days (train_days_cases.jsonl) and validation days (fd_f2 prefixes)."""
    fam = "acme_normal" if kind == "normal" else "acme_inverse"
    out = {}
    for line in TRAIN_CASES.read_text(encoding="utf-8").splitlines():
        if line.strip():
            r = json.loads(line)
            if r["family"] == fam and r["session"] in part:
                out.setdefault(r["test"], r["conditions"])
    pdir = ROOT / "runs/dev/data_all/work/transfer_acme/fd_f2/prefix" / fam
    for f in sorted(pdir.glob("*_f2_nostop.json")):
        test = f.name.split("_")[0]
        if test[:5] in part and test not in out:
            kc = json.loads(f.read_text(encoding="utf-8"))["known_conditions"]
            out[test] = {k: (v.get("value_si") if v.get("value_si") is not None else v.get("raw_value")) for k, v in kc.items()}
    res = {}
    for test, c in out.items():
        if kind == "normal":
            m_tot = c["injected_mass_flow"] / y_of(c["x_fuel_role"], W_C2H4)
        else:
            m_tot = c["injected_mass_flow"] / y_of(c["x_o2_role"], W_O2)
        res[test] = dict(x_o2=c["x_o2_role"], x_fuel=c["x_fuel_role"], m_tot_mg_s=1e6 * m_tot,
                         m_reactant_mg_s=1e6 * c["injected_mass_flow"], p_atm=c["pressure"] / ATM)
    return res


def fnum(s):
    try:
        return float(s)
    except (TypeError, ValueError):
        return math.nan


def cmd_identify(_a):
    lit = json.loads(LIT.read_text(encoding="utf-8"))["tables"]
    part = allowed_sessions()
    normal = list(table_rows("normal", part))
    inverse = list(table_rows("inverse", part))
    model_conds = {k: model_conditions(k, part) for k in ("normal", "inverse")}
    pub = []   # (source, flame, kind, x_o2, x_fuel, m_tot, p_atm or None)
    for fl, xo, xf, mf, mn, tad, z in lit["irace2021_table1"]["rows"]:
        pub.append(("irace2021_table1", fl, "normal", xo, xf, mf + mn, None))
    for fl, xo, xf, mt, tad, z in lit["waddell_table2_1"]["rows"]:
        pub.append(("waddell_table2_1", fl, "normal", xo, xf, mt, 1.0))
    for fl, kind, xo, xf, mt, tad, z, p in lit["waddell_table2_2"]["rows"]:
        pub.append(("waddell_table2_2", fl, kind, xo, xf, mt, p))
    for fl, xf, xo, mr, mt in lit["irace2023b_table1"]["rows"]:
        if fl.startswith("IE"):
            pub.append(("irace2023b_table1", fl, "inverse", xo, xf, mt, None))
    for fl, xf, xo, mo2, z, tad, p in lit["waddell_table4_1"]["rows"]:
        pub.append(("waddell_table4_1", fl, "inverse", xo, xf, None, p, mo2))
    res = []
    for item in pub:
        src, fl, kind, xo, xf, mt, p = item[:7]
        mo2 = item[7] if len(item) > 7 else None
        cands, ramped = [], 0
        mc = model_conds[kind]
        for r in (normal if kind == "normal" else inverse):
            if kind == "normal":
                o_init, o_fin, x_f = fnum(r["XO2 (init)"]), fnum(r["XO2 (final)"]), fnum(r["Xfuel"])
                xf_fin = x_f
            else:
                o_init = o_fin = fnum(r["XO2"])
                x_f = fnum(r["XC2H4 initial"])
                xf_fin = fnum(r["XC2H4 final"])
            m_tab, mo2_tab, p_tab = fnum(r["mtot (mg/s)"]), fnum(r.get("mO2 (mg/s)")), fnum(r["pmean (atm)"])
            src_c = "nasa_table"
            if not all(math.isfinite(v) for v in (o_init, o_fin, x_f, xf_fin, m_tab)):
                c = mc.get(r["Test"])      # ramped rows ("RAMP"): fall back to the conditions the releases read
                if c is None:
                    ramped += 1
                    continue
                o_init = o_fin = c["x_o2"]
                x_f = xf_fin = c["x_fuel"]
                m_tab, mo2_tab, p_tab = c["m_tot_mg_s"], c["m_reactant_mg_s"], c["p_atm"]
                src_c = "release_conditions"
            d_o = min(abs(xo - o_init), abs(xo - (o_init + o_fin) / 2))
            d_f = min(abs(xf - x_f), abs(xf - (x_f + xf_fin) / 2))
            d_m = abs(mt - m_tab) / max(0.03, 0.02 * mt) if mt is not None else abs(mo2 - mo2_tab) / max(0.03, 0.02 * mo2)
            d_p = abs(p - p_tab) / 0.06 if (p is not None and math.isfinite(p_tab)) else 0.0
            parts = (d_o / 0.0045, d_f / 0.006, d_m, d_p)
            if not all(math.isfinite(v) for v in parts):
                ramped += 1
                continue
            score = max(parts)
            if score <= 1.0:
                cands.append(dict(test=r["Test"], partition=r["_partition"], score=round(score, 3), conditions_from=src_c,
                                  x_o2_init=round(o_init, 4), x_o2_final=round(o_fin, 4), p_mean_atm=round(p_tab, 3),
                                  d_xo2=round(d_o, 4), d_xfuel=round(d_f, 4), d_m=round(d_m, 3)))
        cands.sort(key=lambda c: c["score"])
        res.append(dict(source=src, flame=fl, kind=kind, candidates=cands, rows_without_numeric_conditions=ramped))
    summary = dict(train_validation_rows=dict(normal=len(normal), inverse=len(inverse)),
                   matched=sum(1 for r in res if r["candidates"]), unmatched=sum(1 for r in res if not r["candidates"]))
    out = dict(summary=summary, rule="match within 0.0045 in X_O2 (init or init/final mean; published uncertainty 0.004), "
               "0.006 in the fuel fraction, max(0.03 mg/s, 2 %) in total (or O2) mass flow, 0.06 atm in mean pressure where "
               "published; rows of test days are never parsed", flames=res)
    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / "identify.json").write_text(json.dumps(out, indent=1), encoding="utf-8")
    json.dump(out, sys.stdout, indent=1)
    return 0


# --------------------------------------------------------------------------------------------- dry run (train days)
def step_cif(t_ext, stop, horizons):
    """Step-function forecast: extinction at t_ext unless a known planned stop comes first."""
    if stop is not None and math.isfinite(stop) and stop < t_ext:
        return {"extinction": [0.0] * len(horizons), "operator_termination": [1.0 if h >= stop else 0.0 for h in horizons]}
    return {"extinction": [1.0 if h >= t_ext else 0.0 for h in horizons], "operator_termination": [0.0] * len(horizons)}


def cmd_dryrun(_a):
    sys.path.insert(0, str(ROOT / "tools/wsp/ext"))
    import transfer_acme as ta   # noqa: E402
    import transfer_fuel as tf   # noqa: E402
    import acme_script2          # noqa: E402
    rows = [json.loads(l) for l in TRAIN_CASES.read_text(encoding="utf-8").splitlines() if l.strip()]
    assert all(r["days"] == "train" for r in rows)
    out = {}
    tests_seen = {}
    for fam in ("acme_normal", "acme_inverse"):
        kind = "normal" if fam == "acme_normal" else "inverse"
        for path in ("nostop", "stop"):
            sel = [r for r in rows if r["family"] == fam and r["path"] == path and not r["f1_refused"]
                   and not r["surrogate_refused"] and "ibs_contrib" in r]
            outcomes, cif, info = [], {c: [] for c in ta.CAUSES}, []
            for r in sel:
                c = r["conditions"]
                m_r = c["injected_mass_flow"]
                if kind == "normal":
                    m_tot = m_r / y_of(c["x_fuel_role"], W_C2H4)
                else:
                    m_tot = m_r / y_of(c["x_o2_role"], W_O2)
                q = irace(kind, m_r, m_tot, c["zst"], c["pressure"])
                stop = None
                if path == "stop":
                    v = acme_script2.for_test("PSI-10", r["test"]).get("planned_stop_s")
                    stop = v if v is not None and v > 0 else None
                f = step_cif(q["t_ext_erfc_s"], stop, ta.HORIZONS)
                for k in ta.CAUSES:
                    cif[k].append(f[k])
                t = r["endpoint"]["time_s"]
                outcomes.append(dict(censoring="event", t_lower=t, t_upper=t, causes=[r["endpoint"]["cause"]]))
                info.append(dict(test=r["test"], cutoff=r["cutoff_s"], session=r["session"], **q, stop=stop, end=r["endpoint"]))
                tests_seen[(fam, r["test"])] = (q, r["endpoint"])
            s = ta.score(outcomes, cif)
            mine = {k: s[k]["contrib"] for k in ta.CAUSES}
            mine["sum"] = [a + b for a, b in zip(mine["extinction"], mine["operator_termination"])]
            ss = [r["session"] for r in sel]
            # harness check: F1 re-scored here from its stored forecast must equal its stored contribution
            f1cif = {k: [ta.interp_row(r["f1_forecast"]["times_s"],
                                       r["f1_forecast"]["event_cdf"][r["f1_forecast"]["event_categories"].index(k)], ta.HORIZONS)
                         for r in sel] for k in ta.CAUSES}
            s1 = ta.score(outcomes, f1cif)
            diff = max(abs(s1[k]["contrib"][i] - sel[i]["ibs_contrib"]["f1"][k]) for k in ta.CAUSES for i in range(len(sel)))
            block = dict(cases=len(sel), tests=len({r["test"] for r in sel}), sessions=len(set(ss)), models={},
                         harness_check_f1_max_abs_contrib_diff=diff)
            block["models"]["irace_R_plus_erfc_clock"] = {m: tf.means(mine[m], ss)["case_mean"] for m in ("extinction", "operator_termination", "sum")}
            for other in ("f1", "surrogate", "incidence_baseline"):
                v = {m: [r["ibs_contrib"][other][m] for r in sel] for m in ("extinction", "operator_termination", "sum")}
                block["models"][other] = {m: tf.means(v[m], ss)["case_mean"] for m in v}
            out[f"{fam}/{path}"] = block
    # per test (cutoff-free): R classification and event-time error on self-extinguished flames
    per = {}
    for (fam, test), (q, end) in tests_seen.items():
        b = per.setdefault(fam, dict(tests=0, ext=0, ext_R_gt_1=0, stopped=0, stopped_R_gt_1=0, outside_1bar=0, ln_err=[]))
        b["tests"] += 1
        b["outside_1bar"] += not q["in_published_domain"]
        if end["cause"] == "extinction":
            b["ext"] += 1
            b["ext_R_gt_1"] += q["R"] > 1
            if math.isfinite(q["t_ext_erfc_s"]):
                b["ln_err"].append(math.log(q["t_ext_erfc_s"] / end["time_s"]))
        else:
            b["stopped"] += 1
            b["stopped_R_gt_1"] += q["R"] > 1
    for b in per.values():
        e = sorted(b.pop("ln_err"))
        b["ln_tpred_over_tobs"] = dict(n=len(e), median=round(st.median(e), 3) if e else None,
                                       mean_abs=round(st.mean(abs(x) for x in e), 3) if e else None)
    res = dict(note="TRAIN days only; F2 not scored (no train forecasts on disk); outcomes taken as events at the stored "
                    "endpoint time; the erfc clock is not published by Irace et al.", brier_case_mean=out, per_test=per)
    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / "dryrun_train.json").write_text(json.dumps(res, indent=1), encoding="utf-8")
    json.dump(res, sys.stdout, indent=1)
    return 0


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    sub = ap.add_subparsers(dest="cmd", required=True)
    for name, fn in (("literature", cmd_literature), ("identify", cmd_identify), ("dryrun", cmd_dryrun)):
        sub.add_parser(name).set_defaults(fn=fn)
    a = ap.parse_args()
    return a.fn(a)


if __name__ == "__main__":
    sys.exit(main())
