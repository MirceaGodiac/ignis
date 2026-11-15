"""Irace et al.'s radiative-extinction law against the adopted Flame Design releases on the spent TEST days.

Post-hoc, 15 November 2026, after the final run of PROTOCOL 12.11. Nothing is adopted or refused, no IGNIS weight
is read, fitted or tuned. Template: the Xi et al. head-to-head (posthoc_xi_comparator.py).

Inputs, all written by the final run (no new reveal, no evaluator call):
  runs/final/20261115/local/final/flame-design/flame_design/{conditions,endpoints}.csv   test-day endpoints
  runs/final/20261115/local/final/flame-design/forward/prefix/<fam>/*_f2_{nostop,stop}.json  what F2 read
  runs/final/20261115/local/final/flame-design/forward/artifact/<fam>/<role>/*.json      sealed forecasts
The comparator (posthoc_irace_comparator.irace) reads only the conditions in F2's prefix (pressure, reactant
mass flow, burner and ambient mole fractions) and, on the stop path, the same planned stop F2 read.

Comparator = Irace et al. R = r_ss / r_ext (their Eq. 6.9, with Eqs. 4.1, 4.9/6.5/6.7, 4.10, 5.2/5.4 and their
constants) decides WHETHER the flame goes out; WHEN comes from the erfc growth clock, which is ours
(docs/wsp/research/lit_spherical_flames.md), not theirs: their burn times come from a transient CFD driven by
each test's measured burner temperature, which is not a forecast and is not rebuilt. A step forecast: extinction
at t_ext, unless the known planned stop comes first. Also scored: R alone with the clock replaced by "goes out at
the first horizon" (step at 6 s), to separate their criterion from our clock.

Check: F2, F1 and the surrogate re-scored here must reproduce runs/final/20261115/flame-design.json.

    python tools/wsp/final/posthoc_irace_testdays.py
"""
import json
import math
import pathlib
import sys

HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import run_final as RF  # noqa: E402
import posthoc_irace_comparator as IC  # noqa: E402

ROOT = HERE.parents[2]
LOCAL = ROOT / "runs/final/20261115/local/final/flame-design"
REF = ROOT / "runs/final/20261115/flame-design.json"
OUT = ROOT / "runs/final/20261115/posthoc_irace_testdays.json"
ROLES = ("f2", "f1", "surrogate")


def kc_value(kc, k):
    v = kc[k]
    return v.get("value_si") if v.get("value_si") is not None else v.get("raw_value")


def irace_forecast(fam, prefix, horizons, stop, clock):
    kc = prefix["known_conditions"]
    kind = "normal" if fam == "acme_normal" else "inverse"
    m_r = kc_value(kc, "injected_mass_flow")
    x_f, x_o = kc_value(kc, "x_fuel_role"), kc_value(kc, "x_o2_role")
    m_tot = m_r / (IC.y_of(x_f, IC.W_C2H4) if kind == "normal" else IC.y_of(x_o, IC.W_O2))
    # Zst from Eq. 4.1 with the stream compositions (normal: fuel in the burner; inverse: fuel in the ambient)
    z = IC.zst(x_f, x_o)
    q = IC.irace(kind, m_r, m_tot, z, kc_value(kc, "pressure"))
    t_ext = q["t_ext_erfc_s"] if clock == "erfc" else (horizons[0] if q["R"] > 1 else math.inf)
    return IC.step_cif(t_ext, stop, horizons), q


def main():
    tf2, ta, g6, tf, fdm = RF.fd_modules()
    tests = RF.fd_test_day_tests(ta, fdm, LOCAL / "flame_design/conditions.csv", LOCAL / "flame_design/endpoints.csv")
    ref = json.loads(REF.read_text(encoding="utf-8"))
    res = dict(what=__doc__.strip().splitlines()[0], partition="test_days (spent in the final run)", blocks={}, per_test={})
    worst = 0.0
    for fam in ("acme_normal", "acme_inverse"):
        for stop in (False, True):
            key = f"{fam}/{'stop' if stop else 'nostop'}"
            tag = "stop" if stop else "nostop"
            cases = []
            for t in (t for t in tests if t["family"] == fam):
                for c in ta.CUTOFFS:
                    if not ta.at_risk(t["endpoint"], c):
                        continue
                    name = lambda role: f"{t['test']}_c{c:g}_{role}_{tag}.json"  # noqa: E731
                    art = {r: json.loads((LOCAL / "forward/artifact" / fam / r / name(r)).read_text(encoding="utf-8")) for r in ROLES}
                    pre = json.loads((LOCAL / "forward/prefix" / fam / name("f2")).read_text(encoding="utf-8"))
                    st = None
                    if stop and "programmed_stop" in pre["known_conditions"]:
                        v = kc_value(pre["known_conditions"], "programmed_stop")
                        st = v if v and v > 0 else None
                    fi, q = irace_forecast(fam, pre, ta.HORIZONS, st, "erfc")
                    fr, _ = irace_forecast(fam, pre, ta.HORIZONS, st, "R_only")
                    cases.append(dict(test=t["test"], session=t["session"], cutoff=c, outcome=ta.outcome_of(t["endpoint"]),
                                      art=art, irace=fi, irace_R=fr, q=q, stop=st,
                                      refused=[r for r, a in art.items() if not ta.admitted(a)]))
                    if not stop:
                        res["per_test"].setdefault(f"{fam}:{t['test']}", dict(
                            R=round(q["R"], 3), t_ext_erfc_s=q["t_ext_erfc_s"], in_1bar=q["in_published_domain"],
                            end=t["endpoint"]["allowed_causes"][0] if t["endpoint"]["allowed_causes"] else "right",
                            end_s=t["endpoint"]["last_confirmed_active_s"]))
            ans = [k for k in cases if not k["refused"]] if not stop else [k for k in cases if not k["refused"]]
            outcomes = [k["outcome"] for k in ans]
            ss = [k["session"] for k in ans]
            contrib = {}
            for r in ROLES + ("irace", "irace_R"):
                cif = {cz: [(ta.cif_of(k["art"][r]) if r in ROLES else k[r])[cz] for k in ans] for cz in ta.CAUSES}
                s = ta.score(outcomes, cif)
                contrib[r] = {cz: s[cz]["contrib"] for cz in ta.CAUSES}
                contrib[r]["sum"] = [a + b for a, b in zip(contrib[r]["extinction"], contrib[r]["operator_termination"])]
            score = {r: {m: tf.means(v, ss)["case_mean"] for m, v in d.items()} for r, d in contrib.items()}
            paired = {f"f2_minus_{o}": {m: ta.boot([a - b for a, b in zip(contrib["f2"][m], contrib[o][m])], ss)
                                        for m in ("extinction", "operator_termination", "sum")} for o in ("irace", "irace_R")}
            rb = ref["blocks"][key]["all_answer"]
            check = {}
            for r in ROLES:
                for m in ("extinction", "operator_termination", "sum"):
                    d = abs(score[r][m] - rb["score"][r][m]["case_mean"]) if rb["cases"] == len(ans) else float("nan")
                    check[f"{r}/{m}"] = d
                    worst = max(worst, d) if d == d else float("inf")
            res["blocks"][key] = dict(cases=len(ans), tests=len({k["test"] for k in ans}), sessions=len(set(ss)),
                                      reference_cases=rb["cases"], score=score, paired=paired, reproduce_abs_diff=check,
                                      R_gt_1_cases=sum(k["q"]["R"] > 1 for k in ans),
                                      outside_1bar_cases=sum(not k["q"]["in_published_domain"] for k in ans))
    res["reproduced_final_run"] = worst <= 1e-9
    res["max_abs_diff_to_final_run"] = worst
    # per-test: R as a classifier, event-time error on flames that went out
    for fam in ("acme_normal", "acme_inverse"):
        rows = [v for k, v in res["per_test"].items() if k.startswith(fam)]
        ext = [v for v in rows if v["end"] == "extinction"]
        ln = sorted(math.log(v["t_ext_erfc_s"] / v["end_s"]) for v in ext if math.isfinite(v["t_ext_erfc_s"]))
        res[f"summary_{fam}"] = dict(tests=len(rows), extinguished=len(ext), extinguished_R_gt_1=sum(v["R"] > 1 for v in ext),
                                     stopped=len(rows) - len(ext), stopped_R_gt_1=sum(v["R"] > 1 for v in rows if v["end"] != "extinction"),
                                     ln_tpred_over_tobs_median=ln[len(ln) // 2] if ln else None, n_ln=len(ln),
                                     outside_1bar=sum(not v["in_1bar"] for v in rows))
    OUT.write_text(json.dumps(res, indent=1, default=str), encoding="utf-8")
    for key, b in res["blocks"].items():
        print(key, b["cases"], "ref", b["reference_cases"], {r: tuple(round(b["score"][r][m], 4) for m in ("extinction", "operator_termination", "sum")) for r in b["score"]})
        print("   paired f2-irace", {m: b["paired"]["f2_minus_irace"][m] for m in ("sum",)})
    print("reproduced:", res["reproduced_final_run"], res["max_abs_diff_to_final_run"])
    print(json.dumps({k: v for k, v in res.items() if k.startswith("summary_")}, indent=1))


if __name__ == "__main__":
    main()
