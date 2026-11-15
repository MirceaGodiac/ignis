#!/usr/bin/env python3
"""Score the frozen ACME fresh batch (docs/wsp/atlas/splits/ACME-batch-fresh.json): the 26
Flame Design (PSI-10) tests of tiers A1, A2, A3. Tier B1 is not scored. Post-hoc, 15 November 2026; nothing is
adopted, no release, forward, threshold or chain function is changed.

Strict order (each step refuses to run before the previous one has written its file; outputs in runs/fresh/acme/):

  inputs       INPUT SIDE ONLY, from pre-ignition information: the batch's conditions (experiment-table pressure
               and O2, the sibling's flows as the batch prescribes), the script parameter files (old stage sum of
               flame_design.py and the new planned stop of tools/wsp/ext/acme_script2.py). Writes
               input/conditions.csv, input/stops.csv, input/inputs.json and every prefix (all three cutoffs, every
               role, both paths) under prefix/. Opens no sensor channel, setpoint log, TP file or endpoint.
  equivalence  the binary: the final run's ignis.exe (b03a35c8..., RESULTS_FINAL.md) is no longer on disk
               (target-merged/release/ignis.exe was rebuilt at 11:01 on 15 Nov, sha 7a409b32...). This step
               re-runs, with the binary now at run_final.IGNIS_REL, (a) the CFI-G train chain check of
               transfer_f2.py (registered F2 numbers and wsp-dev numbers) and (b) every F2/F1/surrogate forward
               of the Flame Design VALIDATION days that transfer_f2.py ran with b03a35c8, and compares the
               artifacts (status, support, event_cdf, times). Forecasts are made only if both reproduce exactly.
  forecast     forwards: F2 (acme_normal.wsp12 / acme_inverse.wsp12), F1, ethylene as ethane (surrogate), as named
               in tools/wsp/final/releases_final_20261115.json; the incidence baseline of 11.1; the Irace et al.
               comparator of posthoc_irace_testdays.irace_forecast (R + erfc clock, and R only). Writes
               forecasts.json. Still no endpoint.
  measure      flame_design.measure on the 26 tests, thresholds unchanged -> endpoints.csv (THIS HOLDS OUTCOMES).
  score        transfer_acme.score exactly as posthoc_irace_testdays.py: cases at risk at cutoffs 1, 3, 5 s,
               cases every Flame Design release answered, integrated Brier for extinction / operator termination /
               sum, case means (and session-equal means), F2 minus each comparator with the session bootstrap
               (ta.boot, test day = session). Per tier and pooled. -> aggregates.json, cases.jsonl.

Code choices, written BEFORE any forecast was made or any endpoint measured:

  C1  Conditions = the batch's `conditions` block verbatim (pressure, mass flow, x_fuel_role, x_o2_role, Tad), the
      fuel species and the row of conditions.csv of the final run (byte-identical to fd_fuel/flame_design/
      conditions.csv, sha 28485d7b...). For A1 that is the table's own pressure and O2 plus the sibling's mass flow,
      fuel fraction and Tad (the mean over siblings where the batch lists several); Tad is not corrected for the O2
      difference (batch caveat). For A3 it is the table pressure plus the sibling's flows, burner O2 and AMBIENT
      fuel fraction: the batch prescribes the sibling's x_fuel_role although the table has the test's own XC2H4
      initial (22024B2 0.186136 vs 0.167319; 22027B4 0.098757 and 22027B8 0.097500 vs 0.102081). The batch is
      followed; the difference is recorded in inputs.json. A2 rows are the table's own values (identical to the
      batch). zst_initial is recomputed with flame_design.zst_initial from these values, as condition_row does;
      every value is written to input/conditions.csv with the rounding of flame_design.condition_row (pressure 1
      decimal, mass flow %.6e, mole fractions and zst 6 decimals, Tad 1 decimal) and read back from there, so the
      prefixes are built by transfer_acme.fd_prefix from a conditions file exactly as in the chain. The domain
      features of flame_design.SCALES are recomputed and recorded (the forward is the authority on refusals).
  C2  Prefixes: transfer_gen6.prefix (= transfer_acme.fd_prefix + locator paths), the role loop of
      transfer_f2.build: F2 and F1 read ethylene as itself, the surrogate ethane's indicators for an ethylene test
      (a propane test as recorded); no planned stop on the `nostop` path. The prefix carries no observation, no
      onset value and no endpoint; its time origin is the flame onset that flame_design.py measures (the same
      text as every release reads). Prefixes are written for all three cutoffs before any endpoint exists; a
      case is kept at scoring when ta.at_risk(endpoint, cutoff) holds, as in transfer_f2.build.
  C3  Planned stop on the `stop` path, as transfer_f2.build: F2 reads the new stop (acme_script2), F1 and the
      surrogate the old stage sum (flame_design.measure's `script_programmed_stop_s` rule: the sum of STAGE_KEYS
      when the file has a constant-flow stage, read here from the script file alone by flame_design.read_script).
      The chain's programmed_stop locator names endpoints.csv; here it names input/stops.csv, which holds only
      these script-derived values (a locator, not a value the forward reads).
      A1 and A2 are single-stage: acme_script2's planned stop equals its planned first flow change (the start of
      the scripted ramp to zero on A1) and is used as is.
      A3 is multi-stage. The batch conditions are the STAGE-1 flows, and flame_design.py ends a test at the first
      commanded departure from the test flow, so the stop that ends the scored flame is the end of stage 1.
      PRIMARY for A3: F2 reads acme_script2's planned_first_flow_change_s; F1 and the surrogate the stage sum
      truncated after constantFlowTime1Sec (stabilization 1 + 2 + ignition-to-test ramp + constant stage 1, the
      same terms as the old sum). SECONDARY for A3 (path `stop_chain`, reported apart): the chain verbatim
      (planned_stop_s, the end of the last constant stage, and the full stage sum).
  C4  A2 has no `_TP.txt`. flame_design.measure already runs without it (file_names gives tp=None; the ignition
      reference falls back to "sustained flow on + 1 s", flag `no_tp`, the same fallback as s-Flame); only
      condition_row's inclusion flag required the file. measure() is called unchanged; nothing else is changed.
  C5  Endpoint -> outcome exactly as run_final.fd_test_day_tests: status ok and label extinction /
      operator_termination -> event at end_s; burning_at_file_end -> right-censored at end_s; anything else has no
      endpoint (not at risk, recorded with its reason).
  C6  Scoring: per tier (A1, A2, A3), A1+A2 (normal family), and all 26 pooled (one IPCW case set; the chain never
      pools families, so the all-tier pool is reported as such). Cases = those every Flame Design release
      (F2, F1, surrogate) answered; refusals counted apart. Comparators on the same cases: F1, surrogate,
      incidence baseline, Irace (R + erfc clock), Irace (R only). The Irace law is posthoc_irace_testdays'
      irace_forecast unchanged (ethylene molar mass in Zst and Y; applied to the one propane test, 21301C2, as is).
  C7  A1 note: how many A1 labels are operator termination at the scripted ramp: label operator_termination with
      the measured flow change within 2 s of acme_script2's planned first flow change.

    python tools/wsp/fresh/score_acme_fresh.py inputs
    python tools/wsp/fresh/score_acme_fresh.py equivalence
    python tools/wsp/fresh/score_acme_fresh.py forecast
    python tools/wsp/fresh/score_acme_fresh.py measure
    python tools/wsp/fresh/score_acme_fresh.py score
"""
import argparse
import csv
import hashlib
import json
import math
import sys
from collections import Counter
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[2]
sys.path.insert(0, str(REPO / "tools/wsp/ext"))
sys.path.insert(0, str(REPO / "tools/wsp/analysis"))
sys.path.insert(0, str(REPO / "tools/wsp/final"))
import transfer_f2 as tf2  # noqa: E402  (sets the fd_fuel layout of transfer_acme, transfer_gen6, flame_design)
import acme_script2  # noqa: E402

ta, g6, tf, fdm = tf2.ta, tf2.g6, tf2.tf, tf2.tf.fdm

BATCH = REPO / "docs/wsp/atlas/splits/ACME-batch-fresh.json"
COND_SRC = REPO / "runs/final/20261115/local/final/flame-design/flame_design/conditions.csv"
RELEASES = REPO / "tools/wsp/final/releases_final_20261115.json"
IGNIS_REL = "target-merged/release/ignis.exe" if sys.platform == "win32" else "target-merged/release/ignis"
FINAL_IGNIS_SHA = "b03a35c820d08347afb15fa232e6d10deef7d288463b574a415a918df4248ffb"
import os
OUT = REPO / os.environ.get("ACME_FRESH_OUT", "runs/fresh/acme")
INPUT = OUT / "input"
TIERS = ("A1", "A2", "A3")
ROLES = ("f2", "f1", "surrogate")
METRICS = ("extinction", "operator_termination", "sum")
COMPARATORS = ("f1", "surrogate", "incidence_baseline", "irace", "irace_R")
PATHS = ("nostop", "stop", "stop_chain")      # stop_chain: A3 only (C3)
COLS = ["test", "family", "day", "session", "fuel_species", "pressure_pa", "injected_mass_flow_kg_s", "x_o2_role",
        "x_fuel_role", "tad_k_table", "zst_initial", "tier", "conditions_source", "source_row"]

g6.FD_WORK = INPUT          # prefix locators name the files of this run


def sha(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest()


def rel(p):
    return Path(p).resolve().relative_to(REPO).as_posix()


def need(path, step):
    if not path.exists():
        sys.exit(f"{rel(path)} missing: run `{step}` first")


def batch_tests():
    b = json.loads(BATCH.read_text(encoding="utf-8"))
    assert b["status"].startswith("FROZEN"), b["status"]
    tests = [t for t in b["tests"] if t["tier"] in TIERS]
    assert Counter(t["tier"] for t in tests) == Counter(A1=20, A2=3, A3=3)
    return tests


def roles_of():
    fams = json.loads(RELEASES.read_text(encoding="utf-8"))["families"]
    out = {}
    for fam in ("acme_normal", "acme_inverse"):
        f = fams[f"flame_design_{fam.split('_')[1]}"]
        out[fam] = dict(f2=f["release"], f1=f["baselines"]["F1"], surrogate=f["baselines"]["ethylene as ethane"])
        assert out[fam] == tf2.ROLES[fam], fam
    return out


# ------------------------------------------------------------------------------------------------ step 1: inputs
def old_stage_sum(test, upto_stage1=False):
    """flame_design.measure's script_programmed_stop_s from the script file alone (C3); A3 primary: up to stage 1."""
    fn = fdm.file_names(test, fdm.listing())
    if not fn["script"] or not (fdm.FILES / fn["script"]).exists():
        return math.nan, fn["script"]
    s = fdm.read_script(fdm.FILES / fn["script"])
    keys = fdm.STAGE_KEYS
    if upto_stage1:
        keys = keys[: keys.index("constantFlowTime1Sec") + 1]
    has_c = any(k in s for k in ("constantFlowTimeSec", "constantFlowTime1Sec"))
    return (sum(fdm.num(s.get(k, 0)) for k in keys) if has_c else math.nan), fn["script"]


def cmd_inputs(args):
    ta.selftest()
    g6.check_days()
    if (OUT / "forecasts.json").exists() or (OUT / "endpoints.csv").exists():
        sys.exit("forecasts or endpoints already exist: the input side is frozen")
    tests = batch_tests()
    with COND_SRC.open(encoding="utf-8", newline="") as f:
        table = {r["test"]: dict(r, row=i + 2) for i, r in enumerate(csv.DictReader(f))}
    rows, stops, info = [], [], {}
    for t in tests:
        tid, fam, c = t["test_id"], t["family"], t["conditions"]
        tr = table[tid]
        assert tr["family"] == fam and tr["session"] == tid[:5]
        species = tr["fuel_species"]
        zst = fdm.zst_initial(species, c["x_fuel_role"], c["x_o2_role"])
        row = dict(test=tid, family=fam, day=tr["day"], session=tid[:5], fuel_species=species,
                   pressure_pa=fdm.fmt(float(c["pressure_pa"]), 1), injected_mass_flow_kg_s=f"{c['mass_flow_kg_s']:.6e}",
                   x_o2_role=fdm.fmt(float(c["x_o2_role"]), 6), x_fuel_role=fdm.fmt(float(c["x_fuel_role"]), 6),
                   tad_k_table=fdm.fmt(float(c["tad_k"]), 1), zst_initial=fdm.fmt(zst, 6), tier=t["tier"],
                   conditions_source=c["source"], source_row=tr["row"])
        rows.append(row)
        # domain features as flame_design.condition_row, from the rounded values
        sc = fdm.SCALES[fam]
        p, m, xo, xf, tad, z = (float(row[k]) for k in ("pressure_pa", "injected_mass_flow_kg_s", "x_o2_role",
                                                        "x_fuel_role", "tad_k_table", "zst_initial"))
        feats = dict(c_logp=math.log(p / fdm.P_REF_PA), c_logm=math.log(m / sc["mdot_ref"]),
                     c_o2=(xo - sc["o2_ref"]) / sc["o2_scale"], c_xf=math.log(xf / sc["xf_ref"]),
                     c_tad=(tad - fdm.TAD_REF_K) / fdm.TAD_SCALE_K, c_zst=(z - sc["zst_ref"]) / fdm.ZST_SCALE)
        outside = [k for k, (lo, hi) in sc["domain"].items() if not lo <= feats[k] <= hi]
        a3 = t["tier"] == "A3"
        s2 = acme_script2.for_test("PSI-10", tid)
        old_full, script_name = old_stage_sum(tid)
        old_s1, _ = old_stage_sum(tid, upto_stage1=True)
        new_full = s2.get("planned_stop_s")
        new_first = s2.get("planned_first_flow_change_s")
        st = dict(test=tid, script_file=script_name, script_sha256=s2.get("sha256", ""), branch=s2.get("branch"),
                  old_stage_sum_s=old_full, old_stage_sum_stage1_s=old_s1, new_planned_stop_s=new_full,
                  new_planned_first_flow_change_s=new_first, planned_flow_zero_s=s2.get("planned_flow_zero_s"),
                  flags="|".join(s2.get("flags", [])))
        # what each role reads on each path (C3)
        st["stop_f2"] = new_first if a3 else new_full
        st["stop_old"] = old_s1 if a3 else old_full
        st["stop_chain_f2"], st["stop_chain_old"] = new_full, old_full
        if not a3:
            assert new_first == new_full, (tid, new_first, new_full)
        # the batch froze the v2 reading; reader v3 (acme_script2 METHOD_VERSION 3) differs only on 19123L3-L8
        assert abs(new_first - t["script_planned_first_flow_change_s"]) < 1e-9 or (
            acme_script2.METHOD_VERSION == "3" and s2.get("branch") == "v1|fiber_move_no_traverse"), tid
        stops.append(st)
        info[tid] = dict(tier=t["tier"], family=fam, session=tid[:5], fuel=species, features={k: round(v, 4) for k, v in feats.items()},
                         outside_domain=outside, batch_outside=c["outside_f2_domain"], conditions_source=c["source"],
                         table_row_values={k: tr[k] for k in ("injected_mass_flow_kg_s", "x_o2_role", "x_fuel_role", "tad_k_table", "zst_initial")},
                         stops={k: st[k] for k in ("branch", "old_stage_sum_s", "old_stage_sum_stage1_s", "new_planned_stop_s",
                                                   "new_planned_first_flow_change_s", "planned_flow_zero_s", "flags")})
    INPUT.mkdir(parents=True, exist_ok=True)
    for name, data in (("conditions.csv", rows), ("stops.csv", stops)):
        with (INPUT / name).open("w", encoding="utf-8", newline="") as f:
            w = csv.DictWriter(f, fieldnames=list(data[0].keys()), lineterminator="\n")
            w.writeheader()
            w.writerows([{k: ((repr(v) if v == v else "") if isinstance(v, float) else v) for k, v in r.items()} for r in data])
    sha_c, sha_s = sha(INPUT / "conditions.csv"), sha(INPUT / "stops.csv")
    # prefixes, read back from the written files (C1, C2, C3)
    with (INPUT / "conditions.csv").open(encoding="utf-8", newline="") as f:
        cond = [dict(r, row=i + 2) for i, r in enumerate(csv.DictReader(f))]
    with (INPUT / "stops.csv").open(encoding="utf-8", newline="") as f:
        stp = {r["test"]: dict(r, erow=i + 2) for i, r in enumerate(csv.DictReader(f))}
    num = lambda s: float(s) if s not in ("", None) else math.nan  # noqa: E731
    n = 0
    manifest = {}
    for r in cond:
        tid, fam = r["test"], r["family"]
        sr = stp[tid]
        paths = ["nostop", "stop"] + (["stop_chain"] if r["tier"] == "A3" else [])
        for path in paths:
            stop = path != "nostop"
            sf2 = num(sr["stop_chain_f2" if path == "stop_chain" else "stop_f2"])
            sold = num(sr["stop_chain_old" if path == "stop_chain" else "stop_old"])
            col_f2 = "stop_chain_f2" if path == "stop_chain" else "stop_f2"
            col_old = "stop_chain_old" if path == "stop_chain" else "stop_old"
            for c in ta.CUTOFFS:
                for role in ROLES:
                    fuel = "ethane" if (role == "surrogate" and r["fuel_species"] == "ethylene") else "recorded"
                    t = dict(r, part=ta.fd_part(fam, tid), erow=sr["erow"],
                             programmed_stop=(sf2 if role == "f2" else sold) if stop else math.nan)
                    p = g6.prefix(t, c, fuel, stop, sha_c, sha_s)
                    if "programmed_stop" in p["known_conditions"]:
                        m = p["known_conditions"]["programmed_stop"]
                        m["locator"].update(path=rel(INPUT / "stops.csv"), column=col_f2 if role == "f2" else col_old)
                        if role == "f2":
                            m["derivation"].update(method_id=acme_script2.METHOD_ID, method_version=acme_script2.METHOD_VERSION,
                                                   notes="planned flow stop on the forecast clock from the script parameter file "
                                                         "(tools/wsp/ext/acme_script2.py; the stage sum where the file is incomplete)"
                                                         + ("; end of constant stage 1 (planned_first_flow_change_s), "
                                                            "score_acme_fresh.py C3" if (r["tier"] == "A3" and path == "stop") else ""))
                        elif r["tier"] == "A3" and path == "stop":
                            m["derivation"]["notes"] += "; truncated after constantFlowTime1Sec (score_acme_fresh.py C3)"
                        p = ta.finish_prefix(p)
                    if role in ("f2", "f1"):
                        assert "fuel_is_ethane" not in p["known_conditions"]
                    assert not p["observations"] and not p["known_controls"]
                    name = f"{tid}_c{c:g}_{role}_{path}.json"
                    ta.write_json(OUT / "prefix" / fam / name, p)
                    manifest[f"{fam}/{name}"] = dict(input_hash=p["input_hash"], carries_stop="programmed_stop" in p["known_conditions"])
                    n += 1
    rep = dict(what="input side of the ACME fresh batch, written before any forecast or endpoint",
               batch=rel(BATCH), batch_sha256=sha(BATCH), conditions_source=rel(COND_SRC), conditions_source_sha256=sha(COND_SRC),
               conditions_csv_sha256=sha_c, stops_csv_sha256=sha_s, acme_script2_method=[acme_script2.METHOD_ID, acme_script2.METHOD_VERSION],
               prefixes=n, tests=info, prefix_hashes=manifest)
    ta.write_json(INPUT / "inputs.json", rep)
    print(json.dumps(dict(wrote=rel(INPUT), prefixes=n, tests=len(info),
                          outside_domain={k: v["outside_domain"] for k, v in info.items() if v["outside_domain"]}), indent=1))
    return 0


# ------------------------------------------------------------------------------------------------ step 2: binary
def cmd_equivalence(args):
    need(INPUT / "inputs.json", "inputs")
    ta.selftest()
    binary = ta.use_binary(argparse.Namespace(ignis=IGNIS_REL))
    eq = OUT / "equivalence"
    rep = dict(final_run_ignis_sha256=FINAL_IGNIS_SHA, ignis=binary,
               final_binary_on_disk=binary["sha256"] == FINAL_IGNIS_SHA, cfig_chain={}, validation_forwards={})
    ok = True
    # (a) the CFI-G train chain check of transfer_f2.py, into this folder
    chain_ref = json.loads((tf2.WORK / "cfig/chain_check.json").read_text(encoding="utf-8"))
    assert chain_ref["reproduced"] and chain_ref["ignis"]["sha256"] == FINAL_IGNIS_SHA
    for fam, roles in tf2.ROLES.items():
        cases, nar, runs, _ = ta.cfig_cases(fam)
        jobs = []
        for k in cases:
            name = f"{k['test']}_c{k['cutoff']:g}.json"
            pp = eq / "cfig/prefix" / fam / name
            ta.write_json(pp, k["prefix"])
            for role in ("f2", "f1"):
                jobs.append(((k["run_id"], k["cutoff"], role), roles[role], pp, eq / "cfig/artifact" / fam / role / name))
        arts = ta.forward_many(jobs, args.workers)
        got = {}
        for role in ("f2", "f1"):
            a = [arts[(k["run_id"], k["cutoff"], role)] for k in cases]
            assert all(ta.admitted(x) for x in a)
            s = ta.score([k["outcome"] for k in cases], {c: [ta.cif_of(x)[c] for x in a] for c in ta.CAUSES})
            got[role] = {c: s[c]["integrated_brier"] for c in ta.CAUSES}
        ref = chain_ref["families"][fam]["releases"]
        diff = max(abs(got[r][c] - ref[r][c]) for r in got for c in ta.CAUSES)
        rep["cfig_chain"][fam] = dict(cases=len(cases), releases=got, final_run_chain=ref, max_abs_diff=diff)
        ok &= diff == 0.0 and len(cases) == tf2.REGISTERED[fam][2]
    # (b) every F2/F1/surrogate forward of the Flame Design validation days (made with b03a35c8)
    report = json.loads((tf2.WORK / "report.json").read_text(encoding="utf-8"))
    assert report["ignis"]["sha256"] == FINAL_IGNIS_SHA
    keys = ("status", "support", "times_s", "event_categories", "event_cdf", "kind", "model_release_id", "prefix_hash", "run_id")
    for fam in ("acme_normal", "acme_inverse"):
        jobs, refs = [], {}
        for role in ROLES:
            for ap in sorted((tf2.WORK / "artifact" / fam / role).glob("*.json")):
                a = json.loads(ap.read_text(encoding="utf-8"))
                pp = tf2.WORK / "prefix" / fam / ap.name
                if not pp.exists():
                    continue
                refs[(role, ap.name)] = a
                jobs.append(((role, ap.name), a["model_release_id"], pp, eq / "validation/artifact" / fam / role / ap.name))
        arts = ta.forward_many(jobs, args.workers)
        bad = [k for k, a in arts.items() if any(a.get(x) != refs[k].get(x) for x in keys)]
        rep["validation_forwards"][fam] = dict(artifacts=len(jobs), identical=len(jobs) - len(bad), different=[f"{r}/{n}" for r, n in bad[:20]])
        ok &= not bad and len(jobs) > 0
    rep["equivalent"] = bool(ok)
    ta.write_json(OUT / "equivalence.json", rep)
    print(json.dumps({k: v for k, v in rep.items()}, indent=1, default=str)[:4000])
    return 0 if ok else 2


# ------------------------------------------------------------------------------------------------ step 3: forecasts
def load_inputs():
    with (INPUT / "conditions.csv").open(encoding="utf-8", newline="") as f:
        return [r for r in csv.DictReader(f)]


def cmd_forecast(args):
    need(INPUT / "inputs.json", "inputs")
    need(OUT / "equivalence.json", "equivalence")
    if (OUT / "endpoints.csv").exists():
        sys.exit("endpoints already measured: forecasts must precede them")
    eqv = json.loads((OUT / "equivalence.json").read_text(encoding="utf-8"))
    binary = ta.use_binary(argparse.Namespace(ignis=IGNIS_REL))
    if not (eqv["equivalent"] and eqv["ignis"]["sha256"] == binary["sha256"]):
        sys.exit("the binary is not shown equivalent to the final run's: no forecast")
    import posthoc_irace_testdays as PIT
    roles = roles_of()
    inputs = json.loads((INPUT / "inputs.json").read_text(encoding="utf-8"))
    jobs, meta = [], []
    for r in load_inputs():
        tid, fam = r["test"], r["family"]
        for path in PATHS:
            if path == "stop_chain" and r["tier"] != "A3":
                continue
            for c in ta.CUTOFFS:
                for role in ROLES:
                    name = f"{tid}_c{c:g}_{role}_{path}.json"
                    pp = OUT / "prefix" / fam / name
                    p = json.loads(pp.read_text(encoding="utf-8"))
                    assert p["input_hash"] == inputs["prefix_hashes"][f"{fam}/{name}"]["input_hash"]
                    jobs.append(((tid, path, c, role), roles[fam][role], pp, OUT / "artifact" / fam / role / name))
                meta.append((tid, fam, r["tier"], path, c))
    arts = ta.forward_many(jobs, args.workers)
    baseline = {fam: v["cutoffs"] for fam, v in json.loads(ta.BASELINE.read_text(encoding="utf-8"))["families"].items()}
    out = []
    for tid, fam, tier, path, c in meta:
        a = {role: arts[(tid, path, c, role)] for role in ROLES}
        pre = json.loads((OUT / "prefix" / fam / f"{tid}_c{c:g}_f2_{path}.json").read_text(encoding="utf-8"))
        st = None
        if path != "nostop" and "programmed_stop" in pre["known_conditions"]:
            v = PIT.kc_value(pre["known_conditions"], "programmed_stop")
            st = v if v and v > 0 else None
        fi, q = PIT.irace_forecast(fam, pre, ta.HORIZONS, st, "erfc")
        fr, _ = PIT.irace_forecast(fam, pre, ta.HORIZONS, st, "R_only")
        out.append(dict(test=tid, family=fam, tier=tier, session=tid[:5], path=path, cutoff=c,
                        artifact={role: dict(id=x["id"], release=x["model_release_id"], admitted=ta.admitted(x),
                                             refusal="; ".join(x["support"].get("reasons", [])) if not ta.admitted(x) else "",
                                             cif=ta.cif_of(x) if ta.admitted(x) else None) for role, x in a.items()},
                        incidence_baseline={cz: baseline[fam][f"{c:g}"][cz] for cz in ta.CAUSES},
                        irace=fi, irace_R=fr, irace_q=q, stop_read_by_f2=st,
                        carries_stop={role: "programmed_stop" in json.loads((OUT / "prefix" / fam / f"{tid}_c{c:g}_{role}_{path}.json")
                                                                            .read_text(encoding="utf-8"))["known_conditions"] for role in ROLES}))
    rep = dict(what="forecasts of the ACME fresh batch, made before any endpoint was measured", ignis=binary, releases=roles,
               baseline_file=rel(ta.BASELINE), baseline_sha256=sha(ta.BASELINE), horizons_s=ta.HORIZONS, forecasts=out)
    ta.write_json(OUT / "forecasts.json", rep)
    ref = Counter((k["path"], role) for k in out for role in ROLES if not k["artifact"][role]["admitted"])
    print(json.dumps(dict(wrote=rel(OUT / "forecasts.json"), forecasts=len(out), refused=dict((f"{p}/{r}", n) for (p, r), n in ref.items())), indent=1))
    return 0


# ------------------------------------------------------------------------------------------------ step 4: measure
def cmd_measure(_args):
    need(OUT / "forecasts.json", "forecast")
    path = OUT / "endpoints.csv"
    if path.exists():
        sys.exit(f"{rel(path)} exists: measured once")
    names = fdm.listing()
    rows = []
    for r in load_inputs():       # flame_design.measure, thresholds unchanged (C4: tp=None where no _TP.txt)
        try:
            row = fdm.measure(r["test"], fdm.file_names(r["test"], names))
        except Exception as e:  # noqa: BLE001
            row = dict(test=r["test"], status=f"error:{type(e).__name__}", flags=[])
        row["flags"] = "|".join(row.get("flags", []))
        rows.append({k: fdm.fmt(row.get(k, "")) for k in fdm.FIELDS})
    with path.open("w", encoding="utf-8", newline="") as f:
        w = csv.DictWriter(f, fieldnames=fdm.FIELDS)
        w.writeheader()
        w.writerows(rows)
    print(json.dumps(dict(wrote=rel(path), tests=len(rows), status=dict(Counter(r["status"] for r in rows))), indent=1))
    return 0


# ------------------------------------------------------------------------------------------------ step 5: score
def endpoint_of(e):
    """run_final.fd_test_day_tests (C5)."""
    num = lambda s: float(s) if s not in ("", None) else math.nan  # noqa: E731
    end, label = num(e["end_s"]), e["label_sensor"]
    if e["status"] != "ok":
        return None, f"status {e['status']}"
    if label in ta.CAUSES and end == end:
        return dict(censoring="event", last_confirmed_active_s=end, first_terminal_s=end, allowed_causes=[label]), ""
    if label == "burning_at_file_end" and end == end:
        return dict(censoring="right", last_confirmed_active_s=end, first_terminal_s=None, allowed_causes=[]), ""
    return None, f"label {label or 'none'}"


def block(cases):
    """posthoc_irace_testdays.main, one block."""
    ans = [k for k in cases if not k["refused"]]
    out = dict(cases_at_risk=len(cases), cases=len(ans), tests=len({k["test"] for k in ans}), sessions=len({k["session"] for k in ans}),
               refused=dict(cases=len(cases) - len(ans), by_role=dict(Counter(r for k in cases for r in k["refused"])),
                            reasons=dict(Counter(f"{r}: {k['fc']['artifact'][r]['refusal']}" for k in cases for r in k["refused"]))),
               endpoints=dict(Counter(k["outcome"]["causes"][0] if k["outcome"]["causes"] else "right_censored" for k in ans)))
    if not ans:
        return out, {}
    outcomes = [k["outcome"] for k in ans]
    ss = [k["session"] for k in ans]
    contrib = {}
    for r in ROLES + ("incidence_baseline", "irace", "irace_R"):
        cif = {cz: [(k["fc"]["artifact"][r]["cif"] if r in ROLES else k["fc"][r])[cz] for k in ans] for cz in ta.CAUSES}
        s = ta.score(outcomes, cif)
        contrib[r] = {cz: s[cz]["contrib"] for cz in ta.CAUSES}
        contrib[r]["sum"] = [a + b for a, b in zip(contrib[r]["extinction"], contrib[r]["operator_termination"])]
    out["score"] = {r: {m: tf.means(v, ss) for m, v in d.items()} for r, d in contrib.items()}
    out["paired"] = {f"f2_minus_{o}": {m: ta.boot([a - b for a, b in zip(contrib["f2"][m], contrib[o][m])], ss) for m in METRICS}
                     for o in COMPARATORS}
    out["R_gt_1_cases"] = sum(k["fc"]["irace_q"]["R"] > 1 for k in ans)
    return out, {(k["test"], k["cutoff"]): {r: {m: contrib[r][m][i] for m in METRICS} for r in contrib} for i, k in enumerate(ans)}


def cmd_score(_args):
    need(OUT / "endpoints.csv", "measure")
    ta.selftest()
    fc = json.loads((OUT / "forecasts.json").read_text(encoding="utf-8"))
    inputs = json.loads((INPUT / "inputs.json").read_text(encoding="utf-8"))
    with (OUT / "endpoints.csv").open(encoding="utf-8", newline="") as f:
        ends = {r["test"]: r for r in csv.DictReader(f)}
    per_test = {}
    for tid, info in inputs["tests"].items():
        e = ends[tid]
        ep, why = endpoint_of(e)
        num = lambda s: float(s) if s not in ("", None) else math.nan  # noqa: E731
        planned_first = info["stops"]["new_planned_first_flow_change_s"]
        fcs = num(e["flow_change_s"])
        at = [c for c in ta.CUTOFFS if ta.at_risk(ep, c)]
        per_test[tid] = dict(tier=info["tier"], family=info["family"], session=info["session"], status=e["status"],
                             label=e["label_sensor"], end_s=num(e["end_s"]), flow_change_s=fcs, planned_first_flow_change_s=planned_first,
                             flow_change_minus_planned_s=fcs - planned_first if fcs == fcs else None,
                             onset_method=e["onset_method"], ignition_stamp_source=e["ignition_stamp_source"], flags=e["flags"],
                             tp_class=e["tp_class"], labels_agree=e["labels_agree"], endpoint=ep, not_scored_why=why,
                             cutoffs_at_risk=at)
        if ep is not None and not at:
            per_test[tid]["not_scored_why"] = "ended at or before the first cutoff (1 s): not at risk"
    res = dict(what="ACME fresh batch (PSI-10, tiers A1-A3), scored post hoc; F2 minus each comparator",
               batch=inputs["batch"], batch_sha256=inputs["batch_sha256"], ignis=fc["ignis"], releases=fc["releases"],
               endpoints_csv_sha256=sha(OUT / "endpoints.csv"), bootstrap=dict(resamples=ta.N_BOOT, seed=ta.BOOT_SEED, unit="session (test day)"),
               blocks={}, a1_operator_termination_at_scripted_ramp={})
    sets = {"A1": ("A1",), "A2": ("A2",), "A3": ("A3",), "A1+A2 (normal)": ("A1", "A2"), "pooled A1+A2+A3": TIERS}
    rows = []
    for path in PATHS:
        allk = []
        for k in fc["forecasts"]:
            if k["path"] != path:
                continue
            t = per_test[k["test"]]
            if not ta.at_risk(t["endpoint"], k["cutoff"]):
                continue
            allk.append(dict(test=k["test"], tier=k["tier"], session=k["session"], cutoff=k["cutoff"], outcome=ta.outcome_of(t["endpoint"]),
                             fc=k, refused=[r for r in ROLES if not k["artifact"][r]["admitted"]]))
        for name, tiers in sets.items():
            sel = [k for k in allk if k["tier"] in tiers]
            if not sel:
                continue
            b, contrib = block(sel)
            res["blocks"][f"{path}/{name}"] = b
            if name in TIERS:
                for k in sel:
                    rows.append(dict(path=path, tier=name, test=k["test"], session=k["session"], cutoff=k["cutoff"],
                                     outcome=k["outcome"], refused=k["refused"], stop_read_by_f2=k["fc"]["stop_read_by_f2"],
                                     irace_R=k["fc"]["irace_q"]["R"], irace_t_ext_erfc_s=k["fc"]["irace_q"]["t_ext_erfc_s"],
                                     contrib=contrib.get((k["test"], k["cutoff"])),
                                     cif={r: k["fc"]["artifact"][r]["cif"] for r in ROLES}))
    a1 = [v for v in per_test.values() if v["tier"] == "A1"]
    at_ramp = [v for v in a1 if v["label"] == "operator_termination" and v["flow_change_minus_planned_s"] is not None
               and abs(v["flow_change_minus_planned_s"]) <= 2.0]
    res["a1_operator_termination_at_scripted_ramp"] = dict(
        a1_tests=len(a1), labels=dict(Counter(v["label"] for v in a1)), operator_termination_at_scripted_ramp=len(at_ramp),
        flow_change_minus_planned_s=sorted(round(v["flow_change_minus_planned_s"], 2) for v in a1 if v["flow_change_minus_planned_s"] is not None),
        rule="label operator_termination and measured flow change within 2 s of acme_script2's planned first flow change")
    res["not_scored"] = {tid: v["not_scored_why"] for tid, v in per_test.items() if v["not_scored_why"]}
    res["per_test"] = {tid: {k: v[k] for k in ("tier", "status", "label", "end_s", "flow_change_s", "planned_first_flow_change_s",
                                                 "onset_method", "ignition_stamp_source", "tp_class", "labels_agree", "flags",
                                                 "cutoffs_at_risk", "not_scored_why")} for tid, v in per_test.items()}
    ta.write_json(OUT / "aggregates.json", res)
    with (OUT / "cases.jsonl").open("w", encoding="utf-8", newline="\n") as f:
        for r in rows:
            f.write(json.dumps(r, default=str) + "\n")
    tables(res)
    return 0


def tables(res):
    ci = lambda c: "(one session)" if c is None else f"[{c[0]:+.4f}, {c[1]:+.4f}]"  # noqa: E731
    for key, b in res["blocks"].items():
        print(f"\n== {key}: {b['cases']} cases ({b['tests']} tests, {b['sessions']} days); at risk {b['cases_at_risk']}; "
              f"refused {b['refused']['cases']} {b['refused']['by_role']}; endpoints {b['endpoints']}")
        if "score" not in b:
            continue
        for r, v in b["score"].items():
            print(f"   {r:<19} " + "  ".join(f"{v[m]['case_mean']:.4f}" for m in METRICS))
        for n, v in b["paired"].items():
            print(f"   {n:<30} " + "  ".join(f"{v[m]['case_mean']:+.4f} {ci(v[m]['ci95_case_mean'])}" for m in METRICS))
    print("\nA1:", json.dumps(res["a1_operator_termination_at_scripted_ramp"]))
    print("not scored:", json.dumps(res["not_scored"]))


def cmd_tables(_args):
    tables(json.loads((OUT / "aggregates.json").read_text(encoding="utf-8")))
    return 0


def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    for n, fn in (("inputs", cmd_inputs), ("equivalence", cmd_equivalence), ("forecast", cmd_forecast),
                  ("measure", cmd_measure), ("score", cmd_score), ("tables", cmd_tables)):
        p = sub.add_parser(n)
        p.set_defaults(fn=fn)
        p.add_argument("--workers", type=int, default=12)
    a = ap.parse_args()
    return a.fn(a)


if __name__ == "__main__":
    sys.exit(main())
