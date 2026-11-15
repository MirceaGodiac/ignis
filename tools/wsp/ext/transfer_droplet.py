#!/usr/bin/env python3
"""The droplet scoring chain for transfer tests of the frozen FLEX droplet models (PSI-69, methanol and
n-heptane) on sibling droplet investigations (CFI, PSI-39; FLEX-2, PSI-68). The droplet analogue of
`tools/wsp/ext/transfer_acme.py`. This tool fits nothing, searches nothing and changes no model or
existing file. It writes only under `runs/dev/data_all/work/transfer_droplet/`.

The chain: a PrefixBundle file per (test, cutoff) -> `ignis wsp forward` (the frozen release,
unchanged) -> the quantile forecast of the liquid diameter -> the evaluator's own case metrics, ported
line for line below from `crates/ignis-lab/src/eval/scoring.rs` (`common_spec`, `admitted`,
`forecast_metrics`), `scores.rs` (`crps_quantiles`, `central_intervals`, `lpd_quantiles`, `pinball`)
and `util.rs` (`interp`, `pairwise_sum`, `mean`, `fmt_g`).

The prefix is `ignis_core::analysis::build_run_prefix` with the droplet experts' policy (droplet3.rs
and droplet6.rs: allowed conditions x_o2, x_co2, x_he, pressure, fuel; refused: ALWAYS_FORBIDDEN of
prefix.rs): every observation taken and available at or before the cutoff, sorted by (time, id); no
contour, no control; `phase_established_at_s` = 0 when the run is at risk
(`phase_established_at`: the endpoint's terminal/confirmed times when the run has an endpoint, else
the last valid observation at or after the cutoff). Settings: `{"horizon_s": 10}` as
`tools/wsp/dev_eval.py` and `wsp-dev feedback` use for FLEX (cutoffs 1, 3, 5 s).

Baselines computed by the chain itself:
  * persistence: the last valid liquid diameter in the prefix (time <= cutoff, value > 0) carried
    forward as a point mass; its CRPS is the absolute error (exact CRPS of a point mass), so CRPS =
    MAE of the median; band coverage is not defined for a point mass and is not reported.
  * the D-squared law: the hand-written reference releases `baseline-d2` that the FLEX tables carry
    (wsp2 and wsp5 of each fuel), run through the same chain.

Subcommands:

  flex-check   The chain check on FLEX TRAIN only. Two prefix paths are run: (a) `replica`, the
               stored train observations and conditions copied into the prefix as build_run_prefix
               does; (b) `series`, the same train runs exported to the generic series format below
               and rebuilt by the generic builder that an external test uses. Both are scored and
               compared per case with a `wsp-dev feedback --partition train` report of the same
               releases (run first by this command, with the same binaries).
  score        The generic path: `--cases <cases.jsonl>` (format below) and `--releases a,b,...`
               (release ids, or `flex` for the FLEX set below): prefixes, forwards, metrics per case
               and aggregates, persistence beside them. Writes under `--out-dir`. (Not run on CFI or
               FLEX-2 in this commit.)

A row of `cases.jsonl` (one external test):
  {"run_id": "PSI-39:<test>", "session_id": "PSI-39:day:<date>", "investigation": "PSI-39",
   "conditions": {"x_o2": {"value": 0.21, "unit": "1", "source": "...", "evidence": "original_observation"},
                  "x_co2": ..., "x_he": ..., "pressure": {"value": 101325.0, "unit": "Pa", ...},
                  "fuel": {"value": "Heptane", "unit": null, ...}},
   (a condition may carry "resolution": "unresolved_conflict" | "unknown"; the droplet releases
    refuse such a condition, as they refuse it in FLEX)
   "observations": [{"quantity": "liquid_diameter", "time_s": 0.1, "value_si": 0.00301,
                     "quality_flags": [], "source": "<file>", "row": 12}, ...],
   "endpoint": null | {"first_terminal_s": .., "last_confirmed_active_s": .., "censoring": .., "allowed_causes": [..]},
   "time_origin": optional, default the FLEX time origin}
The releases refuse a prefix whose `time_origin` differs from theirs (applicability.time_origin =
the FLEX string). An external series is therefore handed over under the FLEX time origin, which is
a declared equivalence of time bases; the report of a transfer test has to say how that time base
was established for the investigation.

    python tools/wsp/ext/transfer_droplet.py flex-check
    python tools/wsp/ext/transfer_droplet.py score --cases <cases.jsonl> --releases flex --out-dir <dir>
"""
import argparse
import bisect
import hashlib
import json
import math
import os
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(REPO / "tools/wsp/analysis"))
sys.path.insert(0, str(Path(__file__).resolve().parent))
from transfer_acme import canonical, content_hash, ryu  # noqa: E402  (serde_json canonical form, shared)

OUT = REPO / "runs/dev/data_all/work/transfer_droplet"
RECORDS = REPO / "data/work/records/flex"
EXE = REPO / "target/release" / ("ignis.exe" if os.name == "nt" else "ignis")
WSPDEV = REPO / "target/release" / ("wsp-dev.exe" if os.name == "nt" else "wsp-dev")

FLEX_TIME_ORIGIN = "NASA DropletData processed time base (phase entry not adjudicated)"
CUTOFFS = [1.0, 3.0, 5.0]
HORIZON_S = 10.0
# what `ForecastSettings { horizon_s: Some(10), ..Default }.value()` serialises to (times_s skipped)
SETTINGS = {"seed": 0, "quantile_levels": None, "horizon_s": HORIZON_S, "n_points": None}
ALLOWED = ["x_o2", "x_co2", "x_he", "pressure", "fuel"]
ALWAYS_FORBIDDEN = ["k_full", "burning_rate_full", "final_diameter", "extinction_diameter", "duration",
                    "burn_time", "end_time", "final_label", "endpoint", "max_burner_temperature",
                    "peak_temperature", "post_test_composition", "post_test_pressure", "outcome",
                    "publication_outcome"]
INVALID_FLAGS = ["non_positive", "non_finite", "missing", "duplicate_timestamp", "non_monotonic_time"]
SUPPORT_EVIDENCE = ["activity confirmed at the cutoff by the run record"]
NAN = float("nan")

# The FLEX releases of the chain check and of a transfer test: (fuel, role) -> release folder.
RELEASE_DIRS = {
    ("methanol", "adopted_wsp2_d3"): "configs/models/flex/wsp2/methanol/d3-fable",
    ("methanol", "carry_wsp5_d3"): "configs/models/flex/wsp5/methanol/carry-d3",
    ("methanol", "d2law_wsp2"): "configs/models/flex/wsp2/methanol/baseline-d2",
    ("methanol", "d2law_wsp5"): "configs/models/flex/wsp5/methanol/baseline-d2",
    ("heptane", "adopted_wsp2_d2"): "configs/models/flex/wsp2/heptane/d2-fable",
    ("heptane", "carry_wsp5_d2"): "configs/models/flex/wsp5/heptane/carry-d2",
    ("heptane", "d2law_wsp2"): "configs/models/flex/wsp2/heptane/baseline-d2",
    ("heptane", "d2law_wsp5"): "configs/models/flex/wsp5/heptane/baseline-d2",
}
SPLIT_NAME = {"methanol": "flex", "heptane": "flex_heptane"}  # keys of tools/wsp/analysis/lib.py SPLITS
TOLERANCE = 1e-9


def releases_of(fuel):
    out = {}
    for (f, role), d in RELEASE_DIRS.items():
        if f == fuel:
            r = json.loads((REPO / d / "release.json").read_text(encoding="utf-8"))
            out[role] = dict(id=r["id"], model_name=r["model_name"], split_id=r["split_id"], dir=d)
    return out


# ---------------------------------------------------------------------------------------------
# the evaluator's quantile metrics, ported (crates/ignis-lab/src/eval/{scoring,scores}.rs, util.rs)
# ---------------------------------------------------------------------------------------------
def pairwise_sum(a):
    n = len(a)
    if n < 8:
        s = 0.0
        for x in a:
            s += x
        return s
    if n <= 128:
        r = list(a[:8])
        i = 8
        while i + 8 <= n:
            for k in range(8):
                r[k] += a[i + k]
            i += 8
        res = ((r[0] + r[1]) + (r[2] + r[3])) + ((r[4] + r[5]) + (r[6] + r[7]))
        while i < n:
            res += a[i]
            i += 1
        return res
    n2 = n // 2
    n2 -= n2 % 8
    return pairwise_sum(a[:n2]) + pairwise_sum(a[n2:])


def mean(v):
    return NAN if not v else pairwise_sum(v) / len(v)


def interp(x, xp, fp):
    """`util::interp` (np.interp semantics, clamped at the ends)."""
    n = len(xp)
    if n == 0 or x != x:
        return NAN
    if n == 1:
        return fp[0]
    if x <= xp[0]:
        return fp[0]
    if x >= xp[n - 1]:
        return fp[n - 1]
    j = bisect.bisect_right(xp, x) - 1
    if x == xp[j]:
        return fp[j]
    slope = (fp[j + 1] - fp[j]) / (xp[j + 1] - xp[j])
    r = slope * (x - xp[j]) + fp[j]
    if r != r:
        r2 = slope * (x - xp[j + 1]) + fp[j + 1]
        if r2 != r2 and fp[j] == fp[j + 1]:
            return fp[j]
        return r2
    return r


def interp_row(times, row, t):
    """`scoring::interp_row`: NaN outside the forecast range."""
    return [NAN if (not times or x < times[0] or x > times[-1]) else interp(x, times, row) for x in t]


def pinball(q, y, tau):
    return (tau - (1.0 if y < q else 0.0)) * (y - q)


def sort_levels(levels, q):
    o = sorted(range(len(levels)), key=lambda i: levels[i])
    return [levels[i] for i in o], [q[i] for i in o]


def crps_quantiles(levels, q, y):
    if not levels or not math.isfinite(y) or any(not math.isfinite(v) for v in q):
        return NAN
    t, qq = sort_levels(levels, q)
    ext = [0.0] + t + [1.0]
    s = 0.0
    for k in range(len(t)):
        wk = (ext[k + 2] - ext[k]) / 2.0
        s += wk * pinball(qq[k], y, t[k])
    return 2.0 * s


def central_intervals(levels, q, y):
    out = []
    for i, lo in enumerate(levels):
        if lo >= 0.5:
            continue
        for j, hi in enumerate(levels):
            if abs(hi - (1.0 - lo)) < 1e-9:
                a, b = q[i], q[j]
                out.append((round((1.0 - 2.0 * lo) * 1e9) / 1e9, a <= y <= b, b - a))
    return out


def lpd_quantiles(levels, q, y):
    t, qq = sort_levels(levels, q)
    if len(t) < 2 or not math.isfinite(y):
        return NAN
    gaps = [qq[i + 1] - qq[i] for i in range(len(qq) - 1)]
    if any(not (g > 0.0) for g in gaps):
        return NAN
    n = len(t)
    if y < qq[0]:
        s = gaps[0]
        return math.log(t[0] / s) - (qq[0] - y) / s
    if y > qq[n - 1]:
        s = gaps[n - 2]
        return math.log((1.0 - t[n - 1]) / s) - (y - qq[n - 1]) / s
    k = max(0, min(bisect.bisect_right(qq, y) - 1, n - 2))
    return math.log((t[k + 1] - t[k]) / gaps[k])


def fmt_g(x):
    """Python f"{x:g}", as `util::fmt_g`."""
    return f"{x:g}"


def admitted(a):
    """`scoring::admitted`: (bool, reason)."""
    s = a["support"]
    if not s["admitted"]:
        return False, f"support gate {s['gate']}: " + "; ".join(s.get("reasons", []))
    if a["status"] in ("unsupported", "failed", "insufficient_inference"):
        return False, f"status {a['status']}"
    return True, None


def common_spec(artifacts):
    """`scoring::common_spec` for quantity forecasts (no event horizons)."""
    caps = [a for a in artifacts if admitted(a)[0]]
    if not caps:
        return dict(levels=[], crps=None, lpd=None)
    pf = [a for a in caps if a["kind"] == "prefix_forecast"]
    level_sets = [a["quantile_levels"] for a in pf]
    levels = sorted({x for x in level_sets[0] if all(x in s for s in level_sets)}) if level_sets else []
    samples = all("samples" in (a.get("diagnostics") or {}) for a in pf)
    preds = {((a.get("diagnostics") or {}).get("predictive") or {}).get("family") if isinstance(
        (a.get("diagnostics") or {}).get("predictive"), dict) else None for a in pf}
    crps = "samples" if samples and level_sets else ("quantiles" if levels else None)
    lpd = "exact" if len(preds) == 1 and None not in preds else ("quantile_piecewise" if len(levels) >= 2 else None)
    if crps == "samples" or lpd == "exact":
        raise NotImplementedError(f"spec crps={crps} lpd={lpd}: the droplet artifacts carried quantiles only "
                                  "when this port was written; port crps_samples / lpd_* before using it")
    return dict(levels=levels, crps=crps, lpd=lpd)


def forecast_metrics(a, cont, spec):
    """`scoring::forecast_metrics` -> (metrics, n_points, reason)."""
    if not a["times_s"]:
        return {}, 0, "artifact has no forecast times"
    times = a["times_s"]
    t0, t1 = times[0], times[-1]
    q = a.get("quantity")
    cutoff = cont["cutoff_s"]
    t, y = [], []
    for o in cont["observations"]:
        v = o.get("value_si")
        if v is None:
            continue
        oq = o.get("quantity")
        qok = q is None or oq is None or oq == q
        ts = o["time_s"]
        if o.get("valid", True) and qok and ts > cutoff and t0 <= ts <= t1:
            t.append(ts)
            y.append(v)
    if not t:
        return {}, 0, "no valid continuation observation inside the forecast horizon"
    n = len(t)
    m = {}
    levels = a["quantile_levels"]
    if levels:
        qi = [interp_row(times, r, t) for r in a["quantiles"]]
        use = [levels.index(lv) for lv in spec["levels"] if lv in levels]
        lv = [levels[i] for i in use]
        col = lambda j: [qi[i][j] for i in use]  # noqa: E731
        if 0.5 in levels:
            mi = levels.index(0.5)
            m["mae_median"] = mean([abs(qi[mi][j] - y[j]) for j in range(n)])
        if spec["crps"] == "quantiles":
            m["crps"] = mean([crps_quantiles(lv, col(j), y[j]) for j in range(n)])
        if spec["lpd"] == "quantile_piecewise":
            m["lpd"] = mean([lpd_quantiles(lv, col(j), y[j]) for j in range(n)])
        cov = {}
        for j in range(n):
            for nominal, covered, width in central_intervals(lv, col(j), y[j]):
                cov.setdefault(fmt_g(nominal), []).append((covered, width))
        for k, vals in cov.items():
            m[f"coverage@{k}"] = mean([1.0 if c else 0.0 for c, _ in vals])
            m[f"width@{k}"] = mean([w for _, w in vals])
    if a.get("mean"):
        m["mae_mean"] = mean([abs(interp(t[j], times, a["mean"]) - y[j]) for j in range(n)])
    return m, n, None


def score_case(a, cont, spec):
    """`scoring::score_case` for a prefix forecast: (metrics, scored, reason)."""
    ok, why = admitted(a)
    if not ok:
        return {}, False, why
    to = cont.get("time_origin") or None
    ao = (a.get("diagnostics") or {}).get("time_origin") or a.get("time_origin") or None
    if to and ao and to != ao:
        return {}, False, f"time origin differs: artifact {ao}, truth {to}"
    if a["kind"] not in ("prefix_forecast", "metrology"):
        return {}, False, f"kind {a['kind']} has no scorer"
    if (a.get("diagnostics") or {}).get("front_mean_m") is not None:
        return {}, False, "front artifact: not a droplet forecast"
    m, _, why = forecast_metrics(a, cont, spec)
    return m, why is None, why


# ---------------------------------------------------------------------------------------------
# prefix and continuation (ignis_core::analysis / prefix.rs, ignis_lab::eval::deposit)
# ---------------------------------------------------------------------------------------------
def measurement_valid(o):
    v = o.get("value_si")
    return v is not None and math.isfinite(v) and not any(f in INVALID_FLAGS for f in o.get("quality_flags", []))


def phase_established(endpoint, observations, cutoff):
    """`analysis::phase_established_at` -> True (phase entry 0.0) or False."""
    if not (math.isfinite(cutoff) and cutoff >= 0.0):
        return False
    terminal = (endpoint or {}).get("first_terminal_s")
    confirmed = (endpoint or {}).get("last_confirmed_active_s")
    if terminal is not None and terminal <= cutoff:
        return False
    if confirmed is not None:
        return confirmed >= cutoff
    if terminal is not None:
        return True
    last = max((o["time_s"] for o in observations if measurement_valid(o)), default=-math.inf)
    return last >= cutoff


def build_prefix(run_id, session_id, conditions, observations, endpoint, cutoff, time_origin):
    """`analysis::build_run_prefix` + `prefix::build_prefix` with the droplet policy. `observations`
    must be the run's stored order (time, id). Returns the bundle, or raises ValueError (a refusal of
    the prefix builder: a terminal condition in the record)."""
    bad = [k for k in conditions if k in ALWAYS_FORBIDDEN]
    if bad:
        raise ValueError(f"prefix refused: condition {bad[0]!r} is terminal/full-history and cannot be prospective")
    phase = phase_established(endpoint, observations, cutoff)
    obs = [o for o in observations if o["available_at_s"] <= cutoff and o["time_s"] <= cutoff]
    obs.sort(key=lambda o: (o["time_s"], o["id"]))
    bundle = dict(family="flex", run_id=run_id, session_id=session_id, cutoff_s=float(cutoff),
                  time_origin=time_origin, known_conditions={k: conditions[k] for k in sorted(conditions) if k in ALLOWED},
                  known_controls=[], observations=obs, contours=[],
                  phase_established_at_s=0.0 if phase else None,
                  support_evidence=list(SUPPORT_EVIDENCE) if phase else [], transform_versions={})
    bundle["input_hash"] = content_hash(bundle, ("input_hash",))
    return bundle


def continuation(run_id, session_id, observations, endpoint, cutoff, time_origin):
    """`deposit::continuation_payload` (observations after the cutoff, with their validity)."""
    p = dict(run_id=run_id, session_id=session_id, time_origin=time_origin, cutoff_s=cutoff,
             observations=[dict(id=o["id"], time_s=o["time_s"], value_si=o.get("value_si"), quantity=o["quantity"],
                                valid=measurement_valid(o)) for o in observations if o["time_s"] > cutoff])
    if endpoint is not None:
        p["endpoint"] = dict(censoring=endpoint.get("censoring"), t_lower_s=endpoint.get("last_confirmed_active_s"),
                             t_upper_s=endpoint.get("first_terminal_s"), allowed_causes=endpoint.get("allowed_causes"))
    return p


def persistence(prefix, cont):
    """Last valid liquid diameter of the prefix (value > 0) carried forward over (cutoff, cutoff + 10 s].
    Point mass: CRPS = absolute error. Returns dict(mae, n) or None when not defined."""
    last = None
    for o in prefix["observations"]:
        if o["quantity"] == "liquid_diameter" and measurement_valid(o) and o["value_si"] > 0.0:
            last = o["value_si"]
    if last is None:
        return None
    c = prefix["cutoff_s"]
    e = [abs(last - o["value_si"]) for o in cont["observations"]
         if o["valid"] and o["quantity"] == "liquid_diameter" and o["value_si"] is not None
         and c < o["time_s"] <= c + HORIZON_S]
    return dict(crps=mean(e), mae_median=mean(e), n_points=len(e)) if e else None


# ---------------------------------------------------------------------------------------------
# the generic series format (an external test) -> stored-layout records
# ---------------------------------------------------------------------------------------------
def ext_measured(c, key):
    """A condition of a cases.jsonl row as an `ignis_core::contracts::Measured`."""
    v = c["value"]
    num = isinstance(v, (int, float)) and not isinstance(v, bool)
    return dict(
        derivation=dict(agent=None, inputs=[], method="deterministic", method_id=f"wsp.ext.droplet.cond.{key}",
                        method_version="1", notes=c.get("note")),
        evidence_class=c.get("evidence", "original_observation"), flags=list(c.get("flags", [])),
        locator=dict(asset_sha256=c.get("sha256", "0" * 64), char_end=None, char_start=None, column=c.get("column"),
                     frame=None, page=None, path=c.get("source"), pts_s=None, row=c.get("row")),
        raw_unit=c.get("unit"), raw_value=float(v) if num else v, resolution=c.get("resolution", "resolved"),
        uncertainty_reason="not reported by source" if num else None, uncertainty_si=None,
        unit_si=c.get("unit"), value_si=float(v) if num else None)


def ext_observations(row):
    """Observations of a cases.jsonl row in the stored FLEX layout, in stored order (time, id)."""
    out = []
    for i, o in enumerate(row["observations"]):
        q = o.get("quantity", "liquid_diameter")
        v = o.get("value_si")
        out.append(dict(
            available_at_s=float(o.get("available_at_s", o["time_s"])), calibration_id=None, channel=o.get("channel"),
            derivation=dict(agent=None, inputs=[], method="deterministic", method_id=f"wsp.ext.droplet.obs.{q}",
                            method_version="1", notes=o.get("note")),
            evidence_class=o.get("evidence", "source_derived"),
            id=f"{row['run_id']}/{o.get('channel') or 'series'}/{q}/{o.get('row', i)}",
            locator=dict(asset_sha256=o.get("sha256", "0" * 64), char_end=None, char_start=None, column=o.get("column"),
                         frame=None, page=None, path=o.get("source"), pts_s=None, row=o.get("row", i)),
            quality_flags=list(o.get("quality_flags", [])), quantity=q, raw_unit=o.get("raw_unit", "m"),
            raw_value=o.get("raw_value", v), run_id=row["run_id"], time_s=float(o["time_s"]), uncertainty_si=None,
            unit_si="m", value_si=None if v is None else float(v)))
    out.sort(key=lambda o: (o["time_s"], o["id"]))
    return out


def ext_case_inputs(row):
    cond = {k: ext_measured(c, k) for k, c in row["conditions"].items()}
    return dict(run_id=row["run_id"], session_id=row.get("session_id"), conditions=cond,
                observations=ext_observations(row), endpoint=row.get("endpoint"),
                time_origin=row.get("time_origin", FLEX_TIME_ORIGIN))


# ---------------------------------------------------------------------------------------------
# the forward
# ---------------------------------------------------------------------------------------------
def sha256_file(p):
    h = hashlib.sha256()
    with Path(p).open("rb") as f:
        for b in iter(lambda: f.read(1 << 20), b""):
            h.update(b)
    return h.hexdigest()


def use_binaries(args):
    global EXE, WSPDEV
    if getattr(args, "ignis", None):
        EXE = (REPO / args.ignis).resolve()
    if getattr(args, "wsp_dev", None):
        WSPDEV = (REPO / args.wsp_dev).resolve()
    for b in (EXE, WSPDEV):
        if not b.exists():
            sys.exit(f"{b} does not exist (this tool never builds a binary)")
    return {b.name: dict(path=str(b.relative_to(REPO)).replace("\\", "/"), sha256=sha256_file(b)) for b in (EXE, WSPDEV)}


def run_forward(release, prefix_path, out_path, retries=3):
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
    for attempt in range(retries):
        r = subprocess.run(cmd, cwd=REPO, capture_output=True, text=True)
        if r.returncode == 0:
            return json.loads(out_path.read_text(encoding="utf-8"))
        if attempt + 1 < retries:  # another agent may be rebuilding the binary
            import time
            time.sleep(30)
    raise RuntimeError(f"ignis wsp forward failed for {release[:15]} on {prefix_path.name}: "
                       f"{(r.stderr or r.stdout).strip()[-600:]}")


def forward_many(jobs, workers):
    def one(job):
        key, release, pp, op = job
        return key, run_forward(release, pp, op)
    with ThreadPoolExecutor(max_workers=workers) as ex:
        return dict(ex.map(one, jobs))


def write_json(path, obj, indent=1):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(obj, indent=indent, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")


def run_cases(case_inputs, releases, workdir, workers):
    """case_inputs: list of dict(run_id, session_id, conditions, observations, endpoint, time_origin).
    releases: {role: release id}. Returns (rows, not_at_risk, prefix_refusals) where each row is a
    case (run, cutoff) with the artifact of every release."""
    rows, not_at_risk, pre_refused, jobs = [], 0, [], []
    for ci in case_inputs:
        safe = ci["run_id"].replace(":", "_").replace("/", "_")
        for c in CUTOFFS:
            if not phase_established(ci["endpoint"], ci["observations"], c):
                not_at_risk += 1
                continue
            try:
                p = build_prefix(ci["run_id"], ci["session_id"], ci["conditions"], ci["observations"], ci["endpoint"],
                                 c, ci["time_origin"])
            except ValueError as e:
                pre_refused.append(dict(run_id=ci["run_id"], cutoff_s=c, refusal=str(e)))
                continue
            pp = workdir / "prefix" / f"{safe}_c{c:g}.json"
            write_json(pp, p, indent=None)
            row = dict(run_id=ci["run_id"], session_id=ci["session_id"], cutoff_s=c, prefix_path=pp,
                       input_hash=p["input_hash"], persistence=None,
                       cont=continuation(ci["run_id"], ci["session_id"], ci["observations"], ci["endpoint"], c,
                                         ci["time_origin"]))
            row["persistence"] = persistence(p, row["cont"])
            rows.append(row)
            for role, rel in releases.items():
                jobs.append(((ci["run_id"], c, role), rel, pp, workdir / "artifact" / role.replace(":", "__") / f"{safe}_c{c:g}.json"))
    arts = forward_many(jobs, workers)
    for r in rows:
        r["artifacts"] = {role: arts[(r["run_id"], r["cutoff_s"], role)] for role in releases}
    return rows, not_at_risk, pre_refused


def score_rows(rows, releases):
    """Metrics of every case and release, with one spec over every artifact of the set (as wsp-dev
    feedback and `ignis eval compare` do)."""
    spec = common_spec([a for r in rows for a in r["artifacts"].values()])
    for r in rows:
        r["metrics"] = {}
        for role in releases:
            a = r["artifacts"][role]
            m, scored, why = score_case(a, r["cont"], spec)
            r["metrics"][role] = dict(metrics=m, scored=scored, reason=why, admitted=admitted(a)[0],
                                      refusal=None if admitted(a)[0] else admitted(a)[1])
    return spec


def aggregate(rows, role, keys=("crps", "mae_median", "coverage@0.5", "coverage@0.9")):
    cs = [r["metrics"][role]["metrics"] for r in rows if r["metrics"][role]["scored"]]
    out = dict(cases_scored=len(cs), refused=sum(1 for r in rows if not r["metrics"][role]["admitted"]),
               unscored=sum(1 for r in rows if r["metrics"][role]["admitted"] and not r["metrics"][role]["scored"]))
    for k in keys:
        v = [m[k] for m in cs if k in m]
        out[k] = mean(v) if v else None
    return out


# ---------------------------------------------------------------------------------------------
# FLEX train (train rows only)
# ---------------------------------------------------------------------------------------------
def flex_train(fuel):
    """Train archives of a FLEX cohort, read as `wsp-dev`'s train_source does: a row is parsed only
    when its run identity is a train id of the frozen split."""
    import lib
    ids = set(lib.partition_ids(SPLIT_NAME[fuel], "train"))
    exps, obs, ends = {}, {i: [] for i in ids}, {}
    with (RECORDS / "experiments.jsonl").open(encoding="utf-8") as f:
        for line in f:
            r = json.loads(line)
            if r["id"] in ids:
                exps[r["id"]] = r
    assert set(exps) == ids, f"{fuel}: {len(exps)} experiment rows for {len(ids)} train ids"
    keyed = [(f'"run_id": "{i}"', i) for i in ids]
    with (RECORDS / "observations.jsonl").open(encoding="utf-8") as f:
        for line in f:
            k = line.find('"run_id": "')
            if k < 0:
                continue
            rid = line[k + 11:line.find('"', k + 11)]
            if rid in ids:
                o = json.loads(line)
                assert o["run_id"] == rid
                obs[rid].append(o)
    with (RECORDS / "endpoints.jsonl").open(encoding="utf-8") as f:
        for line in f:
            r = json.loads(line)
            if r["run_id"] in ids:
                ends[r["run_id"]] = r
    del keyed
    out = []
    for rid in sorted(ids):
        o = sorted(obs[rid], key=lambda x: (x["time_s"], x["id"]))
        out.append(dict(run_id=rid, session_id=exps[rid].get("session_id"), conditions=exps[rid]["conditions"],
                        observations=o, endpoint=ends.get(rid), time_origin=FLEX_TIME_ORIGIN, record=exps[rid]))
    return out


def to_series_row(ci):
    """A FLEX train run in the generic cases.jsonl format: what an external test would hand over
    (conditions by value and unit, observations by quantity, time, value and flags)."""
    cond = {}
    for k in ALLOWED:
        c = ci["conditions"].get(k)
        if c is None:
            continue
        v = c["value_si"] if c["value_si"] is not None else c["raw_value"]
        cond[k] = dict(value=v, unit=c["unit_si"], resolution=c["resolution"], source=(c.get("locator") or {}).get("path"),
                       column=(c.get("locator") or {}).get("column"), row=(c.get("locator") or {}).get("row"))
    obs = [dict(quantity=o["quantity"], time_s=o["time_s"], available_at_s=o["available_at_s"], value_si=o["value_si"],
                quality_flags=o["quality_flags"], channel=o.get("channel"), source=o["locator"].get("path"),
                column=o["locator"].get("column"), row=o["locator"].get("row"))
           for o in ci["observations"]]
    return dict(run_id=ci["run_id"], session_id=ci["session_id"], investigation="PSI-69", conditions=cond,
                observations=obs, endpoint=ci["endpoint"])


def compare(rows, releases, report):
    """Per-case comparison with a wsp-dev feedback report: every metric of every case."""
    dev = {}
    for rr in report["releases"]:
        dev[rr["release_id"]] = ({(c["run_id"], c["cutoff_s"]): c for c in rr["cases"]},
                                 {(c["run_id"], c["cutoff_s"]): c for c in rr["refused"]})
    out = {}
    for role, rel in releases.items():
        cases, refused = dev[rel]
        mine_cases = {(r["run_id"], r["cutoff_s"]): r["metrics"][role] for r in rows if r["metrics"][role]["admitted"]}
        mine_refused = {(r["run_id"], r["cutoff_s"]) for r in rows if not r["metrics"][role]["admitted"]}
        worst, worst_at, mism = 0.0, None, []
        for key, dc in cases.items():
            mc = mine_cases.get(key)
            if mc is None:
                mism.append(f"{key}: case of wsp-dev missing here")
                continue
            if mc["scored"] != dc["scored"] or set(mc["metrics"]) != set(dc["metrics"]):
                mism.append(f"{key}: scored or metric keys differ")
                continue
            for k, v in dc["metrics"].items():
                d = abs(mc["metrics"][k] - v) if not (v != v and mc["metrics"][k] != mc["metrics"][k]) else 0.0
                if d > worst:
                    worst, worst_at = d, f"{key[0]} c{key[1]:g} {k}"
        extra = set(mine_cases) - set(cases)
        mism += [f"{k}: case here, not in wsp-dev" for k in sorted(extra)]
        if mine_refused != set(refused):
            mism.append(f"refused sets differ: here {len(mine_refused)}, wsp-dev {len(refused)}")
        out[role] = dict(release=rel, cases_wspdev=len(cases), refused_wspdev=len(refused), cases_here=len(mine_cases),
                         refused_here=len(mine_refused), max_abs_diff_any_metric=worst, at=worst_at,
                         mismatches=mism[:20], n_mismatches=len(mism),
                         reproduced=bool(not mism and worst <= TOLERANCE))
    return out


def cmd_flex_check(args):
    binaries = use_binaries(args)
    report = dict(purpose="chain check of tools/wsp/ext/transfer_droplet.py on FLEX TRAIN against wsp-dev feedback",
                  binaries=binaries, settings=SETTINGS, cutoffs_s=CUTOFFS, tolerance=TOLERANCE, fuels={})
    ok = True
    chosen = {}
    for item in (args.releases or "").split(","):
        if item.strip():  # fuel:role=release folder
            fr, d = item.split("=", 1)
            f, role = fr.split(":", 1)
            r = json.loads((REPO / d / "release.json").read_text(encoding="utf-8"))
            chosen.setdefault(f, {})[role] = dict(id=r["id"], model_name=r["model_name"], split_id=r["split_id"], dir=d)
    tag = f"_{args.tag}" if args.tag else ""
    for fuel in (list(chosen) if chosen else ("methanol", "heptane")):
        rels = chosen.get(fuel) or releases_of(fuel)
        ids = {role: r["id"] for role, r in rels.items()}
        work = OUT / "flex" / f"{fuel}{tag}"
        fb = OUT / "flex" / f"wspdev_fb_train_{fuel}{tag}.json"
        cmd = [str(WSPDEV), "feedback", *sum((["--release", r] for r in ids.values()), []), "--partition", "train",
               "--out", str(fb)]
        r = subprocess.run(cmd, cwd=REPO, capture_output=True, text=True)
        if r.returncode != 0:
            sys.exit(f"wsp-dev feedback failed: {r.stderr[-1500:]}")
        dev = json.loads(fb.read_text(encoding="utf-8"))
        train = flex_train(fuel)
        fam = dict(train_runs=len(train), wspdev_runs=dev["runs"], wspdev_not_at_risk=dev["not_at_risk_cases"],
                   wspdev_spec=dev["spec"], releases={k: v["id"] for k, v in rels.items()}, paths={})
        for path in ("replica", "series"):
            if path == "replica":
                inputs = [{k: v for k, v in ci.items() if k != "record"} for ci in train]
            else:
                series = [to_series_row(ci) for ci in train]
                write_json(work / "series_train_cases.json", dict(note="FLEX TRAIN runs in the generic cases format "
                                                                  "(chain check only)", n=len(series)))
                with (work / "series_train_cases.jsonl").open("w", encoding="utf-8", newline="\n") as f:
                    for s in series:
                        f.write(json.dumps(s, ensure_ascii=False) + "\n")
                inputs = [ext_case_inputs(s) for s in series]
            rows, nar, pre_ref = run_cases(inputs, ids, work / path, args.workers)
            spec = score_rows(rows, ids)
            cmpd = compare(rows, ids, dev)
            per = dict(not_at_risk=nar, prefix_refusals=len(pre_ref), cases=len(rows), spec=spec,
                       spec_equal_to_wspdev=(spec["levels"] == dev["spec"]["levels"] and spec["crps"] == dev["spec"]["crps"]
                                             and spec["lpd"] == dev["spec"]["lpd"]),
                       not_at_risk_equal=nar == dev["not_at_risk_cases"], comparison=cmpd,
                       aggregates={role: aggregate(rows, role) for role in ids})
            pers = [r["persistence"] for r in rows if r["persistence"]]
            common = [r for r in rows if all(r["metrics"][role]["scored"] for role in ids) and r["persistence"]]
            per["persistence"] = dict(cases=len(pers), crps=mean([p["crps"] for p in pers]),
                                      on_cases_every_release_scored=dict(
                                          cases=len(common), persistence_crps=mean([r["persistence"]["crps"] for r in common]),
                                          **{f"{role}_crps": mean([r["metrics"][role]["metrics"]["crps"] for r in common])
                                             for role in ids}))
            ok &= per["spec_equal_to_wspdev"] and per["not_at_risk_equal"] and all(c["reproduced"] for c in cmpd.values())
            fam["paths"][path] = per
            if path == "replica":
                # wsp-dev's own aggregate CRPS (case mean) against the chain's, for the report
                fam["wspdev_case_mean_crps"] = {role: mean([c["metrics"]["crps"] for c in rr["cases"] if "crps" in c["metrics"]])
                                                for role in ids for rr in dev["releases"] if rr["release_id"] == ids[role]}
        report["fuels"][fuel] = fam
    report["reproduced"] = bool(ok)
    write_json(OUT / "flex" / f"chain_check{tag}.json", report)
    short = {f: {p: {role: (c["max_abs_diff_any_metric"], c["n_mismatches"]) for role, c in d["paths"][p]["comparison"].items()}
                 for p in d["paths"]} for f, d in report["fuels"].items()}
    print(json.dumps(dict(reproduced=report["reproduced"], max_abs_diff_and_mismatches=short), indent=1))
    return 0 if ok else 2


# ---------------------------------------------------------------------------------------------
# the generic path
# ---------------------------------------------------------------------------------------------
def cmd_score(args):
    binaries = use_binaries(args)
    if args.releases == "flex":
        rel = {f"{f}:{role}": r["id"] for f in ("methanol", "heptane") for role, r in releases_of(f).items()}
    else:
        rel = {r: r for r in args.releases.split(",")}
    rows_in = [json.loads(l) for l in Path(args.cases).read_text(encoding="utf-8").splitlines() if l.strip()]
    inputs = [ext_case_inputs(r) for r in rows_in]
    work = Path(args.out_dir).resolve()
    assert OUT.resolve() in work.parents or work == OUT.resolve(), "--out-dir must be under " + str(OUT)
    rows, nar, pre_ref = run_cases(inputs, rel, work, args.workers)
    # one spec per family of releases that answer the same fuel would be the evaluator's choice; here
    # one spec over the whole set, so that every release is scored on the same level set
    spec = score_rows(rows, rel)
    per_case = [dict(run_id=r["run_id"], session_id=r["session_id"], cutoff_s=r["cutoff_s"], input_hash=r["input_hash"],
                     persistence=r["persistence"], releases=r["metrics"]) for r in rows]
    write_json(work / "cases_scored.json", dict(cases=per_case, prefix_refusals=pre_ref, not_at_risk=nar))
    agg = dict(binaries=binaries, settings=SETTINGS, spec=spec, cases=len(rows), not_at_risk=nar,
               prefix_refusals=len(pre_ref), releases=rel, aggregates={role: aggregate(rows, role) for role in rel},
               persistence=dict(cases=sum(1 for r in rows if r["persistence"]),
                                crps=mean([r["persistence"]["crps"] for r in rows if r["persistence"]])))
    write_json(work / "aggregate.json", agg)
    print(json.dumps(agg, indent=1))
    return 0


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    sub = ap.add_subparsers(dest="cmd", required=True)
    for name, fn in (("flex-check", cmd_flex_check), ("score", cmd_score)):
        p = sub.add_parser(name)
        p.add_argument("--ignis", help="another ignis binary, relative to the repository")
        p.add_argument("--wsp-dev", help="another wsp-dev binary, relative to the repository")
        p.add_argument("--workers", type=int, default=8)
        if name == "flex-check":
            p.add_argument("--releases", help="fuel:role=release folder,... (default: the FLEX set of this file)")
            p.add_argument("--tag", help="suffix of the output files")
        if name == "score":
            p.add_argument("--cases", required=True)
            p.add_argument("--releases", default="flex")
            p.add_argument("--out-dir", required=True)
        p.set_defaults(fn=fn)
    a = ap.parse_args()
    return a.fn(a)


if __name__ == "__main__":
    sys.exit(main())
