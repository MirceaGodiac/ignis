#!/usr/bin/env python3
"""Scoring path of the seventh-generation W0 law releases (PROTOCOL 13.2): one blind look per registered release,
beside every baseline this tool builds, through the data lane's own score() (unchanged).

    python -I tools/wsp/ext/law_score.py look --slug bre --release sha256:... --partition validation --look 13.2
    python -I tools/wsp/ext/law_score.py look --slug bre --release sha256:... --partition train      (the train study)
    python -I tools/wsp/ext/law_score.py score --slug bre --baseline incidence --partition train      (one baseline)

`look` runs the release, then every baseline twice: on all the lane's cases, and on the cases the release answers
(PROTOCOL 13.2 reads the result "on the cases the release answers"). It writes aggregates, counts, refusals and the
reading per scored target (beats the simple baseline / beats the best baseline / loses) to
docs/wsp/feedback/<slug>_w0_result.md (or --result-out). On TRAIN it also writes the per-case study file
runs/dev/data_all/work/transfer_<slug>/law_study_<id16>.jsonl (inputs, forecast, outcome, loss contribution).

Mapping from the law's targets to the lane's (runs/dev/gen7/registrations_draft.md, decisions of PROTOCOL 13.2):
  bre     T1 = flame_end (one cause) cut at the lane's planned stop (planned_stop_s + 0.569 - 0.796 s, the data
          lane's two train medians); the mass left at the stop is operator termination. Clock: the lane's onset.
  spice   T1 = smoke_point_length (fuel_mass_flow at the declared centre, weight zero); T2 = flame_length at the
          given flow. Quantiles x 1000 (mm).
  efield  T1 = ion_current x 1e6 (uA) per level; T2 = blowoff + other rows of flame_end; p_end_in_window at window_s.
  samer, same   d10 quantiles; iss_full_scale probability.
  daft    number_concentration quantiles.
  bass1   quench_u = quench_speed (nose facing the flow) or flat_quench_speed (films, slabs) x 100 (cm/s).
          sustain is not scored (PROTOCOL 13.2).
  slice   blow_off, lifted. Validation lines without cabin oxygen get the median of the cabin oxygen recorded on the
          train lines (PROTOCOL 13.2; the value is written in the result).
  cld     loss = blowoff + other rows at the horizons, with the planned fuel-off as the declared stop;
          p_loss_before_fuel_off = that sum at the fuel-off; radiometer = log10 of the volt quantiles.

Blindness: a validation or held partition needs --look; its outcomes are read only then; only aggregates (per-day
means where the lane gives them) are written; no per-case file is written for it.
"""
import argparse
import copy
import json
import math
import statistics
import subprocess
import sys
from collections import defaultdict
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[2]
sys.path.insert(0, str(HERE))
WORK = REPO / "runs/dev/data_all/work"
NOT_CAUSES = ("active", "scheduled_stop")   # law.rs ACTIVE and STOPPED: the event categories that are not causes
BASS_CODING = "v1"   # plastics coding: v1 (registered), v2, or v3 (after the final run, PROTOCOL 13.8)
SLICE_CASES = "v1"   # SLICE case version: v1 (registered) or v2 (tube from the picture table, tools/wsp/ext/slice_cases_v2.py)
import law_inputs as LI  # noqa: E402

LK = ["0.05", "0.25", "0.5", "0.75", "0.95"]
NAN = float("nan")


def jl(path):
    return [json.loads(l) for l in Path(path).read_text(encoding="utf-8").splitlines() if l.strip()]


def run_law(slug, release, rows, targets, times=None, tag="run"):
    """law_run.py on law rows; returns {case_id: {target: summary}}."""
    d = REPO / "runs/dev/gen7/score_work"
    d.mkdir(parents=True, exist_ok=True)
    cp, op = d / f"{slug}_{tag}_cases.jsonl", d / f"{slug}_{tag}_forward.jsonl"
    cp.write_text("".join(json.dumps(r) + "\n" for r in rows), encoding="utf-8", newline="\n")
    cmd = [sys.executable, str(REPO / "tools/wsp/law_run.py"), "--release", release, "--cases", str(cp), "--out", str(op),
           "--targets", targets]
    if times:
        cmd += ["--times", ",".join(str(t) for t in times)]
    r = subprocess.run(cmd, cwd=REPO, capture_output=True, text=True)
    if r.returncode != 0:
        sys.exit(r.stderr or r.stdout)
    return {x["case_id"]: x["targets"] for x in jl(op)}


def qs(s, scale=1.0):
    return [s["quantiles"][k] * scale for k in LK] if s and s.get("admitted") else None


def refusals(res, target):
    out = defaultdict(int)
    for t in res.values():
        s = t.get(target)
        if s and not s["admitted"]:
            for r in s["reasons"]:
                out[r.split(":")[0].split(" =")[0]] += 1
    return dict(out)


def get(d, path):
    for k in path.split("."):
        if d is None:
            return None
        d = d.get(k) if isinstance(d, dict) else None
    return d


# =============================================================================================== slugs
class Slug:
    """One slug: load(partition) -> ctx; release(ctx, id) -> fc; baselines(ctx) -> {name: fc} (simple first);
    restrict(fc, ref) -> fc on the cases ref answers; score(ctx, fc) -> {target: lane result};
    metrics: {target: [(label, path in the lane result)]}, lower is better; study(ctx, fc) -> rows (train)."""


def bre_slug():
    import bre_cases as L
    from transfer_acme import ipcw_terms
    S = Slug()
    S.metrics = {"t1": [("extinction IBS", "causes.extinction.integrated_brier"),
                        ("operator termination IBS", "causes.operator_termination.integrated_brier"),
                        ("mean of the two causes", "integrated_brier_mean_of_causes")]}
    base = json.loads((WORK / "transfer_bre/baselines_train.json").read_text(encoding="utf-8"))["t1"]
    c = base["planned_stop"]["constants"]
    delta, onset_off = c["stop_minus_planned_median_s"], c["onset_minus_stamp_median_s"]
    inc = {float(cc): v for cc, v in base["incidence"]["constants"].items()}

    def load(part):
        if part == "train":
            rows = jl(WORK / "transfer_bre/cases_train.jsonl")
        else:
            outs = {r["test"]: r["outcome"] for r in jl(WORK / "transfer_bre/cases_validation_outcomes.jsonl")}
            rows = [dict(r, outcome=outs[r["test"]]) for r in jl(WORK / "transfer_bre/cases_validation_inputs.jsonl")]
        return dict(rows=rows, cases=L.t1_cases(rows), inputs={r["test"]: r["inputs"] for r in rows}, part=part)

    def stop_of(ctx, test):
        ps = ctx["inputs"][test].get("planned_stop_s")
        return None if ps is None else ps + delta - onset_off

    def cut(ext_at, stop):
        if stop is None:
            return dict(extinction=[ext_at(h) for h in L.HORIZONS], operator_termination=[0.0] * len(L.HORIZONS))
        fe_stop = ext_at(stop)
        return dict(extinction=[ext_at(h) if h < stop else fe_stop for h in L.HORIZONS],
                    operator_termination=[(1.0 - fe_stop) if h >= stop else 0.0 for h in L.HORIZONS])

    def release(ctx, rid):
        law = {r["case_id"]: r for r in LI.law_rows("bre", ctx["rows"])}
        lrows, keys = [], []
        for k in ctx["cases"]:
            st = stop_of(ctx, k["test"])
            st = st if (st is not None and st > k["cutoff_s"]) else None
            times = sorted(set([h for h in L.HORIZONS if h > k["cutoff_s"]] + ([round(st, 6)] if st is not None else [])))
            cid = f"{k['test']}|{k['cutoff_s']}"
            lrows.append(dict(case_id=cid, cutoff_s=k["cutoff_s"], times_s=times, inputs=law[k["test"]]["inputs"]))
            keys.append((k, cid, st))
        res = run_law("bre", rid, lrows, "flame_end", tag=ctx["part"])
        fc = []
        for k, cid, st in keys:
            s = res[cid]["flame_end"]
            if not s["admitted"]:
                continue
            m = dict(zip(s["times_s"], s["cdf"]["extinction"]))
            tt = sorted(m)
            ext_at = lambda h, m=m, tt=tt: m[max(t for t in tt if t <= h)] if tt and tt[0] <= h else 0.0  # noqa: E731
            fc.append(dict(test=k["test"], cutoff_s=k["cutoff_s"], cif=cut(ext_at, st)))
        ctx["refusals"] = {"t1": refusals(res, "flame_end")}
        ctx["n_asked"] = {"t1": len(lrows)}
        return {"t1": fc}

    def baselines(ctx):
        fi, fp = [], []
        for k in ctx["cases"]:
            fe = inc[k["cutoff_s"]]["extinction"]
            fi.append(dict(test=k["test"], cutoff_s=k["cutoff_s"], cif=inc[k["cutoff_s"]]))
            st = stop_of(ctx, k["test"])
            ext_at = lambda h, fe=fe: fe[max([j for j, x in enumerate(L.HORIZONS) if x <= h], default=0)] if h >= L.HORIZONS[0] else 0.0  # noqa: E731
            fp.append(dict(test=k["test"], cutoff_s=k["cutoff_s"], cif=inc[k["cutoff_s"]] if st is None else cut(ext_at, st)))
        return {"incidence (simple)": {"t1": fi}, "planned stop": {"t1": fp}}

    def restrict(fc, ref):
        keys = {(f["test"], float(f["cutoff_s"])) for f in ref["t1"]}
        return {"t1": [f for f in fc["t1"] if (f["test"], float(f["cutoff_s"])) in keys]}

    def score(ctx, fc):
        # IPCW weights come from the outcomes of the rows scored: score on the tests that have a forecast
        tests = {f["test"] for f in fc["t1"]}
        rows = [r for r in ctx["rows"] if r["test"] in tests]
        return {"t1": L.score("t1", fc["t1"], rows)}

    def study(ctx, fc):
        fmap = {(f["test"], float(f["cutoff_s"])): f for f in fc["t1"]}
        cases = ctx["cases"]
        outcomes = [k["outcome"] for k in cases]
        out = []
        for i, k in enumerate(cases):
            f = fmap.get((k["test"], k["cutoff_s"]))
            contrib = {}
            for cause in L.CAUSES:
                row = []
                for j, h in enumerate(L.HORIZONS):
                    elig, w, ind, _, _ = ipcw_terms(outcomes, cause, h)
                    row.append(w[i] * (ind[i] - f["cif"][cause][j]) ** 2 if (f and elig[i]) else NAN)
                contrib[cause] = L.integrated_brier(L.HORIZONS, row) if f else None
            out.append(dict(case_id=f"{k['test']}|{k['cutoff_s']}", test=k["test"], day=k["day"], cutoff_s=k["cutoff_s"],
                            inputs=ctx["inputs"][k["test"]], forecast=f, outcome=k["outcome"],
                            loss_contribution=dict(ipcw_integrated_brier=contrib)))
        return out
    S.load, S.release, S.baselines, S.restrict, S.score, S.study = load, release, baselines, restrict, score, study
    return S


def spice_slug():
    import spice_cases as L
    S = Slug()
    S.metrics = {"T1": [("quantile score x2, mm, mean of day means", "aggregate_day_mean"), ("pooled mean", "pooled_mean")],
                 "T2": [("quantile score x2, mm, mean of day means", "aggregate_day_mean"), ("pooled mean", "pooled_mean")]}
    train = jl(WORK / "transfer_spice/cases_train.jsonl")

    def load(part):
        if part == "train":
            rows = train
        else:
            outs = {r["case_id"]: r for r in jl(WORK / "transfer_spice/cases_validation_outcomes.jsonl")}
            gf = {r["case_id"]: r.get("inputs_given_flow") for r in jl(WORK / "transfer_spice/cases_validation_inputs_given_flow.jsonl")}
            rows = [dict(r, outcome={k: v for k, v in outs[r["case_id"]].items() if k not in ("case_id", "day")},
                         inputs_given_flow=gf.get(r["case_id"])) for r in jl(WORK / "transfer_spice/cases_validation_inputs.jsonl")]
        return dict(rows=rows, part=part, out={c["case_id"]: {"day": c["day"], "L_sp_mm": c["outcome"]["L_sp_mm"]} for c in rows})

    def release(ctx, rid):
        rows = ctx["rows"]
        t1rows = [dict(r, inputs=dict(r["inputs"], fuel_mass_flow=1e-6)) for r in LI.law_rows("spice", [dict(c, inputs_given_flow=None) for c in rows])]
        t2rows = LI.law_rows("spice", rows)
        r1 = run_law("spice", rid, t1rows, "smoke_point_length", tag=ctx["part"] + "_t1")
        r2 = run_law("spice", rid, t2rows, "flame_length", tag=ctx["part"] + "_t2")
        f1 = {cid: dict(zip(L.QKEYS, qs(t["smoke_point_length"], 1000.0))) for cid, t in r1.items() if t["smoke_point_length"]["admitted"]}
        f2 = {cid: dict(zip(L.QKEYS, qs(t["flame_length"], 1000.0))) for cid, t in r2.items() if t["flame_length"]["admitted"]}
        ctx["refusals"] = {"T1": refusals(r1, "smoke_point_length"), "T2": refusals(r2, "flame_length")}
        ctx["n_asked"] = {"T1": len(t1rows), "T2": len(t2rows)}
        return {"T1": f1, "T2": f2}

    def baselines(ctx):
        rows = ctx["rows"]
        ys = [c["outcome"]["L_sp_mm"] for c in train]
        q0 = L.emp_q(ys)
        byf = defaultdict(list)
        for c in train:
            byf[c["inputs"]["fuel"]].append(c["outcome"]["L_sp_mm"])
        b0 = {c["case_id"]: q0 for c in rows}
        b1 = {c["case_id"]: (L.emp_q(byf[c["inputs"]["fuel"]]) if len(byf[c["inputs"]["fuel"]]) >= 4 else q0) for c in rows}
        # T2: the Lin and Faeth coflow law at its published constants, one log-normal spread from the train residuals
        r = [math.log(c["outcome"]["L_sp_mm"] / L.lin_faeth_coflow_mm(c)) for c in train if L.lin_faeth_coflow_mm(c)]
        sig = statistics.pstdev(r)
        lf = {c["case_id"]: L.lognormal_q(L.lin_faeth_coflow_mm(c), sig) for c in rows
              if c.get("inputs_given_flow") and L.lin_faeth_coflow_mm(c)}
        return {"B0 pooled climatology (simple)": {"T1": b0, "T2": b0}, "B1 per-fuel climatology": {"T1": b1, "T2": b1},
                "Lin and Faeth coflow law, train sigma (T2 only)": {"T2": lf}}

    def restrict(fc, ref):
        return {t: {k: v for k, v in fc[t].items() if k in ref.get(t, {})} for t in fc if t in ref}

    def score(ctx, fc):
        return {t: L.score(f, ctx["out"]) for t, f in fc.items()}

    def study(ctx, fc):
        f1, f2 = fc.get("T1", {}), fc.get("T2", {})
        return [dict(case_id=c["case_id"], day=c["day"], inputs=c["inputs"], inputs_given_flow=c.get("inputs_given_flow"),
                     forecast=dict(T1=f1.get(c["case_id"]), T2=f2.get(c["case_id"])), outcome=c["outcome"],
                     loss_contribution=dict(T1=L.qscore(f1[c["case_id"]], c["outcome"]["L_sp_mm"]) if c["case_id"] in f1 else None,
                                            T2=L.qscore(f2[c["case_id"]], c["outcome"]["L_sp_mm"]) if c["case_id"] in f2 else None))
                for c in ctx["rows"]]
    S.load, S.release, S.baselines, S.restrict, S.score, S.study = load, release, baselines, restrict, score, study
    return S


def efield_slug(slug="efield"):
    import efield_cases as L
    S = Slug()
    S.metrics = {"t1": [("CRPS uA, case mean", "t1_crps_ua"), ("mean of day means", "t1_crps_ua_mean_of_days")],
                 "t2": [("integrated Brier", "t2_integrated_brier"), ("Brier of ends in window", "t2_end_in_window_brier")]}
    b = json.loads((WORK / "transfer_efield/baselines_train.json").read_text(encoding="utf-8"))
    K = b["_constants"]
    train = [c for c in L.load("train") if c["outcomes"].get("status") == "ok"]

    def load(part):
        if part == "train":
            cases = L.load("train")
        else:
            outs = {r["test"]: r["outcomes"] for r in L.load("validation_outcomes")}
            cases = [dict(c, outcomes=outs[c["inputs"]["test"]]) for c in L.load("validation_inputs")]
        return dict(cases=cases, part=part)

    def release(ctx, rid):
        cases = ctx["cases"]
        law = LI.law_rows(slug, [{k: v for k, v in c.items() if k != "outcomes"} for c in cases])
        lv = [r for r in law if r["kind"] == "level"]
        ts = [r for r in law if r["kind"] == "test"]
        r1 = run_law(slug, rid, lv, "ion_current", tag=ctx["part"] + "_t1")
        win = {c["inputs"]["test"]: c["inputs"].get("window_s") for c in cases}
        t2rows = [dict(r, times_s=sorted(set([float(h) for h in L.HORIZONS_S] + ([float(win[r["test"]])] if win.get(r["test"]) else []))))
                  for r in ts]
        r2 = run_law(slug, rid, t2rows, "flame_end", tag=ctx["part"] + "_t2")
        t1, t2 = {}, {}
        for r in ts:
            s = r2[r["case_id"]]["flame_end"]
            if s["admitted"]:
                tt = s["times_s"]
                end = {h: sum(c[k] for n, c in s["cdf"].items() if n not in NOT_CAUSES) for k, h in enumerate(tt)}   # every declared cause (efield3: press)
                t2[r["test"]] = dict(cif={str(int(h)): end[float(h)] for h in L.HORIZONS_S},
                                     p_end_in_window=end[float(win[r["test"]])] if win.get(r["test"]) else None)
        for r in lv:
            q = qs(r1[r["case_id"]]["ion_current"], 1e6)
            if q:
                t1.setdefault(r["test"], {})[r["level_id"]] = q
        ctx["refusals"] = {"t1": refusals(r1, "ion_current"), "t2": refusals(r2, "flame_end")}
        ctx["n_asked"] = {"t1": len(lv), "t2": len(ts)}
        return {"t1": t1, "t2": t2}

    def baselines(ctx):
        cases = ctx["cases"]
        const_t1 = {c["inputs"]["test"]: {l["level_id"]: K["t1_constant_quantiles_ua"] for l in c["inputs"]["levels"]} for c in cases}
        const_t2 = {c["inputs"]["test"]: dict(cif=K["t2_cif"], p_end_in_window=K["t2_p_end_in_window"]) for c in cases}
        # space-charge law up to saturation (the lane's best classical T1 baseline), sigma fitted on train
        ys = [(c, lid, r) for c in train for lid, r in c["outcomes"]["t1"].items() if r["status"] == "ok"]
        pred = lambda c, lid: min(L.i_sat_ua(c["inputs"], L.ETA_DIFFUSION), L.i_scl_ua(c["inputs"], L.level_v(c, lid)))  # noqa: E731
        res = [math.log(max(r["ion_current_ua"], 1e-3) / p) for c, lid, r in ys for p in [pred(c, lid)] if p == p and p > 0]
        m = sum(res) / len(res)
        sg = (sum((x - m) ** 2 for x in res) / len(res)) ** 0.5
        sc_t1 = {}
        for c in cases:
            d = {}
            for l in c["inputs"]["levels"]:
                p = pred(c, l["level_id"])
                if p == p and p > 0:
                    d[l["level_id"]] = L.lognormal_q(p, sg)
            sc_t1[c["inputs"]["test"]] = d
        # stratified incidence (polarity x coflow x burner, Kaplan-Meier within each), as the lane builds it
        strat = lambda c: (c["inputs"]["polarity"], (c["inputs"]["u_coflow_m_s"] or 0) > 0, c["inputs"]["burner"])  # noqa: E731
        groups = defaultdict(list)
        for c in train:
            groups[strat(c)].append(c)

        def km(sub):
            tt = sorted({x["time_s"] for x in sub if x["event"]})
            cif_, s_ = {}, 1.0
            for h in L.HORIZONS_S:
                for x in [y for y in tt if (cif_.get("_l", -1) < y <= h)]:
                    n_ = sum(1 for z in sub if (z["time_s"] if z["event"] else z["observed_s"]) >= x)
                    s_ *= 1 - sum(1 for z in sub if z["event"] and z["time_s"] == x) / n_ if n_ else 1
                cif_["_l"] = h
                cif_[h] = 1 - s_
            return {str(h): cif_[h] for h in L.HORIZONS_S}
        st_cif = {k: km([c["outcomes"]["t2"] for c in v]) for k, v in groups.items()}
        st_p = {}
        for k, v in groups.items():
            cl = [c["outcomes"]["t2"] for c in v if c["outcomes"]["t2"]["censored_by"] in L.CLOSED]
            st_p[k] = sum(x["event"] for x in cl) / len(cl) if cl else K["t2_p_end_in_window"]
        strat_t2 = {}
        for c in cases:
            k = strat(c)
            strat_t2[c["inputs"]["test"]] = dict(cif=st_cif.get(k, K["t2_cif"]), p_end_in_window=st_p.get(k, K["t2_p_end_in_window"]))
        return {"constant (simple)": {"t1": const_t1, "t2": const_t2},
                "space-charge law up to saturation, train sigma (T1)": {"t1": sc_t1},
                "stratified incidence (T2; a stratum absent from train takes the pooled incidence)": {"t2": strat_t2}}

    def restrict(fc, ref):
        out = {}
        if "t1" in fc and "t1" in ref:
            out["t1"] = {t: {l: q for l, q in fc["t1"].get(t, {}).items() if l in lv} for t, lv in ref["t1"].items()}
        if "t2" in fc and "t2" in ref:
            out["t2"] = {t: v for t, v in fc["t2"].items() if t in ref["t2"]}
        return out

    def score(ctx, fc):
        res = {}
        for t in ("t1", "t2"):
            if t not in fc:
                continue
            f = {c["inputs"]["test"]: {"test": c["inputs"]["test"], t: fc[t].get(c["inputs"]["test"], {})} for c in ctx["cases"]
                 if c["inputs"]["test"] in fc[t]}
            r = L.score(f, ctx["cases"])
            res[t] = {k: v for k, v in r.items() if (k.startswith(t) or k in ("days", "per_day", "missing"))}
        return res

    def study(ctx, fc):
        out = []
        for c in ctx["cases"]:
            o, test = c["outcomes"], c["inputs"]["test"]
            if o.get("status") != "ok":
                continue
            for lid, r in o["t1"].items():
                if r["status"] != "ok":
                    continue
                q = fc.get("t1", {}).get(test, {}).get(lid)
                out.append(dict(case_id=f"{test}|{lid}", target="t1", day=c["inputs"]["day"],
                                inputs={k: v for k, v in c["inputs"].items() if k not in ("levels", "pre_field", "sweep")},
                                level=next((l for l in c["inputs"]["levels"] if l["level_id"] == lid), None), forecast=q,
                                outcome=r, loss_contribution=dict(crps_ua=L.crps_quantiles(L.LEVELS_Q, q, r["ion_current_ua"]) if q else None)))
            f2 = fc.get("t2", {}).get(test)
            out.append(dict(case_id=f"{test}|t2", target="t2", day=c["inputs"]["day"],
                            inputs={k: v for k, v in c["inputs"].items() if k not in ("levels", "pre_field", "sweep")}, forecast=f2, outcome=o["t2"],
                            loss_contribution=dict(brier_by_horizon={h: (f2["cif"][str(h)] - y) ** 2 for h in L.HORIZONS_S
                                                                     for y in [L.t2_eligible(o, h)] if y is not None and f2})))
        return out
    S.load, S.release, S.baselines, S.restrict, S.score, S.study = load, release, baselines, restrict, score, study
    return S


def samer_slug(slug):
    import samer_cases as L
    S = Slug()
    S.metrics = {"d10": [("CRPS of ln d10, case mean", "aggregate"), ("day-equal", "day_equal")],
                 "iss": [("Brier, case mean", "aggregate"), ("day-equal", "day_equal")]}
    train = jl(WORK / "transfer_samer/cases_train.jsonl")
    mat = lambda c: c["inputs"]["material"]["value"]  # noqa: E731

    def load(part):
        if slug == "samer" and part == "train":
            rows = train
        elif slug == "samer":
            outs = {r["case_id"]: r for r in jl(WORK / "transfer_samer/cases_validation_outcomes.jsonl")}
            rows = [dict(r, outcome=outs[r["case_id"]]["outcome"]) for r in jl(WORK / "transfer_samer/cases_validation_inputs.jsonl")]
        elif part == "train":
            sys.exit("same has no train partition; its revision may use the SAME-R train anatomy")
        else:
            outs = {r["case_id"]: r for r in jl(WORK / "transfer_same/cases_held_outcomes.jsonl")}
            rows = [dict(r, outcome=outs[r["case_id"]]["outcome"]) for r in jl(WORK / "transfer_same/cases_held_inputs.jsonl")]
        return dict(rows=rows, part=part)

    def release(ctx, rid):
        res = run_law(slug, rid, LI.law_rows(slug, ctx["rows"]), "d10,iss_full_scale", tag=ctx["part"])
        fd = {cid: qs(t["d10"]) for cid, t in res.items() if t["d10"]["admitted"]}
        fi = {cid: t["iss_full_scale"]["probability"] for cid, t in res.items() if t["iss_full_scale"]["admitted"]}
        ctx["refusals"] = {"d10": refusals(res, "d10"), "iss": refusals(res, "iss_full_scale")}
        ctx["n_asked"] = {"d10": len(res), "iss": len(res)}
        return {"d10": fd, "iss": fi}

    def baselines(ctx):
        rows = ctx["rows"]
        sc = [c for c in train if c["outcome"]["scored"]["d10"]]
        lnq = lambda v: [math.exp(x) for x in L.quantiles([math.log(c["outcome"]["d10_m"]["value"]) for c in v])]  # noqa: E731
        allq = lnq(sc)
        bym = defaultdict(list)
        for c in sc:
            bym[mat(c)].append(c)
        iss = [c for c in train if c["outcome"]["scored"]["iss"]]
        yv = lambda c: 1.0 if c["outcome"]["iss_full_scale"]["value"] else 0.0  # noqa: E731
        inc = sum(yv(c) for c in iss) / len(iss)
        incm = defaultdict(list)
        for c in iss:
            incm[mat(c)].append(yv(c))
        return {"B0 / I0 constant (simple, SAME-R train)": {"d10": {c["case_id"]: allq for c in rows}, "iss": {c["case_id"]: inc for c in rows}},
                "B1 / I1 per material (SAME-R train; a material absent from it takes B0 / I0)": {
                    "d10": {c["case_id"]: (lnq(bym[mat(c)]) if mat(c) in bym else allq) for c in rows},
                    "iss": {c["case_id"]: (sum(incm[mat(c)]) / len(incm[mat(c)]) if mat(c) in incm else inc) for c in rows}}}

    def restrict(fc, ref):
        return {t: {k: v for k, v in fc[t].items() if k in ref[t]} for t in fc}

    def score(ctx, fc):
        return {"d10": L.score(fc["d10"], ctx["rows"], "d10"), "iss": L.score(fc["iss"], ctx["rows"], "iss")}

    def study(ctx, fc):
        fd, fi = fc.get("d10", {}), fc.get("iss", {})
        return [dict(case_id=c["case_id"], group=c["group_id"], inputs={k: (v.get("value") if isinstance(v, dict) else v) for k, v in c["inputs"].items()},
                     forecast=dict(d10=fd.get(c["case_id"]), iss=fi.get(c["case_id"])),
                     outcome=dict(d10_m=c["outcome"]["d10_m"]["value"], iss_full_scale=c["outcome"]["iss_full_scale"]["value"],
                                  scored_d10=c["outcome"]["scored"]["d10"], scored_iss=c["outcome"]["scored"]["iss"]),
                     loss_contribution=dict(
                         d10_crps_ln=(L.crps_quantiles(L.LEVELS, [math.log(v) for v in fd[c["case_id"]]], math.log(c["outcome"]["d10_m"]["value"]))
                                      if c["case_id"] in fd and c["outcome"]["scored"]["d10"] else None),
                         iss_brier=((fi[c["case_id"]] - (1.0 if c["outcome"]["iss_full_scale"]["value"] else 0.0)) ** 2
                                    if c["case_id"] in fi and c["outcome"]["scored"]["iss"] else None)))
                for c in ctx["rows"]]
    S.load, S.release, S.baselines, S.restrict, S.score, S.study = load, release, baselines, restrict, score, study
    return S


def daft_slug():
    import daft_cases as L
    S = Slug()
    S.metrics = {"number_concentration": [("CRPS of ln(1 + N per cm3), case mean", "aggregate"), ("group-equal", "group_equal")]}

    def load(part):
        if part == "train":
            sys.exit("daft has no train partition (held whole)")
        return dict(rows=jl(WORK / "transfer_daft/cases_held_inputs.jsonl"), outs=jl(WORK / "transfer_daft/cases_held_outcomes.jsonl"), part=part)

    def release(ctx, rid):
        res = run_law("daft", rid, LI.law_rows("daft", ctx["rows"]), "number_concentration", tag="held")
        ctx["refusals"] = {"number_concentration": refusals(res, "number_concentration")}
        ctx["n_asked"] = {"number_concentration": len(res)}
        return {"number_concentration": {cid: qs(t["number_concentration"]) for cid, t in res.items() if t["number_concentration"]["admitted"]}}

    def baselines(ctx):
        k = json.loads((WORK / "transfer_daft/baseline_smoke_number_per_mass.json").read_text(encoding="utf-8"))["number_per_mass_per_kg"]
        fc = {}
        for r in ctx["rows"]:
            m = r["inputs"]["dusttrak_mass_concentration"]["value"]
            if m is not None:
                fc[r["case_id"]] = [kq * m for kq in k]
        return {"smoke number per mass, SAME-R train (simple)": {"number_concentration": fc}}

    def restrict(fc, ref):
        return {t: {k: v for k, v in fc[t].items() if k in ref[t]} for t in fc}

    def score(ctx, fc):
        return {"number_concentration": L.score(fc["number_concentration"], ctx["outs"])}
    S.load, S.release, S.baselines, S.restrict, S.score, S.study = load, release, baselines, restrict, score, None
    return S


def holds(p_low, p_high):
    """phys_bass1_w0.md section 2: "The law's probability that a flame holds is the smaller of the two." Taken here,
    at scoring time, from the two targets of the sample's geometry (holds_low_flow / holds_high_flow, or the flat_
    pair); the forward cannot nest the minimum. Not used yet: sustain is not scored (PROTOCOL 13.2)."""
    return None if p_low is None or p_high is None else min(p_low, p_high)


def bass1_slug(slug="bass1"):
    import bass1_cases as L
    S = Slug()
    S.metrics = {"quench": [("interval CRPS cm/s, mean of day means", "day_mean"), ("case mean", "case_mean")],
                 "sustain": [("Brier, mean of day means", "day_mean"), ("case mean", "case_mean")],
                 "quench_display": [("interval CRPS, fan display units, mean of day means", "day_mean"), ("case mean", "case_mean")]}
    S.notes = ["sustain: the law answers whether a flame, given that it was established, is still there 60 s after the igniter is "
               "switched off (phys_bass1_w0.md section 9); it does not forecast ignition. P = the smaller of holds_low_flow and "
               "holds_high_flow (rod tips, spheres, sections) or of the flat_ pair (films, slabs up to 5 mm), at the initial oxygen, "
               "the duct speed at ignition, nose radius diameter / 2 (0.01 flat); heat_time 200 s for every case (no case carries a "
               "planned igniter on-time)."]
    b = json.loads((WORK / "transfer_bass1/baselines_train.json").read_text(encoding="utf-8"))

    # v3 (bass1_cases_v3.py, made after the final run, PROTOCOL 13.8): scored like v2, constants from the v3 train file
    cod = {"v2": "_v2", "v3": "_v3"}.get(BASS_CODING, "")    # the lane's coding (docs/wsp/feedback/bass1_coding_v2.md); v1 registered

    def load(part):
        if part == "train":
            rows = jl(WORK / f"transfer_bass1/cases_train{cod}.jsonl")
            outs = [r["outcome"] for r in rows]
        else:
            # "test": the plastics test days, study data after the final run (PROTOCOL 13.8), as the final run built them
            rows = jl(WORK / f"transfer_bass1/cases_{part}_inputs{cod}.jsonl")
            outs = jl(WORK / f"transfer_bass1/cases_{part}_outcomes{cod}.jsonl")
            om = {o["case_id"]: o for o in outs}
            rows = [dict(r, outcome=om.get(r["case_id"], {})) for r in rows]
        return dict(rows=rows, outs=outs, part=part)

    def release(ctx, rid):
        d1 = REPO / "configs/models/law/bass1/d1-fable/release.json"
        LI.BASS1_D1_CONVENTION = d1.exists() and json.loads(d1.read_text(encoding="utf-8"))["id"] == rid
        if slug in ("bass2", "bass3", "bass4", "bass5"):
            law = []
            for r in ctx["rows"]:
                inp, cov = {"bass5": LI.bass5_row, "bass4": LI.bass4_row, "bass3": LI.bass3_row}.get(slug, LI.bass2_row)(r)
                law.append(dict(case_id=r["case_id"], coverage=cov, inputs=inp))
        else:
            law = LI.law_rows("bass1", [{k: v for k, v in r.items() if k != "outcome"} for r in ctx["rows"]])
        res = run_law(slug, rid, law, "quench_speed,flat_quench_speed", tag=ctx["part"])
        fc, ref = {}, defaultdict(int)
        for r in law:
            t = res[r["case_id"]]
            s = t["quench_speed"] if r["coverage"] == "nose" else t["flat_quench_speed"] if r["coverage"] == "flat" else None
            q = qs(s, 100.0)
            if q:
                fc[r["case_id"]] = q
            elif s is None:
                ref[f"not covered: {r['coverage'].split(':', 1)[-1]}"] += 1
            else:
                for x in s["reasons"]:
                    ref[x.split(":")[0].split(" =")[0]] += 1
        # sustain (PROTOCOL 13.2 amendment): the smaller of the two holds targets of the geometry
        srows, sref = [], defaultdict(int)
        for r in ctx["rows"]:
            inp, cov = (LI.bass5_row(r, "sustain") if slug == "bass5" else LI.bass4_row(r, "sustain") if slug == "bass4" else LI.bass3_row(r) if slug == "bass3" else LI.bass2_row(r) if slug == "bass2"
                        else LI.bass1_sustain_row(r))
            srows.append(dict(case_id=r["case_id"], coverage=cov, inputs=inp))
        sres = run_law(slug, rid, srows, "holds_low_flow,holds_high_flow,flat_holds_low_flow,flat_holds_high_flow",
                       tag=ctx["part"] + "_sustain")
        fs = {}
        for r in srows:
            t = sres[r["case_id"]]
            if r["coverage"] == "nose":
                a, b_ = t["holds_low_flow"], t["holds_high_flow"]
            elif r["coverage"] == "flat":
                a, b_ = t["flat_holds_low_flow"], t["flat_holds_high_flow"]
            else:
                sref[f"not covered: {r['coverage'].split(':', 1)[-1]}"] += 1
                continue
            if a["admitted"] and b_["admitted"]:
                fs[r["case_id"]] = holds(a["probability"], b_["probability"])
            else:
                for x in a["reasons"]:
                    sref[x.split(":")[0].split(" =")[0]] += 1
        ctx["refusals"] = {"quench": dict(ref), "sustain": dict(sref)}
        ctx["n_asked"] = {"quench": len(law), "sustain": len(srows)}
        ctx["coverage"] = {r["case_id"]: r["coverage"] for r in law}
        ctx["sustain_coverage"] = {r["case_id"]: r["coverage"] for r in srows}
        ctx["heat_time_default"] = {"quench": sum(1 for r in law if r["coverage"] in ("nose", "flat")),
                                    "sustain": sum(1 for r in srows if r["coverage"] in ("nose", "flat"))}
        return {"quench": fc, "sustain": fs}

    def baselines(ctx):
        rows = ctx["rows"]
        s = b["quench_bhattacharjee_2016"]["log_width_fitted"]
        if cod:   # v2: the lane's v2 constants (baselines_v2.json); Bhattacharjee keeps its formula and its v1 log width
            # v3: its own constants rebuilt from the v3 train file (bass1_baselines_v3.py), as v2's from the v2 train file
            b2 = json.loads((WORK / f"transfer_bass1/baselines{cod}.json").read_text(encoding="utf-8"))
            bh = {}
            for r in rows:
                if r.get("x_o2_initial") is not None:
                    u = L.v_crit_bhattacharjee(r["x_o2_initial"], r.get("pressure_atm"))
                    if u is not None:
                        bh[r["case_id"]] = L.lognormal_quantiles(u, s)
            return {"constant (simple)": {"quench": {r["case_id"]: b2["quench_constant"]["quantiles"] for r in rows},
                                          "display": {r["case_id"]: b2["quench_display_constant"]["quantiles"] for r in rows}},
                    "Bhattacharjee et al. 2016, v1 train log width": {"quench": bh},
                    "train incidence (simple)": {"sustain": {r["case_id"]: b2["sustain_incidence"]["p"] for r in rows}},
                    "always lit": {"sustain": {r["case_id"]: 1.0 for r in rows}}}
        bh = {}
        for r in rows:
            if r.get("x_o2_initial") is not None:
                v = L.v_crit_bhattacharjee(r["x_o2_initial"], r.get("pressure_atm"))
                if v is not None:
                    bh[r["case_id"]] = L.lognormal_quantiles(v, s)
        return {"constant (simple)": {"quench": {r["case_id"]: b["quench_constant"]["quantiles_cm_s"] for r in rows}},
                "Bhattacharjee et al. 2016, train log width": {"quench": bh},
                "train incidence (simple)": {"sustain": {r["case_id"]: b["sustain_incidence"]["p"] for r in rows}},
                "always lit": {"sustain": {r["case_id"]: 1.0 for r in rows}}}

    def restrict(fc, ref):
        # a baseline's native display quantiles ("display") are restricted to the cases the model answers on quench
        return {t: {k: v for k, v in fc[t].items() if k in ref.get("quench" if t == "display" else t, {})} for t in fc}

    def to_display(ctx, f):
        """A cm/s quantile forecast on the fan-display scale through the case's own fan curve (an input): the curve
        is increasing, so each quantile maps to the quantile of the display (u_to_fan, the lane's inverse)."""
        cur = {r["case_id"]: r.get("fan_curve") for r in ctx["rows"]}
        return {k: [L.u_to_fan(q, cur[k]) for q in qs] for k, qs in f.items() if cur.get(k)}

    def score(ctx, fc):
        if not cod:
            return {t: L.score(fc[t], ctx["outs"], t) for t in ("quench", "sustain") if t in fc}
        import bass1_cases_v2 as V2
        res = {t: V2.score(fc[t], ctx["outs"], t) for t in ("quench", "sustain") if t in fc}
        if "quench" in fc or "display" in fc:
            disp = dict(to_display(ctx, fc.get("quench", {})))
            disp.update(fc.get("display", {}))
            res["quench_display"] = V2.score(disp, ctx["outs"], "quench_display")
        return res

    def study(ctx, fc):
        f, fs = fc.get("quench", {}), fc.get("sustain", {})
        return [dict(case_id=r["case_id"], day=r["day_group"], coverage=ctx.get("coverage", {}).get(r["case_id"]),
                     sustain_coverage=ctx.get("sustain_coverage", {}).get(r["case_id"]),
                     inputs={k: v for k, v in r.items() if k not in ("outcome", "fan_curve")}, forecast=f.get(r["case_id"]),
                     forecast_sustain=fs.get(r["case_id"]),
                     sustain_brier=((fs[r["case_id"]] - r["outcome"]["sustain"]) ** 2 if r["case_id"] in fs and r["outcome"].get("sustain") is not None else None),
                     outcome={k: r["outcome"].get(k) for k in ("quench_class", "quench_u_lo_cm_s", "quench_u_hi_cm_s", "sustain", "n2_applied")},
                     scored=L.quench_scored(r["outcome"]),
                     loss_contribution=dict(interval_crps_cm_s=L.crps_interval(sorted(f[r["case_id"]]), r["outcome"]["quench_u_lo_cm_s"], r["outcome"]["quench_u_hi_cm_s"])
                                            if r["case_id"] in f and L.quench_scored(r["outcome"]) else None))
                for r in ctx["rows"]]
    S.load, S.release, S.baselines, S.restrict, S.score, S.study = load, release, baselines, restrict, score, study
    return S


def slice_slug(slug="slice"):
    import slice_cases as L
    S = Slug()
    S.metrics = {"blow_off": [("Brier, mean of day means", "aggregate_day_mean"), ("pooled", "pooled_mean")],
                 "lifted": [("Brier, mean of day means", "aggregate_day_mean"), ("pooled", "pooled_mean")]}
    cv = "_v2" if SLICE_CASES == "v2" else ""   # case version (docs/wsp/feedback/slice_tube_check.md); v1 registered
    train = jl(WORK / f"transfer_slice/cases_train{cv}.jsonl")
    o2 = [c["inputs"]["x_o2_cabin"] for c in train if c["inputs"].get("x_o2_cabin") is not None]
    O2_MEDIAN = statistics.median(o2)
    S.notes = [f"Cabin oxygen supplied to validation lines without one (PROTOCOL 13.2): the median of the {len(o2)} train lines "
               f"that record it, x_o2 = {O2_MEDIAN:.6f}."]

    def load(part):
        if part == "train":
            rows = train
        else:
            outs = {r["case_id"]: r for r in jl(WORK / f"transfer_slice/cases_validation_outcomes{cv}.jsonl")}
            rows = [dict(r, outcome={k: v for k, v in outs[r["case_id"]].items() if k not in ("case_id", "day")})
                    for r in jl(WORK / f"transfer_slice/cases_validation_inputs{cv}.jsonl")]
        return dict(rows=rows, part=part, out={c["case_id"]: {"day": c["day"], **c["outcome"]} for c in rows})

    def release(ctx, rid):
        rows = ctx["rows"]
        if ctx["part"] != "train":
            filled = [dict(r, inputs=dict(r["inputs"], x_o2_cabin=O2_MEDIAN) if r["inputs"].get("x_o2_cabin") is None else r["inputs"]) for r in rows]
            ctx["o2_supplied"] = sum(1 for r in rows if r["inputs"].get("x_o2_cabin") is None)
        else:
            filled = rows
        if slug == "slice2":
            # phys_slice2.md, scoring-time rule: p_lifted = p_lift_noted * (p_past_lift - p_past_blow) / (1 - p_past_blow)
            res = run_law("slice2", rid, LI.law_rows("slice", filled), "blow_off,past_lift,past_blow,lift_noted", tag=ctx["part"] + "_s2")
            fb = {cid: t["blow_off"]["probability"] for cid, t in res.items() if t["blow_off"]["admitted"]}
            fl = {}
            for cid, t in res.items():
                if t["past_lift"]["admitted"]:
                    pl, pb, pn = t["past_lift"]["probability"], t["past_blow"]["probability"], t["lift_noted"]["probability"]
                    fl[cid] = pn * (pl - pb) / (1.0 - pb) if pb < 1.0 else 0.0
            ctx["refusals"] = {"blow_off": refusals(res, "blow_off"), "lifted": refusals(res, "past_lift")}
        else:
            res = run_law("slice", rid, LI.law_rows("slice", filled), "blow_off,lifted", tag=ctx["part"])
            fb = {cid: t["blow_off"]["probability"] for cid, t in res.items() if t["blow_off"]["admitted"]}
            fl = {cid: t["lifted"]["probability"] for cid, t in res.items() if t["lifted"]["admitted"]}
            ctx["refusals"] = {"blow_off": refusals(res, "blow_off"), "lifted": refusals(res, "lifted")}
        ctx["n_asked"] = {"blow_off": len(res), "lifted": len(res)}
        return {"blow_off": fb, "lifted": fl}

    def baselines(ctx):
        rows = ctx["rows"]
        el = [c for c in train if c["outcome"]["eligible"]]
        pb = statistics.fmean(c["outcome"]["blow_off"] for c in el)
        pl = statistics.fmean(c["outcome"]["lifted"] for c in el)
        g = defaultdict(list)
        for c in el:
            g[c["inputs"]["burner_nominal_mm"]].append(c)
        bb, bl = {}, {}
        for c in rows:
            v = g.get(c["inputs"]["burner_nominal_mm"])
            bb[c["case_id"]] = statistics.fmean(x["outcome"]["blow_off"] for x in v) if v else pb
            bl[c["case_id"]] = statistics.fmean(x["outcome"]["lifted"] for x in v) if v else pl
        return {"incidence pooled (simple)": {"blow_off": {c["case_id"]: pb for c in rows}, "lifted": {c["case_id"]: pl for c in rows}},
                "incidence by nominal tube": {"blow_off": bb, "lifted": bl},
                "never (persistence)": {"blow_off": {c["case_id"]: 0.0 for c in rows}, "lifted": {c["case_id"]: 0.0 for c in rows}}}

    def restrict(fc, ref):
        return {t: {k: v for k, v in fc[t].items() if k in ref[t]} for t in fc}

    def score(ctx, fc):
        return {t: L.score(f, ctx["out"], t) for t, f in fc.items()}

    def study(ctx, fc):
        return [dict(case_id=c["case_id"], day=c["day"], test=c.get("test"), step_in_test=c.get("step_in_test"), inputs=c["inputs"],
                     forecast=dict(blow_off=fc.get("blow_off", {}).get(c["case_id"]), lifted=fc.get("lifted", {}).get(c["case_id"])), outcome=c["outcome"],
                     loss_contribution={t: ((fc[t][c["case_id"]] - c["outcome"][t]) ** 2 if c["case_id"] in fc.get(t, {}) and c["outcome"].get("eligible", True)
                                            and c["outcome"].get(t) is not None else None) for t in ("blow_off", "lifted")})
                for c in ctx["rows"]]
    S.load, S.release, S.baselines, S.restrict, S.score, S.study = load, release, baselines, restrict, score, study
    return S


def cld_slug(slug="cld"):
    import cld_cases as L
    S = Slug()
    S.metrics = {"loss": [("integrated Brier", "aggregate.ibs"), ("Brier of lost before fuel-off", "aggregate.brier_any"),
                          ("integrated Brier, mean of day means", "aggregate_day_mean.ibs")],
                 "radiometer": [("CRPS of log10 V", "aggregate.crps"), ("mean of day means", "aggregate_day_mean.crps")]}
    b = json.loads((WORK / "transfer_cld/baselines_train.json").read_text(encoding="utf-8"))
    LC, RC = b["loss/_constants"], b["radiometer/_constants"]

    def load(part):
        if part == "train":
            rows = jl(WORK / "transfer_cld/cases_train.jsonl")
        else:
            outs = {r["case_id"]: r for r in jl(WORK / "transfer_cld/cases_validation_outcomes.jsonl")}
            rows = [dict(r, **{k: v for k, v in outs[r["case_id"]].items() if k not in r}) for r in jl(WORK / "transfer_cld/cases_validation_inputs.jsonl")]
        return dict(rows=rows, part=part, lo={r["case_id"]: r for r in rows if "loss" in (r.get("targets") or [])},
                    ro={r["case_id"]: r for r in rows if "radiometer" in (r.get("targets") or [])})

    def release(ctx, rid):
        law = LI.law_rows(slug, [{k: v for k, v in r.items() if k not in ("outcome", "targets", "audit")} for r in ctx["rows"]])
        for r in law:   # the forward is asked at the lane's horizons and at the planned fuel-off of the case
            w = r["inputs"].get("window")
            r["times_s"] = sorted(set(L.HORIZONS_S + ([float(w)] if w else [])))
        res = run_law(slug, rid, law, "loss,radiometer", times=L.HORIZONS_S, tag=ctx["part"])
        fl, fr = {}, {}
        for r in law:
            t = res[r["case_id"]]
            s = t["loss"]
            if s["admitted"]:
                end = {h: sum(c[k] for n, c in s["cdf"].items() if n not in NOT_CAUSES) for k, h in enumerate(s["times_s"])}   # every declared cause
                p = [end[float(h)] for h in L.HORIZONS_S]
                p = [max(p[:k + 1]) for k in range(len(p))]   # guard the lane's non-decreasing check against round-off only
                fl[r["case_id"]] = dict(p_loss_by=p, p_loss_before_fuel_off=end[float(r["inputs"]["window"])])
            q = qs(t["radiometer"])
            if q:
                fr[r["case_id"]] = dict(quantiles=[math.log10(v) for v in q])   # log of a quantile = quantile of the log
        ctx["refusals"] = {"loss": refusals(res, "loss"), "radiometer": refusals(res, "radiometer")}
        ctx["n_asked"] = {"loss": len(law), "radiometer": len(law)}
        return {"loss": fl, "radiometer": fr}

    def baselines(ctx):
        rows = ctx["rows"]
        wend = lambda c: dict(p_loss_by=list(c), p_loss_before_fuel_off=c[-1])  # noqa: E731
        fuel = lambda r: r["inputs"]["fuel"]  # noqa: E731
        dil = lambda r: r["inputs"]["x_fuel_final"] < r["inputs"]["x_fuel_initial"]  # noqa: E731
        out = {"incidence (simple)": {"loss": {r["case_id"]: wend(LC["incidence"]) for r in rows},
                                      "radiometer": {r["case_id"]: dict(quantiles=RC["constant"]) for r in rows}},
               "by fuel": {"loss": {r["case_id"]: wend(LC["by_fuel"][fuel(r)]) for r in rows if fuel(r) in LC["by_fuel"]},
                           "radiometer": {r["case_id"]: dict(quantiles=RC["by_fuel"][fuel(r)]) for r in rows if fuel(r) in RC["by_fuel"]}},
               "loss by dilution ramp": {"loss": {r["case_id"]: wend(LC["by_dilution_ramp"][str(dil(r))]) for r in rows
                                                 if r["inputs"].get("x_fuel_final") is not None and r["inputs"].get("x_fuel_initial") is not None}},
               "loss, classical LFL at the stoichiometric surface": {"loss": {r["case_id"]: dict(p_loss_by=[L.classical_lfl(r)] * len(L.HORIZONS_S),
                                                                                                     p_loss_before_fuel_off=L.classical_lfl(r)) for r in rows
                                                                                     if "x_fuel_at_stoichiometric_surface_final" in r["inputs"]}},
               "radiometer, Roper length plus fuel offset": {"radiometer": {
                   r["case_id"]: dict(quantiles=[math.log10(r["inputs"]["roper_length_final_m"]) + d for d in RC["roper_offset"][fuel(r)]])
                   for r in rows if r["inputs"].get("roper_length_final_m") and fuel(r) in RC["roper_offset"]}}}
        return out

    def restrict(fc, ref):
        return {t: {k: v for k, v in fc[t].items() if k in ref[t]} for t in fc}

    def score(ctx, fc):
        res = {}
        if "loss" in fc:
            res["loss"] = L.score(fc["loss"], {k: v for k, v in ctx["lo"].items() if k in fc["loss"]}, "loss")
        if "radiometer" in fc:
            res["radiometer"] = L.score(fc["radiometer"], {k: v for k, v in ctx["ro"].items() if k in fc["radiometer"]}, "radiometer")
        return res

    def study(ctx, fc):
        out = []
        for r in ctx["rows"]:
            o = r.get("outcome") or {}
            lc = {}
            if r["case_id"] in fc.get("loss", {}) and r["case_id"] in ctx["lo"]:
                terms, ba = L.loss_terms(fc["loss"][r["case_id"]], o)
                lc["loss_brier_by_horizon"] = dict(zip([f"{h:g}" for h in L.HORIZONS_S], terms))
                lc["loss_brier_any"] = ba
            if r["case_id"] in fc.get("radiometer", {}) and r["case_id"] in ctx["ro"]:
                lc["radiometer_crps_log10"] = L.crps_quantiles(fc["radiometer"][r["case_id"]]["quantiles"], o["radiometer_log10"])
            out.append(dict(case_id=r["case_id"], day=r["day"], targets=r.get("targets"), inputs={k: v for k, v in r["inputs"].items() if k != "script"},
                            at_cutoff=r.get("at_cutoff"), forecast=dict(loss=fc.get("loss", {}).get(r["case_id"]), radiometer=fc.get("radiometer", {}).get(r["case_id"])),
                            outcome=o, loss_contribution=lc))
        return out
    S.load, S.release, S.baselines, S.restrict, S.score, S.study = load, release, baselines, restrict, score, study
    return S


SLUGS = {"bre": bre_slug, "spice": spice_slug, "efield": efield_slug, "samer": lambda: samer_slug("samer"),
         "same": lambda: samer_slug("same"), "daft": daft_slug, "bass1": bass1_slug, "slice": slice_slug, "cld": cld_slug, "cld2": lambda: cld_slug("cld2"), "efield2": lambda: efield_slug("efield2"), "efield3": lambda: efield_slug("efield3"), "cld3": lambda: cld_slug("cld3"), "bass2": lambda: bass1_slug("bass2"), "bass3": lambda: bass1_slug("bass3"), "bass4": lambda: bass1_slug("bass4"), "bass5": lambda: bass1_slug("bass5"), "slice2": lambda: slice_slug("slice2")}


def aggregates_only(v, partition):
    """Drop per-case entries (lists of case ids) from a lane result; keep aggregates and per-day means."""
    if isinstance(v, dict):
        return {k: aggregates_only(x, partition) for k, x in v.items() if k not in ("missing", "missing_forecasts")}
    return v


def n_missing(r):
    for k in ("missing", "missing_forecasts"):
        if k in r:
            return len(r[k]) if isinstance(r[k], (list, dict)) else r[k]
    return r.get("refused")


def cmd_look(args):
    if args.partition != "train" and not args.look:
        sys.exit("a validation or held partition is scored only in a look registered in docs/wsp/PROTOCOL.md: pass --look <section>")
    S = SLUGS[args.slug]()
    ctx = S.load(args.partition)
    fc = S.release(ctx, args.release)
    rel = S.score(ctx, fc)
    bl = S.baselines(ctx)
    bl_all = {n: S.score(ctx, f) for n, f in bl.items()}
    bl_ans = {n: S.score(ctx, S.restrict(f, fc)) for n, f in bl.items()}
    lines = [f"## {args.slug}: release `{args.release}`, partition {args.partition}" + (f", look {args.look}" if args.look else ""), "",
             "Written by `tools/wsp/ext/law_score.py look`; every score is the lane's own `score()`. Lower is better.", ""]
    lines += [f"- {n}" for n in getattr(S, "notes", [])]
    if ctx.get("heat_time_default") is not None:
        lines.append(f"- heat_time = 200 s (no planned igniter on-time among the inputs) for every covered case: {json.dumps(ctx['heat_time_default'])}.")
    if ctx.get("o2_supplied") is not None:
        lines.append(f"- Lines given the supplied oxygen: {ctx['o2_supplied']}.")
    readings = {}
    for t, mets in S.metrics.items():
        if t not in rel:
            continue
        answered = len(fc[t]) if not isinstance(fc[t], list) else len(fc[t])
        if t == "t1" and args.slug.startswith("efield"):
            answered = sum(len(v) for v in fc[t].values())
        lines += ["", f"### target {t}", "",
                  f"- Asked of the release: {ctx['n_asked'][t]}; answered: {answered}; refused: {ctx['n_asked'][t] - answered}; "
                  f"refusal reasons: {json.dumps(ctx['refusals'].get(t, {}))}",
                  f"- Lane cases without a release forecast (not scored): {n_missing(rel[t])}", "",
                  "| source | " + " | ".join(m[0] for m in mets) + " | on |", "|---" * (len(mets) + 2) + "|",
                  "| **release** | " + " | ".join(f"{get(rel[t], m[1]):.4f}" if isinstance(get(rel[t], m[1]), (int, float)) else "-" for m in mets) + " | cases the release answers |"]
        cmp = {}
        for n in bl:
            if t not in bl[n]:
                continue
            for tag, r in (("cases the release answers", bl_ans[n].get(t)), ("all lane cases", bl_all[n].get(t))):
                if r is None:
                    continue
                lines.append(f"| {n} | " + " | ".join(f"{get(r, m[1]):.4f}" if isinstance(get(r, m[1]), (int, float)) else "-" for m in mets) + f" | {tag} |")
            cmp[n] = bl_ans[n].get(t)
        # the reading of PROTOCOL 13.2 on the primary metric, on the cases the release answers
        for m in mets[:2] if t in ("t1",) and args.slug == "bre" else mets[:1]:
            rv = get(rel[t], m[1])
            simple = [n for n in cmp if "simple" in n and cmp[n] is not None][0]
            sv = get(cmp[simple], m[1])
            others = {n: get(r, m[1]) for n, r in cmp.items() if isinstance(get(r, m[1]), (int, float))}
            best_n = min(others, key=others.get)
            if not isinstance(rv, (int, float)) or rv != rv:
                reading = "not scored"
            elif rv > sv:
                reading = "loses (higher than the simple baseline)"
            elif all(rv < v for v in others.values()):
                reading = "beats the best baseline"
            elif rv < sv:
                reading = "beats the simple baseline"
            else:
                reading = "ties the simple baseline"
            readings[f"{t}: {m[0]}"] = dict(release=rv, simple=(simple, sv), best=(best_n, others[best_n]), reading=reading)
            lines.append("")
            lines.append(f"**Reading ({m[0]}):** release {rv:.4f}; simple baseline ({simple}) {sv:.4f}; best baseline ({best_n}) "
                         f"{others[best_n]:.4f}: **{reading}**.")
        lines += ["", "<details><summary>lane results (aggregates and per-day means)</summary>", "", "```json",
                  json.dumps({"release": aggregates_only(rel[t], args.partition),
                              "baselines_on_cases_answered": {n: aggregates_only(bl_ans[n][t], args.partition) for n in bl if t in bl_ans[n]}},
                             indent=1, default=lambda x: None), "```", "", "</details>"]
    md = args.result_out or f"docs/wsp/feedback/{args.slug}_w0_result.md"
    p = REPO / md
    p.parent.mkdir(parents=True, exist_ok=True)
    old = p.read_text(encoding="utf-8") if p.exists() else f"# {args.slug}: seventh-generation W0 law, scores beside the baselines\n\n"
    p.write_text(old + "\n".join(lines) + "\n\n", encoding="utf-8", newline="\n")
    summary = dict(slug=args.slug, release=args.release, partition=args.partition, readings=readings,
                   asked=ctx["n_asked"], answered={t: (sum(len(v) for v in fc[t].values()) if args.slug.startswith("efield") and t == "t1" else len(fc[t])) for t in fc},
                   refusals=ctx["refusals"])
    sp = REPO / "runs/dev/gen7/look_summaries.jsonl"
    with sp.open("a", encoding="utf-8", newline="\n") as f:
        f.write(json.dumps(summary, default=lambda x: None) + "\n")
    if (args.partition == "train" or getattr(args, "study_dev", False)) and S.study:
        tag = args.release.replace("sha256:", "")[:16] + ("" if args.partition == "train" else "_dev_" + args.partition)
        stp = WORK / f"transfer_{args.slug}/law_study_{tag}.jsonl"
        stp.write_text("".join(json.dumps(r, default=lambda x: None) + "\n" for r in S.study(ctx, fc)), encoding="utf-8", newline="\n")
        for n, f in bl.items():   # the baselines' per-case files beside, for the anatomy
            sf = WORK / f"transfer_{args.slug}/law_study_baseline_{n.split(' (')[0].split(',')[0].replace(' ', '_').replace('/', '')[:40]}{'' if args.partition == 'train' else '_dev_' + args.partition}.jsonl"
            sf.write_text("".join(json.dumps(r, default=lambda x: None) + "\n" for r in S.study(ctx, f)), encoding="utf-8", newline="\n")
        print("study file:", stp)
    print(json.dumps(summary, indent=1, default=lambda x: None))
    print("result ->", p)
    return 0


def cmd_score(args):
    """One baseline alone on TRAIN (the Phase 1 test of the scoring path)."""
    if args.partition != "train":
        sys.exit("`score` runs a baseline on train only; use `look` for a registered look")
    S = SLUGS[args.slug]()
    ctx = S.load("train")
    bl = S.baselines(ctx)
    name = next((n for n in bl if n.startswith(args.baseline)), None)
    if name is None:
        sys.exit(f"baselines of {args.slug}: {list(bl)}")
    res = S.score(ctx, bl[name])
    p = REPO / (args.result_out or f"runs/dev/gen7/scorer_test/{args.slug}_train_baselines.md")
    p.parent.mkdir(parents=True, exist_ok=True)
    old = p.read_text(encoding="utf-8") if p.exists() else ""
    p.write_text(old + f"## {args.slug}: baseline {name}, partition train\n\n```json\n{json.dumps(aggregates_only(res, 'train'), indent=1, default=lambda x: None)}\n```\n\n",
                 encoding="utf-8", newline="\n")
    print("aggregates ->", p)
    return 0


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    sub = ap.add_subparsers(dest="cmd", required=True)
    s = sub.add_parser("look")
    s.add_argument("--slug", required=True, choices=list(SLUGS))
    s.add_argument("--release", required=True)
    s.add_argument("--partition", default="train", choices=["train", "validation", "held"])
    s.add_argument("--look")
    s.add_argument("--result-out")
    s.add_argument("--study-dev", action="store_true", help="PROTOCOL 13.4: validation is study data; write its per-case file (development, in-sample)")
    s.set_defaults(fn=cmd_look)
    s = sub.add_parser("score")
    s.add_argument("--slug", required=True, choices=list(SLUGS))
    s.add_argument("--baseline", required=True)
    s.add_argument("--partition", default="train")
    s.add_argument("--result-out")
    s.set_defaults(fn=cmd_score)
    a = ap.parse_args()
    return a.fn(a)


if __name__ == "__main__":
    sys.exit(main())
