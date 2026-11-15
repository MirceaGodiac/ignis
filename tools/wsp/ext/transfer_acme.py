#!/usr/bin/env python3
"""Transfer test of the frozen CFI-G burner-flame models on ACME Flame Design (PSI-10), as registered
in docs/wsp/PROTOCOL.md section 11.1. This tool fits nothing, searches nothing and changes no model,
threshold or existing file. It writes only under `runs/dev/data_all/work/transfer_acme/`.

The scoring chain is: a PrefixBundle file per (test, cutoff) -> `ignis wsp forward` (the frozen
release, unchanged) -> the event probabilities at the 14 development horizons -> the IPCW Brier
score and its integral exactly as `crates/ignis-lab/src/eval/scores.rs` and `scoring.rs` compute
them (ported line for line below: `km_censoring`, `g_at`, `status_at`, `ipcw_terms`, `ipcw_brier`,
`integrated_brier`).

Subcommands, in the order of the protocol:

  cfig            Step 1. Builds the prefixes of the CFI-G TRAIN cases exactly as `wsp-dev feedback`
                  does (`ignis_core::analysis::build_run_prefix`: the record's conditions that the
                  committee policy allows, the control stages known at the cutoff, no observation),
                  runs the forward, scores, and compares with a `wsp-dev feedback` report of the same
                  releases (aggregate, per horizon and per case). Reads train rows only: a line of a
                  records file is parsed only when its run identity is a train id. Also freezes the
                  incidence baseline (write-once).
  cfig-endpoints  Step 2. Runs the endpoint measurement of tools/wsp/ext/flame_design.py, unchanged
                  (its file-name prefix and folder are the only things overridden, from here), on the
                  raw sensor files of CFI-G train tests and compares flame end and label with the
                  table burn time and label of the train records.
  fd              Step 3. Flame Design: cases from conditions.csv (inclusion flags) and endpoints.csv
                  (measured by flame_design.py), prefixes for arm A and arm B without and with the
                  planned stop, forwards, scores, session bootstrap, report.

Choices fixed here BEFORE any Flame Design outcome was computed (the protocol is silent on them):

  * Endpoint of a Flame Design test: the development ingest (`ignis-data/src/families/acme.rs`)
    stores a non-integer burn time as an exact event (censoring `event`, lower = upper = burn time)
    and an integer one as the interval burn time +/- 0.5 s. A measured flame end is not an integer,
    so it is stored as an exact event at `end_s` (seconds after the measured onset), cause from
    `label_sensor`. `burning_at_file_end` is right-censored at the file end. `unreadable`, or a
    status other than ok, has no endpoint; such a test is not at risk at any cutoff, as a
    development run without an endpoint and without observations is not (`phase_established_at`).
  * A case exists when the endpoint is later than the cutoff (`at_risk`), cutoffs 1, 3, 5 s.
  * Conditions are read from conditions.csv as written there (rounded as that file rounds them).
  * Incidence baseline: for a family and a cutoff, over the CFI-G train cases at risk at that
    cutoff, the fraction whose event had happened by the horizon as the scorer counts it (upper end
    of the table interval <= horizon), by cause. It answers every Flame Design case of that family
    and cutoff with the same curve.
  * Scores are computed on the cases every compared release answered; refusals are counted apart.
  * Paired difference: per-case contributions to the integrated Brier (`ibs_contrib` of the
    evaluator, IPCW weights from the whole set of cases). PRIMARY interval: sessions resampled with
    replacement, the pooled case mean of the difference recomputed (sum over drawn sessions of the
    differences / number of drawn cases), percentile 95 %. SECONDARY, shown beside it: the
    evaluator's own `cluster_bootstrap` statistic (mean of per-session means). 10000 resamples,
    seed 20261114 (numpy PCG64).
  * Arm B verdict: the rule of the protocol is applied to each model (normal, inverse) separately,
    on the primary variant (ethane indicators, no planned stop): the families are never pooled.
  * Planned stop: the prefix carries the condition `programmed_stop` = the script's stage sum
    (`script_programmed_stop_s` of endpoints.csv) when the script file gives one; a test without it
    is run without, as in development. The release applies its own typed offset.

Amendment of 14 November, made before any Flame Design endpoint was measured or scored here: the
evaluation groups of Flame Design were frozen (docs/wsp/atlas/splits/PSI-10.json, group = test day).
The tests of its TEST days (normal 19123, 19200, 19206; inverse 21361) are dropped from every arm and
variant: their endpoints are not measured, not read and not scored. The remaining tests are reported
together (primary, as registered) and separately for the days frozen as train and as validation.

    python tools/wsp/ext/transfer_acme.py cfig
    python tools/wsp/ext/transfer_acme.py cfig-endpoints
    python tools/wsp/ext/transfer_acme.py fd-fetch [--execute]
    python tools/wsp/ext/transfer_acme.py fd-tests-file
    python tools/wsp/ext/flame_design.py endpoints --tests-file runs/dev/data_all/work/transfer_acme/fd/tests_remaining.txt
    python tools/wsp/ext/transfer_acme.py fd
    python tools/wsp/ext/transfer_acme.py fd-tables
"""
import argparse
import bisect
import csv
import hashlib
import json
import math
import os
import re
import subprocess
import sys
from collections import Counter
from concurrent.futures import ThreadPoolExecutor
from decimal import Decimal
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(REPO / "tools/wsp/analysis"))
sys.path.insert(0, str(Path(__file__).resolve().parent))
OUT = REPO / "runs/dev/data_all/work/transfer_acme"
FD_WORK = REPO / "runs/dev/data_all/work/flame_design"
BASELINE = OUT / "incidence_baseline_cfig_train.json"
EXE = REPO / "target/release" / ("ignis.exe" if os.name == "nt" else "ignis")  # `--ignis` names another copy

HORIZONS = [6.0, 8.0, 10.0, 12.0, 15.0, 20.0, 25.0, 30.0, 40.0, 50.0, 60.0, 80.0, 100.0, 120.0]
CUTOFFS = [1.0, 3.0, 5.0]
CAUSES = ["extinction", "operator_termination"]
# what `ForecastSettings { times_s, horizon_s: None, ..Default }.value()` serialises to in wsp-dev
SETTINGS = {"seed": 0, "quantile_levels": None, "horizon_s": None, "n_points": None, "times_s": HORIZONS}

RELEASES = {
    "acme_normal": {
        "p1": "sha256:f71fce5f22f272cc1c336d89e32530f53a21640c91797e6a50e6219f985e69a7",
        "carry": "sha256:dce4d7f0b3ce75b6908d8be60313ecf7f7b94d0c25aefd79120af08a71f94a35",
        "reference": "sha256:6044b69bd5d10a150734bea2b6d549a8903dbdc49ddc911f7ff1b6bf2c0b2bff",
    },
    "acme_inverse": {
        "p1": "sha256:d7292e8974eed0f0ade50df68e14e3b9134c5c41ec3b8ddd664e02db7561fcc2",
        "carry": "sha256:ff3449c4123f0ae5e1434f72b9a031588bb93a39a6514369624a906337035c26",
        "reference": "sha256:3d7d010fd8488d7071e756719f4bf8e7a2fb1f8a768179b063f55ec38d226424",
    },
}
# numbers of `wsp-dev feedback` the protocol names (extinction, operator termination), no planned stop
REGISTERED_TRAIN = {"acme_normal": (0.0518, 0.0274), "acme_inverse": (0.0271, 0.0608)}
TOLERANCE = 0.0005

# `ignis_wsp::committee::ALLOWED` and the keys `build_prefix` refuses (ALWAYS_FORBIDDEN + flame3 FORBIDDEN)
ALLOWED = ["pressure", "injected_mass_flow", "x_o2_role", "x_fuel_role", "tad", "zst", "fuel_is_ethane",
           "fuel_is_butane", "fuel_species", "programmed_stop"]
FORBIDDEN = ["k_full", "burning_rate_full", "final_diameter", "extinction_diameter", "duration", "burn_time",
             "end_time", "final_label", "endpoint", "max_burner_temperature", "peak_temperature",
             "post_test_composition", "post_test_pressure", "outcome", "publication_outcome",
             "hc_burnrate", "hc_burn_rate", "tfp", "peak_burner_temperature", "after_test_composition",
             "final_duration", "duration_s", "extinction_time", "ignition_success"]
ACME_TIME_ORIGIN = "hot ignition per table 'burn time (sec)' (not adjudicated against logs)"
FD_TIME_ORIGIN = ("flame onset measured from the raw channels by tools/wsp/ext/flame_design.py "
                  "(radiometer rise above the pre-ignition level)")
N_BOOT, BOOT_SEED = 10000, 20261114
NAN = float("nan")


# ---------------------------------------------------------------------------------------------
# canonical JSON and content hash, as `ignis_core::hashing` (serde_json writes floats with ryu)
# ---------------------------------------------------------------------------------------------
def ryu(x):
    """A finite f64 as serde_json prints it (shortest round-trip digits, ryu's layout)."""
    if x == 0.0:
        return "-0.0" if math.copysign(1.0, x) < 0 else "0.0"
    sign, digits, exp = Decimal(repr(abs(x))).as_tuple()
    ds = "".join(map(str, digits)).lstrip("0")
    k = exp + (len(ds) - len(ds.rstrip("0")))
    ds = ds.rstrip("0")
    n, kk = len(ds), len(ds) + k
    if 0 <= k and kk <= 16:
        body = ds + "0" * k + ".0"
    elif 0 < kk <= 16:
        body = ds[:kk] + "." + ds[kk:]
    elif -5 < kk <= 0:
        body = "0." + "0" * (-kk) + ds
    elif n == 1:
        body = f"{ds}e{kk - 1}"
    else:
        body = f"{ds[0]}.{ds[1:]}e{kk - 1}"
    return ("-" if x < 0 else "") + body


def canonical(v):
    if isinstance(v, dict):
        return "{" + ",".join(json.dumps(k, ensure_ascii=False) + ":" + canonical(v[k]) for k in sorted(v)) + "}"
    if isinstance(v, (list, tuple)):
        return "[" + ",".join(canonical(x) for x in v) + "]"
    if isinstance(v, bool):
        return "true" if v else "false"
    if v is None:
        return "null"
    if isinstance(v, float):
        return ryu(v) if math.isfinite(v) else "null"
    if isinstance(v, int):
        return str(v)
    return json.dumps(v, ensure_ascii=False)


def content_hash(obj, exclude=()):
    v = {k: x for k, x in obj.items() if k not in exclude}
    return "sha256:" + hashlib.sha256(canonical(v).encode("utf-8")).hexdigest()


def finish_prefix(bundle):
    bundle["input_hash"] = content_hash(bundle, ("input_hash",))
    return bundle


# ---------------------------------------------------------------------------------------------
# the evaluator's event scores (crates/ignis-lab/src/eval/scores.rs), ported
# ---------------------------------------------------------------------------------------------
def km_censoring(times, censored):
    uniq = sorted({t for t, c in zip(times, censored) if c})
    g, surv = [], 1.0
    for u in uniq:
        at_risk = sum(1 for t in times if t >= u)
        d = sum(1 for t, c in zip(times, censored) if t == u and c)
        surv *= 1.0 - d / at_risk
        g.append(surv)
    return uniq, g


def g_at(knots, vals, t, left):
    if not knots:
        return 1.0
    pos = bisect.bisect_left(knots, t) if left else bisect.bisect_right(knots, t)
    return 1.0 if pos == 0 else vals[pos - 1]


def _or(a, b):
    return a if a is not None else b


def status_at(o, h):
    """('event', t) | ('alive',) | ('censored',) | ('unknown',) of an outcome at horizon h."""
    c = o["censoring"]
    if c in ("event", "scheduled_stop"):
        te = _or(o["t_upper"], o["t_lower"])
        return ("event", te) if te is not None and te <= h else ("alive",)
    if c == "interval":
        if o["t_upper"] is not None and o["t_upper"] <= h:
            return ("event", o["t_upper"])
        if o["t_lower"] is not None and o["t_lower"] > h:
            return ("alive",)
        return ("censored",)
    if c == "right":
        t = _or(o["t_lower"], o["t_upper"])
        return ("alive",) if t is not None and t > h else ("censored",)
    return ("unknown",)


def ipcw_terms(outcomes, cause, horizon):
    times, cens = [], []
    for o in outcomes:
        if o["censoring"] == "right":
            times.append(_or(_or(o["t_lower"], o["t_upper"]), math.inf))
            cens.append(True)
        elif o["censoring"] in ("event", "scheduled_stop", "interval"):
            times.append(_or(_or(o["t_upper"], o["t_lower"]), math.inf))
            cens.append(False)
    knots, vals = km_censoring(times, cens)
    n = len(outcomes)
    eligible, weight, indicator = [False] * n, [0.0] * n, [0.0] * n
    unknown, ambiguous = [False] * n, [False] * n
    for i, o in enumerate(outcomes):
        st = status_at(o, horizon)
        if st[0] == "unknown":
            unknown[i] = True
            continue
        if cause is not None and st[0] == "event" and len(o["causes"]) != 1:
            ambiguous[i] = True
            continue
        eligible[i] = True
        if st[0] == "censored":
            continue
        g = g_at(knots, vals, st[1], True) if st[0] == "event" else g_at(knots, vals, horizon, False)
        if g > 0.0:
            weight[i] = 1.0 / g
        indicator[i] = 1.0 if st[0] == "event" and (cause is None or o["causes"][0] == cause) else 0.0
    return eligible, weight, indicator, unknown, ambiguous


def ipcw_brier(cif, outcomes, cause, horizon):
    eligible, weight, indicator, unknown, ambiguous = ipcw_terms(outcomes, cause, horizon)
    m = [eligible[i] and math.isfinite(cif[i]) for i in range(len(cif))]
    n = sum(m)
    terms = [weight[i] * (indicator[i] - cif[i]) ** 2 for i in range(len(cif)) if m[i]]
    return dict(brier=math.fsum(terms) / n if n else NAN, n=n,
                censored_before_horizon=sum(1 for i in range(len(cif)) if m[i] and weight[i] == 0.0),
                excluded_unknown=sum(unknown) + sum(1 for i in range(len(cif)) if eligible[i] and not math.isfinite(cif[i])),
                excluded_ambiguous_cause=sum(ambiguous))


def integrated_brier(horizons, briers):
    hb = [(h, b) for h, b in zip(horizons, briers) if math.isfinite(b)]
    if len(hb) < 2 or hb[-1][0] <= hb[0][0]:
        return NAN
    s = math.fsum((hb[i][0] - hb[i - 1][0]) * (hb[i][1] + hb[i - 1][1]) / 2.0 for i in range(1, len(hb)))
    return s / (hb[-1][0] - hb[0][0])


def interp_row(times, row, t):
    """`scoring::interp_row`: linear in time, NaN outside the forecast range."""
    out = []
    for x in t:
        if not times or x < times[0] or x > times[-1]:
            out.append(NAN)
        elif len(times) == 1 or x <= times[0]:
            out.append(row[0])
        elif x >= times[-1]:
            out.append(row[-1])
        else:
            i = bisect.bisect_right(times, x)
            x0, x1 = times[i - 1], times[i]
            out.append(row[i - 1] + (row[i] - row[i - 1]) * (x - x0) / (x1 - x0) if x1 > x0 else row[i])
    return out


def score(outcomes, cifs):
    """`scoring::event_block` for one release: `cifs[cause][i]` is the incidence of case i at HORIZONS.
    Returns per cause the Brier per horizon, its integral, and each case's contribution."""
    res = {}
    for cause in CAUSES:
        f = cifs[cause]
        per_h, contrib = [], [[NAN] * len(HORIZONS) for _ in outcomes]
        for j, hj in enumerate(HORIZONS):
            col = [r[j] for r in f]
            per_h.append(ipcw_brier(col, outcomes, cause, hj))
            eligible, weight, indicator, _, _ = ipcw_terms(outcomes, cause, hj)
            for i in range(len(outcomes)):
                if eligible[i] and math.isfinite(col[i]):
                    contrib[i][j] = weight[i] * (indicator[i] - col[i]) ** 2
        res[cause] = dict(brier=per_h, integrated_brier=integrated_brier(HORIZONS, [p["brier"] for p in per_h]),
                          contrib=[integrated_brier(HORIZONS, row) for row in contrib])
    return res


def selftest():
    k, g = km_censoring([1.0, 2.0, 3.0, 4.0], [False, True, False, False])
    assert k == [2.0] and abs(g[0] - (1 - 1 / 3)) < 1e-15
    assert g_at(k, g, 2.0, True) == 1.0 and abs(g_at(k, g, 2.0, False) - 2 / 3) < 1e-15
    o = [dict(censoring="event", t_lower=1.0, t_upper=1.0, causes=["extinction"]),
         dict(censoring="right", t_lower=2.0, t_upper=None, causes=[]),
         dict(censoring="event", t_lower=3.0, t_upper=3.0, causes=["operator_termination"]),
         dict(censoring="interval", t_lower=3.5, t_upper=4.5, causes=["extinction"])]
    b = ipcw_brier([0.5, 0.5, 0.5, 0.5], o, "extinction", 4.0)
    # event at 1 (w 1, I 1), censored at 2 (w 0), event of another cause at 3 (w 1.5, I 0), straddling (w 0)
    assert b["n"] == 4 and abs(b["brier"] - (0.25 + 1.5 * 0.25) / 4) < 1e-15, b
    assert abs(integrated_brier([0.0, 1.0, 3.0], [0.0, 1.0, 1.0]) - (0.5 + 2.0) / 3.0) < 1e-15
    assert ryu(2e-06) == "2e-6" and ryu(94439.56) == "94439.56" and ryu(-124.0) == "-124.0"
    assert ryu(1e-05) == "0.00001" and ryu(1.25e-07) == "1.25e-7" and ryu(1e16) == "1e16" and ryu(110000.00000000001) == "110000.00000000001"


# ---------------------------------------------------------------------------------------------
# the forward
# ---------------------------------------------------------------------------------------------
def use_binary(args):
    """The `ignis` binary of this run and its sha256. Nothing is built here: the binary must exist."""
    global EXE
    if getattr(args, "ignis", None):
        EXE = (REPO / args.ignis).resolve()
    if not EXE.exists():
        sys.exit(f"{EXE} does not exist; name an existing binary with --ignis (this tool never builds one)")
    h = hashlib.sha256()
    with EXE.open("rb") as f:
        for block in iter(lambda: f.read(1 << 20), b""):
            h.update(block)
    return dict(path=EXE.relative_to(REPO).as_posix(), sha256=h.hexdigest())


def run_forward(release, prefix_path, out_path):
    """`ignis wsp forward` on one prefix file. Returns the artifact (a scientific refusal is an
    artifact with support.admitted = false); raises when the binary cannot run the release."""
    prefix = json.loads(prefix_path.read_text(encoding="utf-8"))
    if out_path.exists():
        try:
            a = json.loads(out_path.read_text(encoding="utf-8"))
            if a.get("prefix_hash") == prefix["input_hash"] and a.get("model_release_id") == release:
                return a
        except json.JSONDecodeError:
            pass
    out_path.parent.mkdir(parents=True, exist_ok=True)
    cmd = [str(EXE), "wsp", "forward", "--release", release, "--prefix", str(prefix_path),
           "--settings", json.dumps(SETTINGS, separators=(",", ":")), "--out", str(out_path)]
    r = subprocess.run(cmd, cwd=REPO, capture_output=True, text=True)
    if r.returncode != 0:
        raise RuntimeError(f"ignis wsp forward failed for {release[:15]} on {prefix_path.name}: "
                           f"{(r.stderr or r.stdout).strip()[-600:]}")
    return json.loads(out_path.read_text(encoding="utf-8"))


def forward_many(jobs, workers):
    """jobs: (key, release, prefix_path, out_path) -> {key: artifact}. The first binary error stops all."""
    def one(job):
        key, release, pp, op = job
        return key, run_forward(release, pp, op)
    with ThreadPoolExecutor(max_workers=workers) as ex:
        return dict(ex.map(one, jobs))


def admitted(a):
    """`scoring::admitted`."""
    return bool(a["support"]["admitted"]) and a["status"] not in ("unsupported", "failed", "insufficient_inference")


def cif_of(a):
    return {c: interp_row(a["times_s"], a["event_cdf"][a["event_categories"].index(c)], HORIZONS) for c in CAUSES}


def write_json(path, obj):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(obj, indent=1, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")


# ---------------------------------------------------------------------------------------------
# CFI-G train records (train rows only)
# ---------------------------------------------------------------------------------------------
def train_rows(family, file, key, ids):
    """Rows of a records file whose top-level identity is a train id. A line is parsed only when the
    identity pattern in it names a train id, and kept only when its own identity is that id."""
    pat = re.compile(r'"%s"\s*:\s*"([^"]+)"' % key)
    path = REPO / "data/work/records" / family / file
    out = {}
    if not path.exists():
        return out
    with path.open(encoding="utf-8") as f:
        for line in f:
            if not any(x in ids for x in pat.findall(line)):
                continue
            r = json.loads(line)
            if r.get(key) in ids:
                assert r[key] not in out or file != "experiments.jsonl", "duplicate train experiment id"
                out[r[key]] = r
    return out


def at_risk(endpoint, cutoff):
    """`ignis_core::analysis::phase_established_at` for a run without observations."""
    if endpoint is None:
        return False
    terminal, confirmed = endpoint.get("first_terminal_s"), endpoint.get("last_confirmed_active_s")
    if terminal is not None and terminal <= cutoff:
        return False
    if confirmed is not None:
        return confirmed >= cutoff
    return terminal is not None


def cfig_prefix(rec, control, cutoff):
    """`ignis_core::analysis::build_run_prefix` with the committee's policy."""
    bad = [k for k in rec["conditions"] if k in FORBIDDEN]
    assert not bad, f"{rec['id']}: forbidden condition {bad}"
    stages = sorted((s for s in (control or {}).get("stages", []) if s["t_start_s"] <= cutoff),
                    key=lambda s: (s["t_start_s"], s["control"]))
    return finish_prefix(dict(
        family=rec["family"], run_id=rec["id"], session_id=rec.get("session_id"), cutoff_s=float(cutoff),
        time_origin=control["time_origin"] if control else ACME_TIME_ORIGIN,
        known_conditions={k: v for k, v in rec["conditions"].items() if k in ALLOWED},
        known_controls=stages, observations=[], contours=[], phase_established_at_s=0.0,
        support_evidence=["activity confirmed at the cutoff by the run record"], transform_versions={}))


def outcome_of(endpoint):
    """`continuation_payload` endpoint -> `scores::EventOutcome`."""
    return dict(censoring=endpoint["censoring"], t_lower=endpoint.get("last_confirmed_active_s"),
                t_upper=endpoint.get("first_terminal_s"), causes=list(endpoint.get("allowed_causes") or []))


def cfig_cases(family):
    import lib
    ids = set(lib.partition_ids(family, "train"))
    exps = train_rows(family, "experiments.jsonl", "id", ids)
    assert set(exps) == ids, f"{family}: {len(exps)} train experiment rows for {len(ids)} train ids"
    ends = train_rows(family, "endpoints.jsonl", "run_id", ids)
    ctrl = {}
    for name in ("control_histories.jsonl", "control_history.jsonl", "extra_control_history.jsonl"):
        ctrl.update(train_rows(family, name, "run_id", ids))
    cases, not_at_risk = [], 0
    for rid in sorted(ids):
        for c in CUTOFFS:
            if not at_risk(ends.get(rid), c):
                not_at_risk += 1
                continue
            cases.append(dict(run_id=rid, test=rid.split(":")[-1], cutoff=c, session=exps[rid].get("session_id"),
                              outcome=outcome_of(ends[rid]), prefix=cfig_prefix(exps[rid], ctrl.get(rid), c)))
    return cases, not_at_risk, len(ids), sum(1 for r in ids if r in ctrl)


def baseline_from(cases_by_family):
    fam = {}
    for family, cases in cases_by_family.items():
        cut = {}
        for c in CUTOFFS:
            sel = [k for k in cases if k["cutoff"] == c]
            assert all(len(k["outcome"]["causes"]) == 1 for k in sel)
            row = dict(n_at_risk=len(sel))
            for cause in CAUSES:
                row[cause] = [sum(1 for k in sel if status_at(k["outcome"], h)[0] == "event"
                                  and k["outcome"]["causes"][0] == cause) / len(sel) for h in HORIZONS]
            cut[f"{c:g}"] = row
        fam[family] = dict(cutoffs=cut, train_cases=len(cases))
    return dict(
        what="incidence baseline of docs/wsp/PROTOCOL.md 11.1",
        definition="for a family and a cutoff, over the CFI-G (PSI-159) TRAIN cases at risk at that cutoff, the "
                   "fraction whose event had happened by the horizon as the evaluator counts it (upper end of the "
                   "table interval <= horizon), by cause; computed once from CFI-G train, never from Flame Design",
        horizons_s=HORIZONS, families=fam)


def cmd_cfig(args):
    selftest()
    binary = use_binary(args)
    adir = OUT / "cfig" / ("artifact" if not args.tag else f"artifact_{args.tag}")
    report = dict(purpose="Step 1 of PROTOCOL 11.1: the scoring chain on CFI-G train", ignis=binary, settings=SETTINGS,
                  tolerance=TOLERANCE, families={})
    all_cases, ok = {}, True
    same_id = Counter()
    for family, short in (("acme_normal", "normal"), ("acme_inverse", "inverse")):
        cases, not_at_risk, runs, with_ctrl = cfig_cases(family)
        all_cases[family] = cases
        pdir = OUT / "cfig" / "prefix" / family
        pdir.mkdir(parents=True, exist_ok=True)
        jobs = []
        for k in cases:
            name = f"{k['test']}_c{k['cutoff']:g}.json"
            k["prefix_path"] = pdir / name
            write_json(k["prefix_path"], k["prefix"])
            for role, rel in RELEASES[family].items():
                jobs.append(((k["run_id"], k["cutoff"], role), rel, k["prefix_path"],
                             adir / family / role / name))
        arts = forward_many(jobs, args.workers)
        if args.tag:  # the same forwards by the binary of the untagged run: are the artifacts the same?
            for (rid, c, role), a in arts.items():
                first = OUT / "cfig" / "artifact" / family / role / f"{rid.split(':')[-1]}_c{c:g}.json"
                if not first.exists():
                    same_id["no artifact of the first run"] += 1
                else:
                    same_id["identical artifact id" if json.loads(first.read_text(encoding="utf-8"))["id"] == a["id"]
                            else "different artifact id"] += 1
        ref_path = OUT / "cfig" / f"wspdev_fb_train_{short}.json"
        ref = json.loads(ref_path.read_text(encoding="utf-8")) if ref_path.exists() else None
        fam = dict(train_runs=runs, runs_with_control_history=with_ctrl, cases=len(cases), not_at_risk=not_at_risk,
                   wsp_dev_report=str(ref_path.relative_to(REPO)) if ref else None, releases={})
        if ref:
            fam["wsp_dev_runs_cases_not_at_risk"] = [ref["runs"], len(ref["releases"][0]["cases"]), ref["not_at_risk_cases"]]
            ok &= fam["wsp_dev_runs_cases_not_at_risk"] == [runs, len(cases), not_at_risk]
        outcomes = [k["outcome"] for k in cases]
        for role, rel in RELEASES[family].items():
            a = [arts[(k["run_id"], k["cutoff"], role)] for k in cases]
            refused = [x for x in a if not admitted(x)]
            assert not refused, f"{family} {role}: {len(refused)} train cases refused: {refused[0]['support']}"
            cifs = {c: [cif_of(x)[c] for x in a] for c in CAUSES}
            s = score(outcomes, cifs)
            row = dict(release=rel, integrated_brier={c: s[c]["integrated_brier"] for c in CAUSES},
                       case_mean_contrib={c: math.fsum(s[c]["contrib"]) / len(cases) for c in CAUSES})
            if ref:
                rr = next(r for r in ref["releases"] if r["release_id"] == rel)
                ev = rr["aggregate"]["events"]["causes"]
                row["wsp_dev_integrated_brier"] = {c: ev[c]["integrated_brier"] for c in CAUSES}
                row["abs_diff_integrated_brier"] = {c: abs(ev[c]["integrated_brier"] - s[c]["integrated_brier"]) for c in CAUSES}
                row["max_abs_diff_brier_per_horizon"] = {
                    c: max(abs(p["brier"] - q["brier"]) for p, q in zip(ev[c]["brier"], s[c]["brier"])) for c in CAUSES}
                # per case: the event table at the last horizon, as wsp-dev prints it (`events_at_end`)
                dev = {(c["run_id"], c["cutoff_s"]): c for c in rr["cases"]}
                d_end, d_ep = 0.0, 0
                for k, x in zip(cases, a):
                    dc = dev[(k["run_id"], k["cutoff"])]
                    d_end = max(d_end, max(abs(p - q[-1]) for p, q in zip(dc["events_at_end"], x["event_cdf"])))
                    e = dc["endpoint"]
                    d_ep += not (e["censoring"] == k["outcome"]["censoring"] and e["t_lower_s"] == k["outcome"]["t_lower"]
                                 and e["t_upper_s"] == k["outcome"]["t_upper"] and e["allowed_causes"] == k["outcome"]["causes"])
                row["per_case_max_abs_diff_events_at_end"] = d_end
                row["per_case_endpoint_mismatches"] = d_ep
                ok &= all(v <= TOLERANCE for v in row["abs_diff_integrated_brier"].values()) and d_ep == 0
            if role == "p1":
                reg = REGISTERED_TRAIN[family]
                row["registered"] = dict(zip(CAUSES, reg))
                row["abs_diff_to_registered"] = {c: abs(s[c]["integrated_brier"] - r) for c, r in zip(CAUSES, reg)}
                ok &= all(v <= TOLERANCE for v in row["abs_diff_to_registered"].values())
            fam["releases"][role] = row
        report["families"][family] = fam
    # the prefix hash against a prefix the Rust path stored for a train case, when one is given
    if args.rust_prefix:
        p = json.loads(Path(args.rust_prefix).read_text(encoding="utf-8"))
        fam = next(f for f, cs in all_cases.items() if any(k["run_id"] == p["run_id"] for k in cs))
        mine = next(k["prefix"] for k in all_cases[fam] if k["run_id"] == p["run_id"] and k["cutoff"] == p["cutoff_s"])
        report["rust_prefix_check"] = dict(file=args.rust_prefix, run_id=p["run_id"], cutoff_s=p["cutoff_s"],
                                           same_json=mine == p, same_input_hash=mine["input_hash"] == p["input_hash"],
                                           rehash_of_rust_prefix_matches=content_hash(p, ("input_hash",)) == p["input_hash"])
    if args.tag:
        report["artifacts_against_the_untagged_run"] = dict(same_id)
        ok &= set(same_id) <= {"identical artifact id"}
    report["reproduced_within_tolerance"] = bool(ok)
    base = baseline_from(all_cases)
    if BASELINE.exists():
        frozen = json.loads(BASELINE.read_text(encoding="utf-8"))
        report["baseline"] = dict(file=str(BASELINE.relative_to(REPO)), state="already frozen; not rewritten",
                                  recomputed_equal=frozen == base)
    elif ok:
        write_json(BASELINE, base)
        report["baseline"] = dict(file=str(BASELINE.relative_to(REPO)), state="frozen now")
    else:
        report["baseline"] = dict(state="not written: the chain did not reproduce")
    if BASELINE.exists():
        report["baseline"]["sha256"] = hashlib.sha256(BASELINE.read_bytes()).hexdigest()
    write_json(OUT / "cfig" / ("chain_check.json" if not args.tag else f"chain_check_{args.tag}.json"), report)
    print(json.dumps(report, indent=1))
    return 0 if ok else 2


# ---------------------------------------------------------------------------------------------
# Step 2: the endpoint measurement on CFI-G train raw files
# ---------------------------------------------------------------------------------------------
def quantiles(v, qs=(0.0, 0.05, 0.25, 0.5, 0.75, 0.95, 1.0)):
    import numpy as np
    return {f"q{int(q * 100):02d}": round(float(np.quantile(v, q)), 3) for q in qs} if len(v) else {}


def cmd_cfig_endpoints(args):
    import flame_design as fd  # the measurement, unchanged; only where it reads from is set here
    import lib
    files = REPO / "runs/dev/nasa/PSI-159"
    pre = "PSI-159_Raw Data_"
    fd.FILES = files
    have = set(os.listdir(files))
    rows = []
    for family in ("acme_normal", "acme_inverse"):
        ids = set(lib.partition_ids(family, "train"))
        ends = train_rows(family, "endpoints.jsonl", "run_id", ids)
        for rid in sorted(ids):
            test = rid.split(":")[-1]
            pick = lambda n: n if n in have else None  # noqa: E731
            fn = dict(data=pick(f"{pre}{test}_data.csv"), setpoints=pick(f"{pre}{test}_mfcSetpoints.csv"),
                      tp=pick(f"{pre}{test}_TP.txt"), script=pick(f"{pre}as run{test}.txt"))
            try:
                m = fd.measure(test, fn)
            except Exception as e:  # as the script's own loop does
                m = dict(test=test, status=f"error:{type(e).__name__}", flags=[])
            e = ends.get(rid) or {}
            lo, hi = e.get("last_confirmed_active_s"), e.get("first_terminal_s")
            causes = e.get("allowed_causes") or []
            rows.append(dict(
                test=test, family=family, status=m["status"], has_tp=int(fn["tp"] is not None),
                has_setpoints=int(fn["setpoints"] is not None), onset_method=m.get("onset_method", ""),
                onset_minus_stamp_s=m.get("onset_minus_stamp_s", NAN), label_sensor=m.get("label_sensor", ""),
                end_s=m.get("end_s", NAN), pmt_end_s=m.get("pmt_end_s", NAN), flow_change_s=m.get("flow_change_s", NAN),
                table_resolved=int(e.get("resolution") == "resolved" and len(causes) == 1 and lo is not None),
                table_label=causes[0] if len(causes) == 1 else "|".join(causes),
                table_burn_s=(lo + hi) / 2.0 if lo is not None and hi is not None else NAN,
                table_censoring=e.get("censoring", ""), flags="|".join(m.get("flags", []))))
    for r in rows:
        r["end_minus_table_s"] = r["end_s"] - r["table_burn_s"]
    path = OUT / "cfig" / "endpoints_check.csv"
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8", newline="") as f:
        w = csv.DictWriter(f, fieldnames=list(rows[0].keys()))
        w.writeheader()
        for r in rows:
            w.writerow({k: (("" if v != v else f"{v:.3f}") if isinstance(v, float) else v) for k, v in r.items()})
    rep = dict(purpose="Step 2 of PROTOCOL 11.1: flame_design.py's endpoint measurement on CFI-G train raw files",
               wrote=str(path.relative_to(REPO)), thresholds="unchanged (module constants of flame_design.py)",
               families={})
    for family in ("acme_normal", "acme_inverse", "both"):
        rr = [r for r in rows if family == "both" or r["family"] == family]
        cmp_ = [r for r in rr if r["status"] == "ok" and r["table_resolved"]
                and r["label_sensor"] in ("extinction", "operator_termination")]
        d = [r["end_minus_table_s"] for r in cmp_ if r["end_minus_table_s"] == r["end_minus_table_s"]]
        same = [r for r in cmp_ if r["label_sensor"] == r["table_label"]]
        d_same = [r["end_minus_table_s"] for r in same]
        rep["families"][family] = dict(
            train_tests=len(rr), status=dict(Counter(r["status"] for r in rr)),
            table_endpoint_resolved=sum(r["table_resolved"] for r in rr),
            measured_ok_and_table_resolved=sum(1 for r in rr if r["status"] == "ok" and r["table_resolved"]),
            sensor_label_of_those=dict(Counter(r["label_sensor"] for r in rr if r["status"] == "ok" and r["table_resolved"])),
            comparable=len(cmp_), labels_agree=len(same),
            confusion_table_vs_sensor=dict(Counter(f"table {r['table_label']} / sensor {r['label_sensor']}" for r in cmp_)),
            onset_method=dict(Counter(r["onset_method"] for r in cmp_)),
            end_minus_table_s=dict(n=len(d), **quantiles(d),
                                   within_0_5s=sum(abs(x) <= 0.5 for x in d), within_1s=sum(abs(x) <= 1.0 for x in d),
                                   within_2s=sum(abs(x) <= 2.0 for x in d), within_5s=sum(abs(x) <= 5.0 for x in d),
                                   beyond_5s=sum(abs(x) > 5.0 for x in d)),
            end_minus_table_s_where_labels_agree=dict(
                n=len(d_same), **quantiles(d_same), within_1s=sum(abs(x) <= 1.0 for x in d_same),
                within_2s=sum(abs(x) <= 2.0 for x in d_same), beyond_5s=sum(abs(x) > 5.0 for x in d_same)),
            # at risk at a cutoff: by the table as development counts it (lower end of the burn-time
            # interval >= cutoff), and by the measured end as Flame Design is counted (end > cutoff)
            cases_at_cutoffs_table_vs_sensor={f"{c:g}": [sum(1 for r in cmp_ if r["table_burn_s"] - 0.5 >= c),
                                                         sum(1 for r in cmp_ if r["end_s"] > c)] for c in CUTOFFS},
            flag_counts=dict(Counter(f for r in rr for f in r["flags"].split("|") if f)))
    write_json(OUT / "cfig" / "endpoints_check.json", rep)
    print(json.dumps(rep, indent=1))
    return 0


# ---------------------------------------------------------------------------------------------
# Step 3: Flame Design
# ---------------------------------------------------------------------------------------------
FD_COND = {  # condition key -> (column of conditions.csv, unit)
    "pressure": ("pressure_pa", "Pa"), "injected_mass_flow": ("injected_mass_flow_kg_s", "kg/s"),
    "x_o2_role": ("x_o2_role", "1"), "x_fuel_role": ("x_fuel_role", "1"), "tad": ("tad_k_table", "K"),
    "zst": ("zst_initial", "1"),
}
# docs/wsp/atlas/splits/PSI-10.json (frozen 14 November): test days by partition. The test days are
# never measured, read or scored here; a day that is not listed is held and stops the run.
FD_DAYS = {
    "acme_normal": dict(train_days={"19115", "19137", "19150", "19156", "19171", "19175", "19182", "19189"},
                        validation_days={"19142", "19196", "21301"}, test_days={"19123", "19200", "19206"}),
    "acme_inverse": dict(train_days={"21328", "21349", "22024", "22027", "22035"},
                         validation_days={"21340", "22018"}, test_days={"21361"}),
}


def fd_part(family, test):
    for part, days in FD_DAYS[family].items():
        if test[:5] in days:
            return part
    sys.exit(f"{test}: day {test[:5]} is in no partition of docs/wsp/atlas/splits/PSI-10.json (held): not run")


# arm B: the fuel handed to the forward as stored indicators (they win over the species text)
SURROGATE = {"ethane": (1.0, 0.0), "propane": (0.0, 0.0), "butane": (0.0, 1.0)}


def measured(value, unit, column, row, sha, method_id, notes, evidence="source_derived", flags=(), path=None):
    num = isinstance(value, float)
    return dict(
        derivation=dict(agent=None, inputs=[], method="deterministic", method_id=method_id, method_version="1", notes=notes),
        evidence_class=evidence, flags=list(flags),
        locator=dict(asset_sha256=sha, char_end=None, char_start=None, column=column, frame=None, page=None,
                     path=path or "runs/dev/data_all/work/flame_design/conditions.csv", pts_s=None, row=row),
        raw_unit=unit, raw_value=value, resolution="resolved",
        uncertainty_reason="not reported by source" if num else None, uncertainty_si=None,
        unit_si=unit, value_si=value if num else None)


def fd_prefix(t, cutoff, fuel, stop, sha_c, sha_e):
    """fuel: 'recorded' (the species text only, as the records carry it) or a key of SURROGATE (the
    species text as recorded plus the two stored indicators). stop: carry `programmed_stop` or not."""
    cond = {}
    for key, (col, unit) in FD_COND.items():
        cond[key] = measured(float(t[col]), unit, col, t["row"], sha_c, f"wsp.ext.flame_design.cond.{key}",
                             "from conditions.csv of tools/wsp/ext/flame_design.py (initial composition; table Tad)")
    cond["fuel_species"] = measured(t["fuel_species"], None, "fuel_species", t["row"], sha_c,
                                    "wsp.ext.flame_design.cond.fuel_species", "fuel as recorded in the experiment table",
                                    evidence="original_observation")
    if fuel != "recorded":
        fe, fb = SURROGATE[fuel]
        note = (f"arm B of PROTOCOL 11.1: a declared surrogate. The recorded fuel is {t['fuel_species']}; the forward "
                f"is handed the indicators of {fuel}")
        cond["fuel_is_ethane"] = measured(fe, "1", "fuel_species", t["row"], sha_c, "wsp.ext.transfer.surrogate_fuel",
                                          note, evidence="ignis_derived", flags=["surrogate_fuel_indicator"])
        cond["fuel_is_butane"] = measured(fb, "1", "fuel_species", t["row"], sha_c, "wsp.ext.transfer.surrogate_fuel",
                                          note, evidence="ignis_derived", flags=["surrogate_fuel_indicator"])
    if stop and t["programmed_stop"] == t["programmed_stop"] and t["programmed_stop"] > 0:
        cond["programmed_stop"] = measured(
            float(t["programmed_stop"]), "s", "script_programmed_stop_s", t["erow"], sha_e,
            "wsp.ext.flame_design.cond.programmed_stop",
            "sum of the script's stage durations up to the end of its last constant-flow stage, from the test's "
            "script parameter file; set before ignition", path="runs/dev/data_all/work/flame_design/endpoints.csv")
    return finish_prefix(dict(
        family=t["family"], run_id=f"PSI-10:{t['family']}:{t['test']}", session_id=f"PSI-10:session:{t['session']}",
        cutoff_s=float(cutoff), time_origin=FD_TIME_ORIGIN, known_conditions=cond, known_controls=[],
        observations=[], contours=[], phase_established_at_s=0.0,
        support_evidence=["activity confirmed at the cutoff by the measured flame end (tools/wsp/ext/flame_design.py)"],
        transform_versions={}))


def cmd_fd_fetch(args):
    """The small text files (sensor csv, setpoints, TP, script) of the included tests, by the data lane's
    command, PSI-10 only, dry run unless --execute. A file is asked for when it is not on disk or not in
    the investigation's manifest (a batch the server cut short leaves files the manifest does not know)."""
    import flame_design as fd
    binary = use_binary(args)
    names = fd.listing()
    with (FD_WORK / "conditions.csv").open(encoding="utf-8", newline="") as f:
        tests = [r["test"] for r in csv.DictReader(f) if r["include_transfer"] == "1"]
    known = set()
    mpath = fd.DATA_ROOT / "manifests/PSI-10.jsonl"
    if mpath.exists():
        known = {json.loads(line).get("file_name") for line in mpath.read_text(encoding="utf-8").splitlines() if line.strip()}
    need = {}
    for t in tests:
        for n in fd.file_names(t, names).values():
            if n and not ((fd.FILES / n).exists() and n in known):
                assert n.endswith(("_data.csv", "_mfcSetpoints.csv", "_TP.txt", ".txt")), n
                need.setdefault(t, []).append(n)
    total = sum(names[n] for v in need.values() for n in v)
    print(json.dumps(dict(ignis=binary, tests=len(tests), tests_with_files_to_fetch=len(need),
                          files=sum(len(v) for v in need.values()), bytes=total, execute=bool(args.execute))), flush=True)
    todo, failed = list(need), 0
    env = dict(os.environ, IGNIS_DATA=fd.DATA_ROOT.relative_to(REPO).as_posix())
    for i in range(0, len(todo), args.batch):
        b = todo[i:i + args.batch]
        alt = "|".join(sorted({re.escape(n[len(fd.PRE):]) for t in b for n in need[t]}))
        cmd = [str(EXE), "sources", "fetch", "PSI-10", "--pattern", f"^{re.escape(fd.PRE)}({alt})$", "--category", "Raw Data"]
        if args.execute:
            cmd.append("--execute")
        for attempt in range(1, args.retries + 2):
            r = subprocess.run(cmd, cwd=REPO, env=env, capture_output=True, text=True)
            if r.returncode == 0:
                break
        failed += r.returncode != 0
        tail = " ".join((r.stdout or r.stderr).strip().split())[-300:]
        print(f"# batch {i // args.batch + 1}: {len(b)} tests, exit {r.returncode} after {attempt} attempt(s): {tail}", flush=True)
    return 1 if failed else 0


def fd_tests():
    cpath, epath = FD_WORK / "conditions.csv", FD_WORK / "endpoints.csv"
    if not epath.exists():
        sys.exit("run `fd-tests-file`, then flame_design.py endpoints --tests-file <that file> first")
    sha_c, sha_e = (hashlib.sha256(p.read_bytes()).hexdigest() for p in (cpath, epath))
    with cpath.open(encoding="utf-8", newline="") as f:
        cond = [dict(r, row=i + 2) for i, r in enumerate(csv.DictReader(f)) if r["include_transfer"] == "1"]
    for r in cond:
        r["part"] = fd_part(r["family"], r["test"])
    keep = {r["test"] for r in cond if r["part"] != "test_days"}
    with epath.open(encoding="utf-8", newline="") as f:  # a row of a test day, if the file has one, is not kept
        ends = {r["test"]: dict(r, erow=i + 2) for i, r in enumerate(csv.DictReader(f)) if r["test"] in keep}
    dropped = {fam: dict(tests=sum(1 for r in cond if r["family"] == fam and r["part"] == "test_days"),
                         days=sorted({r["session"] for r in cond if r["family"] == fam and r["part"] == "test_days"}))
               for fam in FD_DAYS}
    tests = []
    if True:
        for r in cond:
            if r["part"] == "test_days":
                continue
            e = ends.get(r["test"])
            t = dict(r, erow=e["erow"] if e else None, endpoint=None, why="not measured", programmed_stop=NAN,
                     onset_method="", eflags="")
            if e:
                num = lambda s: float(s) if s not in ("", None) else NAN  # noqa: E731
                t.update(programmed_stop=num(e["script_programmed_stop_s"]), onset_method=e["onset_method"], eflags=e["flags"])
                end, label = num(e["end_s"]), e["label_sensor"]
                if e["status"] != "ok":
                    t["why"] = f"status {e['status']}"
                elif label in CAUSES and end == end:
                    t["endpoint"] = dict(censoring="event", last_confirmed_active_s=end, first_terminal_s=end,
                                         allowed_causes=[label])
                    t["why"] = ""
                elif label == "burning_at_file_end" and end == end:
                    t["endpoint"] = dict(censoring="right", last_confirmed_active_s=end, first_terminal_s=None,
                                         allowed_causes=[])
                    t["why"] = ""
                else:
                    t["why"] = f"label {label or 'none'}"
            tests.append(t)
    return tests, sha_c, sha_e, dropped


def cmd_fd_tests_file(_args):
    """The included tests that are not on a test day, for `flame_design.py endpoints --tests-file`."""
    with (FD_WORK / "conditions.csv").open(encoding="utf-8", newline="") as f:
        rows = [r for r in csv.DictReader(f) if r["include_transfer"] == "1"]
    keep = [r["test"] for r in rows if fd_part(r["family"], r["test"]) != "test_days"]
    path = OUT / "fd" / "tests_remaining.txt"
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("\n".join(keep) + "\n", encoding="utf-8", newline="\n")
    print(json.dumps(dict(wrote=path.relative_to(REPO).as_posix(), included=len(rows), on_test_days_dropped=len(rows) - len(keep),
                          remaining=len(keep))))
    return 0


def boot(d, sessions, n_boot=N_BOOT, seed=BOOT_SEED):
    """Session bootstrap of a per-case quantity: pooled case mean (primary) and mean of session means."""
    import numpy as np
    labs = sorted(set(sessions))
    idx = {s: i for i, s in enumerate(labs)}
    tot, cnt = np.zeros(len(labs)), np.zeros(len(labs))
    for x, s in zip(d, sessions):
        tot[idx[s]] += x
        cnt[idx[s]] += 1
    out = dict(case_mean=float(tot.sum() / cnt.sum()), session_equal_mean=float(np.mean(tot / cnt)),
               n_cases=int(cnt.sum()), n_sessions=len(labs))
    if len(labs) < 2:
        out.update(ci95_case_mean=None, ci95_session_equal_mean=None, note="one session: no interval")
        return out
    rng = np.random.default_rng(seed)
    draw = rng.integers(0, len(labs), size=(n_boot, len(labs)))
    pooled = tot[draw].sum(axis=1) / cnt[draw].sum(axis=1)
    equal = (tot / cnt)[draw].mean(axis=1)
    out.update(ci95_case_mean=[float(np.quantile(pooled, 0.025)), float(np.quantile(pooled, 0.975))],
               ci95_session_equal_mean=[float(np.quantile(equal, 0.025)), float(np.quantile(equal, 0.975))],
               n_boot=n_boot, seed=seed)
    return out


def fd_block(name, tag, family, tests, fuel, stop, baseline, sha_c, sha_e, workers, roles=("p1", "carry", "reference")):
    """One arm and variant of one family: prefixes, forwards, scores."""
    pdir = OUT / "fd" / "prefix" / family
    pdir.mkdir(parents=True, exist_ok=True)
    cases, jobs = [], []
    for t in tests:
        for c in CUTOFFS:
            if not at_risk(t["endpoint"], c):
                continue
            pname = f"{t['test']}_c{c:g}_{fuel}_{'stop' if stop else 'nostop'}.json"
            prefix = fd_prefix(t, c, fuel, stop, sha_c, sha_e)
            write_json(pdir / pname, prefix)
            k = dict(test=t["test"], session=t["session"], part=t["part"], cutoff=c, outcome=outcome_of(t["endpoint"]),
                     carries_stop="programmed_stop" in prefix["known_conditions"])
            cases.append(k)
            for role in roles:
                jobs.append(((t["test"], c, role), RELEASES[family][role], pdir / pname,
                             OUT / "fd" / "artifact" / family / role / pname))
    arts = forward_many(jobs, workers)
    for k in cases:
        a = {role: arts[(k["test"], k["cutoff"], role)] for role in roles}
        k["refusal"] = {role: f"{x['support']['gate']}: {'; '.join(x['support']['reasons'])}" for role, x in a.items()
                        if not admitted(x)}
        if not k["refusal"]:
            k["cif"] = {role: cif_of(x) for role, x in a.items()}
    base = baseline["families"][family]["cutoffs"]

    def summarise(sel, n_tests, cases_file=None):
        out = dict(tests_included=n_tests, tests_with_a_case=len({k["test"] for k in sel}),
                   sessions_with_a_case=len({k["session"] for k in sel}), cases_at_risk=len(sel),
                   cases_by_cutoff={f"{c:g}": sum(1 for k in sel if k["cutoff"] == c) for c in CUTOFFS},
                   cases_carrying_programmed_stop=sum(k["carries_stop"] for k in sel), refused={})
        for role in roles:
            reasons = Counter(re.sub(r"-?\d+(\.\d+)?", "#", k["refusal"][role]) for k in sel if role in k["refusal"])
            out["refused"][role] = dict(cases=sum(reasons.values()),
                                        tests=len({k["test"] for k in sel if role in k["refusal"]}), reasons=dict(reasons))
        answered = [k for k in sel if not k["refusal"]]
        out.update(cases_scored=len(answered), tests_scored=len({k["test"] for k in answered}),
                   sessions_scored=len({k["session"] for k in answered}))
        if not answered:
            return out
        outcomes = [k["outcome"] for k in answered]
        out["endpoints_of_scored_cases"] = dict(Counter((o["causes"][0] if o["causes"] else "right_censored") for o in outcomes))
        cifs = {role: {c: [k["cif"][role][c] for k in answered] for c in CAUSES} for role in roles}
        cifs["baseline"] = {c: [base[f"{k['cutoff']:g}"][c] for k in answered] for c in CAUSES}
        sc = {m: score(outcomes, cifs[m]) for m in cifs}
        out["integrated_brier"] = {m: {c: sc[m][c]["integrated_brier"] for c in CAUSES} for m in sc}
        out["brier_by_horizon_extinction"] = {m: [round(q["brier"], 5) for q in sc[m]["extinction"]["brier"]] for m in sc}
        out["censored_before_horizon"] = [q["censored_before_horizon"] for q in sc["p1"]["extinction"]["brier"]]
        sessions = [k["session"] for k in answered]
        if cases_file:  # per-case contributions, for audit. THIS FILE HOLDS OUTCOMES PER TEST.
            cases_file.parent.mkdir(parents=True, exist_ok=True)
            with cases_file.open("w", encoding="utf-8", newline="") as f:
                w = csv.writer(f)
                w.writerow(["test", "session", "days", "cutoff_s", "censoring", "end_s", "cause", "carries_programmed_stop"]
                           + [f"ibs_contrib[{c}]:{m}" for m in sc for c in CAUSES])
                for i, k in enumerate(answered):
                    o = k["outcome"]
                    w.writerow([k["test"], k["session"], k["part"], f"{k['cutoff']:g}", o["censoring"],
                                _or(o["t_upper"], o["t_lower"]), o["causes"][0] if o["causes"] else "", int(k["carries_stop"])]
                               + [f"{sc[m][c]['contrib'][i]:.6f}" for m in sc for c in CAUSES])
            out["cases_file"] = cases_file.relative_to(REPO).as_posix()
        out["paired"] = {
            f"p1_minus_{other}": {c: boot([a - b for a, b in zip(sc["p1"][c]["contrib"], sc[other][c]["contrib"])], sessions)
                                  for c in CAUSES} for other in ("baseline", "carry")}
        return out

    block = dict(name=name, family=family, fuel_handed_to_forward=fuel, planned_stop=bool(stop),
                 **summarise(cases, len(tests), OUT / "fd" / "cases" / f"{tag}_{family}_{fuel}_{'stop' if stop else 'nostop'}.csv"))
    block["by_days"] = {part: summarise([k for k in cases if k["part"] == part], sum(1 for t in tests if t["part"] == part))
                        for part in ("train_days", "validation_days")}
    return block


def verdict(block):
    """The rule of the protocol for arm B, on the extinction score of P1 against the incidence baseline."""
    if "paired" not in block or "p1_minus_baseline" not in block["paired"]:
        return dict(verdict="not computed")
    p = block["paired"]["p1_minus_baseline"]["extinction"]
    p1, b = block["integrated_brier"]["p1"]["extinction"], block["integrated_brier"]["baseline"]["extinction"]

    def rule(ci):
        if ci is None:
            return "undecided"
        if p1 < b and ci[1] < 0:
            return "carry over to ethylene as a surrogate"
        if p1 > b and ci[0] > 0:
            return "do not"
        return "undecided"
    return dict(p1_extinction=p1, baseline_extinction=b, difference=p["case_mean"], ci95=p["ci95_case_mean"],
                verdict=rule(p["ci95_case_mean"]),
                with_the_evaluators_session_equal_interval=dict(difference=p["session_equal_mean"],
                                                                ci95=p["ci95_session_equal_mean"],
                                                                verdict=rule(p["ci95_session_equal_mean"])))


def cmd_fd(args):
    selftest()
    if not BASELINE.exists():
        sys.exit("the incidence baseline is not frozen: run `cfig` first")
    binary = use_binary(args)
    proven = []
    for q in sorted((OUT / "cfig").glob("chain_check*.json")):
        d = json.loads(q.read_text(encoding="utf-8"))
        if d.get("reproduced_within_tolerance") and d.get("ignis", {}).get("sha256") == binary["sha256"]:
            proven.append(q.name)
    if not proven:
        sys.exit("no chain check reproduced the CFI-G train numbers with this binary: nothing is run on Flame Design")
    baseline = json.loads(BASELINE.read_text(encoding="utf-8"))
    tests, sha_c, sha_e, dropped = fd_tests()
    assert not any(t["part"] == "test_days" for t in tests)
    rep = dict(purpose="PROTOCOL 11.1: frozen CFI-G burner-flame models on ACME Flame Design (PSI-10)",
               ignis=binary, chain_check=proven, releases=RELEASES, horizons_s=HORIZONS, cutoffs_s=CUTOFFS, settings=SETTINGS,
               baseline=dict(file=str(BASELINE.relative_to(REPO)), sha256=hashlib.sha256(BASELINE.read_bytes()).hexdigest()),
               inputs=dict(conditions_csv_sha256=sha_c, endpoints_csv_sha256=sha_e),
               bootstrap=dict(resamples=N_BOOT, seed=BOOT_SEED, unit="session", level=0.95),
               amendment="tests of the days frozen as TEST in docs/wsp/atlas/splits/PSI-10.json are dropped before "
                         "measurement and scoring; `by_days` gives the same table for the train and the validation days",
               dropped_test_days=dropped, tests={}, arm_a={}, arm_b={})
    for family in ("acme_normal", "acme_inverse"):
        tt = [t for t in tests if t["family"] == family]
        rep["tests"][family] = dict(
            included=len(tt), sessions=len({t["session"] for t in tt}), by_fuel=dict(Counter(t["fuel_species"] for t in tt)),
            by_days={part: dict(tests=sum(1 for t in tt if t["part"] == part),
                                sessions=len({t["session"] for t in tt if t["part"] == part}),
                                by_fuel=dict(Counter(t["fuel_species"] for t in tt if t["part"] == part)))
                     for part in ("train_days", "validation_days")},
            with_endpoint=sum(t["endpoint"] is not None for t in tt),
            without_endpoint=dict(Counter(t["why"] for t in tt if t["endpoint"] is None)),
            endpoint_kind=dict(Counter((t["endpoint"]["allowed_causes"] or ["right_censored"])[0] for t in tt if t["endpoint"])),
            onset_method=dict(Counter(t["onset_method"] for t in tt if t["endpoint"])),
            with_programmed_stop=sum(1 for t in tt if t["programmed_stop"] == t["programmed_stop"] and t["programmed_stop"] > 0),
            endpoint_flags=dict(Counter(f for t in tt for f in t["eflags"].split("|") if f)))
        eth = [t for t in tt if t["fuel_species"] == "ethylene"]
        if family == "acme_normal":
            prop = [t for t in tt if t["include_unchanged"] == "1"]
            for stop in (False, True):
                b = fd_block("arm A, propane tests of session 21301, unchanged models", "armA_propane", family, prop, "recorded", stop,
                             baseline, sha_c, sha_e, args.workers)
                rep["arm_a"][f"{family}/propane/{'stop' if stop else 'nostop'}"] = b
        b = fd_block("arm A, ethylene tests with the fuel as recorded: refusals", "armA_ethylene", family, eth, "recorded", False,
                     baseline, sha_c, sha_e, args.workers)
        keys = ("tests_included", "tests_with_a_case", "sessions_with_a_case", "cases_at_risk", "cases_scored", "refused")
        rep["arm_a"][f"{family}/ethylene_as_recorded/nostop"] = dict(
            name=b["name"], family=family, fuel_handed_to_forward="recorded", **{k: b[k] for k in keys},
            by_days={part: {k: v[k] for k in keys} for part, v in b["by_days"].items()})
        for fuel in SURROGATE:
            for stop in (False, True):
                b = fd_block(f"arm B, ethylene tests with the indicators of {fuel}", "armB", family, eth, fuel, stop,
                             baseline, sha_c, sha_e, args.workers)
                rep["arm_b"][f"{family}/{fuel}/{'stop' if stop else 'nostop'}"] = b
        rep.setdefault("arm_b_verdict", {})[family] = verdict(rep["arm_b"][f"{family}/ethane/nostop"])
        rep.setdefault("arm_b_verdict_other_variants", {})[family] = {
            key.split("/", 1)[1]: verdict(blk) for key, blk in rep["arm_b"].items()
            if key.startswith(family) and not key.endswith("ethane/nostop")}
    write_json(OUT / "fd" / "report.json", rep)
    print(f"wrote {(OUT / 'fd' / 'report.json').relative_to(REPO)}; `fd-tables` prints it")
    return 0


def cmd_fd_tables(_args):
    """The report of `fd` as plain-text tables."""
    rep = json.loads((OUT / "fd" / "report.json").read_text(encoding="utf-8"))
    f4 = lambda x: "   n/a" if x is None or x != x else f"{x:.4f}"  # noqa: E731
    ci = lambda c: "one session" if c is None else f"[{c[0]:+.4f}, {c[1]:+.4f}]"  # noqa: E731
    for fam, t in rep["tests"].items():
        print(f"{fam}: included {t['included']} tests in {t['sessions']} sessions, fuel {t['by_fuel']}, with endpoint "
              f"{t['with_endpoint']} {t['endpoint_kind']}, without {t['without_endpoint']}, onset {t['onset_method']}, "
              f"with programmed stop {t['with_programmed_stop']}, flags {t['endpoint_flags']}; by days {t['by_days']}")
    def part(label, b):
        print(f"   -- {label}: tests {b['tests_included']}, with a case {b['tests_with_a_case']} in "
              f"{b['sessions_with_a_case']} sessions; cases at risk {b['cases_at_risk']}; scored {b['cases_scored']} cases"
              + (f" of {b['tests_scored']} tests in {b['sessions_scored']} sessions" if "tests_scored" in b else "")
              + (f"; by cutoff {b['cases_by_cutoff']}; carrying the stop {b['cases_carrying_programmed_stop']}"
                 if "cases_by_cutoff" in b else ""))
        for role, r in b["refused"].items():
            if r["cases"]:
                print(f"      refused by {role}: {r['cases']} cases of {r['tests']} tests: {r['reasons']}")
        if "integrated_brier" not in b:
            return
        print(f"      endpoints of scored cases: {b['endpoints_of_scored_cases']}")
        print("      model       extinction  operator_termination")
        for m, v in b["integrated_brier"].items():
            print(f"      {m:<10}  {f4(v['extinction'])}      {f4(v['operator_termination'])}")
        for name, v in b["paired"].items():
            for c in CAUSES:
                x = v[c]
                print(f"      {name} [{c}]: case mean {x['case_mean']:+.4f} {ci(x['ci95_case_mean'])}; "
                      f"session-equal mean {x['session_equal_mean']:+.4f} {ci(x['ci95_session_equal_mean'])} "
                      f"({x['n_cases']} cases, {x['n_sessions']} sessions)")

    print(f"dropped (days frozen as test, not measured, not scored): {rep['dropped_test_days']}")
    for arm in ("arm_a", "arm_b"):
        for key, b in rep[arm].items():
            print(f"\n== {arm} {key}: {b['name']}")
            part("all remaining tests", b)
            for label, v in b["by_days"].items():
                part(label, v)
    print("\nArm B verdict (ethane indicators, no planned stop; all remaining tests):")
    for fam, v in rep["arm_b_verdict"].items():
        print(f"   {fam}: {json.dumps(v)}")
    print("Arm B, the rule on the other variants (not the registered primary):")
    for fam, d in rep["arm_b_verdict_other_variants"].items():
        for k, v in d.items():
            print(f"   {fam} {k}: {v.get('verdict')} (difference {v.get('difference')}, ci {v.get('ci95')})")
    return 0


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    sub = ap.add_subparsers(dest="cmd", required=True)
    p = sub.add_parser("cfig")
    p.add_argument("--rust-prefix", help="a prefix.json the Rust path stored for a TRAIN case: compared with the one built here")
    p.add_argument("--tag", help="keep this run apart (artifact_<tag>, chain_check_<tag>.json) and compare its "
                                 "artifact ids with the untagged run")
    p.set_defaults(fn=cmd_cfig)
    p = sub.add_parser("fd-fetch")
    p.add_argument("--execute", action="store_true", help="download (default: dry run)")
    p.add_argument("--batch", type=int, default=10, help="tests per fetch command")
    p.add_argument("--retries", type=int, default=3, help="repeats of a fetch command the server cut short")
    p.set_defaults(fn=cmd_fd_fetch)
    sub.add_parser("cfig-endpoints").set_defaults(fn=cmd_cfig_endpoints)
    sub.add_parser("fd-tests-file").set_defaults(fn=cmd_fd_tests_file)
    sub.add_parser("fd").set_defaults(fn=cmd_fd)
    sub.add_parser("fd-tables").set_defaults(fn=cmd_fd_tables)
    for q in sub.choices.values():
        q.add_argument("--workers", type=int, default=max(2, (os.cpu_count() or 4) - 2))
        q.add_argument("--ignis", help="path of the ignis binary, relative to the repository (default target/release)")
    args = ap.parse_args()
    return args.fn(args)


if __name__ == "__main__":
    sys.exit(main())
